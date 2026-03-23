# Burrow

**The internet is bloated. We're digging a way out.**

---

## I. The manifesto

The average web page in 2026 weighs 2.8MB. It loads 47 trackers, 12 ad networks, and a consent banner that's more complex than the content you came for. You scroll through AI-generated slop to find a recipe that's buried under someone's life story, seventeen affiliate links, and a video that autoplays at full volume.

We forgot what the internet was for.

Gopher remembered. In 1991, the University of Minnesota built a protocol so simple it fit on a napkin. No fonts. No layout. No JavaScript. Just directories and text. You asked for something, you got it. The entire transaction took milliseconds. The entire experience took your full attention — because there was nothing else competing for it.

Gopher died because the web was more capable. Fair enough. But "more capable" metastasized into "more extractive." The web didn't just add images and layout — it added surveillance capitalism, infinite scroll, engagement algorithms, dark patterns, and a business model that treats your attention as a commodity to be strip-mined.

Burrow is not a time machine. We're not LARPing the '90s. We're taking the *philosophy* of Gopher — text-first, zero-bloat, human-speed, user-owned — and building it for people who are tired of the modern web but don't want to give up modern UX.

This is the internet you'd build if you started over today, knowing everything we know about what went wrong.

**Burrow is a hole in the ground. It's small, it's warm, and it's yours.**

---

## II. Brand identity

### The name

"Burrow" works on four levels:

1. **Gopher burrows** — a direct lineage to the protocol
2. **To burrow** — to dig beneath the surface, to go underground
3. **A burrow is a home** — small, personal, yours
4. **Counter-culture signal** — while everyone else builds skyscrapers, we dig

The name suggests intimacy, ownership, and deliberate smallness. Nobody brags about the size of their burrow. You just invite people in.

### Voice and tone

Burrow's voice is the exact opposite of startup-speak. No "revolutionizing." No "leveraging synergies." No "delighting users." The tone is:

- **Dry.** Like a Belgian beer — complex underneath, served without fanfare.
- **Honest.** If something doesn't work yet, we say so. We don't say "exciting new feature" when we mean "minimum viable hack."
- **Slightly absurd.** The internet is already absurd. Might as well lean in. Our 404 page says: "This hole leads nowhere. Most holes do. That's the charm."
- **Warm.** Not corporate-warm. Actually warm. Like you're being welcomed into someone's home office that smells like old books and fresh coffee.

Example copy at various touchpoints:

| Where | What it says |
|---|---|
| Homepage | "The internet, minus the parts that made you hate the internet." |
| Signup | "Claim your hole." |
| Empty burrow | "There's nothing here yet. That's the most honest page on the internet." |
| Publish button | "Release into the wild" |
| Error page | "Something broke. Probably not your fault. Probably." |
| Logout | "Lock the door behind you?" |
| Delete account | "Fill in the hole? This is permanent. Your words go back to being thoughts." |
| Loading state | "Tunneling..." |
| First phlog post | "Congratulations. You just published something on the internet without a build step, a deploy pipeline, or a prayer to the CDN gods." |
| Terms of service | "Be a person. Don't pretend to be a lot of people. Don't use this to hurt anyone. That's it. The lawyers made us write more below, but that's the gist." |

### Visual identity

#### Logo

The Burrow logo is a monospace forward-slash inside a rounded square: `/`

That's it. The forward-slash is the universal symbol for directory navigation. It's the most fundamental character in any file system. In Gopher, every experience begins with `/`. In Burrow, your identity *is* a path: `/~bruno`.

The rounded square has a 4px radius — just enough to feel friendly without losing the terminal aesthetic. The slash sits off-center, leaning forward, suggesting movement into a directory. Suggesting going deeper.

Colors: The logo works in exactly two colors at any given time. Primary text color on transparent background. Period. No brand color. No gradient. No "passion purple" or "innovation indigo." The logo inherits the color of wherever it lives. It's a chameleon — just like the protocol.

#### Typography

Two fonts, forever:

- **JetBrains Mono** — for everything UI: navigation, addresses, file names, metadata, the address bar, directory listings. This is the "surface" of Burrow. It says: this is a tool, not a magazine.
- **Literata** — for reading. When you open a phlog post or a text file, the content renders in Literata. Warm, highly legible serif. It says: someone wrote this, and it deserves your attention.

No other fonts. No font-size buffet. Three sizes: 14px for UI, 17px for reading, 13px for metadata. That's the entire type scale.

#### Color system

Burrow has no brand color. The entire UI is built from exactly five values:

