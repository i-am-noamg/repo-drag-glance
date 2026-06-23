use crate::model::{AlertHint, AlertSeverity, MetricId, MetricResult, ScanReport};

const OVERLAP_TOP_FILES: usize = 5;
const BUS_FACTOR_DOMINANCE: f64 = 0.60;
const FIREFIGHTING_WARN_PER_YEAR: u64 = 8;

/// Derive simple leadership hints from computed metrics.
pub fn compute_alerts(metrics: &[MetricResult]) -> Vec<AlertHint> {
    let mut alerts = Vec::new();

    let churn_files = file_keys(metrics, MetricId::Churn, OVERLAP_TOP_FILES);
    let bug_files = file_keys(metrics, MetricId::BugHotspots, OVERLAP_TOP_FILES);
    if !churn_files.is_empty() && !bug_files.is_empty() {
        let overlap: Vec<_> = churn_files
            .iter()
            .filter(|f| bug_files.contains(*f))
            .cloned()
            .collect();
        if !overlap.is_empty() {
            alerts.push(AlertHint {
                severity: AlertSeverity::High,
                code: "churn_and_bug_overlap".to_string(),
                message: "Files appear in both high churn and bug hotspots.".to_string(),
                evidence: Some(overlap.join(", ")),
            });
        }
    }

    if let Some((top_name, top_n, total)) = top_author_share(metrics, MetricId::BusFactor) {
        if total > 0 {
            let ratio = top_n as f64 / total as f64;
            if ratio >= BUS_FACTOR_DOMINANCE {
                alerts.push(AlertHint {
                    severity: AlertSeverity::Warning,
                    code: "bus_factor_dominance".to_string(),
                    message: format!(
                        "Top contributor authored {:.0}% of commits in the window.",
                        ratio * 100.0
                    ),
                    evidence: Some(format!("{top_name} ({top_n}/{total})")),
                });
            }
        }
    }

    if let Some(n) = scalar(metrics, MetricId::Firefighting) {
        // Rough annualized bar: if window is ~1 year, n > threshold warns.
        if n >= FIREFIGHTING_WARN_PER_YEAR {
            alerts.push(AlertHint {
                severity: AlertSeverity::Warning,
                code: "firefighting_frequency".to_string(),
                message: format!(
                    "Many revert/hotfix-style commits in window (>= {FIREFIGHTING_WARN_PER_YEAR})."
                ),
                evidence: Some(format!("count={n}")),
            });
        }
    }

    if let Some(msg) = delivery_drop_hint(metrics) {
        alerts.push(AlertHint {
            severity: AlertSeverity::Warning,
            code: "delivery_pace_drop".to_string(),
            message: msg,
            evidence: None,
        });
    }

    alerts
}

pub fn build_report(repo: String, since: String, metrics: Vec<MetricResult>) -> ScanReport {
    let alerts = compute_alerts(&metrics);
    ScanReport {
        repo,
        since,
        metrics,
        alerts,
    }
}

fn file_keys(metrics: &[MetricResult], id: MetricId, top: usize) -> std::collections::HashSet<String> {
    let mut set = std::collections::HashSet::new();
    let Some(m) = metrics.iter().find(|m| m.id == id) else {
        return set;
    };
    let Some(rows) = &m.rows else {
        return set;
    };
    for r in rows.iter().take(top) {
        set.insert(r.key.clone());
    }
    set
}

fn scalar(metrics: &[MetricResult], id: MetricId) -> Option<u64> {
    metrics.iter().find(|m| m.id == id)?.scalar
}

/// Returns (author_name, top_commits, total_commits) for bus factor metric rows.
fn top_author_share(metrics: &[MetricResult], id: MetricId) -> Option<(String, u64, u64)> {
    let m = metrics.iter().find(|m| m.id == id)?;
    let rows = m.rows.as_ref()?;
    let top = rows.first()?;
    let total: u64 = rows.iter().map(|r| r.value).sum();
    Some((top.key.clone(), top.value, total))
}

fn delivery_drop_hint(metrics: &[MetricResult]) -> Option<String> {
    let m = metrics.iter().find(|m| m.id == MetricId::DeliveryPace)?;
    let rows = m.rows.as_ref()?;
    if rows.len() < 4 {
        return None;
    }
    let last = rows.last()?.value;
    let prev: u64 = rows.iter().rev().skip(1).take(3).map(|r| r.value).sum();
    let prev_avg = prev as f64 / 3.0;
    if prev_avg < 1.0 {
        return None;
    }
    if (last as f64) < prev_avg * 0.5 {
        Some(format!(
            "Last month commits ({last}) are below half the trailing 3-month average ({prev_avg:.1})."
        ))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::MetricRow;

    #[test]
    fn overlap_alert() {
        let metrics = vec![
            MetricResult {
                id: MetricId::Churn,
                label: "".into(),
                summary: "".into(),
                rows: Some(vec![MetricRow {
                    key: "a.rs".into(),
                    value: 10,
                    extra: None,
                }]),
                scalar: None,
            },
            MetricResult {
                id: MetricId::BugHotspots,
                label: "".into(),
                summary: "".into(),
                rows: Some(vec![MetricRow {
                    key: "a.rs".into(),
                    value: 2,
                    extra: None,
                }]),
                scalar: None,
            },
        ];
        let a = compute_alerts(&metrics);
        assert!(a.iter().any(|x| x.code == "churn_and_bug_overlap"));
    }
}
