use axum::{
    extract::{ConnectInfo, Form, Path, Query, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
    Router,
};
use chrono::Local;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::net::{IpAddr, SocketAddr};
use std::path;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use burrow::config;
mod render;

const MAX_FILE_SIZE: u64 = 65_536; // 64 KB
const MAX_PATH_DEPTH: usize = 8;
const GUESTBOOK_RATE_SECS: u64 = 30;
const FIREHOSE_PAGE_SIZE: usize = 20;

#[derive(Clone)]
struct AppState {
    domain: Arc<String>,
    guestbook_limiter: Arc<Mutex<HashMap<IpAddr, Instant>>>,
    started_at: Instant,
}

#[tokio::main]
async fn main() {
    // Initialize tracing (RUST_LOG env var controls level, default: info)
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .compact()
        .init();

    let cfg = config::ServerConfig::load();
    let addr = cfg.bind_addr();

    let state = AppState {
        domain: Arc::new(cfg.domain.clone()),
        guestbook_limiter: Arc::new(Mutex::new(HashMap::new())),
        started_at: Instant::now(),
    };

    let app = Router::new()
        .route("/", get(home))
        .route("/robots.txt", get(robots_txt))
        .route("/favicon.ico", get(favicon_ico))
        .route("/health", get(health))
        .route("/stats", get(stats))
        .route("/firehose", get(firehose))
        .route("/random", get(random_burrow))
        .route("/{*path}", get(serve_burrow).post(post_guestbook))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    println!("\n  \x1b[1m/\x1b[0m burrow v0.1.0\n");
    println!("  Tunneling...\n");
    println!("  Domain:         \x1b[36m{}\x1b[0m", cfg.domain);
    println!("  HTTPS gateway:  \x1b[36mhttp://{}\x1b[0m", addr);
    println!("  Burrow root:    \x1b[36m./burrows/\x1b[0m\n");
    println!("  Press Ctrl+C to fill in the hole.\n");

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}

async fn home(State(state): State<AppState>) -> Html<String> {
    let domain = &state.domain;
    let burrows = list_burrows();
    Html(render::home_page(&burrows, domain))
}

async fn robots_txt() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], "User-agent: *\nAllow: /\n")
}

async fn health() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "application/json")], "{\"status\":\"ok\"}")
}

async fn stats(State(state): State<AppState>) -> impl IntoResponse {
    let burrows = list_burrows();
    let burrow_count = burrows.len();
    let mut file_count: usize = 0;
    for burrow in &burrows {
        let dir = path::PathBuf::from("burrows").join(burrow.name.trim_end_matches('/'));
        file_count += count_files_recursive(&dir);
    }
    let uptime_secs = state.started_at.elapsed().as_secs();
    let json = format!(
        "{{\"burrows\":{},\"files\":{},\"uptime_secs\":{}}}",
        burrow_count, file_count, uptime_secs
    );
    ([(header::CONTENT_TYPE, "application/json")], json)
}

fn count_files_recursive(dir: &path::Path) -> usize {
    let mut count = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || name.starts_with('_') {
                continue;
            }
            if entry.path().is_dir() {
                count += count_files_recursive(&entry.path());
            } else {
                count += 1;
            }
        }
    }
    count
}

#[derive(Deserialize)]
struct PaginationParams {
    page: Option<usize>,
}

async fn firehose(State(state): State<AppState>, Query(params): Query<PaginationParams>) -> Html<String> {
    let domain = &state.domain;
    let burrows = list_burrows();
    let mut posts: Vec<(String, String, String, String)> = Vec::new();

    for burrow in &burrows {
        let burrow_name = burrow.name.trim_end_matches('/');
        let phlog_dir = path::PathBuf::from("burrows").join(burrow_name).join("phlog");
        if !phlog_dir.is_dir() {
            continue;
        }
        if let Ok(entries) = fs::read_dir(&phlog_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') || name.starts_with('_') || !name.ends_with(".txt") {
                    continue;
                }
                let content = fs::read_to_string(entry.path()).unwrap_or_default();
                let title = content.lines().next().unwrap_or("").trim_start_matches("# ").to_string();
                let slug = name.trim_end_matches(".txt");
                let date = if name.len() >= 10 { name[..10].to_string() } else { String::new() };
                let url_path = format!("/{}/phlog/{}", burrow_name, slug);
                posts.push((date, title, burrow_name.to_string(), url_path));
            }
        }
    }

    posts.sort_by(|a, b| b.0.cmp(&a.0));

    let page = params.page.unwrap_or(1).max(1);
    let total_pages = (posts.len() + FIREHOSE_PAGE_SIZE - 1) / FIREHOSE_PAGE_SIZE;
    let start = (page - 1) * FIREHOSE_PAGE_SIZE;
    let page_posts: Vec<_> = posts.into_iter().skip(start).take(FIREHOSE_PAGE_SIZE).collect();

    let prev = if page > 1 { Some(page - 1) } else { None };
    let next = if page < total_pages { Some(page + 1) } else { None };

    Html(render::firehose_page(&page_posts, &burrows, domain, prev, next))
}

