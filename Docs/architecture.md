# Architecture

> Burrow v0.2.0 — Last updated 2026-03-23

This document describes the architecture of Burrow: how the pieces fit together,
what each file does, why certain decisions were made, and where the boundaries are.

---

## System overview

```
                    ┌─────────────────────────────────────────────┐
                    │                   User                      │
                    └──┬──────────┬──────────┬───────────────────┘
                       │          │          │
                  CLI (burrow)  Browser   Gemini client
                       │        (HTTP)    (TLS/Gemini)
                    ┌──▼──┐  ┌──▼────┐  ┌──▼──────────┐
                    │cli.rs│  │main.rs│  │gemini_listen │
                    │      │  │ Axum  │  │ (in main.rs) │
                    │Clap 4│  │ 0.8   │  │ tokio-rustls │
                    └──┬───┘  └──┬────┘  └──┬──────────┘
                       │         │          │
                    ┌──▼─────────▼──────────▼─────────────────┐
                    │            src/lib.rs                     │
                    │            └── config.rs                  │
                    │   ServerConfig { domain, aliases, port,   │
                    │     tls_cert, tls_key, gemini_port }      │
                    └────────────────────┬────────────────────┘
                                         │
                    ┌────────────────────▼────────────────────┐
                    │              burrows/                    │
                    │  ~bruno/           ~maya/    ~burrow/    │
                    │    .burrow          .burrow    manual/   │
                    │    about.txt        about.txt  concepts/ │
                    │    phlog/           phlog/     server/   │
                    │    gallery/                              │
                    │    bookmarks.gph                         │
                    │    rings/                                │
                    │    guestbook.gph                         │
                    │  .pings             ← federation data    │
                    └─────────────────────────────────────────┘
                              Filesystem = Database
```

Two binaries, one shared library, one data directory. Three protocols (HTTP,
HTTPS, Gemini). No database, no ORM, no message queue. Files in, pages out.

---

## Binaries

### `burrowd` (server) — `src/main.rs`

The HTTP/HTTPS/Gemini server. Async via Tokio. Reads content from `burrows/`
and renders it as HTML (or Gemtext for Gemini clients).

**Startup sequence:**

```
main()
  ├── tracing_subscriber::init()          ← structured logging (RUST_LOG)
  ├── config::ServerConfig::load()        ← reads burrow.conf (or defaults)
  ├── SearchIndex::build("burrows")       ← full-text index (BM25, in-memory)
  ├── AppState { config, domain, guestbook_limiter, started_at, search_index }
  ├── Router::new()
  │     ├── GET  /                        → home()
  │     ├── GET  /robots.txt              → robots_txt()
  │     ├── GET  /favicon.ico             → favicon_ico()
  │     ├── GET  /health                  → health()
  │     ├── GET  /stats                   → stats()
  │     ├── GET  /firehose                → firehose()
  │     ├── GET  /random                  → random_burrow()
  │     ├── GET  /discover                → discover()
  │     ├── GET  /search                  → search_handler()
  │     ├── GET  /search/index.json       → search_index_json()
  │     ├── GET  /rings                   → rings_page()
  │     ├── POST /ping                    → receive_ping()
  │     ├── GET  /{*path}                 → serve_burrow()
  │     └── POST /{*path}                 → post_guestbook()
  ├── spawn(gemini_listener)              ← if TLS + gemini_port configured
  ├── spawn(send_outgoing_pings)          ← scans posts for gph:// links
  ├── TraceLayer (tower-http)             ← access logging
  └── TcpListener::bind("0.0.0.0:PORT") + axum::serve()
```

**Request flow for `GET /~bruno/phlog/my-post`:**

```
serve_burrow()
  ├── Extract Host header → resolve domain (primary or alias)
  ├── Check virtual routes (feed.xml, atom.xml, feed, atom)
  ├── Check draft enforcement (_ or . in any path segment → 404)
  ├── Check depth limit (>8 segments → 404)
  ├── Read accent color from burrow's .burrow config
  ├── Construct fs_path: burrows/~bruno/phlog/my-post
  ├── tokio::fs::canonicalize(fs_path)
  │     ├── OK → verify starts_with(burrows_root)     ← path traversal guard
  │     └── Err → try .txt, then .gph extension       ← auto-resolve
  ├── Is directory? → list_directory() + directory_page()
  ├── Is gallery directory? → gallery_page() (grid layout)
  ├── Is guestbook.gph? → parse_guestbook() + guestbook_page()
  ├── Is bookmarks.gph? → parse_bookmarks() + bookmarks_page()
  ├── Is gallery item? → art_page() (monospace viewer)
  ├── Is binary? → serve with MIME type + Cache-Control
  └── Otherwise → load mentions + rings → text_page_with_mentions()
```

