//! Git health metrics CLI library.

pub mod alerts;
pub mod cli;
pub mod commands;
pub mod git;
pub mod metrics;
pub mod model;
pub mod report;

pub use model::{AlertHint, AlertSeverity, MetricId, MetricResult, OutputFormat, ScanReport};
