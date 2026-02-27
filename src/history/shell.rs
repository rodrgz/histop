//! Shell history parsing module

use ahash::{AHashMap, AHashSet};
use std::fs;
use std::io::{BufRead, BufReader};

use crate::shared::command_parse::{SplitCommands, get_first_word};

/// Count commands from a history file
///
/// Returns a HashMap of command -> count
pub fn count_from_file(
    file_path: &str,
    ignore: &[String],
    no_hist: bool,
) -> Result<AHashMap<String, usize>, std::io::Error> {
    // 256KB buffer size
    const BUFFER_SIZE: usize = 256 * 1024;

    let mut cmd_count: AHashMap<String, usize> = AHashMap::default();

    let mut filtered_commands: AHashSet<&str> =
        AHashSet::with_capacity(ignore.len() + 2);
    if !no_hist {
        filtered_commands.insert("sudo");
        filtered_commands.insert("doas");
    }
    for s in ignore {
        filtered_commands.insert(s.as_str());
    }

    if file_path == "-" {
        let stdin = std::io::stdin();
        let reader = BufReader::with_capacity(BUFFER_SIZE, stdin.lock());
        count_from_reader(reader, &mut cmd_count, &filtered_commands, no_hist)?;
    } else {
        let file = fs::File::open(file_path)?;
        let reader = BufReader::with_capacity(BUFFER_SIZE, file);
        count_from_reader(reader, &mut cmd_count, &filtered_commands, no_hist)?;
    }

    Ok(cmd_count)
}

fn count_from_reader<R: BufRead>(
    mut reader: R,
    cmd_count: &mut AHashMap<String, usize>,
    filtered_commands: &AHashSet<&str>,
    no_hist: bool,
) -> std::io::Result<()> {
    let mut skip = false;
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

        process_line(line, &mut skip, cmd_count, filtered_commands, no_hist);
    }

    Ok(())
}

fn process_line(
    trimmed_line: &str,
    skip: &mut bool,
    cmd_count: &mut AHashMap<String, usize>,
    filtered_commands: &AHashSet<&str>,
    no_hist: bool,
) {
    // Handle zsh extended history format: ": timestamp:0;command"
    // specific check for not no_hist, because we want to treat the file as raw if no_hist is true
    let is_zsh_extended = !no_hist && trimmed_line.starts_with(": ");
    let actual_line = if is_zsh_extended {
        if let Some((_, cmd)) = trimmed_line.split_once(';') {
            cmd
        } else {
            // Metadata line without command, skip
            *skip = true;
            return;
        }
    } else {
        trimmed_line
    };

    match (
        *skip,
        is_zsh_extended && !trimmed_line.contains(';'),
        !no_hist && actual_line.ends_with('\\'),
    ) {
        (false, false, false) => {
            count_commands(cmd_count, actual_line, filtered_commands, no_hist);
        }
        (false, false, true) => {
            count_commands(cmd_count, actual_line, filtered_commands, no_hist);
            *skip = true;
        }
        (false, true, _) => {
            *skip = true;
        }
        (true, _, true) => {
            *skip = true;
        }
        (true, _, false) => {
            *skip = false;
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
    if no_hist {
        if let Some(first_word) = line.split_whitespace().next()
            && !filtered_commands.contains(first_word)
        {
            increment_count(cmd_count, first_word);
        }
        return;
    }

    if line.contains('|') {
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
    fn test_count_commands_simple() {
        let mut cmd_count = AHashMap::default();
        let filters = AHashSet::from_iter(vec!["sudo", "doas"]);
        count_commands(&mut cmd_count, "ls -la", &filters, false);
        assert_eq!(cmd_count.get("ls"), Some(&1));
    }

    #[test]
    fn test_count_commands_with_pipe() {
        let mut cmd_count = AHashMap::default();
        let filters = AHashSet::from_iter(vec!["sudo", "doas"]);
        count_commands(&mut cmd_count, "ls | grep foo", &filters, false);
        assert_eq!(cmd_count.get("ls"), Some(&1));
        assert_eq!(cmd_count.get("grep"), Some(&1));
    }
}
