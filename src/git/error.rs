use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitError {
    #[error("failed to run git: {0}")]
    Io(#[from] std::io::Error),

    #[error("git command failed with exit code {status}")]
    Failed { status: i32, stderr: String },

    #[error("git produced invalid UTF-8")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("repository path does not exist: {0}")]
    RepoNotFound(PathBuf),

    #[error("path is not a git repository: {0}")]
    NotGitRepository(PathBuf),

    #[error("REPO_DRAG_GLANCE_GIT must be a single-line path to the git executable")]
    InvalidGitBinary,

    #[error(
        "repository has no commits yet (empty history). Make an initial commit or pass --repo pointing at a clone with history."
    )]
    NoCommits,
}

impl GitError {
    /// Raw git stderr when available. Shown only when verbose diagnostics are enabled.
    pub fn git_stderr(&self) -> Option<&str> {
        match self {
            Self::Failed { stderr, .. } => Some(stderr),
            _ => None,
        }
    }
}
