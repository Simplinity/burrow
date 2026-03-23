# burrowd(1) — The Server That Refuses to Grow Up

```
         ___
        /   \      burrowd v0.9.1
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

**burrowd** serves plaintext content from a `burrows/` directory over HTTP with a reading-optimized UI. Think of it as a very opinionated web server that looked at the modern internet, sighed heavily, and decided to do less.

It renders `.txt` files with a clean typographic layout (Literata for prose, JetBrains Mono for code), provides directory listings with the quiet dignity of an FTP server, and serves zero JavaScript. Not "minimal JavaScript." Not "just one tiny script." Zero. The scroll progress bar is pure CSS. The guestbook form is plain HTML. We checked.

Every burrow lives in a `~username/` directory, because tildes were the original social network and we will die on this hill.

## HOW IT WORKS

```
burrows/
  ~alice/
    .burrow            ← config (description, etc.)
    about.txt          ← Alice's about page
    phlog/             ← Alice's phlog (that's "blog" but with a PhD in nostalgia)
      2026-03-20-hello-world.txt
  ~bob/
    .burrow
    about.txt
    phlog/
```

When someone visits `http://yourserver:7070/~alice/phlog/2026-03-20-hello-world`, burrowd:

1. Finds `burrows/~alice/phlog/2026-03-20-hello-world.txt`
2. Reads the file (if it's under 64 KB — we have *standards*, and they're strict)
3. Renders it in a beautiful reading view
4. That's it. There is no step 4. Go outside.

### What Gets Served

| Path | What Happens |
|------|-------------|
| `/` | Home page: lists all `~user/` burrows with descriptions |
| `/~alice` | Directory listing of Alice's burrow |
| `/~alice/about` | Renders `about.txt` (yes, `.txt` is auto-appended) |
| `/~alice/phlog/` | Directory listing of Alice's phlog |
| `/~alice/guestbook` | Guestbook with a form to sign (see GUESTBOOK below) |
| `/~alice/bookmarks` | Public bookmarks page (see BOOKMARKS below) |
| `/~alice/gallery/` | ASCII art gallery with grid preview (see GALLERY below) |
| `/~alice/feed.xml` | Auto-generated RSS feed (see FEEDS below) |
| `/~alice/atom.xml` | Auto-generated Atom feed |
| `/search?q=...` | Full-text search with BM25 ranking (see SEARCH below) |
| `/search/index.json` | Machine-readable search index for federation |
| `/discover` | Discovery page: latest posts, most bookmarked, random burrow, rings |
| `/firehose` | Chronological stream of all posts across all burrows (paginated) |
| `/rings` | List of all webrings on this server |
| `/random` | Redirects to a random burrow. Serendipity as a service. |
| `/health` | Returns `{"status":"ok"}` — for uptime monitors and anxious sysadmins |
| `/stats` | Returns `{"burrows":N,"files":N,"uptime_secs":N}` — server vitals |
| `/robots.txt` | Allows all crawlers. We have nothing to hide. |
| `/favicon.ico` | Transparent 1x1 icon. Stops browsers from 404-ing on every page load. |
| `/servers` | Server directory: curated list of known Burrow servers |
| `/~alice/stats` | Anonymous reader count (page loads this month) |
| `/~alice/subscriptions.opml` | OPML export of bookmarks for feed readers |
| `/.well-known/*` | RFC 8615 well-known URIs (security.txt, humans.txt, etc.) |
| `POST /ping` | Federation ping receiver (see FEDERATION below) |
| `/nonexistent` | A 404 page with existential undertones |

### Content Format

Your `.txt` files support a lightweight markup that we're calling "gph" because naming things is hard:

```
# Headings
Start a line with "# " and it becomes an <h1>. We only support one level
because heading hierarchies are a slippery slope to PowerPoint.

Regular paragraphs
Just write. Like a person. With words.

> Blockquotes
> For when you want to feel literary.

---
Horizontal rules. For dramatic pauses.

  Code blocks
  Indent with two spaces. We'll wrap it in <pre>.
  No syntax highlighting. Your code should speak for itself.

→ https://example.com
External links. The arrow is not optional. Commitment is important.

/~alice/about   Internal links
Three spaces separate the path from the description.

@today
Inline date stamp. Rendered as today's date (YYYY-MM-DD).
The file stays readable in any editor — you just see @today.
```

### Inline Date Stamps

Write `@today` anywhere in a `.gph` or `.txt` file and the server renders it as the current date. Handy for "last updated" lines without manual editing. One magic word, no template engine.

### "Inspired by" Link Convention

Start a post with `← /~maya/phlog/original-post` as the first line and the server renders it as "Inspired by ~maya" with a link. A lightweight citation system — convention, not feature. The server renders the link differently based on position. The writer chooses to credit.

### Guest Author Convention

Name a file `guest-~maya-title.txt` and the server renders it with "Guest post by ~maya" and a link to their burrow. Collaboration without accounts, permissions, or CMS complexity. The filename is the instruction.

### Series Navigation

