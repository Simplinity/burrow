use super::*;

const TEST_DOMAIN: &str = "test.burrow.local";

// ── render_gph tests ────────────────────────────────────────────

#[test]
fn render_gph_heading() {
    let html = render::render_gph("# Hello World");
    assert_eq!(html, "<h1>Hello World</h1>");
}

#[test]
fn render_gph_paragraph() {
    let html = render::render_gph("Just a plain line.");
    assert_eq!(html, "<p>Just a plain line.</p>");
}

#[test]
fn render_gph_blockquote() {
    let html = render::render_gph("> Quoted text here");
    assert_eq!(html, "<blockquote><p>Quoted text here</p></blockquote>");
}

#[test]
fn render_gph_horizontal_rule() {
    let html = render::render_gph("---");
    assert_eq!(html, "<hr>");
}

#[test]
fn render_gph_external_link() {
    let html = render::render_gph("→ https://example.com");
    assert!(html.contains(r#"href="https://example.com""#));
    assert!(html.contains("example.com"));
}

#[test]
fn render_gph_internal_link() {
    let html = render::render_gph("/~bruno/about   My about page");
    assert!(html.contains(r#"href="/~bruno/about""#));
    assert!(html.contains("My about page"));
}

#[test]
fn render_gph_code_block() {
    let html = render::render_gph("  code line 1\n  code line 2\nnot code");
    assert!(html.contains("<pre>"));
    assert!(html.contains("</pre>"));
    assert!(html.contains("code line 1"));
    assert!(html.contains("<p>not code</p>"));
}

#[test]
fn render_gph_code_block_closed_at_end() {
    let html = render::render_gph("  only code");
    assert!(html.starts_with("<pre>"));
    assert!(html.ends_with("</pre>"));
}

#[test]
fn render_gph_empty_lines_skipped() {
    let html = render::render_gph("line one\n\nline two");
    assert_eq!(html, "<p>line one</p><p>line two</p>");
}

#[test]
fn render_gph_expands_at_today() {
    let html = render::render_gph("Last updated: @today");
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    assert!(html.contains(&today));
    assert!(!html.contains("@today"));
}

#[test]
fn extract_inspired_by_detects_first_line() {
    let content = "← /~maya/phlog/on-digital-minimalism\n\n# My Response\n\nSome text.";
    let (inspired, remaining) = render::extract_inspired_by(content);
    assert!(inspired.is_some());
    let (path, author) = inspired.unwrap();
    assert_eq!(path, "/~maya/phlog/on-digital-minimalism");
    assert_eq!(author, "~maya");
    assert!(!remaining.contains("← /~maya"));
    assert!(remaining.contains("# My Response"));
}

#[test]
fn extract_inspired_by_ignores_non_matching() {
    let content = "# Normal Post\n\nNo inspiration line here.";
    let (inspired, remaining) = render::extract_inspired_by(content);
    assert!(inspired.is_none());
    assert_eq!(remaining, content);
}

#[test]
fn extract_inspired_by_skips_empty_lines() {
    let content = "\n\n← /~bruno/about\n\n# Post";
    let (inspired, _) = render::extract_inspired_by(content);
    assert!(inspired.is_some());
    assert_eq!(inspired.unwrap().1, "~bruno");
}

#[test]
fn extract_guest_author_works() {
    assert_eq!(render::extract_guest_author("guest-~maya-on-simplicity.txt"), Some("~maya".to_string()));
    assert_eq!(render::extract_guest_author("guest-~bob-thoughts.gph"), Some("~bob".to_string()));
    assert_eq!(render::extract_guest_author("about.txt"), None);
    assert_eq!(render::extract_guest_author("guest-no-tilde.txt"), None);
}

#[test]
fn extract_series_number_works() {
    assert_eq!(extract_series_number("part-01"), Some(("part-", 1)));
    assert_eq!(extract_series_number("part-12"), Some(("part-", 12)));
    assert_eq!(extract_series_number("chapter_3"), Some(("chapter_", 3)));
    assert_eq!(extract_series_number("essay-on-rust"), None); // "rust" is not a number
    assert_eq!(extract_series_number("no-number-here"), None);
    assert_eq!(extract_series_number("plain"), None);
}

#[test]
fn render_gph_mixed_content() {
    let content = "# Title\n\nSome text.\n\n> A quote\n\n---\n\nMore text.";
    let html = render::render_gph(content);
    assert!(html.contains("<h1>Title</h1>"));
    assert!(html.contains("<p>Some text.</p>"));
    assert!(html.contains("<blockquote>"));
    assert!(html.contains("<hr>"));
    assert!(html.contains("<p>More text.</p>"));
}

// ── HTML escaping tests ─────────────────────────────────────────

#[test]
fn render_gph_escapes_html_in_paragraphs() {
    let html = render::render_gph("<script>alert('xss')</script>");
    assert!(!html.contains("<script>"));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn render_gph_escapes_html_in_headings() {
    let html = render::render_gph("# <b>bold</b>");
    assert!(!html.contains("<b>"));
    assert!(html.contains("&lt;b&gt;"));
}

#[test]
fn render_gph_escapes_html_in_blockquotes() {
    let html = render::render_gph("> <img src=x onerror=alert(1)>");
    assert!(!html.contains("<img"));
    assert!(html.contains("&lt;img"));
}

#[test]
fn render_gph_escapes_html_in_code_blocks() {
    let html = render::render_gph("  <div>hello</div>");
    assert!(!html.contains("<div>"));
    assert!(html.contains("&lt;div&gt;"));
}

#[test]
fn render_gph_escapes_urls_in_href() {
    let html = render::render_gph("→ https://example.com/a\"onmouseover=\"alert(1)");
    assert!(!html.contains(r#"" onmouseover"#));
    assert!(html.contains("&quot;"));
}

#[test]
fn render_gph_escapes_internal_link_href() {
    let html = render::render_gph("/~user/\"onclick=\"alert(1)   description");
    assert!(html.contains("href=\"/~user/&quot;onclick=&quot;alert(1)\""));
    assert!(html.contains("description"));
}

// ── render_entries escaping ─────────────────────────────────────

#[test]
fn home_page_escapes_entry_names() {
    let entries = vec![BurrowEntry {
        name: "<script>alert(1)</script>/".to_string(),
        entry_type: EntryType::Directory,
        description: "safe desc".to_string(),
        meta: "1 items".to_string(),
        path: "/~test".to_string(),
    }];
    let html = render::home_page(&entries, TEST_DOMAIN);
    assert!(!html.contains("<script>alert(1)</script>"));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn home_page_escapes_descriptions() {
    let entries = vec![BurrowEntry {
        name: "~test/".to_string(),
        entry_type: EntryType::Directory,
        description: "<img onerror=alert(1)>".to_string(),
        meta: "1 items".to_string(),
        path: "/~test".to_string(),
    }];
    let html = render::home_page(&entries, TEST_DOMAIN);
    assert!(!html.contains("<img onerror"));
    assert!(html.contains("&lt;img"));
}

// ── Page structure tests ────────────────────────────────────────

#[test]
fn home_page_contains_burrow_count() {
    let entries = vec![
        BurrowEntry {
            name: "~alice/".to_string(),
            entry_type: EntryType::Directory,
            description: "Alice's burrow".to_string(),
            meta: "3 items".to_string(),
            path: "/~alice".to_string(),
        },
        BurrowEntry {
            name: "~bob/".to_string(),
            entry_type: EntryType::Directory,
            description: "Bob's burrow".to_string(),
            meta: "2 items".to_string(),
            path: "/~bob".to_string(),
        },
    ];
    let html = render::home_page(&entries, TEST_DOMAIN);
    assert!(html.contains("2 burrows"));
}

#[test]
fn directory_page_shows_crumbs() {
    let entries = vec![];
    let burrows = vec![];
    let html = render::directory_page("~bruno/phlog", None, &entries, &burrows, TEST_DOMAIN, None);
    assert!(html.contains(TEST_DOMAIN));
    assert!(html.contains("~bruno"));
    assert!(html.contains("phlog"));
}

#[test]
fn text_page_shows_reading_time() {
    let words: Vec<&str> = std::iter::repeat("word").take(230).collect();
    let content = words.join(" ");
    let html = render::text_page("~bruno/test.txt", "test.txt", &content, TEST_DOMAIN, None);
    assert!(html.contains("~1 min read"));
}

#[test]
fn text_page_has_progress_bar() {
    let html = render::text_page("~bruno/test.txt", "test.txt", "Hello", TEST_DOMAIN, None);
    assert!(html.contains("progress"));
    assert!(html.contains("animation-timeline:scroll()"));
    assert!(!html.contains("<script>"));
}

#[test]
fn text_page_uses_shared_head() {
    let html = render::text_page("~bruno/test.txt", "test.txt", "Hello", TEST_DOMAIN, None);
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("burrow v0.1.0"));
    assert!(html.contains("gph://"));
}

#[test]
fn not_found_page_has_back_link() {
    let html = render::not_found_page("nonexistent", TEST_DOMAIN);
    assert!(html.contains(r#"href="/""#));
    assert!(html.contains("Back to the surface"));
}

#[test]
fn not_found_page_escapes_path() {
    let html = render::not_found_page("<script>alert(1)</script>", TEST_DOMAIN);
    assert!(!html.contains("<script>alert(1)</script>"));
    assert!(html.contains("&lt;script&gt;"));
}

// ── read_file_checked tests ─────────────────────────────────────

#[tokio::test]
async fn read_file_checked_reads_small_file() {
    let content = read_file_checked(std::path::Path::new("Cargo.toml")).await;
    assert!(content.contains("[package]"));
}

#[tokio::test]
async fn read_file_checked_returns_empty_for_missing() {
    let content = read_file_checked(std::path::Path::new("nonexistent-file.txt")).await;
    assert!(content.is_empty());
}

// ── read_description tests ──────────────────────────────────────

#[tokio::test]
async fn read_description_from_burrow_file() {
    let desc = read_description(std::path::Path::new("burrows/~bruno")).await;
    assert!(!desc.is_empty());
    assert!(desc.contains("typografie") || desc.contains("ITAD"));
}

#[tokio::test]
async fn read_description_missing_dir() {
    let desc = read_description(std::path::Path::new("burrows/nonexistent")).await;
    assert!(desc.is_empty());
}

// ── first_line_of tests ─────────────────────────────────────────

#[tokio::test]
async fn first_line_strips_heading_prefix() {
    let line = first_line_of(std::path::Path::new("burrows/~bruno/about.txt")).await;
    assert_eq!(line, "About");
}

#[tokio::test]
async fn first_line_missing_file() {
    let line = first_line_of(std::path::Path::new("nonexistent.txt")).await;
    assert!(line.is_empty());
}

// ── list_burrows tests ──────────────────────────────────────────

#[tokio::test]
async fn list_burrows_finds_tilde_dirs() {
    let burrows = list_burrows().await;
    assert!(burrows.len() >= 2); // ~bruno and ~maya
    assert!(burrows.iter().all(|b| b.name.starts_with('~')));
    assert!(burrows.iter().all(|b| b.entry_type == EntryType::Directory));
}

#[tokio::test]
async fn list_burrows_sorted_alphabetically() {
    let burrows = list_burrows().await;
    let names: Vec<&str> = burrows.iter().map(|b| b.name.as_str()).collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted);
}

// ── list_directory tests ────────────────────────────────────────

#[tokio::test]
async fn list_directory_skips_hidden_files() {
    let root = tokio::fs::canonicalize("burrows").await.unwrap();
    let dir = tokio::fs::canonicalize("burrows/~bruno").await.unwrap();
    let entries = list_directory(&dir, &root).await;
    assert!(entries.iter().all(|e| !e.name.starts_with('.')));
}

#[tokio::test]
async fn list_directory_sorts_dirs_first() {
    let root = tokio::fs::canonicalize("burrows").await.unwrap();
    let dir = tokio::fs::canonicalize("burrows/~bruno").await.unwrap();
    let entries = list_directory(&dir, &root).await;
    let dir_count = entries.iter().filter(|e| e.entry_type == EntryType::Directory).count();
    for (i, e) in entries.iter().enumerate() {
        if e.entry_type == EntryType::Directory {
            assert!(i < dir_count, "Directory {} found after files", e.name);
        }
    }
}

#[tokio::test]
async fn list_directory_includes_file_sizes() {
    let root = tokio::fs::canonicalize("burrows").await.unwrap();
    let dir = tokio::fs::canonicalize("burrows/~bruno").await.unwrap();
    let entries = list_directory(&dir, &root).await;
    let files: Vec<_> = entries.iter().filter(|e| e.entry_type == EntryType::Text).collect();
    assert!(files.iter().all(|f| f.meta.contains("B") || f.meta.contains("KB")));
}
