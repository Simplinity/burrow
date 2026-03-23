use chrono::Datelike;
use crate::{BookmarkEntry, BurrowEntry, EntryType, GalleryPiece, GuestbookEntry, Mention, Ring, SearchResult, ring_neighbors, ring_member_href};

const CSS: &str = r#"
@import url('https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500&family=Literata:ital,wght@0,400;0,500;1,400&display=swap');
:root{--surface:#faf9f7;--text:#1a1a1a;--muted:#737373;--faint:#ececea;--accent:#1a8a6a}
@media(prefers-color-scheme:dark){:root{--surface:#161614;--text:#e8e6e1;--muted:#8a8a8a;--faint:#222220;--accent:#3ab89a}}
*{margin:0;padding:0;box-sizing:border-box}
body{font-family:'JetBrains Mono',monospace;background:var(--surface);color:var(--text);min-height:100vh;display:flex;flex-direction:column}
a{color:var(--accent);text-decoration:none}
a:hover{text-decoration:underline}

.topbar{display:flex;align-items:center;gap:12px;padding:12px 24px;border-bottom:1px solid var(--faint)}
.logo{font-size:15px;font-weight:500;letter-spacing:-0.5px}
.logo span{color:var(--accent)}
.addr-bar{flex:1;background:var(--faint);border-radius:6px;padding:8px 14px;font-size:13px;color:var(--muted)}
.addr-bar .host{color:var(--text);font-weight:500}

.container{display:flex;flex:1}
.sidebar{width:220px;border-right:1px solid var(--faint);padding:16px 0;flex-shrink:0}
.sb-label{font-size:10px;font-weight:500;text-transform:uppercase;letter-spacing:0.8px;color:var(--muted);padding:0 20px;margin:16px 0 8px}
.sb-item{display:flex;align-items:center;gap:8px;padding:7px 20px;font-size:13px;color:var(--muted);transition:background 0.15s}
.sb-item:hover{background:var(--faint);text-decoration:none}
.sb-item.active{color:var(--text);font-weight:500}
.sb-icon{color:var(--accent);width:14px;text-align:center;flex-shrink:0}

.main{flex:1;padding:32px 40px;max-width:740px}
.crumbs{font-size:12px;color:var(--muted);margin-bottom:20px}
h1{font-size:18px;font-weight:500;margin-bottom:4px}
.subtitle{font-size:13px;color:var(--muted);margin-bottom:28px;line-height:1.5}

.entry{display:flex;align-items:center;gap:12px;padding:10px 12px;border-radius:6px;transition:background 0.15s;color:var(--text)}
.entry:hover{background:var(--faint);text-decoration:none}
.entry-type{font-size:14px;width:18px;text-align:center;color:var(--accent);flex-shrink:0}
.entry-type.txt{color:var(--muted)}
.entry-name{font-size:14px;font-weight:500;flex:1}
.entry-desc{font-size:12px;color:var(--muted);flex:2}
.entry-meta{font-size:12px;color:var(--muted);text-align:right;min-width:70px}
.section-label{font-size:10px;font-weight:500;text-transform:uppercase;letter-spacing:0.8px;color:var(--muted);margin:24px 0 8px;padding:0 12px}

.reading{font-family:'Literata',serif;font-size:17px;line-height:1.7;color:var(--text);max-width:600px;margin:0 auto;padding:40px 0}
.reading h1{font-family:'Literata',serif;font-size:24px;margin-bottom:8px}
.reading .meta{font-family:'JetBrains Mono',monospace;font-size:13px;color:var(--muted);margin-bottom:32px}
.reading p{margin-bottom:1.2em}
.reading blockquote{border-left:2px solid var(--faint);padding-left:16px;color:var(--muted);margin:1.2em 0}
.reading pre{background:var(--faint);padding:16px;border-radius:4px;overflow-x:auto;font-family:'JetBrains Mono',monospace;font-size:14px;line-height:1.5;margin:1.2em 0}
.reading a{color:var(--accent)}
.reading hr{border:none;border-top:1px solid var(--faint);margin:2em 0}
.progress{position:fixed;top:0;left:0;width:100%;height:2px;background:var(--accent);z-index:100;opacity:0.6;transform-origin:left;transform:scaleX(0);animation:grow-progress linear;animation-timeline:scroll()}
@keyframes grow-progress{from{transform:scaleX(0)}to{transform:scaleX(1)}}

.statusbar{display:flex;justify-content:space-between;padding:6px 24px;border-top:1px solid var(--faint);font-size:11px;color:var(--muted)}
.banner{text-align:center;padding:12px;font-size:12px;color:var(--muted);border-top:1px solid var(--faint)}
.notfound{text-align:center;padding:80px 24px;font-size:15px;color:var(--muted);line-height:1.8}

@media(max-width:700px){.sidebar{display:none}.main{padding:20px 16px}}
"#;

fn seasonal_accent() -> (&'static str, &'static str) {
    let month = chrono::Local::now().month();
    match month {
        3..=5  => ("#2d8a4e", "#4bc87a"),  // spring — fresh green
        6..=8  => ("#b8860b", "#d4a017"),  // summer — warm gold
        9..=11 => ("#a0522d", "#c46d3d"),  // autumn — sienna brown
        _      => ("#3a6ea5", "#5a9fd4"),  // winter — steel blue
    }
}

fn head_with_accent(title: &str, addr: &str, domain: &str, accent: Option<&str>) -> String {
    let title = html_escape(title);
    let addr = html_escape(addr);
    let domain = html_escape(domain);
    let accent_css = match accent {
        Some(color) => format!("\n<style>:root{{--accent:{}}}@media(prefers-color-scheme:dark){{:root{{--accent:{}}}}}</style>", html_escape(color), html_escape(color)),
        None => {
            let (light, dark) = seasonal_accent();
            format!("\n<style>:root{{--accent:{}}}@media(prefers-color-scheme:dark){{:root{{--accent:{}}}}}</style>", light, dark)
        }
    };
    // Extract burrow name from addr for RSS feed link (e.g. "/~bruno/phlog" → "/~bruno/feed.xml")
    let rss_href = addr.split('/').nth(1)
        .filter(|s| s.starts_with('~'))
        .map(|b| format!("/{}/feed.xml", b))
        .unwrap_or_else(|| "/feed.xml".to_string());
    let atom_href = addr.split('/').nth(1)
        .filter(|s| s.starts_with('~'))
        .map(|b| format!("/{}/atom.xml", b))
        .unwrap_or_else(|| "/atom.xml".to_string());
    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>{title} — Burrow</title>
<style>{CSS}</style>{accent_css}
<link rel="alternate" type="application/rss+xml" title="RSS Feed" href="{rss_href}">
<link rel="alternate" type="application/atom+xml" title="Atom Feed" href="{atom_href}">
<link rel="canonical" href="gph://{domain}{addr}">
</head>
<body>
<div class="topbar">
  <a href="/" class="logo"><span>/</span> burrow</a>
  <div class="addr-bar">
    <span style="color:var(--muted)">gph://</span><span class="host">{domain}</span>{addr}
  </div>
</div>"#)
}

fn head(title: &str, addr: &str, domain: &str) -> String {
    head_with_accent(title, addr, domain, None)
}

fn footer(domain: &str) -> String {
    let domain = html_escape(domain);
    format!(r#"<div class="statusbar">
  <span>Local · TLS 1.3 · burrow v0.1.0</span>
  <span>{domain}</span>
</div>
<div class="banner">This page lives on Burrow. <a href="https://burrow.fyi">Claim your own hole →</a></div>
</body></html>"#)
}

fn sidebar(active: &str, entries: &[BurrowEntry]) -> String {
    let mut html = String::from(r#"<div class="sidebar"><div class="sb-label">Burrows</div>"#);
    for e in entries {
        let cls = if e.path == active { " active" } else { "" };
        html.push_str(&format!(
            r#"<a class="sb-item{cls}" href="{path}"><span class="sb-icon">/</span>{name}</a>"#,
            path = html_escape(&e.path), name = html_escape(&e.name), cls = cls
        ));
    }
    let discover_cls = if active == "/discover" { " active" } else { "" };
    let firehose_cls = if active == "/firehose" { " active" } else { "" };
    let rings_cls = if active == "/rings" { " active" } else { "" };
    let servers_cls = if active == "/servers" { " active" } else { "" };
    let search_cls = if active == "/search" { " active" } else { "" };
    html.push_str(&format!(r#"<div class="sb-label">Explore</div>
<a class="sb-item{search_cls}" href="/search"><span class="sb-icon">?</span>Search</a>
<a class="sb-item{discover_cls}" href="/discover"><span class="sb-icon">◊</span>Discover</a>
<a class="sb-item{firehose_cls}" href="/firehose"><span class="sb-icon">≡</span>Firehose</a>
<a class="sb-item{rings_cls}" href="/rings"><span class="sb-icon">◎</span>Rings</a>
<a class="sb-item{servers_cls}" href="/servers"><span class="sb-icon">⊕</span>Servers</a>
<a class="sb-item" href="/random"><span class="sb-icon">↻</span>Random</a>
</div>"#));
    html
}

fn render_entries(entries: &[BurrowEntry]) -> String {
    let dirs: Vec<_> = entries.iter().filter(|e| e.entry_type == EntryType::Directory).collect();
    let files: Vec<_> = entries.iter().filter(|e| e.entry_type != EntryType::Directory).collect();
    let mut html = String::new();

    if !dirs.is_empty() {
        html.push_str(r#"<div class="section-label">Directories</div>"#);
        for e in &dirs {
            html.push_str(&format!(
                r#"<a class="entry" href="{path}"><span class="entry-type">/</span><span class="entry-name">{name}</span><span class="entry-desc">{desc}</span><span class="entry-meta">{meta}</span></a>"#,
                path = html_escape(&e.path), name = html_escape(&e.name), desc = html_escape(&e.description), meta = html_escape(&e.meta)
            ));
        }
    }
    if !files.is_empty() {
        html.push_str(r#"<div class="section-label">Files</div>"#);
        for e in &files {
            html.push_str(&format!(
                r#"<a class="entry" href="{path}"><span class="entry-type txt">¶</span><span class="entry-name">{name}</span><span class="entry-desc">{desc}</span><span class="entry-meta">{meta}</span></a>"#,
                path = html_escape(&e.path), name = html_escape(&e.name), desc = html_escape(&e.description), meta = html_escape(&e.meta)
            ));
        }
    }
    html
}

pub fn home_page(burrows: &[BurrowEntry], domain: &str) -> String {
    let mut html = head(domain, "/", domain);
    html.push_str(&format!(r#"<div class="container">{}<div class="main">
<div class="crumbs"><a href="/">{}</a> /</div>
<h1>{}</h1>
<div class="subtitle">Community burrow server — {}</div>
{}</div></div>"#,
        sidebar("/", burrows),
        html_escape(domain),
        html_escape(domain),
        if burrows.len() == 1 { "1 burrow".to_string() } else { format!("{} burrows", burrows.len()) },
        render_entries(burrows),
    ));
    html.push_str(&footer(domain));
    html
}

pub fn directory_page(path: &str, title: Option<&str>, entries: &[BurrowEntry], burrows: &[BurrowEntry], domain: &str, accent: Option<&str>) -> String {
    let crumbs = build_crumbs(path, domain);
    let display_title = title.unwrap_or(path);
    let desc = entries.first().map(|_| "").unwrap_or("");
    let addr = format!("/{}", path);

    let mut html = head_with_accent(display_title, &addr, domain, accent);
    html.push_str(&format!(r#"<div class="container">{}<div class="main">
<div class="crumbs">{}</div>
<h1>{}/</h1>
<div class="subtitle">{}</div>
{}</div></div>"#,
        sidebar(&format!("/{}", path.split('/').next().unwrap_or("")), burrows),
        crumbs, html_escape(display_title), desc,
        render_entries(entries),
    ));
    html.push_str(&footer(domain));
    html
}

pub fn directory_page_with_neighbors(path: &str, title: Option<&str>, entries: &[BurrowEntry], neighbors: &[(String, Vec<String>)], burrows: &[BurrowEntry], domain: &str, accent: Option<&str>) -> String {
    let crumbs = build_crumbs(path, domain);
    let display_title = title.unwrap_or(path);
    let desc = entries.first().map(|_| "").unwrap_or("");
    let addr = format!("/{}", path);

    let mut html = head_with_accent(display_title, &addr, domain, accent);
    html.push_str(&format!(r#"<div class="container">{}<div class="main">
<div class="crumbs">{}</div>
<h1>{}/</h1>
<div class="subtitle">{}</div>
{}"#,
        sidebar(&format!("/{}", path.split('/').next().unwrap_or("")), burrows),
        crumbs, html_escape(display_title), desc,
        render_entries(entries),
    ));

    // Neighbors block
    if !neighbors.is_empty() {
        html.push_str(r#"<div style="margin-top:28px;padding-top:20px;border-top:1px solid var(--faint);">"#);
        html.push_str(r#"<div style="font-family:'JetBrains Mono',monospace;font-size:11px;font-weight:500;text-transform:uppercase;letter-spacing:0.8px;color:var(--muted);margin-bottom:12px;">Neighbors</div>"#);
        for (member_path, ring_names) in neighbors {
            let display = member_path.trim_start_matches('/');
            let rings_str = ring_names.join(", ");
            html.push_str(&format!(
                r#"<div style="padding:6px 0;font-family:'JetBrains Mono',monospace;font-size:13px;"><a href="{}">{}</a> <span style="font-size:11px;color:var(--muted);">· {}</span></div>"#,
                html_escape_attr(member_path),
                html_escape(display),
                html_escape(&rings_str),
            ));
        }
        html.push_str("</div>");
    }

    html.push_str("</div></div>");
    html.push_str(&footer(domain));
    html
}

pub fn text_page(path: &str, filename: &str, content: &str, domain: &str, accent: Option<&str>) -> String {
    text_page_with_mentions(path, filename, content, &[], &[], "", domain, accent, None, None)
}

use crate::SeriesInfo;

pub fn text_page_with_mentions(path: &str, filename: &str, content: &str, mentions: &[Mention], rings: &[Ring], current_burrow: &str, domain: &str, accent: Option<&str>, series: Option<&SeriesInfo>, last_modified: Option<&str>) -> String {
    let crumbs = build_crumbs(path, domain);

    // Detect "Inspired by" convention: first non-empty line starting with "← /~"
    let (inspired_by, render_content) = extract_inspired_by(content);

    let words = render_content.split_whitespace().count();
    let read_min = (words as f64 / 230.0).ceil() as usize;
    let rendered = render_gph(&render_content);

    let series_meta = if let Some(s) = series {
        format!(" · Part {} of {}", s.current, s.total)
    } else {
        String::new()
    };

    let mut html = head_with_accent(filename, &format!("/{}", path), domain, accent);
    html.push_str(&format!(r#"<div class="progress"></div>
<div style="max-width:680px;margin:0 auto;padding:0 24px;">
<div class="crumbs" style="margin-top:24px;">{crumbs}</div>"#));

    // Render "Inspired by" block if present
    if let Some((inspired_path, inspired_author)) = &inspired_by {
        html.push_str(&format!(
            r#"<div style="font-family:'JetBrains Mono',monospace;font-size:12px;color:var(--muted);margin-bottom:12px;padding:8px 14px;border-left:3px solid var(--faint);">Inspired by <a href="{}" style="color:var(--accent);">{}</a></div>"#,
            html_escape_attr(inspired_path),
            html_escape(inspired_author)
        ));
    }

    // Guest author convention: filename starting with "guest-~name-"
    if let Some(guest) = extract_guest_author(filename) {
        html.push_str(&format!(
            r#"<div style="font-family:'JetBrains Mono',monospace;font-size:12px;color:var(--muted);margin-bottom:12px;padding:8px 14px;border-left:3px solid var(--accent);">Guest post by <a href="/{}" style="color:var(--accent);font-weight:500;">{}</a></div>"#,
            html_escape_attr(&guest),
            html_escape(&guest)
        ));
    }

    let modified_str = last_modified
        .map(|d| format!(" · modified {}", d))
        .unwrap_or_default();

    html.push_str(&format!(r#"<div class="reading">
<div class="meta">~{read_min} min read · {words} words{series_meta}{modified_str}</div>
{rendered}
</div>"#));

    // Series navigation (← Part 2 · Part 3 of 5 · Part 4 →)
    if let Some(s) = series {
        html.push_str(r#"<div style="margin:24px 0 8px;padding:12px 20px;background:var(--faint);border-radius:8px;font-family:'JetBrains Mono',monospace;display:flex;justify-content:space-between;align-items:center;font-size:13px;">"#);
        if let Some(prev) = &s.prev_path {
            html.push_str(&format!(
                r#"<a href="{}" style="color:var(--accent);text-decoration:none;">← Part {}</a>"#,
                html_escape_attr(prev), s.current - 1
            ));
        } else {
            html.push_str("<span></span>");
        }
        html.push_str(&format!(
            r#"<span style="color:var(--muted);">Part {} of {}</span>"#,
            s.current, s.total
        ));
        if let Some(next) = &s.next_path {
            html.push_str(&format!(
                r#"<a href="{}" style="color:var(--accent);text-decoration:none;">Part {} →</a>"#,
                html_escape_attr(next), s.current + 1
            ));
        } else {
            html.push_str("<span></span>");
        }
        html.push_str("</div>");
    }

    // Ring navigation
    for ring in rings {
        let (prev, next) = ring_neighbors(ring, current_burrow);
        html.push_str(r#"<div style="margin:24px 0 8px;padding:12px 20px;background:var(--faint);border-radius:8px;font-family:'JetBrains Mono',monospace;display:flex;justify-content:space-between;align-items:center;font-size:13px;">"#);
        if let Some(p) = &prev {
            html.push_str(&format!(
                r#"<a href="{}" style="color:var(--accent);text-decoration:none;">← Previous</a>"#,
                html_escape(&ring_member_href(p))
            ));
        } else {
            html.push_str("<span></span>");
        }
        html.push_str(&format!(
            r#"<span style="color:var(--muted);">◎ {}</span>"#,
            html_escape(&ring.title)
        ));
        if let Some(n) = &next {
            html.push_str(&format!(
                r#"<a href="{}" style="color:var(--accent);text-decoration:none;">Next →</a>"#,
                html_escape(&ring_member_href(n))
            ));
        } else {
            html.push_str("<span></span>");
        }
        html.push_str("</div>");
    }

    // Mentions section
    if !mentions.is_empty() {
        html.push_str(r#"<div style="margin:32px 0 24px;padding:16px 20px;background:var(--faint);border-radius:8px;font-family:'JetBrains Mono',monospace;">"#);
        html.push_str(&format!(
            r#"<div style="font-size:11px;font-weight:500;text-transform:uppercase;letter-spacing:0.8px;color:var(--muted);margin-bottom:10px;">Mentioned by · {}</div>"#,
            if mentions.len() == 1 { "1 post".to_string() } else { format!("{} posts", mentions.len()) }
        ));
        for m in mentions {
            html.push_str(&format!(
                r#"<a href="{path}" style="display:flex;align-items:center;gap:8px;padding:4px 0;font-size:13px;color:var(--text);text-decoration:none;"><span style="color:var(--accent);">←</span><span>{title}</span><span style="color:var(--muted);font-size:12px;">({burrow})</span></a>"#,
                path = html_escape(&m.source_path),
                title = html_escape(&m.source_title),
                burrow = html_escape(&m.source_burrow),
            ));
        }
        html.push_str("</div>");
    }

    html.push_str("</div>");
    html.push_str(&footer(domain));
    html
}

pub fn not_found_page(path: &str, domain: &str) -> String {
    let mut html = head("404", &format!("/{}", path), domain);
    html.push_str(r#"<div class="notfound">
<p style="font-size:48px;margin-bottom:16px;">/∅</p>
<p>This hole leads nowhere.<br>Most holes do. That's the charm.</p>
<p style="margin-top:24px;"><a href="/">← Back to the surface</a></p>
</div>"#);
    html.push_str(&footer(domain));
    html
}

pub fn guestbook_page(path: &str, entries: &[GuestbookEntry], domain: &str, accent: Option<&str>) -> String {
    let crumbs = build_crumbs(path, domain);
    let burrow_name = path.split('/').next().unwrap_or(path);

    let mut html = head_with_accent("Guestbook", &format!("/{}", path), domain, accent);
    html.push_str(&format!(r#"<div style="max-width:680px;margin:0 auto;padding:0 24px;">
<div class="crumbs" style="margin-top:24px;">{crumbs}</div>
<div class="reading">
<h1>Guestbook</h1>
<div class="meta">{}'s guestbook · {}</div>

<form method="post" style="margin:24px 0 32px;padding:20px;background:var(--faint);border-radius:8px;">
  <div style="margin-bottom:12px;">
    <label style="font-family:'JetBrains Mono',monospace;font-size:12px;color:var(--muted);display:block;margin-bottom:4px;">Name</label>
    <input name="name" required maxlength="40" placeholder="Anonymous gopher"
      style="width:100%;padding:8px 12px;background:var(--surface);border:1px solid var(--faint);border-radius:4px;font-family:'JetBrains Mono',monospace;font-size:14px;color:var(--text);">
  </div>
  <div style="margin-bottom:12px;">
    <label style="font-family:'JetBrains Mono',monospace;font-size:12px;color:var(--muted);display:block;margin-bottom:4px;">Message</label>
    <textarea name="message" required maxlength="500" rows="3" placeholder="Leave your mark..."
      style="width:100%;padding:8px 12px;background:var(--surface);border:1px solid var(--faint);border-radius:4px;font-family:'JetBrains Mono',monospace;font-size:14px;color:var(--text);resize:vertical;"></textarea>
  </div>
  <button type="submit"
    style="padding:8px 20px;background:var(--accent);color:var(--surface);border:none;border-radius:4px;font-family:'JetBrains Mono',monospace;font-size:13px;font-weight:500;cursor:pointer;">Sign the book</button>
</form>
"#,
        html_escape(burrow_name),
        if entries.len() == 1 { "1 entry".to_string() } else { format!("{} entries", entries.len()) },
    ));

    if entries.is_empty() {
        html.push_str(r#"<p style="color:var(--muted);text-align:center;padding:32px 0;">No entries yet. Be the first to sign!</p>"#);
    } else {
        for entry in entries.iter().rev() {
            html.push_str(&format!(
                r#"<div style="border-bottom:1px solid var(--faint);padding:16px 0;">
<div style="display:flex;justify-content:space-between;margin-bottom:6px;">
  <span style="font-family:'JetBrains Mono',monospace;font-size:13px;font-weight:500;">{}</span>
  <span style="font-family:'JetBrains Mono',monospace;font-size:12px;color:var(--muted);">{}</span>
</div>
<p style="margin:0;">{}</p>
</div>"#,
                html_escape(&entry.name),
                html_escape(&entry.date),
                html_escape(&entry.message),
            ));
        }
    }

    html.push_str("</div></div>");
    html.push_str(&footer(domain));
    html
}

pub fn search_page(query: &str, results: &[SearchResult], burrows: &[BurrowEntry], domain: &str) -> String {
    let mut html = head("Search", "/search", domain);

    // Search-specific CSS
    html.push_str(r#"<style>
.search-box{display:flex;gap:8px;margin-bottom:24px}
.search-input{flex:1;padding:10px 14px;background:var(--faint);border:1px solid var(--faint);border-radius:6px;font-family:'JetBrains Mono',monospace;font-size:14px;color:var(--text)}
.search-input:focus{outline:none;border-color:var(--accent)}
.search-btn{padding:10px 20px;background:var(--accent);color:var(--surface);border:none;border-radius:6px;font-family:'JetBrains Mono',monospace;font-size:13px;font-weight:500;cursor:pointer}
.search-result{padding:12px;border-radius:6px;margin-bottom:8px;transition:background 0.15s}
.search-result:hover{background:var(--faint)}
.sr-title{font-size:14px;font-weight:500;color:var(--accent);text-decoration:none;display:block;margin-bottom:4px}
.sr-title:hover{text-decoration:underline}
.sr-meta{font-size:12px;color:var(--muted);margin-bottom:4px}
.sr-snippet{font-size:13px;color:var(--text);line-height:1.5}
.sr-tag{display:inline-block;font-size:11px;padding:1px 6px;background:var(--faint);border-radius:3px;color:var(--muted);margin-right:4px}
</style>"#);

    html.push_str(&format!(r#"<div class="container">{}<div class="main">
<div class="crumbs"><a href="/">{}</a> / search</div>
<h1>Veronica-NG</h1>
<div class="subtitle">Search across all burrows</div>

<form method="get" action="/search" class="search-box">
  <input type="text" name="q" class="search-input" value="{}" placeholder="Search... (try author:~bruno type:phlog fresh:30)">
  <button type="submit" class="search-btn">Search</button>
</form>"#,
        sidebar("/search", burrows),
        html_escape(domain),
        html_escape(query),
    ));

    if query.is_empty() {
        html.push_str(r#"<div style="color:var(--muted);padding:24px 0;text-align:center;line-height:2;">
<p>Search for anything across all burrows.</p>
<p style="font-size:12px;">Operators: <code>author:~name</code> · <code>type:phlog</code> · <code>type:page</code> · <code>fresh:30</code> (days)</p>
</div>"#);
    } else if results.is_empty() {
        html.push_str(&format!(
            r#"<p style="color:var(--muted);padding:24px 0;text-align:center;">No results for "{}"</p>"#,
            html_escape(query)
        ));
    } else {
        html.push_str(&format!(
            r#"<div class="section-label">{} results</div>"#,
            results.len()
        ));
        for r in results {
            html.push_str(&format!(
                r#"<div class="search-result">
<a class="sr-title" href="{path}">{title}</a>
<div class="sr-meta"><span class="sr-tag">{doc_type}</span>{author}{date}</div>
<div class="sr-snippet">{snippet}</div>
</div>"#,
                path = html_escape(&r.path),
                title = html_escape(if r.title.is_empty() { &r.path } else { &r.title }),
                doc_type = html_escape(&r.doc_type),
                author = html_escape(&r.author),
                date = if r.date.is_empty() { String::new() } else { format!(" · {}", html_escape(&r.date)) },
                snippet = html_escape(&r.snippet),
            ));
        }
    }

    html.push_str("</div></div>");
    html.push_str(&footer(domain));
    html
}

pub fn rings_list_page(rings: &[Ring], burrows: &[BurrowEntry], domain: &str) -> String {
    let mut html = head("Rings", "/rings", domain);
    html.push_str(&format!(r#"<div class="container">{}<div class="main">
<div class="crumbs"><a href="/">{}</a> / rings</div>
<h1>Rings</h1>
<div class="subtitle">Curated webrings — {} rings on this server</div>"#,
        sidebar("/rings", burrows),
        html_escape(domain),
        rings.len(),
    ));

    if rings.is_empty() {
        html.push_str(r#"<p style="color:var(--muted);padding:32px 0;text-align:center;">No rings yet. Create one with <code>burrow ring create "Ring Name"</code></p>"#);
    } else {
        for ring in rings {
            html.push_str(&format!(
                r#"<div style="padding:16px 12px;border-bottom:1px solid var(--faint);">
<div style="display:flex;align-items:center;gap:8px;margin-bottom:4px;">
  <span style="color:var(--accent);">◎</span>
  <span style="font-size:14px;font-weight:500;">{title}</span>
  <span style="font-size:12px;color:var(--muted);">by {owner}</span>
</div>
<div style="font-size:13px;color:var(--muted);margin-bottom:8px;">{desc}</div>
<div style="display:flex;flex-wrap:wrap;gap:6px;">"#,
                title = html_escape(&ring.title),
                owner = html_escape(&ring.owner),
                desc = html_escape(&ring.description),
            ));
            for member in &ring.members {
                let href = ring_member_href(member);
                let display = member.trim_start_matches('/');
                let is_external = member.starts_with("gph://");
                let icon = if is_external { "→" } else { "/" };
                html.push_str(&format!(
                    r#"<a href="{href}" style="font-size:12px;padding:2px 8px;background:var(--faint);border-radius:4px;color:var(--accent);text-decoration:none;">{icon}{display}</a>"#,
                    href = html_escape(&href),
                    icon = icon,
                    display = html_escape(display),
                ));
            }
            html.push_str("</div></div>");
        }
    }

    html.push_str("</div></div>");
    html.push_str(&footer(domain));
    html
}

pub fn servers_page(servers: &[crate::ServerEntry], burrows: &[BurrowEntry], domain: &str) -> String {
    let mut html = head("Servers", "/servers", domain);
    html.push_str(&format!(r#"<div class="container">{}<div class="main">
<div class="crumbs"><a href="/">{}</a> / servers</div>
<h1>Known Servers</h1>
<div class="subtitle">{} known Burrow servers — curated by the operator of this one</div>"#,
        sidebar("/servers", burrows),
        html_escape(domain),
        servers.len(),
    ));

    if servers.is_empty() {
        html.push_str(r#"<p style="color:var(--muted);padding:32px 0;text-align:center;">No known servers listed. Add them to <code>servers.conf</code>.</p>"#);
    } else {
        for server in servers {
            let display_url = server.url.trim_start_matches("gph://").trim_start_matches("https://").trim_end_matches('/');
            let href = if server.url.starts_with("gph://") {
                format!("https://{}", server.url.trim_start_matches("gph://"))
            } else {
                server.url.clone()
            };
            html.push_str(&format!(
                r#"<div style="padding:14px 12px;border-bottom:1px solid var(--faint);">
<div style="display:flex;align-items:center;gap:8px;">
  <span style="color:var(--accent);font-size:14px;">⊕</span>
  <a href="{href}" style="font-size:14px;font-weight:500;">{display}</a>
</div>"#,
                href = html_escape_attr(&href),
                display = html_escape(display_url),
            ));
            if !server.description.is_empty() {
                html.push_str(&format!(
                    r#"<div style="font-size:13px;color:var(--muted);margin-top:4px;padding-left:22px;">{}</div>"#,
                    html_escape(&server.description),
                ));
            }
            html.push_str("</div>");
        }
    }

    html.push_str("</div></div>");
    html.push_str(&footer(domain));
    html
}

pub fn gallery_page(path: &str, pieces: &[GalleryPiece], burrows: &[BurrowEntry], domain: &str, accent: Option<&str>) -> String {
    let crumbs = build_crumbs(path, domain);
    let burrow_name = path.split('/').next().unwrap_or(path);

    let mut html = head_with_accent("Gallery", &format!("/{}", path), domain, accent);

    // Gallery-specific CSS
    html.push_str(r#"<style>
.gallery-grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(280px,1fr));gap:16px;padding:8px 0}
.gallery-card{background:var(--faint);border-radius:8px;overflow:hidden;transition:transform 0.15s,box-shadow 0.15s;text-decoration:none;color:var(--text)}
.gallery-card:hover{transform:translateY(-2px);box-shadow:0 4px 12px rgba(0,0,0,0.15);text-decoration:none}
.gallery-preview{padding:12px;overflow:hidden;height:180px;position:relative}
.gallery-preview pre{font-family:'JetBrains Mono',monospace;font-size:7px;line-height:1.2;color:var(--accent);margin:0;white-space:pre;overflow:hidden}
.gallery-preview::after{content:'';position:absolute;bottom:0;left:0;right:0;height:40px;background:linear-gradient(transparent,var(--faint))}
.gallery-info{padding:10px 12px;border-top:1px solid var(--surface);display:flex;justify-content:space-between;align-items:center}
.gallery-title{font-family:'JetBrains Mono',monospace;font-size:13px;font-weight:500}
.gallery-meta{font-family:'JetBrains Mono',monospace;font-size:11px;color:var(--muted)}
</style>"#);

    html.push_str(&format!(r#"<div class="container">{}<div class="main">
<div class="crumbs">{}</div>
<h1>Gallery</h1>
<div class="subtitle">{}'s ASCII art collection — {} pieces</div>"#,
        sidebar(&format!("/{}", burrow_name), burrows),
        crumbs,
        html_escape(burrow_name),
        pieces.len(),
    ));

    if pieces.is_empty() {
        html.push_str(r#"<p style="color:var(--muted);padding:32px 0;text-align:center;">No art yet. Add <code>.txt</code> files to your <code>gallery/</code> directory.</p>"#);
    } else {
        html.push_str(r#"<div class="gallery-grid">"#);
        for piece in pieces {
            html.push_str(&format!(
                r#"<a class="gallery-card" href="/{path}/{slug}">
<div class="gallery-preview"><pre>{preview}</pre></div>
<div class="gallery-info">
  <span class="gallery-title">{title}</span>
  <span class="gallery-meta">{lines}L · {width}W</span>
</div>
</a>"#,
                path = html_escape(path),
                slug = html_escape(&piece.url_path),
                preview = html_escape(&piece.preview),
                title = html_escape(&piece.title),
                lines = piece.line_count,
                width = piece.max_width,
            ));
        }
        html.push_str("</div>");
    }

    html.push_str("</div></div>");
    html.push_str(&footer(domain));
    html
}

pub fn art_page(path: &str, filename: &str, content: &str, domain: &str, accent: Option<&str>) -> String {
    let crumbs = build_crumbs(path, domain);
    let title = content.lines().next().unwrap_or("")
        .trim_start_matches("# ");
    let line_count = content.lines().count();
    let max_width = content.lines().map(|l| l.len()).max().unwrap_or(0);

    // Strip the title line if it's a heading
    let art_content = if content.starts_with("# ") {
        content.lines().skip(1).collect::<Vec<_>>().join("\n")
    } else {
        content.to_string()
    };

    // Calculate font size: scale down for wider pieces
    let font_size = if max_width > 120 { 6 } else if max_width > 80 { 8 } else if max_width > 60 { 10 } else { 12 };

    let mut html = head_with_accent(filename, &format!("/{}", path), domain, accent);

    html.push_str(&format!(r#"<style>
.art-frame{{max-width:900px;margin:0 auto;padding:0 24px}}
.art-canvas{{background:var(--faint);border-radius:8px;padding:24px;overflow-x:auto;margin:24px 0}}
.art-canvas pre{{font-family:'JetBrains Mono',monospace;font-size:{font_size}px;line-height:1.3;color:var(--accent);margin:0;white-space:pre}}
.art-meta{{display:flex;justify-content:space-between;align-items:center;font-family:'JetBrains Mono',monospace;font-size:12px;color:var(--muted);margin-bottom:24px}}
</style>"#));

    html.push_str(&format!(r#"<div class="art-frame">
<div class="crumbs" style="margin-top:24px;">{crumbs}</div>
<h1 style="margin-top:16px;">{title}</h1>
<div class="art-meta">
  <span>{lines} lines · {width} cols</span>
  <span>{filename}</span>
</div>
<div class="art-canvas"><pre>{art}</pre></div>
</div>"#,
        crumbs = crumbs,
        title = html_escape(title),
        lines = line_count,
        width = max_width,
        filename = html_escape(filename),
        art = html_escape(&art_content),
    ));

    html.push_str(&footer(domain));
    html
}

pub fn bookmarks_page(path: &str, bookmarks: &[BookmarkEntry], domain: &str, accent: Option<&str>) -> String {
    let crumbs = build_crumbs(path, domain);
    let burrow_name = path.split('/').next().unwrap_or(path);

    let mut html = head_with_accent("Bookmarks", &format!("/{}", path), domain, accent);
    html.push_str(&format!(r#"<div style="max-width:680px;margin:0 auto;padding:0 24px;">
<div class="crumbs" style="margin-top:24px;">{crumbs}</div>
<div class="reading">
<h1>Bookmarks</h1>
<div class="meta">{}'s bookmarks · {}</div>
"#,
        html_escape(burrow_name),
        if bookmarks.len() == 1 { "1 link".to_string() } else { format!("{} links", bookmarks.len()) },
    ));

    if bookmarks.is_empty() {
        html.push_str(r#"<p style="color:var(--muted);text-align:center;padding:32px 0;">No bookmarks yet.</p>"#);
    } else {
        for entry in bookmarks {
            let icon = if entry.is_external { "→" } else { "/" };
            html.push_str(&format!(
                r#"<div style="border-bottom:1px solid var(--faint);padding:12px 0;">
<div style="display:flex;align-items:center;gap:8px;">
  <span style="color:var(--accent);font-size:14px;width:16px;text-align:center;">{icon}</span>
  <a href="{url}" style="font-size:14px;font-weight:500;">{url_display}</a>
</div>
{desc_html}
</div>"#,
                icon = html_escape(icon),
                url = html_escape_attr(&entry.url),
                url_display = html_escape(&entry.url),
                desc_html = if entry.description.is_empty() {
                    String::new()
                } else {
                    format!(r#"<div style="padding-left:24px;font-size:13px;color:var(--muted);margin-top:4px;">{}</div>"#,
                        html_escape(&entry.description))
                },
            ));
        }
    }

    html.push_str("</div></div>");
    html.push_str(&footer(domain));
    html
}

fn build_crumbs(path: &str, domain: &str) -> String {
    let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
    let mut html = format!(r#"<a href="/">{}</a>"#, html_escape(domain));
    let mut acc = String::new();
    for part in &parts {
        acc.push_str(&format!("/{}", part));
        html.push_str(&format!(r#" / <a href="{}">{}</a>"#, html_escape(&acc), html_escape(part)));
    }
    html
}

/// Extract "Inspired by" convention from content.
/// If the first non-empty line starts with "← /", it's an inspiration link.
/// Returns (Some((path, display_name)), remaining_content) or (None, original_content).
pub fn extract_inspired_by(content: &str) -> (Option<(String, String)>, String) {
    for (i, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        // First non-empty line — check if it starts with "← /"
        if let Some(rest) = line.trim().strip_prefix("← ") {
            let path = rest.trim();
            if path.starts_with('/') {
                // Extract author from path: /~maya/phlog/post → ~maya
                let author = path.split('/')
                    .find(|s| s.starts_with('~'))
                    .unwrap_or(path)
                    .to_string();
                // Remove this line from content
                let remaining: Vec<&str> = content.lines()
                    .enumerate()
                    .filter(|(j, _)| *j != i)
                    .map(|(_, l)| l)
                    .collect();
                return (Some((path.to_string(), author)), remaining.join("\n"));
            }
        }
        // First non-empty line doesn't match — no inspired-by
        break;
    }
    (None, content.to_string())
}

/// Extract guest author from filename convention: guest-~maya-title.txt → Some("~maya")
pub fn extract_guest_author(filename: &str) -> Option<String> {
    let name = filename.trim_end_matches(".txt").trim_end_matches(".gph");
    if let Some(rest) = name.strip_prefix("guest-") {
        // Find the author: ~name up to the next -
        if rest.starts_with('~') {
            let author_end = rest[1..].find('-').map(|i| i + 1).unwrap_or(rest.len());
            let author = &rest[..author_end];
            if !author.is_empty() {
                return Some(author.to_string());
            }
        }
    }
    None
}

pub fn render_gph(content: &str) -> String {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let mut html = String::new();
    let mut in_code = false;

    for raw_line in content.lines() {
        // Only expand @today outside code blocks
        let owned;
        let line = if !in_code && !raw_line.starts_with("  ") {
            owned = raw_line.replace("@today", &today);
            owned.as_str()
        } else {
            raw_line
        };
        if !in_code && line.starts_with("  ") {
            html.push_str("<pre>");
            html.push_str(&html_escape(line.trim_start()));
            html.push('\n');
            in_code = true;
            continue;
        }
        if in_code {
            if line.starts_with("  ") || line.is_empty() {
                html.push_str(&html_escape(if line.is_empty() { "" } else { line.trim_start() }));
                html.push('\n');
                continue;
            } else {
                html.push_str("</pre>");
                in_code = false;
            }
        }

        if let Some(heading) = line.strip_prefix("# ") {
            html.push_str(&format!("<h1>{}</h1>", html_escape(heading)));
        } else if let Some(quote) = line.strip_prefix("> ") {
            html.push_str(&format!("<blockquote><p>{}</p></blockquote>", html_escape(quote)));
        } else if line == "---" {
            html.push_str("<hr>");
        } else if let Some(rest) = line.strip_prefix("→ ") {
            let url = rest.trim();
            html.push_str(&format!(
                r#"<p><a href="{}">{} {}</a></p>"#,
                html_escape_attr(url), html_escape("→"), html_escape(url)
            ));
        } else if line.starts_with("/~") {
            let parts: Vec<&str> = line.splitn(2, "   ").collect();
            let link = parts[0].trim();
            let desc = parts.get(1).unwrap_or(&"");
            html.push_str(&format!(
                r#"<p><a href="{}">{}</a> {}</p>"#,
                html_escape_attr(link), html_escape(link), html_escape(desc)
            ));
        } else if line.is_empty() {
            // skip
        } else {
            html.push_str(&format!("<p>{}</p>", html_escape(line)));
        }
    }
    if in_code {
        html.push_str("</pre>");
    }
    html
}

pub fn firehose_page(posts: &[(String, String, String, String)], burrows: &[BurrowEntry], domain: &str, prev_page: Option<usize>, next_page: Option<usize>) -> String {
    // posts: Vec<(date, title, burrow_name, url_path)>
    let mut html = head("Firehose", "/firehose", domain);
    html.push_str(&format!(r#"<div class="container">{}<div class="main">
<div class="crumbs"><a href="/">{}</a> / firehose</div>
<h1>Firehose</h1>
<div class="subtitle">All recent posts across all burrows</div>"#,
        sidebar("/firehose", burrows),
        html_escape(domain),
    ));

    if posts.is_empty() {
        html.push_str(r#"<p style="color:var(--muted);padding:32px 0;">No posts yet.</p>"#);
    } else {
        html.push_str(r#"<div class="section-label">Recent posts</div>"#);
        for (date, title, burrow, url_path) in posts {
            html.push_str(&format!(
                r#"<a class="entry" href="{path}"><span class="entry-type txt">¶</span><span class="entry-name">{title}</span><span class="entry-desc">{burrow}</span><span class="entry-meta">{date}</span></a>"#,
                path = html_escape(url_path),
                title = html_escape(title),
                burrow = html_escape(burrow),
                date = html_escape(date),
            ));
        }
    }

    // Pagination links
    if prev_page.is_some() || next_page.is_some() {
        html.push_str(r#"<div style="display:flex;justify-content:space-between;padding:20px 12px;font-size:13px;">"#);
        if let Some(p) = prev_page {
            html.push_str(&format!(r#"<a href="/firehose?page={}">← Newer</a>"#, p));
        } else {
            html.push_str("<span></span>");
        }
        if let Some(p) = next_page {
            html.push_str(&format!(r#"<a href="/firehose?page={}">Older →</a>"#, p));
        } else {
            html.push_str("<span></span>");
        }
        html.push_str("</div>");
    }

    html.push_str("</div></div>");
    html.push_str(&footer(domain));
    html
}

pub fn discover_page(burrows: &[BurrowEntry], latest_posts: &[(String, String, String, String)], popular: &[(String, String, usize)], rings: &[Ring], random_pick: Option<&BurrowEntry>, domain: &str) -> String {
    let mut html = head("Discover", "/discover", domain);
    html.push_str(&format!(r#"<div class="container">{}<div class="main">
<div class="crumbs"><a href="/">{}</a> / discover</div>
<h1>Discover</h1>
<div class="subtitle">Explore the burrow network — {} burrows and counting</div>"#,
        sidebar("/discover", burrows),
        html_escape(domain),
        burrows.len(),
    ));

    // Most bookmarked
    if !popular.is_empty() {
        html.push_str(r#"<div class="section-label">Most bookmarked</div>"#);
        for (url, desc, count) in popular {
            let bookmark_label = if *count == 1 { "1 bookmark".to_string() } else { format!("{} bookmarks", count) };
            let display_desc = if desc.is_empty() { url.clone() } else { desc.clone() };
            html.push_str(&format!(
                r#"<a class="entry" href="{path}"><span class="entry-type" style="color:var(--accent);">★</span><span class="entry-name">{desc}</span><span class="entry-desc">{url}</span><span class="entry-meta">{meta}</span></a>"#,
                path = html_escape(url),
                desc = html_escape(&display_desc),
                url = html_escape(url),
                meta = html_escape(&bookmark_label),
            ));
        }
    }

    // Random burrow spotlight
    if let Some(pick) = random_pick {
        html.push_str(&format!(
            r#"<div class="section-label">Random burrow</div>
<a class="entry" href="{path}"><span class="entry-type">/</span><span class="entry-name">{name}</span><span class="entry-desc">{desc}</span><span class="entry-meta"><a href="/random">↻</a></span></a>"#,
            path = html_escape(&pick.path),
            name = html_escape(&pick.name),
            desc = html_escape(&pick.description),
        ));
    }

    // Latest posts
    if !latest_posts.is_empty() {
        html.push_str(r#"<div class="section-label">Latest posts</div>"#);
        for (date, title, burrow, url_path) in latest_posts {
            html.push_str(&format!(
                r#"<a class="entry" href="{path}"><span class="entry-type txt">¶</span><span class="entry-name">{title}</span><span class="entry-desc">{burrow}</span><span class="entry-meta">{date}</span></a>"#,
                path = html_escape(url_path),
                title = html_escape(title),
                burrow = html_escape(burrow),
                date = html_escape(date),
            ));
        }
    }

    // Rings
    if !rings.is_empty() {
        html.push_str(r#"<div class="section-label">Rings</div>"#);
        for ring in rings {
            html.push_str(&format!(
                r#"<a class="entry" href="/rings"><span class="entry-type" style="color:var(--accent);">◎</span><span class="entry-name">{title}</span><span class="entry-desc">{desc}</span><span class="entry-meta">{count} members</span></a>"#,
                title = html_escape(&ring.title),
                desc = html_escape(&ring.owner),
                count = ring.members.len(),
            ));
        }
    }

    // All burrows
    html.push_str(r#"<div class="section-label">All burrows</div>"#);
    for burrow in burrows {
        html.push_str(&format!(
            r#"<a class="entry" href="{path}"><span class="entry-type">/</span><span class="entry-name">{name}</span><span class="entry-desc">{desc}</span><span class="entry-meta">{meta}</span></a>"#,
            path = html_escape(&burrow.path),
            name = html_escape(&burrow.name),
            desc = html_escape(&burrow.description),
            meta = html_escape(&burrow.meta),
        ));
    }

    html.push_str(r#"<div style="text-align:center;padding:24px 0;font-size:13px;color:var(--muted);">
<a href="/firehose">View all posts →</a> · <a href="/random">Random burrow →</a>
</div>"#);

    html.push_str("</div></div>");
    html.push_str(&footer(domain));
    html
}

// ── Gemtext rendering ───────────────────────────────────────────

pub fn render_gph_to_gmi(content: &str) -> String {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let mut gmi = String::new();
    let mut in_code = false;

    for raw_line in content.lines() {
        let owned;
        let line = if !in_code && !raw_line.starts_with("  ") {
            owned = raw_line.replace("@today", &today);
            owned.as_str()
        } else {
            raw_line
        };
        if !in_code && line.starts_with("  ") {
            gmi.push_str("```\n");
            gmi.push_str(line.trim_start());
            gmi.push('\n');
            in_code = true;
            continue;
        }
        if in_code {
            if line.starts_with("  ") || line.is_empty() {
                gmi.push_str(if line.is_empty() { "" } else { line.trim_start() });
                gmi.push('\n');
                continue;
            } else {
                gmi.push_str("```\n");
                in_code = false;
            }
        }

        if line.starts_with("# ") {
            // Headings pass through
            gmi.push_str(line);
            gmi.push('\n');
        } else if line.starts_with("> ") {
            // Quotes pass through
            gmi.push_str(line);
            gmi.push('\n');
        } else if line == "---" {
            // Horizontal rule → blank line
            gmi.push('\n');
        } else if let Some(rest) = line.strip_prefix("→ ") {
            // External link → Gemini link
            let url = rest.trim();
            gmi.push_str(&format!("=> {}\n", url));
        } else if line.starts_with("/~") {
            // Internal link: /~user/path   Description
            let parts: Vec<&str> = line.splitn(2, "   ").collect();
            let link = parts[0].trim();
            let desc = parts.get(1).unwrap_or(&"");
            if desc.is_empty() {
                gmi.push_str(&format!("=> {}\n", link));
            } else {
                gmi.push_str(&format!("=> {} {}\n", link, desc));
            }
        } else if line.is_empty() {
            gmi.push('\n');
        } else {
            gmi.push_str(line);
            gmi.push('\n');
        }
    }
    if in_code {
        gmi.push_str("```\n");
    }
    gmi
}

pub fn home_gmi(burrows: &[BurrowEntry], domain: &str) -> String {
    let mut gmi = String::new();
    gmi.push_str(&format!("# {}\n\n", domain));
    gmi.push_str(&format!("Community burrow server — {} burrows\n\n", burrows.len()));
    for b in burrows {
        let name = b.name.trim_end_matches('/');
        if b.description.is_empty() {
            gmi.push_str(&format!("=> {} {}\n", b.path, name));
        } else {
            gmi.push_str(&format!("=> {} {} — {}\n", b.path, name, b.description));
        }
    }
    gmi.push_str("\n=> /discover Discover\n");
    gmi.push_str("=> /firehose Firehose\n");
    gmi
}

pub fn directory_listing_gmi(path: &str, entries: &[BurrowEntry]) -> String {
    let mut gmi = String::new();
    gmi.push_str(&format!("# {}/\n\n", path));

    let dirs: Vec<_> = entries.iter().filter(|e| e.entry_type == EntryType::Directory).collect();
    let files: Vec<_> = entries.iter().filter(|e| e.entry_type != EntryType::Directory).collect();

    if !dirs.is_empty() {
        gmi.push_str("## Directories\n\n");
        for e in &dirs {
            if e.description.is_empty() {
                gmi.push_str(&format!("=> {} {}\n", e.path, e.name));
            } else {
                gmi.push_str(&format!("=> {} {} — {}\n", e.path, e.name, e.description));
            }
        }
        gmi.push('\n');
    }
    if !files.is_empty() {
        gmi.push_str("## Files\n\n");
        for e in &files {
            if e.description.is_empty() {
                gmi.push_str(&format!("=> {} {} ({})\n", e.path, e.name, e.meta));
            } else {
                gmi.push_str(&format!("=> {} {} — {} ({})\n", e.path, e.name, e.description, e.meta));
            }
        }
    }
    gmi
}

pub fn not_found_gmi(path: &str) -> String {
    format!("# Not Found\n\nThis hole leads nowhere: {}\n\n=> / ← Back to the surface\n", path)
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn html_escape_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}
