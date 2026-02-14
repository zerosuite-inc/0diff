use serde::Serialize;
use similar::{ChangeTag, TextDiff};

#[derive(Clone, Debug, Serialize)]
pub struct FileDiff {
    pub file_path: String,
    pub hunks: Vec<DiffHunk>,
    pub additions: usize,
    pub deletions: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct DiffHunk {
    pub old_start: usize,
    pub old_count: usize,
    pub new_start: usize,
    pub new_count: usize,
    pub lines: Vec<DiffLine>,
}

#[derive(Clone, Debug, Serialize)]
pub enum DiffLine {
    Context(String),
    Add(String),
    Delete(String),
}

pub fn compute_diff(old: &str, new: &str, file_path: &str) -> FileDiff {
    let diff = TextDiff::from_lines(old, new);
    let mut hunks = Vec::new();
    let mut total_additions = 0;
    let mut total_deletions = 0;

    for group in diff.grouped_ops(3) {
        let mut lines = Vec::new();
        let mut old_start = 0;
        let mut old_end = 0;
        let mut new_start = 0;
        let mut new_end = 0;

        for op in &group {
            // Track the range of this group
            if lines.is_empty() {
                old_start = op.old_range().start;
                new_start = op.new_range().start;
            }
            old_end = op.old_range().end;
            new_end = op.new_range().end;

            for change in diff.iter_changes(op) {
                let text = change.value().to_string();
                match change.tag() {
                    ChangeTag::Equal => {
                        lines.push(DiffLine::Context(text));
                    }
                    ChangeTag::Insert => {
                        lines.push(DiffLine::Add(text));
                        total_additions += 1;
                    }
                    ChangeTag::Delete => {
                        lines.push(DiffLine::Delete(text));
                        total_deletions += 1;
                    }
                }
            }
        }

        // Skip empty groups (identical files produce a single empty group)
        if lines.is_empty() {
            continue;
        }

        hunks.push(DiffHunk {
            old_start: old_start + 1, // 1-indexed for display
            old_count: old_end - old_start,
            new_start: new_start + 1,
            new_count: new_end - new_start,
            lines,
        });
    }

    FileDiff {
        file_path: file_path.to_string(),
        hunks,
        additions: total_additions,
        deletions: total_deletions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_diff() {
        let old = "line1\nline2\nline3\n";
        let new = "line1\nmodified\nline3\n";
        let diff = compute_diff(old, new, "test.txt");

        assert_eq!(diff.file_path, "test.txt");
        assert_eq!(diff.additions, 1);
        assert_eq!(diff.deletions, 1);
        assert!(!diff.hunks.is_empty());

        // Check that the hunk contains the expected changes
        let hunk = &diff.hunks[0];
        let has_add = hunk
            .lines
            .iter()
            .any(|l| matches!(l, DiffLine::Add(s) if s.contains("modified")));
        let has_del = hunk
            .lines
            .iter()
            .any(|l| matches!(l, DiffLine::Delete(s) if s.contains("line2")));
        assert!(has_add, "should have addition of 'modified'");
        assert!(has_del, "should have deletion of 'line2'");
    }

    #[test]
    fn test_all_additions() {
        let old = "";
        let new = "line1\nline2\nline3\n";
        let diff = compute_diff(old, new, "new_file.txt");

        assert_eq!(diff.additions, 3);
        assert_eq!(diff.deletions, 0);
        assert!(!diff.hunks.is_empty());
    }

    #[test]
    fn test_all_deletions() {
        let old = "line1\nline2\nline3\n";
        let new = "";
        let diff = compute_diff(old, new, "deleted.txt");

        assert_eq!(diff.additions, 0);
        assert_eq!(diff.deletions, 3);
        assert!(!diff.hunks.is_empty());
    }

    #[test]
    fn test_empty_diff_identical() {
        let text = "same\ncontent\nhere\n";
        let diff = compute_diff(text, text, "same.txt");

        assert_eq!(diff.additions, 0);
        assert_eq!(diff.deletions, 0);
        assert!(diff.hunks.is_empty());
    }

    #[test]
    fn test_multiple_hunks() {
        // Create enough separation so grouped_ops produces multiple groups
        let old_lines: Vec<String> = (1..=20).map(|i| format!("line{}", i)).collect();
        let mut new_lines = old_lines.clone();

        // Change line near start
        new_lines[1] = "changed_near_start".to_string();
        // Change line near end (far enough apart to create separate hunks)
        new_lines[18] = "changed_near_end".to_string();

        let old = old_lines.join("\n") + "\n";
        let new = new_lines.join("\n") + "\n";

        let diff = compute_diff(&old, &new, "multi.txt");
        assert_eq!(diff.additions, 2);
        assert_eq!(diff.deletions, 2);
        // With 3 lines of context and changes at lines 2 and 19, we should get 2 hunks
        assert!(
            diff.hunks.len() >= 2,
            "expected at least 2 hunks, got {}",
            diff.hunks.len()
        );
    }
}