```
--surface:    oklch(99% 0 0)       /* almost-white background */
--text:       oklch(15% 0 0)       /* almost-black text */
--muted:      oklch(55% 0 0)       /* secondary text, borders */
--faint:      oklch(92% 0 0)       /* subtle backgrounds */
--accent:     oklch(55% 0.15 160)  /* teal — links, one interactive element */
```

Dark mode inverts `--surface` and `--text`. The accent stays the same. That's the whole theme engine.

Why no brand color? Because brand colors are marketing. They exist to make you *feel* something about a company. Burrow isn't trying to make you feel anything about Burrow. It's trying to get out of the way so you feel something about *the content*.

The only color you'll notice is the teal accent on links and the current-directory indicator. It's the color of deep water — calm, quiet, and slightly mysterious.

#### Iconography

Burrow uses no icons. Zero. Every affordance is communicated through:

- Text labels (`Publish`, `Search`, `Settings`)
- Punctuation as symbols (`/` = directory, `¶` = text, `→` = link, `?` = search, `*` = bookmark)
- Spatial relationships (indentation = hierarchy, position = priority)

Why? Because icons are ambiguous. What does a hamburger menu mean? What does a gear icon do? What's the difference between a share icon and an export icon? Text is never ambiguous. Text is Gopher's first principle. We're keeping it.

#### The anti-brand

Burrow's strongest brand move is having almost no brand. In a world of hyper-designed startups with custom illustration systems, mascots, and brand guidelines documents longer than novels — Burrow's identity is defined by *absence*.

No mascot. No illustrations. No stock photography. No hero images. No "about the team" photos. No logo animation. No brand video. No swag (okay, maybe one t-shirt: plain black, white monospace text: `gph://~you`).

The brand is the experience. If you use Burrow for a week and someone asks what it looks like, the answer should be: "I don't remember what it looks like. I remember what I read."

---

## III. Protocol design

### Why not just use Gopher?

Classic Gopher (RFC 1436) is beautiful but incomplete for 2026:

- No TLS — everything is plaintext over the wire
- No UTF-8 — limited to ASCII
- No authentication — no personal spaces
- No search integration — Veronica died years ago
- Item types are rigid (0=text, 1=directory, 7=search) with no extensibility
- Clients are few and unmaintained

We also can't just use Gemini (the 2019 Gopher successor) because Gemini made an ideological choice to prohibit inline links, images, and any form of client-side interactivity. That's admirable purism. It's also why Gemini has ~5000 users and no growth trajectory.

### Burrow Protocol (gph://)

Burrow extends Gopher's philosophy without breaking its soul. The protocol has three design principles:

1. **The server does the work.** No client-side execution, ever. No JavaScript, no WASM, no plugins. The server sends complete documents. The client renders them. That's it.
2. **Text is the universal fallback.** Every Burrow page can be rendered as plain UTF-8 text with zero formatting. If your client doesn't understand something, it degrades to text. Always.
3. **Identity is a path, not an account.** Your burrow is `gph://hostname/~name`. It's not an account in a database — it's a directory on a server. If you move servers, you take your directory with you. Like moving apartments.

#### Request/Response

A Burrow request is a single line:

```
gph://phlogosphere.net/~bruno/projects/revend/\r\n
```

The server responds with a typed document. The first line is always a type declaration:

```
=> directory
# ~bruno / projects / revend

/  docs/             Documentation and specs
/  src/              Source files  
¶  README.txt        2.1 KB  ·  Mar 14
¶  CHANGELOG.txt     840 B   ·  Mar 10
→  https://revend.co Production site
?  Search this burrow
```

That's a complete response. No headers, no status codes, no content negotiation, no cookies. The connection closes after the response.

#### Document types

Burrow has exactly six document types:

| Symbol | Type | Purpose |
|---|---|---|
| `/` | Directory | A list of items (files, subdirectories, links) |
| `¶` | Text | A plain UTF-8 text document |
| `→` | Link | A redirect to another Burrow address or external URL |
| `?` | Search | An input prompt that submits a query |
| `◊` | Gemtext | A Gemini-formatted document (for cross-protocol compatibility) |
| `∅` | Binary | A raw file download (images, archives, etc.) |

That's it. Six types. Gopher had nine. HTTP has... don't ask.

#### Burrow Markup (`.gph` files)

Text documents can optionally use Burrow Markup — a formatting system so minimal it barely qualifies as markup:

```
# Heading (single level — no h2/h3/h4/h5/h6 hierarchy nonsense)

Regular paragraph text. Just write. Lines wrap at the client's discretion.
There is no bold. There is no italic. If your words need formatting 
to be understood, rewrite them.

> A blockquote, for when someone else said it better.

  Code block — just indent by two spaces.
  fn main() {
      println!("Hello, gopherspace");
  }

--- (horizontal rule — a breath between sections)

/~bruno/reading-list   A link to another burrow page
→ https://example.com  An external link
```

