# burrow(1) — The Client That Thinks Your Attention Belongs to You

```
         ___
        /   \      burrow client v0.1.0
       | o o |     "Read things. Write things.
        \ _ /      Bookmark the good things."
        /| |\
       / | | \     No notifications.
      *  | |  *    No badges. No bell icon.
```

---

## THE BIG PICTURE

Before we talk about keyboard shortcuts and CLI flags, let's talk about why this exists.

The average web page in 2026 weighs 2.8 MB. It loads 47 trackers, 12 ad networks, and a consent banner more complex than the content you came for. Gopher remembered a better way. In 1991, the University of Minnesota built a protocol so simple it fit on a napkin. Just directories and text. You asked for something, you got it.

Burrow is not a time machine. We're taking the *philosophy* of Gopher — text-first, zero-bloat, human-speed, user-owned — and building it for people who are tired of the modern web but don't want to give up modern UX.

**Identity is a path.** Your address is `gph://server/~you`. The tilde is a direct homage to Unix home directories. Your burrow is not an account in a database — it's a directory on a server. Migration means copying files. Like moving apartments.

**Three protocols, one binary.** HTTPS for the browser crowd (the gateway to the normal world), Gemini for the small-web purists (automatic `.gph` → `.gmi` conversion), and `gph://` for the native clients (coming soon — Tauri desktop, mobile reader). Every protocol serves the same content. burrowd is a polyglot.

**The philosophy of less.** No followers. No likes. No notifications. No comments. No resharing. No DMs. If you appreciate something, bookmark it (public curation) or write a response on your own phlog (public conversation). Both require effort. That's the point. The social dynamics emerge from the content, not from engagement features.

**Discovery without algorithms.** Webrings (human-curated loops of related burrows), public bookmarks (the only ranking signal — how many people saved this?), Veronica-NG full-text search (BM25, same results for everyone, no filter bubble), and the firehose (every post, chronological, anti-algorithmic).

**Federation by URL.** No central server. No ActivityPub. Servers are connected by links, pings (lightweight HTTP POSTs when someone references your work), search index exports (JSON at a URL), and rings that span servers. The simplest model that could possibly work.

**The honest business.** Hosting, not advertising. Free tier has all features, limited storage. Paid tiers add capacity, not capability. Self-hosted is free forever. No venture capital. No growth-at-all-costs. If the company disappears, the software continues. Your files are on your disk.

For the full vision — brand identity, protocol design, social mechanics, revenue model, and the 90-day MVP plan — see `burrow-concept.md` or read the in-burrow version at `/~burrow/concepts/`.

---

## THE CLI — YOUR FIRST CLIENT

The CLI is the client that ships today. It's built, tested, and ready. Everything below works right now, on your machine, without waiting for a GUI.

### Publishing

```bash
# Create a burrow
burrow init bruno

# Write a post (opens $EDITOR with today's date pre-filled)
burrow new "Why I Left Medium"

# Edit an existing file
burrow edit about.txt

# List your burrow
burrow ls
burrow ls phlog/

# Stats: files, size, latest post
burrow status
```

The workflow is: write in your editor, save, done. The file is published. No build step, no deploy pipeline, no prayer to the CDN gods. The filesystem is the CMS.

### Multi-Burrow Management

```bash
# Create multiple burrows
burrow init alice
burrow init bob

# Switch between them
burrow switch alice
burrow switch        # lists all burrows, ← marks the active one
```

Each burrow is independent. Switch between them. Live your multitudes. If there's only one burrow, the CLI uses it automatically. If there are several and you haven't switched, it reads `burrows/.burrow-active`. If that doesn't exist, it tells you to pick one. Politely.

### Guestbook

```bash
# Create a guestbook
burrow guestbook init

# Read entries
burrow guestbook show
```

Visitors sign your guestbook at `/~you/guestbook` in the browser. The CLI lets you read entries. Because it's not the '90s internet without a guestbook.

### Server Management

```bash
# Initialize server config
burrow server init --domain phlogosphere.net --port 7070
```

Creates `burrow.conf` with your domain, port, and optional aliases for custom domains. The server reads this on startup.

### Search

```bash
# Search your active burrow
burrow search "typography"

# Search all burrows on the server
burrow search "rust async" --all
```

Case-insensitive grep with highlighted matches. Fast, local, no index needed. The server has BM25 full-text search at `/search` — the CLI search is simpler but works offline.

### Bookmarks

```bash
# Add a public bookmark
burrow bookmark add https://100r.co -d "Hundred Rabbits — off-grid computing"
burrow bookmark add /~maya/about -d "Maya's about page"

# List your bookmarks
burrow bookmark list

# Remove by number
burrow bookmark remove 3
```

Your bookmarks live at `/~you/bookmarks` as `bookmarks.gph`. They're public. They're your taste, on display. Bookmark counts are the only ranking signal on the Discover page.

### Rings (Webrings)

```bash
# Create a ring
burrow ring create "Deep Web Craft" -d "Writers who care about the web as a medium"

# Add members (local or remote)
burrow ring add deep-web-craft /~maya
burrow ring add deep-web-craft gph://tilde.town/~river

# Show ring members
burrow ring show deep-web-craft

# List your rings
burrow ring list

# Remove a member
burrow ring remove deep-web-craft /~maya
```

Rings span servers. Members can be local paths or `gph://` URLs. A ring can even include another ring (nested rings). Navigation arrows appear at the bottom of every page for every ring the burrow belongs to.

