# Changelog

All notable changes to Burrow are documented in this file.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.2.0] — 2026-03-23

The "we built the whole thing" release. 56 features. Two days. One hole.

### Added

#### Server (`burrowd`)
- **Draft enforcement** — `_` prefixed files/directories return 404 on GET and POST
- **Content limits** — 64 KB max file size, 256 entries/directory, 8 levels max depth
- **Burrow theming** — `accent = #hexcolor` in `.burrow`, injected as CSS custom property
- **Directory config** — `title` support in `.burrow` for custom directory names
- **Static assets** — favicon and robots.txt routes
- **Rate limiting** — guestbook POST rate limiting (in-memory)
- **Access logging** — structured, optional request logging
- **Server stats** — stats endpoint or CLI command
- **`tokio::fs` migration** — async filesystem reads for better performance under load
- **Health check** — `GET /health` endpoint
- **LAN binding** — configurable bind address in `burrow.conf`
- **Binary file serving** — PNG, JPEG, GIF, SVG, WebP, PDF, audio, fonts, archives (2 MB limit)

#### Search — Veronica-NG
- **Full-text search** — in-memory inverted index, built on startup
- **Search endpoint** — `GET /search?q=...` with styled results page
- **BM25 ranking** — with freshness boost (90-day decay) and title boost
- **Search operators** — `author:`, `fresh:`, `type:`
- **Federation** — `/search/index.json` export for cross-server index sharing

#### Discovery & Social
- **Discover page** (`/discover/`) — latest posts, random spotlight, all burrows
- **Firehose** (`/firehose/`) — chronological stream of all new publications
- **Public bookmarks** — per-user bookmarks at `/~user/bookmarks` (`.gph` format)
- **Bookmark ranking** — ★ Most bookmarked section on discover page
- **Random burrow** — spotlight feature on discover page
- **Mentions** — burrow-to-burrow ping, "Mentioned by" on posts

#### Rings (Webrings)
- **`.ring` file format** — title, description, members list
- **Ring navigation** — ← Previous · Ring Name · Next → on pages
- **Ring CLI** — `burrow ring create/add/remove/show/list`
- **Ring directory** — dedicated `/rings` page + listing on discover
- **Cross-server rings** — `gph://` URLs as members
- **Nested rings** — rings within rings
- **Federation ping** — cross-server ring membership

#### Feeds
- **Cross-burrow feed** — server-wide chronological feed (all burrows)
- **Atom feed** — per-burrow Atom feed alongside existing RSS
- **Feed pagination** — paginated feed endpoints

#### Content & Rendering
- **`.gph` in listings** — rendered descriptions in directory listings
- **ASCII art gallery** — `/~user/gallery/` with dedicated art page renderer
- **Reading time** — word count / 250 wpm estimate on text pages
- **Time capsule** — yearly `timecapsule.txt` generation via `burrow timecapsule [year]`

#### CLI (`burrow`)
- **`burrow switch`** — list all burrows with `←` marker, switch with `burrow switch <name>`
- **`burrow preview`** — local draft preview before publishing
- **`burrow push` / `burrow pull`** — remote sync via rsync/SSH
- **`burrow search`** — local grep-based search with highlighting
- **`burrow ring`** — full ring management (create, add, remove, show, list)
- **`burrow timecapsule`** — generate yearly time capsule
- **`burrow register`** — register `gph://` protocol handler
- **`burrow open`** — open `gph://` URLs
- **`burrow backup`** — export/backup tool

#### Protocol
- **Gemini bridge** — `gemini://` serving with `.gph` → `.gmi` conversion, TLS listener
- **TLS support** — native rustls, manual cert via `tls_cert`/`tls_key` in `burrow.conf`
- **`gph://` handler** — protocol registration and opener

#### Infrastructure
- **Dockerfile** + `docker-compose.yml`
- **`install.sh`** — self-hosting one-liner install script
- **`burrowd.service`** — systemd service file
- **Backup tool** — `burrow backup` for export

#### Documentation
- `Docs/architecture.md` — system architecture with ASCII diagrams
- `Docs/burrow-client-manual.md` — client/CLI manual
- `Docs/ideas-for-burrow.md` — 50 feature ideas document
- `Docs/burrowd-manual.md` — massively expanded: theming, drafts, limits, CLI reference, production guide
- `CLAUDE.md` — project quick reference
- `burrows/~burrow/` — self-hosted documentation burrow (concepts, server guides)

