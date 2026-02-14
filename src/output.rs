use crate::config::Config;
use crate::differ::{DiffLine, FileDiff};
use crate::history::HistoryEntry;
use colored::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OutputFormat {
    Terminal,
    Json,
}

/// Print a file diff to stdout.
pub fn print_diff(diff: &FileDiff, format: OutputFormat) {
    match format {
        OutputFormat::Terminal => {
            println!(
                "{} {} {}",
                "---".bold(),
                diff.file_path.bold(),
                format!(
                    "{} {}",
                    format!("+{}", diff.additions).green(),
                    format!("-{}", diff.deletions).red()
                )
            );

            for hunk in &diff.hunks {
                println!(
                    "{}",
                    format!(
                        "@@ -{},{} +{},{} @@",
                        hunk.old_start, hunk.old_count, hunk.new_start, hunk.new_count
                    )
                    .cyan()
                );
                for line in &hunk.lines {
                    match line {
                        DiffLine::Context(text) => print!(" {}", text),
                        DiffLine::Add(text) => print!("{}", format!("+{}", text).green()),
                        DiffLine::Delete(text) => print!("{}", format!("-{}", text).red()),
                    }
                }
            }
        }
        OutputFormat::Json => {
            if let Ok(json) = serde_json::to_string_pretty(diff) {
                println!("{}", json);
            }
        }
    }
}

/// Print a real-time change event (history entry + diff).
pub fn print_change_event(entry: &HistoryEntry, diff: &FileDiff, format: OutputFormat) {
    match format {
        OutputFormat::Terminal => {
            // Extract HH:MM:SS from ISO timestamp
            let time_str = extract_time(&entry.timestamp);

            let author_info = match (&entry.author, &entry.branch) {
                (Some(author), Some(branch)) => format!("({} on {})", author, branch),
                (Some(author), None) => format!("({})", author),
                (None, Some(branch)) => format!("(on {})", branch),
                (None, None) => String::new(),
            };

            let agent_tag = if entry.agent.is_some() {
                format!(" {}", "[AI AGENT]".yellow().bold())
            } else {
                String::new()
            };

            println!(
                "{} {}  {} {}  {}{}",
                format!("[{}]", time_str).dimmed(),
                entry.file.bold().white(),
                format!("+{}", entry.additions).green(),
                format!("-{}", entry.deletions).red(),
                author_info,
                agent_tag,
            );

            // Print diff hunks below
            for hunk in &diff.hunks {
                println!(
                    "{}",
                    format!(
                        "@@ -{},{} +{},{} @@",
                        hunk.old_start, hunk.old_count, hunk.new_start, hunk.new_count
                    )
                    .cyan()
                );
                for line in &hunk.lines {
                    match line {
                        DiffLine::Context(text) => print!(" {}", text),
                        DiffLine::Add(text) => print!("{}", format!("+{}", text).green()),
                        DiffLine::Delete(text) => print!("{}", format!("-{}", text).red()),
                    }
                }
            }
        }
        OutputFormat::Json => {
            #[derive(serde::Serialize)]
            struct ChangeEvent<'a> {
                entry: &'a HistoryEntry,
                diff: &'a FileDiff,
            }
            let event = ChangeEvent { entry, diff };
            if let Ok(json) = serde_json::to_string_pretty(&event) {
                println!("{}", json);
            }
        }
    }
}

/// Print a list of history entries.
pub fn print_history(entries: &[HistoryEntry], format: OutputFormat) {
    match format {
        OutputFormat::Terminal => {
            for entry in entries {
                let time_str = extract_time(&entry.timestamp);
                let author_info = entry
                    .author
                    .as_deref()
                    .map(|a| format!("({})", a))
                    .unwrap_or_default();

                let agent_tag = if entry.agent.is_some() {
                    format!(" {}", "[AI AGENT]".yellow().bold())
                } else {
                    String::new()
                };

                println!(
                    "{} {}  {} {}  {}{}",
                    format!("[{}]", time_str).dimmed(),
                    entry.file.bold().white(),
                    format!("+{}", entry.additions).green(),
                    format!("-{}", entry.deletions).red(),
                    author_info,
                    agent_tag,
                );
            }
        }
        OutputFormat::Json => {
            if let Ok(json) = serde_json::to_string_pretty(entries) {
                println!("{}", json);
            }
        }
    }
}

