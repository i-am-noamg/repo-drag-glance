use std::path::Path;
use std::process::{Command, Stdio};

use super::GitError;

/// Run `git` with `-C repo` and given args; returns stdout as String.
///
/// Stdin is always closed: several git commands (notably `shortlog` with no
/// revision) read commits from stdin when none are given; `Command::output()`
/// would otherwise attach a null stdin and those commands would see an empty
/// history.
pub fn git_stdout(repo: &Path, args: &[&str]) -> Result<String, GitError> {
    if !repo.exists() {
        return Err(GitError::RepoNotFound(repo.to_path_buf()));
    }
    let out = Command::new("git")
        .stdin(Stdio::null())
        .arg("-C")
        .arg(repo.as_os_str())
        .args(args)
        .output()?;
    if out.status.success() {
        return Ok(String::from_utf8(out.stdout)?);
    }
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
    Err(GitError::Failed {
        status: out.status.code().unwrap_or(-1),
        stderr,
    })
}
