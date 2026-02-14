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

use crate::shared::command_parse::get_first_word;

/// Parse fish_history file and count commands
///
/// # Arguments
/// * `file_path` - Path to the fish_history file
/// * `ignore` - List of commands to ignore
///
/// # Returns
/// A HashMap of command -> count
pub fn count_from_file(
    file_path: &str,
    ignore: &[String],
) -> Result<HashMap<String, usize>, std::io::Error> {
    let file = fs::File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut cmd_count: HashMap<String, usize> = HashMap::new();

    let ignore_refs: Vec<&str> = ignore.iter().map(|s| s.as_str()).collect();

    for line in reader.lines() {
        let line = line?;

        // Fish history command lines start with "- cmd: "
        if let Some(cmd) = line.strip_prefix("- cmd: ") {
            let cmd = cmd.trim();
            if !cmd.is_empty() {
                let first_word = get_first_word(cmd, &ignore_refs);
                if !first_word.is_empty() {
                    *cmd_count.entry(first_word.into_owned()).or_default() += 1;
                }
            }
        }
        // Lines starting with "  when:" or "  paths:" are metadata, skip them
    }

    Ok(cmd_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_simple_commands() {
        // Create a temp file with fish history format
        use std::io::Write;
        use std::time::{SystemTime, UNIX_EPOCH};
        let now_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "test_fish_history_{}_{}",
            std::process::id(),
            now_nanos
        ));
        let mut file = fs::File::create(&path).unwrap();
        writeln!(file, "- cmd: ls -la").unwrap();
        writeln!(file, "  when: 1680820391").unwrap();
        writeln!(file, "- cmd: git status").unwrap();
        writeln!(file, "  when: 1680820392").unwrap();
        writeln!(file, "- cmd: ls").unwrap();
        writeln!(file, "  when: 1680820393").unwrap();

        let result = count_from_file(path.to_str().unwrap(), &[]).unwrap();
        assert_eq!(result.get("ls"), Some(&2));
        assert_eq!(result.get("git"), Some(&1));

        fs::remove_file(path).ok();
    }
}
