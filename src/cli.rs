use burrow::config::ServerConfig;
use chrono::Local;
use clap::{Parser, Subcommand};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The internet, minus the parts that made you hate the internet.
#[derive(Parser)]
#[command(name = "burrow", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new burrow
    Init {
        /// Your burrow name (e.g. "bruno" → creates ~bruno)
        name: String,
    },
    /// Create a new phlog post and open it in your editor
    New {
        /// Post title (e.g. "Why plaintext matters")
        title: String,
    },
    /// List your burrow contents
    Ls {
        /// Path within your burrow (default: root)
        path: Option<String>,
    },
    /// Switch active burrow
    Switch {
        /// Burrow name to switch to (e.g. "bruno" or "~bruno")
        name: Option<String>,
    },
    /// Show burrow status
    Status,
    /// Preview a draft file in the terminal
    Preview {
        /// Path to file (e.g. "_draft-post.txt" or "phlog/_wip.txt")
        path: String,
    },
    /// Open a file in your editor
    Edit {
        /// Path to file (e.g. "about.txt" or "phlog/my-post.txt")
        path: String,
    },
    /// Manage your webrings
    Ring {
        #[command(subcommand)]
        command: RingCommands,
    },
    /// Manage your bookmarks
    Bookmark {
        #[command(subcommand)]
        command: BookmarkCommands,
    },
    /// Manage your guestbook
    Guestbook {
        #[command(subcommand)]
        command: GuestbookCommands,
    },
    /// Search across all burrow content
    Search {
        /// Search query (case-insensitive substring match)
        query: String,
        /// Search all burrows, not just the active one
        #[arg(long, short)]
        all: bool,
    },
    /// Sync your burrow to/from a remote server
    Push {
        /// Remote destination (e.g. "user@host:/srv/burrow/burrows/")
        remote: String,
    },
    /// Pull your burrow from a remote server
    Pull {
        /// Remote source (e.g. "user@host:/srv/burrow/burrows/~bruno/")
        remote: String,
    },
    /// Generate a yearly time capsule summary
    Timecapsule {
        /// Year to generate for (default: current year)
        year: Option<i32>,
    },
    /// Generate a colophon.txt for your burrow (metadata, stats, rings)
    Colophon,
    /// Export your burrow as a tar.gz backup
    Export {
        /// Output file path (default: ~/burrow-export-YYYY-MM-DD.tar.gz)
        output: Option<String>,
    },
    /// Open a gph:// URL (protocol handler entry point)
    Open {
        /// A gph:// URL (e.g. "gph://example.com/~bruno/about")
        url: String,
    },
    /// Register gph:// protocol handler on this system
    Register,
    /// Server management
    Server {
        #[command(subcommand)]
        command: ServerCommands,
    },
}

#[derive(Subcommand)]
enum RingCommands {
    /// Create a new ring
    Create {
        /// Ring name (e.g. "deep-web-craft")
        name: String,
        /// Ring description
        #[arg(short, long)]
        desc: Option<String>,
    },
    /// List rings owned by the active burrow
    List,
    /// Show ring details and members
    Show {
        /// Ring name
        name: String,
    },
    /// Add a member to a ring
    Add {
        /// Ring name
        ring: String,
        /// Member path (e.g. "/~maya" or "gph://tilde.town/~river")
        member: String,
    },
    /// Remove a member from a ring
    Remove {
        /// Ring name
        ring: String,
        /// Member path
        member: String,
    },
}

#[derive(Subcommand)]
enum BookmarkCommands {
    /// Add a bookmark
    Add {
        /// URL or internal path (e.g. "https://example.com" or "/~maya/about")
        url: String,
        /// Description of the bookmark
        #[arg(short, long)]
        desc: Option<String>,
    },
    /// List all bookmarks
    List,
    /// Remove a bookmark by number
    Remove {
        /// Bookmark number (from `burrow bookmark list`)
        number: usize,
    },
}

#[derive(Subcommand)]
enum GuestbookCommands {
    /// Create a guestbook for your burrow
    Init,
    /// Show recent guestbook entries
    Show,
}

#[derive(Subcommand)]
enum ServerCommands {
    /// Initialize server configuration
    Init {
        /// Your server's domain name (e.g. "myblog.example.com")
        #[arg(long)]
        domain: String,
        /// Port to listen on (default: 7070)
        #[arg(long, default_value = "7070")]
        port: u16,
    },
}

fn main() {
    let cli = Cli::parse();
    let burrows_root = find_burrows_root();
    let cfg = load_server_url(&burrows_root);

    match cli.command {
        Commands::Init { name } => cmd_init(&burrows_root, &name, &cfg),
        Commands::New { title } => cmd_new(&burrows_root, &title, &cfg),
        Commands::Ls { path } => cmd_ls(&burrows_root, path.as_deref()),
        Commands::Switch { name } => cmd_switch(&burrows_root, name.as_deref()),
        Commands::Preview { path } => cmd_preview(&burrows_root, &path),
        Commands::Status => cmd_status(&burrows_root, &cfg),
        Commands::Search { query, all } => cmd_search(&burrows_root, &query, all),
        Commands::Push { remote } => cmd_push(&burrows_root, &remote),
        Commands::Pull { remote } => cmd_pull(&burrows_root, &remote),
        Commands::Colophon => cmd_colophon(&burrows_root),
        Commands::Timecapsule { year } => cmd_timecapsule(&burrows_root, year),
        Commands::Export { output } => cmd_export(&burrows_root, output.as_deref()),
        Commands::Ring { command } => match command {
            RingCommands::Create { name, desc } => cmd_ring_create(&burrows_root, &name, desc.as_deref()),
            RingCommands::List => cmd_ring_list(&burrows_root),
            RingCommands::Show { name } => cmd_ring_show(&burrows_root, &name),
            RingCommands::Add { ring, member } => cmd_ring_add(&burrows_root, &ring, &member),
            RingCommands::Remove { ring, member } => cmd_ring_remove(&burrows_root, &ring, &member),
        },
        Commands::Bookmark { command } => match command {
            BookmarkCommands::Add { url, desc } => cmd_bookmark_add(&burrows_root, &url, desc.as_deref()),
            BookmarkCommands::List => cmd_bookmark_list(&burrows_root),
            BookmarkCommands::Remove { number } => cmd_bookmark_remove(&burrows_root, number),
        },
        Commands::Guestbook { command } => match command {
            GuestbookCommands::Init => cmd_guestbook_init(&burrows_root),
            GuestbookCommands::Show => cmd_guestbook_show(&burrows_root),
        },
        Commands::Open { url } => cmd_open(&url),
        Commands::Register => cmd_register(),
        Commands::Server { command } => match command {
            ServerCommands::Init { domain, port } => cmd_server_init(&burrows_root, &domain, port),
        },
        Commands::Edit { path } => cmd_edit(&burrows_root, &path),
    }
}

fn load_server_url(burrows_root: &Path) -> String {
    let conf_path = burrows_root.parent().unwrap_or(Path::new(".")).join("burrow.conf");
    if conf_path.exists() {
        let cfg = ServerConfig::load_from(&conf_path);
        if cfg.domain == "localhost" {
            format!("http://localhost:{}", cfg.port)
        } else {
            format!("https://{}", cfg.domain)
        }
    } else {
        "http://localhost:7070".to_string()
    }
}

// ── Find burrow root ────────────────────────────────────────────

