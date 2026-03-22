# burrowd(1) — The Server That Refuses to Grow Up

```
         ___
        /   \      burrowd v0.1.0
       | o o |     "The internet, minus the parts
        \ _ /      that made you hate the internet."
        /| |\
       / | | \     A hole in the ground.
      *  | |  *    On purpose.
```

---

## NAME

**burrowd** — a plaintext content server for people who think the web peaked around 1997 but couldn't quite admit it until now.

## SYNOPSIS

```
burrowd
```

That's it. No flags. No `--verbose --enable-kubernetes --sync-to-cloud --blockchain`. Just `burrowd`. It reads its config. It serves your words. It does not have opinions about your JavaScript framework.

## DESCRIPTION

**burrowd** serves plaintext content from a `burrows/` directory over HTTP, HTTPS, and Gemini — simultaneously, if you're feeling ambitious — with a reading-optimized UI that would make Edward Tufte nod approvingly and then immediately suggest three improvements.

It renders `.txt` files with a clean typographic layout (Literata for prose, JetBrains Mono for everything else), provides directory listings with the quiet dignity of an FTP server, hosts ASCII art galleries, manages webrings, runs a full-text search engine, and steadfastly refuses to execute a single line of JavaScript beyond a scroll progress bar. (We had a heated internal debate about the progress bar. It stayed. Barely. The vote was 3-2 and we don't talk about it at team dinners.)

Every burrow lives in a `~username/` directory, because tildes were the original social network and we will die on this hill.

## HOW IT WORKS

```
burrows/
  ~alice/
    .burrow            ← config (description, accent color, etc.)
    about.txt          ← Alice's about page
    bookmarks.gph      ← Alice's public bookmarks
    guestbook.gph      ← sign it, you coward
    phlog/             ← Alice's phlog (that's "blog" but with a PhD in nostalgia)
      .burrow          ← sort = modified-desc, pin = best-post.txt
      2026-03-20-hello-world.txt
    gallery/           ← ASCII art, because we have culture
      landscape.txt
    rings/             ← webrings, because it's not 2026 without 1998
      deep-web-craft.ring
  ~bob/
    .burrow
    about.txt
    phlog/
```

When someone visits `http://yourserver:7070/~alice/phlog/2026-03-20-hello-world`, burrowd:

1. Finds `burrows/~alice/phlog/2026-03-20-hello-world.txt`
2. Reads the file (if it's under 64 KB — we have *standards*)
3. Renders it in a beautiful reading view with Literata serif at 17px
4. Checks if anyone else on the server linked to this post and shows "Mentioned by" at the bottom
5. Checks if Alice is in any webrings and shows `← Previous · ◎ Ring Name · Next →` navigation
6. That's it. There is no step 7. Go outside.

### What Gets Served

burrowd has more routes than a European rail map, but each one does exactly one thing:

| Path | What Happens |
|------|-------------|
| `/` | Home page: lists all `~user/` burrows. The original social network. |
| `/~alice` | Directory listing of Alice's burrow. Like `ls`, but prettier. |
| `/~alice/about` | Renders `about.txt`. Yes, `.txt` is auto-appended. We're helpful like that. |
| `/~alice/phlog/` | Directory listing, sorted and pinned per `.burrow` config. |
| `/~alice/gallery/` | ASCII art gallery with thumbnail grid. We have *culture*. |
| `/~alice/guestbook` | Guestbook with a form to sign. No JavaScript. The `<form>` tag still works. Who knew. |
| `/~alice/bookmarks` | Public bookmarks. Your taste, on display. |
| `/~alice/feed.xml` | RSS 2.0 feed of Alice's phlog. Auto-generated. Auto-discovered. |
| `/~alice/atom.xml` | Atom 1.0 feed. For the Atom purists. We don't judge. (We judge a little.) |
| `/discover` | The front page of the burrowverse: trending bookmarks, latest posts, rings, random burrow spotlight. |
| `/firehose` | Every post, every burrow, reverse chronological. The anti-algorithm. |
| `/rings` | All webrings on this server, with member tags. It's 1998 and it's *beautiful*. |
| `/search?q=...` | Veronica-NG full-text search. BM25 ranking. Freshness boost. Operators. The works. |
| `/random` | Redirects to a random burrow. The StumbleUpon of plaintext. |
| `/health` | `{"status":"ok"}` — for your monitoring stack that's more complex than this entire server. |
| `/stats` | `{"burrows":N,"files":N,"uptime_secs":N}` — for the dashboards you definitely need. |
| `/ping` | Federation endpoint. Other Burrow servers POST here when their users link to yours. |
| `/search/index.json` | Full document index for federation. Other servers can import this for cross-server search. |
| `/robots.txt` | `Allow: /` — because we have nothing to hide. |
| `/favicon.ico` | 62 bytes of transparent pixel. The most efficient favicon on the internet. |
| `/nonexistent` | A 404 page with existential undertones. "/∅ This hole leads nowhere." |

### Content Format — Burrow Markup (.gph)

Your `.txt` and `.gph` files support a markup format so minimal it barely qualifies as markup. We call it "gph" because naming things is the second hardest problem in computer science, right after cache invalidation and off-by-one errors.

```
# Headings
Start a line with "# " and it becomes an <h1>. We only support one level
because heading hierarchies are a slippery slope to PowerPoint.

Regular paragraphs
Just write. Like a person. With words. No bold. No italic.
If your words need formatting to be understood, rewrite them.

> Blockquotes
> For when someone else said it better.

---
Horizontal rules. For dramatic pauses. Use sparingly,
like hot sauce and semicolons.

  Code blocks
  Indent with two spaces. We'll wrap it in <pre>.
  No syntax highlighting. Your code should speak for itself.
  (If it can't, that's a code review issue, not a rendering issue.)

→ https://example.com
External links. The arrow is not optional. Commitment is important.

/~alice/about   Internal links
Three spaces separate the path from the description.
```

**What's deliberately missing:** Bold. Italic. Tables. Inline links. Images. Headings h2-h6. Custom fonts. Custom colors. Emoji shortcodes. If you just felt a wave of relief, you're our target audience.

### Hidden Files & Drafts

Files starting with `.` or `_` are hidden from directory listings *and* blocked from direct HTTP access. This isn't a suggestion — it's enforced at the path traversal layer. Your `_half-baked-manifesto.txt` is invisible to the world until you remove that underscore.

Preview drafts locally:
```bash
burrow preview _my-draft.txt
```

The `_` convention is the publishing model. No "draft mode" toggle. No "publish" button. Just a filename. Unix had this figured out in 1971.

### Binary & Image Serving

burrowd serves images and binary files with correct MIME types and caching headers:

| Format | MIME type |
|--------|-----------|
| `.png`, `.jpg`, `.gif`, `.webp`, `.svg` | Correct image types |
| `.pdf` | `application/pdf` |
| `.mp3`, `.ogg` | Audio types |
| `.woff`, `.woff2` | Font types |
| `.zip`, `.tar`, `.gz` | Archive types |

Binary files have a 2 MB limit (vs. 64 KB for text). Cache headers are set to 1 hour. Your ASCII art gallery can have a cover image now. We're not monsters.

## CONFIGURATION

### Server Config (`burrow.conf`)

Configuration lives in `burrow.conf` in the working directory. Generate one with:

```bash
burrow server init --domain myhole.example.com --port 7070
```

Full reference:

| Key | Default | Description |
|-----|---------|-------------|
| `domain` | `localhost` | Your server's domain name. Shown in the address bar, feeds, and your sense of identity. |
| `port` | `7070` | Port to listen on. 7070 because 70 was Gopher's port and we're exactly 100x better. (Citation needed.) |
| `tls_cert` | *(none)* | Path to TLS certificate PEM file. Set this and `tls_key` to enable HTTPS. |
| `tls_key` | *(none)* | Path to TLS private key PEM file. |
| `gemini_port` | *(none)* | Port for Gemini protocol (typically 1965). Requires TLS. |

That's five keys. We considered adding more but got distracted reading about the Unix philosophy and never came back.

If `burrow.conf` doesn't exist, burrowd defaults to `localhost:7070` over plain HTTP, which is fine for local development and existential crises.

### Per-Burrow Configuration (`.burrow`)

Each burrow (and subdirectory) can have a `.burrow` file. It's the closest thing to a "settings page" you'll find here:

```
description = ITAD, web craft, and too many opinions about typography
accent = #e06030
title = My Fancy Name
sort = modified-desc
pin = important-post.txt readme.txt
```

| Key | Description |
|-----|-------------|
| `description` | Shown in directory listings next to the burrow name. Your elevator pitch in one line. |
| `accent` | Hex color (`#abc` or `#aabbcc`) that overrides the default teal accent. Your burrow, your color. |
| `title` | Overrides the directory name in listings and headings. Because `phlog/` deserves a better name. |
| `sort` | Sort order: `name-asc` (default), `name-desc`, `modified-desc`, `modified-asc`. Perfect for phlogs. |
| `pin` | Space-separated list of filenames to pin to the top. Your greatest hits, always first. |

## RUNNING

```bash
burrowd

#   / burrow v0.1.0
#
#   Tunneling...
#
#   Domain:         myhole.example.com
#   Local:          http://localhost:7070
#   Network:        http://192.168.0.42:7070
#   Burrow root:    ./burrows/
#
#   Press Ctrl+C to fill in the hole.
```

The server binds to `0.0.0.0` — accessible on your local network. Your phone can read your phlog from the couch. This is what the internet was supposed to feel like.

### TLS (HTTPS)

Add your certificate to `burrow.conf` and burrowd speaks HTTPS natively. No nginx required. No Caddy. No reverse proxy. Just burrowd and your cert:

```conf
tls_cert = /etc/letsencrypt/live/mysite.com/fullchain.pem
tls_key = /etc/letsencrypt/live/mysite.com/privkey.pem
```

burrowd uses `rustls` under the hood — a pure-Rust TLS implementation. No OpenSSL. No `libssl`. No CVE lottery ticket. Just memory-safe cryptography.

### Gemini Protocol

Add a `gemini_port` and burrowd also speaks Gemini — the minimalist protocol that makes Gopher look bloated:

```conf
gemini_port = 1965
tls_cert = /path/to/cert.pem
tls_key = /path/to/key.pem
```

Your `.gph` files are automatically converted to Gemtext (`.gmi`). Gemini clients see your content natively. Directory listings render as Gemini link pages. It's not a separate server — it's the same burrowd, speaking another language.

The conversion is nearly 1:1 because `.gph` markup was designed to be close to Gemtext. `# Heading` stays `# Heading`. `> Quote` stays `> Quote`. `→ URL` becomes `=> URL`. Code blocks get triple backticks. That's it.

### systemd

A ready-made service file is included as `burrowd.service`:

```bash
cp burrowd.service /etc/systemd/system/
systemctl enable burrowd
systemctl start burrowd
```

### Docker

```bash
docker build -t burrow .
docker run -p 7070:7070 -v ./burrows:/burrow/burrows burrow
```

### One-liner Install

```bash
curl -fsSL https://raw.githubusercontent.com/Simplinity/burrow/master/install.sh | bash
```

Detects your OS and architecture, downloads the latest release, installs to `/usr/local/bin` or `~/.local/bin`. Three commands later you're serving plaintext to the world. The entire install is smaller than a typical React component.

### Access Logging

```bash
RUST_LOG=info burrowd      # default — method, path, status, latency
RUST_LOG=debug burrowd     # verbose — for when things are weird
RUST_LOG=warn burrowd      # quiet — only problems
```

Structured logging via `tracing`. Every request gets a single log line. No log4j. No log rotation daemon. No 50GB `/var/log` surprise.

## SECURITY

We take security seriously, even if we don't take much else seriously:

- **Path traversal protection**: All paths are canonicalized via `fs::canonicalize()` and verified to stay inside `burrows/` via `starts_with()`. Your `../../etc/passwd` jokes won't work here. We've heard them all. We've also heard `....//....//etc/passwd` and `%2e%2e%2f`. Nice try.
- **Two-layer HTML escaping**: `html_escape()` for text content (`<` → `&lt;`), `html_escape_attr()` for URL attributes (`"` → `&quot;`). Because XSS is not a feature we're shipping.
- **XML escaping**: RSS and Atom feeds are escaped separately. Your phlog title `<script>alert('rss readers hate this')` will render as literal text, as God intended.
- **File size limits**: 64 KB for text, 2 MB for binaries. If your plaintext file is over 64 KB, you may be writing a novel. We respect that, but we won't serve it. Write a book proposal instead.
- **Path depth limit**: Maximum 8 directory levels. If you need 9, you need a database, not a burrow.
- **Draft enforcement**: Files starting with `_` or `.` are blocked at the HTTP layer. Not hidden. *Blocked.* `403` would be too informative, so we return `404`. The file doesn't exist as far as the internet is concerned.
- **Rate limiting**: Guestbook POST: 1 per 30 seconds per IP. Federation ping POST: max 100 entries. Spammers will need to be *very* patient.
- **TLS**: Native `rustls` — no OpenSSL, no `libssl`, no C code in the TLS stack. Memory safety all the way down.
- **No file uploads**: burrowd serves files. It does not accept them. Content goes in via the CLI or `scp` like a civilized person. The guestbook form is the one exception. The federation ping endpoint is the other exception. We had a very long meeting.

## GUESTBOOK

Every burrow can have a guestbook. Because it's not the '90s internet without one.

```bash
burrow guestbook init
```

Visitors sign it at `/~yourname/guestbook` — a plain HTML form, no JavaScript, works in Lynx, works in curl (if you're that kind of person), works in the browser your grandma uses. The form submits a POST, appends to a file, redirects back. Web 1.0 engineering at its finest.

```
--- Nostalgic Gopher · 2026-03-22 01:56
Your site loads faster than my thoughts. Respect.

--- Anonymous · 2026-03-22 14:30
I haven't seen a guestbook since GeoCities. I'm not crying, you're crying.

--- xXx_DarkCoder_xXx · 2026-03-22 15:12
tried to XSS your guestbook. it escaped my script tag.
i respect that. signing the book instead.
```

| Limit | Value | Why |
|-------|-------|-----|
| Name | 40 chars | You're signing a book, not writing your LinkedIn headline |
| Message | 500 chars | A note, not a blog post. You have your own burrow for that |
| Total entries | 200 | After 200, start a new guestbook. Or bask in your popularity |
| Format injection | `---` → `—` | Nice try with the entry separators |

## BOOKMARKS

Every burrow can have public bookmarks at `/~user/bookmarks`. They're your curated links — your taste, on display.

```bash
burrow bookmark add https://100r.co -d "Hundred Rabbits — off-grid computing"
burrow bookmark add /~maya/about -d "Maya's about page"
burrow bookmark list
burrow bookmark remove 3
```

The `bookmarks.gph` file uses the same link format as regular `.gph` files:

```
→ https://100r.co   Hundred Rabbits — off-grid computing · 2026-03-19
/~maya/about   Maya's about page · 2026-03-20
```

Bookmark counts are aggregated across all burrows and surface on the Discover page as "★ Most bookmarked" — the only ranking signal in Burrow. No algorithm. No engagement score. Just: how many people thought this was worth saving.

## RINGS (WEBRINGS)

Webrings are back. Not ironically. Well, maybe a little ironically. But also sincerely.

A ring is a curated loop of burrows — a navigable circle of related voices. When you're reading a page from a ring member's burrow, navigation arrows appear at the bottom:

```
← Previous · ◎ Deep Web Craft · Next →
```

Click through and you're visiting the next burrow in the ring. It's like channel surfing, but for thoughtful writing.

### Creating a Ring

```bash
burrow ring create "Deep Web Craft" -d "Writers who care about the web as a medium"
burrow ring add deep-web-craft /~maya
burrow ring add deep-web-craft gph://tilde.town/~river
burrow ring show deep-web-craft

#   ◎ /~bruno
#   ○ /~maya
#   ○ gph://tilde.town/~river
```

### Ring Files

Rings live in `~/rings/` as `.ring` files:

```
title = Deep Web Craft
description = Writers who care about the web as a medium

/~bruno
/~maya
gph://tilde.town/~river
```

Members can be local (`/~user`) or remote (`gph://host/~user`) — rings span servers. That's federation without a protocol spec, an ActivityPub implementation, or a PhD in distributed systems.

### Nested Rings

A ring can include another ring:

```
title = Indie Web
description = The broader indie web movement

/~bruno
ring:~bruno/deep-web-craft
gph://indieweb.org/~alice
```

The `ring:` reference is flattened at load time — members from the nested ring are merged in, no duplicates. It's rings all the way down. (But not infinitely. We have stack limits. And taste.)

### Ring Pages

- `/rings` — all rings on the server, with member tags
- `/discover` — rings section with member counts
- Ring navigation appears on every text page of every member

## SEARCH — VERONICA-NG

Gopher's original search engine was called Veronica (Very Easy Rodent-Oriented Net-wide Index to Computerized Archives). The '90s were *wild*.

Burrow resurrects it as Veronica-NG — a full-text search engine built into burrowd. No external service. No Elasticsearch cluster. No 4GB Java heap. Just an in-memory inverted index built on startup from your filesystem. Like a search engine for people who think search engines should be small.

### Usage

Visit `/search` or use the sidebar link. Type a query. Get results. That's it.

```
/search?q=typography                     → full-text search
/search?q=plaintext author:~bruno       → filter by author
/search?q=web type:phlog                → only phlog posts
/search?q=minimalism fresh:30           → last 30 days only
```

### Ranking

Results are ranked by BM25 (the same algorithm Wikipedia uses) with two Burrow-specific tweaks:

1. **Title boost**: Terms found in titles score 3x higher. Because titles are important. That's why they're titles.
2. **Freshness boost**: Recent posts get up to 1.5x score, decaying linearly over 90 days. A post from today scores higher than the same words written a year ago. After 90 days, freshness is neutral. Your evergreen content isn't penalized — it just doesn't get a recency bonus.

There is no personalization. No "recommended for you." No filter bubble. The same query returns the same results for everyone. If that means you occasionally discover something outside your comfort zone — good. That's a feature.

### Federation Index

`/search/index.json` exports a JSON document of all indexed content:

```json
{
  "version": 1,
  "server": "myhole.example.com",
  "documents": [
    {"path": "/~bruno/phlog/...", "title": "...", "author": "~bruno", "type": "phlog", "date": "2026-03-17", "words": 318}
  ]
}
```

Other Burrow servers can import this for cross-server search. No crawling, no scraping, no robots.txt negotiations. Just a JSON file. Federation by URL. The simplest model that could possibly work.

## DISCOVER

`/discover` is the front page of the burrowverse — a daily-changing snapshot of what's happening:

- **★ Most bookmarked** — pages that the most burrow owners saved. The only popularity metric. No likes. No views. Just: someone cared enough to bookmark it.
- **Random burrow** — a different burrow on every page load. The StumbleUpon of plaintext.
- **Latest posts** — newest phlog entries across all burrows. Chronological. Unranked. Anti-algorithmic.
- **Rings** — active webrings with member counts.
- **All burrows** — the full directory. Because sometimes you just want to browse.

## FIREHOSE

`/firehose` is every post, from every burrow, in reverse chronological order. It's the anti-algorithm. No ranking. No filtering. No "based on your interests." Just everything, newest first, paginated.

It's intentionally overwhelming. It's not meant to be read cover-to-cover. It's meant to be glanced at — to get a sense of what's happening. You pick up patterns: "a lot of people are writing about Rust today" or "there's a cluster of climate posts."

## MENTIONS (BURROW-TO-BURROW PING)

When a post links to another post (via `/~user/path` internal links), the target post shows a "Mentioned by" section at the bottom:

```
← Mentioned by · 2 posts
  On digital minimalism (~maya)
  https://other.server/~alice/response (~remote)
```

This works both locally (scanning all posts on the server) and via federation — other Burrow servers can POST to `/ping` to notify you when their users link to your content.

### Federation Pings

**Incoming:** Other servers POST to `/ping` with `source=URL&target=/~user/path`. Stored in `burrows/.pings`.

**Outgoing:** On startup, burrowd scans all posts for `gph://` links and sends HTTP POST pings to the target servers. Already-sent pings are tracked in `burrows/.pings-sent` to avoid spam.

It's like Webmention, but without the W3C spec, the verification dance, or the existential crisis about whether ActivityPub is better. Just: "hey, someone linked to your post." That's it.

## THE ADDRESS BAR

You may notice the address bar shows `gph://yourdomain.com/path`. This is an aesthetic choice referencing the Gopher protocol. It is not a real protocol. (Yet. We registered the handler. `burrow register` creates a system-level URL scheme handler. On macOS it builds a `.app` bundle. On Linux it writes a `.desktop` file. We may have gotten carried away. We regret nothing.)

## GALLERY

Any directory named `gallery/` gets special treatment. Instead of a plain file listing, burrowd renders a CSS grid of ASCII art preview cards — each showing a 12-line monospace preview at 7px, with a gradient fade and hover effects.

Click a card and the full art renders in a dedicated monospace viewer with auto-scaled font size (6-12px based on character width). It's an art exhibition for the terminal-pilled.

```bash
# Add art to your gallery
echo "Your ASCII masterpiece" > burrows/~you/gallery/masterpiece.txt
```

No uploads, no moderation queue, no content policy. If it fits in a `.txt` file and it's under 64 KB, it's art.

## RSS & ATOM

Every burrow gets both an RSS 2.0 and an Atom 1.0 feed. Auto-generated. Auto-discovered via `<link>` tags. No configuration. No plugins. No "install the RSS extension."

```
/~alice/feed.xml    ← RSS 2.0
/~alice/feed        ← same, for the lazy typist
/~alice/atom.xml    ← Atom 1.0
/~alice/atom        ← same, lazy typist edition
```

Your favorite feed reader will find them automatically. You have a feed reader, right? *Right?*

## TIME CAPSULE

```bash
burrow timecapsule 2026
```

Generates a `timecapsule-2026.txt` — a year-in-review of your burrow:

- Post count, total words, average per post
- Reading time, months active, total size
- First post, last post, longest, shortest
- Monthly activity chart with `█` bars
- Chronological post index

It's private nostalgia, automatically generated. Who were you online twelve months ago? The time capsule knows.

## LIMITS

| Thing | Limit | Why |
|-------|-------|-----|
| Text file size | 64 KB | If you need more, write a book. We'll buy a copy. |
| Binary file size | 2 MB | Enough for images. Not enough for video. Intentionally. |
| Path depth | 8 levels | `~user/a/b/c/d/e/f/g/h`? You need therapy, not a deeper directory. |
| Guestbook rate | 1 post / 30 sec / IP | "But I have so many opinions!" — Write a phlog post. |
| Federation pings | 100 per server | We believe in conversation, not DDoS. |
| JavaScript | 1 scroll handler | It is on probation. It knows what it did. |
| Frameworks | 0 | This is a feature, not a limitation. |
| Databases | 0 | The filesystem is the database. `ls` is the admin panel. Always has been. |
| Config keys | 5 (server) + 5 (per-burrow) | Ten total. The entire system configuration fits in a tweet. |
| POST endpoints | 2 (guestbook + ping) | We agonized over the first one. The second one went faster. |
| Protocols spoken | 3 (HTTP, HTTPS, Gemini) | Polyglot, not polygraph. |

## TROUBLESHOOTING

**Q: burrowd says "Address already in use"**
A: Another burrowd is already running. Or something else is on port 7070. `lsof -i :7070` to find the culprit. `kill` to resolve the dispute. We can't help with your commitment issues.

**Q: I see "localhost" everywhere instead of my domain**
A: Run `burrow server init --domain yourdomain.com`. The domain is cosmetic for the UI but important for feeds and your sense of self.

**Q: My file isn't showing up**
A: Does it start with `.` or `_`? Blocked. Is it in the `burrows/~yourname/` directory? Is burrowd running? Have you tried looking at it? Really looking? With your eyes? Sometimes the answer is `ls`.

**Q: Can I use Markdown?**
A: No. We use `.gph` markup. It's simpler. You'll get used to it in about four minutes. Or you won't. Either way, your words will still be perfectly readable in any text editor for the next fifty years, long after Markdown has been replaced by whatever comes next. (Our money is on "just plain text." It always was "just plain text.")

**Q: Where's the admin panel?**
A: `ls`, `cat`, `vim`. The admin panel that ships with every Unix system since 1971. It has zero CVEs related to admin panel exploits.

**Q: Can I add custom CSS/themes?**
A: You can set `accent = #hexcolor` in your `.burrow` config. The rest of the design is the design. We spent time on it. It has a light mode and a dark mode and they both look good and we will not be taking questions at this time.

**Q: How do I add images?**
A: Put them in your burrow directory. burrowd serves PNG, JPEG, GIF, SVG, WebP, and PDF with correct MIME types and cache headers. Reference them from other platforms if you want. This isn't Instagram. There's no filter. There's no crop. It's a file.

**Q: Can I have multiple burrows?**
A: `burrow init alice && burrow init bob && burrow switch alice`. Each burrow is independent. Switch between them. Live your multitudes.

**Q: What's the Gemini stuff about?**
A: Gemini is a protocol from 2019 that makes Gopher look bloated. If you add TLS certs and a `gemini_port` to your config, burrowd serves your content over Gemini too. Same content, different protocol. Your `.gph` files are auto-converted to Gemtext. Gemini users see your burrow in their native client. You didn't have to do anything. You're welcome.

**Q: What's a "phlog"?**
A: "Gopher log." Like "blog" but with better etymological credentials and worse SEO. It's a directory of text files sorted by date. The filesystem is the CMS.

## CLI REFERENCE

The `burrow` CLI is the power tool. Everything the server does, the CLI controls:

| Command | What it does |
|---------|-------------|
| `burrow init <name>` | Dig a new burrow. Creates `~name/` with skeleton files. |
| `burrow new "Title"` | Create a phlog post. Opens `$EDITOR`. Today's date auto-prefixed. |
| `burrow edit <path>` | Open any file in `$EDITOR`. |
| `burrow preview <path>` | Terminal preview with gph rendering. Works on `_` drafts. |
| `burrow ls [path]` | List burrow contents. Like `ls` but aware of `.burrow` config. |
| `burrow status` | Stats: files, size, latest post, disk usage. |
| `burrow switch [name]` | Switch active burrow (or list all). |
| `burrow search "query" [--all]` | Grep-style search with highlighted matches. `--all` searches every burrow. |
| `burrow push <remote>` | Sync burrow to remote server via rsync/SSH. |
| `burrow pull <remote>` | Sync burrow from remote server. |
| `burrow export [output]` | Backup as `.tar.gz`. Your data. Your files. Your backup. |
| `burrow timecapsule [year]` | Generate a year-in-review. Nostalgia as a service. |
| `burrow bookmark add <url>` | Add a public bookmark. |
| `burrow bookmark list` | Show your bookmarks. |
| `burrow bookmark remove <n>` | Remove by number. |
| `burrow ring create "Name"` | Create a webring. |
| `burrow ring add <ring> <member>` | Add a member (local or `gph://` remote). |
| `burrow ring remove <ring> <member>` | Remove a member. |
| `burrow ring show <ring>` | Display ring members. |
| `burrow ring list` | List your rings. |
| `burrow guestbook init` | Create a guestbook. |
| `burrow guestbook show` | Read guestbook entries. |
| `burrow open <gph://url>` | Open a `gph://` URL — local preview or browser. |
| `burrow register` | Register `gph://` protocol handler on your OS. |
| `burrow server init` | Configure server domain and port. |

## SEE ALSO

- `burrow-concept.md` — the full vision document. Warning: contains ambition, dry humor, and a surprisingly detailed revenue model.
- Your favorite text editor — the real authoring tool. Vim, Emacs, nano, ed. We don't judge. (We judge a little. Use what makes you happy.)
- Your favorite feed reader — subscribe to `/~user/feed.xml` (RSS) or `/~user/atom.xml` (Atom). NetNewsWire, Miniflux, Elfeed. You have one, right?
- `/firehose` — the cross-burrow chronological feed. The anti-algorithm.
- `/discover` — the front page of the burrowverse.
- `/rings` — webrings. It's not retro, it's *timeless*.
- `/search` — Veronica-NG. Named after a search engine from 1992. We are very normal about history.
- RFC 1436 — the Gopher protocol spec, for historical context and mild nostalgia.
- RFC 7764 — Gemini is not an RFC but it feels like it should be.
- Your terminal — `ls`, `cat`, `grep`. The original content management system.

## AUTHORS

Built with Rust, Axum, Tokio, and a stubborn belief that the internet should be readable.

No JavaScript frameworks were mass-produced in the making of this server. No containers were orchestrated. No microservices were architected. No standups were held. No sprints were planned. No OKRs were aligned. No stakeholders were engaged.

One person sat down and wrote a server that serves text files. Then another person read those text files and thought they were good.

That's the whole story.

## COLOPHON

**Runtime dependencies:** Tokio (async), Axum (HTTP), tower-http (logging), rustls (TLS), tokio-rustls (Gemini TLS), chrono (dates), clap (CLI), serde (config parsing). That's it. Eight crates. The `node_modules` folder of a typical "Hello World" React app contains more code than this entire server.

**Test suite:** 36 tests covering gph rendering, HTML escaping, XSS prevention, directory listing, file operations, and page structure. They all pass. Every time. Because there are 36 of them, not 36,000.

**Binary size:** Under 10 MB for a release build. For comparison, the Slack desktop app is 300 MB and it just shows you messages. burrowd serves files over three protocols, runs a search engine, manages webrings, handles federation, and renders typography — in 3% of the space.

**Memory usage:** Roughly 15 MB at idle. The search index scales linearly with content. A server with 1,000 burrows and 10,000 posts would use approximately 50 MB. Your browser tab showing this manual is using more RAM right now.

The fonts (JetBrains Mono and Literata) do more aesthetic heavy lifting than the entire backend. We're okay with that. Typography is important. Your words deserve to look good.

```
        ___
       /   \
      | o o |     burrowd — because sometimes less is more,
       \ _ /      and sometimes more was never needed.
       /| |\
      / | | \     53 features. 3 protocols. 1 binary.
     *  | |  *    0 JavaScript frameworks.

                  Now go write something.
```
