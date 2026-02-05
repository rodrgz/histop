//! Shell history parsing module

use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};

/// Count commands from a history file
///
/// Returns a HashMap of command -> count
pub fn count_from_file(
    file_path: &str,
    ignore: &[String],
    no_hist: bool,
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

        match (skip, line.starts_with(": "), line.ends_with("\\")) {
            (false, false, false) => {
                count_commands(&mut cmd_count, &line, &filtered_commands, no_hist);
            }
            (false, false, true) => {
                count_commands(&mut cmd_count, &line, &filtered_commands, no_hist);
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
                cmd_count
                    .entry(first_word.to_string())
                    .and_modify(|count| *count += 1)
                    .or_default();
            }
        }
    } else {
        let first_word = get_first_word(line, filtered_commands);
        if !first_word.is_empty() {
            cmd_count
                .entry(first_word.to_string())
                .and_modify(|count| *count += 1)
                .or_default();
        }
    }
}

fn clean_line(line: &str) -> String {
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;
    let mut cleaned_line = String::with_capacity(line.len());

    for c in line.chars() {
        match c {
            '\'' => {
                in_single_quotes = !in_single_quotes;
                cleaned_line.push(c);
            }
            '\"' => {
                in_double_quotes = !in_double_quotes;
                cleaned_line.push(c);
            }
            '|' if in_single_quotes || in_double_quotes => {
                cleaned_line.push(' ');
            }
            _ => {
                cleaned_line.push(c);
            }
        }
    }

    cleaned_line
}

fn get_first_word<'a>(subcommand: &'a str, filtered_commands: &[&str]) -> &'a str {
    for w in subcommand.trim().split_whitespace() {
        if filtered_commands.contains(&w) || w.contains('=') {
            continue;
        } else if w.starts_with('\\') && w.len() > 1 {
            if filtered_commands.contains(&&w[1..]) {
                continue;
            } else {
                return &w[1..];
            }
        } else {
            return w;
        }
    }
    ""
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_first_word_simple() {
        let filters = vec!["sudo", "doas"];
        assert_eq!(get_first_word("ls -la", &filters), "ls");
    }

    #[test]
    fn test_get_first_word_with_sudo() {
        let filters = vec!["sudo", "doas"];
        assert_eq!(get_first_word("sudo apt update", &filters), "apt");
    }

    #[test]
    fn test_get_first_word_with_doas() {
        let filters = vec!["sudo", "doas"];
        assert_eq!(get_first_word("doas pacman -S vim", &filters), "pacman");
    }

    #[test]
    fn test_get_first_word_env_var_prefix() {
        let filters = vec![];
        assert_eq!(get_first_word("FOO=bar cmd arg", &filters), "cmd");
    }

    #[test]
    fn test_get_first_word_escaped_command() {
        let filters = vec![];
        assert_eq!(get_first_word("\\ls -la", &filters), "ls");
    }

    #[test]
    fn test_get_first_word_escaped_filtered() {
        let filters = vec!["sudo"];
        assert_eq!(get_first_word("\\sudo apt", &filters), "apt");
    }

    #[test]
    fn test_get_first_word_empty() {
        let filters = vec![];
        assert_eq!(get_first_word("", &filters), "");
    }

    #[test]
    fn test_get_first_word_whitespace_only() {
        let filters = vec![];
        assert_eq!(get_first_word("   ", &filters), "");
    }

    #[test]
    fn test_clean_line_no_pipe() {
        let result = clean_line("ls -la");
        assert_eq!(result, "ls -la");
    }

    #[test]
    fn test_clean_line_pipe_outside_quotes() {
        let result = clean_line("ls | grep foo");
        assert_eq!(result, "ls | grep foo");
    }

    #[test]
    fn test_clean_line_pipe_in_single_quotes() {
        let result = clean_line("echo 'hello | world'");
        assert!(!result.contains('|')); // pipe replaced with space
    }

    #[test]
    fn test_clean_line_pipe_in_double_quotes() {
        let result = clean_line(r#"echo "hello | world""#);
        assert!(!result.contains('|')); // pipe replaced with space
    }
}