fn find_burrows_root() -> PathBuf {
    // Walk up from cwd looking for a burrows/ directory
    let mut dir = env::current_dir().unwrap();
    loop {
        let candidate = dir.join("burrows");
        if candidate.is_dir() {
            return candidate;
        }
        if !dir.pop() {
            break;
        }
    }
    // Default: ./burrows in cwd
    PathBuf::from("burrows")
}

fn find_active_burrow(burrows_root: &Path) -> Option<String> {
    // If there's only one burrow, use it
    let burrows: Vec<String> = fs::read_dir(burrows_root)
        .ok()?
        .flatten()
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.starts_with('~') && e.path().is_dir()
        })
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    match burrows.len() {
        1 => Some(burrows[0].clone()),
        _ => {
            // Check .burrow-active file
            let active_file = burrows_root.join(".burrow-active");
            fs::read_to_string(active_file).ok().map(|s| s.trim().to_string())
        }
    }
}

fn require_active_burrow(burrows_root: &Path) -> String {
    match find_active_burrow(burrows_root) {
        Some(name) => name,
        None => {
            eprintln!("  No active burrow found.");
            eprintln!("  Run `burrow init <name>` to create one.");
            std::process::exit(1);
        }
    }
}

fn burrow_path(burrows_root: &Path, name: &str) -> PathBuf {
    burrows_root.join(name)
}

// ── Commands ────────────────────────────────────────────────────

fn cmd_init(burrows_root: &Path, name: &str, server_url: &str) {
    let name = if name.starts_with('~') {
        name.to_string()
    } else {
        format!("~{}", name)
    };

    let root = burrow_path(burrows_root, &name);

    if root.exists() {
        eprintln!("  Burrow {} already exists.", name);
        std::process::exit(1);
    }

    // Create directory structure
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(root.join("phlog")).unwrap();

    // .burrow config
    fs::write(
        root.join(".burrow"),
        "description = A fresh burrow\n",
    )
    .unwrap();

    // about.txt
    fs::write(
        root.join("about.txt"),
        format!("# About\n\nThis is {}'s burrow. It's empty, but it's honest.\n", name),
    )
    .unwrap();

    // Set as active burrow
    fs::write(burrows_root.join(".burrow-active"), &name).unwrap();

    println!();
    println!("  \x1b[1m/\x1b[0m Burrow created.");
    println!();
    println!("  \x1b[36m{}\x1b[0m", name);
    println!("    about.txt");
    println!("    phlog/");
    println!();
    println!("  Your burrow is live at \x1b[36m{}/{}\x1b[0m", server_url, name);
    println!();
    println!("  Write your first post:");
    println!("    \x1b[1mburrow new \"My first post\"\x1b[0m");
    println!();
}

fn cmd_switch(burrows_root: &Path, name: Option<&str>) {
    let burrows: Vec<String> = fs::read_dir(burrows_root)
        .unwrap()
        .flatten()
        .filter(|e| {
            let n = e.file_name().to_string_lossy().to_string();
            n.starts_with('~') && e.path().is_dir()
        })
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    if burrows.is_empty() {
        eprintln!("  No burrows found. Run `burrow init <name>` first.");
        std::process::exit(1);
    }

    let active = find_active_burrow(burrows_root);

    match name {
        Some(n) => {
            let target = if n.starts_with('~') { n.to_string() } else { format!("~{}", n) };
            if !burrows.contains(&target) {
                eprintln!("  Burrow {} not found.", target);
                eprintln!("  Available: {}", burrows.join(", "));
                std::process::exit(1);
            }
            fs::write(burrows_root.join(".burrow-active"), &target).unwrap();
            println!();
            println!("  Switched to \x1b[36m{}\x1b[0m", target);
            println!();
        }
        None => {
            // List all burrows with active marker
            println!();
            println!("  \x1b[1m/\x1b[0m Burrows");
            println!();
            let mut sorted = burrows.clone();
            sorted.sort();
            for b in &sorted {
                let marker = if active.as_deref() == Some(b.as_str()) { " \x1b[32m◀\x1b[0m" } else { "" };
                println!("    \x1b[36m{}\x1b[0m{}", b, marker);
            }
            println!();
            println!("  Switch with: \x1b[1mburrow switch <name>\x1b[0m");
            println!();
        }
    }
}

fn cmd_preview(burrows_root: &Path, path: &str) {
    let name = require_active_burrow(burrows_root);
    let root = burrow_path(burrows_root, &name);
    let filepath = root.join(path);

    // Try with .txt extension if not found
    let filepath = if filepath.exists() {
        filepath
    } else if filepath.with_extension("txt").exists() {
        filepath.with_extension("txt")
    } else {
        eprintln!("  File not found: {}", path);
        std::process::exit(1);
    };

    let content = fs::read_to_string(&filepath).unwrap_or_default();
    let filename = filepath.file_name().unwrap().to_string_lossy();
    let words = content.split_whitespace().count();
    let read_min = (words as f64 / 230.0).ceil() as usize;

    println!();
    println!("  \x1b[1m{}\x1b[0m", filename);
    println!("  \x1b[90m~{} min read · {} words\x1b[0m", read_min, words);
    println!("  \x1b[90m{}\x1b[0m", "─".repeat(50));
    println!();

    for line in content.lines() {
        if let Some(heading) = line.strip_prefix("# ") {
            println!("  \x1b[1;36m{}\x1b[0m", heading);
        } else if let Some(quote) = line.strip_prefix("> ") {
            println!("  \x1b[90m│ {}\x1b[0m", quote);
        } else if line == "---" {
            println!("  \x1b[90m{}\x1b[0m", "─".repeat(50));
        } else if let Some(rest) = line.strip_prefix("→ ") {
            println!("  \x1b[36m→ {}\x1b[0m", rest);
        } else if line.starts_with("  ") {
            println!("  \x1b[33m{}\x1b[0m", line);
        } else if line.is_empty() {
            println!();
        } else {
            println!("  {}", line);
        }
    }
    println!();
    println!("  \x1b[90m{}\x1b[0m", "─".repeat(50));
    if filename.starts_with('_') {
        println!("  \x1b[90mDraft — not visible on the server\x1b[0m");
    }
    println!();
}

fn cmd_new(burrows_root: &Path, title: &str, server_url: &str) {
    let name = require_active_burrow(burrows_root);
    let root = burrow_path(burrows_root, &name);
    let phlog_dir = root.join("phlog");

    if !phlog_dir.exists() {
        fs::create_dir_all(&phlog_dir).unwrap();
    }

    // Generate filename: YYYY-MM-DD-slugified-title.txt
    let date = Local::now().format("%Y-%m-%d").to_string();
    let slug = slugify(title);
    let filename = format!("{}-{}.txt", date, slug);
    let filepath = phlog_dir.join(&filename);

    if filepath.exists() {
        eprintln!("  File already exists: phlog/{}", filename);
        std::process::exit(1);
    }

    // Create file with title header
    fs::write(&filepath, format!("# {}\n\n", title)).unwrap();

    println!();
    println!("  \x1b[1mCreated:\x1b[0m phlog/{}", filename);

    // Open in editor
    let editor = env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
    println!("  Opening in {}...", editor);
    println!();

    let status = Command::new(&editor)
        .arg(&filepath)
        .status();

    match status {
        Ok(s) if s.success() => {
            let size = fs::metadata(&filepath).map(|m| m.len()).unwrap_or(0);
            if size <= 3 {
                // File is empty or just the header — user probably quit without writing
                fs::remove_file(&filepath).unwrap_or(());
                println!("  Empty post discarded.");
            } else {
                println!(
                    "  \x1b[32mPublished!\x1b[0m View at \x1b[36m{}/{}/phlog/{}\x1b[0m",
                    server_url,
                    name,
                    filename.trim_end_matches(".txt")
                );
            }
        }
        _ => {
            eprintln!("  Could not open editor '{}'. Set $EDITOR to your preferred editor.", editor);
        }
    }
    println!();
}

