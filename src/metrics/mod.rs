use std::path::Path;

use crate::git::{
    count_by_month, count_files_per_commit, count_firefighting_lines, log_bug_hotspots,
    log_commit_months, log_name_only_since, log_oneline_since, parse_shortlog, shortlog_sn,
    top_n_counts,
};
use crate::model::{MetricId, MetricResult, MetricRow};
use crate::git::GitError;

/// Default `--since` for churn / bugs / firefighting when not specified.
pub const DEFAULT_SINCE: &str = "1 year ago";

/// Default number of rows for file-based metrics.
pub const DEFAULT_TOP: usize = 20;

pub struct ScanOptions<'a> {
    pub repo: &'a Path,
    pub since: &'a str,
    pub top: usize,
}

pub fn metric_churn(opts: &ScanOptions) -> Result<MetricResult, GitError> {
    let lines = log_name_only_since(opts.repo, opts.since)?;
    let counts = count_files_per_commit(&lines);
    let top = top_n_counts(counts, opts.top);
    let rows: Vec<MetricRow> = top
        .iter()
        .map(|(k, v)| MetricRow {
            key: k.clone(),
            value: *v,
            extra: None,
        })
        .collect();
    let summary = format!("{} files in top {}", rows.len(), opts.top);
    Ok(MetricResult {
        id: MetricId::Churn,
        label: MetricId::Churn.label().to_string(),
        summary,
        rows: Some(rows),
        scalar: None,
    })
}

pub fn metric_bus_factor(opts: &ScanOptions) -> Result<MetricResult, GitError> {
    let text = shortlog_sn(opts.repo, Some(opts.since))?;
    let parsed = parse_shortlog(&text);
    let rows: Vec<MetricRow> = parsed
        .iter()
        .take(opts.top)
        .map(|(name, n)| MetricRow {
            key: name.clone(),
            value: *n,
            extra: None,
        })
        .collect();
    let total: u64 = parsed.iter().map(|(_, n)| n).sum();
    let summary = format!("{} contributors (window), {} commits", rows.len(), total);
    Ok(MetricResult {
        id: MetricId::BusFactor,
        label: MetricId::BusFactor.label().to_string(),
        summary,
        rows: Some(rows),
        scalar: Some(total),
    })
}

pub fn metric_bug_hotspots(opts: &ScanOptions) -> Result<MetricResult, GitError> {
    let lines = log_bug_hotspots(opts.repo, opts.since)?;
    let counts = count_files_per_commit(&lines);
    let top = top_n_counts(counts, opts.top);
    let rows: Vec<MetricRow> = top
        .iter()
        .map(|(k, v)| MetricRow {
            key: k.clone(),
            value: *v,
            extra: None,
        })
        .collect();
    let summary = format!("{} hotspot files (top {})", rows.len(), opts.top);
    Ok(MetricResult {
        id: MetricId::BugHotspots,
        label: MetricId::BugHotspots.label().to_string(),
        summary,
        rows: Some(rows),
        scalar: None,
    })
}

pub fn metric_delivery_pace(_opts: &ScanOptions) -> Result<MetricResult, GitError> {
    let months = log_commit_months(_opts.repo)?;
    let counts = count_by_month(&months);
    let rows: Vec<MetricRow> = counts
        .iter()
        .map(|(k, v)| MetricRow {
            key: k.clone(),
            value: *v,
            extra: None,
        })
        .collect();
    let summary = format!("{} months of history", rows.len());
    Ok(MetricResult {
        id: MetricId::DeliveryPace,
        label: MetricId::DeliveryPace.label().to_string(),
        summary,
        rows: Some(rows),
        scalar: Some(months.len() as u64),
    })
}

pub fn metric_firefighting(opts: &ScanOptions) -> Result<MetricResult, GitError> {
    let lines = log_oneline_since(opts.repo, opts.since)?;
    let n = count_firefighting_lines(&lines);
    let summary = format!(
        "{} matching commits (since {}) out of {} total oneline commits",
        n,
        opts.since,
        lines.len()
    );
    Ok(MetricResult {
        id: MetricId::Firefighting,
        label: MetricId::Firefighting.label().to_string(),
        summary,
        rows: None,
        scalar: Some(n),
    })
}

pub fn run_single(id: MetricId, opts: &ScanOptions) -> Result<MetricResult, GitError> {
    match id {
        MetricId::Churn => metric_churn(opts),
        MetricId::BusFactor => metric_bus_factor(opts),
        MetricId::BugHotspots => metric_bug_hotspots(opts),
        MetricId::DeliveryPace => metric_delivery_pace(opts),
        MetricId::Firefighting => metric_firefighting(opts),
    }
}

pub fn run_all(opts: &ScanOptions) -> Result<Vec<MetricResult>, GitError> {
    let mut out = Vec::new();
    for id in MetricId::all() {
        out.push(run_single(*id, opts)?);
    }
    Ok(out)
}