Name files with a numbered pattern — `part-01.txt`, `part-02.txt`, `chapter_3.txt` — and the server detects the series automatically. Each page shows "Part X of Y" in the meta line and a navigation bar at the bottom:

```
← Part 1    Part 2 of 3    Part 3 →
```

Filesystem convention, not metadata. The filename is the instruction. No config needed. Both `-` and `_` separators work. Requires at least 2 parts to activate.

### Slow Reading Mode

Add `?slow=1` to any text page URL for a more comfortable reading experience:

```
/~alice/phlog/my-post           ← normal (17px, 680px)
/~alice/phlog/my-post?slow=1    ← slow mode (21px, 580px, extra whitespace)
```

No cookie. No setting. No state. Just a URL parameter that injects CSS. The typographic equivalent of a comfortable chair. Share the `?slow=1` link and the receiver gets the same view.

### Hidden Files & Drafts

Files starting with `.` are hidden from directory listings. The `.burrow` config file uses this.

Files and directories starting with `_` are **drafts**: hidden from listings *and* inaccessible via HTTP. Visiting `/~alice/_secret-manifesto` returns a 404, even if the file exists. Rename it to `secret-manifesto.txt` when you're ready to face the world. We won't judge. Much.

### Burrow Anniversary

The server detects the date of your earliest post and shows a subtle "Est. YYYY" badge next to your burrow name in directory listings. No confetti, no fanfare — just a quiet acknowledgment of duration. Celebrates persistence, not popularity.

### Neighbors List

If your burrow shares a ring with other burrows, they appear as "Neighbors" on your burrow root page. Not followers — just people in the same circles. Derived from existing ring data. No social graph. No follow button.

### Anonymous Reader Count

`/~user/stats` shows one number: how many pages were loaded this month. No per-post breakdown. No daily graph. No visitor data. Just: "347 page loads in March 2026." An `AtomicU64` per burrow, reset on the first of the month, persisted to `burrows/.stats` to survive restarts. The minimum viable metric.

### Canonical URL

The server sends a `<link rel="canonical" href="gph://...">` tag in the HTML `<head>` of every page. Search engines know the Burrow version is the source. One tag. Zero config. Invisible to humans. Useful to machines.

### Last-Modified Date

The server shows the file modification date in the meta line of every text page. Readers know how old a text is. The filesystem already knows — the server just shows it.

### `.well-known/` Support

Files in `burrows/.well-known/` are served at `/.well-known/*` per RFC 8615. Standard internet conventions: `security.txt`, `humans.txt`, `webfinger`. The server serves files from a directory. That's what it does.

### OPML Export

Every burrow with a `bookmarks.gph` automatically gets an OPML export at `/~user/subscriptions.opml`. Import your bookmarks into any RSS reader. Standard format. Read-only export. Your data, your reader.

## CONFIGURATION

Configuration lives in `burrow.conf` in the working directory. Generate one with:

```
burrow server init --domain myhole.example.com --port 7070
```

The file looks like this:

```conf
# Burrow server configuration
# Generated by: burrow server init

domain = myhole.example.com
port = 7070
aliases = blog.example.com, old-domain.net
tls_cert = /etc/letsencrypt/live/myhole.example.com/fullchain.pem
tls_key = /etc/letsencrypt/live/myhole.example.com/privkey.pem
gemini_port = 1965
gph_port = 1970
```

We started with two keys. Then reality happened. We're not proud of eight, but we're not adding a ninth. (Narrator: they will.)

### Configuration Reference

| Key | Default | Description |
|-----|---------|-------------|
| `domain` | `localhost` | Your server's primary domain name. Shown in the address bar, breadcrumbs, RSS feeds, and the warm fuzzy feeling of having an identity on the internet. |
| `port` | `7070` | Port to listen on. 7070 because 70 was Gopher's port and we're exactly 100x better. (Citation needed.) Set to `80` for bare HTTP in production — see HOSTING below. |
| `aliases` | *(none)* | Comma-separated list of alternate domain names. The server reads the `Host` header and uses the matching alias for breadcrumbs, feed URLs, and canonical references. Your burrow, many doors. DNS is your problem. |
| `tls_cert` | *(none)* | Path to a PEM certificate chain. If both `tls_cert` and `tls_key` are set, burrowd serves HTTPS. No Caddy. No nginx. Just burrowd and a certificate. |
| `tls_key` | *(none)* | Path to the private key. Keep this file readable only by the burrowd user, or you'll have bigger problems than typography. |
| `gemini_port` | *(none)* | Port for the Gemini protocol listener (typically `1965`). Requires TLS to be configured. Your `.gph` files are automatically converted to Gemtext. Because one protocol is never enough. |
| `gph_port` | *(none)* | Port for the native gph:// protocol listener (typically `1970`). Requires TLS to be configured. Raw `.gph` content with structured `@` metadata. Because three protocols were never enough either. |
| `compression` | `false` | Enable gzip and Brotli compression. Text compresses spectacularly — a 64 KB `.txt` becomes ~8 KB over the wire. Automatic if the client accepts it. Set to `true` and forget about it. |

