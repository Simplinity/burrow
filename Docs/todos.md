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
- [ ] `tokio::fs` migration (async filesystem reads under load)
- [x] Static asset serving (favicon, robots.txt)
- [x] Rate limiting on guestbook POST
- [x] Access logging (structured, optional)

## CLI features

- [x] `burrow switch` — multi-user CLI (switch active burrow)
- [x] `burrow preview` — local draft preview before publishing
- [ ] `burrow push` / `burrow pull` — remote sync (requires auth)
- [ ] `burrow search` — CLI search via Veronica-NG

## Rings (webrings)

- [ ] `.ring` file format (title, description, members list)
- [ ] Ring navigation on pages (← Previous · Ring Name · Next →)
- [ ] Ring creation via CLI (`burrow ring create`)
- [ ] Ring listing on Discover page
- [ ] Cross-server ring support

## Veronica-NG (search)

- [ ] Full-text index of public burrow content
- [ ] Search endpoint (`/search?q=...`)
- [ ] BM25 ranking + freshness boost
- [ ] Search operators: `author:`, `server:`, `ring:`, `fresh:`, `type:`
- [ ] Search UI in the gateway
- [ ] Federation: voluntary index submission to relay nodes

## Discovery & social

- [ ] Discover page (`/discover/`) — trending, most-bookmarked, random burrow
- [x] Firehose (`/firehose/`) — chronological stream of all new publications
- [ ] Public bookmarks per user (`/~user/bookmarks/`)
- [ ] Bookmark counts as search ranking signal
- [x] Random burrow feature on discover page
- [ ] Burrow-to-burrow ping (cross-reference notifications)

## Feed / timeline

- [x] Cross-burrow chronological feed (all burrows on server)
- [x] Per-burrow Atom feed (alongside existing RSS)
- [x] Feed pagination

## Content & rendering

- [x] `.gph` rendering in directory listing descriptions (currently plaintext only)
- [x] Burrow theming — per-burrow accent color via `.burrow` config
- [ ] ASCII art gallery (`/gallery/`)
- [x] Reading time estimate (word count / 250 wpm, shown briefly)
- [ ] Yearly `timecapsule.txt` generation
- [ ] Image/binary file serving (paid tier)

## Protocol

- [ ] Gemini bridge (`gemini://` serving, `.gph` → `.gmi` conversion)
- [ ] TLS support (native or Let's Encrypt auto-config)
- [ ] Custom domain support (paid tier)
- [ ] `gph://` protocol handler registration

## Infrastructure

- [x] Docker image / `docker-compose.yml`
- [ ] Self-hosting one-liner install script
- [x] Systemd service file
- [x] Backup / export tool
- [x] Health check endpoint

---

*Features derived from [burrow-concept.md](burrow-concept.md). Priority: server features and CLI first, then rings + search, then discovery + social.*
