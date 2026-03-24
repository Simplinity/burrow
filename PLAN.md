# Burrow Client — Native Swift/AppKit

## Why Native

Tauri wraps a WebView in HTML. That's a browser pretending not to be a browser. The manifesto says: "The client renders. The server serves text." A WebView renders HTML — that's the server's job. The client should render .gph natively.

**The client is not a browser. It's a protocol client.** Like a mail client renders emails, like a Gemini client renders Gemtext, the Burrow client renders .gph. No HTML. No CSS. No WebView. No JavaScript. Native text rendering with native UI.

## Architecture

```
┌────────────────────────────────────────────────────────────────┐
│                    macOS App (Swift/AppKit)                     │
│                                                                │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │ NSToolbar                                                │  │
│  │  [← Back] [→ Forward] [⌂ Home]  [gph://localhost/~bruno]│  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                │
│  ┌─────────────┬────────────────────────────────────────────┐  │
│  │ NSOutlineView│ NSScrollView + NSTextView                 │  │
│  │ (sidebar)    │ (content — rendered .gph)                 │  │
│  │              │                                           │  │
│  │ BURROWS      │ localhost / ~bruno                        │  │
│  │  / ~bruno/   │ ~bruno/                                   │  │
│  │  / ~burrow/  │ ITAD, web craft, en te veel meningen...   │  │
│  │  / ~maya/    │                                           │  │
│  │              │   DIRECTORIES                             │  │
│  │ EXPLORE      │   /  Phlog/        Thoughts on the...     │  │
│  │  ? Search    │   /  gallery/                             │  │
│  │  ◊ Discover  │   /  projects/     Things I'm building    │  │
│  │  ≡ Firehose  │                                           │  │
│  │  ◎ Rings     │   FILES                                   │  │
│  │  ⊕ Servers   │   ¶  about.txt     About                  │  │
│  │  ↻ Random    │   ¶  now.txt       Now                    │  │
│  │              │                                           │  │
│  └─────────────┴────────────────────────────────────────────┘  │
│                                                                │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │ Status bar: Local · gph:// · burrow v0.9.2    localhost  │  │
│  └──────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────┘
```

## Technology

- **Language:** Swift 5.9+
- **Framework:** AppKit (not SwiftUI — we need precise text control)
- **Text rendering:** NSTextView + NSAttributedString
- **Fonts:** Literata (serif, content) + JetBrains Mono (mono, UI + code)
- **Network:** Foundation URLSessionStreamTask or NIO for raw TCP
- **Storage:** UserDefaults (bookmarks, history) + ~/Library/Application Support/Burrow/
- **Build:** Xcode project or Swift Package Manager
- **Target:** macOS 13+ (Ventura)

## The gph:// Protocol (what the server sends)

### Response Format

Every response starts with `=> TYPE` followed by optional `@ key=value` metadata lines, then content.

### Response Types

#### 1. `=> directory` — directory listing

```
=> directory
@ accent=#d35400
# ~bruno
ITAD, web craft, en te veel meningen over typografie

/	/~bruno/phlog	Phlog/	Thoughts on the web, typography, and digital minimalism	8 items
/	/~bruno/gallery	gallery/		5 items
¶	/~bruno/about.txt	about.txt	About	313 B

## Neighbors

/	/~maya	~maya	Deep Web Craft, Indie Web
```

Fields are TAB-separated: `TYPE\tPATH\tDISPLAY_NAME\tDESCRIPTION\tMETA`

Entry types:
- `/` = directory
- `¶` = text file
- `→` = external link

Sections:
- `# heading` = page title
- Lines after heading before first entry = description/subtitle
- `## Section` = section header (Directories, Files, Neighbors, etc.)

#### 2. `=> text` — text content (.gph markup)

```
=> text
@ words=57 read_min=1 modified=2026-03-17 author=~bruno accent=#d35400
@ ring=Deep Web Craft ring_prev=https://tilde.town/~river ring_next=/~maya
@ inspired_by=/~maya/phlog/post inspired_author=~maya
@ guest_author=~maya

# About

I build things and care about how text looks on screens.
```

