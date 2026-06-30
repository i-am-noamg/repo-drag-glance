use std::io::{stdout, IsTerminal};

use colored::ColoredString;
use colored::Colorize;

/// ANSI styling for table output; disabled for JSON, `--no-color`, or `NO_COLOR`.
#[derive(Clone, Copy, Debug)]
pub struct Style {
    enabled: bool,
}

impl Style {
    pub fn new(no_color: bool) -> Self {
        let enabled = stdout().is_terminal() && !no_color && std::env::var_os("NO_COLOR").is_none();
        Self { enabled }
    }

    fn paint<F>(&self, s: &str, f: F) -> String
    where
        F: Fn(&str) -> ColoredString,
    {
        if self.enabled {
            f(s).to_string()
        } else {
            s.to_string()
        }
    }

    pub fn warning(&self, s: &str) -> String {
        self.paint(s, |x| x.bright_yellow().bold())
    }

    pub fn header_label(&self, s: &str) -> String {
        self.paint(s, |x| x.bold())
    }

    pub fn section(&self, s: &str) -> String {
        self.paint(s, |x| x.cyan().bold())
    }

    pub fn summary(&self, s: &str) -> String {
        self.paint(s, |x| x.bright_black())
    }

    pub fn scalar_label(&self, s: &str) -> String {
        self.paint(s, |x| x.bold())
    }

    pub fn scalar_value(&self, s: &str) -> String {
        self.paint(s, |x| x.green().bold())
    }

    pub fn alert_severity(&self, severity: crate::model::AlertSeverity) -> String {
        let label = format!("{severity:?}");
        match severity {
            crate::model::AlertSeverity::Info => self.paint(&label, |x| x.blue().bold()),
            crate::model::AlertSeverity::Warning => self.paint(&label, |x| x.yellow().bold()),
            crate::model::AlertSeverity::High => self.paint(&label, |x| x.red().bold()),
        }
    }

    pub fn alert_code(&self, s: &str) -> String {
        self.paint(s, |x| x.magenta().bold())
    }

    pub fn evidence_label(&self, s: &str) -> String {
        self.paint(s, |x| x.bright_black())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_style_is_plain_text() {
        let style = Style { enabled: false };
        let s = style.warning("Warning: test");
        assert_eq!(s, "Warning: test");
        assert!(!s.contains('\x1b'));
    }
}