**State management:**

```rust
struct AppState {
    config: Arc<ServerConfig>,          // domain, aliases, TLS paths, gemini port
    domain: Arc<String>,                // primary domain (convenience)
    guestbook_limiter: Arc<Mutex<HashMap<IpAddr, Instant>>>,  // rate limiting
    started_at: Instant,                // for /stats uptime
    search_index: Arc<SearchIndex>,     // BM25 inverted index
}
```

All filesystem reads use `tokio::fs` (async, non-blocking). Host header is
extracted from each request to resolve the correct domain for aliases.

### `burrow` (CLI) — `src/cli.rs`

Content management tool. 20 subcommands across core, social, archival, and protocol.

**Command tree:**

```
burrow
  ├── init <name>                    Create a new ~name/ burrow
  ├── new "<title>"                  Create dated phlog post, open $EDITOR
  ├── edit <path>                    Open file in $EDITOR
  ├── ls [path]                      List burrow contents
  ├── status                         Show burrow stats
  ├── switch [name]                  List/switch active burrow
  ├── preview <path>                 Terminal gph preview (including drafts)
  ├── search <query> [--all]         Grep across burrow content
  ├── bookmark
  │     ├── add <url> -d "desc"      Add public bookmark
  │     ├── list                     List bookmarks
  │     └── remove <N>               Remove by number
  ├── ring
  │     ├── create <name> -d "desc"  Create a webring
  │     ├── add <slug> <member>      Add member
  │     ├── remove <slug> <member>   Remove member
  │     ├── show <slug>              Show members
  │     └── list                     List rings
  ├── guestbook
  │     ├── init                     Create guestbook.gph
  │     └── show                     Display entries
  ├── export [output.tar.gz]         Backup active burrow
  ├── push <remote>                  rsync push to remote server
  ├── pull <remote>                  rsync pull from remote server
  ├── timecapsule [year]             Generate yearly stats summary
  ├── open <gph://url>               Open gph:// URL
  ├── register                       Register gph:// protocol handler
  └── server
        └── init --domain --port     Generate burrow.conf
```

**Active burrow detection:**

```
find_active_burrow()
  ├── Count ~dirs in burrows/
  ├── If exactly 1 → use it
  └── If >1 → read burrows/.burrow-active
```

---

## Shared library — `src/lib.rs` + `src/config.rs`

```rust
pub struct ServerConfig {
    pub domain: String,          // default: "localhost"
    pub aliases: Vec<String>,    // default: empty
    pub port: u16,               // default: 7070
    pub tls_cert: Option<String>,
    pub tls_key: Option<String>,
    pub gemini_port: Option<u16>,
}
```

Both binaries import `burrow::config`. The config format is line-based:

```
domain = phlogosphere.net
port = 7070
aliases = burrow.bruno.be, myburrow.com
tls_cert = /path/to/cert.pem
tls_key = /path/to/key.pem
gemini_port = 1965
```

Parsed line-by-line with `strip_prefix()`. No TOML, no YAML, no serde for config.

Key methods:
- `resolve_domain(host)` — matches Host header against aliases, falls back to primary
- `is_known_host(host)` — returns true if host matches domain or any alias
- `has_tls()` — true if both cert and key are configured
- `has_gemini()` — true if gemini_port is set and TLS is configured

---

## Rendering — `src/render.rs`

All HTML (and Gemtext) generation lives here. No templates, no template engine.
String concatenation with format macros.

**Page types:**