**What's deliberately missing:**

- No bold or italic — because emphasis should come from sentence structure, not decoration
- No inline links — because links deserve their own line and your full attention
- No images — because images are binary content, served separately, intentionally loaded
- No tables — because if you need a table, you need a spreadsheet, not a web page
- No headings hierarchy — because if your document needs six levels of headings, it needs to be six documents
- No custom colors or fonts — because your content isn't your brand
- No metadata/frontmatter — because the filesystem *is* the metadata (filename = title, directory = category, modification date = date)

### Identity and namespace

Every Burrow user gets a tilde address: `gph://server/~name`

The tilde (`~`) prefix is a direct homage to Unix home directories (`/home/~user`) and the tilde clubs of the early web (`sdf.org/~user`). It signals: this is a *person's space*, not a product page.

Your burrow address is your identity. There's no separate "profile page." Your root directory *is* your profile. Want to tell people about yourself? Create an `about.txt`. Want to share your bookmarks? Create a `links.gph`. Want to publish a blog? Create a `phlog/` directory.

#### Burrow addresses as identity

The address `gph://phlogosphere.net/~bruno` carries information:

- `phlogosphere.net` — the server (community) you're on
- `~bruno` — your chosen name within that community

This is like email addresses. `bruno@gmail.com` and `bruno@proton.me` are different contexts, different identities, same person. You can have burrows on multiple servers. You can self-host. The protocol doesn't care.

#### Migration

Your burrow is a directory of files. To migrate between servers:

1. Download your directory (`burrow pull`)
2. Upload to new server (`burrow push`)
3. The old server can optionally redirect (`→ gph://newserver/~bruno`)

No export tools, no API, no "download your data" button that gives you a useless JSON blob. It's files. You copy them. Done.

### The HTTPS gateway

Every Burrow server runs a parallel HTTPS gateway. The address `gph://phlogosphere.net/~bruno/about.txt` is automatically accessible at `https://phlogosphere.net/~bruno/about.txt`.

The gateway renders Burrow content in a minimal HTML wrapper — Literata font, comfortable line-height, no navigation except breadcrumbs. It's a *reading view*, not an "app experience." It looks like a really clean blog.

This is the key to virality: when you share a Burrow link on Twitter, Discord, or WhatsApp, people can read it in their browser. They don't need a client. They don't need to understand the protocol. They just see a clean, fast, beautiful text page that loads in 50ms and has zero ads.

At the bottom of every gateway page, a single line:

> `This page lives on Burrow. Claim your own hole →`

That's the entire conversion funnel.

### The Gemini bridge

Burrow also speaks Gemini. Every burrow is accessible at `gemini://server/~name`. This is important because:

1. Gemini has a small but passionate community. Burrow should embrace them, not compete with them.
2. Gemini clients already exist on every platform.
3. It signals: we're building a protocol, not a walled garden. We talk to other protocols.

The Gemini bridge is automatic. Your `.gph` files are converted to `.gmi` on the fly. Gemini users see your content natively in their clients. They can follow you, link to you, and vice versa.

---

## IV. The app — Burrow client

### Platform strategy

The Burrow client ships as three things:

1. **Desktop app** (macOS, Windows, Linux) — Electron-free. Built with Tauri (Rust backend, web frontend). The entire app is under 8MB. For comparison, Slack is 300MB. The app is a native window with a single webview. It starts in under a second.

2. **CLI tool** (`burrow`) — for publishing and managing your burrow from the terminal. This is the power-user path. `burrow publish phlog/my-post.txt` and it's live. `burrow pull` to backup your entire burrow. `burrow search "rust async"` to search gopherspace from the command line.

3. **Mobile app** (iOS, Android) — a reader-first experience. No publishing from mobile (intentionally). Mobile is for reading, bookmarking, and discovering. Publishing is a deliberate, sit-down act. This is a design statement: we don't want you micro-blogging from the toilet. Write something that's worth writing.

There is no web app. The HTTPS gateway is read-only. If you want to *participate* in Burrow — publish, bookmark, follow rings — you use the native client. This is intentional friction. It filters for people who care enough to install something.

### The reading experience

When you open a text file in Burrow, the entire UI disappears. No sidebar. No toolbar. No address bar. Just text, rendered in Literata at 17px, with comfortable 1.7 line-height and a maximum line width of 65 characters.

