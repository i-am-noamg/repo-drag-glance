use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitError {
    #[error("failed to run git: {0}")]
    Io(#[from] std::io::Error),

    #[error("git exited with status {status}: {stderr}")]
    Failed { status: i32, stderr: String },

    #[error("git produced invalid UTF-8")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("repository path does not exist: {0}")]
    RepoNotFound(PathBuf),

    #[error(
        "repository has no commits yet (empty history). Make an initial commit or pass --repo pointing at a clone with history."
    )]
    NoCommits,
}