Metadata:
- `words` — word count
- `read_min` — estimated reading time in minutes
- `modified` — last modification date (YYYY-MM-DD)
- `author` — burrow author (~user)
- `accent` — hex color for this burrow
- `ring` — ring membership (can appear multiple times) with `ring_prev`/`ring_next`
- `inspired_by` / `inspired_author` — inspiration credit
- `guest_author` — guest post credit
- `series_current` / `series_total` / `series_prev` / `series_next` — series info

Content is raw .gph markup:
- `# Heading` — h1
- `> Quote` — blockquote
- `---` — horizontal rule
- `  indented` — code block (2 spaces)
- `→ https://url` — external link
- `/~user/path   description` — internal link (3 spaces separate path from description)
- `@today` — expands to current date (only outside code blocks)
- Everything else — paragraph

#### 3. `=> guestbook` — guestbook content

```
=> guestbook
@ accent=#d35400

--- Alice · 2026-03-22 14:30
This is great!

--- Bob · 2026-03-21 09:15
Nice site.
```

Same format as a .gph file but with guestbook entries. Client should render entries + a sign form.

#### 4. `=> bookmarks` — bookmarks file

```
=> bookmarks
@ accent=#d35400

→ https://example.com   An interesting site
/~maya/about   Maya's about page
```

Rendered as a list of links with descriptions.

#### 5. `=> search` — search prompt

```
=> search
? Search all burrows
```

The `?` line is a prompt. Client should show a search input field.

#### 6. `=> redirect` — redirect to another path

```
=> redirect
/~maya
```

Client navigates to the target path.

#### 7. `=> error` — error page

```
=> error
Not found
```

Client renders an error message.

#### 8. `=> binary` — binary file metadata

```
=> binary
@ mime=image/png size=45678 path=/~bruno/gallery/photo.png
Binary file. Fetch via HTTPS gateway.
```

Client can fetch the file via HTTPS as fallback, or display the metadata.

## App Structure

```
Burrow/
  Burrow.xcodeproj
  Burrow/
    App/
      AppDelegate.swift          — app lifecycle
      MainWindowController.swift — window setup (toolbar, split view)

    Views/
      SidebarViewController.swift — NSOutlineView with burrows + explore
      ContentViewController.swift — NSScrollView + NSTextView for content
      AddressBarField.swift       — NSTextField subclass for gph:// URLs

    Protocol/
      GphClient.swift            — TCP connection, send request, receive response
      GphResponse.swift          — parsed response (type, metadata, content)
      GphParser.swift            — parse raw text into GphResponse

    Rendering/
      GphRenderer.swift          — .gph markup → NSAttributedString
      DirectoryRenderer.swift    — directory entries → NSAttributedString
      GuestbookRenderer.swift    — guestbook entries → NSAttributedString
      SearchRenderer.swift       — search results → NSAttributedString
      ThemeManager.swift         — colors, fonts, dark mode, seasonal accent

    Navigation/
      NavigationManager.swift    — history stack, back/forward
      BookmarkManager.swift      — local bookmarks, persistence

    Resources/
      Literata-Regular.ttf       — bundled font
      Literata-Italic.ttf
      Literata-Medium.ttf
      JetBrainsMono-Regular.ttf  — bundled font
      JetBrainsMono-Medium.ttf
      Assets.xcassets             — app icon
```

## Rendering Rules

The client renders .gph into NSAttributedString. No HTML. No WebView.

### Typography

| Element | Font | Size | Color |
|---------|------|------|-------|
| Body text | Literata Regular | 16pt | --text |
| Headings | Literata Medium | 22pt | --text |
| Code blocks | JetBrains Mono | 14pt | --text on --faint bg |
| Blockquotes | Literata Italic | 16pt | --muted, left border |
| Links (internal) | JetBrains Mono | 14pt | --accent |
| Links (external) | JetBrains Mono | 14pt | --accent |
| Directory names | JetBrains Mono Medium | 14pt | --text |
| Directory descriptions | JetBrains Mono | 13pt | --muted |
| File sizes / meta | JetBrains Mono | 12pt | --muted |
| Section headers | JetBrains Mono | 10pt uppercase | --muted |
| Sidebar items | JetBrains Mono | 13pt | --muted (--text when active) |
| Status bar | JetBrains Mono | 11pt | --muted |

### Colors (Light Mode)

