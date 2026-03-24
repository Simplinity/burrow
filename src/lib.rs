pub mod config;
pub mod render;

// ── Shared types ────────────────────────────────────────────────
// These types are used by both the server (burrowd) and the client.

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

#[derive(Debug, Clone)]
pub struct GuestbookEntry {
    pub name: String,
    pub date: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct GalleryPiece {
    pub filename: String,
    pub title: String,
    pub preview: String,
    pub url_path: String,
    pub line_count: usize,
    pub max_width: usize,
}

#[derive(Debug, Clone)]
pub struct BookmarkEntry {
    pub url: String,
    pub description: String,
    pub is_external: bool,
}

#[derive(Debug, Clone)]
pub struct Mention {
    pub source_path: String,
    pub source_title: String,
    pub source_burrow: String,
}

#[derive(Clone)]
pub struct Ring {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub owner: String,
    pub members: Vec<String>,
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

#[derive(Debug, Clone)]
pub struct SeriesInfo {
    pub current: usize,
    pub total: usize,
    pub prev_path: Option<String>,
    pub next_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ServerEntry {
    pub url: String,
    pub description: String,
}

// ── Ring helpers ─────────────────────────────────────────────────

pub fn ring_neighbors(ring: &Ring, current_burrow: &str) -> (Option<String>, Option<String>) {
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

pub fn ring_member_href(member: &str) -> String {
    if let Some(rest) = member.strip_prefix("gph://") {
        format!("https://{}", rest)
    } else {
        member.to_string()
    }
}
