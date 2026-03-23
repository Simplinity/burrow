use axum::{
    extract::{ConnectInfo, Form, Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
    Router,
};
use chrono::Local;
use serde::Deserialize;
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::path;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tower_http::compression::CompressionLayer;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use burrow::config;
mod render;

fn extract_host(headers: &HeaderMap) -> Option<String> {
    headers.get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

const MAX_FILE_SIZE: u64 = 65_536; // 64 KB
const MAX_PATH_DEPTH: usize = 8;
const GUESTBOOK_RATE_SECS: u64 = 30;
const FIREHOSE_PAGE_SIZE: usize = 20;

#[derive(Clone)]
struct AppState {
    config: Arc<std::sync::RwLock<Arc<config::ServerConfig>>>,
    domain: Arc<String>,
    guestbook_limiter: Arc<Mutex<HashMap<IpAddr, Instant>>>,
    started_at: Instant,
    search_index: Arc<std::sync::RwLock<Arc<SearchIndex>>>,
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
                return truncate_chars(trimmed, 150);
            }
        }
        // Fallback: first non-empty line after title
        content.lines().skip(1)
            .find(|l| !l.trim().is_empty())
            .map(|l| truncate_chars(l.trim(), 150))
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

#[tokio::main]
async fn main() {
    // Initialize tracing (RUST_LOG env var controls level, default: info)
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .compact()
        .init();

    let cfg = config::ServerConfig::load();
    let addr = cfg.bind_addr();

    // Build search index
    let search_index = SearchIndex::build().await;

    let state = AppState {
        config: Arc::new(std::sync::RwLock::new(Arc::new(cfg.clone()))),
        domain: Arc::new(cfg.domain.clone()),
        guestbook_limiter: Arc::new(Mutex::new(HashMap::new())),
        started_at: Instant::now(),
        search_index: Arc::new(std::sync::RwLock::new(Arc::new(search_index))),
    };

    let app = Router::new()
        .route("/", get(home))
        .route("/robots.txt", get(robots_txt))
        .route("/favicon.ico", get(favicon_ico))
        .route("/health", get(health))
        .route("/stats", get(stats))
        .route("/firehose", get(firehose))
        .route("/random", get(random_burrow))
        .route("/discover", get(discover))
        .route("/rings", get(rings_page))
        .route("/servers", get(servers_page))
        .route("/search", get(search_handler))
        .route("/search/index.json", get(search_index_json))
        .route("/ping", axum::routing::post(receive_ping))
        .route("/.well-known/{*path}", get(well_known))
        .route("/{*path}", get(serve_burrow).post(post_guestbook))
        .layer(TraceLayer::new_for_http())
        .layer(SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("content-security-policy"),
            header::HeaderValue::from_static(
                "default-src 'none'; style-src 'unsafe-inline' https://fonts.googleapis.com; font-src https://fonts.gstatic.com; img-src 'self'; frame-ancestors 'none'"
            ),
        ));

    let app = if cfg.compression {
        app.layer(CompressionLayer::new()).with_state(state.clone())
    } else {
        app.with_state(state.clone())
    };

    let local_ip = local_ip_address().unwrap_or_else(|| "127.0.0.1".to_string());

    println!("\n  \x1b[1m/\x1b[0m burrow v0.1.0\n");
    println!("  Tunneling...\n");
    println!("  Domain:         \x1b[36m{}\x1b[0m", cfg.domain);
    if !cfg.aliases.is_empty() {
        println!("  Aliases:        \x1b[36m{}\x1b[0m", cfg.aliases.join(", "));
    }
    if cfg.compression {
        println!("  Compression:    \x1b[36menabled (gzip/brotli)\x1b[0m");
    }

    // Spawn Gemini listener if configured
    if cfg.has_gemini() {
        let gemini_addr = cfg.gemini_bind_addr().unwrap();
        let cert = cfg.tls_cert.clone().unwrap();
        let key = cfg.tls_key.clone().unwrap();
        println!("  Gemini:         \x1b[36mgemini://{}\x1b[0m", gemini_addr);
        tokio::spawn(gemini_listener(state.clone(), gemini_addr, cert, key));
    }

    // Spawn outgoing federation pings (background, fire-and-forget)
    let ping_domain = cfg.domain.clone();
    tokio::spawn(async move {
        send_outgoing_pings(&ping_domain).await;
    });

    // SIGHUP handler: reload config and search index without restart
    #[cfg(unix)]
    {
        let reload_state = state.clone();
        tokio::spawn(async move {
            use tokio::signal::unix::{signal, SignalKind};
            let mut hup = signal(SignalKind::hangup()).expect("failed to listen for SIGHUP");
            loop {
                hup.recv().await;
                tracing::info!("SIGHUP received — reloading config and search index");
                // Reload config
                let new_cfg = config::ServerConfig::load();
                *reload_state.config.write().unwrap() = Arc::new(new_cfg);
                // Rebuild search index
                let new_index = SearchIndex::build().await;
                *reload_state.search_index.write().unwrap() = Arc::new(new_index);
                tracing::info!("Reload complete");
            }
        });
    }

    if cfg.has_tls() {
        let cert_path = cfg.tls_cert.as_ref().unwrap();
        let key_path = cfg.tls_key.as_ref().unwrap();

        println!("  Local:          \x1b[36mhttps://localhost:{}\x1b[0m", cfg.port);
        println!("  Network:        \x1b[36mhttps://{}:{}\x1b[0m", local_ip, cfg.port);
        println!("  TLS cert:       \x1b[36m{}\x1b[0m", cert_path);
        println!("  TLS key:        \x1b[36m{}\x1b[0m", key_path);
        println!("  Burrow root:    \x1b[36m./burrows/\x1b[0m\n");
        println!("  Press Ctrl+C to fill in the hole.\n");

        let tls_config = axum_server::tls_rustls::RustlsConfig::from_pem_file(cert_path, key_path)
            .await
            .unwrap_or_else(|e| {
                eprintln!("  \x1b[31m✗\x1b[0m Failed to load TLS certificates: {}", e);
                eprintln!("    cert: {}", cert_path);
                eprintln!("    key:  {}", key_path);
                std::process::exit(1);
            });

        let addr: SocketAddr = addr.parse().unwrap();
        axum_server::bind_rustls(addr, tls_config)
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .await
            .unwrap();
    } else {
        println!("  Local:          \x1b[36mhttp://localhost:{}\x1b[0m", cfg.port);
        println!("  Network:        \x1b[36mhttp://{}:{}\x1b[0m", local_ip, cfg.port);
        println!("  Burrow root:    \x1b[36m./burrows/\x1b[0m\n");
        if !cfg.has_gemini() {
            println!("  \x1b[90mTip: Add tls_cert, tls_key, and gemini_port to burrow.conf for Gemini\x1b[0m\n");
        }
        println!("  Press Ctrl+C to fill in the hole.\n");

        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
    }
}

