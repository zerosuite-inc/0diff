use clap::{Parser, Subcommand};
use std::process;

use zerodiff::config::Config;
use zerodiff::differ;
use zerodiff::git::GitRepo;
use zerodiff::history::HistoryStore;
use zerodiff::output::{self, OutputFormat};
use zerodiff::watcher;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Parser)]
#[command(
    name = "0diff",
    version,
    about = "Real-time code modification tracking for the multi-agent era",
    long_about = "Know who changed what. Even when it's not human.\n\n\
        0diff watches your codebase for changes, tracks diffs with git metadata,\n\
        and detects AI agent edits — all in real-time."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output as JSON (available on all commands)
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Create .0diff.toml config in current directory
    Init,

    /// Watch files for changes (foreground)
    Watch,

    /// Show current diff vs last commit for a file
    Diff {
        /// File path to diff
        file: String,
    },

    /// Show recent change history
    Log {
        /// Filter by author name
        #[arg(long)]
        author: Option<String>,

        /// Filter by agent name (e.g. "Claude", "Copilot")
        #[arg(long)]
        agent: Option<String>,

        /// Maximum number of entries to show
        #[arg(short = 'n', long, default_value = "20")]
        limit: usize,
    },

    /// Show what's being watched
    Status,
}

fn main() {
    let cli = Cli::parse();
    let format = if cli.json {
        OutputFormat::Json
    } else {
        OutputFormat::Terminal
    };

    let result = match cli.command {
        Commands::Init => cmd_init(format),
        Commands::Watch => cmd_watch(format),
        Commands::Diff { file } => cmd_diff(&file, format),
        Commands::Log {
            author,
            agent,
            limit,
        } => cmd_log(author, agent, limit, format),
        Commands::Status => cmd_status(format),
    };

    if let Err(e) = result {
        output::print_error(&e.to_string(), format);
        process::exit(1);
    }
}

fn cmd_init(format: OutputFormat) -> Result<()> {
    let path = std::env::current_dir()?.join(".0diff.toml");
    if path.exists() {
        output::print_warning(".0diff.toml already exists", format);
        return Ok(());
    }

    let template = Config::template();
    std::fs::write(&path, template)?;

    let dir = std::env::current_dir()?.join(".0diff");
    std::fs::create_dir_all(&dir)?;

    output::print_success("Created .0diff.toml and .0diff/ directory", format);
    Ok(())
}

fn cmd_watch(format: OutputFormat) -> Result<()> {
    let config = load_config()?;
    output::print_info(
        &format!(
            "Watching {} paths for changes...",
            config.watch.paths.len()
        ),
        format,
    );
    output::print_info("Press Ctrl+C to stop", format);
    watcher::run(config, format)?;
    Ok(())
}

fn cmd_diff(file: &str, format: OutputFormat) -> Result<()> {
    let config = load_config_or_default();
    let path = std::path::Path::new(file);
    if !path.exists() {
        return Err(format!("File not found: {}", file).into());
    }

    let git = GitRepo::new(".")?;
    let committed = git.file_contents_at_head(file)?;
    let current = std::fs::read_to_string(file)?;

    let diff = differ::compute_diff(&committed, &current, file);
    let diff = if config.filter.ignore_whitespace {
        zerodiff::filter::filter_whitespace_changes(diff)
    } else {
        diff
    };

    if diff.hunks.is_empty() {
        output::print_info(&format!("No changes in {}", file), format);
    } else {
        output::print_diff(&diff, format);
    }
    Ok(())
}

fn cmd_log(
    author: Option<String>,
    agent: Option<String>,
    limit: usize,
    format: OutputFormat,
) -> Result<()> {
    let store = HistoryStore::open(".")?;
    let entries = store.query(author.as_deref(), agent.as_deref(), limit)?;

    if entries.is_empty() {
        output::print_info("No history entries found", format);
    } else {
        output::print_history(&entries, format);
    }
    Ok(())
}

fn cmd_status(format: OutputFormat) -> Result<()> {
    let config = load_config()?;
    let git = GitRepo::new(".");

    let git_info = git.as_ref().ok().map(|g| {
        let branch = g.current_branch().unwrap_or_default();
        let author = g.current_author().unwrap_or_default();
        (branch, author)
    });

    output::print_status(&config, git_info.as_ref(), format);
    Ok(())
}

fn load_config() -> Result<Config> {
    let path = std::env::current_dir()?.join(".0diff.toml");
    if !path.exists() {
        return Err("No .0diff.toml found. Run `0diff init` first.".into());
    }
    Config::load(&path)
}

fn load_config_or_default() -> Config {
    let path = std::env::current_dir()
        .ok()
        .map(|p| p.join(".0diff.toml"));
    match path {
        Some(p) if p.exists() => Config::load(&p).unwrap_or_default(),
        _ => Config::default(),
    }
}
