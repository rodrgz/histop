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
use std::io::{self, BufRead, BufReader};

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
    let mut reader = BufReader::new(file);
    let mut cmd_count: HashMap<String, usize> = HashMap::with_capacity(256);

    let ignore_refs: Vec<&str> = ignore.iter().map(|s| s.as_str()).collect();
    let mut line_buf: Vec<u8> = Vec::with_capacity(256);

    loop {
        line_buf.clear();
        let bytes_read = reader.read_until(b'\n', &mut line_buf)?;
        if bytes_read == 0 {
            break;
        }

        let line = std::str::from_utf8(&line_buf).map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidData, e)
        })?;

        // Fish history command lines start with "- cmd: "
        if let Some(cmd) = line.strip_prefix("- cmd: ") {
            if let Some(first_word) = get_first_word(cmd, &ignore_refs) {
                increment_count(&mut cmd_count, first_word);
            }
        }
        // Lines starting with "  when:" or "  paths:" are metadata, skip them
    }

    Ok(cmd_count)
}

#[inline]
fn increment_count(cmd_count: &mut HashMap<String, usize>, first_word: &str) {
    if let Some(existing) = cmd_count.get_mut(first_word) {
        *existing += 1;
    } else {
        cmd_count.insert(first_word.to_string(), 1);
    }
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
