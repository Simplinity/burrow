use crate::{BurrowEntry, EntryType, GuestbookEntry};

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
.progress{position:fixed;top:0;left:0;height:2px;background:var(--accent);transition:width 0.1s;z-index:100;opacity:0.6}

.statusbar{display:flex;justify-content:space-between;padding:6px 24px;border-top:1px solid var(--faint);font-size:11px;color:var(--muted)}
.banner{text-align:center;padding:12px;font-size:12px;color:var(--muted);border-top:1px solid var(--faint)}
.notfound{text-align:center;padding:80px 24px;font-size:15px;color:var(--muted);line-height:1.8}

@media(max-width:700px){.sidebar{display:none}.main{padding:20px 16px}}
"#;

fn head(title: &str, addr: &str, domain: &str) -> String {
    let title = html_escape(title);
    let addr = html_escape(addr);
    let domain = html_escape(domain);
    // Extract burrow name from addr for RSS feed link (e.g. "/~bruno/phlog" → "/~bruno/feed.xml")
    let rss_href = addr.split('/').nth(1)
        .filter(|s| s.starts_with('~'))
        .map(|b| format!("/{}/feed.xml", b))
        .unwrap_or_else(|| "/feed.xml".to_string());
    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>{title} — Burrow</title>
<style>{CSS}</style>
<link rel="alternate" type="application/rss+xml" title="RSS Feed" href="{rss_href}">
</head>
<body>
<div class="topbar">
  <a href="/" class="logo"><span>/</span> burrow</a>
  <div class="addr-bar">
    <span style="color:var(--muted)">gph://</span><span class="host">{domain}</span>{addr}
  </div>
</div>"#)
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
    html.push_str(r#"<div class="sb-label">Discover</div>
<a class="sb-item" href="javascript:void(0)"><span class="sb-icon">?</span>Veronica-NG</a>
<a class="sb-item" href="javascript:void(0)"><span class="sb-icon">*</span>Rings</a>
</div>"#);
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
<div class="subtitle">Community burrow server — {} burrows</div>
{}</div></div>"#,
        sidebar("/", burrows),
        html_escape(domain),
        html_escape(domain),
        burrows.len(),
        render_entries(burrows),
    ));
    html.push_str(&footer(domain));
    html
}

pub fn directory_page(path: &str, entries: &[BurrowEntry], burrows: &[BurrowEntry], domain: &str) -> String {
    let crumbs = build_crumbs(path, domain);
    let desc = entries.first().map(|_| "").unwrap_or("");
    let addr = format!("/{}", path);

    let mut html = head(path, &addr, domain);
    html.push_str(&format!(r#"<div class="container">{}<div class="main">
<div class="crumbs">{}</div>
<h1>{}/</h1>
<div class="subtitle">{}</div>
{}</div></div>"#,
        sidebar(&format!("/{}", path.split('/').next().unwrap_or("")), burrows),
        crumbs, html_escape(path), desc,
        render_entries(entries),
    ));
    html.push_str(&footer(domain));
    html
}

pub fn text_page(path: &str, filename: &str, content: &str, domain: &str) -> String {
    let crumbs = build_crumbs(path, domain);
    let words = content.split_whitespace().count();
    let read_min = (words as f64 / 230.0).ceil() as usize;
    let rendered = render_gph(content);

    let mut html = head(filename, &format!("/{}", path), domain);
    html.push_str(&format!(r#"<div class="progress" id="prog"></div>
<div style="max-width:680px;margin:0 auto;padding:0 24px;">
<div class="crumbs" style="margin-top:24px;">{crumbs}</div>
<div class="reading">
<div class="meta">~{read_min} min read · {words} words</div>
{rendered}
</div>
</div>"#));
    html.push_str(&footer(domain));
    html.replace("</body></html>", r#"<script>
window.addEventListener('scroll',()=>{
  const h=document.documentElement;
  const pct=(h.scrollTop/(h.scrollHeight-h.clientHeight))*100;
  document.getElementById('prog').style.width=pct+'%';
});
</script>
</body></html>"#)
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

pub fn guestbook_page(path: &str, entries: &[GuestbookEntry], domain: &str) -> String {
    let crumbs = build_crumbs(path, domain);
    let burrow_name = path.split('/').next().unwrap_or(path);

    let mut html = head("Guestbook", &format!("/{}", path), domain);
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

pub fn render_gph(content: &str) -> String {
    let mut html = String::new();
    let mut in_code = false;

    for line in content.lines() {
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