### Preview

```bash
# Preview a file in the terminal (including _ drafts)
burrow preview _work-in-progress.txt
burrow preview phlog/my-post.txt
```

Renders `.gph` markup in your terminal — headings, quotes, links, code blocks. Works on draft files that the server won't serve. See what your readers will see, without publishing.

### Archival & Sync

```bash
# Export a backup
burrow export ~/backups/alice-2026-03.tar.gz

# Push your burrow to a remote server
burrow push user@phlogosphere.net:/srv/burrow/burrows/

# Pull from remote
burrow pull user@phlogosphere.net:/srv/burrow/burrows/~bruno/

# Generate a year-in-review
burrow timecapsule 2026
```

`export` creates a tar.gz backup. `push`/`pull` use rsync over SSH — your burrow is files, you copy them. `timecapsule` produces a yearly stats summary with post counts, word totals, and a chronological index.

### Protocol Handler

```bash
# Register gph:// URL handler on your OS
burrow register

# Open a gph:// URL
burrow open gph://phlogosphere.net/~bruno/about
```

On macOS, `register` creates a `.app` bundle. On Linux, a `.desktop` file. Click a `gph://` link anywhere on your system and Burrow handles it. `open` previews the content locally if possible, otherwise opens it in your default browser.

---

## THE DESKTOP CLIENT (COMING SOON)

The Tauri desktop client is next. Here's what it will be:

**Small.** Under 8 MB. Slack is 300 MB. The Burrow client does more with 3% of the space.

**Fast.** Starts in under a second. No Electron. No Chromium. Tauri uses your system's native webview. Rust backend, HTML/CSS frontend.

**Keyboard-first.** Every action has a shortcut:

| Key | Action |
|-----|--------|
| `Space` | Scroll down |
| `Shift+Space` | Scroll up |
| `Esc` | Go up one directory |
| `Enter` | Follow link |
| `b` | Toggle bookmarks panel |
| `s` | Open search |
| `/` | Quick-navigate (type a path) |
| `[` / `]` | Previous / next in directory |
| `g` | Go to address |

Mouse and touch work fine. But the keyboard shortcuts make it feel like a power tool, not a consumption app.

**Built-in editor.** Monospace. No formatting toolbar. No preview pane — what you write is what gets published. Character count in the corner. Auto-save every 30 seconds. `Cmd+Enter` to publish. One power feature: link completion — type `/~` and it suggests from your bookmarks and recent visits.

**Native `gph://`.** The desktop client speaks the Burrow protocol directly. No HTML wrapping. No browser overhead. Just the protocol, the renderer, and your words.

---

## THE MOBILE APP (COMING LATER)

iOS and Android. Reader-first. **No publishing from mobile.**

This is a design statement. We don't want you micro-blogging from the toilet. Write something worth writing. Sit down. Think about it. Use a keyboard. Publishing is a deliberate act.

Mobile is for reading, bookmarking, and discovering. Fast, offline-capable, and treats your phone as what it should be: a reading device, not a content extraction machine.

---

## THERE IS NO WEB APP

The HTTPS gateway is read-only. If you want to participate — publish, bookmark, join rings — you use the native client or the CLI.

This is intentional friction. It filters for people who care enough to install something.

The web showed us what happens when publishing has zero friction: you get a billion posts a day and most of them are noise. Burrow's mild friction is a quality filter. Not a gate — anyone can install the client. But a speed bump that says: are you sure you have something to say?

---

## THE READING EXPERIENCE

When you open a text file — in the browser gateway, the CLI, or the upcoming desktop client — the UI disappears. What remains:

**Typography.** Literata serif at 17px. Warm, highly legible. Line-height 1.7. Max width 65 characters. The content is the interface.

**Progress.** A thin 2px line at the top fills as you scroll. When you reach the end, it pulses once and fades.

**Reading time.** Below the title, a ghost line: `~4 min read · 847 words`. It fades after three seconds. You know what you're committing to, but the information doesn't compete with the words.

**Colors.** Warm off-white background (`#faf9f7`), soft almost-black text (`#1a1a1a`). The effect is subtle: it feels like paper, not a screen. Dark mode inverts these. The accent color stays.

**No distractions.** No sidebar. No toolbar. No notification badge. No "recommended for you" panel. Just you and someone else's words. That used to be called "reading."

---

## SEE ALSO

- `burrowd-manual.md` — the server manual. Same Big Picture, different responsibilities.
- `burrow-concept.md` — the full vision document. Contains ambition and a revenue model.
- Your favorite text editor — the real authoring tool.
- Your favorite feed reader — for `/~user/feed.xml` (RSS) or `/~user/atom.xml` (Atom).
- Your terminal — `ls`, `cat`, `grep`. The original content management system.
- `/~burrow/concepts/` — the in-burrow version of the Big Picture. Dogfooding at its finest.

## AUTHORS

Built with Rust, Clap, and the conviction that a CLI is a perfectly good user interface.

The desktop client will be built with Tauri, because Electron taught us what happens when you ship an entire browser to display a text editor. The mobile app will be built with whatever framework produces the smallest binary. We have opinions about binary size. Strong ones.

## COLOPHON

```
        ___
       /   \
      | o o |     burrow — read things, write things,
       \ _ /      bookmark the good things.
       /| |\
      / | | \     No notifications. No badges.
     *  | |  *    No "you have 12 unread."

                  You open it when you want to.
                  You close it when you're done.
```