If `burrow.conf` doesn't exist, burrowd defaults to `localhost:7070`, which is fine for local development and existential crises.

### Custom Domains (Aliases)

Your burrow can answer to multiple domain names. Point DNS to your server, add them to the config:

```conf
domain = phlogosphere.net
aliases = burrow.bruno.be, myburrow.com
```

When someone visits `burrow.bruno.be`, the breadcrumbs say `burrow.bruno.be`, the RSS feed URLs use `burrow.bruno.be`, and the search index references `burrow.bruno.be`. Same content, different door. The `domain` is canonical — aliases are vanity mirrors.

No per-burrow custom domains (yet). No automatic SSL per alias (bring your own wildcard or multi-domain cert). No DNS management. You're an adult. You can edit a zone file.

### Per-Burrow Config (`.burrow`)

Each `~user/` directory (and its subdirectories) can contain a `.burrow` file:

```
description = ITAD, web craft, en te veel meningen over typografie
accent = #d35400
title = Bruno's Hole
sort = modified-desc
pin = about.txt
```

| Key | Description |
|-----|-------------|
| `description` | One-line description shown in directory listings and the sidebar. Keep it short. Think "dating profile bio," not "LinkedIn summary." |
| `accent` | Custom accent color as a hex value (`#abc` or `#aabbcc`). Changes links, icons, buttons, and the progress bar. Your burrow, your vibe. If absent, the color follows the season: spring green, summer gold, autumn brown, winter blue. Four colors per year. The server knows what month it is. |
| `title` | Overrides the directory name in listings and page headings. For when `~bruno` doesn't capture the full scope of your existential ambitions. |
| `sort` | Directory sort order: `name-asc` (default), `name-desc`, `modified-asc`, `modified-desc`. The phlog/ directory probably wants `modified-desc`. The gallery/ probably doesn't care. |
| `pin` | Pin a file to the top of the directory listing, regardless of sort order. The equivalent of taping a note to the fridge. Only one pin per directory — we're minimalists, not Pinterest. |

The `.burrow` file works in subdirectories too. Put one in `phlog/.burrow` with `sort = modified-desc` and your latest post rises to the top. Put one in `gallery/.burrow` with `title = The Gallery` and it shows up with a proper name instead of "gallery/" in the parent listing.

## RUNNING

```bash
# Start the server (from the directory containing burrows/)
burrowd

# What you'll see:
#
#   / burrow v0.9.1
#
#   Tunneling...
#
#   Domain:         myhole.example.com
#   Aliases:        blog.example.com, old-domain.net
#   HTTPS gateway:  http://0.0.0.0:7070
#   Gemini:         gemini://0.0.0.0:1965
#   gph://:         gph://0.0.0.0:1970
#   LAN:            http://192.168.0.42:7070
#   Burrow root:    ./burrows/
#
#   Press Ctrl+C to fill in the hole.
```

The server binds to `0.0.0.0` — all network interfaces. This means it's accessible from your LAN immediately. The startup banner shows your LAN IP for convenience. If you want localhost-only, put a firewall in front of it. We trust you to know what you're doing. (We trust you more than most software does, actually.)

## SECURITY

We take security seriously, even if we don't take much else seriously:

- **Path traversal protection**: All paths are canonicalized and verified to stay inside `burrows/`. Your `../../etc/passwd` jokes won't work here. We've heard them all.
- **HTML escaping**: All user content is escaped before rendering. Both in text content (`<` → `&lt;`) and in URL attributes (`"` → `&quot;`). XSS is not a feature.
- **File size limit**: 64 KB for text files, 2 MB for binary files (images, audio, PDF). If your plaintext file is over 64 KB, you may be writing a novel. We respect that, but we won't serve it. Write a book. Get an editor. Not a text editor — an actual human editor.
- **Depth limit**: Maximum 8 directory levels below your `~user/`. If you need `~alice/a/b/c/d/e/f/g/h/i/`, you don't need Burrow, you need therapy. Or a database. Possibly both.
- **Draft enforcement**: Files and directories starting with `_` or `.` return 404 on HTTP. The filesystem knows they exist, but the server pretends they don't. Method acting, but for web servers.
- **Guestbook rate limiting**: One guestbook post per 30 seconds per IP address. Spammers will need to be very, very patient.
- **Federation ping cap**: Maximum 100 stored pings per server. We remember who mentioned you, but we don't remember *everyone* who mentioned you. Storage has limits. Memory doesn't have to.
- **Content-Security-Policy**: Strict CSP header on all responses — no inline scripts, no external resources, no iframes. The browser refuses everything except what the server itself serves. XSS becomes architecturally impossible, not just escaped.
- **ETag caching**: The server sends ETags based on file modification time. Browsers cache automatically. Conditional GET returns 304 Not Modified when content hasn't changed. Less bandwidth, faster loads, zero configuration.
- **SIGHUP hot-reload**: Send `SIGHUP` to burrowd and it reloads `burrow.conf` and all `.burrow` files without restart. No downtime for config changes. Unix convention.
- **No file uploads**: burrowd serves files. It does not accept them. Content goes in via the CLI or your file manager like a civilized person. (The guestbook form and `/ping` endpoint are the exceptions — one appends text, the other stores a JSON reference. We had meetings about both.)

