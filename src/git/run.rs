use std::ffi::OsStr;
use std::path::Path;
use std::process::{Command, Stdio};

use super::GitError;

/// Environment variables safe to pass through to the git subprocess.
const SAFE_ENV_KEYS: &[&str] = &[
    "PATH",
    "PATHEXT",
    "HOME",
    "USER",
    "USERPROFILE",
    "HOMEDRIVE",
    "HOMEPATH",
    "APPDATA",
    "LOCALAPPDATA",
    "SYSTEMROOT",
    "WINDIR",
    "COMSPEC",
    "LANG",
    "LC_ALL",
    "LC_CTYPE",
    "LC_MESSAGES",
    "TERM",
    "TMP",
    "TEMP",
    "TZ",
    "SSL_CERT_FILE",
    "SSL_CERT_DIR",
    "SystemDrive",
    "SystemRoot",
];

/// Run `git` with `-C repo` and given args; returns stdout as String.
///
/// Stdin is always closed: several git commands (notably `shortlog` with no
/// revision) read commits from stdin when none are given; `Command::output()`
/// would otherwise attach a null stdin and those commands would see an empty
/// history.
///
/// The subprocess environment is scrubbed: `GIT_*` and other injection-prone
/// variables are not inherited. Set `REPO_DRAG_GLANCE_GIT` to override the git
/// binary path (must not contain newlines).
pub fn git_stdout(repo: &Path, args: &[&str]) -> Result<String, GitError> {
    if !repo.exists() {
        return Err(GitError::RepoNotFound(repo.to_path_buf()));
    }
    let git = git_binary()?;
    let out = configure_command(&git, repo, args).output()?;
    if out.status.success() {
        return Ok(String::from_utf8(out.stdout)?);
    }
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
    Err(classify_git_failure(
        repo,
        out.status.code().unwrap_or(-1),
        &stderr,
    ))
}

fn git_binary() -> Result<String, GitError> {
    let git = std::env::var("REPO_DRAG_GLANCE_GIT").unwrap_or_else(|_| "git".to_string());
    validate_git_binary(&git)
}

fn validate_git_binary(git: &str) -> Result<String, GitError> {
    if git.is_empty() || git.contains('\n') || git.contains('\0') {
        return Err(GitError::InvalidGitBinary);
    }
    Ok(git.to_string())
}

fn configure_command(git: &str, repo: &Path, args: &[&str]) -> Command {
    let mut cmd = Command::new(git);
    cmd.stdin(Stdio::null())
        .arg("-C")
        .arg(repo.as_os_str())
        .args(args);
    cmd.env_clear();
    for (key, value) in std::env::vars_os() {
        if is_safe_env_key(&key) {
            cmd.env(key, value);
        }
    }
    cmd
}

fn is_safe_env_key(key: &OsStr) -> bool {
    let key = key.to_string_lossy();
    if key.starts_with("REPO_DRAG_GLANCE_") {
        return false;
    }
    if key.starts_with("GIT_") {
        return false;
    }
    if matches!(
        key.as_ref(),
        "LD_PRELOAD"
            | "LD_LIBRARY_PATH"
            | "LD_AUDIT"
            | "DYLD_INSERT_LIBRARIES"
            | "DYLD_LIBRARY_PATH"
            | "DYLD_FRAMEWORK_PATH"
    ) {
        return false;
    }
    SAFE_ENV_KEYS.iter().any(|safe| key == *safe)
}

fn classify_git_failure(repo: &Path, status: i32, stderr: &str) -> GitError {
    let lower = stderr.to_ascii_lowercase();
    if lower.contains("not a git repository") {
        return GitError::NotGitRepository(repo.to_path_buf());
    }
    GitError::Failed {
        status,
        stderr: stderr.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_git_and_injection_env_keys() {
        assert!(!is_safe_env_key(OsStr::new("GIT_DIR")));
        assert!(!is_safe_env_key(OsStr::new("GIT_WORK_TREE")));
        assert!(!is_safe_env_key(OsStr::new("LD_PRELOAD")));
        assert!(!is_safe_env_key(OsStr::new("REPO_DRAG_GLANCE_VERBOSE")));
        assert!(is_safe_env_key(OsStr::new("PATH")));
        assert!(is_safe_env_key(OsStr::new("HOME")));
    }

    #[test]
    fn rejects_invalid_git_binary_override() {
        assert!(matches!(
            validate_git_binary("git\n"),
            Err(GitError::InvalidGitBinary)
        ));
        assert!(validate_git_binary("git").is_ok());
    }
}
