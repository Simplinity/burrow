use axum::{
    extract::Path,
    response::Html,
    routing::get,
    Router,
};
use std::fs;
use std::path::PathBuf;

mod render;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(home))
        .route("/{*path}", get(serve_burrow));

    let addr = "127.0.0.1:7070";
    println!("\n  \x1b[1m/\x1b[0m burrow v0.1.0\n");
    println!("  Tunneling...\n");
    println!("  HTTPS gateway:  \x1b[36mhttp://{}\x1b[0m", addr);
    println!("  Burrow root:    \x1b[36m./burrows/\x1b[0m\n");
    println!("  Press Ctrl+C to fill in the hole.\n");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn home() -> Html<String> {
    let burrows = list_burrows();
    Html(render::home_page(&burrows))
}

async fn serve_burrow(Path(path): Path<String>) -> Html<String> {
    let fs_path = PathBuf::from("burrows").join(&path);

    if fs_path.is_dir() {
        let entries = list_directory(&fs_path);
        Html(render::directory_page(&path, &entries))
    } else if fs_path.exists() {
        let content = fs::read_to_string(&fs_path).unwrap_or_default();
        let filename = fs_path.file_name().unwrap().to_str().unwrap();
        Html(render::text_page(&path, filename, &content))
    } else if fs_path.with_extension("txt").exists() {
        let real_path = fs_path.with_extension("txt");
        let content = fs::read_to_string(&real_path).unwrap_or_default();
        let filename = real_path.file_name().unwrap().to_str().unwrap();
        Html(render::text_page(&path, filename, &content))
    } else {
        Html(render::not_found_page(&path))
    }
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
    Link,
}

fn list_burrows() -> Vec<BurrowEntry> {
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

fn list_directory(dir: &PathBuf) -> Vec<BurrowEntry> {
    let mut entries = Vec::new();
    if let Ok(items) = fs::read_dir(dir) {
        for item in items.flatten() {
            let name = item.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || name.starts_with('_') {
                continue;
            }
            let path = item.path();
            let relative = path.strip_prefix("burrows").unwrap().to_string_lossy().to_string();

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

fn read_description(dir: &PathBuf) -> String {
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

fn first_line_of(path: &PathBuf) -> String {
    fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .next()
        .unwrap_or("")
        .trim_start_matches("# ")
        .to_string()
}
