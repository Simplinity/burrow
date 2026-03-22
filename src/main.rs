use axum::{
    extract::{Form, Path, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
    Router,
};
use chrono::Local;
use serde::Deserialize;
use std::fs;
use std::path;
use std::sync::Arc;

use burrow::config;
mod render;

const MAX_FILE_SIZE: u64 = 1_048_576; // 1 MB

#[tokio::main]
async fn main() {
    let cfg = config::ServerConfig::load();
    let domain = Arc::new(cfg.domain.clone());
    let addr = cfg.bind_addr();

    let app = Router::new()
        .route("/", get(home))
        .route("/{*path}", get(serve_burrow).post(post_guestbook))
        .with_state(domain);

    println!("\n  \x1b[1m/\x1b[0m burrow v0.1.0\n");
    println!("  Tunneling...\n");
    println!("  Domain:         \x1b[36m{}\x1b[0m", cfg.domain);
    println!("  HTTPS gateway:  \x1b[36mhttp://{}\x1b[0m", addr);
    println!("  Burrow root:    \x1b[36m./burrows/\x1b[0m\n");
    println!("  Press Ctrl+C to fill in the hole.\n");

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn home(State(domain): State<Arc<String>>) -> Html<String> {
    let burrows = list_burrows();
    Html(render::home_page(&burrows, &domain))
}

async fn serve_burrow(Path(path): Path<String>, State(domain): State<Arc<String>>) -> Response {
    let burrows_root = fs::canonicalize("burrows").unwrap_or_else(|_| path::PathBuf::from("burrows"));

    // Virtual routes: feed.xml is generated, not a real file
    if path.ends_with("/feed.xml") || path.ends_with("/feed") {
        let burrow_name = path.split('/').next().unwrap_or("");
        let burrow_dir = path::PathBuf::from("burrows").join(burrow_name);
        if burrow_dir.is_dir() && burrow_name.starts_with('~') {
            let xml = generate_feed(burrow_name, &burrow_dir, &domain);
            return ([(header::CONTENT_TYPE, "application/rss+xml; charset=utf-8")], xml).into_response();
        }
        return (StatusCode::NOT_FOUND, Html(render::not_found_page(&path, &domain))).into_response();
    }

    let fs_path = path::PathBuf::from("burrows").join(&path);

    // Path traversal protection: canonicalize and verify prefix
    let canonical = match fs::canonicalize(&fs_path) {
        Ok(p) => p,
        Err(_) => {
            // File doesn't exist — try with .txt then .gph extension
            let with_txt = fs_path.with_extension("txt");
            let with_gph = fs_path.with_extension("gph");
            if let Ok(p) = fs::canonicalize(&with_txt) {
                if p.starts_with(&burrows_root) {
                    let content = read_file_checked(&p);
                    let filename = p.file_name().unwrap().to_str().unwrap();
                    return Html(render::text_page(&path, filename, &content, &domain)).into_response();
                }
            }
            if let Ok(p) = fs::canonicalize(&with_gph) {
                if p.starts_with(&burrows_root) {
                    let content = read_file_checked(&p);
                    let filename = p.file_name().unwrap().to_str().unwrap();
                    if filename == "guestbook.gph" {
                        let entries = parse_guestbook(&content);
                        return Html(render::guestbook_page(&path, &entries, &domain)).into_response();
                    }
                    return Html(render::text_page(&path, filename, &content, &domain)).into_response();
                }
            }
            return (StatusCode::NOT_FOUND, Html(render::not_found_page(&path, &domain))).into_response();
        }
    };

    if !canonical.starts_with(&burrows_root) {
        return (StatusCode::NOT_FOUND, Html(render::not_found_page(&path, &domain))).into_response();
    }

    if canonical.is_dir() {
        let burrows = list_burrows();
        let entries = list_directory(&canonical, &burrows_root);
        Html(render::directory_page(&path, &entries, &burrows, &domain)).into_response()
    } else {
        let content = read_file_checked(&canonical);
        let filename = canonical.file_name().unwrap().to_str().unwrap();
        if filename == "guestbook.gph" {
            let entries = parse_guestbook(&content);
            Html(render::guestbook_page(&path, &entries, &domain)).into_response()
        } else {
            Html(render::text_page(&path, filename, &content, &domain)).into_response()
        }
    }
}

fn read_file_checked(path: &path::Path) -> String {
    let size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    if size > MAX_FILE_SIZE {
        return format!(
            "This file is too large to display ({:.1} MB). Maximum is {} MB.",
            size as f64 / 1_048_576.0,
            MAX_FILE_SIZE / 1_048_576
        );
    }
    fs::read_to_string(path).unwrap_or_default()
}

#[derive(Debug, Clone)]
pub struct BurrowEntry {
    pub name: String,
    pub entry_type: EntryType,
    pub description: String,
    pub meta: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EntryType {
    Directory,
    Text,
}

pub fn list_burrows() -> Vec<BurrowEntry> {
    let mut entries = Vec::new();
    if let Ok(dirs) = fs::read_dir("burrows") {
        for entry in dirs.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('~') && entry.path().is_dir() {
                let desc = read_description(&entry.path());
                let count = fs::read_dir(entry.path()).map(|d| d.count()).unwrap_or(0);
                entries.push(BurrowEntry {
                    path: format!("/{}", name),
                    name: format!("{}/", name),
                    entry_type: EntryType::Directory,
                    description: desc,
                    meta: if count == 1 { "1 item".to_string() } else { format!("{} items", count) },
                });
            }
        }
    }
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    entries
}

fn list_directory(dir: &path::Path, burrows_root: &path::Path) -> Vec<BurrowEntry> {
    let mut entries = Vec::new();
    if let Ok(items) = fs::read_dir(dir) {
        for item in items.flatten() {
            let name = item.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || name.starts_with('_') {
                continue;
            }
            let path = item.path();
            let relative = path.strip_prefix(burrows_root)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();

            if path.is_dir() {
                let desc = read_description(&path);
                let count = fs::read_dir(&path).map(|d| d.count()).unwrap_or(0);
                entries.push(BurrowEntry {
                    path: format!("/{}", relative),
                    name: format!("{}/", name),
                    entry_type: EntryType::Directory,
                    description: desc,
                    meta: if count == 1 { "1 item".to_string() } else { format!("{} items", count) },
                });
            } else {
                let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                let size_str = if size < 1024 {
                    format!("{} B", size)
                } else {
                    format!("{:.1} KB", size as f64 / 1024.0)
                };
                entries.push(BurrowEntry {
                    path: format!("/{}", relative),
                    name: name.clone(),
                    entry_type: EntryType::Text,
                    description: first_line_of(&path),
                    meta: size_str,
                });
            }
        }
    }
    entries.sort_by(|a, b| {
        let type_ord = |e: &BurrowEntry| if e.entry_type == EntryType::Directory { 0 } else { 1 };
        type_ord(a).cmp(&type_ord(b)).then(a.name.cmp(&b.name))
    });
    entries
}

fn read_description(dir: &path::Path) -> String {
    let burrow_file = dir.join(".burrow");
    if burrow_file.exists() {
        if let Ok(content) = fs::read_to_string(&burrow_file) {
            for line in content.lines() {
                if let Some(desc) = line.strip_prefix("description = ") {
                    return desc.to_string();
                }
            }
        }
    }
    String::new()
}

fn first_line_of(path: &path::Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .next()
        .unwrap_or("")
        .trim_start_matches("# ")
        .to_string()
}

// ── RSS Feed ────────────────────────────────────────────────────

fn generate_feed(burrow_name: &str, burrow_dir: &path::Path, domain: &str) -> String {
    let desc = read_description(burrow_dir);
    let phlog_dir = burrow_dir.join("phlog");

    let base_url = if domain == "localhost" {
        format!("http://localhost:7070/{}", burrow_name)
    } else {
        format!("https://{}/{}", domain, burrow_name)
    };

    let mut items = Vec::new();

    if phlog_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&phlog_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') || name.starts_with('_') || !name.ends_with(".txt") {
                    continue;
                }
                let content = fs::read_to_string(entry.path()).unwrap_or_default();
                let title = content.lines().next().unwrap_or("").trim_start_matches("# ").to_string();
                let slug = name.trim_end_matches(".txt");

                // Extract date from filename: YYYY-MM-DD-slug.txt
                let date = if name.len() >= 10 {
                    &name[..10]
                } else {
                    "2000-01-01"
                };

                // Preview: first ~300 chars of content after the title
                let preview: String = content.lines().skip(1)
                    .filter(|l| !l.is_empty())
                    .take(5)
                    .collect::<Vec<_>>()
                    .join(" ");
                let preview = xml_escape(&preview);

                items.push((date.to_string(), format!(
                    r#"    <item>
      <title>{}</title>
      <link>{}/phlog/{}</link>
      <guid>{}/phlog/{}</guid>
      <pubDate>{}</pubDate>
      <description>{}</description>
    </item>"#,
                    xml_escape(&title),
                    base_url, slug,
                    base_url, slug,
                    date,
                    preview,
                )));
            }
        }
    }

    // Sort by date descending
    items.sort_by(|a, b| b.0.cmp(&a.0));
    let items_xml: String = items.iter().map(|(_, xml)| xml.as_str()).collect::<Vec<_>>().join("\n");

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>{}</title>
    <link>{}</link>
    <description>{}</description>
    <language>en</language>
{}
  </channel>