The background is warm off-white (`#faf9f7`), not harsh `#ffffff`. The text color is soft almost-black (`#1a1a1a`), not `#000000`. The effect is subtle but visceral: it feels like paper, not a screen.

Navigation is keyboard-first:

| Key | Action |
|---|---|
| `Space` | Scroll down |
| `Shift+Space` | Scroll up |
| `Esc` | Go up one directory |
| `Enter` on a link | Follow link |
| `b` | Toggle bookmarks panel |
| `s` | Open search |
| `/` | Quick-navigate (type a path) |
| `[` / `]` | Previous / next item in directory |
| `g` | Go to address (shows address bar) |

Mouse and touch work fine — but the keyboard shortcuts make Burrow feel like a power tool, not a consumption app.

#### The reading progress indicator

No scroll bar. Instead, a thin 2px line at the very top of the screen fills from left to right as you read. When you reach the end, it pulses once and fades. That's the only UI element visible during reading.

#### Reading time estimation

When you open a text file, a small ghost line appears below the title for exactly 3 seconds: `~4 min read · 847 words`. Then it fades. This respects your attention: you know what you're committing to, but the information doesn't linger and compete with the content.

### Directory browsing

Directories are the heart of Burrow. A directory listing looks like this:

```
/~bruno/projects/

  /  revend/              ITAD platform · B2B SaaS
  /  dotfiles/            Configs, shell scripts
  /  experiments/         Half-baked ideas · Updated 2d ago
  ¶  README.txt           About these projects · 1.2 KB
  ¶  ideas.txt            Running list · 840 B
  →  https://github.com/bruno   My other life
```

Each item shows:
- Type symbol (`/`, `¶`, `→`, `?`)
- Name
- Description (optional, author-written)
- Metadata (size for files, "Updated Xd ago" for active directories)

Directories are clickable and keyboard-navigable. The current selection has a subtle teal left-border. You never feel lost because the breadcrumb trail at the top always shows your full path.

#### Directory cosmetics

Authors can add a `.burrow` config file to any directory to customize its appearance:

```
title = Projects
description = Things I'm building or have built
sort = modified-desc
pin = revend/ README.txt
```

That's it. No themes, no custom CSS, no banner images. You control the title, description, sort order, and which items are pinned to the top. Everything else is the client's responsibility.

### Publishing

#### The phlog

A "phlog" (Gopher blog, from "gopher log") is just a directory of text files:

```
/~bruno/phlog/

  ¶  2026-03-17-why-i-left-medium.txt
  ¶  2026-03-10-itad-market-size.txt
  ¶  2026-02-28-supabase-tips.txt
  ¶  2026-02-14-first-post.txt
```

There's no database. No CMS. No build step. You write a `.txt` or `.gph` file, put it in your phlog directory, and it's published. The filename is the slug. The first line of the file is the title. The modification date is the publication date.

To edit a post, you edit the file. To delete a post, you delete the file. To rearrange posts, you rename the files. It's a filesystem. You already know how to use it.

#### The editor

The Burrow client includes a built-in text editor. It's deliberately minimal:

- Monospace font (JetBrains Mono)
- No formatting toolbar
- No preview pane (what you write is what gets published — there's no rendering step)
- Character count in the bottom-right corner
- Auto-save every 30 seconds to local draft
- `Cmd+Enter` to publish

The editor has exactly one feature beyond a plain textarea: **link completion**. When you type `/~` or `→ `, it offers autocomplete suggestions from your bookmarks and recent visits. This makes cross-linking between burrows effortless.

#### Publishing from the CLI

For people who prefer their own editor:

```bash
# Write in vim, nano, whatever you want
vim ~/phlog/my-new-post.txt

# Publish
burrow push phlog/my-new-post.txt

# Or publish the whole directory
burrow push phlog/
```

The CLI tool handles TLS, authentication, and transfer. It speaks to the Burrow server over a simple PUT extension to the protocol (authenticated, encrypted).

#### Draft visibility

Files prefixed with `_` are drafts. They're visible only to you. `_unfinished-thoughts.txt` exists in your burrow but is invisible to everyone else. Remove the `_` and it's published. No "draft mode" toggle, no publish button — just a filename convention.

### Search — Veronica-NG

Gopher's original search engine was called Veronica (Very Easy Rodent-Oriented Net-wide Index to Computerized Archives — the '90s were wild). Burrow resurrects it.

Veronica-NG is a full-text search engine that crawls all public burrows. It's built into the client — press `s` to search from anywhere.

#### How search works

Every Burrow server voluntarily submits its public content index to the Veronica-NG network (a federation of search nodes, not a single server). The index contains:

