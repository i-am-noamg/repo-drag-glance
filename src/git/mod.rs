mod error;
mod run;

pub use error::GitError;
pub use run::git_stdout;

use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Fails when the path is missing, not a git repo, or has no commits yet.
pub fn check_has_commits(repo: &Path) -> Result<(), GitError> {
    if !repo.exists() {
        return Err(GitError::RepoNotFound(repo.to_path_buf()));
    }
    let out = git_stdout(repo, &["rev-list", "--all", "--max-count=1"])?;
    if out.trim().is_empty() {
        return Err(GitError::NoCommits);
    }
    Ok(())
}

/// Lines of `git log --oneline --since=...`
pub fn log_oneline_since(repo: &Path, since: &str) -> Result<Vec<String>, GitError> {
    let out = git_stdout(
        repo,
        &["log", "--oneline", "--since", since],
    )?;
    Ok(out
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect())
}

/// Commit hash lines + file paths for churn-style logs.
pub fn log_name_only_since(repo: &Path, since: &str) -> Result<Vec<String>, GitError> {
    let out = git_stdout(
        repo,
        &[
            "log",
            "--since",
            since,
            "--name-only",
            "--format=%H",
        ],
    )?;
    Ok(out.lines().map(str::trim).map(String::from).collect())
}

/// Same as churn but with grep on message.
pub fn log_bug_hotspots(repo: &Path, since: &str) -> Result<Vec<String>, GitError> {
    let out = git_stdout(
        repo,
        &[
            "log",
            "-i",
            "-E",
            "--grep=fix|bug|broken",
            "--since",
            since,
            "--name-only",
            "--format=%H",
        ],
    )?;
    Ok(out.lines().map(str::trim).map(String::from).collect())
}

/// One line per commit: `%ad` with month format.
pub fn log_commit_months(repo: &Path) -> Result<Vec<String>, GitError> {
    let out = git_stdout(
        repo,
        &[
            "log",
            "--format=%ad",
            "--date=format:%Y-%m",
        ],
    )?;
    Ok(out
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect())
}

/// `git shortlog -sn --no-merges` for **reachable commits from `HEAD`** (current
/// checkout), with optional `--since`.
///
/// A revision is always passed (`HEAD`). With no revision, `git shortlog`
/// reads from stdin; our subprocess uses a closed stdin (see `git_stdout`),
/// which would yield an empty result.
///
/// We intentionally do **not** use `--all`: that walks every ref under
/// `refs/`, which double-counts shared history and pulls in stale branches.
pub fn shortlog_sn(repo: &Path, since: Option<&str>) -> Result<String, GitError> {
    match since {
        None => git_stdout(repo, &["shortlog", "-sn", "--no-merges", "HEAD"]),
        Some(s) => git_stdout(
            repo,
            &["shortlog", "-sn", "--no-merges", "--since", s, "HEAD"],
        ),
    }
}

/// Parse `git shortlog` lines: "   123\tName"
pub fn parse_shortlog(text: &str) -> Vec<(String, u64)> {
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim_start();
        let Some((num, name)) = line.split_once('\t') else {
            continue;
        };
        if let Ok(n) = num.trim().parse::<u64>() {
            out.push((name.trim().to_string(), n));
        }
    }
    out
}

/// True if line looks like a full or abbreviated git object id (no path slashes).
pub fn looks_like_commit_hash(line: &str) -> bool {
    let line = line.trim();
    if line.is_empty() || line.contains('/') || line.contains('\\') {
        return false;
    }
    let bytes = line.as_bytes();
    if bytes.len() < 7 || bytes.len() > 40 {
        return false;
    }
    bytes.iter().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F'))
}

/// Count file touches per path: one increment per file per commit (unique files per commit).
pub fn count_files_per_commit(lines: &[String]) -> HashMap<String, u64> {
    let mut counts: HashMap<String, u64> = HashMap::new();
    let mut current: HashSet<String> = HashSet::new();
    for line in lines {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        if looks_like_commit_hash(t) {
            for f in &current {
                *counts.entry(f.clone()).or_insert(0) += 1;
            }
            current.clear();
            continue;
        }
        current.insert(t.to_string());
    }
    for f in current {
        *counts.entry(f).or_insert(0) += 1;
    }
    counts
}

/// Sort map by count descending, take top n.
pub fn top_n_counts(map: HashMap<String, u64>, n: usize) -> Vec<(String, u64)> {
    let mut v: Vec<(String, u64)> = map.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    v.truncate(n);
    v
}

/// Count identical month strings.
pub fn count_by_month(months: &[String]) -> Vec<(String, u64)> {
    let mut m: HashMap<String, u64> = HashMap::new();
    for month in months {
        *m.entry(month.clone()).or_insert(0) += 1;
    }
    let mut v: Vec<(String, u64)> = m.into_iter().collect();
    v.sort_by(|a, b| a.0.cmp(&b.0));
    v
}

const FIRE_KEYWORDS: &[&str] = &["revert", "hotfix", "emergency", "rollback"];

/// Count oneline subjects that match firefighting keywords (case insensitive).
pub fn count_firefighting_lines(onelines: &[String]) -> u64 {
    onelines
        .iter()
        .filter(|line| {
            let lower = line.to_ascii_lowercase();
            FIRE_KEYWORDS.iter().any(|k| lower.contains(k))
        })
        .count() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_detection() {
        assert!(looks_like_commit_hash("a1b2c3d"));
        assert!(looks_like_commit_hash("a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b"));
        assert!(!looks_like_commit_hash("src/main.rs"));
        assert!(!looks_like_commit_hash("nothex"));
    }

    #[test]
    fn count_files_per_commit_basic() {
        let lines = vec![
            "abc1234".to_string(),
            "a.rs".to_string(),
            "b.rs".to_string(),
            "def7890".to_string(),
            "a.rs".to_string(),
        ];
        let m = count_files_per_commit(&lines);
        assert_eq!(m.get("a.rs"), Some(&2));
        assert_eq!(m.get("b.rs"), Some(&1));
    }

    #[test]
    fn parse_shortlog_line() {
        let text = "   10\tAlice\n    2\tBob\n";
        let v = parse_shortlog(text);
        assert_eq!(v, vec![("Alice".to_string(), 10), ("Bob".to_string(), 2)]);
    }

    #[test]
    fn firefighting_filter() {
        let lines = vec![
            "abc Revert bad change".to_string(),
            "def docs: update".to_string(),
            "ghi HOTFIX patch".to_string(),
        ];
        assert_eq!(count_firefighting_lines(&lines), 2);
    }
}
