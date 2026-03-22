# Ideas for Burrow

**50 features that respect the manifesto.**

Every idea below follows the sacred constraints: no notifications, no likes, no algorithms, text-first, filesystem-as-database, human curation over automation, privacy by default, ownership by design. If an idea breaks a constraint, it doesn't belong here.

---

## I. Writing & Publishing

### 1. Footnotes (`[1]` syntax)
Footnotes as a separate block at the bottom of the page, with back-links. Fits `.gph` because it doesn't break the reading flow — the footnote sits on its own line, just like links. Academic writing without Markdown complexity.

**Why it fits:** Text-first. No inline formatting. The constraint is the feature.

### 2. Inline date stamp (`@today`)
Write `@today` in your `.gph` and the server renders it as the current date. Handy for "last updated" without manual editing. One magic word, no template engine.

**Why it fits:** One expansion rule. Zero config. The file stays readable in a text editor — you just see `@today`.

### 3. `burrow dictate` — speech-to-text publishing
Pipe audio to whisper.cpp, output to `.txt`. Publishing without a keyboard. Fits "deliberate act of publishing" because you must consciously run the command.

**Why it fits:** CLI-first. Intentional. The output is a plain text file. No cloud transcription service.

### 4. Series numbering — automatic part navigation
Name files `part-01.txt`, `part-02.txt` and the server renders "Part 1 of 5" with prev/next navigation. Longer essays without a single gigantic file.

**Why it fits:** Filesystem convention, not metadata. The filename is the instruction. No config needed.

### 5. Colophon generation — automatic `colophon.txt`
`burrow colophon` generates metadata about your burrow: creation date, total words written, tools used, rings you're a member of. An automatic "about this site" page.

**Why it fits:** Self-documentation. The output is a .txt file you can edit or delete.

### 6. Writing streaks — private, local only
The CLI shows how many consecutive days you've published something. Only visible to you. No badge, no public number. Just: "you've written 12 days in a row." Quiet motivation.

**Why it fits:** No social pressure. No gamification. Private by design. The streak exists in your terminal, nowhere else.

### 7. Word-of-the-day writing prompt
`burrow prompt` gives you a random word or short quote as writing inspiration. No AI generation — a curated list of 1000 words/phrases baked into the binary.

**Why it fits:** Offline. No network call. No personalization. The same binary gives the same prompts to everyone on the same day.

---

## II. Reading & Navigation

### 8. Reading positions remembered (client-side)
The client remembers where you were in a long document. Reopen it → you're where you left off. Like a bookmark in a real book.

**Why it fits:** Local state. No tracking. No server-side session. The client remembers, the server doesn't know.

### 9. `reading-list.gph` — personal reading list
A special file where you put links you want to read later. CLI: `burrow read-later /~maya/phlog/post`. No algorithm, no platform "saved for later" — just a file you manage yourself.

**Why it fits:** It's a file. You own it. You edit it with vim. That's the entire feature.

### 10. Breadcrumb trail as navigation memory
The server remembers your last 5 visited pages in a cookie-less session and shows them as "recent trail" at the bottom. No tracking — pure navigation convenience, expires after 10 minutes.

**Why it fits:** Stateless. No cookies. No user tracking. Ephemeral by design.

### 11. Keyboard navigation in directory listings
`j`/`k` to navigate items, `Enter` to open, `Esc` to go back. Works in the browser via that one JavaScript handler already on probation. Vim users will weep with joy.

**Why it fits:** No additional JavaScript. Same scroll handler, extended with keydown events. The probation continues.

### 12. Length indicator in directory listings
Next to each file, a subtle bar indicating length (short/medium/long). You know before clicking how much reading awaits. Not exact word counts — a feeling.

**Why it fits:** Respects attention. Informs without overwhelming. Three levels, not a number.

### 13. "Slow reading" mode
A URL parameter `?slow=1` that renders the page with extra-large margins, bigger text (21px), and more whitespace. For when you truly want to read, not scan.

**Why it fits:** The typographic equivalent of a comfortable chair. Opt-in. No setting to remember. Just a URL.

---

## III. Social & Discovery

### 14. Neighbors list — automatic suggestions
If your burrow is in the same ring as someone else, you're "neighbors." The server shows a small "Neighbors" block on your profile. Not followers — just: people in the same circles.

**Why it fits:** Derived from existing data (rings). No social graph. No follow button. Just proximity.

### 15. Guest author convention (`guest-~maya-title.txt`)
A filename starting with `guest-~maya-` renders with "Guest post by ~maya" and a link to their burrow. Collaboration without accounts, permissions, or CMS complexity.

**Why it fits:** Filesystem convention. No auth system. No "contributor" role. Just a filename.

### 16. Quiet mentions — ping without notification
The current ping sends an HTTP POST. A "quiet mention" is a link to someone without a ping — for when you reference someone but don't want to nudge them. Syntax: `//~user/path` (double slash = silent link).

**Why it fits:** Respects attention. Not every reference is a conversation starter. Sometimes you just want to cite.

### 17. Seasonal theme — automatic accent color per season
If you don't set an accent, the color follows the season subtly: spring green, summer gold, autumn brown, winter blue. Four colors per year. The server checks the date. No config.

