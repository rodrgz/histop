//! Simple TOML-like configuration file parser.
//!
//! Parses a subset of TOML for histop configuration without external dependencies.
//! Supports: strings, integers, and arrays of strings.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::output::color::ColorMode;

/// Configuration loaded from file
#[derive(Debug, Default)]
pub struct FileConfig {
    /// Commands to ignore
    pub ignore: Option<Vec<String>>,
    /// Bar size
    pub bar_size: Option<usize>,
    /// Number of commands to show
    pub count: Option<usize>,
    /// Color mode
    pub color: Option<ColorMode>,
    /// More than threshold
    pub more_than: Option<usize>,
}

impl FileConfig {
    /// Load configuration from the default location
    /// (~/.config/histop/config.toml)
    pub fn load_default() -> Option<Self> {
        let home = std::env::var("HOME").ok()?;
        let config_path = Path::new(&home).join(".config/histop/config.toml");

        if config_path.exists() { Self::load(&config_path).ok() } else { None }
    }

    /// Load configuration from a specific path
    pub fn load(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        Self::parse(&content)
    }

    /// Parse configuration from string content
    pub fn parse(content: &str) -> Result<Self, String> {
        let mut config = FileConfig::default();
        let values = parse_toml(content)?;

        for (key, parsed) in values {
            match key.as_str() {
                "ignore" => {
                    let arr =
                        parse_string_array(&parsed.value).map_err(|e| {
                            format!(
                                "Line {}: invalid 'ignore' value: {}",
                                parsed.line, e
                            )
                        })?;
                    config.ignore = Some(arr);
                }
                "bar_size" => {
                    let n = parse_integer(&parsed.value).map_err(|e| {
                        format!(
                            "Line {}: invalid 'bar_size' value: {}",
                            parsed.line, e
                        )
                    })?;
                    config.bar_size =
                        Some(parse_positive_integer(n, "bar_size")?);
                }
                "count" => {
                    let n = parse_integer(&parsed.value).map_err(|e| {
                        format!(
                            "Line {}: invalid 'count' value: {}",
                            parsed.line, e
                        )
                    })?;
                    config.count = Some(parse_positive_integer(n, "count")?);
                }
                "more_than" => {
                    let n = parse_integer(&parsed.value).map_err(|e| {
                        format!(
                            "Line {}: invalid 'more_than' value: {}",
                            parsed.line, e
                        )
                    })?;
                    config.more_than =
                        Some(parse_non_negative_integer(n, "more_than")?);
                }
                "color" => {
                    let color = parse_string(&parsed.value).map_err(|e| {
                        format!(
                            "Line {}: invalid 'color' value: {}",
                            parsed.line, e
                        )
                    })?;
                    let parsed_color = ColorMode::parse(color).ok_or_else(|| {
                        format!(
                            "Line {}: invalid 'color' value '{}'. Use auto, always, or never",
                            parsed.line, color
                        )
                    })?;
                    config.color = Some(parsed_color);
                }
                _ => {
                    return Err(format!(
                        "Line {}: unknown key '{}'",
                        parsed.line, key
                    ));
                }
            }
        }

        Ok(config)
    }
}

/// Simple TOML value types
#[derive(Debug, Clone)]
enum Value {
    String(String),
    Integer(i64),
    Array(Vec<Value>),
}

#[derive(Debug, Clone)]
struct ParsedValue {
    value: Value,
    line: usize,
}

/// Parse a simple TOML file (subset of TOML spec)
fn parse_toml(content: &str) -> Result<HashMap<String, ParsedValue>, String> {
    let mut values = HashMap::new();

    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Skip section headers for now (we only support top-level keys)
        if line.starts_with('[') {
            continue;
        }

        // Parse key = value
        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim();
            let value_str = line[eq_pos + 1..].trim();

            let value = parse_value(value_str)
                .map_err(|e| format!("Line {}: {}", line_num + 1, e))?;

            values.insert(
                key.to_string(),
                ParsedValue { value, line: line_num + 1 },
            );
        }
    }

    Ok(values)
}

/// Parse a TOML value
fn parse_value(s: &str) -> Result<Value, String> {
    let s = s.trim();

    // String (quoted)
    if (s.starts_with('"') && s.ends_with('"'))
        || (s.starts_with('\'') && s.ends_with('\''))
    {
        let inner = &s[1..s.len() - 1];
        return Ok(Value::String(inner.to_string()));
    }

    // Array
    if s.starts_with('[') && s.ends_with(']') {
        let inner = &s[1..s.len() - 1];
        let items = parse_array(inner)?;
        return Ok(Value::Array(items));
    }

    // Integer
    if let Ok(n) = s.parse::<i64>() {
        return Ok(Value::Integer(n));
    }

    // Unquoted string (for simple values like color = auto)
    Ok(Value::String(s.to_string()))
}

