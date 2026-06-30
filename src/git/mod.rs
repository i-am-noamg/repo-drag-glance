mod error;
mod run;

pub use error::GitError;
pub use run::git_stdout;

use std::collections::HashMap;
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
    let out = git_stdout(repo, &["log", "--oneline", "--since", since])?;
    Ok(out
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect())
}

/// Churn-style log: `git log --format=format: --name-only --since=... [-- pathspec...]`
pub fn log_name_only_since(
    repo: &Path,
    since: &str,
    pathspecs: &[&str],
) -> Result<Vec<String>, GitError> {
    let mut args = vec![
        "log".to_string(),
        "--format=format:".to_string(),
        "--name-only".to_string(),
        "--since".to_string(),
        since.to_string(),
    ];
    append_pathspecs(&mut args, pathspecs);
    git_stdout_lines(repo, &args)
}

/// Bug hotspot log: `git log -i -E --grep=... --name-only --format= [-- pathspec...]`
pub fn log_bug_hotspots(repo: &Path, pathspecs: &[&str]) -> Result<Vec<String>, GitError> {
    let mut args = vec![
        "log".to_string(),
        "-i".to_string(),
        "-E".to_string(),
        "--grep=fix|bug|broken".to_string(),
        "--name-only".to_string(),
        "--format=".to_string(),
    ];
    append_pathspecs(&mut args, pathspecs);
    git_stdout_lines(repo, &args)
}

/// One line per commit: `%ad` with month format, scoped by `--since`.
pub fn log_commit_months(repo: &Path, since: &str) -> Result<Vec<String>, GitError> {
    let out = git_stdout(
        repo,
        &[
            "log",
            "--format=%ad",
            "--date=format:%Y-%m",
            "--since",
            since,
        ],
    )?;
    Ok(out
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect())
}

/// Full-history shortlog on `HEAD` (blog: `git shortlog -sn --no-merges`).
pub fn shortlog_sn_all(repo: &Path) -> Result<String, GitError> {
    git_stdout(repo, &["shortlog", "-sn", "--no-merges", "HEAD"])
}

/// Recent-window shortlog on `HEAD` (blog: `--since="6 months ago"`).
pub fn shortlog_sn_since(repo: &Path, since: &str) -> Result<String, GitError> {
    git_stdout(
        repo,
        &["shortlog", "-sn", "--no-merges", "--since", since, "HEAD"],
    )
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

/// Normalize git path strings to forward slashes for consistent matching on Windows.
fn normalize_git_path(path: &str) -> String {
    path.replace('\\', "/")
}

/// True when `path` is under any listed source dir prefix (or when no dirs given).
pub fn path_matches_source_dirs(path: &str, source_dirs: &[String]) -> bool {
    if source_dirs.is_empty() {
        return true;
    }
    let path = normalize_git_path(path);
    for dir in source_dirs {
        let prefix = normalize_git_path(dir.trim_end_matches(['/', '\\']));
        if path == prefix || path.starts_with(&format!("{prefix}/")) {
            return true;
        }
    }
    false
}

/// Count non-empty path lines (blog: `sort | uniq -c`), optionally filtered by source dirs.
pub fn count_path_lines(lines: &[String], source_dirs: &[String]) -> HashMap<String, u64> {
    let mut counts: HashMap<String, u64> = HashMap::new();
    for line in lines {
        let t = normalize_git_path(line.trim());
        if t.is_empty() {
            continue;
        }
        if !path_matches_source_dirs(&t, source_dirs) {
            continue;
        }
        *counts.entry(t).or_insert(0) += 1;
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

fn append_pathspecs(args: &mut Vec<String>, pathspecs: &[&str]) {
    if pathspecs.is_empty() {
        return;
    }
    args.push("--".to_string());
    args.extend(pathspecs.iter().map(|s| (*s).to_string()));
}

fn git_stdout_lines(repo: &Path, args: &[String]) -> Result<Vec<String>, GitError> {
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let out = git_stdout(repo, &refs)?;
    Ok(out.lines().map(str::trim).map(String::from).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_path_lines_basic() {
        let lines = vec![
            "src/a.rs".to_string(),
            "src/a.rs".to_string(),
            "README.md".to_string(),
        ];
        let m = count_path_lines(&lines, &[]);
        assert_eq!(m.get("src/a.rs"), Some(&2));
        assert_eq!(m.get("README.md"), Some(&1));
    }

    #[test]
    fn count_path_lines_source_filter() {
        let lines = vec![
            "src/a.rs".to_string(),
            "Cargo.lock".to_string(),
            "apps/foo.ts".to_string(),
        ];
        let dirs = vec!["src".to_string(), "apps".to_string()];
        let m = count_path_lines(&lines, &dirs);
        assert_eq!(m.get("src/a.rs"), Some(&1));
        assert_eq!(m.get("apps/foo.ts"), Some(&1));
        assert!(!m.contains_key("Cargo.lock"));
    }

    #[test]
    fn path_matches_source_dirs_filter() {
        let dirs = vec!["src".to_string()];
        assert!(path_matches_source_dirs("src/lib.rs", &dirs));
        assert!(path_matches_source_dirs("src", &dirs));
        assert!(path_matches_source_dirs(r"src\lib.rs", &dirs));
        assert!(!path_matches_source_dirs("Cargo.lock", &dirs));
    }

    #[test]
    fn count_path_lines_normalizes_backslashes() {
        let lines = vec![r"src\a.rs".to_string(), "src/a.rs".to_string()];
        let dirs = vec!["src".to_string()];
        let m = count_path_lines(&lines, &dirs);
        assert_eq!(m.get("src/a.rs"), Some(&2));
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