- Full text of all public `.txt` and `.gph` files
- Directory structures and descriptions
- Author-set tags (from `.burrow` config)
- Bookmark counts (see Discovery section)

Search results are ranked by:

1. **Text relevance** (BM25 full-text scoring)
2. **Bookmark count** (how many people bookmarked this page)
3. **Freshness** (newer content gets a small boost)
4. **Ring membership** (content that's in curated rings scores slightly higher)

There is no personalization. No "recommended for you." No filter bubble. The same search query returns the same results for everyone. If that means you occasionally discover something outside your comfort zone — good. That's the point.

#### Search operators

```
rust async              → full text search
author:~bruno           → filter by author
server:phlogosphere.net → filter by server  
ring:rust-lang          → filter by ring membership
fresh:7d                → only results from last 7 days
type:phlog              → only phlog posts
```

### Discovery — Rings and signals

#### Rings (the webring revival)

A Ring is a curated collection of burrows, maintained by a person or group. Think of it as a "blogroll on steroids" — not just a list of links, but a navigable loop.

When you're reading a page that belongs to a ring, two small arrows appear at the bottom: `← Previous in ring · Ring: "Deep Web Craft" · Next in ring →`

Clicking through a ring is like flipping through a curated magazine. Each page is a different voice, but someone thought they belonged together. Rings are how you discover new burrows.

Anyone can create a ring:

```
/~bruno/rings/deep-web-craft.ring

title = Deep Web Craft
description = Writers who care about the web as a medium
members =
  gph://sdf.org/~maya
  gph://phlogosphere.net/~kai
  gph://tilde.town/~river
  gph://circumlunar.space/~fox
```

A `.ring` file is just a list of members. To join someone's ring, you ask them (out-of-band — email, DM, carrier pigeon) and they add your address. To leave, they remove it. There's no application process, no algorithm, no approval queue. It's human curation, deliberately.

Rings can be nested: a "Science Writing" ring might include the "Physics" ring and the "Biology" ring. This creates an organic taxonomy — not controlled by anyone, emergent from the community.

#### Public bookmarks

Burrow has one social signal: bookmarks. When you bookmark a page, that bookmark is public by default. You can see anyone's bookmarks at `gph://server/~name/bookmarks/`.

Your bookmarks are a curated collection of things you found valuable. They're your public taste. They function as:

- **A recommendation engine** — if you like someone's writing, check their bookmarks for more good stuff
- **A ranking signal** — Veronica-NG uses bookmark counts for search ranking
- **A social connection** — if two people bookmark the same obscure page, they probably have something in common

Importantly: bookmark counts are never displayed on the bookmarked page itself. The author doesn't see a number. There's no "500 bookmarks!" badge. The count only affects search ranking and is visible in the bookmarker's public list. This prevents the dopamine loop of posting-for-likes. You write because you have something to say, not because you're chasing a number.

#### The firehose

`gph://phlogosphere.net/firehose/` is a real-time stream of all new publications across all burrows on that server. It's the equivalent of "Latest" on Hacker News — unfiltered, unranked, chronological.

The firehose is intentionally overwhelming. It's not meant to be read cover-to-cover. It's meant to be glanced at, to get a sense of what's happening in the community. You pick up patterns: "a lot of people are writing about Rust today" or "there's a cluster of climate posts."

You can filter the firehose by type (`phlog`, `link`, `directory`), but not by topic. Topic filtering is what algorithms do. The firehose is the anti-algorithm.

#### Discover page

`gph://phlogosphere.net/discover/` is a daily curated snapshot:

```
Discover · March 17, 2026

TRENDING IN RINGS
  Deep Web Craft → ~kai wrote about typography as interface
  Rust Belt → ~river published a crate for Gopher protocol parsing
  Thoughtful Tech → ~maya's reflection on digital minimalism

MOST BOOKMARKED (7 days)
  ¶ ~fox/phlog/2026-03-15-why-plaintext-matters.txt     142 bookmarks
  ¶ ~luna/phlog/2026-03-12-leaving-twitter-for-good.txt  98 bookmarks
  ¶ ~sam/phlog/2026-03-16-making-sourdough-in-gph.txt    87 bookmarks

NEW RINGS
  "Homelab Chronicles" by ~atlas — 12 members
  "European Indie Web" by ~bruno — 8 members

RANDOM BURROW
  gph://tilde.town/~iris → "I write about moss, mycology, and mistakes."
```

The "Random Burrow" section is key. Every page load shows a different random burrow. This is the StumbleUpon moment — the unexpected discovery that makes you feel like the internet is still a place of wonder, not a strip mall.

### Social mechanics — the philosophy of less

Burrow deliberately lacks:

- **Followers / following.** You don't follow people. You bookmark their pages. If you want to know when they publish something new, you subscribe to their directory via Burrow's built-in feed system (which is just an auto-updating directory listing — no separate RSS/Atom format needed).
- **Likes / reactions.** No hearts, no thumbs-up, no "🔥". If you appreciate something, bookmark it (public curation) or write a response on your own phlog and link to the original (public conversation).
- **Comments.** There is no comment section. If you have something to say about someone's writing, write about it on your own burrow and link back. This forces you to create something of your own rather than firing off a two-word reaction. It raises the bar for discourse. If your response isn't worth its own page, it's probably not worth saying.
- **Resharing / reposting.** You can't reshare someone's post. You can link to it from your burrow. The difference: a link says "go read this." A reshare says "look at me for finding this." Burrow is about the content, not the curator.
- **Notifications.** Burrow has no notification system. At all. No bell icon. No badge count. No "you have 12 unread." You open Burrow when you want to. You close it when you're done. It never reaches out to pull you back in. Ever.
- **DMs / chat.** Burrow is not a communication tool. It's a publishing tool. If you want to talk to someone, use email. There's probably a `contact.txt` in their burrow with their email address.

What remains after stripping all this away? A reading and writing tool. That's it. You publish things, you read things, you bookmark the good things. The social dynamics emerge from the content, not from engagement features.

### Special features

#### The guestbook

An homage to the early web. Any burrow can have a `guestbook.gph` — a page where visitors can leave a short message (max 500 characters, multi-line). Messages are chronological, newest-last.

```
/~bruno/guestbook.gph

Bruno's guestbook · 12 entries

  ~maya · Mar 15 · "Your ITAD writeup was exactly what I needed. Thank you."
  ~kai · Mar 12 · "Nice burrow. The phlog/projects ratio is perfect."
  ~anonymous · Mar 10 · "Found you through the Deep Web Craft ring. Following."
  
  [Sign the guestbook]
```

Guestbooks are the only interactive element in Burrow beyond search. They're intentionally limited: 500 characters, no formatting, no editing, no deleting (except by the burrow owner), rate-limited to one post per 30 seconds per IP. They're the digital equivalent of signing a visitor's book at a museum — brief, personal, and surprisingly meaningful.

#### The ASCII art gallery

Every burrow can have its own ASCII art gallery. Create a `gallery/` directory in your burrow, add `.txt` files with your art, and the server renders them in a grid preview — the first 10 lines of each piece in monospace cards. Click to view full-size in a dedicated art viewer.

This seems whimsical, but it serves a serious purpose: it establishes that Burrow values *craft* — the careful arrangement of characters to create beauty within constraints. That philosophy extends to everything on the platform. A well-written phlog post is a form of craft. A thoughtfully organized burrow is a form of craft. The constraint of plaintext isn't a limitation — it's a creative medium.

#### Yearly archival and `timecapsule.txt`

`burrow timecapsule` generates a `timecapsule-YYYY.txt` — a snapshot of your burrow's state for a given year. How many posts you wrote, total words, the posts themselves, your bookmarks, your rings. It's a public record of your year in plaintext.

Run it when you want. The CLI reads the filesystem and generates a `.txt` file you can publish, edit, or delete. It's a mirror held up at your discretion: who were you online this year? It's the kind of feature that makes people stay for years.

#### Reading challenges

Burrow supports community reading challenges. A server operator or any user can create one:

```
/~bruno/challenges/100-phlogs-2026.challenge

title = 100 Phlogs 2026
description = Read 100 different phlog posts from 100 different authors
goal = 100
type = unique-authors
start = 2026-01-01
end = 2026-12-31
participants = 47
```

Your client tracks your reading automatically (locally — not reported to any server). When you open the challenge page, it shows your progress. Challenges are voluntary, public, and un-gamified: no leaderboards, no badges, no XP. Just a personal count and the satisfaction of reaching a goal.

#### Burrow-to-burrow ping

When you link to someone's phlog post from your own phlog, their server receives a "ping" — a lightweight notification that someone referenced their work. The author sees these pings in their `.burrow/pings.log`:

```
2026-03-17 gph://phlogosphere.net/~bruno/phlog/2026-03-17-response.txt
  → referenced your phlog/2026-03-15-why-plaintext-matters.txt
```

This is the Burrow equivalent of trackbacks/webmentions, but radically simpler: no spec, no verification, just a log entry. The author can read your response, ignore it, or write a counter-response on their own phlog. The conversation is public, distributed, and nobody owns it.

---

## V. Technical architecture

### Server

The Burrow server is a single Rust binary. It does four things:

1. Serves Gopher protocol requests on port 70
2. Serves Burrow protocol requests on port 1965 (shared with Gemini, with protocol detection)
3. Runs the HTTPS gateway on port 443
4. Maintains the full-text search index for Veronica-NG

The server reads from a filesystem. Each user's burrow is a directory on disk. There's no database for content — the filesystem *is* the database. An SQLite database handles:

- Authentication tokens
- Bookmark counts
- Guestbook entries
- Ping logs
- Search index metadata

A single Burrow server can host thousands of burrows on a $5/month VPS. The protocol is so lightweight that bandwidth is measured in kilobytes, not gigabytes.

#### Self-hosting

Self-hosting a Burrow server is a one-liner:

```bash
curl -fsSL https://get.burrow.sh | sh
burrow server init --domain myblog.example.com
burrow server start
```

The server auto-configures TLS via Let's Encrypt, creates your `~admin` burrow, and starts serving. From zero to running in under a minute.

### Federation model

Burrow servers are independent but interconnected. There's no central server, no federation protocol, no ActivityPub. Instead:

- **Search federation** — servers voluntarily submit their content index to Veronica-NG relay nodes. Any server can run a relay. Multiple relays exist. There's no single search provider.
- **Ping federation** — when you link to a page on another server, your server sends a lightweight HTTP POST to the target server. That's the only server-to-server communication.
- **Ring federation** — rings can span multiple servers. The ring file just contains addresses. No coordination needed.
- **Gateway federation** — every server runs its own HTTPS gateway. No central web frontend.

This is "federation by URL" — the simplest possible model. Servers don't need to know about each other. They just serve content at addresses. Links connect them. Search indexes them. That's it.

### Content limits

Burrow enforces opinionated size limits:

| Limit | Value | Why |
|---|---|---|
| Max file size | 64 KB | If your text is longer than 64KB, write a book. Seriously. |
| Max files per directory | 256 | If you have more than 256 entries, you need subdirectories. |
| Max total burrow size (free tier) | 1 MB | Forces intentionality. Every file earns its place. |
| Max total burrow size (paid) | 100 MB | Plenty for a lifetime of writing. |
| Max guestbook message | 280 chars | A visitor should leave a note, not a review. |
| Max filename length | 64 chars | Keep it readable. |
| Max directory depth | 8 levels | You're organizing a burrow, not a corporate SharePoint. |

These limits are *features*, not restrictions. They create a shared aesthetic. Every burrow on the network is small, fast, and intentional. Nobody's burrow takes 3 seconds to load. Nobody's directory listing scrolls for pages.

---

## VI. Revenue model

### The honest business

Burrow makes money from hosting. That's it. Not advertising, not data, not "premium" features that gatekeep the experience.

#### Free tier

- One burrow per server
- 1 MB total storage
- Full publishing, reading, bookmarking, rings
- HTTPS gateway
- Gemini bridge
- CLI access
- All features

#### Tunnel plan — $3/month or $30/year

- 100 MB storage
- Custom domain support (`gph://myname.com`)
- Priority Veronica-NG indexing
- Email-to-burrow publishing (send an email, it becomes a phlog post)
- Analytics: total page loads per post per month (just the number — no visitor data, no referrers, no geography, no devices)

#### Cavern plan — $8/month or $80/year

- 1 GB storage
- Everything in Tunnel
- Binary file hosting (images, PDFs, archives)
- Multiple burrows on one account
- API access for automation
- Early access to new features

#### Self-hosted — Free, forever

Run your own server. Zero cost to us. We provide the software, documentation, and community support. We don't want to lock you in. If Burrow-the-company disappears tomorrow, Burrow-the-protocol continues.

### Why this works financially

A 1 MB text burrow costs approximately $0.002/month to host and serve. A $3/month subscriber is 1500x profitable per burrow. Even with infrastructure, search indexing, and team costs, the unit economics are absurdly good — because text is tiny and the protocol is efficient.

If 2% of free users convert to Tunnel and 0.5% to Cavern (conservative SaaS benchmarks), a community of 100,000 users generates approximately $12,000/month from Tunnel and $4,000/month from Cavern. That's $192,000/year from 100K users — enough for a small team to run the service indefinitely.

The goal isn't unicorn scale. It's sustainability. Burrow should make enough money to keep the lights on and the team paid, while never being tempted to compromise on the "no ads, no tracking, no algorithmic manipulation" promise.

---

## VII. MVP — the first 90 days

### What ships on day 1

The MVP is brutally scoped. It answers one question: **can 1,000 people publish and read text in a beautiful, fast, zero-bloat environment?**

#### Day 1 features:

1. **The manifest page** — `burrow.sh` is a single HTML page with the manifesto text, a waitlist signup (just email, no frills), and a screenshot of the client
2. **Desktop client** (macOS only, Tauri) — directory browsing, text reading, address bar navigation, keyboard shortcuts, bookmarks (local only)
3. **CLI tool** — `burrow init`, `burrow push`, `burrow pull`, `burrow publish`
4. **Single server** — `phlogosphere.net`, hosting the first 1,000 burrows
5. **Publishing** — text files and basic `.gph` markup
6. **HTTPS gateway** — every burrow readable on the web
7. **Reading experience** — Literata rendering, reading progress, reading time estimate

#### What does NOT ship on day 1:

- Mobile app
- Windows/Linux clients
- Veronica-NG search
- Rings
- Public bookmarks
- Guestbooks
- Gemini bridge
- Federation
- Paid tiers
- Self-hosting documentation

#### Day 1 stack:

```
Server:      Rust (Tokio async runtime)
Protocol:    Custom TCP + TLS 1.3
Gateway:     Axum HTTP server
Storage:     Filesystem + SQLite
Client:      Tauri (Rust + HTML/CSS/JS)
CLI:         Rust (clap)
Index:       Tantivy (Rust full-text search)
Deploy:      Single Hetzner VPS, 4GB RAM, 80GB SSD
Domain:      phlogosphere.net
```

Total infrastructure cost: approximately $15/month.

### Week 1-2: The provocatie

Before the app exists, the manifesto drops. `burrow.sh` goes live with nothing but text and an email signup.

Amplification strategy:
- Post the manifesto as a thread on Twitter/X (screenshot-optimized)
- Submit to Hacker News with title: "We're building a Gopher client for 2026"
- Post in Gemini community spaces (they'll be curious, not hostile, if we do the bridge right)
- Reddit: r/degoogle, r/privacy, r/indieweb, r/minimalism
- One blogger with reach in the "small web" space gets early access and writes about it

Success metric: 3,000 email signups in 14 days.

### Week 3-6: Invite beta

First 1,000 people from the waitlist get access. Each gets a burrow address. Onboarding:

1. Download the client (one click)
2. Log in (magic link, no password)
3. Your burrow exists at `gph://phlogosphere.net/~yourname`
4. The first thing you see: your empty burrow with the message: "There's nothing here yet. That's the most honest page on the internet."
5. A single prompt: "Write your first post →"
6. The editor opens. You type. You hit `Cmd+Enter`. It's live.

That flow — from zero to published — must take under 3 minutes. If it takes longer, we've failed.

### Week 6-12: The creator tools

Based on beta feedback, ship:

- **Public bookmarks** — the first social signal
- **Rings** — let beta users create the first curated collections
- **Veronica-NG** — search across all beta burrows
- **Windows and Linux clients**
- **Guestbooks** — the first interactive element
- **Gemini bridge** — open the door to the Gemini community

### Week 12-16: Growth

- **Mobile app** (iOS first, read-only)
- **Discover page** — trending, most-bookmarked, random burrow
- **Paid tiers** launch (Tunnel and Cavern)
- **Self-hosting documentation** and Docker image

### Week 16-20: Federation

- **Open the protocol spec** — anyone can build a Burrow server
- **Veronica-NG relay federation** — multiple search nodes
- **Ping federation** — cross-server references
- **Custom domains** for paid users

---

## VIII. The long game

Burrow is not trying to replace the web. The web is fine for web applications, e-commerce, video streaming, and interactive tools. Burrow is for the other thing — the thing the web used to be for before it became an application platform: **reading and writing**.

If Burrow succeeds, five years from now:

- 500,000 people have burrows across hundreds of servers
- Writers, researchers, and thinkers use it as their primary publishing platform
- Veronica-NG is the first place people search when they want thoughtful writing instead of SEO-optimized AI slop
- The Burrow protocol is an RFC and multiple independent implementations exist
- The HTTPS gateway means Burrow content ranks in Google — and it ranks *well*, because it loads fast, it's accessible, and it's human-written
- The company is profitable, small (under 10 people), and has never taken venture capital

The biggest risk isn't technical. It's cultural. Burrow has to be *cool enough* for early adopters but *useful enough* for everyone else. It has to be punk but not pretentious. Minimal but not austere. Nostalgic but not backwards.

The answer is in the name. A burrow is small, warm, and yours. That's always been the promise of the personal internet. We just forgot, for a while, what it felt like.

**Now let's start digging.**
