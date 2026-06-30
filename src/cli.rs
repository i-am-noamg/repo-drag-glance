use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::model::OutputFormat;

#[derive(Parser, Debug)]
#[command(
    name = "repo-drag-glance",
    version,
    about = "Five git-log drag diagnostics on an unfamiliar codebase before you read code"
)]
pub struct Cli {
    /// Disable ANSI colors in terminal output.
    #[arg(long = "no-color", global = true)]
    pub no_color: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Run all metrics and optional alerts.
    Scan {
        #[command(flatten)]
        common: CommonOpts,
    },
    /// Run a single metric by id (churn, bus_factor, bug_hotspots, delivery_pace, firefighting).
    Metrics {
        /// Metric name or alias.
        metric: String,
        #[command(flatten)]
        common: CommonOpts,
    },
    /// Show how a metric is computed.
    Explain {
        /// Metric name or alias.
        metric: String,
    },
}

#[derive(clap::Args, Debug)]
pub struct CommonOpts {
    /// Path to the git repository.
    #[arg(long, default_value = ".")]
    pub repo: PathBuf,

    /// Source directories for file-based metrics (churn, bug_hotspots).
    /// Repeatable, e.g. `--source-dir src --source-dir apps`.
    #[arg(long = "source-dir")]
    pub source_dirs: Vec<String>,

    /// Time window for churn, firefighting, and delivery pace (`git --since`).
    #[arg(long, default_value = "1 year ago")]
    pub since: String,

    /// Recent window for bus-factor departed-contributor check.
    #[arg(long, default_value = "6 months ago")]
    pub recent_since: String,

    /// Max rows for file/author tables.
    #[arg(long, default_value_t = 20)]
    pub top: usize,

    /// Output format.
    #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
    pub format: OutputFormat,
}