/// Parse array contents
fn parse_array(s: &str) -> Result<Vec<Value>, String> {
    let mut items = Vec::new();
    let s = s.trim();

    if s.is_empty() {
        return Ok(items);
    }

    // Simple comma-separated parsing
    for item in s.split(',') {
        let item = item.trim();
        if !item.is_empty() {
            items.push(parse_value(item)?);
        }
    }

    Ok(items)
}

fn parse_integer(value: &Value) -> Result<i64, String> {
    match value {
        Value::Integer(n) => Ok(*n),
        _ => Err(format!("expected integer, got {}", value_type(value))),
    }
}

fn parse_string(value: &Value) -> Result<&str, String> {
    match value {
        Value::String(s) => Ok(s),
        _ => Err(format!("expected string, got {}", value_type(value))),
    }
}

fn parse_string_array(value: &Value) -> Result<Vec<String>, String> {
    match value {
        Value::Array(arr) => {
            let mut out = Vec::with_capacity(arr.len());
            for item in arr {
                match item {
                    Value::String(s) => out.push(s.clone()),
                    _ => {
                        return Err(format!(
                            "expected array of strings, found {}",
                            value_type(item)
                        ));
                    }
                }
            }
            Ok(out)
        }
        _ => Err(format!("expected array, got {}", value_type(value))),
    }
}

fn value_type(value: &Value) -> &'static str {
    match value {
        Value::String(_) => "string",
        Value::Integer(_) => "integer",
        Value::Array(_) => "array",
    }
}

fn parse_positive_integer(
    value: i64,
    key: &str,
) -> Result<usize, String> {
    if value <= 0 {
        return Err(format!("{} must be a positive integer", key));
    }
    usize::try_from(value).map_err(|_| format!("{} is too large", key))
}

fn parse_non_negative_integer(
    value: i64,
    key: &str,
) -> Result<usize, String> {
    if value < 0 {
        return Err(format!("{} must be a non-negative integer", key));
    }
    usize::try_from(value).map_err(|_| format!("{} is too large", key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_config() {
        let content = r#"
# Histop configuration
count = 30
bar_size = 20
color = "auto"
ignore = ["ls", "cd", "exit"]
"#;
        let config = FileConfig::parse(content).unwrap();
        assert_eq!(config.count, Some(30));
        assert_eq!(config.bar_size, Some(20));
        assert_eq!(config.color, Some(ColorMode::Auto));
        assert_eq!(
            config.ignore,
            Some(vec!["ls".to_string(), "cd".to_string(), "exit".to_string()])
        );
    }

    #[test]
    fn test_parse_empty_config() {
        let content = "";
        let config = FileConfig::parse(content).unwrap();
        assert!(config.count.is_none());
        assert!(config.ignore.is_none());
    }

    #[test]
    fn test_parse_with_comments() {
        let content = r#"
# This is a comment
count = 10 # inline comments not supported, this will fail
"#;
        let config = FileConfig::parse(content);
        assert!(config.is_err());
    }

    #[test]
    fn test_parse_unquoted_string() {
        let content = "color = auto";
        let config = FileConfig::parse(content).unwrap();
        assert_eq!(config.color, Some(ColorMode::Auto));
    }

    #[test]
    fn test_parse_negative_count_rejected() {
        let content = "count = -1";
        let config = FileConfig::parse(content);
        assert!(config.is_err());
    }

    #[test]
    fn test_parse_zero_bar_size_rejected() {
        let content = "bar_size = 0";
        let config = FileConfig::parse(content);
        assert!(config.is_err());
    }

    #[test]
    fn test_parse_negative_more_than_rejected() {
        let content = "more_than = -1";
        let config = FileConfig::parse(content);
        assert!(config.is_err());
    }

    #[test]
    fn test_parse_zero_more_than_allowed() {
        let content = "more_than = 0";
        let config = FileConfig::parse(content).unwrap();
        assert_eq!(config.more_than, Some(0));
    }

    #[test]
    fn test_parse_unknown_key_rejected() {
        let content = "invalid_key = 1";
        let config = FileConfig::parse(content);
        assert!(config.is_err());
    }

    #[test]
    fn test_parse_type_mismatch_rejected() {
        let content = "count = \"10\"";
        let config = FileConfig::parse(content);
        assert!(config.is_err());
    }

    #[test]
    fn test_parse_ignore_type_mismatch_rejected() {
        let content = "ignore = [\"ls\", 1]";
        let config = FileConfig::parse(content);
        assert!(config.is_err());
    }

    #[test]
    fn test_parse_invalid_color_rejected() {
        let content = "color = \"sometimes\"";
        let config = FileConfig::parse(content);
        assert!(config.is_err());
    }
}
