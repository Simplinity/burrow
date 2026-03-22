# Architecture

> Burrow v0.1.0 — Last updated 2026-03-22

This document describes the architecture of Burrow: how the pieces fit together,
what each file does, why certain decisions were made, and where the boundaries are.

---

## System overview

```
                    ┌─────────────────────────────────────────────┐
                    │                   User                      │
                    └──────────┬──────────────────┬───────────────┘
                               │                  │
                          CLI (burrow)        Browser (HTTP)
                               │                  │
                    ┌──────────▼──────┐  ┌────────▼────────────┐
                    │   src/cli.rs    │  │   src/main.rs        │
                    │                 │  │   (burrowd)          │
                    │   Clap 4        │  │   Axum 0.8 + Tokio   │
                    │   derive-based  │  │                      │
                    └────────┬────────┘  └────────┬─────────────┘
                             │                    │
                    ┌────────▼────────────────────▼─────────────┐
                    │            src/lib.rs                      │
                    │            └── config.rs                   │
                    │            ServerConfig { domain, port }   │
                    └────────────────────┬──────────────────────┘
                                         │
                    ┌────────────────────▼──────────────────────┐
                    │              burrows/                      │
                    │   ~bruno/                ~maya/            │
                    │     .burrow               .burrow          │
                    │     about.txt             about.txt        │
                    │     phlog/                phlog/            │
                    │     guestbook.gph                          │
                    └───────────────────────────────────────────┘
                              Filesystem = Database
```

Two binaries, one shared library, one data directory. No database, no ORM,
no message queue, no Redis, no Docker. Files in, HTML out.

---

## Binaries

### `burrowd` (server) — `src/main.rs`

The HTTP server. Stateless, single-threaded async via Tokio. Reads content
from `burrows/` and renders it as HTML.

**Startup sequence:**

```
main()
  ├── config::ServerConfig::load()      ← reads burrow.conf (or defaults)
  ├── Arc::new(domain)                  ← shared state for all handlers
  ├── Router::new()
  │     ├── GET  /                      → home()
  │     ├── GET  /{*path}               → serve_burrow()
  │     └── POST /{*path}               → post_guestbook()
  └── TcpListener::bind() + axum::serve()
```

**Request flow for `GET /~bruno/phlog/my-post`:**

```
serve_burrow()
  ├── Check virtual routes (feed.xml)   ← short-circuit if RSS
  ├── Construct fs_path: burrows/~bruno/phlog/my-post
  ├── fs::canonicalize(fs_path)
  │     ├── OK → verify starts_with(burrows_root)   ← path traversal guard
  │     └── Err → try .txt extension, then .gph     ← auto-resolve
  ├── Is directory? → directory_page()
  ├── Is guestbook.gph? → guestbook_page()
  └── Otherwise → text_page()
```

**State management:**
- `State<Arc<String>>` — the configured domain, injected into every handler
- No session state, no cookies, no auth tokens
- All reads are synchronous `std::fs` calls (blocking in async context — acceptable
  at current scale, would need `tokio::fs` or `spawn_blocking` under load)

### `burrow` (CLI) — `src/cli.rs`

Content management tool. Creates burrows, writes posts, manages guestbooks,
configures the server.

**Command tree:**

```
burrow
  ├── init <name>              Create a new ~name/ burrow
  ├── new "<title>"            Create dated phlog post, open $EDITOR
  ├── ls [path]                List burrow contents
  ├── status                   Show burrow stats
  ├── edit <path>              Open file in $EDITOR
  ├── guestbook
  │     ├── init               Create guestbook.gph
  │     └── show               Display entries in terminal
  └── server
        └── init --domain      Generate burrow.conf
```

**Active burrow detection:**

```
find_active_burrow()
  ├── Count ~dirs in burrows/
  ├── If exactly 1 → use it
  └── If >1 → read burrows/.burrow-active
```

**Config awareness:** The CLI reads `burrow.conf` (via `ServerConfig::load_from()`)
to display correct URLs in output. If domain is `localhost`, shows `http://localhost:PORT`.
Otherwise, shows `https://DOMAIN`.

---

## Shared library — `src/lib.rs` + `src/config.rs`

```rust
// lib.rs
pub mod config;

// config.rs
pub struct ServerConfig {
    pub domain: String,    // default: "localhost"
    pub port: u16,         // default: 7070
}
```