**Why it fits:** Zero config. Gentle. Ephemeral. The server acknowledges the passage of time.

### 18. Burrow anniversary
On the anniversary of your first post, the server shows a subtle "Est. 2026" badge next to your burrow name in directory listings. No confetti, no fanfare — just a quiet acknowledgment of duration.

**Why it fits:** Celebrates persistence, not popularity. No counter. Just a year.

### 19. "Inspired by" link convention
A convention in `.gph`: `← /~maya/phlog/original-post` at the beginning of your post. The server renders it as "Inspired by ~maya" with a link. Lightweight citation system.

**Why it fits:** Convention, not feature. The server just renders the link differently based on position. The writer chooses to credit.

### 20. Anonymous reader count
`/~user/stats` shows one number: how many pages were loaded in total this month. No per-post breakdown, no daily graph, no visitor data. One number. "347 page loads this month." That's enough.

**Why it fits:** The minimum viable metric. No vanity. No dopamine. Just: people are reading.

---

## IV. Federation & Network

### 21. Server directory (`/servers`)
A page listing other known Burrow servers. Manually maintained by the operator in a `servers.conf` file. No automatic discovery — human curation of the network.

**Why it fits:** The operator decides which servers to list. No crawler. No registry. Just a text file.

### 22. Cross-server ring metadata sync
If a ring has remote members, the server periodically fetches their `/~user/.burrow` for description and status. No content sync — only metadata so the ring page stays current.

**Why it fits:** Minimal data. Read-only. The remote server serves a file it already has. No new endpoints.

