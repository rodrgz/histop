//! Shared utilities for command parsing and processing.

use ahash::AHashSet;

/// Extract the first meaningful word(s) from a command.
///
/// # Arguments
/// * `cmd` - The command string to parse
/// * `filtered` - Set of commands to skip (like sudo, doas)
///
/// # Returns
/// The first command word, if one exists
#[inline]
pub fn get_first_word<'a>(cmd: &'a str, filtered: &AHashSet<&str>) -> Option<&'a str> {
    for w in cmd.split_whitespace() {
        // Skip end-of-options marker used in wrappers like sudo/doas
        if w == "--" {
            continue;
        }

        // Handle escaped commands (\ls -> ls) first
        // Also handle cases like \sudo, \\sudo, reboot\
        // We trim leading and trailing backslashes
        let clean_word = w.trim_matches('\\');

        // Handle cases where \ is used as a separator or artifact inside the word
        // e.g. cd\numount -> cd
        // e.g. lsblk\\\n -> lsblk (where \n is literal n)
        // We take the first component before any remaining backslash
        let first_component = clean_word.split('\\').next().unwrap_or(clean_word);
        
        // Handle paths (e.g. ./mvnw -> mvnw, /bin/ls -> ls)
        // We use rfind instead of Path::new to avoid overhead
        let command_name = if let Some(idx) = first_component.rfind(std::path::is_separator) {
            &first_component[idx + 1..]
        } else {
            first_component
        };

        if command_name.is_empty() {
            continue;
        }

        // Skip filtered commands (sudo, doas, etc.)
        if filtered.contains(command_name) {
            continue;
        }

        // Skip environment variable assignments (FOO=bar) but not expansions ($FOO)
        if !command_name.starts_with('$') && command_name.contains('=') {
            continue;
        }

        return Some(command_name);
    }

    None
}


/// Iterator that splits a command line by pipes `|`, respecting quotes.
///
/// This avoids allocating a new String just to mask pipes inside quotes.
pub struct SplitCommands<'a> {
    remaining: &'a str,
}

impl<'a> SplitCommands<'a> {
    pub fn new(line: &'a str) -> Self {
        Self { remaining: line }
    }
}