fn cmd_ls(burrows_root: &Path, path: Option<&str>) {
    let name = require_active_burrow(burrows_root);
    let root = burrow_path(burrows_root, &name);

    let dir = match path {
        Some(p) => root.join(p),
        None => root.clone(),
    };

    if !dir.is_dir() {
        eprintln!("  Not a directory: {}", dir.display());
        std::process::exit(1);
    }

    let rel_path = dir.strip_prefix(burrows_root).unwrap_or(&dir);
    println!();
    println!("  \x1b[1m/{}/\x1b[0m", rel_path.display());
    println!();

    let mut entries: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .flatten()
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            !name.starts_with('.') && !name.starts_with('_')
        })
        .collect();

    entries.sort_by(|a, b| {
        let a_dir = a.path().is_dir();
        let b_dir = b.path().is_dir();
        b_dir.cmp(&a_dir).then(a.file_name().cmp(&b.file_name()))
    });

    for entry in &entries {
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();

        if path.is_dir() {
            let count = fs::read_dir(&path).map(|d| d.count()).unwrap_or(0);
            println!(
                "    \x1b[36m/\x1b[0m  {:<30} \x1b[90m{} items\x1b[0m",
                format!("{}/", name),
                count
            );
        } else {
            let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            let size_str = if size < 1024 {
                format!("{} B", size)
            } else {
                format!("{:.1} KB", size as f64 / 1024.0)
            };
            let desc = first_line_of(&path);
            let symbol = if name.ends_with(".gph") { "→" } else { "¶" };
            println!(
                "    \x1b[90m{}\x1b[0m  {:<30} \x1b[90m{}  ·  {}\x1b[0m",
                symbol, name, size_str, desc
            );
        }
    }

    if entries.is_empty() {
        println!("    \x1b[90m(empty)\x1b[0m");
    }
    println!();
}

fn cmd_status(burrows_root: &Path, server_url: &str) {
    let name = require_active_burrow(burrows_root);
    let root = burrow_path(burrows_root, &name);

    let (file_count, total_size) = count_files_recursive(&root);
    let desc = read_description(&root);

    let size_str = if total_size < 1024 {
        format!("{} B", total_size)
    } else if total_size < 1_048_576 {
        format!("{:.1} KB", total_size as f64 / 1024.0)
    } else {
        format!("{:.2} MB", total_size as f64 / 1_048_576.0)
    };

    // Content limits from the concept doc
    let max_size: u64 = 1_048_576; // 1 MB free tier
    let pct = (total_size as f64 / max_size as f64 * 100.0).min(100.0);

    println!();
    println!("  \x1b[1m/\x1b[0m {}", name);
    if !desc.is_empty() {
        println!("  \x1b[90m{}\x1b[0m", desc);
    }
    println!();
    println!("  Files:    {}", file_count);
    println!("  Size:     {} / 1 MB \x1b[90m({:.0}%)\x1b[0m", size_str, pct);
    println!("  Server:   \x1b[36m{}/{}\x1b[0m", server_url, name);
    println!();

    // Show phlog post count if phlog exists
    let phlog_dir = root.join("phlog");
    if phlog_dir.is_dir() {
        let post_count = fs::read_dir(&phlog_dir)
            .map(|d| {
                d.flatten()
                    .filter(|e| {
                        let n = e.file_name().to_string_lossy().to_string();
                        !n.starts_with('.') && !n.starts_with('_')
                    })
                    .count()
            })
            .unwrap_or(0);
        println!("  Phlog:    {} posts", post_count);

        // Show latest post
        if let Some(latest) = latest_phlog_post(&phlog_dir) {
            println!("  Latest:   \x1b[90m{}\x1b[0m", latest);
        }
        println!();
    }
}

