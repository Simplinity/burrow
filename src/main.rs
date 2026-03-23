use axum::{
<<<<<<< HEAD
    extract::{ConnectInfo, Form, Path, Query, State},
    http::{header, HeaderMap, StatusCode},
=======
    extract::{Form, Path, State},
    http::{header, StatusCode},
>>>>>>> 5c5546a (docs: rewrite CHANGELOG for v0.2.0, remove todos.md)
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

fn extract_host(headers: &HeaderMap) -> Option<String> {
    headers.get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

const MAX_FILE_SIZE: u64 = 65_536; // 64 KB
const MAX_DIR_ENTRIES: usize = 256;
const MAX_PATH_DEPTH: usize = 8;
<<<<<<< HEAD
const GUESTBOOK_RATE_SECS: u64 = 30;
const FIREHOSE_PAGE_SIZE: usize = 20;

#[derive(Clone)]
struct AppState {
    config: Arc<config::ServerConfig>,
    domain: Arc<String>,
    guestbook_limiter: Arc<Mutex<HashMap<IpAddr, Instant>>>,
    started_at: Instant,
    search_index: Arc<SearchIndex>,
}

// ── Search Index ────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SearchDoc {
    pub path: String,        // /~bruno/phlog/2026-03-10-the-weight-of-a-webpage
    pub title: String,
    pub author: String,      // ~bruno
    pub content: String,     // full text for snippet extraction
    pub doc_type: String,    // "phlog", "page", "guestbook", "gallery"
    pub date: String,        // YYYY-MM-DD or empty
    pub word_count: usize,
}

#[derive(Debug)]
pub struct SearchIndex {
    docs: Vec<SearchDoc>,
    // Inverted index: term → Vec<(doc_index, term_frequency)>
    index: HashMap<String, Vec<(usize, f64)>>,
    avg_doc_len: f64,
}

impl Clone for SearchIndex {
    fn clone(&self) -> Self {
        Self {
            docs: self.docs.clone(),
            index: self.index.clone(),
            avg_doc_len: self.avg_doc_len,
        }
    }
}

impl SearchIndex {
    async fn build() -> Self {
        let mut docs = Vec::new();

        // Scan all burrows
        if let Ok(mut dirs) = fs::read_dir("burrows").await {
            while let Ok(Some(burrow_entry)) = dirs.next_entry().await {
                let author = burrow_entry.file_name().to_string_lossy().to_string();
                if !author.starts_with('~') || !burrow_entry.path().is_dir() {
                    continue;
                }
                // Index all .txt and .gph files recursively
                Self::index_dir(&burrow_entry.path(), &author, &mut docs).await;
            }
        }

        // Build inverted index
        let mut index: HashMap<String, Vec<(usize, f64)>> = HashMap::new();
        let total_words: usize = docs.iter().map(|d| d.word_count).sum();
        let avg_doc_len = if docs.is_empty() { 1.0 } else { total_words as f64 / docs.len() as f64 };

        for (doc_idx, doc) in docs.iter().enumerate() {
            let mut term_counts: HashMap<String, usize> = HashMap::new();
            for word in Self::tokenize(&doc.content) {
                *term_counts.entry(word).or_insert(0) += 1;
            }
            // Also index title terms with a boost
            for word in Self::tokenize(&doc.title) {
                *term_counts.entry(word).or_insert(0) += 3; // title boost
            }

            let total_terms = term_counts.values().sum::<usize>() as f64;
            for (term, count) in term_counts {
                let tf = count as f64 / total_terms.max(1.0);
                index.entry(term).or_default().push((doc_idx, tf));
            }
        }

        tracing::info!("Search index built: {} documents, {} terms", docs.len(), index.len());

        Self { docs, index, avg_doc_len }
    }

    async fn index_dir(dir: &path::Path, author: &str, docs: &mut Vec<SearchDoc>) {
        if let Ok(mut entries) = fs::read_dir(dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') || name.starts_with('_') {
                    continue;
                }
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    Box::pin(Self::index_dir(&entry_path, author, docs)).await;
                    continue;
                }
                if !name.ends_with(".txt") && !name.ends_with(".gph") {
                    continue;
                }

                let content = match fs::read_to_string(&entry_path).await {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                let title = content.lines().next().unwrap_or("")
                    .trim_start_matches("# ").to_string();

                // Determine type and path
                let rel = entry_path.strip_prefix("burrows")
                    .unwrap_or(&entry_path)
                    .to_string_lossy()
                    .to_string();
                let slug = rel.trim_end_matches(".txt").trim_end_matches(".gph");
                let url_path = format!("/{}", slug);

                let doc_type = if rel.contains("/phlog/") {
                    "phlog"
                } else if rel.contains("/gallery/") {
                    "gallery"
                } else if name == "guestbook.gph" {
                    "guestbook"
                } else {
                    "page"
                };

                // Extract date from phlog filename
                let date = if doc_type == "phlog" && name.len() >= 10 {
                    name[..10].to_string()
                } else {
                    String::new()
                };

                let word_count = content.split_whitespace().count();

                docs.push(SearchDoc {
                    path: url_path,
                    title,
                    author: author.to_string(),
                    content,
                    doc_type: doc_type.to_string(),
                    date,
                    word_count,
                });
            }
        }
    }

    fn tokenize(text: &str) -> Vec<String> {
        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() >= 2)
            .map(|w| w.to_string())
            .collect()
    }

    /// Search with BM25 ranking + freshness boost.
    /// Supports operators: author:~name, type:phlog, fresh:7 (days)
    fn search(&self, query: &str) -> Vec<SearchResult> {
        let (terms, filters) = Self::parse_query(query);

        if terms.is_empty() {
            return Vec::new();
        }

        let n = self.docs.len() as f64;
        let k1 = 1.5_f64;
        let b = 0.75_f64;

        let mut scores: HashMap<usize, f64> = HashMap::new();

        for term in &terms {
            if let Some(postings) = self.index.get(term) {
                let df = postings.len() as f64;
                let idf = ((n - df + 0.5) / (df + 0.5) + 1.0).ln();

                for &(doc_idx, tf) in postings {
                    let doc = &self.docs[doc_idx];
                    let dl = doc.word_count as f64;
                    let bm25 = idf * (tf * (k1 + 1.0)) / (tf + k1 * (1.0 - b + b * dl / self.avg_doc_len));
                    *scores.entry(doc_idx).or_insert(0.0) += bm25;
                }
            }
        }

        // Apply freshness boost: recent docs get up to 1.5x score
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        for (doc_idx, score) in scores.iter_mut() {
            let doc = &self.docs[*doc_idx];
            if !doc.date.is_empty() {
                let age_days = date_diff_days(&doc.date, &today);
                // Boost: 1.5x for today, decays to 1.0x over 90 days
                let freshness = 1.0 + 0.5 * (1.0 - (age_days as f64 / 90.0).min(1.0));
                *score *= freshness;
            }
        }

        // Filter and collect results
        let mut results: Vec<SearchResult> = scores.into_iter()
            .filter(|(doc_idx, _)| {
                let doc = &self.docs[*doc_idx];
                Self::matches_filters(doc, &filters)
            })
            .map(|(doc_idx, score)| {
                let doc = &self.docs[doc_idx];
                let snippet = Self::extract_snippet(&doc.content, &terms);
                SearchResult {
                    path: doc.path.clone(),
                    title: doc.title.clone(),
                    author: doc.author.clone(),
                    snippet,
                    score,
                    date: doc.date.clone(),
                    doc_type: doc.doc_type.clone(),
                }
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(20);
        results
    }

    fn parse_query(query: &str) -> (Vec<String>, SearchFilters) {
        let mut terms = Vec::new();
        let mut filters = SearchFilters::default();

        for part in query.split_whitespace() {
            if let Some(val) = part.strip_prefix("author:") {
                filters.author = Some(val.to_string());
            } else if let Some(val) = part.strip_prefix("type:") {
                filters.doc_type = Some(val.to_string());
            } else if let Some(val) = part.strip_prefix("fresh:") {
                if let Ok(days) = val.parse::<u32>() {
                    filters.fresh_days = Some(days);
                }
            } else if let Some(val) = part.strip_prefix("ring:") {
                filters.ring = Some(val.to_string());
            } else {
                let word = part.to_lowercase();
                if word.len() >= 2 {
                    terms.push(word);
                }
            }
        }

        (terms, filters)
    }

    fn matches_filters(doc: &SearchDoc, filters: &SearchFilters) -> bool {
        if let Some(ref author) = filters.author {
            if !doc.author.contains(author) {
                return false;
            }
        }
        if let Some(ref doc_type) = filters.doc_type {
            if doc.doc_type != *doc_type {
                return false;
            }
        }
        if let Some(days) = filters.fresh_days {
            if doc.date.is_empty() {
                return false;
            }
            let today = chrono::Local::now().format("%Y-%m-%d").to_string();
            if date_diff_days(&doc.date, &today) > days as i64 {
                return false;
            }
        }
        true
    }

    fn extract_snippet(content: &str, terms: &[String]) -> String {
        // Find the first line containing any search term
        for line in content.lines().skip(1) { // skip title
            let line_lower = line.to_lowercase();
            if terms.iter().any(|t| line_lower.contains(t)) {
                let trimmed = line.trim();
                return if trimmed.len() > 150 {
                    format!("{}...", &trimmed[..147])
                } else {
                    trimmed.to_string()
                };
            }
        }
        // Fallback: first non-empty line after title
        content.lines().skip(1)
            .find(|l| !l.trim().is_empty())
            .map(|l| {
                let t = l.trim();
                if t.len() > 150 { format!("{}...", &t[..147]) } else { t.to_string() }
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Default)]
struct SearchFilters {
    author: Option<String>,
    doc_type: Option<String>,
    fresh_days: Option<u32>,
    ring: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub path: String,
    pub title: String,
    pub author: String,
    pub snippet: String,
    pub score: f64,
    pub date: String,
    pub doc_type: String,
}

fn date_diff_days(date_str: &str, today_str: &str) -> i64 {
    use chrono::NaiveDate;
    let d = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap_or(NaiveDate::from_ymd_opt(2000, 1, 1).unwrap());
    let t = NaiveDate::parse_from_str(today_str, "%Y-%m-%d").unwrap_or(NaiveDate::from_ymd_opt(2000, 1, 1).unwrap());
    (t - d).num_days().abs()
}
=======
>>>>>>> 5c5546a (docs: rewrite CHANGELOG for v0.2.0, remove todos.md)

#[tokio::main]
async fn main() {
    let cfg = config::ServerConfig::load();
    let domain = Arc::new(cfg.domain.clone());
    let addr = cfg.bind_addr();

<<<<<<< HEAD
    // Build search index
    let search_index = SearchIndex::build().await;

    let state = AppState {
        config: Arc::new(cfg.clone()),
        domain: Arc::new(cfg.domain.clone()),
        guestbook_limiter: Arc::new(Mutex::new(HashMap::new())),
        started_at: Instant::now(),
        search_index: Arc::new(search_index),
    };

=======
>>>>>>> 5c5546a (docs: rewrite CHANGELOG for v0.2.0, remove todos.md)
    let app = Router::new()
        .route("/", get(home))
        .route("/{*path}", get(serve_burrow).post(post_guestbook))
        .with_state(domain);

    println!("\n  \x1b[1m/\x1b[0m burrow v0.1.0\n");
    println!("  Tunneling...\n");
    println!("  Domain:         \x1b[36m{}\x1b[0m", cfg.domain);
<<<<<<< HEAD
    if !cfg.aliases.is_empty() {
        println!("  Aliases:        \x1b[36m{}\x1b[0m", cfg.aliases.join(", "));
    }
=======
    println!("  HTTPS gateway:  \x1b[36mhttp://{}\x1b[0m", addr);
    println!("  Burrow root:    \x1b[36m./burrows/\x1b[0m\n");
    println!("  Press Ctrl+C to fill in the hole.\n");
>>>>>>> 5c5546a (docs: rewrite CHANGELOG for v0.2.0, remove todos.md)

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

<<<<<<< HEAD
async fn home(headers: HeaderMap, State(state): State<AppState>) -> Html<String> {
    let domain = state.config.resolve_domain(extract_host(&headers).as_deref());
    let burrows = list_burrows().await;
=======
async fn home(State(domain): State<Arc<String>>) -> Html<String> {
    let burrows = list_burrows();
>>>>>>> 5c5546a (docs: rewrite CHANGELOG for v0.2.0, remove todos.md)
    Html(render::home_page(&burrows, &domain))
}

fn inject_accent(html: String, path: &str) -> String {
    let accent = accent_for_path(path);
    if accent.is_none() { return html; }
    html.replace("</head>", &format!("{}</head>", render::accent_style(accent.as_deref())))
}

async fn serve_burrow(Path(path): Path<String>, State(domain): State<Arc<String>>) -> Response {
    let burrows_root = fs::canonicalize("burrows").unwrap_or_else(|_| path::PathBuf::from("burrows"));

<<<<<<< HEAD
async fn stats(State(state): State<AppState>) -> impl IntoResponse {
    let burrows = list_burrows().await;
    let burrow_count = burrows.len();
    let mut file_count: usize = 0;
    for burrow in &burrows {
        let dir = path::PathBuf::from("burrows").join(burrow.name.trim_end_matches('/'));
        file_count += count_files_recursive(&dir).await;
    }
    let uptime_secs = state.started_at.elapsed().as_secs();
    let json = format!(
        "{{\"burrows\":{},\"files\":{},\"uptime_secs\":{}}}",
        burrow_count, file_count, uptime_secs
    );
    ([(header::CONTENT_TYPE, "application/json")], json)
}

async fn count_files_recursive(dir: &path::Path) -> usize {
    let mut count = 0;
    if let Ok(mut entries) = fs::read_dir(dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || name.starts_with('_') {
                continue;
            }
            if entry.path().is_dir() {
                count += Box::pin(count_files_recursive(&entry.path())).await;
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

async fn firehose(headers: HeaderMap, State(state): State<AppState>, Query(params): Query<PaginationParams>) -> Html<String> {
    let domain = state.config.resolve_domain(extract_host(&headers).as_deref());
    let burrows = list_burrows().await;
    let mut posts: Vec<(String, String, String, String)> = Vec::new();

    for burrow in &burrows {
        let burrow_name = burrow.name.trim_end_matches('/');
        let phlog_dir = path::PathBuf::from("burrows").join(burrow_name).join("phlog");
        if !fs::try_exists(&phlog_dir).await.unwrap_or(false) {
            continue;
        }
        if let Ok(mut entries) = fs::read_dir(&phlog_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') || name.starts_with('_') || !name.ends_with(".txt") {
                    continue;
                }
                let content = fs::read_to_string(entry.path()).await.unwrap_or_default();
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

    Html(render::firehose_page(&page_posts, &burrows, &domain, prev, next))
}

async fn random_burrow() -> Response {
    let burrows = list_burrows().await;
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

async fn discover(headers: HeaderMap, State(state): State<AppState>) -> Html<String> {
    let domain = state.config.resolve_domain(extract_host(&headers).as_deref());
    let burrows = list_burrows().await;

    // Gather all posts across all burrows
    let mut all_posts: Vec<(String, String, String, String)> = Vec::new();
    for burrow in &burrows {
        let burrow_name = burrow.name.trim_end_matches('/');
        let phlog_dir = path::PathBuf::from("burrows").join(burrow_name).join("phlog");
        if !fs::try_exists(&phlog_dir).await.unwrap_or(false) {
            continue;
        }
        if let Ok(mut entries) = fs::read_dir(&phlog_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') || name.starts_with('_') || !name.ends_with(".txt") {
                    continue;
                }
                let content = fs::read_to_string(entry.path()).await.unwrap_or_default();
                let title = content.lines().next().unwrap_or("").trim_start_matches("# ").to_string();
                let slug = name.trim_end_matches(".txt");
                let date = if name.len() >= 10 { name[..10].to_string() } else { String::new() };
                let url_path = format!("/{}/phlog/{}", burrow_name, slug);
                all_posts.push((date, title, burrow_name.to_string(), url_path));
            }
        }
    }
    all_posts.sort_by(|a, b| b.0.cmp(&a.0));

    // Latest 5 posts
    let latest: Vec<_> = all_posts.iter().take(5).cloned().collect();

    // Random burrow pick
    let random_pick = if !burrows.is_empty() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos() as usize;
        Some(&burrows[nanos % burrows.len()])
    } else {
        None
    };

    // Count bookmark mentions across all burrows
    let popular = count_bookmark_mentions(&burrows).await;
    let rings = load_all_rings().await;

    Html(render::discover_page(&burrows, &latest, &popular, &rings, random_pick, &domain))
}

#[derive(Deserialize)]
struct SearchParams {
    q: Option<String>,
}

async fn search_handler(headers: HeaderMap, State(state): State<AppState>, Query(params): Query<SearchParams>) -> Html<String> {
    let domain = state.config.resolve_domain(extract_host(&headers).as_deref());
    let burrows = list_burrows().await;
    let query = params.q.unwrap_or_default();

    let results = if query.is_empty() {
        Vec::new()
    } else {
        state.search_index.search(&query)
    };

    Html(render::search_page(&query, &results, &burrows, &domain))
}

async fn search_index_json(headers: HeaderMap, State(state): State<AppState>) -> impl IntoResponse {
    let domain = state.config.resolve_domain(extract_host(&headers).as_deref());
    let mut json = String::from("{\"version\":1,\"server\":\"");
    json.push_str(&domain.replace('"', "\\\""));
    json.push_str("\",\"documents\":[");

    for (i, doc) in state.search_index.docs.iter().enumerate() {
        if i > 0 { json.push(','); }
        json.push_str(&format!(
            "{{\"path\":\"{}\",\"title\":\"{}\",\"author\":\"{}\",\"type\":\"{}\",\"date\":\"{}\",\"words\":{}}}",
            doc.path.replace('"', "\\\""),
            doc.title.replace('"', "\\\""),
            doc.author.replace('"', "\\\""),
            doc.doc_type,
            doc.date,
            doc.word_count,
        ));
    }
    json.push_str("]}");

    ([(header::CONTENT_TYPE, "application/json; charset=utf-8")], json)
}

async fn rings_page(headers: HeaderMap, State(state): State<AppState>) -> Html<String> {
    let domain = state.config.resolve_domain(extract_host(&headers).as_deref());
    let burrows = list_burrows().await;
    let rings = load_all_rings().await;
    Html(render::rings_list_page(&rings, &burrows, &domain))
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

async fn serve_burrow(headers: HeaderMap, Path(path): Path<String>, State(state): State<AppState>) -> Response {
    let domain = state.config.resolve_domain(extract_host(&headers).as_deref());
    let burrows_root = fs::canonicalize("burrows").await.unwrap_or_else(|_| path::PathBuf::from("burrows"));

    // Virtual routes: feeds are generated, not real files
=======
    // Virtual routes: feed.xml is generated, not a real file
>>>>>>> 5c5546a (docs: rewrite CHANGELOG for v0.2.0, remove todos.md)
    if path.ends_with("/feed.xml") || path.ends_with("/feed") {
        let burrow_name = path.split('/').next().unwrap_or("");
        let burrow_dir = path::PathBuf::from("burrows").join(burrow_name);
        if burrow_dir.is_dir() && burrow_name.starts_with('~') {
            let xml = generate_feed(burrow_name, &burrow_dir, &domain);
            return ([(header::CONTENT_TYPE, "application/rss+xml; charset=utf-8")], xml).into_response();
        }
        return (StatusCode::NOT_FOUND, Html(render::not_found_page(&path, &domain))).into_response();
    }

    // Draft protection: any path segment starting with _ is hidden
    if path.split('/').any(|seg| seg.starts_with('_')) {
        return (StatusCode::NOT_FOUND, Html(render::not_found_page(&path, &domain))).into_response();
    }

    // Depth limit: max 8 levels below ~user (e.g. ~user/a/b/c/d/e/f/g/h)
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if segments.len() > MAX_PATH_DEPTH + 1 {  // +1 for ~user itself
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
                    return Html(inject_accent(render::text_page(&path, filename, &content, &domain), &path)).into_response();
                }
            }
            if let Ok(p) = fs::canonicalize(&with_gph) {
                if p.starts_with(&burrows_root) {
                    let content = read_file_checked(&p);
                    let filename = p.file_name().unwrap().to_str().unwrap();
                    if filename == "guestbook.gph" {
                        let entries = parse_guestbook(&content);
                        return Html(inject_accent(render::guestbook_page(&path, &entries, &domain), &path)).into_response();
                    }
                    return Html(inject_accent(render::text_page(&path, filename, &content, &domain), &path)).into_response();
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
        Html(inject_accent(render::directory_page(&path, &entries, &burrows, &domain), &path)).into_response()
    } else {
        let content = read_file_checked(&canonical);
        let filename = canonical.file_name().unwrap().to_str().unwrap();
        if filename == "guestbook.gph" {
            let entries = parse_guestbook(&content);
            Html(inject_accent(render::guestbook_page(&path, &entries, &domain), &path)).into_response()
        } else {
            Html(inject_accent(render::text_page(&path, filename, &content, &domain), &path)).into_response()
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
    entries.truncate(MAX_DIR_ENTRIES);
    entries
}

fn read_description(dir: &path::Path) -> String {
    read_burrow_field(dir, "description")
}

pub fn read_accent(dir: &path::Path) -> Option<String> {
    let val = read_burrow_field(dir, "accent");
    if val.is_empty() { None } else { Some(val) }
}

fn read_burrow_field(dir: &path::Path, key: &str) -> String {
    let burrow_file = dir.join(".burrow");
    if burrow_file.exists() {
        if let Ok(content) = fs::read_to_string(&burrow_file) {
            let prefix = format!("{} = ", key);
            for line in content.lines() {
                if let Some(val) = line.strip_prefix(&prefix) {
                    return val.to_string();
                }
            }
        }
    }
    String::new()
}

/// Read accent color for a URL path by extracting the ~user segment
pub fn accent_for_path(path: &str) -> Option<String> {
    let burrow_name = path.split('/').find(|s| s.starts_with('~'))?;
    let burrow_dir = path::PathBuf::from("burrows").join(burrow_name);
    read_accent(&burrow_dir)
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
<<<<<<< HEAD
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
=======
>>>>>>> 5c5546a (docs: rewrite CHANGELOG for v0.2.0, remove todos.md)
    Path(path): Path<String>,
    State(domain): State<Arc<String>>,
    Form(form): Form<GuestbookForm>,
) -> Response {
<<<<<<< HEAD
    let domain = state.config.resolve_domain(extract_host(&headers).as_deref());

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
=======
    // Draft protection
    if path.split('/').any(|seg| seg.starts_with('_')) {
        return (StatusCode::NOT_FOUND, Html(render::not_found_page(&path, &domain))).into_response();
>>>>>>> 5c5546a (docs: rewrite CHANGELOG for v0.2.0, remove todos.md)
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
