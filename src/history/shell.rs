//! Shell history parsing module

use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};

use crate::shared::command_parse::{clean_line, get_first_word};

/// Count commands from a history file
///
/// Returns a HashMap of command -> count
pub fn count_from_file(
    file_path: &str,
    ignore: &[String],
    no_hist: bool,
) -> Result<HashMap<String, usize>, std::io::Error> {
    let file = fs::File::open(file_path)?;
    let mut reader = BufReader::new(file);

    let mut filtered_commands: Vec<&str> = Vec::with_capacity(ignore.len() + 2);
    if !no_hist {
        filtered_commands.extend(["sudo", "doas"]);
    }

    // Add user-specified ignore commands
    let ignore_refs: Vec<&str> = ignore.iter().map(|s| s.as_str()).collect();
    filtered_commands.extend(ignore_refs);

    let mut skip = false;
    let mut cmd_count: HashMap<String, usize> = HashMap::with_capacity(256);
    let mut line_buf: Vec<u8> = Vec::with_capacity(256);

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

        // Handle zsh extended history format: ": timestamp:0;command"
        let is_zsh_extended = line.starts_with(": ");
        let actual_line = if is_zsh_extended {
            if let Some((_, cmd)) = line.split_once(';') {
                cmd
            } else {
                // Metadata line without command, skip
                skip = true;
                continue;
            }
        } else {
            line
        };

        match (skip, is_zsh_extended && !line.contains(';'), actual_line.ends_with('\\')) {
            (false, false, false) => {
                count_commands(&mut cmd_count, actual_line, &filtered_commands, no_hist);
            }
            (false, false, true) => {
                count_commands(&mut cmd_count, actual_line, &filtered_commands, no_hist);
                skip = true;
            }
            (false, true, _) => {
                skip = true;
            }
            (true, _, true) => {
                skip = true;
            }
            (true, _, false) => {
                skip = false;
            }
        }
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
    if line.contains("|") && !no_hist {
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
    } else {
        if let Some(first_word) = get_first_word(line, filtered_commands) {
            increment_count(cmd_count, first_word);
        }
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
    fn test_count_commands_simple() {
        let mut cmd_count = HashMap::new();
        let filters = vec!["sudo", "doas"];
        count_commands(&mut cmd_count, "ls -la", &filters, false);
        assert_eq!(cmd_count.get("ls"), Some(&1));
    }

    #[test]
    fn test_count_commands_with_pipe() {
        let mut cmd_count = HashMap::new();
        let filters = vec!["sudo", "doas"];
        count_commands(&mut cmd_count, "ls | grep foo", &filters, false);
        assert_eq!(cmd_count.get("ls"), Some(&1));
        assert_eq!(cmd_count.get("grep"), Some(&1));
    }
}