fn cmd_colophon(burrows_root: &Path) {
    let name = require_active_burrow(burrows_root);
    let root = burrow_path(burrows_root, &name);
    let desc = read_description(&root);

    let (file_count, total_size) = count_files_recursive(&root);
    let size_str = if total_size < 1024 {
        format!("{} B", total_size)
    } else if total_size < 1_048_576 {
        format!("{:.1} KB", total_size as f64 / 1024.0)
    } else {
        format!("{:.2} MB", total_size as f64 / 1_048_576.0)
    };

    // Count total words across all text files
    let total_words = count_words_recursive(&root);

    // Count phlog posts
    let phlog_dir = root.join("phlog");
    let post_count = if phlog_dir.is_dir() {
        fs::read_dir(&phlog_dir)
            .map(|d| d.flatten().filter(|e| {
                let n = e.file_name().to_string_lossy().to_string();
                !n.starts_with('.') && !n.starts_with('_') && (n.ends_with(".txt") || n.ends_with(".gph"))
            }).count())
            .unwrap_or(0)
    } else {
        0
    };

    // Find earliest and latest post dates
    let mut dates: Vec<String> = Vec::new();
    if phlog_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&phlog_dir) {
            for entry in entries.flatten() {
                let n = entry.file_name().to_string_lossy().to_string();
                if n.len() >= 10 && !n.starts_with('.') && !n.starts_with('_')
                    && n.as_bytes()[4] == b'-' && n.as_bytes()[7] == b'-'
                    && n[..4].chars().all(|c| c.is_ascii_digit())
                {
                    dates.push(n[..10].to_string());
                }
            }
        }
    }
    dates.sort();
    let first_post = dates.first().cloned().unwrap_or_else(|| "—".to_string());
    let latest_post = dates.last().cloned().unwrap_or_else(|| "—".to_string());

    // Find rings
    let rings_dir = root.join("rings");
    let mut ring_names: Vec<String> = Vec::new();
    if rings_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&rings_dir) {
            for entry in entries.flatten() {
                let n = entry.file_name().to_string_lossy().to_string();
                if n.ends_with(".ring") {
                    // Read title from ring file
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        for line in content.lines() {
                            if let Some(title) = line.strip_prefix("title = ") {
                                ring_names.push(title.trim().to_string());
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    ring_names.sort();

    // Check for guestbook
    let has_guestbook = root.join("guestbook.gph").exists();
    let guestbook_entries = if has_guestbook {
        fs::read_to_string(root.join("guestbook.gph"))
            .unwrap_or_default()
            .matches("\n--- ")
            .count()
            + if fs::read_to_string(root.join("guestbook.gph")).unwrap_or_default().starts_with("--- ") { 1 } else { 0 }
    } else {
        0
    };

    // Check for bookmarks
    let bookmark_count = if root.join("bookmarks.gph").exists() {
        fs::read_to_string(root.join("bookmarks.gph"))
            .unwrap_or_default()
            .lines()
            .filter(|l| l.starts_with("→ ") || l.starts_with("→ "))
            .count()
    } else {
        0
    };

    // Check for gallery
    let gallery_dir = root.join("gallery");
    let gallery_count = if gallery_dir.is_dir() {
        fs::read_dir(&gallery_dir)
            .map(|d| d.flatten().filter(|e| {
                let n = e.file_name().to_string_lossy().to_string();
                !n.starts_with('.') && !n.starts_with('_')
            }).count())
            .unwrap_or(0)
    } else {
        0
    };

    // Build the colophon
    let today = Local::now().format("%Y-%m-%d").to_string();
    let mut colophon = format!("# Colophon\n\nGenerated on {}.\n\n", today);

    colophon.push_str(&format!("This burrow belongs to {}.\n", name));
    if !desc.is_empty() {
        colophon.push_str(&format!("{}\n", desc));
    }
    colophon.push('\n');

    colophon.push_str("---\n\n");
    colophon.push_str(&format!("  Files:         {}\n", file_count));
    colophon.push_str(&format!("  Total size:    {}\n", size_str));
    colophon.push_str(&format!("  Total words:   {}\n", total_words));
    if post_count > 0 {
        colophon.push_str(&format!("  Phlog posts:   {}\n", post_count));
        colophon.push_str(&format!("  First post:    {}\n", first_post));
        colophon.push_str(&format!("  Latest post:   {}\n", latest_post));
    }
    if has_guestbook {
        colophon.push_str(&format!("  Guestbook:     {} entries\n", guestbook_entries));
    }
    if bookmark_count > 0 {
        colophon.push_str(&format!("  Bookmarks:     {}\n", bookmark_count));
    }
    if gallery_count > 0 {
        colophon.push_str(&format!("  Gallery:       {} pieces\n", gallery_count));
    }

    if !ring_names.is_empty() {
        colophon.push_str("\n---\n\n");
        colophon.push_str("Member of:\n\n");
        for ring in &ring_names {
            colophon.push_str(&format!("  {}\n", ring));
        }
    }

    colophon.push_str("\n---\n\n");
    colophon.push_str("Built with Burrow. Served as plaintext.\n");
    colophon.push_str("No JavaScript. No tracking. No algorithms.\n");
    colophon.push_str("Just words.\n");

    // Write to colophon.txt
    let colophon_path = root.join("colophon.txt");
    fs::write(&colophon_path, &colophon).unwrap();

    println!();
    println!("  \x1b[1m/\x1b[0m Colophon generated for {}", name);
    println!();
    println!("  {}", colophon_path.display());
    println!();
    println!("  {} files · {} words · {} posts",
        file_count, total_words, post_count);
    if !ring_names.is_empty() {
        println!("  Member of: {}", ring_names.join(", "));
    }
    println!();
    println!("  View at \x1b[36m/{}/colophon\x1b[0m", name);
    println!();
}

fn count_words_recursive(dir: &Path) -> usize {
    let mut total = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || name.starts_with('_') {
                continue;
            }
            let path = entry.path();
            if path.is_dir() {
                total += count_words_recursive(&path);
            } else if name.ends_with(".txt") || name.ends_with(".gph") {
                if let Ok(content) = fs::read_to_string(&path) {
                    total += content.split_whitespace().count();
                }
            }
        }
    }
    total
}

fn cmd_edit(burrows_root: &Path, path: &str) {
    let name = require_active_burrow(burrows_root);
    let root = burrow_path(burrows_root, &name);
    let filepath = root.join(path);

    // Try with .txt extension if not found
    let filepath = if filepath.exists() {
        filepath
    } else if filepath.with_extension("txt").exists() {
        filepath.with_extension("txt")
    } else {
        eprintln!("  File not found: {}", path);
        std::process::exit(1);
    };

    let editor = env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());

    let status = Command::new(&editor)
        .arg(&filepath)
        .status();

    match status {
        Ok(s) if s.success() => {
            println!();
            println!("  \x1b[32mSaved.\x1b[0m");
            println!();
        }
        _ => {
            eprintln!("  Could not open editor '{}'.", editor);
        }
    }
}

fn cmd_export(burrows_root: &Path, output: Option<&str>) {
    let name = require_active_burrow(burrows_root);
    let root = burrow_path(burrows_root, &name);

    if !root.is_dir() {
        eprintln!("  Burrow {} not found.", name);
        std::process::exit(1);
    }

    let date = Local::now().format("%Y-%m-%d").to_string();
    let default_name = format!("burrow-export-{}-{}.tar.gz", name.trim_start_matches('~'), date);
    let output_path = output.unwrap_or(&default_name);

    let status = Command::new("tar")
        .arg("czf")
        .arg(output_path)
        .arg("-C")
        .arg(burrows_root)
        .arg(&name)
        .status();

    match status {
        Ok(s) if s.success() => {
            let size = fs::metadata(output_path).map(|m| m.len()).unwrap_or(0);
            let size_str = if size < 1024 {
                format!("{} B", size)
            } else if size < 1_048_576 {
                format!("{:.1} KB", size as f64 / 1024.0)
            } else {
                format!("{:.2} MB", size as f64 / 1_048_576.0)
            };
            println!();
            println!("  \x1b[1m/\x1b[0m Exported \x1b[36m{}\x1b[0m", name);
            println!();
            println!("  File:  \x1b[36m{}\x1b[0m", output_path);
            println!("  Size:  {}", size_str);
            println!();
        }
        _ => {
            eprintln!("  Export failed. Make sure `tar` is installed.");
            std::process::exit(1);
        }
    }
}

fn cmd_server_init(burrows_root: &Path, domain: &str, port: u16) {
    // Validate domain
    let domain = domain.trim();
    if domain.is_empty() {
        eprintln!("  Domain cannot be empty.");
        std::process::exit(1);
    }
    if domain.contains("://") {
        eprintln!("  Domain should not include a protocol (e.g. use \"myblog.com\" not \"https://myblog.com\").");
        std::process::exit(1);
    }
    if domain.contains(' ') {
        eprintln!("  Domain cannot contain spaces.");
        std::process::exit(1);
    }

    // Create burrows directory if it doesn't exist
    if !burrows_root.exists() {
        fs::create_dir_all(burrows_root).unwrap();
    }

    // Write burrow.conf next to the burrows/ directory
    let conf_path = burrows_root.parent().unwrap_or(Path::new(".")).join("burrow.conf");

    let existed = conf_path.exists();

    let cfg = ServerConfig {
        domain: domain.to_string(),
        aliases: Vec::new(),
        port,
        tls_cert: None,
        tls_key: None,
        gemini_port: None,
    };
    cfg.save(&conf_path);

    println!();
    if existed {
        println!("  \x1b[1m/\x1b[0m Server reconfigured.");
    } else {
        println!("  \x1b[1m/\x1b[0m Server configured.");
    }
    println!();
    println!("  Domain:  \x1b[36m{}\x1b[0m", domain);
    println!("  Port:    \x1b[36m{}\x1b[0m", port);
    println!("  Config:  \x1b[90m{}\x1b[0m", conf_path.display());
    println!();
    println!("  Start the server:");
    println!("    \x1b[1mburrowd\x1b[0m");
    println!();
}

fn cmd_guestbook_init(burrows_root: &Path) {
    let name = require_active_burrow(burrows_root);
    let root = burrow_path(burrows_root, &name);
    let guestbook = root.join("guestbook.gph");

    if guestbook.exists() {
        eprintln!("  Guestbook already exists for {}.", name);
        std::process::exit(1);
    }

    fs::write(&guestbook, "").unwrap();

    println!();
    println!("  \x1b[1m/\x1b[0m Guestbook created for {}.", name);
    println!();
    println!("  Visitors can sign it at \x1b[36m/{}/guestbook\x1b[0m", name);
    println!();
}

fn cmd_guestbook_show(burrows_root: &Path) {
    let name = require_active_burrow(burrows_root);
    let root = burrow_path(burrows_root, &name);
    let guestbook = root.join("guestbook.gph");

    if !guestbook.exists() {
        eprintln!("  No guestbook found. Run `burrow guestbook init` first.");
        std::process::exit(1);
    }

    let content = fs::read_to_string(&guestbook).unwrap_or_default();
    if content.trim().is_empty() {
        println!();
        println!("  \x1b[90mNo entries yet. Share the link!\x1b[0m");
        println!();
        return;
    }

    println!();
    println!("  \x1b[1m/\x1b[0m Guestbook — {}", name);
    println!();

    let mut count = 0;
    let mut current_name = String::new();
    let mut current_date = String::new();
    let mut current_msg = String::new();

    let print_entry = |name: &str, date: &str, msg: &str, count: &mut usize| {
        if !name.is_empty() {
            *count += 1;
            println!("  \x1b[36m{}\x1b[0m  \x1b[90m{}\x1b[0m", name, date);
            println!("  {}", msg.trim());
            println!();
        }
    };

    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("--- ") {
            print_entry(&current_name, &current_date, &current_msg, &mut count);
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
    print_entry(&current_name, &current_date, &current_msg, &mut count);

    println!("  \x1b[90m{} entries total\x1b[0m", count);
    println!();
}

// ── Rings ───────────────────────────────────────────────────────

fn ring_path(burrows_root: &Path, burrow_name: &str, ring_name: &str) -> PathBuf {
    burrow_path(burrows_root, burrow_name).join("rings").join(format!("{}.ring", ring_name))
}

fn cmd_ring_create(burrows_root: &Path, name: &str, desc: Option<&str>) {
    let burrow_name = require_active_burrow(burrows_root);
    let root = burrow_path(burrows_root, &burrow_name);
    let rings_dir = root.join("rings");
    let slug = slugify(name);

    fs::create_dir_all(&rings_dir).unwrap();

    let path = rings_dir.join(format!("{}.ring", slug));
    if path.exists() {
        eprintln!("  Ring '{}' already exists.", slug);
        std::process::exit(1);
    }

    let description = desc.unwrap_or(name);
    let content = format!(
        "title = {}\ndescription = {}\n\n/{}\n",
        name, description, burrow_name
    );
    fs::write(&path, content).unwrap();

    println!();
    println!("  \x1b[1m/\x1b[0m Ring created: \x1b[36m{}\x1b[0m", name);
    println!("  Owner: {}", burrow_name);
    println!("  File:  \x1b[90m{}\x1b[0m", path.display());
    println!();
    println!("  Add members:");
    println!("    burrow ring add {} /~someone", slug);
    println!("    burrow ring add {} gph://other.server/~user", slug);
    println!();
}

fn cmd_ring_list(burrows_root: &Path) {
    let burrow_name = require_active_burrow(burrows_root);
    let root = burrow_path(burrows_root, &burrow_name);
    let rings_dir = root.join("rings");

    if !rings_dir.is_dir() {
        println!();
        println!("  \x1b[90mNo rings yet. Create one with `burrow ring create \"Ring Name\"`\x1b[0m");
        println!();
        return;
    }

    let mut rings = Vec::new();
    if let Ok(entries) = fs::read_dir(&rings_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".ring") {
                let slug = name.trim_end_matches(".ring").to_string();
                let content = fs::read_to_string(entry.path()).unwrap_or_default();
                let title = content.lines()
                    .find_map(|l| l.strip_prefix("title = "))
                    .unwrap_or(&slug)
                    .to_string();
                let member_count = content.lines()
                    .filter(|l| l.starts_with('/') || l.starts_with("gph://"))
                    .count();
                rings.push((slug, title, member_count));
            }
        }
    }

    if rings.is_empty() {
        println!();
        println!("  \x1b[90mNo rings yet. Create one with `burrow ring create \"Ring Name\"`\x1b[0m");
        println!();
        return;
    }

    rings.sort_by(|a, b| a.1.cmp(&b.1));

    println!();
    println!("  \x1b[1m/\x1b[0m Rings — {}", burrow_name);
    println!();
    for (slug, title, members) in &rings {
        println!("  \x1b[36m{}\x1b[0m  {}  \x1b[90m({} members)\x1b[0m", slug, title, members);
    }
    println!();
}

