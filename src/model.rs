use serde::{Deserialize, Serialize};

/// How to render CLI output.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, clap::ValueEnum)]
pub enum OutputFormat {
    #[value(name = "table")]
    #[default]
    Table,
    #[value(name = "json")]
    Json,
}

/// Known metric identifiers (aligned with `docs/git-metrics.md`).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricId {
    Churn,
    BusFactor,
    BugHotspots,
    DeliveryPace,
    Firefighting,
}

impl MetricId {
    pub fn as_str(self) -> &'static str {
        match self {
            MetricId::Churn => "churn",
            MetricId::BusFactor => "bus_factor",
            MetricId::BugHotspots => "bug_hotspots",
            MetricId::DeliveryPace => "delivery_pace",
            MetricId::Firefighting => "firefighting",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            MetricId::Churn => "High-churn files",
            MetricId::BusFactor => "Ownership (shortlog)",
            MetricId::BugHotspots => "Bug hotspots",
            MetricId::DeliveryPace => "Delivery pace",
            MetricId::Firefighting => "Firefighting",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            MetricId::Churn => "Files changed most often in the time window.",
            MetricId::BusFactor => "Contributors ranked by commit count (no merges).",
            MetricId::BugHotspots => "Files touched in commits matching fix|bug|broken.",
            MetricId::DeliveryPace => "Commit counts grouped by year-month.",
            MetricId::Firefighting => "Commits whose subject matches revert/hotfix/emergency/rollback.",
        }
    }

    pub fn all() -> &'static [MetricId] {
        &[
            MetricId::Churn,
            MetricId::BusFactor,
            MetricId::BugHotspots,
            MetricId::DeliveryPace,
            MetricId::Firefighting,
        ]
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "churn" | "high_churn" | "1" => Some(MetricId::Churn),
            "bus_factor" | "shortlog" | "ownership" | "2" => Some(MetricId::BusFactor),
            "bug_hotspots" | "bugs" | "3" => Some(MetricId::BugHotspots),
            "delivery_pace" | "delivery" | "4" => Some(MetricId::DeliveryPace),
            "firefighting" | "reverts" | "5" => Some(MetricId::Firefighting),
            _ => None,
        }
    }
}

/// One row in a tabular metric (e.g. file + count).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetricRow {
    pub key: String,
    pub value: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<String>,
}

/// Result of computing a single metric.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetricResult {
    pub id: MetricId,
    pub label: String,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rows: Option<Vec<MetricRow>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scalar: Option<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertSeverity {
    Info,
    Warning,
    High,
}

/// Lightweight hint for leadership review.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AlertHint {
    pub severity: AlertSeverity,
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
}

/// Full scan payload for table or JSON output.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanReport {
    pub repo: String,
    pub since: String,
    pub recent_since: String,
    pub source_dirs: Vec<String>,
    pub metrics: Vec<MetricResult>,
    pub alerts: Vec<AlertHint>,
}
