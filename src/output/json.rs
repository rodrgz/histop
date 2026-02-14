use std::fmt::Write;

use super::CommandEntry;

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
