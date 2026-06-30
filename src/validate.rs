use std::path::Path;

use anyhow::{bail, Context, Result};

use crate::cli::CommonOpts;

const MAX_SINCE_LEN: usize = 256;
const MAX_SOURCE_DIR_LEN: usize = 4096;
const MAX_TOP: usize = 1_000;

/// Git-related strings and CLI bounds before any subprocess runs.
pub fn validate_common_opts(opts: &CommonOpts) -> Result<()> {
    validate_since("since", &opts.since)?;
    validate_since("recent-since", &opts.recent_since)?;
    for dir in &opts.source_dirs {
        validate_source_dir(dir)?;
    }
    if opts.top == 0 {
        bail!("--top must be at least 1");
    }
    if opts.top > MAX_TOP {
        bail!("--top must be at most {MAX_TOP}");
    }
    validate_repo_path(&opts.repo)?;
    Ok(())
}

fn validate_repo_path(repo: &Path) -> Result<()> {
    if repo.as_os_str().is_empty() {
        bail!("--repo must not be empty");
    }
    Ok(())
}

fn validate_since(name: &str, value: &str) -> Result<()> {
    let value = value.trim();
    if value.is_empty() {
        bail!("--{name} must not be empty");
    }
    if value.starts_with('-') {
        bail!("--{name} must not start with '-'");
    }
    if value.len() > MAX_SINCE_LEN {
        bail!("--{name} must be at most {MAX_SINCE_LEN} characters");
    }
    if value.contains('\0') {
        bail!("--{name} contains invalid characters");
    }
    Ok(())
}

fn validate_source_dir(value: &str) -> Result<()> {
    let value = value.trim();
    if value.is_empty() {
        bail!("--source-dir must not be empty");
    }
    if value == "--" {
        bail!("--source-dir must not be '--'");
    }
    if value.starts_with('-') {
        bail!("--source-dir must not start with '-'");
    }
    if value.contains('\0') || value.contains(':') {
        bail!(
            "--source-dir {value:?} is invalid: pathspec magic and control characters are not allowed"
        );
    }
    if value.len() > MAX_SOURCE_DIR_LEN {
        bail!("--source-dir must be at most {MAX_SOURCE_DIR_LEN} characters");
    }
    if value.starts_with('/') || value.starts_with('\\') {
        bail!("--source-dir must be a relative path inside the repository");
    }
    for component in value.split(['/', '\\']) {
        if component == ".." {
            bail!("--source-dir must not contain '..' components");
        }
    }
    Ok(())
}

/// Normalize validated `--source-dir` values for git pathspecs and Rust filtering.
pub fn normalize_source_dirs(dirs: &[String]) -> Result<Vec<String>> {
    dirs.iter()
        .map(|d| {
            validate_source_dir(d).with_context(|| format!("invalid --source-dir {d:?}"))?;
            let trimmed = d.trim().trim_end_matches(['/', '\\']).to_string();
            Ok(trimmed)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn opts_with_source_dirs(dirs: Vec<&str>) -> CommonOpts {
        CommonOpts {
            repo: PathBuf::from("."),
            source_dirs: dirs.into_iter().map(String::from).collect(),
            since: "1 year ago".into(),
            recent_since: "6 months ago".into(),
            top: 20,
            format: crate::model::OutputFormat::Table,
        }
    }

    #[test]
    fn rejects_empty_source_dir() {
        assert!(validate_source_dir("").is_err());
        assert!(validate_source_dir("   ").is_err());
    }

    #[test]
    fn rejects_pathspec_magic_and_parent_segments() {
        assert!(validate_source_dir(":(glob)src").is_err());
        assert!(validate_source_dir("../src").is_err());
        assert!(validate_source_dir("src/../lib").is_err());
        assert!(validate_source_dir("-src").is_err());
        assert!(validate_source_dir("--").is_err());
        assert!(validate_source_dir("/abs").is_err());
    }

    #[test]
    fn accepts_normal_source_dirs() {
        assert!(validate_source_dir("src").is_ok());
        assert!(validate_source_dir("apps/web").is_ok());
        let normalized = normalize_source_dirs(&["src/".into(), " apps ".into()]).unwrap();
        assert_eq!(normalized, vec!["src", "apps"]);
    }

    #[test]
    fn rejects_invalid_since_and_top() {
        let mut opts = opts_with_source_dirs(vec![]);
        opts.since = "-1 day".into();
        assert!(validate_common_opts(&opts).is_err());
        opts.since = "1 year ago".into();
        opts.top = 0;
        assert!(validate_common_opts(&opts).is_err());
        opts.top = MAX_TOP + 1;
        assert!(validate_common_opts(&opts).is_err());
    }
}
