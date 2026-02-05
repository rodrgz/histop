//! Fish shell history parser
//!
//! Parses the fish_history file format, which uses a YAML-like structure:
//! ```yaml
//! - cmd: some command
//!   when: 1680820391
//!   paths:
//!     - /some/path
//! ```

use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};

/// Parse fish_history file and count commands
///
/// # Arguments
/// * `file_path` - Path to the fish_history file
/// * `ignore` - List of commands to ignore
/// * `_verbose` - Enable verbose output (reserved for future use)
///
/// # Returns
/// A HashMap of command -> count
pub fn count_from_file(
    file_path: &str,
    ignore: &[String],
    _verbose: bool,
) -> Result<HashMap<String, usize>, std::io::Error> {
    let file = fs::File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut cmd_count: HashMap<String, usize> = HashMap::new();

    for line in reader.lines() {
        let line = line?;

        // Fish history command lines start with "- cmd: "
        if let Some(cmd) = line.strip_prefix("- cmd: ") {
            let cmd = cmd.trim();
            if !cmd.is_empty() {
                let first_word = get_first_word(cmd, ignore);
                if !first_word.is_empty() {
                    *cmd_count.entry(first_word.to_string()).or_default() += 1;
                }
            }
        }
        // Lines starting with "  when:" or "  paths:" are metadata, skip them
    }

    Ok(cmd_count)
}

/// Extract the first meaningful word from a command
///
/// Skips environment variable assignments (FOO=bar) but preserves
/// variable expansions ($EDITOR).
fn get_first_word<'a>(cmd: &'a str, ignore: &[String]) -> &'a str {
    for w in cmd.split_whitespace() {
        // Skip env var assignments (FOO=bar) but not expansions ($FOO)
        if w.contains('=') && !w.starts_with('$') {
            continue;
        }
        // Skip ignored commands
        if ignore.iter().any(|i| i == w) {
            continue;
        }
        return w;
    }
    ""
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_first_word_simple() {
        let ignore = vec![];
        assert_eq!(get_first_word("ls -la", &ignore), "ls");
    }

    #[test]
    fn test_get_first_word_with_env_var() {
        let ignore = vec![];
        assert_eq!(get_first_word("FOO=bar cmd arg", &ignore), "cmd");
    }

    #[test]
    fn test_get_first_word_preserves_expansion() {
        let ignore = vec![];
        assert_eq!(get_first_word("$EDITOR file.txt", &ignore), "$EDITOR");
    }

    #[test]
    fn test_get_first_word_with_ignore() {
        let ignore = vec!["sudo".to_string()];
        assert_eq!(get_first_word("sudo apt update", &ignore), "apt");
    }

    #[test]
    fn test_empty_command() {
        let ignore = vec![];
        assert_eq!(get_first_word("", &ignore), "");
    }
}