| Name | Hex |
|------|-----|
| --surface | #faf9f7 |
| --text | #1a1a1a |
| --muted | #737373 |
| --faint | #ececea |
| --accent | #1a8a6a (default, overridden by burrow accent) |

### Colors (Dark Mode)

| Name | Hex |
|------|-----|
| --surface | #161614 |
| --text | #e8e6e1 |
| --muted | #8a8a8a |
| --faint | #222220 |
| --accent | #3ab89a |

### Seasonal Accent (when no burrow accent set)

| Season | Months | Color |
|--------|--------|-------|
| Spring | Mar-May | #2d8a4e (green) |
| Summer | Jun-Aug | #c4841d (gold) |
| Autumn | Sep-Nov | #a0522d (brown) |
| Winter | Dec-Feb | #4a7ab5 (blue) |

### .gph Rendering

For each line in the .gph content:

1. `# Heading` → NSAttributedString with heading font, extra spacing above
2. `> Quote` → indented paragraph with left border (via NSParagraphStyle), italic
3. `---` → horizontal line (draw rect in NSTextView or use attachment)
4. `  code line` → monospace font, faint background (via NSParagraphStyle backgroundColor)
5. `→ https://url` → clickable link with → prefix, accent color
6. `/~user/path   desc` → clickable internal link with / prefix, accent color
7. `@today` → expand to YYYY-MM-DD (only outside code blocks)
8. Empty line → paragraph spacing
9. Everything else → body text paragraph in Literata

Links are clickable via NSTextView delegate (`textView(_:clickedOnLink:at:)`).

### Directory Rendering

