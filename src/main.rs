use clap::Parser;

use repo_drag_glance::cli::{Cli, Command};
use repo_drag_glance::commands;
use repo_drag_glance::git::GitError;

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err:#}");
        if verbose_diagnostics() {
            print_verbose_details(&err);
        }
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Scan { common } => commands::run_scan(&common, cli.no_color),
        Command::Metrics { metric, common } => {
            commands::run_metrics(&metric, &common, cli.no_color)
        }
        Command::Explain { metric } => commands::run_explain(&metric, cli.no_color),
    }
}

fn verbose_diagnostics() -> bool {
    matches!(
        std::env::var("REPO_DRAG_GLANCE_VERBOSE").as_deref(),
        Ok("1") | Ok("true") | Ok("yes")
    )
}

fn print_verbose_details(err: &anyhow::Error) {
    for cause in err.chain().skip(1) {
        eprintln!("  caused by: {cause}");
    }
    for cause in err.chain() {
        if let Some(git_err) = cause.downcast_ref::<GitError>() {
            if let Some(stderr) = git_err.git_stderr() {
                eprintln!("  git stderr: {stderr}");
            }
        }
    }
}
