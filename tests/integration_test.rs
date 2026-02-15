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
    fn test_bash_with_ignore() {
        let path = fixtures_path().join("bash_history");
        let ignore = vec!["ls".to_string(), "cd".to_string()];
        let result = histop::history::count_from_file(
            path.to_str().unwrap(),
            &ignore,
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
        )
        .unwrap();

        // Same content as bash, should have same counts
        assert_eq!(result.get("ls"), Some(&5));
        assert_eq!(result.get("git"), Some(&6));
        assert_eq!(result.get("cargo"), Some(&4));
    }

}

mod fish_history {
    use super::*;

    #[test]
    fn test_parse_fish_history() {
        let path = fixtures_path().join("fish_history");
        let result = histop::history::fish::count_from_file(
            path.to_str().unwrap(),
            &[],
            false,
        )
        .unwrap();

        // Count commands
        assert_eq!(result.get("ls"), Some(&5));
        assert_eq!(result.get("git"), Some(&6));
        assert_eq!(result.get("cargo"), Some(&4));
        assert_eq!(result.get("nvim"), Some(&3));
        assert_eq!(result.get("apt"), Some(&1)); // sudo apt -> apt
    }

    #[test]
    fn test_fish_with_ignore() {
        let path = fixtures_path().join("fish_history");
        let ignore = vec!["ls".to_string()];
        let result = histop::history::fish::count_from_file(
            path.to_str().unwrap(),
            &ignore,
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
    use histop::output::color::ColorMode;

    #[test]
    fn test_parse_config() {
        let content = r#"
count = 15
bar_size = 30
color = "always"
ignore = ["ls", "cd"]
"#;
        let config = FileConfig::parse(content).unwrap();
        assert_eq!(config.count, Some(15));
        assert_eq!(config.bar_size, Some(30));
        assert_eq!(config.color, Some(ColorMode::Always));
        assert_eq!(config.ignore, Some(vec!["ls".to_string(), "cd".to_string()]));
    }
}

mod utils {
    use histop::shared::command_parse::{get_first_word, SplitCommands};
    use ahash::AHashSet;

    #[test]
    fn test_get_first_word() {
        let filters = AHashSet::new();
        let result = get_first_word("git status --short", &filters);
        assert_eq!(result, Some("git"));
    }

    #[test]
    fn test_split_commands_splits_on_pipes() {
        let parts: Vec<&str> = SplitCommands::new("cat file | grep pattern | wc -l").collect();
        assert_eq!(parts, vec!["cat file ", " grep pattern ", " wc -l"]);
    }
}