async fn home(headers: HeaderMap, State(state): State<AppState>) -> Html<String> {
    let domain = state.config.read().unwrap().resolve_domain(extract_host(&headers).as_deref());
    let burrows = list_burrows().await;
    Html(render::home_page(&burrows, &domain))
}

async fn robots_txt() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], "User-agent: *\nAllow: /\n")
}

async fn health() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "application/json")], "{\"status\":\"ok\"}")
}

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
    let domain = state.config.read().unwrap().resolve_domain(extract_host(&headers).as_deref());
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
    let domain = state.config.read().unwrap().resolve_domain(extract_host(&headers).as_deref());
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
    let domain = state.config.read().unwrap().resolve_domain(extract_host(&headers).as_deref());
    let burrows = list_burrows().await;
    let query = params.q.unwrap_or_default();

    let results = if query.is_empty() {
        Vec::new()
    } else {
        state.search_index.read().unwrap().search(&query)
    };

    Html(render::search_page(&query, &results, &burrows, &domain))
}

async fn search_index_json(headers: HeaderMap, State(state): State<AppState>) -> impl IntoResponse {
    let domain = state.config.read().unwrap().resolve_domain(extract_host(&headers).as_deref());
    let mut json = String::from("{\"version\":1,\"server\":\"");
    json.push_str(&domain.replace('"', "\\\""));
    json.push_str("\",\"documents\":[");

    let idx = state.search_index.read().unwrap().clone();
    for (i, doc) in idx.docs.iter().enumerate() {
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
    let domain = state.config.read().unwrap().resolve_domain(extract_host(&headers).as_deref());
    let burrows = list_burrows().await;
    let rings = load_all_rings().await;
    Html(render::rings_list_page(&rings, &burrows, &domain))
}

async fn servers_page(headers: HeaderMap, State(state): State<AppState>) -> Html<String> {
    let domain = state.config.read().unwrap().resolve_domain(extract_host(&headers).as_deref());
    let burrows = list_burrows().await;
    let servers = load_servers().await;
    Html(render::servers_page(&servers, &burrows, &domain))
}

async fn well_known(Path(path): Path<String>) -> Response {
    // Sanitize: no path traversal, no subdirectories beyond one level
    if path.contains("..") || path.contains('/') {
        return StatusCode::NOT_FOUND.into_response();
    }
    let file_path = path::PathBuf::from("burrows/.well-known").join(&path);
    if let Ok(content) = fs::read_to_string(&file_path).await {
        ([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], content).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
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

#[derive(Deserialize)]
struct BurrowParams {
    slow: Option<u8>,
}

async fn serve_burrow(headers: HeaderMap, Path(path): Path<String>, Query(params): Query<BurrowParams>, State(state): State<AppState>) -> Response {
    let slow = params.slow.unwrap_or(0) == 1;
    let domain = state.config.read().unwrap().resolve_domain(extract_host(&headers).as_deref());
    let burrows_root = fs::canonicalize("burrows").await.unwrap_or_else(|_| path::PathBuf::from("burrows"));

    // Virtual routes: feeds are generated, not real files
    if path.ends_with("/feed.xml") || path.ends_with("/feed") {
        let burrow_name = path.split('/').next().unwrap_or("");
        let burrow_dir = path::PathBuf::from("burrows").join(burrow_name);
        if fs::try_exists(&burrow_dir).await.unwrap_or(false) && burrow_name.starts_with('~') {
            let xml = generate_feed(burrow_name, &burrow_dir, &domain).await;
            return ([(header::CONTENT_TYPE, "application/rss+xml; charset=utf-8")], xml).into_response();
        }
        return (StatusCode::NOT_FOUND, Html(render::not_found_page(&path, &domain))).into_response();
    }
    if path.ends_with("/atom.xml") || path.ends_with("/atom") {
        let burrow_name = path.split('/').next().unwrap_or("");
        let burrow_dir = path::PathBuf::from("burrows").join(burrow_name);
        if fs::try_exists(&burrow_dir).await.unwrap_or(false) && burrow_name.starts_with('~') {
            let xml = generate_atom_feed(burrow_name, &burrow_dir, &domain).await;
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
    let accent = match path.split('/').next().filter(|s| s.starts_with('~')) {
        Some(b) => read_accent(&path::PathBuf::from("burrows").join(b)).await,
        None => None,
    };

    let fs_path = path::PathBuf::from("burrows").join(&path);

    // Path traversal protection: canonicalize and verify prefix
    let canonical = match fs::canonicalize(&fs_path).await {
        Ok(p) => p,
        Err(_) => {
            // File doesn't exist — try with .txt then .gph extension
            let with_txt = fs_path.with_extension("txt");
            let with_gph = fs_path.with_extension("gph");
            if let Ok(p) = fs::canonicalize(&with_txt).await {
                if p.starts_with(&burrows_root) {
                    let etag = generate_etag(&p).await;
                    if let Some(ref e) = etag { if etag_matches(&headers, e) { return (StatusCode::NOT_MODIFIED, [(header::ETAG, e.clone())]).into_response(); } }
                    let content = read_file_checked(&p).await;
                    let filename = p.file_name().unwrap().to_str().unwrap();
                    if is_gallery_item(&path) {
                        return html_response_with_etag(&headers, render::art_page(&path, filename, &content, &domain, accent.as_deref()), etag);
                    }
                    let mut mentions = find_mentions_of(&path).await;
                    let remote_pings = load_received_pings(&format!("/{}", path)).await;
                    mentions.extend(remote_pings);
                    let burrow_name = path.split('/').next().unwrap_or("");
                    let all_rings = load_all_rings().await;
                    let rings = find_rings_for_burrow(&all_rings, burrow_name);
                    let series = detect_series(&p, &path).await;
                    let mut html = render::text_page_with_mentions(&path, filename, &content, &mentions, &rings, burrow_name, &domain, accent.as_deref(), series.as_ref());
                    if slow { html = inject_slow_mode(html); }
                    return html_response_with_etag(&headers, html, etag);
                }
            }
            if let Ok(p) = fs::canonicalize(&with_gph).await {
                if p.starts_with(&burrows_root) {
                    let etag = generate_etag(&p).await;
                    if let Some(ref e) = etag { if etag_matches(&headers, e) { return (StatusCode::NOT_MODIFIED, [(header::ETAG, e.clone())]).into_response(); } }
                    let content = read_file_checked(&p).await;
                    let filename = p.file_name().unwrap().to_str().unwrap();
                    if filename == "guestbook.gph" {
                        let entries = parse_guestbook(&content);
                        return html_response_with_etag(&headers, render::guestbook_page(&path, &entries, &domain, accent.as_deref()), etag);
                    }
                    if filename == "bookmarks.gph" {
                        let bookmarks = parse_bookmarks(&content);
                        return html_response_with_etag(&headers, render::bookmarks_page(&path, &bookmarks, &domain, accent.as_deref()), etag);
                    }
                    let mut html = render::text_page(&path, filename, &content, &domain, accent.as_deref());
                    if slow { html = inject_slow_mode(html); }
                    return html_response_with_etag(&headers, html, etag);
                }
            }
            return (StatusCode::NOT_FOUND, Html(render::not_found_page(&path, &domain))).into_response();
        }
    };

    if !canonical.starts_with(&burrows_root) {
        return (StatusCode::NOT_FOUND, Html(render::not_found_page(&path, &domain))).into_response();
    }

    if canonical.is_dir() {
        // Gallery: special rendering for gallery/ directories
        let dir_name = canonical.file_name().and_then(|f| f.to_str()).unwrap_or("");
        if dir_name == "gallery" {
            let pieces = load_gallery(&canonical).await;
            let burrows = list_burrows().await;
            return Html(render::gallery_page(&path, &pieces, &burrows, &domain, accent.as_deref())).into_response();
        }

        let burrows = list_burrows().await;
        let entries = list_directory(&canonical, &burrows_root).await;
        let title = read_title(&canonical).await;
        Html(render::directory_page(&path, title.as_deref(), &entries, &burrows, &domain, accent.as_deref())).into_response()
    } else {
        let filename = canonical.file_name().unwrap().to_str().unwrap();

        // Binary/image file serving — serve raw with correct MIME type
        if let Some(mime) = binary_mime_type(filename) {
            return serve_binary_file(&canonical, mime).await;
        }

        let etag = generate_etag(&canonical).await;
        if let Some(ref e) = etag { if etag_matches(&headers, e) { return (StatusCode::NOT_MODIFIED, [(header::ETAG, e.clone())]).into_response(); } }

        let content = read_file_checked(&canonical).await;
        if filename == "guestbook.gph" {
            let entries = parse_guestbook(&content);
            html_response_with_etag(&headers, render::guestbook_page(&path, &entries, &domain, accent.as_deref()), etag)
        } else if filename == "bookmarks.gph" {
            let bookmarks = parse_bookmarks(&content);
            html_response_with_etag(&headers, render::bookmarks_page(&path, &bookmarks, &domain, accent.as_deref()), etag)
        } else if is_gallery_item(&path) {
            html_response_with_etag(&headers, render::art_page(&path, filename, &content, &domain, accent.as_deref()), etag)
        } else {
            let mut mentions = find_mentions_of(&path).await;
            let remote_pings = load_received_pings(&format!("/{}", path)).await;
            mentions.extend(remote_pings);
            let burrow_name = path.split('/').next().unwrap_or("");
            let all_rings = load_all_rings().await;
            let rings = find_rings_for_burrow(&all_rings, burrow_name);
            let series = detect_series(&canonical, &path).await;
            let mut html = render::text_page_with_mentions(&path, filename, &content, &mentions, &rings, burrow_name, &domain, accent.as_deref(), series.as_ref());
            if slow { html = inject_slow_mode(html); }
            html_response_with_etag(&headers, html, etag)
        }
    }
}

const SLOW_CSS: &str = "<style>.reading{font-size:21px!important;line-height:1.9!important;max-width:580px!important;margin:0 auto!important}.reading p{margin:1.8em 0!important}.reading blockquote{margin:2em 0!important}.reading pre{margin:2em 0!important}.meta{margin-bottom:2em!important}</style>";

fn inject_slow_mode(html: String) -> String {
    html.replace("</head>", &format!("{}</head>", SLOW_CSS))
}

// ── Series Detection ─────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SeriesInfo {
    pub current: usize,
    pub total: usize,
    pub prev_path: Option<String>,
    pub next_path: Option<String>,
}

/// Detect if a file is part of a series (part-01.txt, part-02.txt, etc.)
/// Returns SeriesInfo if the file matches the pattern and siblings exist.
async fn detect_series(canonical: &path::Path, url_path: &str) -> Option<SeriesInfo> {
    let filename = canonical.file_name()?.to_str()?;
    let parent = canonical.parent()?;

    // Match patterns: part-01.txt, part-02.txt, etc.
    let stem = filename.strip_suffix(".txt")
        .or_else(|| filename.strip_suffix(".gph"))?;

    // Extract prefix and number: "part-01" → ("part-", 1)
    let (prefix, current_num) = extract_series_number(stem)?;

    // Scan directory for siblings with same prefix
    let mut parts: Vec<(usize, String)> = Vec::new();
    if let Ok(mut entries) = fs::read_dir(parent).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            let entry_stem = name.strip_suffix(".txt")
                .or_else(|| name.strip_suffix(".gph"));
            if let Some(s) = entry_stem {
                if let Some((p, num)) = extract_series_number(s) {
                    if p == prefix {
                        parts.push((num, name));
                    }
                }
            }
        }
    }

    // Only a series if there are at least 2 parts
    if parts.len() < 2 {
        return None;
    }

    parts.sort_by_key(|(n, _)| *n);
    let total = parts.len();
    let current_idx = parts.iter().position(|(n, _)| *n == current_num)?;
    let current = current_idx + 1; // 1-indexed for display

    // Build prev/next paths relative to the URL
    let url_parent = url_path.rsplit_once('/').map(|(p, _)| p).unwrap_or("");
    let prev_path = if current_idx > 0 {
        let prev_file = &parts[current_idx - 1].1;
        let prev_slug = prev_file.strip_suffix(".txt")
            .or_else(|| prev_file.strip_suffix(".gph"))
            .unwrap_or(prev_file);
        Some(format!("/{}/{}", url_parent, prev_slug))
    } else {
        None
    };
    let next_path = if current_idx < parts.len() - 1 {
        let next_file = &parts[current_idx + 1].1;
        let next_slug = next_file.strip_suffix(".txt")
            .or_else(|| next_file.strip_suffix(".gph"))
            .unwrap_or(next_file);
        Some(format!("/{}/{}", url_parent, next_slug))
    } else {
        None
    };

    Some(SeriesInfo { current, total, prev_path, next_path })
}

/// Extract series prefix and number from a stem like "part-01" → Some(("part-", 1))
fn extract_series_number(stem: &str) -> Option<(&str, usize)> {
    // Find the last group of digits preceded by a separator (- or _)
    for sep in ['-', '_'] {
        if let Some(pos) = stem.rfind(sep) {
            let prefix = &stem[..=pos]; // includes separator
            let num_str = &stem[pos + 1..];
            if !num_str.is_empty() && num_str.chars().all(|c| c.is_ascii_digit()) {
                if let Ok(num) = num_str.parse::<usize>() {
                    return Some((prefix, num));
                }
            }
        }
    }
    None
}

/// Truncate a string at a character boundary (safe for multi-byte UTF-8)
fn truncate_chars(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_chars - 3).collect();
        format!("{}...", truncated)
    }
}

