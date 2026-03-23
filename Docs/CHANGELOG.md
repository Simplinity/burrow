# Changelog

All notable changes to Burrow are documented in this file.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.9.1] — 2026-03-23

The "everything you need, nothing you don't" release. 26 features. Zero JavaScript. 42 tests.

### Added

#### Server (`burrowd`)
- **`@today` date stamps** — code-block aware inline date expansion (YYYY-MM-DD)
- **Series navigation** — `part-01`, `part-02` pattern with "Part X of Y" and prev/next links
- **Slow reading mode** — `?slow=1` URL parameter (21px, 580px, extra whitespace)
- **Seasonal accent colors** — spring green, summer gold, autumn brown, winter blue
- **"Inspired by" link convention** — `← /~user/path` as first line renders as citation
- **Guest author convention** — `guest-~name-title.txt` renders with "Guest post by ~name"
- **Burrow anniversary** — "Est. YYYY" badge from earliest post date
- **Neighbors list** — ring-derived neighbors shown on burrow root pages
- **Anonymous reader count** — `/~user/stats` shows monthly page loads (AtomicU64 per burrow)
- **Canonical gph:// URL** — `<link rel="canonical" href="gph://...">` in HTML head
- **Last-Modified date** — file modification date shown in meta line on text pages
- **Server directory** (`/servers`) — curated list of known Burrow servers from `servers.conf`
- **ETag caching** — conditional GET returns 304 Not Modified, based on file mtime
- **Content-Security-Policy header** — strict CSP on all responses (no inline scripts, no external resources)
- **Optional gzip/Brotli compression** — `compression = true` in `burrow.conf`
- **SIGHUP hot-reload** — reload config without restart (Unix convention)
- **`.well-known/` support** — RFC 8615 well-known URIs served from `burrows/.well-known/`
- **OPML export** — `/~user/subscriptions.opml` from bookmarks
- **Digital testament** — `will.txt` generated on `burrow init` (instructions for burrow end-of-life)
- **Page load statistics** — persisted to `burrows/.stats`, survives restarts
- **Zero JavaScript CSS progress bar** — `animation-timeline: scroll()`, no JS fallback

#### CLI (`burrow`)
- **`burrow colophon`** — generate publishable colophon.txt (stats, rings, metadata)
- **`burrow lint`** — validate .gph files for common errors (broken links, line length, size)
- **`burrow import`** — Markdown to .gph conversion (one-way migration)
- **`burrow export-static`** — generate complete static HTML site for any host
- **`burrow changelog`** — generate changelog.txt from file modification times
- **`burrow read-later`** / **`burrow reading-list`** — private reading list (`_reading-list.gph`)
- **Writing streaks** — consecutive publishing days shown in `burrow status` (private, local only)

### Changed
- Config options: 6 → 7 (server), added `compression`
- Tests: 38 → 42

---

## [0.3.0] — 2026-03-23

The "polish and soul" release. Zero JavaScript. Seasonal colors. Your words deserve better defaults.

### Added

#### Server (`burrowd`)
- **Zero JavaScript** — scroll progress bar replaced with CSS `animation-timeline: scroll()`. The probation ended. The handler was replaced by a stylesheet.
- **`@today` date stamps** — write `@today` in any `.gph`/`.txt` file, rendered as current date (YYYY-MM-DD)
- **Series navigation** — files named `part-01.txt`, `part-02.txt` etc. get automatic "Part X of Y" with ← → navigation
- **Slow reading mode** — `?slow=1` URL parameter for 21px/580px comfortable reading view
- **Seasonal accent colors** — spring green, summer gold, autumn brown, winter blue. Four colors per year. No config.
- **Custom domains** — `aliases` config key, Host header resolution for multi-domain serving
- **Production hosting guide** — port 80, setcap, HTTPS with certbot, systemd, Docker

#### CLI (`burrow`)
- **`burrow colophon`** — generates publishable colophon.txt (files, words, posts, dates, rings, gallery)
- **`burrow read-later`** — save links to private `_reading-list.gph` (invisible to HTTP)
- **`burrow reading-list`** — show your private reading list

#### Documentation
- Complete audit: 18 undocumented features added to server manual
- Architecture.md fully rewritten (routes, state, dependencies, security model)
- 4 new `~burrow/server/` articles (binary files, operations, custom domains, writing extras)
- All CLI commands documented in both server and client manuals
- `ideas-for-burrow.md` — 50 feature ideas with manifesto rationale

### Changed
- Default accent color: seasonal instead of static teal
- All `std::fs` in server → `tokio::fs` (async, non-blocking)
- Server binds to `0.0.0.0` (all interfaces) with LAN IP in startup banner
- Tests: 36 → 38 (added `@today` expansion + series number extraction)

### Fixed
- Bind address documentation (was `127.0.0.1`, actual is `0.0.0.0`)
- LIMITS table (was 2+2 config options, actual is 6+5)
- Directory cap 256 removed from docs (not enforced in code)
- Merge conflict resolution in main.rs from upstream branch
- Pin config: was documented as plural, actual is single file
- Federation ping format: was form-encoded in docs, actual is JSON

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

## Project info

| | |
|---|---|
| **Language** | Rust (edition 2021) |
| **Framework** | Axum 0.8, Tokio |
| **CLI** | Clap 4 (derive) |
| **Binaries** | `burrowd` (server), `burrow` (CLI) |
| **Protocols** | HTTP, HTTPS (rustls), Gemini |
| **JavaScript** | 0 |
| **License** | TBD |
| **Repository** | [github.com/Simplinity/burrow](https://github.com/Simplinity/burrow) |
| **Tests** | 42 unit tests |
