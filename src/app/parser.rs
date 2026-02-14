use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, BufReader};

use crate::app::AppError;
use crate::{fish, history};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HistoryFormat {
    Shell,
    Fish,
}

pub(super) fn load_command_counts(
    file: &str,
    ignore: &[String],
    no_hist: bool,
) -> Result<HashMap<String, usize>, AppError> {
    match detect_history_format(file)? {
        HistoryFormat::Fish => fish::count_from_file(file, ignore).map_err(|source| AppError::HistoryRead {
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

fn detect_history_format(path: &str) -> Result<HistoryFormat, AppError> {
    let file = fs::File::open(path).map_err(|source| AppError::HistoryRead {
        parser: "shell",
        path: path.to_string(),
        source,
    })?;
    let reader = BufReader::new(file);

    let mut fish_score = 0_u32;
    let mut shell_score = 0_u32;
    let mut inspected = 0_u32;

    for line_result in reader.lines() {
        let line = match line_result {
            Ok(line) => line,
            Err(e) => {
                if e.kind() == io::ErrorKind::InvalidData {
                    continue;
                }
                return Err(AppError::HistoryRead {
                    parser: "shell",
                    path: path.to_string(),
                    source: e,
                });
            }
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        inspected += 1;
        if trimmed.starts_with("- cmd: ") {
            fish_score += 4;
        } else if trimmed.starts_with("when: ") || trimmed.starts_with("paths:") {
            fish_score += 2;
        } else if line.starts_with("  when: ") || line.starts_with("  paths: ") || line.starts_with("  - ") {
            fish_score += 1;
        } else if line.starts_with(": ") && line.contains(';') {
            shell_score += 3;
        } else {
            shell_score += 1;
        }

        if inspected >= 64 {
            break;
        }
    }

    if fish_score > shell_score {
        Ok(HistoryFormat::Fish)
    } else {
        Ok(HistoryFormat::Shell)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
            detect_history_format(path.to_str().unwrap()).unwrap(),
            HistoryFormat::Fish
        );
    }

    #[test]
    fn test_detect_history_format_shell_fixture() {
        let path = fixtures_path().join("bash_history");
        assert_eq!(
            detect_history_format(path.to_str().unwrap()).unwrap(),
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
            detect_history_format(path_in_fish_dir.to_str().unwrap()).unwrap(),
            HistoryFormat::Shell
        );

        fs::remove_file(path_in_fish_dir).ok();
        fs::remove_dir_all(path).ok();
    }
}
