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

    /// Column titles for tabular metrics (`key`, `value` pairs).
    pub fn row_columns(self) -> (&'static str, &'static str) {
        match self {
            MetricId::Churn => ("file", "changes"),
            MetricId::BusFactor => ("author", "commits"),
            MetricId::BugHotspots => ("file", "touches"),
            MetricId::DeliveryPace => ("month", "commits"),
            MetricId::Firefighting => ("", ""),
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
#[derive(Clone, Debug, Deserialize)]
pub struct MetricResult {
    pub id: MetricId,
    pub label: String,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rows: Option<Vec<MetricRow>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scalar: Option<u64>,
}

impl Serialize for MetricResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut field_count = 3;
        if self.rows.is_some() {
            field_count += 1;
        }
        if self.scalar.is_some() {
            field_count += 1;
        }

        let mut state = serializer.serialize_struct("MetricResult", field_count)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("label", &self.label)?;
        state.serialize_field("summary", &self.summary)?;
        if let Some(rows) = &self.rows {
            state.serialize_field("rows", &NamedMetricRows {
                id: self.id,
                rows,
            })?;
        }
        if let Some(scalar) = self.scalar {
            state.serialize_field("scalar", &scalar)?;
        }
        state.end()
    }
}

struct NamedMetricRows<'a> {
    id: MetricId,
    rows: &'a [MetricRow],
}

impl Serialize for NamedMetricRows<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;

        let (key_name, value_name) = self.id.row_columns();
        let mut seq = serializer.serialize_seq(Some(self.rows.len()))?;
        for row in self.rows {
            seq.serialize_element(&NamedMetricRow {
                key_name,
                value_name,
                row,
            })?;
        }
        seq.end()
    }
}

struct NamedMetricRow<'a> {
    key_name: &'static str,
    value_name: &'static str,
    row: &'a MetricRow,
}

impl Serialize for NamedMetricRow<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("MetricRow", 2)?;
        state.serialize_field(self.key_name, &self.row.key)?;
        state.serialize_field(self.value_name, &self.row.value)?;
        state.end()
    }
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
    pub repo: String,
    pub since: String,
    pub recent_since: String,
    pub source_dirs: Vec<String>,
    pub metrics: Vec<MetricResult>,
    pub alerts: Vec<AlertHint>,
}