fn cmd_ring_show(burrows_root: &Path, name: &str) {
    let burrow_name = require_active_burrow(burrows_root);
    let path = ring_path(burrows_root, &burrow_name, name);

    if !path.exists() {
        eprintln!("  Ring '{}' not found. Use `burrow ring list` to see available rings.", name);
        std::process::exit(1);
    }

    let content = fs::read_to_string(&path).unwrap_or_default();
    let title = content.lines().find_map(|l| l.strip_prefix("title = ")).unwrap_or(name);
    let desc = content.lines().find_map(|l| l.strip_prefix("description = ")).unwrap_or("");
    let members: Vec<&str> = content.lines()
        .filter(|l| l.starts_with('/') || l.starts_with("gph://"))
        .collect();

    println!();
    println!("  \x1b[1m/\x1b[0m Ring: \x1b[36m{}\x1b[0m", title);
    if !desc.is_empty() {
        println!("  \x1b[90m{}\x1b[0m", desc);
    }
    println!();
    for (i, member) in members.iter().enumerate() {
        let marker = if i == 0 { "◎" } else { "○" };
        println!("  {} \x1b[36m{}\x1b[0m", marker, member);
    }
    println!();
    println!("  \x1b[90m{} members\x1b[0m", members.len());
    println!();
}

fn cmd_ring_add(burrows_root: &Path, ring_name: &str, member: &str) {
    let burrow_name = require_active_burrow(burrows_root);
    let path = ring_path(burrows_root, &burrow_name, ring_name);

    if !path.exists() {
        eprintln!("  Ring '{}' not found.", ring_name);
        std::process::exit(1);
    }

    let content = fs::read_to_string(&path).unwrap_or_default();
    // Check if already a member
    if content.lines().any(|l| l.trim() == member) {
        eprintln!("  {} is already a member of '{}'.", member, ring_name);
        std::process::exit(1);
    }

    let mut new_content = content;
    if !new_content.ends_with('\n') {
        new_content.push('\n');
    }
    new_content.push_str(member);
    new_content.push('\n');
    fs::write(&path, new_content).unwrap();

    println!();
    println!("  \x1b[1m/\x1b[0m Added \x1b[36m{}\x1b[0m to ring '{}'.", member, ring_name);
    println!();
}

fn cmd_ring_remove(burrows_root: &Path, ring_name: &str, member: &str) {
    let burrow_name = require_active_burrow(burrows_root);
    let path = ring_path(burrows_root, &burrow_name, ring_name);

    if !path.exists() {
        eprintln!("  Ring '{}' not found.", ring_name);
        std::process::exit(1);
    }

    let content = fs::read_to_string(&path).unwrap_or_default();
    let new_lines: Vec<&str> = content.lines()
        .filter(|l| l.trim() != member)
        .collect();

    if new_lines.len() == content.lines().count() {
        eprintln!("  {} is not a member of '{}'.", member, ring_name);
        std::process::exit(1);
    }

    let mut new_content = new_lines.join("\n");
    new_content.push('\n');
    fs::write(&path, new_content).unwrap();

    println!();
    println!("  \x1b[1m/\x1b[0m Removed \x1b[36m{}\x1b[0m from ring '{}'.", member, ring_name);
    println!();
}

// ── Protocol Handler ────────────────────────────────────────────

