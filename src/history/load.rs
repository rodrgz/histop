use ahash::AHashMap;
use std::io;

use crate::command::FrequencyTable;

use super::{
    HistoryFormat, detect_history_format, fish, powershell, shell, tcsh,
};

pub(crate) struct HistoryLoadError {
    pub(crate) parser: &'static str,
    pub(crate) path: String,
    pub(crate) source: io::Error,
}

pub(crate) fn load_command_frequencies(
    file: &str,
    ignore: &[String],
    no_hist: bool,
) -> Result<FrequencyTable, HistoryLoadError> {
    if no_hist {
        return with_context(
            "raw",
            file,
            shell::count_from_file(file, ignore, no_hist),
        );
    }

    let history_format =
        detect_history_format(file).map_err(|source| HistoryLoadError {
            parser: "shell",
            path: file.to_string(),
            source,
        })?;

    let (parser, counts) = match history_format {
        HistoryFormat::Fish => (
            "fish",
            fish::count_from_file(file, ignore, no_hist),
        ),
        HistoryFormat::Shell => (
            "shell",
            shell::count_from_file(file, ignore, no_hist),
        ),
        HistoryFormat::PowerShell => (
            "powershell",
            powershell::count_from_file(file, ignore, no_hist),
        ),
        HistoryFormat::Tcsh => (
            "tcsh",
            tcsh::count_from_file(file, ignore, no_hist),
        ),
    };

    with_context(parser, file, counts)
}

fn with_context(
    parser: &'static str,
    path: &str,
    counts: Result<AHashMap<String, usize>, io::Error>,
) -> Result<FrequencyTable, HistoryLoadError> {
    counts
        .map(FrequencyTable::from_counts)
        .map_err(|source| HistoryLoadError {
            parser,
            path: path.to_string(),
            source,
        })
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
        let now_nanos =
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        std::env::temp_dir().join(format!(
            "{}_{}_{}",
            prefix,
            std::process::id(),
            now_nanos
        ))
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
    fn test_detect_history_format_tcsh_fixture() {
        let path = fixtures_path().join("tcsh_history");
        assert_eq!(
            detect_history_format(path.to_str().unwrap()).unwrap(),
            HistoryFormat::Tcsh
        );
    }

    #[test]
    fn test_detect_history_format_powershell_filename() {
        let base = unique_temp_path("powershell_history_detection");
        fs::create_dir_all(&base).unwrap();
        let path = base.join("ConsoleHost_history.txt");
        let mut file = fs::File::create(&path).unwrap();
        writeln!(file, "git status").unwrap();
        writeln!(file, "ls -la").unwrap();

        assert_eq!(
            detect_history_format(path.to_str().unwrap()).unwrap(),
            HistoryFormat::PowerShell
        );

        fs::remove_file(path).ok();
        fs::remove_dir_all(base).ok();
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
            detect_history_format(path_in_fish_dir.to_str().unwrap())
                .unwrap(),
            HistoryFormat::Shell
        );

        fs::remove_file(path_in_fish_dir).ok();
        fs::remove_dir_all(path).ok();
    }
}
