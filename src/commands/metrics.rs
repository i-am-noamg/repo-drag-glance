use anyhow::{bail, Context};

use crate::alerts;
use crate::cli::CommonOpts;
use crate::git;
use crate::metrics::{self, ScanOptions};
use crate::model::MetricId;
use crate::report;

pub fn run_metrics(name: &str, common: &CommonOpts) -> anyhow::Result<()> {
    let Some(id) = MetricId::parse(name) else {
        bail!("unknown metric {:?}; try: churn, bus_factor, bug_hotspots, delivery_pace, firefighting", name);
    };
    git::check_has_commits(&common.repo).context("repository check")?;
    let opts = ScanOptions {
        repo: &common.repo,
        since: &common.since,
        top: common.top,
    };
    let m = metrics::run_single(id, &opts).context("collect metric")?;
    let repo = common
        .repo
        .canonicalize()
        .unwrap_or_else(|_| common.repo.clone())
        .display()
        .to_string();
    let report = alerts::build_report(repo, common.since.clone(), vec![m]);
    report::print_report(&report, common.format).context("render report")?;
    Ok(())
}