| Function | Route | Layout |
|----------|-------|--------|
| `home_page()` | `/` | Topbar + sidebar + burrow list |
| `directory_page()` | `/~user/path/` | Topbar + sidebar + entry list (sort/pin) |
| `text_page()` | `/~user/file` | Reading view + progress bar + reading time |
| `text_page_with_mentions()` | `/~user/file` | Reading view + mentions + ring navigation |
| `guestbook_page()` | `/~user/guestbook` | Reading view + entries + form |
| `bookmarks_page()` | `/~user/bookmarks` | Bookmark list with descriptions |
| `gallery_page()` | `/~user/gallery/` | Grid of ASCII art previews |
| `art_page()` | `/~user/gallery/piece` | Monospace `<pre>` art viewer |
| `search_page()` | `/search` | Search box + ranked results |
| `firehose_page()` | `/firehose` | Chronological post stream + pagination |
| `discover_page()` | `/discover` | Latest, popular, random, rings, all burrows |
| `rings_list_page()` | `/rings` | All webrings with members |
| `not_found_page()` | any 404 | Centered message with existential undertones |
| `render_gph_to_gmi()` | Gemini | .gph → Gemtext conversion |
| `home_gmi()` | Gemini `/` | Gemtext home page |
| `directory_listing_gmi()` | Gemini dirs | Gemtext directory listing |

**Shared components:**

```
head(title, addr, domain, accent)  ← DOCTYPE, meta, CSS, topbar, feed autodiscovery
footer(domain)                     ← status bar + banner
sidebar(active, entries)           ← burrow nav + Explore links (Search, Discover, etc.)
build_crumbs(path, domain)         ← breadcrumb navigation
render_entries(entries)             ← directory/file listing grid
render_gph(content)                ← .gph markup → HTML
ring_nav_html(rings, burrow, path) ← ring navigation bars (← Previous · Ring · Next →)
```

**CSS design system** (embedded in `const CSS`):

```
Light:  --surface: #faf9f7   --text: #1a1a1a   --accent: var(--custom-accent, #1a8a6a)
Dark:   --surface: #161614   --text: #e8e6e1   --accent: var(--custom-accent, #3ab89a)

Fonts:  JetBrains Mono (UI, code, gallery)
        Literata (reading view prose)

Breakpoint: 700px (sidebar hidden on mobile)
Accent override: per-burrow via .burrow accent = #hexcolor
```

---

## Search engine — Veronica-NG

In-memory inverted index, built at startup by scanning all `burrows/~*/` content.

```
SearchIndex
  ├── docs: Vec<SearchDoc>            ← all indexed documents
  ├── term_index: HashMap<String, Vec<(usize, f64)>>  ← term → (doc_id, tf)
  ├── doc_count: usize
  └── avg_doc_len: f64

SearchDoc { path, title, author, content, doc_type, date, word_count }
```

**Ranking:** BM25 with parameters k1=1.2, b=0.75. Title matches get 3x boost.
Freshness boost: 90-day exponential decay curve. Deterministic — same query,
same results, for everyone.

**Operators:** `author:~name`, `type:kind`, `fresh:N` — parsed from query string,
applied as post-filters after BM25 scoring.

**Federation export:** `/search/index.json` — JSON array of all indexed documents
with path, title, author, date, word count. Read-only. Other servers can fetch
and merge into their own index.

---

## Special files

| File | Location | Purpose |
|------|----------|---------|
| `burrow.conf` | Project root | Server config (6 keys) |
| `.burrow` | Any `~user/` dir or subdir | Per-directory config (description, accent, title, sort, pin) |
| `.burrow-active` | `burrows/` | Tracks active burrow for CLI |
| `guestbook.gph` | Any `~user/` dir | Triggers guestbook rendering + POST |
| `bookmarks.gph` | Any `~user/` dir | Public bookmarks page |
| `*.ring` | `~user/rings/` | Webring definition (title, description, members) |
| `.pings` | `burrows/` | Federation ping storage (JSON, max 100) |

### Ring file format

```
title = Deep Web Craft
description = Writers who care about the web as a medium

/~bruno
/~maya
gph://tilde.town/~river
ring:~bruno/indie-web
```

Members are local paths, `gph://` remote URLs, or `ring:~owner/slug` for nested rings.
Nested rings are recursively resolved (members flattened, duplicates removed).

---

## Virtual routes

Routes that don't map directly to files on disk:

