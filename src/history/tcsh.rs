use ahash::{AHashMap, AHashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn count_from_file(
    file_path: &str,
    ignore: &[String],
    no_hist: bool,
) -> Result<AHashMap<String, usize>, std::io::Error> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut cmd_count = AHashMap::default();

    let mut filtered_commands = AHashSet::with_capacity(ignore.len() + 2);
    if !no_hist {
        filtered_commands.insert("sudo");
        filtered_commands.insert("doas");
    }
    for s in ignore {
        filtered_commands.insert(s.as_str());
    }

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        process_command(&mut cmd_count, &line, &filtered_commands, no_hist);
    }

    Ok(cmd_count)
}

fn process_command(
    cmd_count: &mut AHashMap<String, usize>,
    line: &str,
    filtered_commands: &AHashSet<&str>,
    no_hist: bool,
) {
    use crate::shared::command_parse::{SplitCommands, get_first_word};

    if line.contains('|') && !no_hist {
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

fn increment_count(
    cmd_count: &mut AHashMap<String, usize>,
    first_word: &str,
) {
    *cmd_count.entry(first_word.to_string()).or_insert(0) += 1;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_count_tcsh() {
        let now_nanos =
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!(
            "test_tcsh_{}_{}.history",
            std::process::id(),
            now_nanos
        ));
        let mut file = File::create(&path).unwrap();
        writeln!(file, "#+1680820391").unwrap();
        writeln!(file, "ls -la").unwrap();
        writeln!(file, "#+1680820392").unwrap();
        writeln!(file, "git status").unwrap();
        writeln!(file, "#+1680820393").unwrap();
        writeln!(file, "ls").unwrap();

        let result =
            count_from_file(path.to_str().unwrap(), &[], false).unwrap();
        assert_eq!(result.get("ls"), Some(&2));
        assert_eq!(result.get("git"), Some(&1));

        fs::remove_file(path).ok();
    }
}
