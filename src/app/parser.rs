use std::collections::HashMap;

use crate::app::AppError;
use crate::history::{self, HistoryFormat};

pub(super) fn load_command_counts(
    file: &str,
    ignore: &[String],
    no_hist: bool,
) -> Result<HashMap<String, usize>, AppError> {
    let history_format = history::detect_history_format(file).map_err(|source| AppError::HistoryRead {
        parser: "shell",
        path: file.to_string(),
        source,
    })?;
    match history_format {
        HistoryFormat::Fish => history::fish::count_from_file(file, ignore, no_hist).map_err(|source| AppError::HistoryRead {
            parser: "fish",
            path: file.to_string(),
            source,
        }),
        HistoryFormat::Shell => history::count_from_file(file, ignore, no_hist).map_err(|source| {
            AppError::HistoryRead {
                parser: "shell",
                path: file.to_string(),
                source,
            }
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn fixtures_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
    }

    fn unique_temp_path(prefix: &str) -> PathBuf {
        let now_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{}_{}_{}", prefix, std::process::id(), now_nanos))
    }

    #[test]
    fn test_detect_history_format_fish_fixture() {
        let path = fixtures_path().join("fish_history");
        assert_eq!(
            history::detect_history_format(path.to_str().unwrap()).unwrap(),
            HistoryFormat::Fish
        );
    }

    #[test]
    fn test_detect_history_format_shell_fixture() {
        let path = fixtures_path().join("bash_history");
        assert_eq!(
            history::detect_history_format(path.to_str().unwrap()).unwrap(),
            HistoryFormat::Shell
        );
    }

    #[test]
    fn test_detect_history_format_ignores_fish_like_path_name() {
        let path = unique_temp_path("bash_content_in_fish_dir");
        let path_in_fish_dir = path.join("fish").join("history.txt");
        fs::create_dir_all(path_in_fish_dir.parent().unwrap()).unwrap();

        let mut file = fs::File::create(&path_in_fish_dir).unwrap();
        writeln!(file, "git status").unwrap();
        writeln!(file, "ls -la").unwrap();

        assert_eq!(
            history::detect_history_format(path_in_fish_dir.to_str().unwrap()).unwrap(),
            HistoryFormat::Shell
        );

        fs::remove_file(path_in_fish_dir).ok();
        fs::remove_dir_all(path).ok();
    }
}