/// Print status information about the current configuration.
pub fn print_status(config: &Config, git_info: Option<&(String, String)>, format: OutputFormat) {
    match format {
        OutputFormat::Terminal => {
            println!(
                "{} {}",
                "Watching:".bold(),
                config.watch.paths.join(", ")
            );
            println!(
                "{} {}",
                "Extensions:".bold(),
                config.watch.extensions.join(", ")
            );
            println!(
                "{} {}",
                "Ignoring:".bold(),
                config.watch.ignore.join(", ")
            );
            if let Some((branch, author)) = git_info {
                println!(
                    "{} branch={}, author={}",
                    "Git:".bold(),
                    branch.cyan(),
                    author.cyan()
                );
            } else {
                println!("{} {}", "Git:".bold(), "not available".dimmed());
            }
            println!(
                "{} {}",
                "Agent detection:".bold(),
                config.agents.detect_patterns.join(", ")
            );
        }
        OutputFormat::Json => {
            #[derive(serde::Serialize)]
            struct StatusInfo<'a> {
                watch_paths: &'a [String],
                extensions: &'a [String],
                ignore: &'a [String],
                git_branch: Option<&'a str>,
                git_author: Option<&'a str>,
                agent_patterns: &'a [String],
            }
            let status = StatusInfo {
                watch_paths: &config.watch.paths,
                extensions: &config.watch.extensions,
                ignore: &config.watch.ignore,
                git_branch: git_info.map(|(b, _)| b.as_str()),
                git_author: git_info.map(|(_, a)| a.as_str()),
                agent_patterns: &config.agents.detect_patterns,
            };
            if let Ok(json) = serde_json::to_string_pretty(&status) {
                println!("{}", json);
            }
        }
    }
}

pub fn print_success(msg: &str, format: OutputFormat) {
    match format {
        OutputFormat::Terminal => println!("{} {}", "✓".green(), msg),
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({"status": "success", "message": msg})
            );
        }
    }
}

pub fn print_error(msg: &str, format: OutputFormat) {
    match format {
        OutputFormat::Terminal => eprintln!("{} {}", "✗".red(), msg),
        OutputFormat::Json => {
            eprintln!(
                "{}",
                serde_json::json!({"status": "error", "message": msg})
            );
        }
    }
}

pub fn print_warning(msg: &str, format: OutputFormat) {
    match format {
        OutputFormat::Terminal => println!("{} {}", "⚠".yellow(), msg),
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({"status": "warning", "message": msg})
            );
        }
    }
}

pub fn print_info(msg: &str, format: OutputFormat) {
    match format {
        OutputFormat::Terminal => println!("{} {}", "ℹ".cyan(), msg),
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({"status": "info", "message": msg})
            );
        }
    }
}