## GUESTBOOK

Every burrow can have a guestbook. Because it's not the '90s internet without one.

```bash
# Create a guestbook for your burrow
burrow guestbook init
```

This creates a `guestbook.gph` file in your burrow. Visitors can sign it at `/~yourname/guestbook` — there's a form, it works, no JavaScript required. (The form is plain HTML. We checked.)

Entries are stored as plaintext in the file itself:

```
--- Nostalgic Gopher · 2026-03-22 01:56
Your site loads faster than my thoughts. Respect.

--- Anonymous · 2026-03-22 14:30
I haven't seen a guestbook since GeoCities. I'm not crying, you're crying.
```

**Limits** — because the internet is why we can't have nice things:

| Thing | Limit |
|-------|-------|
| Name length | 40 characters |
| Message length | 500 characters |
| Total entries | 200 per guestbook |
| Format injection | `---` in input is replaced with `—` |

View entries from the CLI:

```bash
burrow guestbook show
```

## FEEDS (RSS & Atom)

Every burrow automatically gets both an RSS and an Atom feed. No configuration. No plugins. No "install the RSS extension." It just works, like things used to.

```
/~alice/feed.xml    ← RSS 2.0 feed
/~alice/feed        ← same thing, for the lazy typist
/~alice/atom.xml    ← Atom 1.0 feed
/~alice/atom        ← same thing, for the even lazier typist
```

Both feeds include all `.txt` posts from the `phlog/` directory, sorted newest first, with a content preview. Your favorite feed reader will auto-discover both via `<link rel="alternate">` tags in every page's `<head>`.

The feeds use whatever domain matches the incoming `Host` header (see CUSTOM DOMAINS above). If you're on `localhost`, URLs say `http://localhost:7070`. If you set a domain, they say `https://yourdomain.com`. We assume HTTPS because it's 2026.

## SEARCH (Veronica-NG)

burrowd builds a full-text search index at startup. Every public `.txt` and `.gph` file is tokenized, indexed, and scored using BM25 — the same ranking algorithm your favorite search engine pretends it doesn't use.

```
/search?q=typography         ← basic search
/search?q=author:~bruno      ← filter by author
/search?q=fresh:30 rust      ← posts from the last 30 days about rust
/search?q=type:phlog async   ← only phlog posts about async
```

### Search Operators

| Operator | Example | Effect |
|----------|---------|--------|
| `author:~name` | `author:~bruno` | Only results from that burrow |
| `type:kind` | `type:phlog` | Filter by document type (phlog, page, guestbook, gallery) |
| `fresh:N` | `fresh:7` | Only posts from the last N days |

Results are ranked by BM25 relevance with a freshness boost — recent posts score higher, with a 90-day decay curve. Title matches score 3x body matches. The algorithm is deterministic: same query, same results, for everyone. No filter bubble. No personalization. No "because you liked..."

### Federation Index

The search index is exported at `/search/index.json` — a machine-readable JSON document that other Burrow servers can fetch to include your content in their search results. Voluntary, read-only, and entirely opt-in by the remote server.

## BOOKMARKS

Every burrow can have a public bookmarks page. Create a `bookmarks.gph` file:

```
→ https://100r.co
Hundred Rabbits — off-grid computing

→ /~maya/phlog/on-digital-minimalism
Maya's excellent piece on less

→ https://solar.lowtechmagazine.com
Low-Tech Magazine — the solar-powered website
```

The server renders this at `/~user/bookmarks` with a dedicated bookmarks layout. Bookmark counts are the **only ranking signal** on the Discover page — the more people who bookmark a burrow, the higher it appears in "Most Bookmarked." This is Burrow's answer to likes, stars, and upvotes: public curation by people who care enough to save something.

## GALLERY

Any directory named `gallery/` gets special treatment. Instead of a boring file listing, burrowd renders a grid of ASCII art previews — the first 10 lines of each `.txt` file, in a monospace card layout. Click a piece to see it full-size in a dedicated art viewer with a monospace `<pre>` block.

Put your ASCII art in `~you/gallery/`:

```
burrows/~alice/gallery/
  sunset.txt
  coffee.txt
  maze.txt
```

Each file is just text. The server does the rest. No image formats, no uploads, no media management. The constraint forces creativity. Some of the best art on Burrow will be 40 characters wide and made entirely of `#` signs.

## RINGS (Webrings)

Webrings are back, and they're better than ever. A ring is a curated loop of burrows — members linked in a circle, with navigation arrows at the bottom of every page.

Ring files live in `~user/rings/`:

```
burrows/~alice/rings/deep-web-craft.ring
```

