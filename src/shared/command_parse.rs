//! Shared utilities for command parsing and processing.

/// Extract the first meaningful word(s) from a command.
///
/// # Arguments
/// * `cmd` - The command string to parse
/// * `filtered` - Commands to skip (like sudo, doas)
///
/// # Returns
/// The first command word, if one exists
#[inline]
pub fn get_first_word<'a>(cmd: &'a str, filtered: &[&str]) -> Option<&'a str> {
    if filtered.is_empty() {
        for w in cmd.split_whitespace() {
            // Skip end-of-options marker used in wrappers like sudo/doas
            if w == "--" {
                continue;
            }

            // Skip environment variable assignments (FOO=bar) but not expansions ($FOO)
            if w.contains('=') && !w.starts_with('$') {
                continue;
            }

            if let Some(unescaped) = w.strip_prefix('\\') {
                if !unescaped.is_empty() {
                    return Some(unescaped);
                }
                continue;
            }

            return Some(w);
        }
        return None;
    }

    for w in cmd.split_whitespace() {
        // Skip filtered commands (sudo, doas, etc.)
        if w == "--" || filtered.contains(&w) {
            continue;
        }

        // Skip environment variable assignments (FOO=bar) but not expansions ($FOO)
        if w.contains('=') && !w.starts_with('$') {
            continue;
        }

        // Handle escaped commands (\ls -> ls)
        if let Some(unescaped) = w.strip_prefix('\\') {
            if unescaped.is_empty() || filtered.contains(&unescaped) {
                continue;
            }
            return Some(unescaped);
        }

        return Some(w);
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
        let filters = vec!["sudo", "doas"];
        assert_eq!(get_first_word("ls -la", &filters), Some("ls"));
    }

    #[test]
    fn test_get_first_word_with_sudo() {
        let filters = vec!["sudo", "doas"];
        assert_eq!(get_first_word("sudo apt update", &filters), Some("apt"));
    }

    #[test]
    fn test_get_first_word_with_doas() {
        let filters = vec!["sudo", "doas"];
        assert_eq!(get_first_word("doas pacman -S vim", &filters), Some("pacman"));
    }

    #[test]
    fn test_get_first_word_env_var_prefix() {
        let filters = vec![];
        assert_eq!(get_first_word("FOO=bar cmd arg", &filters), Some("cmd"));
    }

    #[test]
    fn test_get_first_word_escaped_command() {
        let filters = vec![];
        assert_eq!(get_first_word("\\ls -la", &filters), Some("ls"));
    }

    #[test]
    fn test_get_first_word_escaped_filtered() {
        let filters = vec!["sudo"];
        assert_eq!(get_first_word("\\sudo apt", &filters), Some("apt"));
    }

    #[test]
    fn test_get_first_word_empty() {
        let filters: Vec<&str> = vec![];
        assert_eq!(get_first_word("", &filters), None);
    }

    #[test]
    fn test_get_first_word_whitespace_only() {
        let filters: Vec<&str> = vec![];
        assert_eq!(get_first_word("   ", &filters), None);
    }

    #[test]
    fn test_get_first_word_preserves_expansion() {
        let filters: Vec<&str> = vec![];
        assert_eq!(get_first_word("$EDITOR file.txt", &filters), Some("$EDITOR"));
    }

    #[test]
    fn test_get_first_word_skips_double_dash_after_wrapper() {
        let filters = vec!["sudo", "doas"];
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
