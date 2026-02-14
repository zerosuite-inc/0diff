use crate::differ::{DiffHunk, DiffLine, FileDiff};

/// Remove hunks where all changes are whitespace-only.
/// Recomputes additions/deletions after filtering.
pub fn filter_whitespace_changes(diff: FileDiff) -> FileDiff {
    let filtered_hunks: Vec<DiffHunk> = diff
        .hunks
        .into_iter()
        .filter(|hunk| !is_whitespace_only_hunk(hunk))
        .collect();

    let mut additions = 0;
    let mut deletions = 0;
    for hunk in &filtered_hunks {
        for line in &hunk.lines {
            match line {
                DiffLine::Add(_) => additions += 1,
                DiffLine::Delete(_) => deletions += 1,
                DiffLine::Context(_) => {}
            }
        }
    }

    FileDiff {
        file_path: diff.file_path,
        hunks: filtered_hunks,
        additions,
        deletions,
    }
}

/// A hunk is whitespace-only if every Add/Delete pair differs only in whitespace.
/// We pair up deletions and additions in order; if all pairs are whitespace-only
/// and there are no unpaired changes, the hunk is whitespace-only.
fn is_whitespace_only_hunk(hunk: &DiffHunk) -> bool {
    let adds: Vec<&str> = hunk
        .lines
        .iter()
        .filter_map(|l| match l {
            DiffLine::Add(s) => Some(s.as_str()),
            _ => None,
        })
        .collect();

    let dels: Vec<&str> = hunk
        .lines
        .iter()
        .filter_map(|l| match l {
            DiffLine::Delete(s) => Some(s.as_str()),
            _ => None,
        })
        .collect();

    // Must have same number of adds and deletes to be a pure whitespace change
    if adds.len() != dels.len() {
        return false;
    }

    // If there are no changes at all, not a whitespace hunk (it's just context)
    if adds.is_empty() {
        return false;
    }

    adds.iter()
        .zip(dels.iter())
        .all(|(a, d)| is_whitespace_only_change(a, d))
}

/// Two lines differ only in whitespace if, after trimming and collapsing
/// internal whitespace, they produce the same string.
fn is_whitespace_only_change(a: &str, b: &str) -> bool {
    normalize_whitespace(a) == normalize_whitespace(b)
}

/// Trim and collapse all internal whitespace runs to a single space.
fn normalize_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<&str>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::differ::compute_diff;

    #[test]
    fn test_whitespace_filter_removes_whitespace_only() {
        let old = "hello   world\n";
        let new = "hello world\n";
        let diff = compute_diff(old, new, "ws.txt");

        assert!(!diff.hunks.is_empty(), "should detect a change");

        let filtered = filter_whitespace_changes(diff);
        assert!(
            filtered.hunks.is_empty(),
            "whitespace-only change should be filtered out"
        );
        assert_eq!(filtered.additions, 0);
        assert_eq!(filtered.deletions, 0);
    }

    #[test]
    fn test_whitespace_filter_keeps_real_changes() {
        let old = "hello world\n";
        let new = "goodbye world\n";
        let diff = compute_diff(old, new, "real.txt");

        let filtered = filter_whitespace_changes(diff);
        assert!(
            !filtered.hunks.is_empty(),
            "real changes should be kept"
        );
        assert_eq!(filtered.additions, 1);
        assert_eq!(filtered.deletions, 1);
    }

    #[test]
    fn test_whitespace_filter_mixed_hunk() {
        // A hunk with both whitespace and real changes should be kept
        let old = "line1\nhello   world\nline3\n";
        let new = "LINE1\nhello world\nline3\n";
        let diff = compute_diff(old, new, "mixed.txt");

        let filtered = filter_whitespace_changes(diff);
        assert!(
            !filtered.hunks.is_empty(),
            "hunk with real changes should be kept even if some are whitespace"
        );
    }

    #[test]
    fn test_whitespace_filter_empty_diff() {
        let text = "same\n";
        let diff = compute_diff(text, text, "same.txt");
        let filtered = filter_whitespace_changes(diff);

        assert!(filtered.hunks.is_empty());
        assert_eq!(filtered.additions, 0);
        assert_eq!(filtered.deletions, 0);
    }

    #[test]
    fn test_whitespace_filter_indentation_change() {
        let old = "  indented\n";
        let new = "    indented\n";
        let diff = compute_diff(old, new, "indent.txt");

        let filtered = filter_whitespace_changes(diff);
        assert!(
            filtered.hunks.is_empty(),
            "indentation-only change should be filtered"
        );
    }

    #[test]
    fn test_whitespace_filter_tabs_vs_spaces() {
        let old = "\tindented\n";
        let new = "    indented\n";
        let diff = compute_diff(old, new, "tabs.txt");

        let filtered = filter_whitespace_changes(diff);
        assert!(
            filtered.hunks.is_empty(),
            "tabs-vs-spaces change should be filtered"
        );
    }

    #[test]
    fn test_normalize_whitespace() {
        assert_eq!(normalize_whitespace("  hello   world  "), "hello world");
        assert_eq!(normalize_whitespace("\thello\tworld\t"), "hello world");
        assert_eq!(normalize_whitespace("no_change"), "no_change");
        assert_eq!(normalize_whitespace("   "), "");
    }

    #[test]
    fn test_is_whitespace_only_change() {
        assert!(is_whitespace_only_change("hello  world", "hello world"));
        assert!(is_whitespace_only_change("  hello  ", "hello"));
        assert!(!is_whitespace_only_change("hello", "goodbye"));
        assert!(!is_whitespace_only_change("hello world", "helloworld"));
    }
}
