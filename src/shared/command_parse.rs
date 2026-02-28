//! Shared utilities for command parsing and processing.

use ahash::AHashSet;
use bstr::ByteSlice;

/// Extract the first meaningful word(s) from a command.
///
/// # Arguments
/// * `cmd` - The command string to parse
/// * `filtered` - Set of commands to skip (like sudo, doas)
///
/// # Returns
/// The first command word, if one exists
#[inline]
pub fn get_first_word<'a>(
    cmd: &'a str,
    filtered: &AHashSet<&str>,
) -> Option<&'a str> {
    let bytes = cmd.as_bytes();
    let mut i = 0;
    let len = bytes.len();

    while i < len {
        // Skip whitespace efficiently
        while i < len && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= len {
            break;
        }

        let start = i;
        while i < len && !bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        let w = &cmd[start..i];

        // 1. Skip end-of-options marker (O(1))
        if w == "--" {
            continue;
        }

        // 2. Skip comments (O(1)) - check raw word first
        if w.starts_with('#') {
            return None;
        }

        // 3. Skip flags (O(1)) - check raw word first
        if w.starts_with('-') {
            continue;
        }

        // 4. Handle escaped commands (\ls -> ls) and separators
        // We do this BEFORE filtered check to handle \sudo
        let clean_word = w.trim_matches('\\');

        // Handle cases like cd\numount -> cd
        let first_component =
            if let Some(idx) = clean_word.as_bytes().find_byte(b'\\') {
                &clean_word[..idx]
            } else {
                clean_word
            };

        // Handle paths (e.g. /bin/ls -> ls)
        let command_name =
            if let Some(idx) = first_component.rfind(std::path::is_separator) {
                &first_component[idx + 1..]
            } else {
                first_component
            };

        if command_name.is_empty() {
            continue;
        }

        // Keep behavior for escaped comments/flags (\#comment, \-f)
        if command_name.starts_with('#') {
            return None;
        }
        if command_name.starts_with('-') {
            continue;
        }

        // 5. Skip filtered commands (sudo, doas, etc.) - after cleaning
        if filtered.contains(command_name) {
            continue;
        }

        // 6. Skip environment variable assignments (FOO=bar) - O(n) check last
        if !command_name.starts_with('$')
            && command_name.as_bytes().find_byte(b'=').is_some()
        {
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
        assert_eq!(
            get_first_word("doas pacman -S vpm", &filters),
            Some("pacman")
        );
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
        assert_eq!(
            get_first_word("../scripts/deploy.sh", &filters),
            Some("deploy.sh")
        );
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
        assert_eq!(
            get_first_word("\\nsystemctl", &filters),
            Some("nsystemctl")
        );
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
    fn test_get_first_word_ignores_comments_and_flags() {
        let filters = AHashSet::from_iter(vec!["sudo"]);

        // Comment should be ignored if it appears as the command
        assert_eq!(get_first_word("# comment", &filters), None);
        assert_eq!(get_first_word("   #indented comment", &filters), None);

        // Flags should be ignored if they appear as the command (e.g. after filtered command)
        assert_eq!(get_first_word("-d", &filters), None);
        assert_eq!(get_first_word("--flag", &filters), None);

        // sudo -i -> -i is a flag, should be ignored?
        // If the user runs `sudo -i`, we probably want to ignore `-i` and return None or look for next word?
        // Current behavior for `sudo -i` would be `Some("-i")` if not filtered.
        // User says "-d" and "-I" are showing up, likely from things like `sudo -i` or just `-d` on a line?
        // Let's assume we want to skip ANY word starting with `-` at the start of command resolution.
        assert_eq!(get_first_word("sudo -i", &filters), None);
        // Simple parser sees "user" as the next word after skipping -u.
        // This is acceptable as it filters the flag itself.
        assert_eq!(get_first_word("sudo -u user id", &filters), Some("user"));
    }

    #[test]
    fn test_get_first_word_escaped_comment_and_flag() {
        let filters = AHashSet::from_iter(vec!["sudo"]);
        assert_eq!(get_first_word("\\#comment", &filters), None);
        assert_eq!(get_first_word("\\-f", &filters), None);
    }

    #[test]
    fn test_get_first_word_skips_double_dash_after_wrapper() {
        let filters = AHashSet::from_iter(vec!["sudo", "doas"]);
        assert_eq!(
            get_first_word("doas -- systemctl stop sshd", &filters),
            Some("systemctl")
        );
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
        let parts: Vec<&str> =
            SplitCommands::new("echo 'hello | world'").collect();
        assert_eq!(parts, vec!["echo 'hello | world'"]);
    }

    #[test]
    fn test_split_commands_pipe_in_double_quotes() {
        let parts: Vec<&str> =
            SplitCommands::new(r#"echo "hello | world""#).collect();
        assert_eq!(parts, vec![r#"echo "hello | world""#]);
    }

    #[test]
    fn test_split_commands_mixed() {
        let parts: Vec<&str> =
            SplitCommands::new("echo 'foo | bar' | grep baz").collect();
        assert_eq!(parts, vec!["echo 'foo | bar' ", " grep baz"]);
    }
}
