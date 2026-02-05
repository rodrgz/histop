//! Output formatting module for different output formats.

use std::fmt::Write;

use crate::bar::RenderedBar;

/// Output format for results
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum OutputFormat {
    /// Default text output with bar graphs
    #[default]
    Text,
    /// JSON output
    Json,
    /// CSV output
    Csv,
}

impl OutputFormat {
    /// Parse from string (for CLI argument)
    #[inline]
    pub fn parse(s: &str) -> Option<Self> {
        if s.eq_ignore_ascii_case("text") {
            Some(Self::Text)
        } else if s.eq_ignore_ascii_case("json") {
            Some(Self::Json)
        } else if s.eq_ignore_ascii_case("csv") {
            Some(Self::Csv)
        } else {
            None
        }
    }
}

/// Command count entry for output formatting
#[derive(Debug)]
pub struct CommandEntry {
    pub command: String,
    pub count: usize,
    pub percentage: f64,
}

impl CommandEntry {
    pub fn new(command: String, count: usize, total: usize) -> Self {
        let percentage = if total > 0 {
            (count as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        Self {
            command,
            count,
            percentage,
        }
    }
}

/// Format output as JSON (no external dependencies)
pub fn format_json(entries: &[CommandEntry]) -> String {
    // Pre-allocate with estimated size (avg ~80 chars per entry)
    let mut result = String::with_capacity(entries.len() * 80 + 4);
    result.push_str("[\n");

    for (i, entry) in entries.iter().enumerate() {
        // Escape special characters in command
        let escaped_cmd = entry
            .command
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t");

        let _ = write!(
            result,
            "  {{\n    \"command\": \"{}\",\n    \"count\": {},\n    \"percentage\": {:.2}\n  }}",
            escaped_cmd, entry.count, entry.percentage
        );

        if i < entries.len() - 1 {
            result.push(',');
        }
        result.push('\n');
    }

    result.push(']');
    result
}

/// Format output as CSV
pub fn format_csv(entries: &[CommandEntry]) -> String {
    // Pre-allocate with estimated size (avg ~30 chars per entry + header)
    let mut result = String::with_capacity(entries.len() * 30 + 30);
    result.push_str("command,count,percentage\n");

    for entry in entries {
        // Escape CSV fields
        let escaped_cmd = if entry.command.contains(',')
            || entry.command.contains('"')
            || entry.command.contains('\n')
        {
            let mut escaped = String::with_capacity(entry.command.len() + 2);
            escaped.push('"');
            for c in entry.command.chars() {
                if c == '"' {
                    escaped.push_str("\"\"");
                } else {
                    escaped.push(c);
                }
            }
            escaped.push('"');
            escaped
        } else {
            entry.command.clone()
        };

        let _ = write!(result, "{},{},{:.2}\n", escaped_cmd, entry.count, entry.percentage);
    }

    result
}

/// Convert RenderedBars to CommandEntries for alternative output formats
pub fn bars_to_entries(bars: &[RenderedBar], total: usize) -> Vec<CommandEntry> {
    bars.iter()
        .map(|bar| {
            let count: usize = bar.count_str.trim().parse().unwrap_or(0);
            CommandEntry::new(bar.label.clone(), count, total)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_from_str() {
        assert_eq!(OutputFormat::parse("text"), Some(OutputFormat::Text));
        assert_eq!(OutputFormat::parse("json"), Some(OutputFormat::Json));
        assert_eq!(OutputFormat::parse("csv"), Some(OutputFormat::Csv));
        assert_eq!(OutputFormat::parse("invalid"), None);
    }

    #[test]
    fn test_format_json() {
        let entries = vec![
            CommandEntry::new("ls".to_string(), 10, 100),
            CommandEntry::new("git".to_string(), 5, 100),
        ];
        let json = format_json(&entries);
        assert!(json.contains("\"command\": \"ls\""));
        assert!(json.contains("\"count\": 10"));
        assert!(json.contains("\"percentage\": 10.00"));
    }

    #[test]
    fn test_format_csv() {
        let entries = vec![
            CommandEntry::new("ls".to_string(), 10, 100),
            CommandEntry::new("git".to_string(), 5, 100),
        ];
        let csv = format_csv(&entries);
        assert!(csv.starts_with("command,count,percentage\n"));
        assert!(csv.contains("ls,10,10.00"));
    }

    #[test]
    fn test_json_escaping() {
        let entries = vec![CommandEntry::new("echo \"hello\"".to_string(), 1, 1)];
        let json = format_json(&entries);
        assert!(json.contains("echo \\\"hello\\\""));
    }

    #[test]
    fn test_csv_escaping() {
        let entries = vec![CommandEntry::new("echo,hello".to_string(), 1, 1)];
        let csv = format_csv(&entries);
        assert!(csv.contains("\"echo,hello\""));
    }
}