| Route | Handler | Output |
|-------|---------|--------|
| `/~user/feed.xml` or `/~user/feed` | `generate_feed()` | RSS 2.0 XML |
| `/~user/atom.xml` or `/~user/atom` | `generate_atom_feed()` | Atom 1.0 XML |
| `/search?q=...` | `search_handler()` | HTML search results |
| `/search/index.json` | `search_index_json()` | JSON federation index |
| `/discover` | `discover()` | HTML discovery page |
| `/firehose?page=N` | `firehose()` | HTML paginated stream |
| `/rings` | `rings_page()` | HTML ring listing |
| `/random` | `random_burrow()` | 302 redirect to random burrow |
| `/health` | `health()` | `{"status":"ok"}` JSON |
| `/stats` | `stats()` | `{"burrows":N,"files":N,"uptime_secs":N}` JSON |
| `POST /ping` | `receive_ping()` | Federation ping receiver |

Feeds are generated per-request by scanning `phlog/` directories. Search uses
the pre-built in-memory index. Stats are computed on-demand.

---

## Security model

```
                        Request
                           │
                    ┌──────▼──────┐
                    │ Draft check  │  _ or . prefix in any segment → 404
                    └──────┬──────┘
                           │
                    ┌──────▼──────┐
                    │ Depth check  │  > 8 segments → 404
                    └──────┬──────┘
                           │
                    ┌──────▼──────┐
                    │ Canonicalize │  tokio::fs::canonicalize()
                    │   path       │  resolves symlinks, ../, etc.
                    └──────┬──────┘
                           │
                    ┌──────▼──────┐
                    │ Prefix check │  canonical.starts_with(burrows_root)
                    │              │  rejects anything outside burrows/
                    └──────┬──────┘
                           │
                    ┌──────▼──────┐
                    │ Size check   │  text: 64 KB, binary: 2 MB
                    └──────┬──────┘
                           │
                    ┌──────▼──────┐
                    │ Escape       │  html_escape() for text
                    │ output       │  html_escape_attr() for href
                    │              │  xml_escape() for feeds
                    └──────┬──────┘
                           │
                        Response
```

**Guestbook protection:**
- Name: max 40 chars, Message: max 500 chars
- Entry cap: 200 per guestbook
- `---` replacement prevents format injection
- Rate limiting: 1 post per 30 seconds per IP address
- POST only accepted for files named exactly `guestbook.gph`

**Federation protection:**
- Max 100 stored pings per server
- Pings are stored as JSON, not executed

---

## Dependency tree

```
burrow v0.2.0
├── axum 0.8              HTTP framework (server)
├── axum-server 0.7       TLS support via rustls (server)
├── tokio 1               Async runtime (server)
├── tokio-rustls           TLS for Gemini listener (server)
├── rustls-pemfile         PEM certificate parsing (server)
├── tower-http 0.6        Access logging TraceLayer (server)
├── tracing                Structured logging (server)
├── tracing-subscriber     Log output formatting (server)
├── clap 4                CLI argument parsing (CLI)
├── chrono 0.4            Date formatting (both)
└── serde 1               Form deserialization (server, guestbook POST)
```

---

## Design principles

1. **Filesystem is the database.** No abstraction layer between content and storage.
   `cat` reads it, `vim` edits it, `ls` lists it. Always will.

2. **Two binaries, one library.** Server and CLI share config but nothing else.
   They can evolve independently.

3. **No JavaScript.** Zero. The scroll progress bar is pure CSS (`animation-timeline: scroll()`).
   One HTML form. The server renders complete pages. The browser's job is to display them.

4. **Inline CSS.** One `const CSS` string, no build step, no PostCSS, no Tailwind.
   Per-burrow accent color override via CSS custom properties.

5. **Plaintext is the source of truth.** `.txt` and `.gph` files are readable
   in any editor, on any OS, for the next 50 years. The server adds presentation;
   it never owns the content.

6. **Three protocols, one binary.** HTTP, HTTPS, and Gemini from the same process.
   Same content, different wire formats.

7. **Async I/O.** All filesystem reads use `tokio::fs`. No blocking the event loop.

8. **Human curation over algorithms.** Rings are manually curated. Bookmarks are
   manually chosen. Discovery is by exploration, not recommendation.