Both binaries import `burrow::config`. The config format is intentionally trivial:

```
# burrow.conf
domain = phlogosphere.net
port = 7070
```

Parsed line-by-line with `strip_prefix()`. No TOML, no YAML, no serde for config.
Two keys. That's it.

---

## Rendering — `src/render.rs`

All HTML generation lives here. No templates, no template engine. String concatenation
with format macros.

**Page types:**

| Function | Route | Layout |
|----------|-------|--------|
| `home_page()` | `/` | Topbar + sidebar + entry list |
| `directory_page()` | `/~user/path/` | Topbar + sidebar + entry list |
| `text_page()` | `/~user/file` | Topbar + reading view (no sidebar) |
| `guestbook_page()` | `/~user/guestbook` | Topbar + reading view + form |
| `not_found_page()` | any 404 | Topbar + centered message |

**Shared components:**

```
head(title, addr, domain)     ← DOCTYPE, meta, CSS, topbar with gph:// address bar
footer(domain)                ← status bar + banner
sidebar(active, entries)      ← burrow nav + Discover links
build_crumbs(path, domain)    ← breadcrumb navigation
render_entries(entries)        ← directory/file listing grid
render_gph(content)            ← .gph markup → HTML
```

**CSS design system** (embedded in `const CSS`):

```
Light:  --surface: #faf9f7   --text: #1a1a1a   --accent: #1a8a6a
Dark:   --surface: #161614   --text: #e8e6e1   --accent: #3ab89a

Fonts:  JetBrains Mono (UI, code)
        Literata (reading view prose)

Breakpoint: 700px (sidebar hidden on mobile)
```

**Why no template engine?** The pages are structurally simple. A template engine
would add a dependency, a build step, and a mental context switch for ~6 page types
that rarely change. Raw strings are greppable, debuggable, and have zero overhead.

---

## Content format — `.gph`

A minimal markup language parsed by `render_gph()`:

```
Line prefix        Renders as
─────────────────────────────────
# Text             <h1>
> Text             <blockquote>
---                <hr>
→ URL              <a href> (external link)
/~user/path  desc  <a href> (internal link, 3-space separator)
  indented         <pre> (code block, 2-space indent)
(anything else)    <p>
(empty line)       (skipped)
```

The parser is a single-pass line scanner. Code blocks are stateful (tracks `in_code`
flag). Everything else is stateless line-by-line.

---

## Special files

| File | Location | Purpose |
|------|----------|---------|
| `burrow.conf` | Project root | Server config (domain, port) |
| `.burrow` | Each `~user/` dir | Burrow metadata (`description = ...`) |
| `.burrow-active` | `burrows/` | Tracks active burrow for CLI |
| `guestbook.gph` | Any `~user/` dir | Triggers guestbook rendering + POST |

### Guestbook storage format

```
--- Name · 2026-03-22 14:30
Message text here.

--- Another Name · 2026-03-22 15:00
Another message.
```

Parsed by `parse_guestbook()`: splits on `--- ` prefix lines, extracts name/date
from ` · ` separator. Format injection prevented by replacing `---` in user input
with `—`.

---

## Virtual routes

Some routes don't map to files on disk:

| Route | Handler | Output |
|-------|---------|--------|
| `/~user/feed.xml` | `generate_feed()` | RSS 2.0 XML |
| `/~user/feed` | same | RSS 2.0 XML |

The feed is generated on every request by scanning `phlog/` for `.txt` files,
extracting dates from filenames (`YYYY-MM-DD-slug.txt`), and building XML.
No caching. At current scale, this is fine — a directory listing + a few file reads.

---

## Security model

```
                        Request
                           │
                    ┌──────▼──────┐
                    │ Canonicalize │  fs::canonicalize()
                    │   path       │  resolves symlinks, ../, etc.
                    └──────┬──────┘
                           │
                    ┌──────▼──────┐
                    │ Prefix check │  canonical.starts_with(burrows_root)
                    │              │  rejects anything outside burrows/
                    └──────┬──────┘
                           │
                    ┌──────▼──────┐
                    │ Read file    │  size check (1 MB max)
                    └──────┬──────┘
                           │
                    ┌──────▼──────┐
                    │ Escape       │  html_escape() for text
                    │ output       │  html_escape_attr() for href
                    │              │  xml_escape() for RSS
                    └──────┬──────┘
                           │
                        Response
```