</rss>"#,
        xml_escape(burrow_name),
        base_url,
        xml_escape(&desc),
        items_xml,
    )
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

// ── Guestbook ───────────────────────────────────────────────────

const MAX_GUESTBOOK_ENTRIES: usize = 200;
const MAX_GUESTBOOK_MSG_LEN: usize = 500;
const MAX_GUESTBOOK_NAME_LEN: usize = 40;

#[derive(Debug, Clone)]
pub struct GuestbookEntry {
    pub name: String,
    pub date: String,
    pub message: String,
}

#[derive(Deserialize)]
struct GuestbookForm {
    name: String,
    message: String,
}

fn parse_guestbook(content: &str) -> Vec<GuestbookEntry> {
    let mut entries = Vec::new();
    let mut current_name = String::new();
    let mut current_date = String::new();
    let mut current_msg = String::new();

    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("--- ") {
            // Save previous entry if any
            if !current_name.is_empty() {
                entries.push(GuestbookEntry {
                    name: current_name.clone(),
                    date: current_date.clone(),
                    message: current_msg.trim().to_string(),
                });
            }
            // Parse header: "--- name · date"
            let parts: Vec<&str> = rest.splitn(2, " · ").collect();
            current_name = parts.first().unwrap_or(&"").to_string();
            current_date = parts.get(1).unwrap_or(&"").to_string();
            current_msg = String::new();
        } else if !current_name.is_empty() {
            if !current_msg.is_empty() {
                current_msg.push('\n');
            }
            current_msg.push_str(line);
        }
    }
    // Don't forget the last entry
    if !current_name.is_empty() {
        entries.push(GuestbookEntry {
            name: current_name,
            date: current_date,
            message: current_msg.trim().to_string(),
        });
    }
    entries
}

