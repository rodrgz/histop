//! Shared line-by-line history parser for simple formats (PowerShell, tcsh).
//!
//! Reads bytes and skips invalid UTF-8 lines instead of aborting the
//! entire file, matching the behavior of `shell.rs` and `fish.rs`.

use ahash::{AHashMap, AHashSet};
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

use bstr::ByteSlice;
use memmap2::Mmap;

use crate::shared::command_parse::{SplitCommands, get_first_word};

/// Count commands from a history file, skipping lines that fail the
/// provided `skip_line` predicate or contain invalid UTF-8.
pub fn count_from_file<F>(
    file_path: &str,
    ignore: &[String],
    no_hist: bool,
    skip_line: F,
) -> Result<AHashMap<String, usize>, std::io::Error>
where
    F: Fn(&str) -> bool,
{
    let mut cmd_count = AHashMap::default();

    let mut filtered_commands = AHashSet::with_capacity(ignore.len() + 2);
    if !no_hist {
        filtered_commands.insert("sudo");
        filtered_commands.insert("doas");
    }
    for s in ignore {
        filtered_commands.insert(s.as_str());
    }

    if file_path == "-" {
        let stdin = std::io::stdin();
        let mut reader = BufReader::new(stdin.lock());
        let mut line_buf = Vec::with_capacity(256);
        loop {
            line_buf.clear();
            let bytes_read = reader.read_until(b'\n', &mut line_buf)?;
            if bytes_read == 0 {
                break;
            }
            let line = match line_buf.to_str() {
                Ok(s) => trim_line_end(s),
                Err(_) => continue,
            };
            if line.trim().is_empty() || skip_line(line) {
                continue;
            }
            count_commands(&mut cmd_count, line, &filtered_commands, no_hist);
        }
    } else {
        let file = File::open(file_path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        for line_bytes in bstr::ByteSlice::lines(&*mmap) {
            let line = match line_bytes.to_str() {
                Ok(s) => trim_line_end(s),
                Err(_) => continue,
            };

            if line.trim().is_empty() || skip_line(line) {
                continue;
            }

            count_commands(&mut cmd_count, line, &filtered_commands, no_hist);
        }
    }

    Ok(cmd_count)
}

#[inline]
pub(super) fn trim_line_end(line: &str) -> &str {
    line.trim_end_matches(['\n', '\r'])
}

pub(super) fn count_commands(
    cmd_count: &mut AHashMap<String, usize>,
    line: &str,
    filtered_commands: &AHashSet<&str>,
    no_hist: bool,
) {
    if !no_hist && line.as_bytes().find_byte(b'|').is_some() {
        for subcommand in SplitCommands::new(line) {
            if let Some(first_word) =
                get_first_word(subcommand, filtered_commands)
            {
                increment_count(cmd_count, first_word);
            }
        }
    } else if let Some(first_word) = get_first_word(line, filtered_commands) {
        increment_count(cmd_count, first_word);
    }
}

#[inline]
fn increment_count(
    cmd_count: &mut AHashMap<String, usize>,
    first_word: &str,
) {
    if let Some(count) = cmd_count.get_mut(first_word) {
        *count += 1;
    } else {
        cmd_count.insert(first_word.to_string(), 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_invalid_utf8_skipped_not_fatal() {
        let now_nanos =
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!(
            "test_simple_history_utf8_{}_{}.txt",
            std::process::id(),
            now_nanos
        ));
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(b"ls -la\n").unwrap();
        file.write_all(b"\xFF\xFE invalid\n").unwrap();
        file.write_all(b"git status\n").unwrap();

        let result =
            count_from_file(path.to_str().unwrap(), &[], false, |_| false)
                .unwrap();
        assert_eq!(result.get("ls"), Some(&1));
        assert_eq!(result.get("git"), Some(&1));

        fs::remove_file(path).ok();
    }

    #[test]
    fn test_skip_line_predicate() {
        let now_nanos =
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!(
            "test_simple_history_skip_{}_{}.txt",
            std::process::id(),
            now_nanos
        ));
        let mut file = std::fs::File::create(&path).unwrap();
        writeln!(file, "#+12345").unwrap();
        writeln!(file, "ls -la").unwrap();
        writeln!(file, "#+67890").unwrap();
        writeln!(file, "git status").unwrap();

        let result =
            count_from_file(path.to_str().unwrap(), &[], false, |line| {
                line.trim().starts_with('#')
            })
            .unwrap();
        assert_eq!(result.get("ls"), Some(&1));
        assert_eq!(result.get("git"), Some(&1));

        fs::remove_file(path).ok();
    }
}
