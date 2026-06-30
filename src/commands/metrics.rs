use anyhow::{bail, Context};

use crate::alerts;
use crate::cli::CommonOpts;
use crate::git;
use crate::metrics::{self, ScanOptions};
use crate::model::MetricId;
use crate::report;
use crate::validate;

pub fn run_metrics(name: &str, common: &CommonOpts, no_color: bool) -> anyhow::Result<()> {
    let Some(id) = MetricId::parse(name) else {
        bail!("unknown metric {:?}; try: churn, bus_factor, bug_hotspots, delivery_pace, firefighting", name);
    };
    validate::validate_common_opts(common).context("invalid arguments")?;
    let source_dirs = validate::normalize_source_dirs(&common.source_dirs)?;
    git::check_has_commits(&common.repo).context("repository check")?;
    let opts = ScanOptions {
        repo: &common.repo,
        since: &common.since,
        recent_since: &common.recent_since,
        source_dirs: &source_dirs,
        top: common.top,
    };
    let m = metrics::run_single(id, &opts).context("collect metric")?;
    let repo = common
        .repo
        .canonicalize()
        .unwrap_or_else(|_| common.repo.clone())
        .display()
        .to_string();
    let report = alerts::build_report(
        repo,
        common.since.clone(),
        common.recent_since.clone(),
        source_dirs.clone(),
        vec![m],
        &opts,
    );
    report::print_report(&report, common.format, no_color).context("render report")?;
    Ok(())
}
