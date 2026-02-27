use std::fmt::Write;

use super::CommandEntry;

/// Escape a string for JSON output per RFC 8259.
///
/// Handles `\`, `"`, and all control characters (U+0000..U+001F).
fn escape_json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\x08' => out.push_str("\\b"),
            '\x0C' => out.push_str("\\f"),
            c if c < '\x20' => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out
}

/// Format output as JSON (no external dependencies)
pub fn format_json(entries: &[CommandEntry]) -> String {
    // Pre-allocate with estimated size (avg ~80 chars per entry)
    let mut result = String::with_capacity(entries.len() * 80 + 4);
    result.push_str("[\n");

    for (i, entry) in entries.iter().enumerate() {
        let escaped_cmd = escape_json_string(&entry.command);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_json_common_chars() {
        assert_eq!(escape_json_string("hello"), "hello");
        assert_eq!(escape_json_string("a\\b"), "a\\\\b");
        assert_eq!(escape_json_string("a\"b"), "a\\\"b");
        assert_eq!(escape_json_string("a\nb"), "a\\nb");
        assert_eq!(escape_json_string("a\rb"), "a\\rb");
        assert_eq!(escape_json_string("a\tb"), "a\\tb");
    }

    #[test]
    fn test_escape_json_control_characters() {
        assert_eq!(escape_json_string("\x00"), "\\u0000");
        assert_eq!(escape_json_string("\x01"), "\\u0001");
        assert_eq!(escape_json_string("\x08"), "\\b");
        assert_eq!(escape_json_string("\x0C"), "\\f");
        assert_eq!(escape_json_string("\x1F"), "\\u001f");
        // Mixed: normal + control
        assert_eq!(escape_json_string("a\x07b"), "a\\u0007b");
    }
}
