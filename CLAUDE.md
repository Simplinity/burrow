# Burrow

The internet, minus the parts that made you hate the internet.

A Gopher-inspired plaintext phlog server and CLI, built in Rust.

## Quick reference

| | |
|---|---|
| **Language** | Rust (edition 2021) |
| **Framework** | Axum 0.8, Tokio |
| **CLI parser** | Clap 4 (derive) |
| **Build** | `cargo build` |
| **Test** | `cargo test` |
| **Lint** | `cargo clippy` |
| **Binaries** | `burrowd` (server), `burrow` (CLI) |

## Project structure

```
src/
  main.rs      Server binary (burrowd) — routes, handlers, guestbook POST
  cli.rs       CLI binary (burrow) — init, new, ls, status, edit, guestbook, server
  lib.rs       Shared library — re-exports config module
  config.rs    Server configuration (burrow.conf parsing/writing)
  render.rs    HTML rendering — all pages, gph markup, CSS
  tests.rs     Unit tests (36) — render, escaping, XSS, file helpers

burrows/       Content root — each ~user/ is a burrow
  ~bruno/      Sample burrow
  ~maya/       Sample burrow

Docs/          Project documentation
```

## Binaries

- **`burrowd`** — HTTP server. Reads `burrow.conf`, serves `burrows/` directory on configured port.
- **`burrow`** — CLI tool. Create burrows, write posts, manage guestbooks, configure server.

## Key conventions

- Filesystem-as-database: no DB, content lives in `burrows/~user/` directories
- `.burrow` file in each burrow root holds `description = ...`
- `.burrow-active` in `burrows/` tracks which burrow the CLI operates on
- `_` prefix hides files from directory listings (draft convention)
- `.gph` files use custom markup format (headings, quotes, code, links)
- `guestbook.gph` is a special file: server renders it with a form + POST handler
- `/~user/feed.xml` is a virtual route — RSS feed generated from `phlog/` directory

## Security model

- Path traversal protection: `fs::canonicalize()` + `starts_with()` on all file access
- Two-level HTML escaping: `html_escape()` for text, `html_escape_attr()` for href attributes
- XML escaping for RSS output
- Guestbook input: truncation limits, `---` format injection prevention, 200 entry cap
- 1 MB max file size for served content

## Documentation

- [Docs/burrowd-manual.md](Docs/burrowd-manual.md) — Server manual (man page style)
- [Docs/CHANGELOG.md](Docs/CHANGELOG.md) — Version history
- [Docs/architecture.md](Docs/architecture.md) — System architecture and design
- [Docs/burrow-concept.md](Docs/burrow-concept.md) — Product vision and roadmap
- [Docs/todos.md](Docs/todos.md) — Feature backlog and progress
