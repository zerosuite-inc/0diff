use serde::Serialize;
use std::path::PathBuf;
use std::process::Command;

#[derive(Clone, Debug)]
pub struct GitRepo {
    root: PathBuf,
}

#[derive(Clone, Debug, Serialize)]
pub struct BlameInfo {
    pub author: String,
    pub commit: String,
    pub date: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct CommitInfo {
    pub hash: String,
    pub author: String,
    pub message: String,
    pub date: String,
    pub co_authors: Vec<String>,
}

impl GitRepo {
    pub fn new(path: &str) -> Result<GitRepo, Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(["-C", path, "rev-parse", "--git-dir"])
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "Not a git repository: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )
            .into());
        }

        let canonical = std::fs::canonicalize(path)?;
        Ok(GitRepo { root: canonical })
    }

    pub fn current_branch(&self) -> Result<String, Box<dyn std::error::Error>> {
        self.run_git(&["rev-parse", "--abbrev-ref", "HEAD"])
    }

    pub fn current_author(&self) -> Result<String, Box<dyn std::error::Error>> {
        self.run_git(&["config", "user.name"])
    }

    pub fn file_contents_at_head(&self, file: &str) -> Result<String, Box<dyn std::error::Error>> {
        let arg = format!("HEAD:{}", file);
        match self.run_git(&["show", &arg]) {
            Ok(contents) => Ok(contents),
            Err(_) => {
                // File is new/untracked — return empty string
                Ok(String::new())
            }
        }
    }

    pub fn blame_line(
        &self,
        file: &str,
        line: usize,
    ) -> Result<BlameInfo, Box<dyn std::error::Error>> {
        let line_spec = format!("{},{}", line, line);
        let output = self.run_git(&["blame", "-L", &line_spec, "--porcelain", file])?;

        let mut author = String::new();
        let mut commit = String::new();
        let mut date = String::new();

        for raw_line in output.lines() {
            if commit.is_empty() && raw_line.len() >= 40 {
                // First line contains the commit hash
                commit = raw_line.split_whitespace().next().unwrap_or("").to_string();
            }
            if let Some(name) = raw_line.strip_prefix("author ") {
                author = name.to_string();
            }
            if let Some(ts) = raw_line.strip_prefix("committer-time ") {
                date = ts.to_string();
            }
        }

        Ok(BlameInfo {
            author,
            commit,
            date,
        })
    }

    pub fn recent_commits(
        &self,
        limit: usize,
    ) -> Result<Vec<CommitInfo>, Box<dyn std::error::Error>> {
        let limit_arg = format!("-{}", limit);
        let output =
            self.run_git(&["log", &limit_arg, "--format=%H%n%an%n%s%n%aI%n%b%n---END---"])?;

        let mut commits = Vec::new();

        for block in output.split("---END---") {
            let block = block.trim();
            if block.is_empty() {
                continue;
            }

            let mut lines = block.lines();
            let hash = lines.next().unwrap_or("").to_string();
            let author = lines.next().unwrap_or("").to_string();
            let message = lines.next().unwrap_or("").to_string();
            let date = lines.next().unwrap_or("").to_string();

            // Remaining lines are the body — extract Co-Authored-By
            let body: String = lines.collect::<Vec<_>>().join("\n");
            let co_authors = body
                .lines()
                .filter_map(|l| {
                    let trimmed = l.trim();
                    if let Some(rest) = trimmed.strip_prefix("Co-Authored-By:") {
                        Some(rest.trim().to_string())
                    } else if let Some(rest) = trimmed.strip_prefix("Co-authored-by:") {
                        Some(rest.trim().to_string())
                    } else {
                        None
                    }
                })
                .collect();

            if !hash.is_empty() {
                commits.push(CommitInfo {
                    hash,
                    author,
                    message,
                    date,
                    co_authors,
                });
            }
        }

        Ok(commits)
    }

    fn run_git(&self, args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.root)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("git {} failed: {}", args.join(" "), stderr.trim()).into());
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}
