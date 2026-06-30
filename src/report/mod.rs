mod style;

use anyhow::Context;

use crate::model::{MetricId, MetricResult, OutputFormat, ScanReport};
use crate::sanitize;

pub use style::Style;

const SOURCE_DIR_WARNING: &str = "No --source-dir set; file metrics include the whole repo. The blog runs churn/bug commands from src/ or app/ to avoid lockfiles and generated files.";

pub fn source_dir_warnings(metrics: &[MetricResult], source_dirs: &[String]) -> Vec<String> {
    if source_dirs.is_empty()
        && metrics
            .iter()
            .any(|m| m.id == MetricId::Churn || m.id == MetricId::BugHotspots)
    {
        vec![format!("Warning: {SOURCE_DIR_WARNING}")]
    } else {
        vec![]
    }
}

pub fn render(report: &ScanReport, format: OutputFormat, no_color: bool) -> anyhow::Result<String> {
    let report = sanitize_report(report);
    match format {
        OutputFormat::Json => serde_json::to_string_pretty(&report).context("serialize JSON"),
        OutputFormat::Table => Ok(render_table(&report, Style::new(no_color))),
    }
}

fn sanitize_report(report: &ScanReport) -> ScanReport {
    ScanReport {
        warnings: report
            .warnings
            .iter()
            .map(|w| sanitize::display_text(w))
            .collect(),
        repo: sanitize::display_text(&report.repo),
        since: sanitize::display_text(&report.since),
        recent_since: sanitize::display_text(&report.recent_since),
        source_dirs: report
            .source_dirs
            .iter()
            .map(|d| sanitize::display_text(d))
            .collect(),
        metrics: report.metrics.iter().map(sanitize_metric).collect(),
        alerts: report.alerts.iter().map(sanitize_alert).collect(),
    }
}

fn sanitize_metric(metric: &MetricResult) -> MetricResult {
    MetricResult {
        id: metric.id,
        label: sanitize::display_text(&metric.label),
        summary: sanitize::display_text(&metric.summary),
        rows: metric.rows.as_ref().map(|rows| {
            rows.iter()
                .map(|row| crate::model::MetricRow {
                    key: sanitize::display_text(&row.key),
                    value: row.value,
                    extra: row.extra.as_ref().map(|e| sanitize::display_text(e)),
                })
                .collect()
        }),
        scalar: metric.scalar,
    }
}

fn sanitize_alert(alert: &crate::model::AlertHint) -> crate::model::AlertHint {
    crate::model::AlertHint {
        severity: alert.severity,
        code: sanitize::display_text(&alert.code),
        message: sanitize::display_text(&alert.message),
        evidence: alert.evidence.as_ref().map(|e| sanitize::display_text(e)),
    }
}

pub fn print_report(
    report: &ScanReport,
    format: OutputFormat,
    no_color: bool,
) -> anyhow::Result<()> {
    let s = render(report, format, no_color)?;
    println!("{s}");
    Ok(())
}

