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
    track_subcommands: bool,
    verbose: bool,
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

    let (mut skip, mut non_utf8) = (false, false);
    let mut cmd_count: HashMap<String, usize> = HashMap::new();

    for line_result in reader.lines() {
        let line = match line_result {
            Ok(line) => line,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::InvalidData {
                    if !non_utf8 {
                        if verbose {
                            eprintln!(
                                "Non-UTF-8 character detected in input stream, skipping line"
                            );
                        }
                        non_utf8 = true;
                    }
                    continue;
                } else {
                    return Err(e);
                }
            }
        };

        // Handle zsh extended history format: ": timestamp:0;command"
        let actual_line = if line.starts_with(": ") {
            if let Some(idx) = line.find(';') {
                &line[idx + 1..]
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
                count_commands(&mut cmd_count, actual_line, &filtered_commands, no_hist, track_subcommands);
            }
            (false, false, true) => {
                count_commands(&mut cmd_count, actual_line, &filtered_commands, no_hist, track_subcommands);
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
    track_subcommands: bool,
) {
    if line.contains("|") && !no_hist {
        let cleaned_line = clean_line(line);
        for subcommand in cleaned_line.split('|') {
            let first_word = get_first_word(subcommand, filtered_commands, track_subcommands);
            if !first_word.is_empty() {
                *cmd_count.entry(first_word).or_default() += 1;
            }
        }
    } else {
        let first_word = get_first_word(line, filtered_commands, track_subcommands);
        if !first_word.is_empty() {
            *cmd_count.entry(first_word).or_default() += 1;
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
        count_commands(&mut cmd_count, "ls -la", &filters, false, false);
        assert_eq!(cmd_count.get("ls"), Some(&1));
    }

    #[test]
    fn test_count_commands_with_pipe() {
        let mut cmd_count = HashMap::new();
        let filters = vec!["sudo", "doas"];
        count_commands(&mut cmd_count, "ls | grep foo", &filters, false, false);
        assert_eq!(cmd_count.get("ls"), Some(&1));
        assert_eq!(cmd_count.get("grep"), Some(&1));
    }

    #[test]
    fn test_count_commands_with_subcommands() {
        let mut cmd_count = HashMap::new();
        let filters: Vec<&str> = vec![];
        count_commands(&mut cmd_count, "git status", &filters, false, true);
        assert_eq!(cmd_count.get("git status"), Some(&1));
    }
}
