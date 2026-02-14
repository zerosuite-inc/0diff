use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HistoryEntry {
    pub timestamp: String,
    pub file: String,
    pub additions: usize,
    pub deletions: usize,
    pub author: Option<String>,
    pub branch: Option<String>,
    pub agent: Option<String>,
    pub summary: String,
}

pub struct HistoryStore {
    dir: PathBuf,
}

impl HistoryStore {
    /// Open (or create) the `.0diff/` history directory under `project_root`.
    pub fn open(project_root: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let dir = PathBuf::from(project_root).join(".0diff");
        fs::create_dir_all(&dir)?;
        Ok(Self { dir })
    }

    /// Append a single history entry as a JSON line.
    pub fn append(&self, entry: &HistoryEntry) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.history_path();
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        let json = serde_json::to_string(entry)?;
        writeln!(file, "{}", json)?;
        Ok(())
    }

    /// Query history entries with optional filters, newest first.
    pub fn query(
        &self,
        author: Option<&str>,
        agent: Option<&str>,
        limit: usize,
    ) -> Result<Vec<HistoryEntry>, Box<dyn std::error::Error>> {
        let mut entries = self.all_entries()?;

        // Filter by author (case-insensitive contains)
        if let Some(author_filter) = author {
            let filter_lower = author_filter.to_lowercase();
            entries.retain(|e| {
                e.author
                    .as_ref()
                    .map(|a| a.to_lowercase().contains(&filter_lower))
                    .unwrap_or(false)
            });
        }

        // Filter by agent (case-insensitive contains)
        if let Some(agent_filter) = agent {
            let filter_lower = agent_filter.to_lowercase();
            entries.retain(|e| {
                e.agent
                    .as_ref()
                    .map(|a| a.to_lowercase().contains(&filter_lower))
                    .unwrap_or(false)
            });
        }

        // Newest first
        entries.reverse();

        // Take limit
        entries.truncate(limit);

        Ok(entries)
    }

    /// Rotate history: remove entries older than `max_days` and truncate if
    /// the file exceeds `max_size_mb`.
    pub fn rotate(
        &self,
        max_size_mb: u64,
        max_days: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let entries = self.all_entries()?;
        let cutoff = Utc::now() - chrono::Duration::days(max_days as i64);

        // Filter out entries older than max_days
        let mut kept: Vec<HistoryEntry> = entries
            .into_iter()
            .filter(|e| {
                DateTime::parse_from_rfc3339(&e.timestamp)
                    .map(|dt| dt.with_timezone(&Utc) >= cutoff)
                    .unwrap_or(true) // keep entries with unparseable timestamps
            })
            .collect();

        // If serialized size still exceeds max_size_mb, drop oldest entries
        let max_bytes = max_size_mb * 1024 * 1024;
        loop {
            let size: usize = kept
                .iter()
                .map(|e| serde_json::to_string(e).unwrap_or_default().len() + 1) // +1 for newline
                .sum();
            if (size as u64) <= max_bytes || kept.is_empty() {
                break;
            }
            // Remove oldest (first) entry
            kept.remove(0);
        }

        // Rewrite the file
        let path = self.history_path();
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;

        for entry in &kept {
            let json = serde_json::to_string(entry)?;
            writeln!(file, "{}", json)?;
        }

        Ok(())
    }

    /// Read and deserialize all history entries, skipping invalid lines.
    pub fn all_entries(&self) -> Result<Vec<HistoryEntry>, Box<dyn std::error::Error>> {
        let path = self.history_path();
        if !path.exists() {
            return Ok(Vec::new());
        }

        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Ok(entry) = serde_json::from_str::<HistoryEntry>(trimmed) {
                entries.push(entry);
            }
            // Skip invalid lines gracefully
        }

        Ok(entries)
    }

    fn history_path(&self) -> PathBuf {
        self.dir.join("history.jsonl")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn unique_test_dir() -> PathBuf {
        let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "zerodiff_history_test_{}_{}",
            std::process::id(),
            id
        ));
        // Clean up from previous runs
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    fn make_entry(file: &str, additions: usize, deletions: usize) -> HistoryEntry {
        HistoryEntry {
            timestamp: Utc::now().to_rfc3339(),
            file: file.to_string(),
            additions,
            deletions,
            author: Some("Juste".to_string()),
            branch: Some("main".to_string()),
            agent: None,
            summary: format!("+{} -{}", additions, deletions),
        }
    }

    #[test]
    fn test_open_creates_directory() {
        let tmp = unique_test_dir();
        fs::create_dir_all(&tmp).unwrap();
        let root = tmp.to_str().unwrap();
        let store = HistoryStore::open(root).unwrap();
        assert!(store.dir.exists());
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_append_and_read() {
        let tmp = unique_test_dir();
        fs::create_dir_all(&tmp).unwrap();
        let root = tmp.to_str().unwrap();
        let store = HistoryStore::open(root).unwrap();

        let entry = make_entry("src/main.rs", 10, 3);
        store.append(&entry).unwrap();

        let entries = store.all_entries().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].file, "src/main.rs");
        assert_eq!(entries[0].additions, 10);
        assert_eq!(entries[0].deletions, 3);
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_query_newest_first() {
        let tmp = unique_test_dir();
        fs::create_dir_all(&tmp).unwrap();
        let root = tmp.to_str().unwrap();
        let store = HistoryStore::open(root).unwrap();

        for i in 0..5 {
            let entry = make_entry(&format!("file{}.rs", i), i, 0);
            store.append(&entry).unwrap();
        }

        let results = store.query(None, None, 10).unwrap();
        assert_eq!(results.len(), 5);
        // Newest first means file4.rs should be first
        assert_eq!(results[0].file, "file4.rs");
        assert_eq!(results[4].file, "file0.rs");
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_query_limit() {
        let tmp = unique_test_dir();
        fs::create_dir_all(&tmp).unwrap();
        let root = tmp.to_str().unwrap();
        let store = HistoryStore::open(root).unwrap();

        for i in 0..10 {
            store
                .append(&make_entry(&format!("f{}.rs", i), i, 0))
                .unwrap();
        }

        let results = store.query(None, None, 3).unwrap();
        assert_eq!(results.len(), 3);
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_query_filter_author() {
        let tmp = unique_test_dir();
        fs::create_dir_all(&tmp).unwrap();
        let root = tmp.to_str().unwrap();
        let store = HistoryStore::open(root).unwrap();

        let mut entry1 = make_entry("a.rs", 1, 0);
        entry1.author = Some("Alice".to_string());
        store.append(&entry1).unwrap();

        let mut entry2 = make_entry("b.rs", 2, 0);
        entry2.author = Some("Bob".to_string());
        store.append(&entry2).unwrap();

        let results = store.query(Some("alice"), None, 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].file, "a.rs");
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_query_filter_agent() {
        let tmp = unique_test_dir();
        fs::create_dir_all(&tmp).unwrap();
        let root = tmp.to_str().unwrap();
        let store = HistoryStore::open(root).unwrap();

        let mut entry1 = make_entry("a.rs", 1, 0);
        entry1.agent = Some("Claude".to_string());
        store.append(&entry1).unwrap();

        let mut entry2 = make_entry("b.rs", 2, 0);
        entry2.agent = None;
        store.append(&entry2).unwrap();

        let results = store.query(None, Some("claude"), 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].file, "a.rs");
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_all_entries_skips_invalid_lines() {
        let tmp = unique_test_dir();
        fs::create_dir_all(&tmp).unwrap();
        let root = tmp.to_str().unwrap();
        let store = HistoryStore::open(root).unwrap();

        // Write valid entry, invalid line, then valid entry
        let path = store.history_path();
        let entry = make_entry("good.rs", 1, 0);
        let json = serde_json::to_string(&entry).unwrap();
        fs::write(&path, format!("{}\nthis is garbage\n{}\n", json, json)).unwrap();

        let entries = store.all_entries().unwrap();
        assert_eq!(entries.len(), 2);
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_rotate_removes_old_entries() {
        let tmp = unique_test_dir();
        fs::create_dir_all(&tmp).unwrap();
        let root = tmp.to_str().unwrap();
        let store = HistoryStore::open(root).unwrap();

        // Write an entry with a very old timestamp
        let mut old_entry = make_entry("old.rs", 1, 0);
        old_entry.timestamp = "2020-01-01T00:00:00Z".to_string();
        store.append(&old_entry).unwrap();

        // Write a recent entry
        let recent_entry = make_entry("recent.rs", 2, 0);
        store.append(&recent_entry).unwrap();

        store.rotate(10, 30).unwrap();

        let entries = store.all_entries().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].file, "recent.rs");
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_empty_history() {
        let tmp = unique_test_dir();
        fs::create_dir_all(&tmp).unwrap();
        let root = tmp.to_str().unwrap();
        let store = HistoryStore::open(root).unwrap();

        let entries = store.all_entries().unwrap();
        assert!(entries.is_empty());

        let results = store.query(None, None, 10).unwrap();
        assert!(results.is_empty());
        let _ = fs::remove_dir_all(&tmp);
    }
}
