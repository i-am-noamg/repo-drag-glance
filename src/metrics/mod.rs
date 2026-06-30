use std::path::Path;

use crate::git::GitError;
use crate::git::{
    count_by_month, count_firefighting_lines, count_path_lines, log_bug_hotspots,
    log_commit_months, log_name_only_since, log_oneline_since, parse_shortlog, shortlog_sn_all,
    shortlog_sn_since, top_n_counts,
};
use crate::model::{MetricId, MetricResult, MetricRow};

/// Default `--since` for churn / firefighting when not specified.
pub const DEFAULT_SINCE: &str = "1 year ago";

/// Default recent window for bus-factor secondary shortlog.
pub const DEFAULT_RECENT_SINCE: &str = "6 months ago";

/// Default number of rows for file-based metrics.
pub const DEFAULT_TOP: usize = 20;

pub struct ScanOptions<'a> {
    pub repo: &'a Path,
    pub since: &'a str,
    pub recent_since: &'a str,
    pub source_dirs: &'a [String],
    pub top: usize,
}

fn pathspec_refs(source_dirs: &[String]) -> Vec<String> {
    source_dirs.to_vec()
}

pub fn metric_churn(opts: &ScanOptions) -> Result<MetricResult, GitError> {
    let pathspecs = pathspec_refs(opts.source_dirs);
    let specs: Vec<&str> = pathspecs.iter().map(String::as_str).collect();
    let lines = log_name_only_since(opts.repo, opts.since, &specs)?;
    let counts = count_path_lines(&lines, opts.source_dirs);
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
    let text = shortlog_sn_all(opts.repo)?;
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
    let contributor_count = parsed.len();
    let summary = format!(
        "{contributor_count} contributors (showing top {}), {total} commits (full history on HEAD)",
        rows.len()
    );
    Ok(MetricResult {
        id: MetricId::BusFactor,
        label: MetricId::BusFactor.label().to_string(),
        summary,
        rows: Some(rows),
        scalar: Some(total),
    })
}

/// Recent-window shortlog for bus-factor alerts (not a standalone metric).
pub fn bus_factor_recent_authors(opts: &ScanOptions) -> Result<Vec<(String, u64)>, GitError> {
    let text = shortlog_sn_since(opts.repo, opts.recent_since)?;
    Ok(parse_shortlog(&text))
}

pub fn metric_bug_hotspots(opts: &ScanOptions) -> Result<MetricResult, GitError> {
    let pathspecs = pathspec_refs(opts.source_dirs);
    let specs: Vec<&str> = pathspecs.iter().map(String::as_str).collect();
    let lines = log_bug_hotspots(opts.repo, &specs)?;
    let counts = count_path_lines(&lines, opts.source_dirs);
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

pub fn metric_delivery_pace(opts: &ScanOptions) -> Result<MetricResult, GitError> {
    let months = log_commit_months(opts.repo, opts.since)?;
    let counts = count_by_month(&months);
    let rows: Vec<MetricRow> = counts
        .iter()
        .map(|(k, v)| MetricRow {
            key: k.clone(),
            value: *v,
            extra: None,
        })
        .collect();
    let summary = format!(
        "{} months with commits since {}",
        rows.len(),
        opts.since
    );
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