fn render_table(report: &ScanReport, style: Style) -> String {
    use std::fmt::Write;
    let mut buf = String::new();
    for w in &report.warnings {
        writeln!(&mut buf, "{}", style.warning(w)).unwrap();
    }
    if !report.warnings.is_empty() {
        writeln!(&mut buf).unwrap();
    }
    writeln!(&mut buf, "{} {}", style.header_label("Repo:"), report.repo).unwrap();
    writeln!(
        &mut buf,
        "{} {}",
        style.header_label("Since (churn/firefighting/delivery pace):"),
        report.since
    )
    .unwrap();
    writeln!(
        &mut buf,
        "{} {}",
        style.header_label("Recent since (bus factor):"),
        report.recent_since
    )
    .unwrap();
    if report.source_dirs.is_empty() {
        writeln!(
            &mut buf,
            "{} (none — whole repo for file metrics)",
            style.header_label("Source dirs:")
        )
        .unwrap();
    } else {
        writeln!(
            &mut buf,
            "{} {}",
            style.header_label("Source dirs:"),
            report.source_dirs.join(", ")
        )
        .unwrap();
    }
    writeln!(&mut buf).unwrap();
    for m in &report.metrics {
        writeln!(&mut buf, "{}", style.section(&format!("== {} ==", m.label))).unwrap();
        writeln!(&mut buf, "{}", style.summary(&m.summary)).unwrap();
        if let Some(rows) = &m.rows {
            if !rows.is_empty() {
                let (key_col, value_col) = m.id.row_columns();
                let mut builder = tabled::builder::Builder::default();
                builder.push_record([key_col, value_col]);
                for r in rows {
                    builder.push_record([&r.key, &r.value.to_string()]);
                }
                let mut table = builder.build();
                table.with(tabled::settings::Style::rounded());
                writeln!(&mut buf, "{table}").unwrap();
            }
        } else if let Some(s) = m.scalar {
            writeln!(
                &mut buf,
                "{} {}",
                style.scalar_label("value:"),
                style.scalar_value(&s.to_string())
            )
            .unwrap();
        }
        writeln!(&mut buf).unwrap();
    }
    if !report.alerts.is_empty() {
        writeln!(&mut buf, "{}", style.section("== Alerts ==")).unwrap();
        for a in &report.alerts {
            writeln!(
                &mut buf,
                "[{}] {} — {}",
                style.alert_severity(a.severity),
                style.alert_code(&a.code),
                a.message
            )
            .unwrap();
            if let Some(e) = &a.evidence {
                writeln!(&mut buf, "  {} {}", style.evidence_label("evidence:"), e).unwrap();
            }
        }
    }
    buf
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AlertHint, AlertSeverity, MetricId, MetricResult, MetricRow};

    fn sample_report() -> ScanReport {
        ScanReport {
            warnings: vec!["Warning: no source dir".into()],
            repo: "/tmp".into(),
            since: "1 year ago".into(),
            recent_since: "6 months ago".into(),
            source_dirs: vec!["src".into()],
            metrics: vec![
                MetricResult {
                    id: MetricId::Churn,
                    label: "High-churn files".into(),
                    summary: "1 files in top 20".into(),
                    rows: Some(vec![MetricRow {
                        key: "src/lib.rs".into(),
                        value: 4,
                        extra: None,
                    }]),
                    scalar: None,
                },
                MetricResult {
                    id: MetricId::Firefighting,
                    label: "Firefighting".into(),
                    summary: "3 matching commits".into(),
                    rows: None,
                    scalar: Some(3),
                },
            ],
            alerts: vec![AlertHint {
                severity: AlertSeverity::Warning,
                code: "test".into(),
                message: "ok".into(),
                evidence: Some("count=3".into()),
            }],
        }
    }

    #[test]
    fn json_roundtrip_shape() {
        let report = sample_report();
        let s = render(&report, OutputFormat::Json, false).unwrap();
        assert!(s.contains("firefighting"));
        assert!(s.contains("alerts"));
        assert!(!s.contains('\x1b'));
    }

    #[test]
    fn table_uses_metric_column_names() {
        let report = sample_report();
        let s = render(&report, OutputFormat::Table, true).unwrap();
        assert!(s.contains("file"));
        assert!(s.contains("changes"));
        assert!(!s.contains("String"));
        assert!(!s.contains("u64"));
    }

    #[test]
    fn json_rows_use_metric_column_names() {
        let report = sample_report();
        let s = render(&report, OutputFormat::Json, false).unwrap();
        assert!(s.contains("\"file\": \"src/lib.rs\""));
        assert!(s.contains("\"changes\": 4"));
        assert!(!s.contains("\"key\""));
    }

    #[test]
    fn table_no_color_has_no_ansi() {
        let report = sample_report();
        let s = render(&report, OutputFormat::Table, true).unwrap();
        assert!(s.contains("Firefighting"));
        assert!(s.contains("== Alerts =="));
        assert!(!s.contains('\x1b'));
    }
}
