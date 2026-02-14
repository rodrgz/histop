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
