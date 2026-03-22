# Todos

> Burrow v0.1.0 — Last updated 2026-03-22

---

## Done (v0.1.0)

- [x] Server binary (`burrowd`) — Axum HTTP, static file serving, directory listings
- [x] CLI binary (`burrow`) — init, new, ls, status, edit, server init
- [x] Shared config library (`lib.rs` + `config.rs`)
- [x] `.gph` markup rendering (headings, blockquotes, code, links, rules)
- [x] Home page with burrow listing
- [x] Directory browsing with sidebar navigation
- [x] Text reading view (Literata font, reading progress bar)
- [x] Path traversal protection (canonicalize + prefix check)
- [x] HTML escaping (text + attribute contexts)
- [x] Configurable domain (`burrow server init --domain`)
- [x] HTTPS gateway (reverse proxy ready)
- [x] Guestbook — `.gph` form, POST handler, entry cap, format injection prevention
- [x] RSS feed — `/~user/feed.xml` virtual route, autodiscovery `<link>`
- [x] Dark mode (CSS `prefers-color-scheme`)
- [x] Mobile-responsive layout (700px breakpoint)
- [x] 36 unit tests
- [x] Documentation: CLAUDE.md, CHANGELOG.md, architecture.md, burrowd-manual.md

---

## Server features

- [x] Draft visibility — files prefixed with `_` hidden from listings and HTTP
- [x] Content limits enforcement (64 KB max file, 256 files/dir, 8 levels depth)
- [x] Directory `.burrow` config: `title` support (sort/pin pending)
- [x] Server stats endpoint or CLI command
- [x] `tokio::fs` migration (async filesystem reads under load)
- [x] Static asset serving (favicon, robots.txt)
- [x] Rate limiting on guestbook POST
- [x] Access logging (structured, optional)

## CLI features

- [x] `burrow switch` — multi-user CLI (switch active burrow)
- [x] `burrow preview` — local draft preview before publishing
- [x] `burrow push` / `burrow pull` — remote sync via rsync/SSH
- [x] `burrow search` — local grep-based search with highlighting

## Rings (webrings)

- [x] `.ring` file format (title, description, members list)
- [x] Ring navigation on pages (← Previous · Ring Name · Next →)
- [x] Ring creation via CLI (`burrow ring create/add/remove/show/list`)
- [x] Ring listing on Discover page + dedicated `/rings` page
- [x] Cross-server ring support (`gph://` URLs as members)

## Veronica-NG (search)

- [x] Full-text index of public burrow content (in-memory inverted index, built on startup)
- [x] Search endpoint (`/search?q=...`)
- [x] BM25 ranking + freshness boost (90-day decay, title boost)
- [x] Search operators: `author:`, `fresh:`, `type:`
- [x] Search UI in the gateway (Veronica-NG page with search box + styled results)
- [x] Federation: `/search/index.json` export endpoint

## Discovery & social

- [x] Discover page (`/discover/`) — latest posts, random spotlight, all burrows
- [x] Firehose (`/firehose/`) — chronological stream of all new publications
- [x] Public bookmarks per user (`/~user/bookmarks`)
- [x] Bookmark counts as discover ranking signal (★ Most bookmarked section)
- [x] Random burrow feature on discover page
- [x] Burrow-to-burrow ping (← "Mentioned by" on posts)

## Feed / timeline

- [x] Cross-burrow chronological feed (all burrows on server)
- [x] Per-burrow Atom feed (alongside existing RSS)
- [x] Feed pagination

## Content & rendering

- [x] `.gph` rendering in directory listing descriptions (currently plaintext only)
- [x] Burrow theming — per-burrow accent color via `.burrow` config
- [x] ASCII art gallery (`/~user/gallery/`)
- [x] Reading time estimate (word count / 250 wpm, shown briefly)
- [x] Yearly `timecapsule.txt` generation (`burrow timecapsule [year]`)
- [x] Image/binary file serving (PNG, JPEG, GIF, SVG, WebP, PDF, audio, fonts, archives — 2 MB limit)

## Protocol

- [x] Gemini bridge (`gemini://` serving, `.gph` → `.gmi` conversion, TLS listener on configurable port)
- [x] TLS support (native rustls, manual cert via `tls_cert`/`tls_key` in burrow.conf)
- [ ] Custom domain support (paid tier)
- [x] `gph://` protocol handler (`burrow register` + `burrow open`)

## Infrastructure

- [x] Docker image / `docker-compose.yml`
- [x] Self-hosting one-liner install script
- [x] Systemd service file
- [x] Backup / export tool
- [x] Health check endpoint

---

*Features derived from [burrow-concept.md](burrow-concept.md). Priority: server features and CLI first, then rings + search, then discovery + social.*
