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

use crate::shared::command_parse::{clean_line, get_first_word};

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
    no_hist: bool,
) -> Result<HashMap<String, usize>, std::io::Error> {
    let file = fs::File::open(file_path)?;
    let mut reader = BufReader::new(file);
    let mut cmd_count: HashMap<String, usize> = HashMap::with_capacity(256);

    let mut filtered_commands: Vec<&str> = Vec::with_capacity(ignore.len() + 2);
    if !no_hist {
        filtered_commands.extend(["sudo", "doas"]);
    }
    let ignore_refs: Vec<&str> = ignore.iter().map(|s| s.as_str()).collect();
    filtered_commands.extend(ignore_refs);
    let mut line_buf: Vec<u8> = Vec::with_capacity(256);
    let mut current_cmd: Option<String> = None;

    loop {
        line_buf.clear();
        let bytes_read = reader.read_until(b'\n', &mut line_buf)?;
        if bytes_read == 0 {
            break;
        }

        let line = match std::str::from_utf8(&line_buf) {
            Ok(line) => trim_line_end(line),
            Err(_) => continue,
        };

        // Fish history command lines start with "- cmd: "
        if let Some(cmd) = line.strip_prefix("- cmd: ") {
            if let Some(existing_cmd) = current_cmd.take() {
                count_commands(&mut cmd_count, &existing_cmd, &filtered_commands, no_hist);
            }
            current_cmd = Some(cmd.to_string());
            continue;
        }

        if let Some(cmd) = current_cmd.as_mut() {
            // Multiline fish command continuation:
            // - cmd: doas -- \
            //   systemctl stop sshd
            if cmd.ends_with('\\')
                && line.starts_with("  ")
                && !line.starts_with("  when: ")
                && !line.starts_with("  paths:")
                && !line.starts_with("  - ")
            {
                cmd.pop();
                *cmd = cmd.trim_end().to_string();
                cmd.push(' ');
                cmd.push_str(line.trim_start());
                continue;
            }
        }

        // Metadata ends the current command entry
        if line.starts_with("  when: ") || line.starts_with("  paths:") || line.starts_with("  - ") {
            if let Some(existing_cmd) = current_cmd.take() {
                count_commands(&mut cmd_count, &existing_cmd, &filtered_commands, no_hist);
            }
        }
    }

    if let Some(existing_cmd) = current_cmd.take() {
        count_commands(&mut cmd_count, &existing_cmd, &filtered_commands, no_hist);
    }

    Ok(cmd_count)
}

#[inline]
fn trim_line_end(line: &str) -> &str {
    line.trim_end_matches(['\n', '\r'])
}

fn count_commands(
    cmd_count: &mut HashMap<String, usize>,
    line: &str,
    filtered_commands: &[&str],
    no_hist: bool,
) {
    if line.contains('|') && !no_hist {
        if line.contains('\'') || line.contains('"') {
            let cleaned_line = clean_line(line);
            for subcommand in cleaned_line.split('|') {
                if let Some(first_word) = get_first_word(subcommand, filtered_commands) {
                    increment_count(cmd_count, first_word);
                }
            }
        } else {
            for subcommand in line.split('|') {
                if let Some(first_word) = get_first_word(subcommand, filtered_commands) {
                    increment_count(cmd_count, first_word);
                }
            }
        }
    } else if let Some(first_word) = get_first_word(line, filtered_commands) {
        increment_count(cmd_count, first_word);
    }
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

        let result = count_from_file(path.to_str().unwrap(), &[], false).unwrap();
        assert_eq!(result.get("ls"), Some(&2));
        assert_eq!(result.get("git"), Some(&1));

        fs::remove_file(path).ok();
    }

    #[test]
    fn test_count_multiline_command_with_doas_wrapper() {
        use std::io::Write;
        use std::time::{SystemTime, UNIX_EPOCH};
        let now_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "test_fish_multiline_{}_{}",
            std::process::id(),
            now_nanos
        ));
        let mut file = fs::File::create(&path).unwrap();
        writeln!(file, "- cmd: doas -- \\").unwrap();
        writeln!(file, "  systemctl stop sshd").unwrap();
        writeln!(file, "  when: 1680820391").unwrap();

        let result = count_from_file(path.to_str().unwrap(), &[], false).unwrap();
        assert_eq!(result.get("systemctl"), Some(&1));
        assert_eq!(result.get("doas"), None);
        assert_eq!(result.get("--"), None);

        fs::remove_file(path).ok();
    }

    #[test]
    fn test_invalid_utf8_line_is_ignored() {
        use std::io::Write;
        use std::time::{SystemTime, UNIX_EPOCH};
        let now_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "test_fish_invalid_utf8_{}_{}",
            std::process::id(),
            now_nanos
        ));
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(b"- cmd: ls -la\n").unwrap();
        file.write_all(b"\xFF\xFE\xFD\n").unwrap();
        file.write_all(b"- cmd: git status\n").unwrap();
        file.write_all(b"  when: 1680820391\n").unwrap();

        let result = count_from_file(path.to_str().unwrap(), &[], false).unwrap();
        assert_eq!(result.get("ls"), Some(&1));
        assert_eq!(result.get("git"), Some(&1));

        fs::remove_file(path).ok();
    }
}