```
title = Deep Web Craft
description = Writers who care about the web as a medium

/~alice
/~bob
gph://tilde.town/~river
ring:~alice/indie-web
```

Members can be local (`/~user`) or remote (`gph://server/~user`). The `ring:~owner/slug` syntax includes all members of another ring — rings of rings, recursively resolved, for when your community has subcommunities.

The server renders ring navigation bars at the bottom of every text page for every ring the burrow belongs to:

```
← Previous    ◎ Deep Web Craft    Next →
```

All rings on the server are listed at `/rings`. The Discover page shows them too.

## DISCOVER

The Discover page (`/discover`) is the homepage of the community. No algorithm. No recommendation engine. Just:

- **Latest posts** — the 10 newest phlog posts across all burrows, chronological
- **Most bookmarked** — burrows ranked by how many other burrows bookmarked them (★ count)
- **Random burrow** — a random spotlight, refreshed on every page load
- **Rings** — all webrings on this server
- **All burrows** — the complete list with descriptions

This is the only "social" page on the server. It exists to help you find things worth reading. It does not track what you click, how long you read, or whether you came back. It shows you what exists. You decide what matters.

### Firehose

`/firehose` is the raw chronological stream — every phlog post from every burrow, newest first, paginated 20 per page. No curation. No ranking. Pure chronology. The anti-algorithm.

## MENTIONS & FEDERATION

### Local Mentions

When you link to another burrow's post in your writing, the server detects it. The linked post displays a "Mentioned by" section at the bottom, listing everyone who referenced it. This is the closest thing Burrow has to notifications — except it's not a notification. Nobody gets pinged. Nobody gets a badge. The mention just... appears, quietly, for anyone who visits the page.

### Federation Pings

When burrowd starts, it scans all posts for `gph://` links to remote servers. For each one, it sends an HTTP POST to the remote server's `/ping` endpoint:

```json
{"source": "https://yourserver.com/~alice/phlog/my-post", "target": "gph://remote.server/~bob/about"}
```

The remote server stores the ping (up to 100 per server) and displays it as a remote mention. No protocol negotiation. No handshake. Just: "hey, I linked to you." The simplest possible federation.

Incoming pings are stored in `burrows/.pings` as a JSON file. They're displayed alongside local mentions on the target post.

## BINARY FILE SERVING

burrowd serves more than text. Images, audio, PDFs, and other binary files are served with correct MIME types and `Cache-Control` headers:

