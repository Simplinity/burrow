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
}

fn main() {
    let cli = Cli::parse();
    let burrows_root = find_burrows_root();

    match cli.command {
        Commands::Init { name } => cmd_init(&burrows_root, &name),
        Commands::New { title } => cmd_new(&burrows_root, &title),
        Commands::Ls { path } => cmd_ls(&burrows_root, path.as_deref()),
        Commands::Status => cmd_status(&burrows_root),
        Commands::Edit { path } => cmd_edit(&burrows_root, &path),
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

fn cmd_init(burrows_root: &Path, name: &str) {
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
        format!("description = A fresh burrow\n"),
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
    println!("  Your burrow is live at \x1b[36mhttp://127.0.0.1:7070/{}\x1b[0m", name);
    println!();
    println!("  Write your first post:");
    println!("    \x1b[1mburrow new \"My first post\"\x1b[0m");
    println!();
}

fn cmd_new(burrows_root: &Path, title: &str) {
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
                    "  \x1b[32mPublished!\x1b[0m View at \x1b[36mhttp://127.0.0.1:7070/{}/phlog/{}\x1b[0m",
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

fn cmd_status(burrows_root: &Path) {
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
    println!("  Server:   \x1b[36mhttp://127.0.0.1:7070/{}\x1b[0m", name);
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