**Two escape functions:**
- `html_escape(s)` — replaces `& < >` (for text content)
- `html_escape_attr(s)` — replaces `& < > " '` (for attribute values like `href`)

**Guestbook input validation:**
- Name: max 40 chars
- Message: max 500 chars
- Entry cap: 200 per guestbook
- `---` replacement prevents format injection
- Empty name or message → redirect (no entry created)
- POST only accepted for files named exactly `guestbook.gph`
- Path traversal check on POST path

---

## Data flow

### Writing a post (CLI → filesystem → server)

```
$ burrow new "My post"
    │
    ├── Generate filename: 2026-03-22-my-post.txt
    ├── Write template: "# My post\n\n"
    ├── Open $EDITOR
    ├── On save: file exists in burrows/~user/phlog/
    └── On empty: delete file (cleanup)

Browser → GET /~user/phlog/2026-03-22-my-post
    │
    ├── serve_burrow() resolves to .txt file
    ├── read_file_checked() reads content
    ├── render_gph() parses markup
    └── text_page() wraps in reading view
```

### Signing a guestbook (browser → server → filesystem)

```
Browser → POST /~user/guestbook (form data: name, message)
    │
    ├── Resolve to guestbook.gph
    ├── Validate + truncate input
    ├── Replace "---" with "—"
    ├── Count existing entries (cap: 200)
    ├── Append: "--- Name · Date\nMessage\n"
    └── 303 Redirect → GET /~user/guestbook
```

---

## Dependency tree

```
burrow v0.1.0
├── axum 0.8        HTTP framework (server only)
├── tokio 1         Async runtime (server only)
├── clap 4          CLI argument parsing (CLI only)
├── chrono 0.4      Date formatting (both binaries)
└── serde 1         Form deserialization (server only, for guestbook POST)
```

Five direct dependencies. No ORM, no database driver, no logging framework,
no error handling library.

---

## What's not here (and why)

| Missing thing | Why it's missing |
|---------------|-----------------|
| Database | Filesystem is the database. `ls` is your query language. |
| Authentication | Content is public. Authors use the CLI + filesystem access. |
| Caching | Pages are small, reads are fast. Premature optimization. |
| Logging framework | `println!` at startup. Reverse proxy handles access logs. |
| Error handling lib | `unwrap()` on startup-critical paths. Graceful fallbacks elsewhere. |
| `tokio::fs` | Blocking I/O in async handlers. Fine for single-digit concurrent users. |
| Template engine | 6 page types. Raw format strings are simpler. |
| WebSocket | No real-time features. Everything is request-response. |
| JavaScript | One scroll progress bar. One HTML form. That's the total client-side code. |

---

## File map

```
src/
  lib.rs          1 line     Re-exports config module
  config.rs       56 lines   ServerConfig: load, save, defaults
  main.rs         440 lines  Server: routes, handlers, guestbook, RSS, types
  render.rs       290 lines  HTML generation: pages, components, .gph parser, CSS
  cli.rs          530 lines  CLI: all commands, helpers, active burrow detection
  tests.rs        316 lines  36 unit tests: render, escaping, file helpers

Docs/
  architecture.md            This file
  burrowd-manual.md          Server manual (man page style)
  CHANGELOG.md               Version history
  burrow-concept.md          Product vision and roadmap

burrows/                     Content root (filesystem-as-database)
  ~bruno/                    Sample burrow
  ~maya/                     Sample burrow

CLAUDE.md                    Project quick reference
Cargo.toml                   Rust project config, two [[bin]] sections
```

---

## Design principles

1. **Filesystem is the database.** No abstraction layer between content and storage.
   `cat` reads it, `vim` edits it, `ls` lists it. Always will.

2. **Two binaries, one library.** Server and CLI share config but nothing else.
   They can evolve independently.

3. **No JavaScript.** One scroll handler. One HTML form. The server renders
   complete pages. The browser's job is to display them.

4. **Inline CSS.** One `const CSS` string, no build step, no PostCSS, no Tailwind.
   The entire design system fits in 55 lines.

5. **Plaintext is the source of truth.** `.txt` and `.gph` files are readable
   in any editor, on any OS, for the next 50 years. The server adds presentation;
   it never owns the content.
