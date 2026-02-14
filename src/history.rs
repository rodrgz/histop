//! Shell history parsing module

use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};

use crate::utils::{clean_line, get_first_word};

/// Count commands from a history file
///
/// Returns a HashMap of command -> count
pub fn count_from_file(
    file_path: &str,
    ignore: &[String],
    no_hist: bool,
) -> Result<HashMap<String, usize>, std::io::Error> {
    let file = fs::File::open(file_path)?;
    let reader = BufReader::new(file);

    let mut filtered_commands: Vec<&str> = Vec::new();
    if !no_hist {
        filtered_commands.extend(["sudo", "doas"]);
    }

    // Add user-specified ignore commands
    let ignore_refs: Vec<&str> = ignore.iter().map(|s| s.as_str()).collect();
    filtered_commands.extend(ignore_refs);

    let mut skip = false;
    let mut cmd_count: HashMap<String, usize> = HashMap::new();

    for line_result in reader.lines() {
        let line = match line_result {
            Ok(line) => line,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::InvalidData {
                    continue;
                } else {
                    return Err(e);
                }
            }
        };

        // Handle zsh extended history format: ": timestamp:0;command"
        let actual_line = if line.starts_with(": ") {
            if let Some((_, cmd)) = line.split_once(';') {
                cmd
            } else {
                // Metadata line without command, skip
                skip = true;
                continue;
            }
        } else {
            line.as_str()
        };

        match (skip, line.starts_with(": ") && !line.contains(';'), line.ends_with("\\")) {
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

fn count_commands(
    cmd_count: &mut HashMap<String, usize>,
    line: &str,
    filtered_commands: &[&str],
    no_hist: bool,
) {
    if line.contains("|") && !no_hist {
        let cleaned_line = clean_line(line);
        for subcommand in cleaned_line.split('|') {
            let first_word = get_first_word(subcommand, filtered_commands);
            if !first_word.is_empty() {
                *cmd_count.entry(first_word.into_owned()).or_default() += 1;
            }
        }
    } else {
        let first_word = get_first_word(line, filtered_commands);
        if !first_word.is_empty() {
            *cmd_count.entry(first_word.into_owned()).or_default() += 1;
        }
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
