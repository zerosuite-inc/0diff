use crate::agents::AgentDetector;
use crate::config::Config;
use crate::differ;
use crate::filter;
use crate::git::GitRepo;
use crate::history::{HistoryEntry, HistoryStore};
use crate::output::{self, OutputFormat};

use chrono::Utc;
use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

pub fn run(config: Config, format: OutputFormat) -> Result<(), Box<dyn std::error::Error>> {
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_flag = shutdown.clone();

    ctrlc::set_handler(move || {
        shutdown_flag.store(true, Ordering::SeqCst);
    })?;

    // Initialize file cache — read current contents of all watched files
    let mut file_cache: HashMap<PathBuf, String> = HashMap::new();
    for watch_path in &config.watch.paths {
        let path = Path::new(watch_path);
        if path.exists() {
            cache_directory(&mut file_cache, path, &config);
        }
    }

    output::print_info(
        &format!("Cached {} files", file_cache.len()),
        format,
    );

    // Set up git integration (optional)
    let git = GitRepo::new(".").ok();
    let agent_detector = AgentDetector::new(config.agents.detect_patterns.clone());
    let history = HistoryStore::open(".")?;

    // Set up file watcher with debouncing
    let (tx, rx) = std::sync::mpsc::channel();
    let debounce_duration = Duration::from_millis(config.watch.debounce_ms);
    let mut debouncer = new_debouncer(debounce_duration, tx)?;

    // Watch configured paths
    for watch_path in &config.watch.paths {
        let path = Path::new(watch_path);
        if path.exists() {
            debouncer
                .watcher()
                .watch(path, notify::RecursiveMode::Recursive)?;
        }
    }

    // Event loop
    while !shutdown.load(Ordering::SeqCst) {
        match rx.recv_timeout(Duration::from_millis(250)) {
            Ok(Ok(events)) => {
                for event in events {
                    if shutdown.load(Ordering::SeqCst) {
                        break;
                    }

                    let path = &event.path;

                    // Skip if not a watched file
                    let relative = make_relative(path);
                    if !config.should_watch(&relative) {
                        continue;
                    }

                    match event.kind {
                        DebouncedEventKind::Any => {
                            if path.is_file() {
                                handle_file_change(
                                    path,
                                    &relative,
                                    &mut file_cache,
                                    &config,
                                    &git,
                                    &agent_detector,
                                    &history,
                                    format,
                                );
                            } else if !path.exists() {
                                handle_file_delete(
                                    path,
                                    &relative,
                                    &mut file_cache,
                                    &git,
                                    &agent_detector,
                                    &history,
                                    format,
                                );
                            }
                        }
                        _ => {
                            // Other event kinds — skip
                        }
                    }
                }
            }
            Ok(Err(error)) => {
                output::print_error(&format!("Watch error: {}", error), format);
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // No events — continue loop (check shutdown flag)
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                break;
            }
        }
    }

    // Rotate history on exit
    let _ = history.rotate(config.history.max_size_mb, config.history.max_days);

    output::print_info("Stopped watching", format);
    Ok(())
}

fn handle_file_change(
    path: &Path,
    relative: &Path,
    cache: &mut HashMap<PathBuf, String>,
    config: &Config,
    git: &Option<GitRepo>,
    detector: &AgentDetector,
    history: &HistoryStore,
    format: OutputFormat,
) {
    let new_contents = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let old_contents = cache.get(path).cloned().unwrap_or_default();

    let rel_str = relative.to_string_lossy().to_string();
    let diff = differ::compute_diff(&old_contents, &new_contents, &rel_str);

    let diff = if config.filter.ignore_whitespace {
        filter::filter_whitespace_changes(diff)
    } else {
        diff
    };

    if diff.hunks.is_empty() || (diff.additions + diff.deletions) < config.filter.min_lines_changed {
        cache.insert(path.to_path_buf(), new_contents);
        return;
    }

    let (author, branch) = if config.git.enabled {
        let author = git.as_ref().and_then(|g| g.current_author().ok());
        let branch = git.as_ref().and_then(|g| g.current_branch().ok());
        (author, branch)
    } else {
        (None, None)
    };

    let recent_commit = git
        .as_ref()
        .and_then(|g| g.recent_commits(1).ok())
        .and_then(|commits| commits.into_iter().next());
    let agent = detector.tag_for_entry(recent_commit.as_ref());

    let entry = HistoryEntry {
        timestamp: Utc::now().to_rfc3339(),
        file: rel_str,
        additions: diff.additions,
        deletions: diff.deletions,
        author,
        branch,
        agent,
        summary: format!("+{} -{}", diff.additions, diff.deletions),
    };

    let _ = history.append(&entry);
    output::print_change_event(&entry, &diff, format);
    cache.insert(path.to_path_buf(), new_contents);
}

fn handle_file_delete(
    path: &Path,
    relative: &Path,
    cache: &mut HashMap<PathBuf, String>,
    git: &Option<GitRepo>,
    detector: &AgentDetector,
    history: &HistoryStore,
    format: OutputFormat,
) {
    let old_contents = cache.remove(path).unwrap_or_default();
    if old_contents.is_empty() {
        return;
    }

    let rel_str = relative.to_string_lossy().to_string();
    let line_count = old_contents.lines().count();

    let author = git.as_ref().and_then(|g| g.current_author().ok());
    let branch = git.as_ref().and_then(|g| g.current_branch().ok());

    let recent_commit = git
        .as_ref()
        .and_then(|g| g.recent_commits(1).ok())
        .and_then(|commits| commits.into_iter().next());
    let agent = detector.tag_for_entry(recent_commit.as_ref());

    let entry = HistoryEntry {
        timestamp: Utc::now().to_rfc3339(),
        file: rel_str,
        additions: 0,
        deletions: line_count,
        author,
        branch,
        agent,
        summary: format!("+0 -{}", line_count),
    };

    let _ = history.append(&entry);
    let diff = differ::compute_diff(&old_contents, "", &entry.file);
    output::print_change_event(&entry, &diff, format);
}

fn cache_directory(cache: &mut HashMap<PathBuf, String>, dir: &Path, config: &Config) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let dir_name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let should_skip = config.watch.ignore.iter().any(|pattern| {
                let clean = pattern.trim_end_matches('/');
                dir_name == clean
            });
            if !should_skip {
                cache_directory(cache, &path, config);
            }
        } else if path.is_file() {
            let relative = make_relative(&path);
            if config.should_watch(&relative) {
                if let Ok(contents) = std::fs::read_to_string(&path) {
                    cache.insert(path, contents);
                }
            }
        }
    }
}

fn make_relative(path: &Path) -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_default();
    path.strip_prefix(&cwd).unwrap_or(path).to_path_buf()
}