const MAX_BINARY_SIZE: u64 = 2_097_152; // 2 MB for images/binaries

fn binary_mime_type(filename: &str) -> Option<&'static str> {
    let ext = filename.rsplit('.').next()?.to_lowercase();
    match ext.as_str() {
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "gif" => Some("image/gif"),
        "svg" => Some("image/svg+xml"),
        "webp" => Some("image/webp"),
        "ico" => Some("image/x-icon"),
        "pdf" => Some("application/pdf"),
        "mp3" => Some("audio/mpeg"),
        "ogg" => Some("audio/ogg"),
        "woff" | "woff2" => Some("font/woff2"),
        "zip" => Some("application/zip"),
        "tar" => Some("application/x-tar"),
        "gz" => Some("application/gzip"),
        _ => None,
    }
}

async fn serve_binary_file(path: &path::Path, mime: &str) -> Response {
    let size = fs::metadata(path).await.map(|m| m.len()).unwrap_or(0);
    if size > MAX_BINARY_SIZE {
        return (StatusCode::PAYLOAD_TOO_LARGE, "File too large").into_response();
    }
    match fs::read(path).await {
        Ok(bytes) => {
            (
                [
                    (header::CONTENT_TYPE, mime.to_string()),
                    (header::CACHE_CONTROL, "public, max-age=3600".to_string()),
                ],
                bytes,
            ).into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read file").into_response(),
    }
}

/// Generate an ETag from a file's modification time and size
async fn generate_etag(path: &path::Path) -> Option<String> {
    let meta = fs::metadata(path).await.ok()?;
    let modified = meta.modified().ok()?;
    let secs = modified.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs();
    let size = meta.len();
    Some(format!("\"{}{}\"", secs, size))
}

/// Check if client's If-None-Match header matches the ETag — return true if 304 should be sent
fn etag_matches(headers: &HeaderMap, etag: &str) -> bool {
    if let Some(inm) = headers.get(header::IF_NONE_MATCH) {
        if let Ok(val) = inm.to_str() {
            return val == etag || val == "*";
        }
    }
    false
}

/// Build an HTML response with ETag header attached.
fn html_response_with_etag(_headers: &HeaderMap, html: String, etag: Option<String>) -> Response {
    let mut response = Html(html).into_response();
    if let Some(etag_val) = etag {
        response.headers_mut().insert(header::ETAG, header::HeaderValue::from_str(&etag_val).unwrap());
    }
    response
}

async fn read_file_checked(path: &path::Path) -> String {
    let size = fs::metadata(path).await.map(|m| m.len()).unwrap_or(0);
    if size > MAX_FILE_SIZE {
        return format!(
            "This file is too large to display ({:.1} KB). Maximum is {} KB.",
            size as f64 / 1024.0,
            MAX_FILE_SIZE / 1024
        );
    }
    fs::read_to_string(path).await.unwrap_or_default()
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

pub async fn list_burrows() -> Vec<BurrowEntry> {
    let mut entries = Vec::new();
    if let Ok(mut dirs) = fs::read_dir("burrows").await {
        while let Ok(Some(entry)) = dirs.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('~') && entry.path().is_dir() {
                let desc = read_description(&entry.path()).await;
                let count = count_dir_items(&entry.path()).await;
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

async fn count_dir_items(dir: &path::Path) -> usize {
    let mut count = 0;
    if let Ok(mut entries) = fs::read_dir(dir).await {
        while let Ok(Some(_)) = entries.next_entry().await {
            count += 1;
        }
    }
    count
}

async fn list_directory(dir: &path::Path, burrows_root: &path::Path) -> Vec<BurrowEntry> {
    let mut entries = Vec::new();
    if let Ok(mut items) = fs::read_dir(dir).await {
        while let Ok(Some(item)) = items.next_entry().await {
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
                let desc = read_description(&path).await;
                let display_name = read_title(&path).await.unwrap_or_else(|| name.clone());
                let count = count_dir_items(&path).await;
                entries.push(BurrowEntry {
                    path: format!("/{}", relative),
                    name: format!("{}/", display_name),
                    entry_type: EntryType::Directory,
                    description: desc,
                    meta: if count == 1 { "1 item".to_string() } else { format!("{} items", count) },
                });
            } else {
                let size = fs::metadata(&path).await.map(|m| m.len()).unwrap_or(0);
                let size_str = if size < 1024 {
                    format!("{} B", size)
                } else {
                    format!("{:.1} KB", size as f64 / 1024.0)
                };
                entries.push(BurrowEntry {
                    path: format!("/{}", relative),
                    name: name.clone(),
                    entry_type: EntryType::Text,
                    description: first_line_of(&path).await,
                    meta: size_str,
                });
            }
        }
    }
    // Read sort and pin config
    let sort_mode = read_sort(dir).await;
    let pinned = read_pin(dir).await;

    // Apply sort
    match sort_mode.as_deref() {
        Some("name-asc") | None => {
            entries.sort_by(|a, b| {
                let type_ord = |e: &BurrowEntry| if e.entry_type == EntryType::Directory { 0 } else { 1 };
                type_ord(a).cmp(&type_ord(b)).then(a.name.cmp(&b.name))
            });
        }
        Some("name-desc") => {
            entries.sort_by(|a, b| {
                let type_ord = |e: &BurrowEntry| if e.entry_type == EntryType::Directory { 0 } else { 1 };
                type_ord(a).cmp(&type_ord(b)).then(b.name.cmp(&a.name))
            });
        }
        Some("modified-desc") => {
            // For phlog-style dirs: sort by filename descending (dates sort naturally)
            entries.sort_by(|a, b| b.name.cmp(&a.name));
        }
        Some("modified-asc") => {
            entries.sort_by(|a, b| a.name.cmp(&b.name));
        }
        _ => {
            entries.sort_by(|a, b| {
                let type_ord = |e: &BurrowEntry| if e.entry_type == EntryType::Directory { 0 } else { 1 };
                type_ord(a).cmp(&type_ord(b)).then(a.name.cmp(&b.name))
            });
        }
    }

    // Apply pin: move pinned items to the top, preserving pin order
    if !pinned.is_empty() {
        let mut pinned_entries = Vec::new();
        let mut rest = Vec::new();
        for entry in entries {
            let entry_name = entry.name.trim_end_matches('/');
            if pinned.iter().any(|p| p == entry_name || p == &entry.name) {
                pinned_entries.push(entry);
            } else {
                rest.push(entry);
            }
        }
        // Sort pinned entries by pin order
        pinned_entries.sort_by(|a, b| {
            let a_name = a.name.trim_end_matches('/');
            let b_name = b.name.trim_end_matches('/');
            let a_pos = pinned.iter().position(|p| p == a_name || p == &a.name).unwrap_or(usize::MAX);
            let b_pos = pinned.iter().position(|p| p == b_name || p == &b.name).unwrap_or(usize::MAX);
            a_pos.cmp(&b_pos)
        });
        pinned_entries.extend(rest);
        entries = pinned_entries;
    }

    entries
}

async fn read_description(dir: &path::Path) -> String {
    let burrow_file = dir.join(".burrow");
    if let Ok(content) = fs::read_to_string(&burrow_file).await {
        for line in content.lines() {
            if let Some(desc) = line.strip_prefix("description = ") {
                return desc.to_string();
            }
        }
    }
    String::new()
}

async fn read_title(dir: &path::Path) -> Option<String> {
    let burrow_file = dir.join(".burrow");
    if let Ok(content) = fs::read_to_string(burrow_file).await {
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

async fn read_sort(dir: &path::Path) -> Option<String> {
    let burrow_file = dir.join(".burrow");
    if let Ok(content) = fs::read_to_string(burrow_file).await {
        for line in content.lines() {
            if let Some(val) = line.strip_prefix("sort = ") {
                let val = val.trim();
                if !val.is_empty() {
                    return Some(val.to_string());
                }
            }
        }
    }
    None
}

async fn read_pin(dir: &path::Path) -> Vec<String> {
    let burrow_file = dir.join(".burrow");
    if let Ok(content) = fs::read_to_string(burrow_file).await {
        for line in content.lines() {
            if let Some(val) = line.strip_prefix("pin = ") {
                return val.split_whitespace().map(|s| s.to_string()).collect();
            }
        }
    }
    Vec::new()
}

async fn read_accent(dir: &path::Path) -> Option<String> {
    let burrow_file = dir.join(".burrow");
    if let Ok(content) = fs::read_to_string(burrow_file).await {
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

async fn first_line_of(path: &path::Path) -> String {
    fs::read_to_string(path)
        .await
        .unwrap_or_default()
        .lines()
        .next()
        .unwrap_or("")
        .trim_start_matches("# ")
        .to_string()
}

// ── RSS Feed ────────────────────────────────────────────────────

async fn generate_feed(burrow_name: &str, burrow_dir: &path::Path, domain: &str) -> String {
    let desc = read_description(burrow_dir).await;
    let phlog_dir = burrow_dir.join("phlog");

    let base_url = if domain == "localhost" {
        format!("http://localhost:7070/{}", burrow_name)
    } else {
        format!("https://{}/{}", domain, burrow_name)
    };

    let mut items = Vec::new();

    if fs::try_exists(&phlog_dir).await.unwrap_or(false) {
        if let Ok(mut entries) = fs::read_dir(&phlog_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') || name.starts_with('_') || !name.ends_with(".txt") {
                    continue;
                }
                let content = fs::read_to_string(entry.path()).await.unwrap_or_default();
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

async fn generate_atom_feed(burrow_name: &str, burrow_dir: &path::Path, domain: &str) -> String {
    let desc = read_description(burrow_dir).await;
    let phlog_dir = burrow_dir.join("phlog");

    let base_url = if domain == "localhost" {
        format!("http://localhost:7070/{}", burrow_name)
    } else {
        format!("https://{}/{}", domain, burrow_name)
    };

    let mut items = Vec::new();

    if fs::try_exists(&phlog_dir).await.unwrap_or(false) {
        if let Ok(mut entries) = fs::read_dir(&phlog_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') || name.starts_with('_') || !name.ends_with(".txt") {
                    continue;
                }
                let content = fs::read_to_string(entry.path()).await.unwrap_or_default();
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

// ── Rings ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ServerEntry {
    pub url: String,
    pub description: String,
}

async fn load_servers() -> Vec<ServerEntry> {
    let mut servers = Vec::new();
    if let Ok(content) = fs::read_to_string("servers.conf").await {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            // Format: gph://server.com   Description here
            let parts: Vec<&str> = line.splitn(2, "   ").collect();
            let url = parts[0].trim().to_string();
            let desc = parts.get(1).map(|s| s.trim().to_string()).unwrap_or_default();
            if !url.is_empty() {
                servers.push(ServerEntry { url, description: desc });
            }
        }
    }
    servers
}

#[derive(Clone)]
pub struct Ring {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub owner: String,
    pub members: Vec<String>,
}

fn parse_ring_file(content: &str, slug: &str, owner: &str) -> Ring {
    let mut title = String::new();
    let mut description = String::new();
    let mut members = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(val) = line.strip_prefix("title = ") {
            title = val.to_string();
        } else if let Some(val) = line.strip_prefix("description = ") {
            description = val.to_string();
        } else if line.starts_with('/') || line.starts_with("gph://") || line.starts_with("ring:") {
            members.push(line.to_string());
        }
    }

    if title.is_empty() {
        title = slug.to_string();
    }

    Ring {
        slug: slug.to_string(),
        title,
        description,
        owner: owner.to_string(),
        members,
    }
}

async fn load_all_rings() -> Vec<Ring> {
    let mut rings = Vec::new();
    if let Ok(mut dirs) = fs::read_dir("burrows").await {
        while let Ok(Some(burrow_entry)) = dirs.next_entry().await {
            let burrow_name = burrow_entry.file_name().to_string_lossy().to_string();
            if !burrow_name.starts_with('~') || !burrow_entry.path().is_dir() {
                continue;
            }
            let rings_dir = burrow_entry.path().join("rings");
            if let Ok(mut ring_files) = fs::read_dir(&rings_dir).await {
                while let Ok(Some(ring_entry)) = ring_files.next_entry().await {
                    let name = ring_entry.file_name().to_string_lossy().to_string();
                    if !name.ends_with(".ring") {
                        continue;
                    }
                    let slug = name.trim_end_matches(".ring");
                    if let Ok(content) = fs::read_to_string(ring_entry.path()).await {
                        rings.push(parse_ring_file(&content, slug, &burrow_name));
                    }
                }
            }
        }
    }
    // Resolve nested ring references before sorting
    resolve_nested_rings(&mut rings);

    rings.sort_by(|a, b| a.title.cmp(&b.title));
    rings
}

/// Resolve nested ring references. A member like `ring:~bruno/deep-web-craft`
/// is expanded to the members of that ring (without duplicates).
fn resolve_nested_rings(rings: &mut Vec<Ring>) {
    // Build a lookup: "~owner/slug" → member list
    let lookup: HashMap<String, Vec<String>> = rings.iter()
        .map(|r| (format!("{}/{}", r.owner, r.slug), r.members.clone()))
        .collect();

    for ring in rings.iter_mut() {
        let mut resolved = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for member in &ring.members {
            if let Some(ref_key) = member.strip_prefix("ring:") {
                // Expand nested ring
                if let Some(nested_members) = lookup.get(ref_key) {
                    for nm in nested_members {
                        if !nm.starts_with("ring:") && seen.insert(nm.clone()) {
                            resolved.push(nm.clone());
                        }
                    }
                }
            } else if seen.insert(member.clone()) {
                resolved.push(member.clone());
            }
        }

        ring.members = resolved;
    }
}

fn find_rings_for_burrow(rings: &[Ring], burrow_name: &str) -> Vec<Ring> {
    let local_path = format!("/{}", burrow_name);
    rings.iter()
        .filter(|r| r.members.iter().any(|m| m == &local_path))
        .cloned()
        .collect()
}

/// For a given ring and current burrow, find the previous and next members.
fn ring_neighbors(ring: &Ring, current_burrow: &str) -> (Option<String>, Option<String>) {
    let local_path = format!("/{}", current_burrow);
    let pos = ring.members.iter().position(|m| m == &local_path);
    match pos {
        Some(i) => {
            let prev = if i == 0 { ring.members.last() } else { ring.members.get(i - 1) };
            let next = if i == ring.members.len() - 1 { ring.members.first() } else { ring.members.get(i + 1) };
            (prev.cloned(), next.cloned())
        }
        None => (None, None),
    }
}

/// Convert a ring member path to an HTTP href.
/// Local: /~user → /~user
/// Remote: gph://host/~user → https://host/~user
fn ring_member_href(member: &str) -> String {
    if let Some(rest) = member.strip_prefix("gph://") {
        format!("https://{}", rest)
    } else {
        member.to_string()
    }
}

// ── Mentions ────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Mention {
    pub source_path: String,
    pub source_title: String,
    pub source_burrow: String,
}

/// Find all posts across all burrows that contain an internal link to the given path.
async fn find_mentions_of(target_path: &str) -> Vec<Mention> {
    let target_with_slash = format!("/{}", target_path);
    let mut mentions = Vec::new();

    if let Ok(mut dirs) = fs::read_dir("burrows").await {
        while let Ok(Some(burrow_entry)) = dirs.next_entry().await {
            let burrow_name = burrow_entry.file_name().to_string_lossy().to_string();
            if !burrow_name.starts_with('~') || !burrow_entry.path().is_dir() {
                continue;
            }
            // Scan phlog/ directory
            let phlog_dir = burrow_entry.path().join("phlog");
            if let Ok(mut posts) = fs::read_dir(&phlog_dir).await {
                while let Ok(Some(post_entry)) = posts.next_entry().await {
                    let name = post_entry.file_name().to_string_lossy().to_string();
                    if name.starts_with('.') || name.starts_with('_') || !name.ends_with(".txt") {
                        continue;
                    }
                    let slug = name.trim_end_matches(".txt");
                    let post_path = format!("{}/phlog/{}", burrow_name, slug);

                    // Don't mention yourself
                    if post_path == target_path {
                        continue;
                    }

                    if let Ok(content) = fs::read_to_string(post_entry.path()).await {
                        if content.contains(&target_with_slash) {
                            let title = content.lines().next().unwrap_or("")
                                .trim_start_matches("# ").to_string();
                            mentions.push(Mention {
                                source_path: format!("/{}", post_path),
                                source_title: title,
                                source_burrow: burrow_name.clone(),
                            });
                        }
                    }
                }
            }
        }
    }

    mentions.sort_by(|a, b| a.source_path.cmp(&b.source_path));
    mentions
}

// ── Federation Ping ─────────────────────────────────────────────

#[derive(Deserialize)]
struct PingForm {
    source: String,   // e.g. "https://other.server/~alice/phlog/2026-03-20-my-post"
    target: String,   // e.g. "/~bruno/phlog/2026-03-10-the-weight-of-a-webpage"
}

async fn receive_ping(Form(form): Form<PingForm>) -> impl IntoResponse {
    let source = form.source.trim().to_string();
    let target = form.target.trim().to_string();

    // Validate: target must be a local path
    if !target.starts_with("/~") {
        return (StatusCode::BAD_REQUEST, "Invalid target");
    }

    // Validate: source must be a URL
    if !source.starts_with("http://") && !source.starts_with("https://") && !source.starts_with("gph://") {
        return (StatusCode::BAD_REQUEST, "Invalid source");
    }

    // Rate limit: simple check — max 100 pings stored
    let pings_file = path::PathBuf::from("burrows/.pings");
    let existing = fs::read_to_string(&pings_file).await.unwrap_or_default();
    let ping_count = existing.lines().filter(|l| !l.is_empty()).count();
    if ping_count >= 100 {
        return (StatusCode::TOO_MANY_REQUESTS, "Ping log full");
    }

    // Append ping
    let date = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
    let entry = format!("{} {} → {}\n", date, source, target);

    let mut content = existing;
    content.push_str(&entry);
    let _ = fs::write(&pings_file, content).await;

    tracing::info!("Received ping: {} → {}", source, target);

    (StatusCode::OK, "OK")
}

/// Scan all posts for outgoing gph:// links and send federation pings.
/// Call this on startup or periodically.
async fn send_outgoing_pings(domain: &str) {
    let base_url = if domain == "localhost" {
        "http://localhost:7070".to_string()
    } else {
        format!("https://{}", domain)
    };

    // Track which pings we've already sent
    let sent_file = path::PathBuf::from("burrows/.pings-sent");
    let already_sent = fs::read_to_string(&sent_file).await.unwrap_or_default();
    let mut sent_set: std::collections::HashSet<String> = already_sent.lines()
        .map(|l| l.to_string())
        .collect();
    let mut new_pings = Vec::new();

    if let Ok(mut dirs) = fs::read_dir("burrows").await {
        while let Ok(Some(burrow_entry)) = dirs.next_entry().await {
            let burrow_name = burrow_entry.file_name().to_string_lossy().to_string();
            if !burrow_name.starts_with('~') || !burrow_entry.path().is_dir() {
                continue;
            }
            let phlog_dir = burrow_entry.path().join("phlog");
            if let Ok(mut posts) = fs::read_dir(&phlog_dir).await {
                while let Ok(Some(post_entry)) = posts.next_entry().await {
                    let name = post_entry.file_name().to_string_lossy().to_string();
                    if name.starts_with('.') || name.starts_with('_') || !name.ends_with(".txt") {
                        continue;
                    }
                    if let Ok(content) = fs::read_to_string(post_entry.path()).await {
                        let slug = name.trim_end_matches(".txt");
                        let source = format!("{}/{}/phlog/{}", base_url, burrow_name, slug);

                        // Find gph:// links in content
                        for line in content.lines() {
                            let target_url = if let Some(rest) = line.strip_prefix("→ ") {
                                let url = rest.trim();
                                if url.starts_with("gph://") { Some(url.to_string()) } else { None }
                            } else {
                                None
                            };

                            if let Some(target) = target_url {
                                let ping_key = format!("{} → {}", source, target);
                                if sent_set.contains(&ping_key) {
                                    continue;
                                }

                                // Convert gph:// to https:// for the ping POST
                                let target_server = target.strip_prefix("gph://")
                                    .and_then(|r| r.find('/').map(|pos| &r[..pos]));
                                let target_path = target.strip_prefix("gph://")
                                    .and_then(|r| r.find('/').map(|pos| &r[pos..]));

                                if let (Some(server), Some(path)) = (target_server, target_path) {
                                    let ping_url = format!("https://{}/ping", server);
                                    let body = format!("source={}&target={}", urlencod(&source), urlencod(path));

                                    // Fire-and-forget HTTP POST
                                    match reqwest_post_simple(&ping_url, &body).await {
                                        Ok(_) => {
                                            tracing::info!("Sent ping: {} → {}", source, target);
                                        }
                                        Err(e) => {
                                            tracing::debug!("Ping failed to {}: {}", ping_url, e);
                                        }
                                    }
                                }

                                sent_set.insert(ping_key.clone());
                                new_pings.push(ping_key);
                            }
                        }
                    }
                }
            }
        }
    }

    // Save sent pings
    if !new_pings.is_empty() {
        let mut content = already_sent;
        for ping in &new_pings {
            content.push_str(ping);
            content.push('\n');
        }
        let _ = fs::write(&sent_file, content).await;
        tracing::info!("Sent {} new federation pings", new_pings.len());
    }
}

fn urlencod(s: &str) -> String {
    s.replace('%', "%25")
        .replace(' ', "%20")
        .replace('&', "%26")
        .replace('=', "%3D")
        .replace('?', "%3F")
        .replace('#', "%23")
}

/// Simple HTTP POST using raw TCP — no reqwest dependency needed
async fn reqwest_post_simple(url: &str, body: &str) -> Result<(), String> {
    use tokio::net::TcpStream;

    let url_without_scheme = url.strip_prefix("https://").or_else(|| url.strip_prefix("http://"))
        .ok_or("Invalid URL")?;
    let (host, path) = match url_without_scheme.find('/') {
        Some(pos) => (&url_without_scheme[..pos], &url_without_scheme[pos..]),
        None => (url_without_scheme, "/"),
    };

    // For simplicity, we'll just attempt a plain HTTP POST
    // (In production, you'd use a proper HTTP client with TLS)
    let addr = format!("{}:80", host);
    let stream = TcpStream::connect(&addr).await.map_err(|e| e.to_string())?;
    let request = format!(
        "POST {} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        path, host, body.len(), body
    );

    let (_, mut writer) = tokio::io::split(stream);
    writer.write_all(request.as_bytes()).await.map_err(|e| e.to_string())?;
    Ok(())
}

/// Load received federation pings for a given target path
async fn load_received_pings(target_path: &str) -> Vec<Mention> {
    let pings_file = path::PathBuf::from("burrows/.pings");
    let content = fs::read_to_string(&pings_file).await.unwrap_or_default();
    let target_with_slash = format!("→ {}", target_path);

    content.lines()
        .filter(|l| l.contains(&target_with_slash))
        .filter_map(|l| {
            // Format: "2026-03-22 14:30 https://other.server/~alice/post → /~bruno/post"
            let parts: Vec<&str> = l.splitn(3, ' ').collect();
            if parts.len() >= 3 {
                let source_and_target: Vec<&str> = parts[2].splitn(2, " → ").collect();
                if let Some(source) = source_and_target.first() {
                    Some(Mention {
                        source_path: source.to_string(),
                        source_title: source.to_string(),
                        source_burrow: "remote".to_string(),
                    })
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}

// ── Gallery ─────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct GalleryPiece {
    pub filename: String,
    pub title: String,
    pub preview: String,
    pub url_path: String,
    pub line_count: usize,
    pub max_width: usize,
}

async fn load_gallery(dir: &path::Path) -> Vec<GalleryPiece> {
    let mut pieces = Vec::new();
    if let Ok(mut entries) = fs::read_dir(dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || name.starts_with('_') || (!name.ends_with(".txt") && !name.ends_with(".gph")) {
                continue;
            }
            let content = fs::read_to_string(entry.path()).await.unwrap_or_default();
            let title = content.lines().next().unwrap_or("")
                .trim_start_matches("# ")
                .to_string();

            // Preview: first 12 lines (skip title if it's a heading)
            let preview_lines: Vec<&str> = content.lines()
                .skip(if content.starts_with("# ") { 1 } else { 0 })
                .filter(|l| !l.is_empty() || true) // keep empty lines for art spacing
                .take(12)
                .collect();
            let preview = preview_lines.join("\n");

            let line_count = content.lines().count();
            let max_width = content.lines().map(|l| l.len()).max().unwrap_or(0);

            let slug = name.trim_end_matches(".txt").trim_end_matches(".gph").to_string();

            pieces.push(GalleryPiece {
                title: if title.is_empty() { slug.clone() } else { title },
                url_path: slug,
                filename: name,
                preview,
                line_count,
                max_width,
            });
        }
    }
    pieces.sort_by(|a, b| a.filename.cmp(&b.filename));
    pieces
}

fn is_gallery_item(path: &str) -> bool {
    let parts: Vec<&str> = path.split('/').collect();
    parts.len() >= 3 && parts.iter().any(|p| *p == "gallery")
}

// ── Bookmarks ───────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct BookmarkEntry {
    pub url: String,
    pub description: String,
    pub is_external: bool,
}

fn parse_bookmarks(content: &str) -> Vec<BookmarkEntry> {
    let mut bookmarks = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix("→ ") {
            // External: → URL   Description · date
            let parts: Vec<&str> = rest.splitn(2, "   ").collect();
            let url = parts[0].trim().to_string();
            let desc = parts.get(1).unwrap_or(&"").to_string();
            bookmarks.push(BookmarkEntry { url, description: desc, is_external: true });
        } else if line.starts_with('/') {
            // Internal: /path   Description · date
            let parts: Vec<&str> = line.splitn(2, "   ").collect();
            let url = parts[0].trim().to_string();
            let desc = parts.get(1).unwrap_or(&"").to_string();
            bookmarks.push(BookmarkEntry { url, description: desc, is_external: false });
        }
    }
    bookmarks
}

/// Scan all bookmarks.gph files across burrows and count how many times each
/// internal path is bookmarked. Returns the top 5 most-bookmarked paths.
async fn count_bookmark_mentions(burrows: &[BurrowEntry]) -> Vec<(String, String, usize)> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    let mut titles: HashMap<String, String> = HashMap::new();

    for burrow in burrows {
        let burrow_name = burrow.name.trim_end_matches('/');
        let bm_path = path::PathBuf::from("burrows").join(burrow_name).join("bookmarks.gph");
        if let Ok(content) = fs::read_to_string(&bm_path).await {
            let entries = parse_bookmarks(&content);
            for entry in &entries {
                if !entry.is_external && entry.url.starts_with('/') {
                    *counts.entry(entry.url.clone()).or_insert(0) += 1;
                    if !entry.description.is_empty() && !titles.contains_key(&entry.url) {
                        titles.insert(entry.url.clone(), entry.description.clone());
                    }
                }
            }
        }
    }

    let mut sorted: Vec<_> = counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));

    sorted.into_iter()
        .take(5)
        .map(|(url, count)| {
            let title = titles.get(&url).cloned().unwrap_or_default();
            (url, title, count)
        })
        .collect()
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
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(path): Path<String>,
    State(state): State<AppState>,
    Form(form): Form<GuestbookForm>,
) -> Response {
    let domain = state.config.read().unwrap().resolve_domain(extract_host(&headers).as_deref());

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

    let burrows_root = fs::canonicalize("burrows").await.unwrap_or_else(|_| path::PathBuf::from("burrows"));
    let fs_path = path::PathBuf::from("burrows").join(&path);

    // Must end with "guestbook" and resolve to guestbook.gph
    let gph_path = if fs_path.extension().is_none() {
        fs_path.with_extension("gph")
    } else {
        fs_path.clone()
    };

    let canonical = match fs::canonicalize(&gph_path).await {
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
    let existing = fs::read_to_string(&canonical).await.unwrap_or_default();
    let entry_count = existing.matches("\n--- ").count() + if existing.starts_with("--- ") { 1 } else { 0 };
    if entry_count >= MAX_GUESTBOOK_ENTRIES {
        return Redirect::to(&format!("/{}", path)).into_response();
    }

    // Append new entry
    let date = Local::now().format("%Y-%m-%d %H:%M").to_string();
    let entry = format!("\n--- {} · {}\n{}\n", name, date, message);

    let mut content = existing;
    content.push_str(&entry);
    let _ = fs::write(&canonical, content).await;

    Redirect::to(&format!("/{}", path)).into_response()
}

fn local_ip_address() -> Option<String> {
    use std::net::UdpSocket;
    // Connect to a public IP to determine the local interface address
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let addr = socket.local_addr().ok()?;
    Some(addr.ip().to_string())
}

// ── Gemini Protocol ─────────────────────────────────────────────

async fn gemini_listener(state: AppState, bind_addr: String, tls_cert: String, tls_key: String) {
    use tokio_rustls::TlsAcceptor;
    use std::io::BufReader;

    // Load TLS config
    let cert_data = std::fs::read(&tls_cert).unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m Gemini: failed to read cert {}: {}", tls_cert, e);
        std::process::exit(1);
    });
    let key_data = std::fs::read(&tls_key).unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m Gemini: failed to read key {}: {}", tls_key, e);
        std::process::exit(1);
    });

    let certs = rustls_pemfile::certs(&mut BufReader::new(cert_data.as_slice()))
        .filter_map(|c| c.ok())
        .collect::<Vec<_>>();
    let key = rustls_pemfile::private_key(&mut BufReader::new(key_data.as_slice()))
        .ok()
        .flatten()
        .unwrap_or_else(|| {
            eprintln!("  \x1b[31m✗\x1b[0m Gemini: no private key found in {}", tls_key);
            std::process::exit(1);
        });

    let tls_config = tokio_rustls::rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .unwrap_or_else(|e| {
            eprintln!("  \x1b[31m✗\x1b[0m Gemini: TLS config error: {}", e);
            std::process::exit(1);
        });

    let acceptor = TlsAcceptor::from(Arc::new(tls_config));
    let listener = tokio::net::TcpListener::bind(&bind_addr).await.unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m Gemini: failed to bind {}: {}", bind_addr, e);
        std::process::exit(1);
    });

    tracing::info!("Gemini listening on {}", bind_addr);

    loop {
        let (stream, _addr) = match listener.accept().await {
            Ok(s) => s,
            Err(_) => continue,
        };
        let acceptor = acceptor.clone();
        let state = state.clone();
        tokio::spawn(async move {
            if let Ok(mut tls_stream) = acceptor.accept(stream).await {
                handle_gemini_request(&mut tls_stream, &state).await;
            }
        });
    }
}

