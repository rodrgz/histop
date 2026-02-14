//! Shared utilities for command parsing and processing.

use std::borrow::Cow;

/// Extract the first meaningful word(s) from a command.
///
/// # Arguments
/// * `cmd` - The command string to parse
/// * `filtered` - Commands to skip (like sudo, doas)
///
/// # Returns
/// The first command word
#[inline]
pub fn get_first_word<'a>(cmd: &'a str, filtered: &[&str]) -> Cow<'a, str> {
    let mut words = cmd.split_whitespace().peekable();

    while let Some(w) = words.next() {
        // Skip filtered commands (sudo, doas, etc.)
        if filtered.contains(&w) {
            continue;
        }

        // Skip environment variable assignments (FOO=bar) but not expansions ($FOO)
        if w.contains('=') && !w.starts_with('$') {
            continue;
        }

        // Handle escaped commands (\ls -> ls)
        let word = if w.starts_with('\\') && w.len() > 1 {
            let unescaped = &w[1..];
            if filtered.contains(&unescaped) {
                continue;
            }
            unescaped
        } else {
            w
        };

        return Cow::Borrowed(word);
    }

    Cow::Borrowed("")
}

/// Clean a command line by replacing pipes inside quotes with spaces.
///
/// This allows proper splitting of piped commands while preserving
/// pipes that are part of string arguments.
#[inline]
pub fn clean_line(line: &str) -> String {
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;
    let mut cleaned_line = String::with_capacity(line.len());

    // Use bytes() for faster ASCII scanning (quotes and pipes are ASCII)
    for b in line.bytes() {
        match b {
            b'\'' => {
                in_single_quotes = !in_single_quotes;
                cleaned_line.push(b as char);
            }
            b'"' => {
                in_double_quotes = !in_double_quotes;
                cleaned_line.push(b as char);
            }
            b'|' if in_single_quotes || in_double_quotes => {
                cleaned_line.push(' ');
            }
            _ => {
                cleaned_line.push(b as char);
            }
        }
    }

    cleaned_line
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
        let filters: Vec<&str> = vec![];
        assert_eq!(get_first_word("", &filters), "");
    }

    #[test]
    fn test_get_first_word_whitespace_only() {
        let filters: Vec<&str> = vec![];
        assert_eq!(get_first_word("   ", &filters), "");
    }

    #[test]
    fn test_get_first_word_preserves_expansion() {
        let filters: Vec<&str> = vec![];
        assert_eq!(get_first_word("$EDITOR file.txt", &filters), "$EDITOR");
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
        assert!(!result.contains('|'));
    }

    #[test]
    fn test_clean_line_pipe_in_double_quotes() {
        let result = clean_line(r#"echo "hello | world""#);
        assert!(!result.contains('|'));
    }
}