async fn random_burrow() -> Response {
    let burrows = list_burrows();
    if burrows.is_empty() {
        return Redirect::to("/").into_response();
    }
    // Simple pseudo-random: use nanosecond component of current time
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as usize;
    let idx = nanos % burrows.len();
    Redirect::to(&burrows[idx].path).into_response()
}

async fn favicon_ico() -> impl IntoResponse {
    // 1x1 transparent ICO (62 bytes)
    const ICO: &[u8] = &[
        0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01, 0x01, 0x00, 0x00, 0x01, 0x00,
        0x18, 0x00, 0x30, 0x00, 0x00, 0x00, 0x16, 0x00, 0x00, 0x00, 0x28, 0x00,
        0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x01, 0x00,
        0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00,
    ];
    ([(header::CONTENT_TYPE, "image/x-icon")], ICO)
}

async fn serve_burrow(Path(path): Path<String>, State(state): State<AppState>) -> Response {
    let domain = &state.domain;
    let burrows_root = fs::canonicalize("burrows").unwrap_or_else(|_| path::PathBuf::from("burrows"));

    // Virtual routes: feeds are generated, not real files
    if path.ends_with("/feed.xml") || path.ends_with("/feed") {
        let burrow_name = path.split('/').next().unwrap_or("");
        let burrow_dir = path::PathBuf::from("burrows").join(burrow_name);
        if burrow_dir.is_dir() && burrow_name.starts_with('~') {
            let xml = generate_feed(burrow_name, &burrow_dir, &domain);
            return ([(header::CONTENT_TYPE, "application/rss+xml; charset=utf-8")], xml).into_response();
        }
        return (StatusCode::NOT_FOUND, Html(render::not_found_page(&path, &domain))).into_response();
    }
    if path.ends_with("/atom.xml") || path.ends_with("/atom") {
        let burrow_name = path.split('/').next().unwrap_or("");
        let burrow_dir = path::PathBuf::from("burrows").join(burrow_name);
        if burrow_dir.is_dir() && burrow_name.starts_with('~') {
            let xml = generate_atom_feed(burrow_name, &burrow_dir, &domain);
            return ([(header::CONTENT_TYPE, "application/atom+xml; charset=utf-8")], xml).into_response();
        }
        return (StatusCode::NOT_FOUND, Html(render::not_found_page(&path, &domain))).into_response();
    }

    // Draft visibility: block any path segment starting with _ or .
    if path.split('/').any(|seg| seg.starts_with('_') || seg.starts_with('.')) {
        return (StatusCode::NOT_FOUND, Html(render::not_found_page(&path, &domain))).into_response();
    }

    // Depth limit: reject paths deeper than MAX_PATH_DEPTH levels
    if path.split('/').filter(|s| !s.is_empty()).count() > MAX_PATH_DEPTH {
        return (StatusCode::NOT_FOUND, Html(render::not_found_page(&path, &domain))).into_response();
    }

    // Extract accent color from the burrow's .burrow config
    let accent = path.split('/').next()
        .filter(|s| s.starts_with('~'))
        .and_then(|b| read_accent(&path::PathBuf::from("burrows").join(b)));

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
                    return Html(render::text_page(&path, filename, &content, &domain, accent.as_deref())).into_response();
                }
            }
            if let Ok(p) = fs::canonicalize(&with_gph) {
                if p.starts_with(&burrows_root) {
                    let content = read_file_checked(&p);
                    let filename = p.file_name().unwrap().to_str().unwrap();
                    if filename == "guestbook.gph" {
                        let entries = parse_guestbook(&content);
                        return Html(render::guestbook_page(&path, &entries, &domain, accent.as_deref())).into_response();
                    }
                    return Html(render::text_page(&path, filename, &content, &domain, accent.as_deref())).into_response();
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
        let title = read_title(&canonical);
        Html(render::directory_page(&path, title.as_deref(), &entries, &burrows, &domain, accent.as_deref())).into_response()
    } else {
        let content = read_file_checked(&canonical);
        let filename = canonical.file_name().unwrap().to_str().unwrap();
        if filename == "guestbook.gph" {
            let entries = parse_guestbook(&content);
            Html(render::guestbook_page(&path, &entries, &domain, accent.as_deref())).into_response()
        } else {
            Html(render::text_page(&path, filename, &content, &domain, accent.as_deref())).into_response()
        }
    }
}

