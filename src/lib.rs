//! Five git-log drag diagnostics on an unfamiliar codebase before you read code.

pub mod alerts;
pub mod cli;
pub mod commands;
pub mod git;
pub mod metrics;
pub mod model;
pub mod report;
pub mod sanitize;
pub mod validate;

pub use model::{AlertHint, AlertSeverity, MetricId, MetricResult, OutputFormat, ScanReport};
