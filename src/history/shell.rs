//! Shell history parsing module

use ahash::AHashMap;
use bstr::ByteSlice;
use memmap2::Mmap;
use std::fs;
use std::io::{BufRead, BufReader};

use crate::command::{CommandMode, CommandPolicy, FrequencyTable};

/// Count commands from a history file
///
/// Returns a HashMap of command -> count
pub fn count_from_file(
    file_path: &str,
    ignore: &[String],
    no_hist: bool,
) -> Result<AHashMap<String, usize>, std::io::Error> {
    let mode = if no_hist { CommandMode::Raw } else { CommandMode::History };
    let policy = CommandPolicy::new(ignore, mode);
    let mut frequencies = FrequencyTable::default();

    if file_path == "-" {
        let stdin = std::io::stdin();
        let reader = BufReader::new(stdin.lock());
        count_from_reader(reader, &mut frequencies, &policy, no_hist)?;
    } else {
        let file = fs::File::open(file_path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        count_from_bytes(&mmap, &mut frequencies, &policy, no_hist);
    }

    Ok(frequencies.into_counts())
}

fn count_from_bytes(
    bytes: &[u8],
    frequencies: &mut FrequencyTable,
    policy: &CommandPolicy<'_>,
    no_hist: bool,
) {
    let mut skip = false;
    for line_bytes in bstr::ByteSlice::lines(bytes) {
        let line = match line_bytes.to_str() {
            Ok(s) => trim_line_end(s),
            Err(_) => {
                continue;
            }
        };

        process_line(line, &mut skip, frequencies, policy, no_hist);
    }
}

fn count_from_reader<R: BufRead>(
    mut reader: R,
    frequencies: &mut FrequencyTable,
    policy: &CommandPolicy<'_>,
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

        let line = match line_buf.to_str() {
            Ok(s) => trim_line_end(s),
            Err(_) => continue,
        };

        process_line(line, &mut skip, frequencies, policy, no_hist);
    }

    Ok(())
}

fn process_line(
    trimmed_line: &str,
    skip: &mut bool,
    frequencies: &mut FrequencyTable,
    policy: &CommandPolicy<'_>,
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
            frequencies.record_line(actual_line, policy);
        }
        (false, false, true) => {
            frequencies.record_line(actual_line, policy);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_commands_simple() {
        let ignored = Vec::new();
        let policy = CommandPolicy::new(&ignored, CommandMode::History);
        let mut frequencies = FrequencyTable::default();
        frequencies.record_line("ls -la", &policy);
        assert_eq!(frequencies.into_counts().get("ls"), Some(&1));
    }

    #[test]
    fn test_count_commands_with_pipe() {
        let ignored = Vec::new();
        let policy = CommandPolicy::new(&ignored, CommandMode::History);
        let mut frequencies = FrequencyTable::default();
        frequencies.record_line("ls | grep foo", &policy);
        let counts = frequencies.into_counts();
        assert_eq!(counts.get("ls"), Some(&1));
        assert_eq!(counts.get("grep"), Some(&1));
    }
}
