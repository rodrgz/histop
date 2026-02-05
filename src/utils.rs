//! Shared utilities for command parsing and processing.

/// Commands that support subcommand tracking.
/// When subcommand mode is enabled, we track "git status" instead of just "git".
pub const SUBCOMMAND_TOOLS: &[&str] = &[
    "git", "cargo", "npm", "yarn", "pnpm", "docker", "kubectl", "systemctl",
    "apt", "dnf", "pacman", "brew", "nix", "rustup", "go", "pip", "poetry",
];

/// Extract the first meaningful word(s) from a command.
///
/// # Arguments
/// * `cmd` - The command string to parse
/// * `filtered` - Commands to skip (like sudo, doas)
/// * `track_subcommands` - If true, include subcommand for known tools
///
/// # Returns
/// The first command word (or command + subcommand if tracking)
pub fn get_first_word(
    cmd: &str,
    filtered: &[&str],
    track_subcommands: bool,
) -> String {
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

        // Check if we should track subcommand
        if track_subcommands && SUBCOMMAND_TOOLS.contains(&word) {
            if let Some(sub) = words.next() {
                // Skip flags as subcommands
                if !sub.starts_with('-') {
                    return format!("{} {}", word, sub);
                }
            }
        }
        return word.to_string();
    }

    String::new()
}

/// Clean a command line by replacing pipes inside quotes with spaces.
///
/// This allows proper splitting of piped commands while preserving
/// pipes that are part of string arguments.
pub fn clean_line(line: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_first_word_simple() {
        let filters = vec!["sudo", "doas"];
        assert_eq!(get_first_word("ls -la", &filters, false), "ls");
    }

    #[test]
    fn test_get_first_word_with_sudo() {
        let filters = vec!["sudo", "doas"];
        assert_eq!(get_first_word("sudo apt update", &filters, false), "apt");
    }

    #[test]
    fn test_get_first_word_with_doas() {
        let filters = vec!["sudo", "doas"];
        assert_eq!(get_first_word("doas pacman -S vim", &filters, false), "pacman");
    }

    #[test]
    fn test_get_first_word_env_var_prefix() {
        let filters = vec![];
        assert_eq!(get_first_word("FOO=bar cmd arg", &filters, false), "cmd");
    }

    #[test]
    fn test_get_first_word_escaped_command() {
        let filters = vec![];
        assert_eq!(get_first_word("\\ls -la", &filters, false), "ls");
    }

    #[test]
    fn test_get_first_word_escaped_filtered() {
        let filters = vec!["sudo"];
        assert_eq!(get_first_word("\\sudo apt", &filters, false), "apt");
    }

    #[test]
    fn test_get_first_word_empty() {
        let filters: Vec<&str> = vec![];
        assert_eq!(get_first_word("", &filters, false), "");
    }

    #[test]
    fn test_get_first_word_whitespace_only() {
        let filters: Vec<&str> = vec![];
        assert_eq!(get_first_word("   ", &filters, false), "");
    }

    #[test]
    fn test_get_first_word_preserves_expansion() {
        let filters: Vec<&str> = vec![];
        assert_eq!(get_first_word("$EDITOR file.txt", &filters, false), "$EDITOR");
    }

    #[test]
    fn test_subcommand_tracking_git() {
        let filters: Vec<&str> = vec![];
        assert_eq!(get_first_word("git status", &filters, true), "git status");
    }

    #[test]
    fn test_subcommand_tracking_git_with_flag() {
        let filters: Vec<&str> = vec![];
        // Flags are not subcommands
        assert_eq!(get_first_word("git -v", &filters, true), "git");
    }

    #[test]
    fn test_subcommand_tracking_cargo() {
        let filters: Vec<&str> = vec![];
        assert_eq!(get_first_word("cargo build --release", &filters, true), "cargo build");
    }

    #[test]
    fn test_subcommand_tracking_disabled() {
        let filters: Vec<&str> = vec![];
        assert_eq!(get_first_word("git status", &filters, false), "git");
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
