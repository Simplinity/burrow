use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use std::fs;
use std::path;
use std::sync::Arc;

mod config;
mod render;

const MAX_FILE_SIZE: u64 = 1_048_576; // 1 MB

#[tokio::main]
async fn main() {
    let cfg = config::ServerConfig::load();
    let domain = Arc::new(cfg.domain.clone());
    let addr = cfg.bind_addr();

    let app = Router::new()
        .route("/", get(home))
        .route("/{*path}", get(serve_burrow))
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
    let fs_path = path::PathBuf::from("burrows").join(&path);

    // Path traversal protection: canonicalize and verify prefix
    let canonical = match fs::canonicalize(&fs_path) {
        Ok(p) => p,
        Err(_) => {
            // File doesn't exist — try with .txt extension before giving up
            let with_txt = fs_path.with_extension("txt");
            match fs::canonicalize(&with_txt) {
                Ok(p) if p.starts_with(&burrows_root) => {
                    let content = read_file_checked(&p);
                    let filename = p.file_name().unwrap().to_str().unwrap();
                    return Html(render::text_page(&path, filename, &content, &domain)).into_response();
                }
                _ => {
                    return (StatusCode::NOT_FOUND, Html(render::not_found_page(&path, &domain))).into_response();
                }
            }
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
        Html(render::text_page(&path, filename, &content, &domain)).into_response()
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
                    meta: format!("{} items", count),
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
                    meta: format!("{} items", count),
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

#[cfg(test)]
mod tests;
