use anyhow::Context;

use crate::model::{OutputFormat, ScanReport};

pub fn render(report: &ScanReport, format: OutputFormat) -> anyhow::Result<String> {
    match format {
        OutputFormat::Json => serde_json::to_string_pretty(report).context("serialize JSON"),
        OutputFormat::Table => Ok(render_table(report)),
    }
}

pub fn print_report(report: &ScanReport, format: OutputFormat) -> anyhow::Result<()> {
    let s = render(report, format)?;
    println!("{s}");
    Ok(())
}

fn render_table(report: &ScanReport) -> String {
    use std::fmt::Write;
    let mut buf = String::new();
    writeln!(&mut buf, "Repo: {}", report.repo).unwrap();
    writeln!(&mut buf, "Since: {}", report.since).unwrap();
    writeln!(&mut buf).unwrap();
    for m in &report.metrics {
        writeln!(&mut buf, "== {} ==", m.label).unwrap();
        writeln!(&mut buf, "{}", m.summary).unwrap();
        if let Some(rows) = &m.rows {
            if !rows.is_empty() {
                let data: Vec<(String, u64)> =
                    rows.iter().map(|r| (r.key.clone(), r.value)).collect();
                let mut table = tabled::Table::new(data);
                table.with(tabled::settings::Style::rounded());
                writeln!(&mut buf, "{table}").unwrap();
            }
        } else if let Some(s) = m.scalar {
            writeln!(&mut buf, "value: {s}").unwrap();
        }
        writeln!(&mut buf).unwrap();
    }
    if !report.alerts.is_empty() {
        writeln!(&mut buf, "== Alerts ==").unwrap();
        for a in &report.alerts {
            writeln!(
                &mut buf,
                "[{:?}] {} — {}",
                a.severity, a.code, a.message
            )
            .unwrap();
            if let Some(e) = &a.evidence {
                writeln!(&mut buf, "  evidence: {e}").unwrap();
            }
        }
    }
    buf
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AlertHint, AlertSeverity, MetricId, MetricResult};

    #[test]
    fn json_roundtrip_shape() {
        let report = ScanReport {
            repo: "/tmp".into(),
            since: "1 year ago".into(),
            metrics: vec![MetricResult {
                id: MetricId::Firefighting,
                label: "x".into(),
                summary: "y".into(),
                rows: None,
                scalar: Some(3),
            }],
            alerts: vec![AlertHint {
                severity: AlertSeverity::Info,
                code: "test".into(),
                message: "ok".into(),
                evidence: None,
            }],
        };
        let s = render(&report, OutputFormat::Json).unwrap();
        assert!(s.contains("firefighting"));
        assert!(s.contains("alerts"));
    }
}