impl<'a> Iterator for SplitCommands<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_empty() {
            return None;
        }

        let mut in_single_quotes = false;
        let mut in_double_quotes = false;
        let mut split_idx = None;

        for (i, b) in self.remaining.bytes().enumerate() {
            match b {
                b'\'' => in_single_quotes = !in_single_quotes,
                b'"' => in_double_quotes = !in_double_quotes,
                b'|' if !in_single_quotes && !in_double_quotes => {
                    split_idx = Some(i);
                    break;
                }
                _ => {}
            }
        }

        if let Some(idx) = split_idx {
            let (chunk, rest) = self.remaining.split_at(idx);
            self.remaining = &rest[1..]; // Skip the pipe
            Some(chunk)
        } else {
            let chunk = self.remaining;
            self.remaining = "";
            Some(chunk)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_first_word_simple() {
        let filters = AHashSet::from_iter(vec!["sudo", "doas"]);
        assert_eq!(get_first_word("ls -la", &filters), Some("ls"));
    }

    #[test]
    fn test_get_first_word_with_sudo() {
        let filters = AHashSet::from_iter(vec!["sudo", "doas"]);
        assert_eq!(get_first_word("sudo apt update", &filters), Some("apt"));
    }

    #[test]
    fn test_get_first_word_with_doas() {
        let filters = AHashSet::from_iter(vec!["sudo", "doas"]);
        assert_eq!(get_first_word("doas pacman -S vim", &filters), Some("pacman"));
    }

    #[test]
    fn test_get_first_word_env_var_prefix() {
        let filters = AHashSet::new();
        assert_eq!(get_first_word("FOO=bar cmd arg", &filters), Some("cmd"));
    }

    #[test]
    fn test_get_first_word_escaped_command() {
        let filters = AHashSet::new();
        assert_eq!(get_first_word("\\ls -la", &filters), Some("ls"));
    }

    #[test]
    fn test_get_first_word_escaped_filtered() {
        let filters = AHashSet::from_iter(vec!["sudo"]);
        assert_eq!(get_first_word("\\sudo apt", &filters), Some("apt"));
    }

    #[test]
    fn test_get_first_word_path_normalization() {
        let filters = AHashSet::new();
        assert_eq!(get_first_word("./mvnw clean", &filters), Some("mvnw"));
        assert_eq!(get_first_word("/bin/ls -la", &filters), Some("ls"));
        assert_eq!(get_first_word("../scripts/deploy.sh", &filters), Some("deploy.sh"));
        assert_eq!(
            get_first_word("/nix/store/something/bin/grep", &filters),
            Some("grep")
        );
    }

    #[test]
    fn test_get_first_word_path_filtered() {
        let filters = AHashSet::from_iter(vec!["sudo", "grep"]);
        assert_eq!(get_first_word("/usr/bin/sudo apt", &filters), Some("apt"));
        // treating "grep" as a wrapper/ignored command means we get the argument
        assert_eq!(get_first_word("/bin/grep foo", &filters), Some("foo"));
    }

    #[test]
    fn test_get_first_word_aggressive_cleaning() {
        let filters = AHashSet::from_iter(vec!["sudo"]);
        // \sudo -> sudo (filtered) -> apt
        assert_eq!(get_first_word("\\sudo apt", &filters), Some("apt"));
        // \\sudo -> sudo (filtered) -> apt
        assert_eq!(get_first_word("\\\\sudo apt", &filters), Some("apt"));
        // reboot\ -> reboot
        assert_eq!(get_first_word("reboot\\", &filters), Some("reboot"));
        // lsblk\\\n -> lsblk (assuming \n is literal)
        assert_eq!(get_first_word("lsblk\\\\\\n", &filters), Some("lsblk"));
        // cd\numount -> cd
        assert_eq!(get_first_word("cd\\numount", &filters), Some("cd"));
        // cd\n -> cd
        assert_eq!(get_first_word("cd\\n", &filters), Some("cd"));
        // cd\\\n -> cd
        assert_eq!(get_first_word("cd\\\\\\n", &filters), Some("cd"));
        // \nsystemctl -> nsystemctl (trims leading \, then takes nsystemctl)
        assert_eq!(get_first_word("\\nsystemctl", &filters), Some("nsystemctl"));
    }

    #[test]
    fn test_get_first_word_empty() {
        let filters = AHashSet::new();
        assert_eq!(get_first_word("", &filters), None);
    }

    #[test]
    fn test_get_first_word_whitespace_only() {
        let filters = AHashSet::new();
        assert_eq!(get_first_word("   ", &filters), None);
    }

    #[test]
    fn test_get_first_word_preserves_expansion() {
        let filters = AHashSet::new();
        assert_eq!(get_first_word("$EDITOR file.txt", &filters), Some("$EDITOR"));
    }

    #[test]
    fn test_get_first_word_skips_double_dash_after_wrapper() {
        let filters = AHashSet::from_iter(vec!["sudo", "doas"]);
        assert_eq!(get_first_word("doas -- systemctl stop sshd", &filters), Some("systemctl"));
    }

    #[test]
    fn test_split_commands_no_pipe() {
        let parts: Vec<&str> = SplitCommands::new("ls -la").collect();
        assert_eq!(parts, vec!["ls -la"]);
    }

    #[test]
    fn test_split_commands_pipe_outside_quotes() {
        let parts: Vec<&str> = SplitCommands::new("ls | grep foo").collect();
        assert_eq!(parts, vec!["ls ", " grep foo"]);
    }

    #[test]
    fn test_split_commands_pipe_in_single_quotes() {
        let parts: Vec<&str> = SplitCommands::new("echo 'hello | world'").collect();
        assert_eq!(parts, vec!["echo 'hello | world'"]);
    }

    #[test]
    fn test_split_commands_pipe_in_double_quotes() {
        let parts: Vec<&str> = SplitCommands::new(r#"echo "hello | world""#).collect();
        assert_eq!(parts, vec![r#"echo "hello | world""#]);
    }

    #[test]
    fn test_split_commands_mixed() {
        let parts: Vec<&str> = SplitCommands::new("echo 'foo | bar' | grep baz").collect();
        assert_eq!(parts, vec!["echo 'foo | bar' ", " grep baz"]);
    }
}