fn cmd_open(url: &str) {
    // Parse gph:// URL → extract domain and path
    let after_scheme = url
        .strip_prefix("gph://")
        .or_else(|| url.strip_prefix("gph:"))
        .unwrap_or(url);

    // Split into domain and path: "example.com/~bruno/about" → ("example.com", "/~bruno/about")
    let (domain, path) = match after_scheme.find('/') {
        Some(pos) => (&after_scheme[..pos], &after_scheme[pos..]),
        None => (after_scheme, "/"),
    };

    // If the path contains a ~user, try local preview first
    let user_path = path.trim_start_matches('/');
    if user_path.starts_with('~') {
        let burrows_root = find_burrows_root();
        let fs_path = burrows_root.join(user_path);

        // Try with .txt extension
        let with_txt = if fs_path.extension().is_none() {
            fs_path.with_extension("txt")
        } else {
            fs_path.clone()
        };

        if with_txt.exists() {
            // Local file exists — preview it
            println!();
            println!("  \x1b[1m/\x1b[0m Opening local: {}", user_path);
            println!();
            let content = fs::read_to_string(&with_txt).unwrap_or_default();
            print_gph_preview(&content);
            return;
        }
    }

    // Otherwise, open in browser as https://
    let https_url = format!("https://{}{}", domain, path);
    println!();
    println!("  \x1b[1m/\x1b[0m Opening \x1b[36m{}\x1b[0m", https_url);
    println!();

    let result = if cfg!(target_os = "macos") {
        Command::new("open").arg(&https_url).status()
    } else if cfg!(target_os = "linux") {
        Command::new("xdg-open").arg(&https_url).status()
    } else {
        eprintln!("  Unsupported platform. Open manually: {}", https_url);
        return;
    };

    if let Err(e) = result {
        eprintln!("  Failed to open browser: {}", e);
    }
}

fn print_gph_preview(content: &str) {
    for line in content.lines() {
        if let Some(heading) = line.strip_prefix("# ") {
            println!("  \x1b[1m{}\x1b[0m", heading);
        } else if let Some(quote) = line.strip_prefix("> ") {
            println!("  \x1b[90m│\x1b[0m \x1b[3m{}\x1b[0m", quote);
        } else if line == "---" {
            println!("  \x1b[90m────────────────────────────\x1b[0m");
        } else if line.starts_with("  ") {
            println!("  \x1b[36m{}\x1b[0m", line);
        } else if let Some(rest) = line.strip_prefix("→ ") {
            println!("  \x1b[36m→\x1b[0m {}", rest);
        } else if line.is_empty() {
            println!();
        } else {
            println!("  {}", line);
        }
    }
    println!();
}

fn cmd_register() {
    let burrow_bin = env::current_exe().unwrap_or_else(|_| PathBuf::from("burrow"));
    let burrow_path = burrow_bin.display().to_string();

    if cfg!(target_os = "macos") {
        register_macos(&burrow_path);
    } else if cfg!(target_os = "linux") {
        register_linux(&burrow_path);
    } else {
        eprintln!("  Protocol handler registration not supported on this platform.");
        std::process::exit(1);
    }
}

fn register_macos(burrow_path: &str) {
    if !cfg!(target_os = "macos") { return; }
    // Create a minimal .app wrapper in ~/Applications/
    let app_dir = PathBuf::from(env::var("HOME").unwrap_or_else(|_| ".".to_string()))
        .join("Applications/BurrowHandler.app/Contents");
    let macos_dir = app_dir.join("MacOS");

    fs::create_dir_all(&macos_dir).unwrap();

    // Info.plist with URL scheme registration
    let plist = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleIdentifier</key>
    <string>com.burrow.handler</string>
    <key>CFBundleName</key>
    <string>Burrow Handler</string>
    <key>CFBundleExecutable</key>
    <string>burrow-handler</string>
    <key>CFBundleURLTypes</key>
    <array>
        <dict>
            <key>CFBundleURLName</key>
            <string>Gopher Protocol Handler</string>
            <key>CFBundleURLSchemes</key>
            <array>
                <string>gph</string>
            </array>
        </dict>
    </array>
</dict>
</plist>"#);

    fs::write(app_dir.join("Info.plist"), plist).unwrap();

    // Shell script that forwards to burrow open
    let handler_script = format!("#!/bin/sh\nexec \"{}\" open \"$@\"\n", burrow_path);
    let handler_path = macos_dir.join("burrow-handler");
    fs::write(&handler_path, handler_script).unwrap();

    // Make executable
    Command::new("chmod").args(["+x", &handler_path.display().to_string()]).status().ok();

    // Register with Launch Services
    let app_path = app_dir.parent().unwrap().display().to_string();
    Command::new("/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister")
        .args(["-R", &app_path])
        .status()
        .ok();

    println!();
    println!("  \x1b[1m/\x1b[0m gph:// protocol handler registered.");
    println!();
    println!("  App:     \x1b[36m{}\x1b[0m", app_path);
    println!("  Handler: \x1b[36m{}\x1b[0m", burrow_path);
    println!();
    println!("  Try it:  \x1b[36mopen gph://localhost/~bruno/about\x1b[0m");
    println!();
}

fn register_linux(burrow_path: &str) {
    if !cfg!(target_os = "linux") { return; }
    // Create .desktop file for xdg-open
    let desktop_dir = PathBuf::from(env::var("HOME").unwrap_or_else(|_| ".".to_string()))
        .join(".local/share/applications");
    fs::create_dir_all(&desktop_dir).unwrap();

    let desktop_entry = format!(r#"[Desktop Entry]
Type=Application
Name=Burrow Handler
Exec={} open %u
Terminal=true
MimeType=x-scheme-handler/gph;
NoDisplay=true
"#, burrow_path);

    let desktop_path = desktop_dir.join("burrow-handler.desktop");
    fs::write(&desktop_path, desktop_entry).unwrap();

    // Register with xdg-mime
    Command::new("xdg-mime")
        .args(["default", "burrow-handler.desktop", "x-scheme-handler/gph"])
        .status()
        .ok();

    // Update desktop database
    Command::new("update-desktop-database")
        .arg(&desktop_dir.display().to_string())
        .status()
        .ok();

    println!();
    println!("  \x1b[1m/\x1b[0m gph:// protocol handler registered.");
    println!();
    println!("  Desktop: \x1b[36m{}\x1b[0m", desktop_path.display());
    println!("  Handler: \x1b[36m{}\x1b[0m", burrow_path);
    println!();
    println!("  Try it:  \x1b[36mxdg-open gph://localhost/~bruno/about\x1b[0m");
    println!();
}

// ── Search ──────────────────────────────────────────────────────

fn cmd_search(burrows_root: &Path, query: &str, all: bool) {
    let query_lower = query.to_lowercase();
    let mut total_matches = 0;

    let dirs: Vec<PathBuf> = if all {
        // Search all burrows
        fs::read_dir(burrows_root)
            .into_iter()
            .flat_map(|d| d.flatten())
            .filter(|e| e.file_name().to_string_lossy().starts_with('~') && e.path().is_dir())
            .map(|e| e.path())
            .collect()
    } else {
        let name = require_active_burrow(burrows_root);
        vec![burrow_path(burrows_root, &name)]
    };

    println!();
    println!("  \x1b[1m/\x1b[0m Searching for \x1b[36m{}\x1b[0m...", query);
    println!();

    for dir in &dirs {
        let burrow_name = dir.file_name().unwrap().to_string_lossy().to_string();
        search_dir(dir, &burrow_name, &query_lower, query, &mut total_matches);
    }

    if total_matches == 0 {
        println!("  \x1b[90mNo matches found.\x1b[0m");
    } else {
        println!("  \x1b[90m{} match{} found\x1b[0m", total_matches, if total_matches == 1 { "" } else { "es" });
    }
    println!();
}

fn search_dir(dir: &Path, burrow_name: &str, query_lower: &str, query_raw: &str, total: &mut usize) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || name.starts_with('_') {
                continue;
            }
            let path = entry.path();
            if path.is_dir() {
                search_dir(&path, burrow_name, query_lower, query_raw, total);
            } else if name.ends_with(".txt") || name.ends_with(".gph") {
                if let Ok(content) = fs::read_to_string(&path) {
                    let relative = path.strip_prefix(
                        path.ancestors().find(|a| a.file_name().map(|f| f.to_string_lossy().starts_with('~')).unwrap_or(false)).unwrap_or(&path)
                    ).unwrap_or(&path);

                    for (line_num, line) in content.lines().enumerate() {
                        if line.to_lowercase().contains(query_lower) {
                            *total += 1;
                            // Highlight match in context
                            let trimmed = line.trim();
                            let display = if trimmed.len() > 80 {
                                format!("{}...", &trimmed[..77])
                            } else {
                                trimmed.to_string()
                            };
                            // Color the matching part
                            let highlighted = highlight_match(&display, query_raw);
                            println!("  \x1b[36m{}\x1b[0m \x1b[90m{}:{}\x1b[0m", burrow_name, relative.display(), line_num + 1);
                            println!("    {}", highlighted);
                            println!();
                        }
                    }
                }
            }
        }
    }
}

