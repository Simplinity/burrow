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
    /// Show burrow status
    Status,
    /// Open a file in your editor
    Edit {
        /// Path to file (e.g. "about.txt" or "phlog/my-post.txt")
        path: String,
    },
    /// Manage your guestbook
    Guestbook {
        #[command(subcommand)]
        command: GuestbookCommands,
    },
    /// Switch active burrow (or list all burrows)
    Switch {
        /// Burrow name to switch to (e.g. "bruno" or "~bruno"). Omit to list all.
        name: Option<String>,
    },
    /// Server management
    Server {
        #[command(subcommand)]
        command: ServerCommands,
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
        Commands::Status => cmd_status(&burrows_root, &cfg),
        Commands::Guestbook { command } => match command {
            GuestbookCommands::Init => cmd_guestbook_init(&burrows_root),
            GuestbookCommands::Show => cmd_guestbook_show(&burrows_root),
        },
        Commands::Switch { name } => cmd_switch(&burrows_root, name.as_deref()),
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

fn cmd_switch(burrows_root: &Path, name: Option<&str>) {
    let burrows: Vec<String> = fs::read_dir(burrows_root)
        .unwrap_or_else(|_| { eprintln!("  No burrows/ directory found."); std::process::exit(1); })
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

    let active = find_active_burrow(burrows_root).unwrap_or_default();

    match name {
        Some(target) => {
            let target = if target.starts_with('~') {
                target.to_string()
            } else {
                format!("~{}", target)
            };

            if !burrows.contains(&target) {
                eprintln!("  Burrow {} not found.", target);
                eprintln!();
                eprintln!("  Available burrows:");
                for b in &burrows {
                    eprintln!("    {}", b);
                }
                std::process::exit(1);
            }

            fs::write(burrows_root.join(".burrow-active"), &target).unwrap();
            println!();
            println!("  \x1b[1m/\x1b[0m Switched to \x1b[36m{}\x1b[0m", target);
            println!();
        }
        None => {
            println!();
            println!("  \x1b[1m/\x1b[0m Burrows");
            println!();
            for b in &burrows {
                let marker = if *b == active { " \x1b[36m←\x1b[0m" } else { "" };
                let desc = read_description(&burrows_root.join(b));
                if desc.is_empty() {
                    println!("  {}{}", b, marker);
                } else {
                    println!("  {}  \x1b[90m{}\x1b[0m{}", b, desc, marker);
                }
            }
            println!();
            println!("  Switch with: \x1b[1mburrow switch <name>\x1b[0m");
            println!();
        }
    }
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
