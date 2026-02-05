//! Integration tests for histop

use std::path::PathBuf;

// Get the path to test fixtures
fn fixtures_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

mod bash_history {
    use super::*;

    #[test]
    fn test_parse_bash_history() {
        let path = fixtures_path().join("bash_history");
        let result = histop::history::count_from_file(
            path.to_str().unwrap(),
            &[],
            false,
            false,
            false,
        )
        .unwrap();

        // Verify expected counts
        assert_eq!(result.get("ls"), Some(&5)); // ls appears 5 times (including in pipe)
        assert_eq!(result.get("git"), Some(&6)); // git appears 6 times
        assert_eq!(result.get("cargo"), Some(&4)); // cargo appears 4 times
        assert_eq!(result.get("nvim"), Some(&3)); // nvim appears 3 times
        assert_eq!(result.get("apt"), Some(&1)); // sudo apt -> apt
        assert_eq!(result.get("grep"), Some(&1)); // from pipe
    }

    #[test]
    fn test_bash_with_subcommands() {
        let path = fixtures_path().join("bash_history");
        let result = histop::history::count_from_file(
            path.to_str().unwrap(),
            &[],
            false,
            true, // track subcommands
            false,
        )
        .unwrap();

        // With subcommand tracking
        assert_eq!(result.get("git status"), Some(&2));
        assert_eq!(result.get("git commit"), Some(&1));
        assert_eq!(result.get("cargo build"), Some(&2));
    }

    #[test]
    fn test_bash_with_ignore() {
        let path = fixtures_path().join("bash_history");
        let ignore = vec!["ls".to_string(), "cd".to_string()];
        let result = histop::history::count_from_file(
            path.to_str().unwrap(),
            &ignore,
            false,
            false,
            false,
        )
        .unwrap();

        // ls and cd should be ignored
        assert_eq!(result.get("ls"), None);
        assert_eq!(result.get("cd"), None);
        // Others should still be present
        assert!(result.get("git").is_some());
    }
}

mod zsh_history {
    use super::*;

    #[test]
    fn test_parse_zsh_extended_history() {
        let path = fixtures_path().join("zsh_history");
        let result = histop::history::count_from_file(
            path.to_str().unwrap(),
            &[],
            false,
            false,
            false,
        )
        .unwrap();

        // Same content as bash, should have same counts
        assert_eq!(result.get("ls"), Some(&5));
        assert_eq!(result.get("git"), Some(&6));
        assert_eq!(result.get("cargo"), Some(&4));
    }

    #[test]
    fn test_zsh_with_subcommands() {
        let path = fixtures_path().join("zsh_history");
        let result = histop::history::count_from_file(
            path.to_str().unwrap(),
            &[],
            false,
            true,
            false,
        )
        .unwrap();

        assert_eq!(result.get("git status"), Some(&2));
        assert_eq!(result.get("cargo build"), Some(&2));
    }
}

mod fish_history {
    use super::*;

    #[test]
    fn test_parse_fish_history() {
        let path = fixtures_path().join("fish_history");
        let result = histop::fish::count_from_file(
            path.to_str().unwrap(),
            &[],
            false,
            false,
        )
        .unwrap();

        // Count commands
        assert_eq!(result.get("ls"), Some(&5));
        assert_eq!(result.get("git"), Some(&6));
        assert_eq!(result.get("cargo"), Some(&4));
        assert_eq!(result.get("nvim"), Some(&3));
    }

    #[test]
    fn test_fish_with_subcommands() {
        let path = fixtures_path().join("fish_history");
        let result = histop::fish::count_from_file(
            path.to_str().unwrap(),
            &[],
            true,
            false,
        )
        .unwrap();

        assert_eq!(result.get("git status"), Some(&2));
        assert_eq!(result.get("git commit"), Some(&1));
    }

    #[test]
    fn test_fish_with_ignore() {
        let path = fixtures_path().join("fish_history");
        let ignore = vec!["ls".to_string()];
        let result = histop::fish::count_from_file(
            path.to_str().unwrap(),
            &ignore,
            false,
            false,
        )
        .unwrap();

        assert_eq!(result.get("ls"), None);
        assert!(result.get("git").is_some());
    }
}

mod output_formats {
    use histop::output::{format_csv, format_json, CommandEntry};

    #[test]
    fn test_json_output() {
        let entries = vec![
            CommandEntry::new("ls".to_string(), 10, 30),
            CommandEntry::new("git".to_string(), 20, 30),
        ];

        let json = format_json(&entries);
        assert!(json.contains("\"command\": \"ls\""));
        assert!(json.contains("\"count\": 10"));
        assert!(json.contains("\"command\": \"git\""));
        assert!(json.contains("\"count\": 20"));
    }

    #[test]
    fn test_csv_output() {
        let entries = vec![
            CommandEntry::new("ls".to_string(), 10, 30),
            CommandEntry::new("git".to_string(), 20, 30),
        ];

        let csv = format_csv(&entries);
        assert!(csv.starts_with("command,count,percentage\n"));
        assert!(csv.contains("ls,10,"));
        assert!(csv.contains("git,20,"));
    }
}

mod config {
    use histop::config::FileConfig;
    use histop::color::ColorMode;

    #[test]
    fn test_parse_config() {
        let content = r#"
count = 15
bar_size = 30
color = "always"
subcommands = true
ignore = ["ls", "cd"]
"#;
        let config = FileConfig::parse(content).unwrap();
        assert_eq!(config.count, Some(15));
        assert_eq!(config.bar_size, Some(30));
        assert_eq!(config.color, Some(ColorMode::Always));
        assert_eq!(config.subcommands, Some(true));
        assert_eq!(config.ignore, Some(vec!["ls".to_string(), "cd".to_string()]));
    }
}

mod utils {
    use histop::utils::{clean_line, get_first_word, SUBCOMMAND_TOOLS};

    #[test]
    fn test_subcommand_tools_contains_expected() {
        assert!(SUBCOMMAND_TOOLS.contains(&"git"));
        assert!(SUBCOMMAND_TOOLS.contains(&"cargo"));
        assert!(SUBCOMMAND_TOOLS.contains(&"npm"));
        assert!(SUBCOMMAND_TOOLS.contains(&"docker"));
    }

    #[test]
    fn test_get_first_word_with_subcommand() {
        let result = get_first_word("git status --short", &[], true);
        assert_eq!(result, "git status");
    }

    #[test]
    fn test_clean_line_preserves_outer_pipes() {
        let result = clean_line("cat file | grep pattern | wc -l");
        assert_eq!(result, "cat file | grep pattern | wc -l");
    }
}