fn read_file_checked(path: &path::Path) -> String {
    let size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    if size > MAX_FILE_SIZE {
        return format!(
            "This file is too large to display ({:.1} KB). Maximum is {} KB.",
            size as f64 / 1024.0,
            MAX_FILE_SIZE / 1024
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
                let display_name = read_title(&path).unwrap_or_else(|| name.clone());
                let count = fs::read_dir(&path).map(|d| d.count()).unwrap_or(0);
                entries.push(BurrowEntry {
                    path: format!("/{}", relative),
                    name: format!("{}/", display_name),
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

fn read_title(dir: &path::Path) -> Option<String> {
    let burrow_file = dir.join(".burrow");
    if let Ok(content) = fs::read_to_string(burrow_file) {
        for line in content.lines() {
            if let Some(title) = line.strip_prefix("title = ") {
                let title = title.trim();
                if !title.is_empty() {
                    return Some(title.to_string());
                }
            }
        }
    }
    None
}

fn read_accent(dir: &path::Path) -> Option<String> {
    let burrow_file = dir.join(".burrow");
    if let Ok(content) = fs::read_to_string(burrow_file) {
        for line in content.lines() {
            if let Some(color) = line.strip_prefix("accent = ") {
                let color = color.trim();
                // Validate: must be a hex color like #abc or #aabbcc
                if color.starts_with('#') && (color.len() == 4 || color.len() == 7)
                    && color[1..].chars().all(|c| c.is_ascii_hexdigit())
                {
                    return Some(color.to_string());
                }
            }
        }
    }
    None
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

fn generate_atom_feed(burrow_name: &str, burrow_dir: &path::Path, domain: &str) -> String {
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
                let date = if name.len() >= 10 { &name[..10] } else { "2000-01-01" };

                let preview: String = content.lines().skip(1)
                    .filter(|l| !l.is_empty())
                    .take(5)
                    .collect::<Vec<_>>()
                    .join(" ");

                items.push((date.to_string(), format!(
                    r#"  <entry>
    <title>{}</title>
    <link href="{}/phlog/{}"/>
    <id>{}/phlog/{}</id>
    <updated>{}T00:00:00Z</updated>
    <summary>{}</summary>
  </entry>"#,
                    xml_escape(&title),
                    base_url, slug,
                    base_url, slug,
                    date,
                    xml_escape(&preview),
                )));
            }
        }
    }

    items.sort_by(|a, b| b.0.cmp(&a.0));
    let updated = items.first().map(|(d, _)| format!("{}T00:00:00Z", d)).unwrap_or_else(|| "2000-01-01T00:00:00Z".to_string());
    let entries_xml: String = items.iter().map(|(_, xml)| xml.as_str()).collect::<Vec<_>>().join("\n");

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>{}</title>
  <link href="{}"/>
  <link rel="self" href="{}/atom.xml"/>
  <id>{}</id>
  <updated>{}</updated>
  <subtitle>{}</subtitle>
{}
</feed>"#,
        xml_escape(burrow_name),
        base_url,
        base_url,
        base_url,
        updated,
        xml_escape(&desc),
        entries_xml,
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
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(path): Path<String>,
    State(state): State<AppState>,
    Form(form): Form<GuestbookForm>,
) -> Response {
    let domain = &state.domain;

    // Rate limit: one guestbook post per GUESTBOOK_RATE_SECS per IP
    {
        let mut limiter = state.guestbook_limiter.lock().unwrap();
        let now = Instant::now();
        if let Some(last) = limiter.get(&addr.ip()) {
            if now.duration_since(*last).as_secs() < GUESTBOOK_RATE_SECS {
                return (StatusCode::TOO_MANY_REQUESTS, "Slow down — try again in a moment.").into_response();
            }
        }
        limiter.insert(addr.ip(), now);
    }

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
