//! Fish shell history parser
//!
//! Parses the fish_history file format, which uses a YAML-like structure:
//! ```yaml
//! - cmd: some command
//!   when: 1680820391
//!   paths:
//!     - /some/path
//! ```

use ahash::{AHashMap, AHashSet};
use std::fs;
use std::io::{BufRead, BufReader};

use crate::shared::command_parse::{get_first_word, SplitCommands};

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
) -> Result<AHashMap<String, usize>, std::io::Error> {
    // 256KB buffer size
    const BUFFER_SIZE: usize = 256 * 1024;

    let mut cmd_count: AHashMap<String, usize> = AHashMap::default();

    let mut filtered_commands: AHashSet<&str> = AHashSet::with_capacity(ignore.len() + 2);
    if !no_hist {
        filtered_commands.insert("sudo");
        filtered_commands.insert("doas");
    }
    for s in ignore {
        filtered_commands.insert(s.as_str());
    }

    let file = fs::File::open(file_path)?;
    let reader = BufReader::with_capacity(BUFFER_SIZE, file);
    count_from_reader(reader, &mut cmd_count, &filtered_commands, no_hist)?;

    Ok(cmd_count)
}


fn count_from_reader<R: BufRead>(
    mut reader: R,
    cmd_count: &mut AHashMap<String, usize>,
    filtered_commands: &AHashSet<&str>,
    no_hist: bool,
) -> std::io::Result<()> {
    let mut current_cmd = String::with_capacity(256);
    let mut line_buf = Vec::with_capacity(256);

    loop {
        line_buf.clear();
        let bytes_read = reader.read_until(b'\n', &mut line_buf)?;
        if bytes_read == 0 {
            break;
        }

        let line = match std::str::from_utf8(&line_buf) {
            Ok(s) => trim_line_end(s),
            Err(_) => continue,
        };

        process_line(
            line,
            &mut current_cmd,
            cmd_count,
            filtered_commands,
            no_hist,
        );
    }

    if !current_cmd.is_empty() {
        count_commands(cmd_count, &current_cmd, filtered_commands, no_hist);
    }

    Ok(())
}

fn process_line(
    trimmed_line: &str,
    current_cmd: &mut String,
    cmd_count: &mut AHashMap<String, usize>,
    filtered_commands: &AHashSet<&str>,
    no_hist: bool,
) {
    // Fish history command lines start with "- cmd: "
    if let Some(cmd) = trimmed_line.strip_prefix("- cmd: ") {
        if !current_cmd.is_empty() {
            count_commands(cmd_count, current_cmd, filtered_commands, no_hist);
        }
        current_cmd.clear();
        current_cmd.push_str(cmd);
        return;
    }

    if !current_cmd.is_empty() {
        // Multiline fish command continuation:
        // - cmd: doas -- \
        //   systemctl stop sshd
        if current_cmd.ends_with('\\')
            && trimmed_line.starts_with("  ")
            && !trimmed_line.starts_with("  when: ")
            && !trimmed_line.starts_with("  paths:")
            && !trimmed_line.starts_with("  - ")
        {
            current_cmd.pop();
            let len = current_cmd.trim_end().len();
            current_cmd.truncate(len);
            current_cmd.push(' ');
            current_cmd.push_str(trimmed_line.trim_start());
            return;
        }

        // Metadata ends the current command entry
        if trimmed_line.starts_with("  when: ")
            || trimmed_line.starts_with("  paths:")
            || trimmed_line.starts_with("  - ")
        {
            count_commands(cmd_count, current_cmd, filtered_commands, no_hist);
            current_cmd.clear();
        }
    }
}

#[inline]
fn trim_line_end(line: &str) -> &str {
    line.trim_end_matches(['\n', '\r'])
}

fn count_commands(
    cmd_count: &mut AHashMap<String, usize>,
    line: &str,
    filtered_commands: &AHashSet<&str>,
    no_hist: bool,
) {
    if line.contains('|') && !no_hist {
        for subcommand in SplitCommands::new(line) {
            if let Some(first_word) = get_first_word(subcommand, filtered_commands) {
                increment_count(cmd_count, first_word);
            }
        }
    } else if let Some(first_word) = get_first_word(line, filtered_commands) {
        increment_count(cmd_count, first_word);
    }
}

#[inline]
fn increment_count(cmd_count: &mut AHashMap<String, usize>, first_word: &str) {
    if let Some(existing) = cmd_count.get_mut(first_word) {
        *existing += 1;
    } else {
        cmd_count.insert(first_word.to_string(), 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ahash::AHashMap;

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