async fn handle_gemini_request<S: AsyncReadExt + AsyncWriteExt + Unpin>(stream: &mut S, state: &AppState) {
    // Read request line: "gemini://host/path\r\n" (max 1024 bytes per spec)
    let mut buf = [0u8; 1026];
    let n = match stream.read(&mut buf).await {
        Ok(n) if n > 0 => n,
        _ => return,
    };

    let request = String::from_utf8_lossy(&buf[..n]);
    let request = request.trim();

    // Parse URL
    let url_path = if let Some(rest) = request.strip_prefix("gemini://") {
        // Strip domain part: "host/path" → "/path"
        match rest.find('/') {
            Some(pos) => &rest[pos..],
            None => "/",
        }
    } else {
        // Malformed request
        let _ = stream.write_all(b"59 Bad request\r\n").await;
        return;
    };

    let url_path = url_path.trim_end_matches('\r').trim_end_matches('\n');
    let domain = &state.domain;

    // Route
    let response = match url_path {
        "/" => {
            let burrows = list_burrows().await;
            let body = render::home_gmi(&burrows, domain);
            format!("20 text/gemini; charset=utf-8\r\n{}", body)
        }
        "/discover" => {
            let burrows = list_burrows().await;
            let body = render::home_gmi(&burrows, domain);
            format!("20 text/gemini; charset=utf-8\r\n# Discover\n\n{}", body)
        }
        _ => {
            gemini_serve_path(url_path, domain).await
        }
    };

    let _ = stream.write_all(response.as_bytes()).await;
}