### 23. Ping digest — weekly summary
Instead of real-time ping notifications (which we don't have, because no notifications): a weekly `_pings-digest.txt` generated by the CLI. `burrow pings` shows who linked to you this week.

**Why it fits:** Pull, not push. You check when you want. The digest is a file you can read, ignore, or delete.

### 24. Server health federation
Servers ping each other periodically with a simple `GET /health`. If a server in your ring is down, the ring navigation shows "(offline)" next to that member.

**Why it fits:** One GET request. Boolean result. No monitoring platform. Just: is it up?

### 25. Webmention compatibility
Accept standard Webmention POST requests alongside the native ping format. WordPress blogs and IndieWeb sites can ping Burrow too. One-way — we receive, we send our own format.

**Why it fits:** Interoperability without complexity. We accept a POST we already understand. No outgoing Webmention.

### 26. OPML export (`/~user/subscriptions.opml`)
Export all your bookmarks as OPML so you can import them into any RSS reader. Burrow → the rest of the internet, not the other way around.

**Why it fits:** Standard format. Read-only export. Your data, your reader. No lock-in.

---

## V. Content & Media

### 27. Colorless syntax highlighting — structural emphasis
Code blocks get no color but gain structure: keywords in **bold**, strings in *italic* (via font-weight/style, no color). Readable in black and white. Fits the "no color" ethos.

**Why it fits:** Typography, not chromatography. The highlight is structural, not decorative. Works in print.

### 28. Inline ASCII diagrams (`~~~` block)
A `~~~` block (triple tilde) that the server renders with a slightly smaller monospace font and a subtle background. For quick diagrams, tables, and schemas that don't need real table markup.

**Why it fits:** One new syntax element. Degrades to plain text. The tilde is already a Burrow character.

### 29. `.gph` → PDF export
`burrow export-pdf phlog/my-post.txt` generates a beautiful PDF with Literata, correct margins, and a colophon. Your words, on paper (digital paper), without the server.

**Why it fits:** Archival. Portability. The PDF is yours. No server needed to read it.

### 30. Audio attachment convention
A file `post.txt` + `post.mp3` in the same directory → the server shows a simple `<audio>` player below the text. No upload system — just two files with the same name. Podcasting for minimalists.

**Why it fits:** Filesystem convention. No upload endpoint. No media management. Two files, same name, done.

### 31. `.bib` reference list
A `references.bib` file in your burrow that the server can render as a bibliography page. BibTeX format, no fancy citation engine. Academics will appreciate this.

**Why it fits:** Standard format. One file. The server renders what's already there.

### 32. Canonical URL header
The server sends a `Link: <gph://...>; rel="canonical"` header. Search engines know the Burrow version is the source, not a scrape. SEO without SEO tools.

**Why it fits:** One HTTP header. Zero config. Invisible to humans. Useful to machines.

---

## VI. CLI & Tooling

### 33. `burrow diff` — compare versions
Shows a diff of a file between your local burrow and the remote server. Know what changed before you push.

**Why it fits:** CLI tool. No version control system required. Just: what's different?

### 34. `burrow lint` — .gph validation
Checks your `.gph` for common errors: broken internal links, lines over 80 characters, missing heading, files over 64KB. No style enforcement — just error detection.

**Why it fits:** Catches mistakes. Doesn't enforce opinions. A spell-checker for structure.

### 35. `burrow stats --global` — server stats via CLI
Fetches `/stats` and `/search/index.json` and displays: burrow count, total words, most active ring, newest post. Your server at a glance.

**Why it fits:** Read-only. Uses endpoints that already exist. One command, one summary.

### 36. `burrow import` — Markdown → .gph conversion
Import a `.md` file and convert to `.gph`. Strips bold/italic (with warning), converts links to `.gph` format, preserves headings. Migration path from other platforms.

**Why it fits:** One-way conversion. The output is a .gph file you own. No ongoing dependency.

### 37. `burrow watch` — live reload
Watches filesystem changes and rebuilds the search index automatically. For development: write, save, refresh browser. No build step but instant updates.

**Why it fits:** Development tool. Watches the filesystem — the thing that's already the database.

### 38. `burrow archive` — WARC export
Export your entire burrow as a WARC file (Web ARChive format). Standard format for web archiving. The Internet Archive can import it directly.

**Why it fits:** Preservation. Standard format. Your burrow, in the format libraries use. Durability as a feature.

---

## VII. Server & Architecture

### 39. ETag caching
The server sends ETags based on file modification time. Browsers and clients cache automatically. Less bandwidth, faster loads, zero configuration.

**Why it fits:** HTTP standard. No config. Invisible improvement. The protocol does the work.

### 40. Conditional GET (`If-Modified-Since`)
The server respects `If-Modified-Since` headers. If the file hasn't changed → 304 Not Modified. Zero bytes over the wire. Perfect for RSS readers that poll.

**Why it fits:** Saves bandwidth for everyone. Standard HTTP. The server already knows the modification time.

### 41. `.well-known/` support
Serve files from a `.well-known/` directory for standard internet protocols: `security.txt`, `humans.txt`, `webfinger`. Interoperability with the rest of the web.

**Why it fits:** Convention, not feature. The server serves files from a directory. That's what it does.

### 42. Content-Security-Policy header
Strict CSP: no inline scripts, no external resources, no iframes. The browser refuses everything except what the server itself serves. XSS becomes architecturally impossible, not just escaped.

**Why it fits:** Defense in depth. One header. The server already serves no JavaScript (except the probationary one). CSP makes it a contract.

### 43. Gzip/Brotli compression
Text compresses spectacularly well. A 64KB `.txt` becomes ~8KB over the wire. The server compresses automatically if the client accepts it.

**Why it fits:** Invisible. Automatic. The fastest page load is fewer bytes. No config needed.

### 44. Static site export
`burrow export-static ./output/` generates a complete static HTML site of your burrow. Host it on GitHub Pages, Netlify, or a USB stick. Your burrow without the server.

**Why it fits:** Ultimate portability. No server lock-in. Your words, as HTML, on any host. The exit strategy that proves there's no lock-in.

### 45. Hot-reloading config
`SIGHUP` to burrowd reloads `burrow.conf` and all `.burrow` files without restart. No downtime for config changes.

**Why it fits:** Unix convention. One signal. No restart dance. The server stays up.

---

## VIII. Archiving & Durability

### 46. Automatic `changelog.txt`
The server maintains a log of when files were added/modified/deleted. Visible at `/~user/changelog`. Transparency: readers see when something was last updated.

**Why it fits:** Honesty. A dated log of changes. No "stealth edits." The filesystem already has modification times — this just makes them visible.

### 47. Version snapshots — manual saves
`burrow snapshot "before the big rewrite"` creates a tar.gz with date and label. Not git — just files at a moment in time. Simpler than version control, more useful than nothing.

**Why it fits:** A file. With a date. And a label. That's the entire version control system.

### 48. Tombstones — gravestones for deleted posts
When you delete a post, optionally leave a `_tombstone-slug.txt` that says: "This post existed from March 2026 to June 2026. The author chose to remove it." Respectful deletion. No 404, no pretending-it-never-existed.

**Why it fits:** Acknowledges that things were written and then un-written. The `_` prefix keeps it hidden from listings but accessible to anyone with the URL. Honest.

### 49. `Last-Modified` as first-class metadata
The server shows the modification date prominently on every page. Readers know how old a text is. In a world of undated content, a date is an act of honesty.

**Why it fits:** The filesystem already knows. The server just shows it. Zero storage cost. Maximum transparency.

### 50. Digital testament (`will.txt`)
A special file containing instructions for what should happen to your burrow if you close your account or the server shuts down: archive on Internet Archive, redirect to another address, or just: "let it disappear." Ownership until the end.

**Why it fits:** Your data. Your decision. Even about the ending. No platform decides what happens to your words — you do.

---

## Prioritization Guide

**Quick wins (< 1 day each):** 2, 12, 17, 18, 32, 39, 40, 42, 43, 45, 49

**Medium effort (1-3 days):** 1, 4, 8, 11, 14, 19, 20, 26, 27, 28, 30, 33, 34, 35, 36, 41, 44, 46

**Larger features (3+ days):** 3, 5, 6, 7, 9, 10, 13, 15, 16, 21, 22, 23, 24, 25, 29, 31, 37, 38, 47, 48, 50

---

*Every idea respects the manifesto: no notifications, no likes, no algorithms, text-first, filesystem-as-database, human curation over automation, privacy by default, ownership by design. If it doesn't fit on a napkin, it doesn't fit in Burrow.*