Parse TAB-separated entries. Render as:
- Section headers (## DIRECTORIES, ## FILES) in small caps
- Each entry: icon + name + description + meta, properly aligned
- `/` entries: accent-colored `/`, bold name
- `¶` entries: muted `¶`, regular name
- `→` entries: accent-colored `→`, link

### Metadata Rendering

Above the content, show:
- Breadcrumbs: `localhost / ~bruno / about`
- If inspired_by: "Inspired by ~maya" with link
- If guest_author: "Guest post by ~maya" with link
- Reading time: "~1 min read · 57 words"
- Last modified: "Modified: 2026-03-17"

Below the content, show:
- Series navigation: "← Part 2 · Part 3 of 5 · Part 4 →"
- Ring navigation: "← Previous · Ring Name · Next →"

## Native UI Components

### NSToolbar

Items:
1. **Back button** (NSToolbarItem) — standard back arrow, Cmd+[
2. **Forward button** (NSToolbarItem) — standard forward arrow, Cmd+]
3. **Home button** (NSToolbarItem) — house icon, Cmd+Shift+H
4. **Address bar** (NSToolbarItem with NSTextField) — gph:// URL, Cmd+L to focus
5. **Bookmarks button** (NSToolbarItem) — star icon, Cmd+B

### NSSplitView

- Left: sidebar (220pt, collapsible)
- Right: content area (flexible)

### Sidebar (NSOutlineView)

Two sections:
1. **BURROWS** — fetched from root directory listing, shows ~user/ entries
2. **EXPLORE** — hardcoded: Search, Discover, Firehose, Rings, Servers, Random

Each item has an icon (the gph type symbol) and a label. Clicking navigates.

### Content (NSScrollView + NSTextView)

- Read-only NSTextView
- Literata font for prose, JetBrains Mono for code/UI
- Links are clickable (delegate handles navigation)
- Scroll position resets to top on navigation
- No editing capability

### Status Bar (NSView at bottom)

Shows: `Local · gph:// · burrow v0.9.2` on the left, `hostname` on the right.

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Cmd+[ | Back |
| Cmd+] | Forward |
| Cmd+L | Focus address bar |
| Cmd+Shift+H | Home |
| Cmd+B | Toggle bookmarks panel |
| Cmd+F | Find in page (NSTextFinder) |
| Cmd+D | Bookmark current page |
| Cmd+R | Reload current page |
| Escape | Clear address bar / close search |
| Space | Page down (standard NSTextView) |
| Shift+Space | Page up |

## Navigation Flow

1. User types URL in address bar → press Enter
2. `NavigationManager.navigate(url)` called
3. `GphClient.fetch(url)` — opens TCP socket, sends `url\r\n`, reads until EOF
4. `GphParser.parse(response)` — returns `GphResponse` with type, metadata, content
5. Based on response type:
   - `directory` → `DirectoryRenderer.render(response)` → NSAttributedString
   - `text` → `GphRenderer.render(response)` → NSAttributedString
   - `guestbook` → `GuestbookRenderer.render(response)` → NSAttributedString
   - `search` → show search prompt or `SearchRenderer.render(response)`
   - `redirect` → `NavigationManager.navigate(target)`
   - `error` → show error view
6. Set NSTextView.textStorage to the rendered NSAttributedString
7. Update address bar, breadcrumbs, sidebar active state
8. Push URL to history stack

## Server Changes Required

### 1. Protocol documentation
The gph:// protocol is already functional. No structural changes needed.

### 2. Guestbook signing over gph://
Currently guestbook signing uses `?name=X&message=Y` in the URL. This works but is inelegant. Consider a proper write protocol:

```
gph://localhost/~bruno/guestbook?name=Alice&message=Hello
```

This already works — the server parses query parameters. No change needed.

### 3. Search queries
Already works: `gph://localhost/search?q=typography`. No change needed.

### 4. Binary files
The server currently returns metadata only for binary files over gph://. For the native client, we could:
- Stream raw bytes after the header (client detects binary type and handles accordingly)
- Or keep the HTTPS fallback for images

**Decision: HTTPS fallback for now.** Binary rendering in NSTextView via NSTextAttachment is possible but can wait for v2.

### 5. Slow reading mode
The `?slow=1` parameter is HTTP-specific. For the native client, this should be a client-side preference (bigger font size in NSTextView). No server change needed.

### 6. Page counters
The server already counts page loads per burrow for HTTP requests. Add the same counter increment for gph:// requests. **Small server change needed.**

## Build Order

### Phase 1: Window + Protocol
1. Create Xcode project with AppDelegate
2. Set up main window with NSToolbar (back, forward, home, address bar)
3. Set up NSSplitView (sidebar + content)
4. Implement GphClient — TCP connect, send, receive, parse
5. Test: connect to localhost:1970, display raw response in NSTextView

### Phase 2: Rendering
6. Implement GphRenderer — .gph markup → NSAttributedString
7. Implement DirectoryRenderer — directory entries → NSAttributedString
8. Bundle Literata + JetBrains Mono fonts
9. Implement ThemeManager — light/dark mode, accent colors
10. Test: navigate to ~bruno/about, verify typography matches browser

### Phase 3: Navigation
11. Implement NavigationManager — history, back/forward
12. Wire address bar to navigation
13. Wire sidebar clicks to navigation
14. Wire content links to navigation (NSTextView delegate)
15. Implement redirect handling
16. Test: full navigation flow — home → ~bruno → phlog → post → back → forward

### Phase 4: Features
17. Implement sidebar — fetch burrows from root, hardcode explore items
18. Implement search — show NSTextField prompt, submit query, render results
19. Implement guestbook view + sign form
20. Implement bookmarks — Cmd+D to add, Cmd+B to show panel
21. Implement status bar
22. Test: all features match browser functionality

### Phase 5: Polish
23. Seasonal accent colors
24. Inspired-by rendering
25. Guest author rendering
26. Series navigation
27. Ring navigation bars
28. Reading time + word count display
29. Last-modified display
30. Breadcrumbs
31. Dark mode support
32. Error pages
33. Connection timeout handling
34. App icon
35. Test: side-by-side with browser, verify visual parity

## Success Criteria

1. Zero WebView — all rendering via NSTextView + NSAttributedString
2. Zero HTML — no HTML generated or parsed anywhere
3. Zero JavaScript — obviously
4. Native macOS look — NSToolbar, NSSplitView, standard keyboard shortcuts
5. Visual parity — content typography matches the HTTPS gateway (same fonts, same sizes, same colors)
6. Full protocol support — all 8 response types handled
7. Navigation works — address bar, sidebar, content links, back/forward, home
8. Offline graceful — timeout errors shown, app doesn't hang
9. Dark mode — follows system preference
10. Accent colors — per-burrow accent from server metadata
