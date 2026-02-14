use std::fmt::Write;

use super::CommandEntry;

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
