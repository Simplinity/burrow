# Newly implemented features

**50 features that respect the manifesto.**

Every idea below follows the sacred constraints: no notifications, no likes, no algorithms, text-first, filesystem-as-database, human curation over automation, privacy by default, ownership by design. If an idea breaks a constraint, it doesn't belong here.



DONE:

### Inline date stamp (`@today`)

Write `@today` in your `.gph` and the server renders it as the current date. Handy for "last updated" without manual editing. One magic word, no template engine.

**Why it fits:** One expansion rule. Zero config. The file stays readable in a text editor — you just see `@today`.

### Series numbering — automatic part navigation

Name files `part-01.txt`, `part-02.txt` and the server renders "Part 1 of 5" with prev/next navigation. Longer essays without a single gigantic file.

**Why it fits:** Filesystem convention, not metadata. The filename is the instruction. No config needed.

### Colophon generation — automatic `colophon.txt`

`burrow colophon` generates metadata about your burrow: creation date, total words written, tools used, rings you're a member of. An automatic "about this site" page.

**Why it fits:** Self-documentation. The output is a .txt file you can edit or delete.

### `reading-list.gph` — personal reading list

A special file where you put links you want to read later. CLI: `burrow read-later /~maya/phlog/post`. No algorithm, no platform "saved for later" — just a file you manage yourself.

**Why it fits:** It's a file. You own it. You edit it with vim. That's the entire feature.

### "Slow reading" mode

A URL parameter `?slow=1` that renders the page with extra-large margins, bigger text (21px), and more whitespace. For when you truly want to read, not scan.

**Why it fits:** The typographic equivalent of a comfortable chair. Opt-in. No setting to remember. Just a URL.

### Seasonal theme — automatic accent color per season

If you don't set an accent, the color follows the season subtly: spring green, summer gold, autumn brown, winter blue. Four colors per year. The server checks the date. No config.

**Why it fits:** Zero config. Gentle. Ephemeral. The server acknowledges the passage of time.

### "Inspired by" link convention

A convention in `.gph`: `← /~maya/phlog/original-post` at the beginning of your post. The server renders it as "Inspired by ~maya" with a link. Lightweight citation system.

**Why it fits:** Convention, not feature. The server just renders the link differently based on position. The writer chooses to credit.

### Server directory (`/servers`)

A page listing other known Burrow servers. Manually maintained by the operator in a `servers.conf` file. No automatic discovery — human curation of the network.

**Why it fits:** The operator decides which servers to list. No crawler. No registry. Just a text file.

### `burrow lint` — .gph validation

Checks your `.gph` for common errors: broken internal links, lines over 80 characters, missing heading, files over 64KB. No style enforcement — just error detection.

**Why it fits:** Catches mistakes. Doesn't enforce opinions. A spell-checker for structure.

### `burrow import` — Markdown to .gph conversion

Import a `.md` file and convert to `.gph`. Strips bold/italic (with warning), converts links to `.gph` format, preserves headings. Migration path from other platforms.

**Why it fits:** One-way conversion. The output is a .gph file you own. No ongoing dependency.

### `.well-known/` support

Serve files from a `.well-known/` directory for standard internet protocols: `security.txt`, `humans.txt`, `webfinger`. Interoperability with the rest of the web.

**Why it fits:** Convention, not feature. The server serves files from a directory. That's what it does.

### Gzip/Brotli compression

Text compresses spectacularly well. A 64KB `.txt` becomes ~8KB over the wire. The server compresses automatically if the client accepts it.

**Why it fits:** Invisible. Automatic. The fastest page load is fewer bytes. No config needed.

### Static site export

`burrow export-static ./output/` generates a complete static HTML site of your burrow. Host it on GitHub Pages, Netlify, or a USB stick. Your burrow without the server.

**Why it fits:** Ultimate portability. No server lock-in. Your words, as HTML, on any host. The exit strategy that proves there's no lock-in.

### Hot-reloading config

`SIGHUP` to burrowd reloads `burrow.conf` and all `.burrow` files without restart. No downtime for config changes.

**Why it fits:** Unix convention. One signal. No restart dance. The server stays up.

### Content-Security-Policy header

Strict CSP: no inline scripts, no external resources, no iframes. The browser refuses everything except what the server itself serves. XSS becomes architecturally impossible, not just escaped.

**Why it fits:** Defense in depth. One header. The server serves zero JavaScript. CSP makes that a contract, not just a promise.

### Digital testament (`will.txt`)

A special file containing instructions for what should happen to your burrow if you close your account or the server shuts down: archive on Internet Archive, redirect to another address, or just: "let it disappear." Ownership until the end.

**Why it fits:** Your data. Your decision. Even about the ending. No platform decides what happens to your words — you do.

### `burrow changelog`

Run manually. Reads `mtime` of all files and generates a `changelog.txt` sorted by date. No watcher. No state. A snapshot at the moment you ask for it. Deliberate.

**Why it fits:** Like `burrow colophon` and `burrow timecapsule` — a snapshot when you ask. Not automatic.

### ETag caching

The server sends ETags based on file modification time. Browsers and clients cache automatically. Less bandwidth, faster loads, zero configuration.

**Why it fits:** HTTP standard. No config. Invisible improvement. The protocol does the work.

### OPML export (`/~user/subscriptions.opml`)

Export all your bookmarks as OPML so you can import them into any RSS reader. Burrow to the rest of the internet, not the other way around.

**Why it fits:** Standard format. Read-only export. Your data, your reader. No lock-in.

### Writing streaks — private, local only

The CLI shows how many consecutive days you've published something. Only visible to you. No badge, no public number. Just: "you've written 12 days in a row." Quiet motivation.

**Why it fits:** No social pressure. No gamification. Private by design. The streak exists in your terminal, nowhere else.

### Neighbors list — automatic suggestions

If your burrow is in the same ring as someone else, you're "neighbors." The server shows a small "Neighbors" block on your profile. Not followers — just: people in the same circles.

**Why it fits:** Derived from existing data (rings). No social graph. No follow button. Just proximity.

### Guest author convention (`guest-~maya-title.txt`)

A filename starting with `guest-~maya-` renders with "Guest post by ~maya" and a link to their burrow. Collaboration without accounts, permissions, or CMS complexity.

**Why it fits:** Filesystem convention. No auth system. No "contributor" role. Just a filename.

### Burrow anniversary

On the anniversary of your first post, the server shows a subtle "Est. 2026" badge next to your burrow name in directory listings. No confetti, no fanfare — just a quiet acknowledgment of duration.

**Why it fits:** Celebrates persistence, not popularity. No counter. Just a year.

### Canonical URL header

The server sends a `Link: <gph://...>; rel="canonical"` header. Search engines know the Burrow version is the source, not a scrape. SEO without SEO tools.

**Why it fits:** One HTTP header. Zero config. Invisible to humans. Useful to machines.

### Last-Modified as first-class metadata

The server shows the modification date prominently on every page. Readers know how old a text is. In a world of undated content, a date is an act of honesty.

**Why it fits:** The filesystem already knows. The server just shows it. Zero storage cost. Maximum transparency.

### Anonymous reader count (`/~user/stats`)

Shows one number: how many pages were loaded this month. No per-post breakdown. No daily graph. No visitor data. Just: "347 page loads in March 2026." The minimum viable metric. No vanity. No dopamine. Just: people are reading. An `AtomicU64` per burrow, reset on the first of the month, persisted to `burrows/.stats` to survive restarts.

**Why it fits:** The absolute minimum. One number. No analytics. No tracking. Just a count.