/// Extract HH:MM:SS from an ISO 8601 timestamp string.
fn extract_time(timestamp: &str) -> String {
    // Try to find the T separator and extract time portion
    if let Some(t_pos) = timestamp.find('T') {
        let after_t = &timestamp[t_pos + 1..];
        // Take up to 8 characters (HH:MM:SS)
        let time_part: String = after_t.chars().take(8).collect();
        if time_part.len() >= 8 {
            return time_part;
        }
    }
    // Fallback: return the raw timestamp
    timestamp.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::differ::{DiffHunk, DiffLine, FileDiff};
    use crate::history::HistoryEntry;

    fn sample_diff() -> FileDiff {
        FileDiff {
            file_path: "src/main.rs".to_string(),
            hunks: vec![DiffHunk {
                old_start: 1,
                old_count: 3,
                new_start: 1,
                new_count: 4,
                lines: vec![
                    DiffLine::Context("line1\n".to_string()),
                    DiffLine::Delete("old_line\n".to_string()),
                    DiffLine::Add("new_line\n".to_string()),
                    DiffLine::Add("extra_line\n".to_string()),
                    DiffLine::Context("line3\n".to_string()),
                ],
            }],
            additions: 2,
            deletions: 1,
        }
    }

    fn sample_entry() -> HistoryEntry {
        HistoryEntry {
            timestamp: "2026-02-14T10:30:00Z".to_string(),
            file: "src/main.rs".to_string(),
            additions: 2,
            deletions: 1,
            author: Some("Juste".to_string()),
            branch: Some("main".to_string()),
            agent: None,
            summary: "+2 -1".to_string(),
        }
    }

    #[test]
    fn test_json_diff_is_valid_json() {
        let diff = sample_diff();
        let json = serde_json::to_string_pretty(&diff).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["file_path"], "src/main.rs");
        assert_eq!(parsed["additions"], 2);
        assert_eq!(parsed["deletions"], 1);
    }

    #[test]
    fn test_json_history_is_valid_json() {
        let entries = vec![sample_entry()];
        let json = serde_json::to_string_pretty(&entries).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.is_array());
        assert_eq!(parsed[0]["file"], "src/main.rs");
        assert_eq!(parsed[0]["author"], "Juste");
    }

    #[test]
    fn test_json_change_event_is_valid_json() {
        #[derive(serde::Serialize)]
        struct ChangeEvent<'a> {
            entry: &'a HistoryEntry,
            diff: &'a FileDiff,
        }
        let entry = sample_entry();
        let diff = sample_diff();
        let event = ChangeEvent {
            entry: &entry,
            diff: &diff,
        };
        let json = serde_json::to_string_pretty(&event).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["entry"]["file"], "src/main.rs");
        assert_eq!(parsed["diff"]["additions"], 2);
    }

    #[test]
    fn test_json_status_is_valid_json() {
        #[derive(serde::Serialize)]
        struct StatusInfo {
            watch_paths: Vec<String>,
            extensions: Vec<String>,
            ignore: Vec<String>,
            git_branch: Option<String>,
            git_author: Option<String>,
            agent_patterns: Vec<String>,
        }
        let status = StatusInfo {
            watch_paths: vec!["src/".to_string()],
            extensions: vec!["rs".to_string()],
            ignore: vec!["target/".to_string()],
            git_branch: Some("main".to_string()),
            git_author: Some("Juste".to_string()),
            agent_patterns: vec!["Claude".to_string()],
        };
        let json = serde_json::to_string_pretty(&status).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["git_branch"], "main");
        assert_eq!(parsed["agent_patterns"][0], "Claude");
    }

    #[test]
    fn test_json_success_message() {
        let json = serde_json::json!({"status": "success", "message": "done"});
        let parsed: serde_json::Value = serde_json::from_str(&json.to_string()).unwrap();
        assert_eq!(parsed["status"], "success");
        assert_eq!(parsed["message"], "done");
    }

    #[test]
    fn test_json_error_message() {
        let json = serde_json::json!({"status": "error", "message": "fail"});
        let parsed: serde_json::Value = serde_json::from_str(&json.to_string()).unwrap();
        assert_eq!(parsed["status"], "error");
        assert_eq!(parsed["message"], "fail");
    }

    #[test]
    fn test_extract_time() {
        assert_eq!(extract_time("2026-02-14T10:30:45Z"), "10:30:45");
        assert_eq!(
            extract_time("2026-02-14T23:59:59+00:00"),
            "23:59:59"
        );
        // Fallback for non-ISO
        assert_eq!(extract_time("not a timestamp"), "not a timestamp");
    }

    #[test]
    fn test_output_format_debug() {
        assert_eq!(format!("{:?}", OutputFormat::Terminal), "Terminal");
        assert_eq!(format!("{:?}", OutputFormat::Json), "Json");
    }

    #[test]
    fn test_output_format_equality() {
        assert_eq!(OutputFormat::Terminal, OutputFormat::Terminal);
        assert_eq!(OutputFormat::Json, OutputFormat::Json);
        assert_ne!(OutputFormat::Terminal, OutputFormat::Json);
    }

    #[test]
    fn test_agent_entry_json() {
        let mut entry = sample_entry();
        entry.agent = Some("Claude".to_string());
        let json = serde_json::to_string(&entry).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["agent"], "Claude");
    }
}