| Extension | MIME Type | Max Size |
|-----------|-----------|----------|
| `.png` `.jpg` `.jpeg` `.gif` `.svg` `.webp` | image/* | 2 MB |
| `.mp3` `.ogg` `.wav` | audio/* | 2 MB |
| `.pdf` | application/pdf | 2 MB |
| `.woff` `.woff2` | font/* | 2 MB |
| `.tar.gz` `.zip` | application/* | 2 MB |

Binary files are served raw — no HTML wrapping, no preview. The browser handles rendering. Text files (`.txt`, `.gph`) still get the full reading-view treatment. The 2 MB limit for binaries is separate from the 64 KB limit for text — because a 64 KB PNG would be one and a half pixels.

## GEMINI BRIDGE

If you configure `gemini_port` and TLS in `burrow.conf`, burrowd speaks the Gemini protocol alongside HTTP(S). Every `.gph` file is automatically converted to Gemtext (`.gmi`):

```conf
gemini_port = 1965
tls_cert = /path/to/cert.pem
tls_key = /path/to/key.pem
```

The Gemini listener serves the same content: home page, directory listings, text files, feeds, and even the discover page — all in Gemtext format. Point any Gemini client at `gemini://yourserver:1965/` and it just works.

The conversion is lossy by design: `.gph` headings become Gemini headings, links become Gemini links, blockquotes stay blockquotes. Code blocks and horizontal rules translate cleanly. What doesn't translate is the typography — Gemini clients bring their own fonts. That's fine. The words are what matter.

## GPH:// NATIVE PROTOCOL

As of v0.9.1, burrowd speaks its own native protocol: `gph://`. TCP+TLS. One request, one response, connection closes. No headers. No status codes. No cookies. No negotiation. The protocol equivalent of a firm handshake.

If Gemini is Gopher's responsible younger sibling, gph:// is the one who read all the same books but formed its own opinions about metadata.

### Configuration

Add one line to `burrow.conf`:

```conf
gph_port = 1970
tls_cert = /path/to/cert.pem
tls_key = /path/to/key.pem
```

Requires TLS (same cert as HTTPS and Gemini — one cert, four protocols). Port 1970 because Gopher was born in 1991 and we wanted something that felt like an origin story.

### Request Format

A single line over TLS:

```
gph://host/path\r\n
```

That's the entire request specification. One line. No headers. No `Accept-Encoding`. No `User-Agent`. No `X-Forwarded-For-The-Love-Of-God-Stop-Adding-Headers`. Just: where do you want to go?

### Response Format

The server responds with a typed document, then closes the connection:

```
=> type
@ metadata_key=value metadata_key2=value2
[content]
```

The first line declares the document type. Optional `@` lines carry metadata. Everything after is raw content in `.gph` markup. The client renders it however it likes — the server doesn't care about your font choices.

### Response Types

| Symbol | Type | Purpose |
|--------|------|---------|
| `/` | `=> directory` | Directory listing |
| `¶` | `=> text` | Raw .gph text content |
| | `=> guestbook` | Guestbook content |
| | `=> bookmarks` | Bookmarks content |
| `?` | `=> search` | Search prompt or results |
| `→` | `=> redirect` | Redirect to another path |
| `∅` | `=> binary` | Binary file metadata |
| | `=> error` | Error message |
| | `=> ok` | Success confirmation |

Nine types. Gopher had nine. HTTP has... we lost count somewhere around `418 I'm a teapot`.

### Metadata Lines (`@` prefix)

Text responses include `@` metadata lines between the type header and content. These are for the client — the server provides the data, the client decides what to show:

```
=> text
@ words=230 read_min=1 modified=2026-03-23 author=~bruno
@ inspired_by=/~maya/phlog/post inspired_author=~maya
@ guest_author=~maya
@ series_current=2 series_total=5 series_prev=/path series_next=/path
@ ring=Deep Web Craft ring_prev=/~maya ring_next=/~river
@ mention=/~maya/phlog/response mention_title=My Response mention_burrow=~maya
# The actual content starts here...
```

Word count, reading time, modification date, series position, ring navigation, mentions — everything the HTTP version computes and bakes into HTML, the gph:// version sends as structured data. The client renders. The server computes. Separation of concerns, as the gods intended.

### Routes

Every route that works over HTTP works over gph://. All of them:

```
gph://host/                        Home page (directory of burrows)
gph://host/~user/path              Any file or directory
gph://host/search?q=rust           Full-text search
gph://host/discover                Discovery page
gph://host/firehose                Chronological post stream
gph://host/rings                   Webring directory
gph://host/servers                 Server directory
gph://host/random                  Random burrow redirect
gph://host/~user/feed              RSS feed
gph://host/~user/stats             Reader count
```

Same content. Different wire format. The server doesn't care which door you came through.

### Guestbook Signing

Write support over gph:// uses query strings — because sometimes you want to leave a mark:

```
gph://host/~user/guestbook?name=Alice&message=Your+server+loads+faster+than+my+thoughts
```

Same rate limits, same truncation, same `---` injection prevention as the HTTP POST form. Just a different way to say hello.

### Design Principles

1. **One line in, complete document out, connection closes.** No keepalive. No streaming. No WebSockets. You ask, you receive, you leave.
2. **No headers, no status codes, no cookies.** The response type line *is* the status. `=> error` means something went wrong. `=> redirect` means go somewhere else. No 301 vs 302 debate. No `Content-Type: application/json; charset=utf-8; boundary=unnecessary`.
3. **The client renders.** The server sends raw content and metadata. Typography, layout, colors — that's the client's job. Your client, your aesthetics.
4. **Metadata in `@` lines.** Structured, parseable, ignoreable. A client that doesn't understand `@ ring=...` can skip it. A client that does can render ring navigation. Progressive enhancement via structured text.
5. **Content is raw `.gph` markup.** Same files, same format, different transport. Write once, serve over HTTP, Gemini, and gph://. Three protocols, one `phlog/` directory.

## THE ADDRESS BAR

The address bar shows `gph://yourdomain.com/path` — and now it means something. The protocol is real. The clients are coming. The bug reports can stop.

## LIMITS

| Thing | Limit | Why |
|-------|-------|-----|
| Text file size | 64 KB | You're writing a phlog, not *War and Peace*. |
| Binary file size | 2 MB | Images and audio get more room. Generosity has limits. |
| Directory depth | 8 levels | After 8 levels you're not organizing, you're nesting |
| Guestbook entries | 200 per burrow | The internet is why we can't have nice things |
| Guestbook rate | 1 post / 30 sec / IP | Spammers hate this one weird trick |
| Federation pings | 100 stored | We remember mentions. Not all of them. |
| Firehose page size | 20 posts | Pagination. It's what adults do. |
| JavaScript | 0 | CSS handles the progress bar. We are proud. Very proud. |
| Frameworks | 0 | This is a feature, not a limitation |
| Databases | 0 | The filesystem is the database. Always has been. |
| Config options | 8 (server) + 5 (burrow) | We started with 2. Reality happened. We kept going. |
| Tests | 42 | Covering render, escaping, XSS, series, dates, compression, ETags |
| POST endpoints | 2 (guestbook + ping) | We agonized over the first. The second was easier. |
| Protocols | 4 (HTTP, HTTPS, Gemini, gph://) | One protocol is never enough. Four is a lifestyle choice. |

## TROUBLESHOOTING

**Q: burrowd says "Address already in use"**
A: Another burrowd is already running. Or something else is on port 7070. Kill it or change the port. We can't help you with your commitment issues.

**Q: I see "phlogosphere.net" everywhere instead of my domain**
A: Run `burrow server init --domain yourdomain.com`. We used to hardcode the domain. We got better.

**Q: My file isn't showing up**
A: Does it start with `.` or `_`? Those are hidden. Is it in the `burrows/~yourname/` directory? Is burrowd running? Have you tried looking at it? Really looking?

**Q: Can I use Markdown?**
A: No. We use our own format. It's simpler. You'll get used to it. Or you won't. Either way, your words will still be there, perfectly readable in any text editor for the next fifty years, long after Markdown has been replaced by whatever comes next.

**Q: Where's the admin panel?**
A: It's called "your terminal." `ls`, `cat`, `vim`. The admin panel that ships with every Unix system since 1971.

**Q: Can I add custom CSS/themes?**
A: You get one knob: `accent = #hexcolor` in your `.burrow` file. It changes the accent color across your entire burrow — links, icons, buttons, that progress bar we're still not sure about. That's it. No custom fonts, no background images, no `marquee` tags. The constraint *is* the design.

## HOSTING — Running in Production

So you've decided to face the internet. Brave. Here's how to take burrowd from `localhost` to a real server. We'll assume a fresh VPS (Hetzner, DigitalOcean, Linode — whatever gives you a public IP and SSH access).

### The Absolute Minimum

```bash
# On your server:
scp burrowd yourserver:/usr/local/bin/
scp -r burrows/ yourserver:/srv/burrow/
ssh yourserver
cd /srv/burrow
burrow server init --domain yoursite.com --port 80
sudo setcap 'cap_net_bind_service=+ep' /usr/local/bin/burrowd
burrowd
```

That's it. Your burrow is live on port 80. No nginx. No Docker. No Kubernetes. No "infrastructure as code." Just a binary, a directory, and a dream.

The `setcap` line is the magic: it grants burrowd permission to bind to ports below 1024 (like 80) without running as root. This is the Unix equivalent of a hall pass. The binary keeps it. No `sudo` needed after that.

### With HTTPS (recommended for the civilized)

```bash
# Get a certificate (Let's Encrypt via certbot)
sudo apt install certbot
sudo certbot certonly --standalone -d yoursite.com

# Configure burrowd
cat > burrow.conf << 'EOF'
domain = yoursite.com
port = 443
tls_cert = /etc/letsencrypt/live/yoursite.com/fullchain.pem
tls_key = /etc/letsencrypt/live/yoursite.com/privkey.pem
EOF

sudo setcap 'cap_net_bind_service=+ep' /usr/local/bin/burrowd
burrowd
```

Now you're serving HTTPS on port 443 with a real certificate. The modern web, powered by four lines of config. You still need to renew the cert every 90 days (`certbot renew` in a cron job). We considered automating this but decided that's certbot's job, not ours.

### With systemd (for servers that outlive your SSH session)

A service file is included (`burrowd.service`):

```bash
sudo cp burrowd.service /etc/systemd/system/
sudo systemctl enable burrowd
sudo systemctl start burrowd
```

It runs as a `burrow` user (create one first: `sudo useradd -r -s /bin/false burrow`), restarts on failure, and sets `RUST_LOG=info` for access logging. Your server survives reboots, SSH disconnections, and mild existential crises.

### With Docker (for people who enjoy layers)

```bash
docker build -t burrow .
docker run -d -p 80:7070 -v /srv/burrow/burrows:/burrow/burrows burrow
```

The Dockerfile is a multi-stage Alpine build. The final image is about 15 MB. That's smaller than most hero images on modern landing pages.

### Port 80 vs. Reverse Proxy

You have two choices, and they're both fine:

| Approach | When to use it |
|----------|---------------|
| **Direct (port 80/443)** | Single site, single binary, you want simplicity. burrowd handles everything. |
| **Reverse proxy** | Multiple sites on one IP, or you want nginx/Caddy to handle TLS termination, HTTP→HTTPS redirect, rate limiting, etc. Put burrowd on port 7070 and proxy to it. |

If you're running just Burrow on a VPS — go direct. If you're running Burrow alongside other things — use a reverse proxy. There is no third option. (There used to be a third option, but it involved Java and we don't talk about it.)

### Access Logging

burrowd logs all HTTP requests via `tower-http` and `tracing`. Control verbosity with `RUST_LOG`:

```bash
RUST_LOG=info burrowd       # default — method, path, status, latency
RUST_LOG=debug burrowd      # verbose — headers, body size, timing
RUST_LOG=warn burrowd       # quiet — errors and warnings only
RUST_LOG=error burrowd      # silent — only errors
```

Logs go to stderr. Redirect to a file if you want history: `burrowd 2>> /var/log/burrow.log`. Or let systemd's journal handle it. We're not your parents.

## THE CLI — burrow(1)

The server's partner in crime. All content management happens here.

### Core

```
burrow init <name>                Create a new ~name/ burrow
burrow new "<title>"              Create a dated phlog post, open $EDITOR
burrow edit <path>                Open a file in $EDITOR
burrow ls [path]                  List burrow contents
burrow status                     Show burrow stats (files, size, latest post)
burrow switch                     List all burrows (← marks the active one)
burrow switch <name>              Switch active burrow
burrow preview <path>             Preview a file in the terminal (gph rendering)
burrow search <query> [--all]     Grep across your burrow (or all burrows)
```

### Social

```
burrow bookmark add <url> -d "description"    Add a public bookmark
burrow bookmark list                          List your bookmarks
burrow bookmark remove <N>                    Remove by number
burrow ring create "<name>" -d "description"  Create a webring
burrow ring add <slug> <member>               Add member (local or gph://)
burrow ring remove <slug> <member>            Remove member
burrow ring show <slug>                       Show ring members
burrow ring list                              List your rings
burrow guestbook init                         Create guestbook.gph
burrow guestbook show                         Display entries in terminal
```

### Archival & Sync

```
burrow export [output.tar.gz]     Backup your burrow as tar.gz
burrow export-static [output/]    Generate complete static HTML site
burrow push <remote>              Push to remote server via rsync/SSH
burrow pull <remote>              Pull from remote server via rsync/SSH
burrow timecapsule [year]         Generate yearly stats summary
burrow colophon                   Generate colophon.txt (stats, rings, metadata)
burrow changelog                  Generate changelog.txt from file mtimes
burrow lint                       Validate .gph files for common errors
burrow import <file.md>           Convert Markdown to .gph format
```

### Reading List (Private)

```
burrow read-later <url> -d "desc" Save a link to read later (private)
burrow reading-list               Show your reading list
```

Your reading list lives in `_reading-list.gph` — prefixed with `_`, so it's invisible to HTTP. Same `.gph` format as bookmarks, but private. Your intentions, for yourself. Edit it with vim, delete it with rm. That's the entire feature.

### Protocol & Server

```
burrow open <gph://url>           Open a gph:// URL (local preview or browser)
burrow register                   Register gph:// protocol handler (macOS/Linux)
burrow server init                Generate burrow.conf
```

`burrow switch` is how you juggle multiple burrows. Running it without arguments shows all burrows with their descriptions and a little `←` arrow pointing at the active one. Running it with a name writes `.burrow-active` and you're done.

If there's only one burrow, the CLI uses it automatically. If there are several and you haven't explicitly switched, it reads `burrows/.burrow-active`. If that doesn't exist, it tells you to pick one. Politely.

## THE BIG PICTURE

Burrow is three things:

1. **A server** (`burrowd`) — you're reading its manual. It serves plaintext content over HTTP, HTTPS, Gemini, and gph://. It has opinions about typography and none about your JavaScript framework.

2. **A CLI** (`burrow`) — creates burrows, writes posts, manages guestbooks. The admin panel that ships with every Unix since 1971, slightly improved.

3. **A protocol** (`gph://`) — the native Burrow wire format. Lighter than HTML, more structured than Gemini. TCP+TLS with typed responses and `@` metadata lines. One request, one document, connection closes.

Together, they form a publishing platform for people who think the internet should be *readable*. No algorithms, no notifications, no engagement metrics, no dark patterns. Text on a screen. Links between pages. People writing for other people.

If you want to understand *why* Burrow exists, read the concept document. If you want to understand *how* it works, keep reading this manual. If you want to understand *whether* it's for you — create a burrow, write something, and see if it feels like coming home.

For the full philosophy, architecture, and roadmap: `/~burrow/concepts/` on any running Burrow server. It reads better in Burrow than in a Markdown file. We're biased.

## SEE ALSO

- `/~burrow/concepts/` — the full vision, served by Burrow itself (meta!)
- `/~burrow/server/` — the server manual, as burrow articles
- `burrow-concept.md` — the vision document in Markdown (for when the server is off)
- `architecture.md` — how the pieces fit together (warning: contains ASCII diagrams)
- `ideas-for-burrow.md` — 50 future feature ideas, each justified against the manifesto
- Your favorite text editor — the real authoring tool
- Your favorite feed reader — for subscribing to `/~user/feed.xml` (RSS) or `/~user/atom.xml` (Atom)
- RFC 1436 — the Gopher protocol spec, for historical context and mild nostalgia

## AUTHORS

Built with Rust, Axum, and a stubborn belief that the internet should be readable.

## COLOPHON

No JavaScript was harmed, employed, or even considered in the making of this server. The scroll progress bar is CSS (`animation-timeline: scroll()`). The guestbook form is plain HTML. The per-burrow theming is a CSS custom property. The entire client-side codebase is zero bytes of executable code. We didn't just minimize JavaScript — we eliminated it. The probation ended. The handler was not promoted. It was replaced. By a stylesheet.

The fonts (JetBrains Mono and Literata) do more heavy lifting than the entire backend. We're okay with that. Typography is important. Your words deserve to look good. Even if you insist on making your accent color `#ff00ff`.

```
        burrowd — because sometimes less is more,
        and sometimes more was never needed.

        Now go write something.
```
