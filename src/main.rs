use clap::Parser;

use vprdashboard::cli::{Cli, Command};
use vprdashboard::commands;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Scan { common } => commands::run_scan(&common, cli.no_color),
        Command::Metrics { metric, common } => {
            commands::run_metrics(&metric, &common, cli.no_color)
        }
        Command::Explain { metric } => commands::run_explain(&metric, cli.no_color),
    }
}