### Changed
- `MAX_FILE_SIZE` reduced from 1 MB to 64 KB for text content
- File size error messages display in KB
- Generic `.burrow` config parser replaces single-field reader
- All HTML responses inject per-burrow accent color
- Directory listings capped at 256 entries
- Repository set to public on GitHub

### Security
- Draft paths blocked at HTTP level (both GET and POST)
- Path depth enforcement (max 8 levels)
- Directory entry cap prevents memory abuse
- Accent color values escaped via `html_escape_attr()`
- Rate limiting on guestbook POST

---

## [0.1.0] — 2026-03-22

First public release. The hole is open.

### Added

#### Server (`burrowd`)
- Filesystem-based content server over HTTP with Axum 0.8
- Serves `burrows/~username/` directories as individual user spaces
- `.gph` markup format: headings, blockquotes, code blocks, external/internal links, horizontal rules
- Reading view with Literata serif font, JetBrains Mono for code, scroll progress bar
- Directory listings with file sizes, descriptions, and sorted dirs-first layout
- Sidebar navigation with burrow list and Discover section
- Automatic `.txt` and `.gph` extension resolution (request `/about`, get `about.txt`)
- Dark mode and light mode via `prefers-color-scheme`
- Responsive layout: sidebar hidden on mobile (<700px)
- 404 page with personality
- Configurable domain via `burrow.conf` (generated by `burrow server init`)
- Status bar and banner footer on all pages

#### Guestbook
- `guestbook.gph` — visitors can sign a guestbook via HTML form
- `POST /~user/guestbook` appends entries to the file
- Input limits: 40 char name, 500 char message, 200 entries max
- Format injection protection: `---` in user input replaced with `—`
- Post-Redirect-Get pattern prevents double submissions

#### RSS Feed
- Auto-generated RSS 2.0 feed at `/~user/feed.xml` (also `/~user/feed`)
- Pulls all `.txt` posts from `phlog/` directory, sorted newest first
- Content preview in `<description>` from first 5 lines
- `<link rel="alternate">` autodiscovery in all HTML pages
- Correct `application/rss+xml` content type
- Uses configured domain for feed URLs

#### CLI (`burrow`)
- `burrow init <name>` — create a new burrow with `about.txt` and `phlog/`
- `burrow new "<title>"` — create a dated phlog post, open in `$EDITOR`, discard if empty
- `burrow ls [path]` — list burrow contents with file sizes and descriptions
- `burrow status` — show burrow stats: file count, size, storage usage percentage
- `burrow edit <path>` — open a file in `$EDITOR` (auto-resolves `.txt` extension)
- `burrow guestbook init` — create a guestbook for the active burrow
- `burrow guestbook show` — display guestbook entries in terminal
- `burrow server init --domain <domain> [--port <port>]` — generate `burrow.conf`
- Active burrow auto-detection (single burrow) or `.burrow-active` file (multiple)
- Domain validation: rejects protocol prefixes, spaces, empty strings
- CLI URLs read from `burrow.conf` — shows `https://domain` or `http://localhost:port`

#### Shared Library
- `src/lib.rs` with `config` module shared between `burrowd` and `burrow` CLI
- `ServerConfig::load()` and `ServerConfig::load_from(path)` for flexible config loading
- `ServerConfig::save(path)` for consistent config file generation

#### Security
- Path traversal protection via `fs::canonicalize()` + `starts_with()` prefix check
- HTML escaping for all rendered text content (`html_escape`)
- Attribute-safe escaping for `href` values (`html_escape_attr` — also escapes `"` and `'`)
- XML escaping for RSS feed output
- 1 MB file size limit on served content
- Hidden files (`.` and `_` prefixed) excluded from directory listings

#### Documentation
- `Docs/burrowd-manual.md` — comprehensive server manual in tongue-in-cheek man page style
- `Docs/burrow-concept.md` — product vision and roadmap

### Fixed
- Pluralization: "1 entry" / "2 entries", "1 burrow" / "2 burrows", "1 item" / "2 items"
- Clippy warnings: unnecessary `let` binding, redundant `format!()` on string literal

---

## Backlog

| Feature | Status |
|---------|--------|
| Custom domain support (paid tier) | Planned |
| Directory sort/pin in `.burrow` config | Partial |

---

## Project info

| | |
|---|---|
| **Language** | Rust (edition 2021) |
| **Framework** | Axum 0.8, Tokio |
| **CLI** | Clap 4 (derive) |
| **Binaries** | `burrowd` (server), `burrow` (CLI) |
| **License** | TBD |
| **Repository** | [github.com/Simplinity/burrow](https://github.com/Simplinity/burrow) |
| **Tests** | 36+ unit tests |