async fn gemini_serve_path(url_path: &str, domain: &str) -> String {
    let path = url_path.trim_start_matches('/');

    if path.is_empty() {
        let burrows = list_burrows().await;
        let body = render::home_gmi(&burrows, domain);
        return format!("20 text/gemini; charset=utf-8\r\n{}", body);
    }

    // Draft visibility
    if path.split('/').any(|seg| seg.starts_with('_') || seg.starts_with('.')) {
        return format!("51 Not found\r\n");
    }

    // Depth limit
    if path.split('/').filter(|s| !s.is_empty()).count() > MAX_PATH_DEPTH {
        return format!("51 Not found\r\n");
    }

    let burrows_root = fs::canonicalize("burrows").await.unwrap_or_else(|_| path::PathBuf::from("burrows"));
    let fs_path = path::PathBuf::from("burrows").join(path);

    // Try exact path, then .txt, then .gph
    let canonical = if let Ok(p) = fs::canonicalize(&fs_path).await {
        p
    } else if let Ok(p) = fs::canonicalize(fs_path.with_extension("txt")).await {
        p
    } else if let Ok(p) = fs::canonicalize(fs_path.with_extension("gph")).await {
        p
    } else {
        return format!("51 Not found\r\n");
    };

    // Path traversal check
    if !canonical.starts_with(&burrows_root) {
        return format!("51 Not found\r\n");
    }

    if canonical.is_dir() {
        let entries = list_directory(&canonical, &burrows_root).await;
        let body = render::directory_listing_gmi(path, &entries);
        format!("20 text/gemini; charset=utf-8\r\n{}", body)
    } else {
        let content = read_file_checked(&canonical).await;
        let body = render::render_gph_to_gmi(&content);
        format!("20 text/gemini; charset=utf-8\r\n{}", body)
    }
}

#[cfg(test)]
mod tests;