fn highlight_match(text: &str, query: &str) -> String {
    let lower = text.to_lowercase();
    let query_lower = query.to_lowercase();
    if let Some(pos) = lower.find(&query_lower) {
        let before = &text[..pos];
        let matched = &text[pos..pos + query.len()];
        let after = &text[pos + query.len()..];
        format!("{}\x1b[1;33m{}\x1b[0m{}", before, matched, after)
    } else {
        text.to_string()
    }
}

// ── Time Capsule ────────────────────────────────────────────────

fn cmd_timecapsule(burrows_root: &Path, year: Option<i32>) {
    let name = require_active_burrow(burrows_root);
    let root = burrow_path(burrows_root, &name);
    let phlog_dir = root.join("phlog");
    let year = year.unwrap_or_else(|| Local::now().format("%Y").to_string().parse().unwrap());
    let year_str = year.to_string();

    if !phlog_dir.is_dir() {
        eprintln!("  No phlog/ directory found. Write some posts first!");
        std::process::exit(1);
    }

    // Gather all posts from the given year
    let mut posts: Vec<(String, String, usize)> = Vec::new(); // (date, title, word_count)
    let mut total_words: usize = 0;
    let mut total_bytes: u64 = 0;
    let mut months_active: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut longest_post: (String, usize) = (String::new(), 0);
    let mut shortest_post: (String, usize) = (String::new(), usize::MAX);

    if let Ok(entries) = fs::read_dir(&phlog_dir) {
        for entry in entries.flatten() {
            let filename = entry.file_name().to_string_lossy().to_string();
            if !filename.starts_with(&year_str) || filename.starts_with('.') || filename.starts_with('_') {
                continue;
            }
            if !filename.ends_with(".txt") {
                continue;
            }

            let content = fs::read_to_string(entry.path()).unwrap_or_default();
            let title = content.lines().next().unwrap_or("").trim_start_matches("# ").to_string();
            let words = content.split_whitespace().count();
            let bytes = fs::metadata(entry.path()).map(|m| m.len()).unwrap_or(0);
            let date = if filename.len() >= 10 { filename[..10].to_string() } else { filename.clone() };

            // Track month (YYYY-MM)
            if date.len() >= 7 {
                months_active.insert(date[..7].to_string());
            }

            if words > longest_post.1 {
                longest_post = (title.clone(), words);
            }
            if words < shortest_post.1 {
                shortest_post = (title.clone(), words);
            }

            total_words += words;
            total_bytes += bytes;
            posts.push((date, title, words));
        }
    }

    if posts.is_empty() {
        eprintln!("  No posts found for {}.", year);
        std::process::exit(1);
    }

    posts.sort_by(|a, b| a.0.cmp(&b.0));

    let first_post = &posts[0];
    let last_post = &posts[posts.len() - 1];
    let avg_words = total_words / posts.len();
    let reading_time = total_words / 230;

    // Generate the time capsule content
    let mut capsule = String::new();
    capsule.push_str(&format!("# Time Capsule — {}\n\n", year));
    capsule.push_str(&format!("A year in {}'s burrow.\n\n", name));
    capsule.push_str("---\n\n");

    // Stats
    capsule.push_str(&format!("  Posts written:     {}\n", posts.len()));
    capsule.push_str(&format!("  Words total:       {}\n", format_number(total_words)));
    capsule.push_str(&format!("  Average per post:  {} words\n", avg_words));
    capsule.push_str(&format!("  Total reading:     ~{} min\n", reading_time));
    capsule.push_str(&format!("  Months active:     {}/12\n", months_active.len()));
    capsule.push_str(&format!("  Total size:        {}\n", format_bytes(total_bytes)));
    capsule.push_str("\n---\n\n");

    // Highlights
    capsule.push_str(&format!("First post:    {} ({})\n", first_post.1, first_post.0));
    capsule.push_str(&format!("Last post:     {} ({})\n", last_post.1, last_post.0));
    capsule.push_str(&format!("Longest:       {} ({} words)\n", longest_post.0, longest_post.1));
    if shortest_post.1 < usize::MAX {
        capsule.push_str(&format!("Shortest:      {} ({} words)\n", shortest_post.0, shortest_post.1));
    }
    capsule.push_str("\n---\n\n");

    // Monthly breakdown
    capsule.push_str("Month by month:\n\n");
    let month_names = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    for (i, month_name) in month_names.iter().enumerate() {
        let month_key = format!("{}-{:02}", year, i + 1);
        let month_posts: Vec<_> = posts.iter().filter(|(d, _, _)| d.starts_with(&month_key)).collect();
        if month_posts.is_empty() {
            capsule.push_str(&format!("  {}  ·\n", month_name));
        } else {
            let month_words: usize = month_posts.iter().map(|(_, _, w)| w).sum();
            let bar = "█".repeat(month_posts.len().min(20));
            capsule.push_str(&format!("  {}  {} {} posts, {} words\n", month_name, bar, month_posts.len(), month_words));
        }
    }
    capsule.push_str("\n---\n\n");

    // All posts
    capsule.push_str("All posts:\n\n");
    for (date, title, words) in &posts {
        capsule.push_str(&format!("  {}  {}  ({} words)\n", date, title, words));
    }
    capsule.push_str(&format!("\n---\n\nGenerated by burrow on {}.\n", Local::now().format("%Y-%m-%d")));

    // Write to file
    let output_path = root.join(format!("timecapsule-{}.txt", year));
    fs::write(&output_path, &capsule).unwrap();

    println!();
    println!("  \x1b[1m/\x1b[0m Time Capsule — {}", year);
    println!();
    println!("  {} posts · {} words · {} months active",
        posts.len(), format_number(total_words), months_active.len());
    println!();
    println!("  Saved to \x1b[36m{}\x1b[0m", output_path.display());
    println!();
}

fn format_number(n: usize) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

fn format_bytes(b: u64) -> String {
    if b >= 1_048_576 {
        format!("{:.1} MB", b as f64 / 1_048_576.0)
    } else if b >= 1024 {
        format!("{:.1} KB", b as f64 / 1024.0)
    } else {
        format!("{} B", b)
    }
}

