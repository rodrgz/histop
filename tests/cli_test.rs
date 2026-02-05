//! CLI integration tests for histop
//!
//! Tests all command-line options documented in `histop -h`

use std::path::PathBuf;
use std::process::Command;

/// Get the path to the histop binary
fn histop_bin() -> PathBuf {
    // CARGO_BIN_EXE_<name> is set by Cargo during integration tests
    // and points to the correct binary location in any build environment
    PathBuf::from(env!("CARGO_BIN_EXE_histop"))
}

/// Get the path to test fixtures
fn fixtures_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// Helper to run histop with given arguments
fn run_histop(args: &[&str]) -> std::process::Output {
    Command::new(histop_bin())
        .args(args)
        .output()
        .expect("Failed to execute histop")
}

mod help_flag {
    use super::*;

    #[test]
    fn test_help_short_flag() {
        let output = run_histop(&["-h"]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        assert!(output.status.success());
        assert!(stdout.contains("Usage: histop"));
        assert!(stdout.contains("-h, --help"));
        assert!(stdout.contains("-f <FILE>"));
        assert!(stdout.contains("-c <COUNT>"));
        assert!(stdout.contains("-a"));
        assert!(stdout.contains("-m <MORE_THAN>"));
        assert!(stdout.contains("-i <IGNORE>"));
        assert!(stdout.contains("-b <BAR_SIZE>"));
        assert!(stdout.contains("-n"));
        assert!(stdout.contains("-nh"));
        assert!(stdout.contains("-np"));
        assert!(stdout.contains("-nc"));
        assert!(stdout.contains("-v"));
        assert!(stdout.contains("-F"));
        assert!(stdout.contains("-s, --subcommands"));
        assert!(stdout.contains("-o, --output"));
        assert!(stdout.contains("--color"));
        assert!(stdout.contains("--config"));
    }

    #[test]
    fn test_help_long_flag() {
        let output = run_histop(&["--help"]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        assert!(output.status.success());
        assert!(stdout.contains("Usage: histop"));
    }
}

mod file_flag {
    use super::*;

    #[test]
    fn test_file_flag_with_bash_history() {
        let path = fixtures_path().join("bash_history");
        let output = run_histop(&["-f", path.to_str().unwrap()]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        assert!(output.status.success());
        // Should show commands from the history file
        assert!(stdout.contains("git") || stdout.contains("ls"));
    }

    #[test]
    fn test_file_flag_with_zsh_history() {
        let path = fixtures_path().join("zsh_history");
        let output = run_histop(&["-f", path.to_str().unwrap()]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        assert!(output.status.success());
        assert!(stdout.contains("git") || stdout.contains("ls"));
    }

    #[test]
    fn test_file_flag_with_nonexistent_file() {
        let output = run_histop(&["-f", "/nonexistent/path/to/history"]);
        
        assert!(!output.status.success());
    }
}

mod count_flag {
    use super::*;

    #[test]
    fn test_count_flag_limits_output() {
        let path = fixtures_path().join("bash_history");
        let output = run_histop(&["-f", path.to_str().unwrap(), "-c", "3"]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        assert!(output.status.success());
        // Count the number of output lines (excluding empty lines)
        let lines: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
        assert!(lines.len() <= 3, "Expected at most 3 lines, got {}", lines.len());
    }

    #[test]
    fn test_count_flag_with_one() {
        let path = fixtures_path().join("bash_history");
        let output = run_histop(&["-f", path.to_str().unwrap(), "-c", "1"]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        assert!(output.status.success());
        let lines: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
        assert_eq!(lines.len(), 1, "Expected 1 line, got {}", lines.len());
    }

    #[test]
    fn test_count_flag_invalid_value() {
        let path = fixtures_path().join("bash_history");
        let output = run_histop(&["-f", path.to_str().unwrap(), "-c", "invalid"]);
        
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Invalid") || stderr.contains("must be a positive integer"));
    }

    #[test]
    fn test_count_flag_zero_value() {
        let path = fixtures_path().join("bash_history");
        let output = run_histop(&["-f", path.to_str().unwrap(), "-c", "0"]);
        
        assert!(!output.status.success());
    }
}

mod all_flag {
    use super::*;

    #[test]
    fn test_all_flag_shows_all_commands() {
        let path = fixtures_path().join("bash_history");
        
        // Run with -a (all) flag
        let output_all = run_histop(&["-f", path.to_str().unwrap(), "-a"]);
        let stdout_all = String::from_utf8_lossy(&output_all.stdout);
        
        // Run with default count
        let output_default = run_histop(&["-f", path.to_str().unwrap()]);
        let stdout_default = String::from_utf8_lossy(&output_default.stdout);
        
        assert!(output_all.status.success());
        
        // -a should show at least as many lines as default
        let lines_all: Vec<&str> = stdout_all.lines().filter(|l| !l.is_empty()).collect();
        let lines_default: Vec<&str> = stdout_default.lines().filter(|l| !l.is_empty()).collect();
        
        assert!(lines_all.len() >= lines_default.len());
    }
}

mod more_than_flag {
    use super::*;

    #[test]
    fn test_more_than_flag_filters_commands() {
        let path = fixtures_path().join("bash_history");
        
        // Run without filter
        let output_all = run_histop(&["-f", path.to_str().unwrap(), "-a"]);
        
        // Run with -m 2 (only commands used more than 2 times)
        let output_filtered = run_histop(&["-f", path.to_str().unwrap(), "-a", "-m", "2"]);
        
        assert!(output_all.status.success());
        assert!(output_filtered.status.success());
        
        let stdout_all = String::from_utf8_lossy(&output_all.stdout);
        let lines_all: Vec<&str> = stdout_all
            .lines()
            .filter(|l| !l.is_empty())
            .collect();
        
        let stdout_filtered = String::from_utf8_lossy(&output_filtered.stdout);
        let lines_filtered: Vec<&str> = stdout_filtered
            .lines()
            .filter(|l| !l.is_empty())
            .collect();
        
        // Filtered should have fewer or equal commands
        assert!(lines_filtered.len() <= lines_all.len());
    }

    #[test]
    fn test_more_than_flag_with_high_threshold() {
        let path = fixtures_path().join("bash_history");
        // Set threshold higher than any command count
        let output = run_histop(&["-f", path.to_str().unwrap(), "-m", "1000"]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        assert!(output.status.success());
        // Should have no output or minimal output
        let lines: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
        assert!(lines.is_empty(), "Expected no output with high threshold");
    }
}

mod ignore_flag {
    use super::*;

    #[test]
    fn test_ignore_single_command() {
        let path = fixtures_path().join("bash_history");
        let output = run_histop(&["-f", path.to_str().unwrap(), "-a", "-i", "ls"]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        assert!(output.status.success());
        // "ls" should not appear in output
        assert!(!stdout.lines().any(|line| {
            // Check if "ls" appears as a command (at the end of line, not as part of another command)
            line.trim().ends_with(" ls") || line.trim().ends_with("\tls")
        }));
    }

    #[test]
    fn test_ignore_multiple_commands() {
        let path = fixtures_path().join("bash_history");
        let output = run_histop(&["-f", path.to_str().unwrap(), "-a", "-i", "ls|cd|git"]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        assert!(output.status.success());
        // None of the ignored commands should appear
        for cmd in ["ls", "cd", "git"] {
            assert!(!stdout.lines().any(|line| line.trim().ends_with(&format!(" {}", cmd))));
        }
    }
}

mod bar_size_flag {
    use super::*;

    #[test]
    fn test_bar_size_flag() {
        let path = fixtures_path().join("bash_history");
        let output = run_histop(&["-f", path.to_str().unwrap(), "-b", "10", "-c", "1"]);
        
        assert!(output.status.success());
        // The bar should be rendered (contains bar characters)
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("│") || stdout.contains("█") || stdout.contains("░"));
    }

    #[test]
    fn test_bar_size_invalid() {
        let path = fixtures_path().join("bash_history");
        let output = run_histop(&["-f", path.to_str().unwrap(), "-b", "invalid"]);
        
        assert!(!output.status.success());
    }
}

mod no_bar_flag {
    use super::*;

    #[test]
    fn test_no_bar_flag_hides_bar() {
        let path = fixtures_path().join("bash_history");
        let output = run_histop(&["-f", path.to_str().unwrap(), "-n", "-c", "3"]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        assert!(output.status.success());
        // Should not contain bar characters
        assert!(!stdout.contains("█"));
        assert!(!stdout.contains("░"));
        assert!(!stdout.contains("▓"));
    }
}

mod no_hist_flag {
    use super::*;

    #[test]
    fn test_no_hist_flag() {
        let path = fixtures_path().join("bash_history");
        // -nh disables history mode (doesn't filter sudo/doas)
        let output = run_histop(&["-f", path.to_str().unwrap(), "-nh", "-a"]);
        
        assert!(output.status.success());
    }
}

mod no_percentage_flags {
    use super::*;

    #[test]
    fn test_no_percentage_flag() {
        let path = fixtures_path().join("bash_history");
        let output = run_histop(&["-f", path.to_str().unwrap(), "-np", "-c", "3"]);
        
        assert!(output.status.success());
        // Output should still work without percentage in bar
    }

    #[test]
    fn test_no_cumulative_flag() {
        let path = fixtures_path().join("bash_history");
        let output = run_histop(&["-f", path.to_str().unwrap(), "-nc", "-c", "3"]);
        
        assert!(output.status.success());
    }

    #[test]
    fn test_both_no_percentage_flags() {
        let path = fixtures_path().join("bash_history");
        let output = run_histop(&["-f", path.to_str().unwrap(), "-np", "-nc", "-c", "3"]);
        
        assert!(output.status.success());
    }
}

mod verbose_flag {
    use super::*;

    #[test]
    fn test_verbose_flag() {
        let path = fixtures_path().join("bash_history");
        let output = run_histop(&["-f", path.to_str().unwrap(), "-v", "-c", "3"]);
        
        assert!(output.status.success());
    }
}

mod fish_format_flag {
    use super::*;

    #[test]
    fn test_fish_format_flag() {
        let path = fixtures_path().join("fish_history");
        let output = run_histop(&["-f", path.to_str().unwrap(), "-F"]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        assert!(output.status.success());
        // Should parse fish history and show commands
        assert!(stdout.contains("git") || stdout.contains("ls"));
    }

    #[test]
    fn test_fish_format_with_non_fish_file() {
        let path = fixtures_path().join("bash_history");
        // Forcing fish format on bash history should still not crash
        let output = run_histop(&["-f", path.to_str().unwrap(), "-F"]);
        
        // Should not crash, but may produce different results
        assert!(output.status.success());
    }
}

mod subcommands_flag {
    use super::*;

    #[test]
    fn test_subcommands_short_flag() {
        let path = fixtures_path().join("bash_history");
        let output = run_histop(&["-f", path.to_str().unwrap(), "-s", "-a"]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        assert!(output.status.success());
        // With subcommand tracking, should show "git status" instead of just "git"
        assert!(stdout.contains("git status") || stdout.contains("git commit") || stdout.contains("cargo build"));
    }

    #[test]
    fn test_subcommands_long_flag() {
        let path = fixtures_path().join("bash_history");
        let output = run_histop(&["-f", path.to_str().unwrap(), "--subcommands", "-a"]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        assert!(output.status.success());
        assert!(stdout.contains("git status") || stdout.contains("git commit") || stdout.contains("cargo build"));
    }
}

mod output_format_flag {
    use super::*;

    #[test]
    fn test_output_json_short() {
        let path = fixtures_path().join("bash_history");
        let output = run_histop(&["-f", path.to_str().unwrap(), "-o", "json", "-c", "3"]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        assert!(output.status.success());
        // JSON output should contain JSON structure
        assert!(stdout.contains("["));
        assert!(stdout.contains("]"));
        assert!(stdout.contains("\"command\":"));
        assert!(stdout.contains("\"count\":"));
        assert!(stdout.contains("\"percentage\":"));
    }

    #[test]
    fn test_output_json_long() {
        let path = fixtures_path().join("bash_history");
        let output = run_histop(&["-f", path.to_str().unwrap(), "--output", "json", "-c", "3"]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        assert!(output.status.success());
        assert!(stdout.contains("\"command\":"));
    }

    #[test]
    fn test_output_csv() {
        let path = fixtures_path().join("bash_history");
        let output = run_histop(&["-f", path.to_str().unwrap(), "-o", "csv", "-c", "3"]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        assert!(output.status.success());
        // CSV should have header
        assert!(stdout.contains("command,count,percentage"));
    }

    #[test]
    fn test_output_text_explicit() {
        let path = fixtures_path().join("bash_history");
        let output = run_histop(&["-f", path.to_str().unwrap(), "-o", "text", "-c", "3"]);
        
        assert!(output.status.success());
    }

    #[test]
    fn test_output_invalid_format() {
        let path = fixtures_path().join("bash_history");
        let output = run_histop(&["-f", path.to_str().unwrap(), "-o", "invalid"]);
        
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Invalid") || stderr.contains("output format"));
    }
}

mod color_flag {
    use super::*;

    #[test]
    fn test_color_always() {
        let path = fixtures_path().join("bash_history");
        let output = run_histop(&["-f", path.to_str().unwrap(), "--color", "always", "-c", "3"]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        assert!(output.status.success());
        // With color always, should contain ANSI escape codes
        assert!(stdout.contains("\x1b[") || stdout.contains("\u{1b}["));
    }

    #[test]
    fn test_color_never() {
        let path = fixtures_path().join("bash_history");
        let output = run_histop(&["-f", path.to_str().unwrap(), "--color", "never", "-c", "3"]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        assert!(output.status.success());
        // With color never, should not contain ANSI escape codes
        assert!(!stdout.contains("\x1b["));
    }

    #[test]
    fn test_color_auto() {
        let path = fixtures_path().join("bash_history");
        let output = run_histop(&["-f", path.to_str().unwrap(), "--color", "auto", "-c", "3"]);
        
        assert!(output.status.success());
        // Auto mode - when not in terminal, should not have colors
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(!stdout.contains("\x1b["));
    }

    #[test]
    fn test_color_invalid() {
        let path = fixtures_path().join("bash_history");
        let output = run_histop(&["-f", path.to_str().unwrap(), "--color", "invalid"]);
        
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Invalid") || stderr.contains("color mode"));
    }
}

mod config_flag {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_config_flag_with_valid_config() {
        let path = fixtures_path().join("bash_history");
        
        // Create a temporary config file
        let config_dir = std::env::temp_dir().join("histop_test_config");
        std::fs::create_dir_all(&config_dir).ok();
        let config_path = config_dir.join("test_config.toml");
        
        let mut file = std::fs::File::create(&config_path).unwrap();
        writeln!(file, "count = 2").unwrap();
        writeln!(file, "color = \"never\"").unwrap();
        
        let output = run_histop(&[
            "-f", path.to_str().unwrap(),
            "--config", config_path.to_str().unwrap()
        ]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        assert!(output.status.success());
        
        // Should respect config count = 2
        let lines: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
        assert!(lines.len() <= 2, "Expected at most 2 lines from config, got {}", lines.len());
        
        // Cleanup
        std::fs::remove_file(&config_path).ok();
    }

    #[test]
    fn test_config_flag_with_nonexistent_file() {
        let path = fixtures_path().join("bash_history");
        let output = run_histop(&[
            "-f", path.to_str().unwrap(),
            "--config", "/nonexistent/config.toml"
        ]);
        
        assert!(!output.status.success());
    }
}

mod invalid_options {
    use super::*;

    #[test]
    fn test_invalid_option() {
        let output = run_histop(&["--invalid-option"]);
        
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Invalid option"));
    }

    #[test]
    fn test_unknown_short_option() {
        let output = run_histop(&["-z"]);
        
        assert!(!output.status.success());
    }
}

mod combined_flags {
    use super::*;

    #[test]
    fn test_multiple_flags_combined() {
        let path = fixtures_path().join("bash_history");
        let output = run_histop(&[
            "-f", path.to_str().unwrap(),
            "-c", "5",
            "-s",
            "-n",
            "--color", "never"
        ]);
        
        assert!(output.status.success());
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
        
        assert!(lines.len() <= 5);
        // No bar characters (due to -n)
        assert!(!stdout.contains("█"));
        // No ANSI codes (due to --color never)
        assert!(!stdout.contains("\x1b["));
    }

    #[test]
    fn test_json_output_with_subcommands() {
        let path = fixtures_path().join("bash_history");
        let output = run_histop(&[
            "-f", path.to_str().unwrap(),
            "-o", "json",
            "-s",
            "-c", "5"
        ]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        assert!(output.status.success());
        assert!(stdout.contains("\"command\":"));
        // Should contain subcommand format
        assert!(stdout.contains("git ") || stdout.contains("cargo "));
    }

    #[test]
    fn test_csv_output_with_all_and_more_than() {
        let path = fixtures_path().join("bash_history");
        let output = run_histop(&[
            "-f", path.to_str().unwrap(),
            "-o", "csv",
            "-a",
            "-m", "1"
        ]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        assert!(output.status.success());
        assert!(stdout.contains("command,count,percentage"));
    }
}