async fn post_guestbook(
    Path(path): Path<String>,
    State(domain): State<Arc<String>>,
    Form(form): Form<GuestbookForm>,
) -> Response {
    let burrows_root = fs::canonicalize("burrows").unwrap_or_else(|_| path::PathBuf::from("burrows"));
    let fs_path = path::PathBuf::from("burrows").join(&path);

    // Must end with "guestbook" and resolve to guestbook.gph
    let gph_path = if fs_path.extension().is_none() {
        fs_path.with_extension("gph")
    } else {
        fs_path.clone()
    };

    let canonical = match fs::canonicalize(&gph_path) {
        Ok(p) if p.starts_with(&burrows_root) => p,
        _ => {
            return (StatusCode::NOT_FOUND, Html(render::not_found_page(&path, &domain))).into_response();
        }
    };

    if canonical.file_name().and_then(|f| f.to_str()) != Some("guestbook.gph") {
        return (StatusCode::BAD_REQUEST, "Not a guestbook").into_response();
    }

    // Validate input
    let name = form.name.trim().to_string();
    let message = form.message.trim().to_string();

    if name.is_empty() || message.is_empty() {
        return Redirect::to(&format!("/{}", path)).into_response();
    }

    // Truncate to limits
    let name: String = name.chars().take(MAX_GUESTBOOK_NAME_LEN).collect();
    let message: String = message.chars().take(MAX_GUESTBOOK_MSG_LEN).collect();

    // Sanitize: strip any "---" from user input to prevent format injection
    let name = name.replace("---", "—");
    let message = message.replace("---", "—");

    // Check entry count
    let existing = fs::read_to_string(&canonical).unwrap_or_default();
    let entry_count = existing.matches("\n--- ").count() + if existing.starts_with("--- ") { 1 } else { 0 };
    if entry_count >= MAX_GUESTBOOK_ENTRIES {
        return Redirect::to(&format!("/{}", path)).into_response();
    }

    // Append new entry
    let date = Local::now().format("%Y-%m-%d %H:%M").to_string();
    let entry = format!("\n--- {} · {}\n{}\n", name, date, message);

    let mut content = fs::read_to_string(&canonical).unwrap_or_default();
    content.push_str(&entry);
    fs::write(&canonical, content).unwrap_or(());

    Redirect::to(&format!("/{}", path)).into_response()
}

#[cfg(test)]
mod tests;