// ── Push / Pull ─────────────────────────────────────────────────

fn cmd_push(burrows_root: &Path, remote: &str) {
    let name = require_active_burrow(burrows_root);
    let local = burrow_path(burrows_root, &name);

    // Ensure remote ends with /
    let remote = if remote.ends_with('/') {
        remote.to_string()
    } else {
        format!("{}/", remote)
    };

    // Build the remote target: if remote is a burrows/ dir, append the burrow name
    let remote_target = if remote.contains(&name) {
        remote.clone()
    } else {
        format!("{}{}/", remote, name)
    };

    println!();
    println!("  \x1b[1m/\x1b[0m Pushing {} → {}", name, remote_target);
    println!();

    let status = Command::new("rsync")
        .args([
            "-avz",
            "--exclude", ".burrow-active",
            "--delete",
            &format!("{}/", local.display()),
            &remote_target,
        ])
        .status();

    match status {
        Ok(s) if s.success() => {
            println!();
            println!("  \x1b[32m✓\x1b[0m Push complete.");
            println!();
        }
        Ok(s) => {
            eprintln!("  \x1b[31m✗\x1b[0m rsync exited with code {}", s.code().unwrap_or(-1));
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("  \x1b[31m✗\x1b[0m Failed to run rsync: {}", e);
            eprintln!("  Make sure rsync is installed.");
            std::process::exit(1);
        }
    }
}

fn cmd_pull(burrows_root: &Path, remote: &str) {
    let name = require_active_burrow(burrows_root);
    let local = burrow_path(burrows_root, &name);

    // Ensure remote ends with /
    let remote = if remote.ends_with('/') {
        remote.to_string()
    } else {
        format!("{}/", remote)
    };

    println!();
    println!("  \x1b[1m/\x1b[0m Pulling {} ← {}", name, remote);
    println!();

    let status = Command::new("rsync")
        .args([
            "-avz",
            "--exclude", ".burrow-active",
            &remote,
            &format!("{}/", local.display()),
        ])
        .status();

    match status {
        Ok(s) if s.success() => {
            println!();
            println!("  \x1b[32m✓\x1b[0m Pull complete.");
            println!();
        }
        Ok(s) => {
            eprintln!("  \x1b[31m✗\x1b[0m rsync exited with code {}", s.code().unwrap_or(-1));
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("  \x1b[31m✗\x1b[0m Failed to run rsync: {}", e);
            eprintln!("  Make sure rsync is installed.");
            std::process::exit(1);
        }
    }
}

// ── Bookmarks ───────────────────────────────────────────────────

fn bookmarks_path(burrows_root: &Path, name: &str) -> PathBuf {
    burrow_path(burrows_root, name).join("bookmarks.gph")
}

fn cmd_bookmark_add(burrows_root: &Path, url: &str, desc: Option<&str>) {
    let name = require_active_burrow(burrows_root);
    let path = bookmarks_path(burrows_root, &name);

    let date = Local::now().format("%Y-%m-%d").to_string();
    let description = desc.unwrap_or(url);

    // Format: → URL   Description · date
    // or for internal: /path   Description · date
    let line = if url.starts_with("http://") || url.starts_with("https://") {
        format!("→ {}   {} · {}\n", url, description, date)
    } else {
        format!("{}   {} · {}\n", url, description, date)
    };

    let mut content = fs::read_to_string(&path).unwrap_or_default();
    content.push_str(&line);
    fs::write(&path, content).unwrap();

    println!();
    println!("  \x1b[1m/\x1b[0m Bookmark added to {}.", name);
    println!("  \x1b[36m{}\x1b[0m  {}", url, description);
    println!();
}

fn cmd_bookmark_list(burrows_root: &Path) {
    let name = require_active_burrow(burrows_root);
    let path = bookmarks_path(burrows_root, &name);

    if !path.exists() {
        println!();
        println!("  \x1b[90mNo bookmarks yet. Add one with `burrow bookmark add <url>`\x1b[0m");
        println!();
        return;
    }

    let content = fs::read_to_string(&path).unwrap_or_default();
    let bookmarks = parse_bookmarks(&content);

    if bookmarks.is_empty() {
        println!();
        println!("  \x1b[90mNo bookmarks yet. Add one with `burrow bookmark add <url>`\x1b[0m");
        println!();
        return;
    }

    println!();
    println!("  \x1b[1m/\x1b[0m Bookmarks — {}", name);
    println!();

    for (i, (url, desc)) in bookmarks.iter().enumerate() {
        println!("  \x1b[90m{:>3}\x1b[0m  \x1b[36m{}\x1b[0m", i + 1, url);
        if !desc.is_empty() && desc != url {
            println!("       \x1b[90m{}\x1b[0m", desc);
        }
    }
    println!();
    println!("  \x1b[90m{} bookmarks\x1b[0m", bookmarks.len());
    println!();
}

fn cmd_bookmark_remove(burrows_root: &Path, number: usize) {
    let name = require_active_burrow(burrows_root);
    let path = bookmarks_path(burrows_root, &name);

    if !path.exists() {
        eprintln!("  No bookmarks file found.");
        std::process::exit(1);
    }

    let content = fs::read_to_string(&path).unwrap_or_default();
    let lines: Vec<&str> = content.lines().collect();
    // Only count non-empty lines as bookmarks
    let bookmark_lines: Vec<(usize, &str)> = lines.iter().enumerate()
        .filter(|(_, l)| !l.trim().is_empty())
        .map(|(i, l)| (i, *l))
        .collect();

    if number == 0 || number > bookmark_lines.len() {
        eprintln!("  Invalid bookmark number. Use `burrow bookmark list` to see them.");
        std::process::exit(1);
    }

    let remove_idx = bookmark_lines[number - 1].0;
    let new_lines: Vec<&str> = lines.iter().enumerate()
        .filter(|(i, _)| *i != remove_idx)
        .map(|(_, l)| *l)
        .collect();

    let mut new_content = new_lines.join("\n");
    if !new_content.is_empty() {
        new_content.push('\n');
    }
    fs::write(&path, new_content).unwrap();

    println!();
    println!("  \x1b[1m/\x1b[0m Bookmark #{} removed.", number);
    println!();
}

fn parse_bookmarks(content: &str) -> Vec<(String, String)> {
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
            bookmarks.push((url, desc));
        } else if line.starts_with('/') {
            // Internal: /path   Description · date
            let parts: Vec<&str> = line.splitn(2, "   ").collect();
            let url = parts[0].trim().to_string();
            let desc = parts.get(1).unwrap_or(&"").to_string();
            bookmarks.push((url, desc));
        }
    }
    bookmarks
}

// ── Helpers ─────────────────────────────────────────────────────

fn slugify(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn first_line_of(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .next()
        .unwrap_or("")
        .trim_start_matches("# ")
        .to_string()
}

fn read_description(dir: &Path) -> String {
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

fn count_files_recursive(dir: &Path) -> (usize, u64) {
    let mut count = 0;
    let mut size = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }
            if path.is_dir() {
                let (c, s) = count_files_recursive(&path);
                count += c;
                size += s;
            } else {
                count += 1;
                size += fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            }
        }
    }
    (count, size)
}

fn latest_phlog_post(phlog_dir: &Path) -> Option<String> {
    let mut posts: Vec<String> = fs::read_dir(phlog_dir)
        .ok()?
        .flatten()
        .filter(|e| {
            let n = e.file_name().to_string_lossy().to_string();
            !n.starts_with('.') && !n.starts_with('_')
        })
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    posts.sort();
    posts.last().cloned()
}
