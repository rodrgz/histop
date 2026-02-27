use ahash::AHashMap;

pub fn count_from_file(
    file_path: &str,
    ignore: &[String],
    no_hist: bool,
) -> Result<AHashMap<String, usize>, std::io::Error> {
    // Tcsh history uses '#' timestamp lines (e.g. #+1680820391).
    super::simple_history::count_from_file(file_path, ignore, no_hist, |line| {
        line.trim().starts_with('#')
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::fs::File;
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_count_tcsh() {
        let now_nanos =
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!(
            "test_tcsh_{}_{}.history",
            std::process::id(),
            now_nanos
        ));
        let mut file = File::create(&path).unwrap();
        writeln!(file, "#+1680820391").unwrap();
        writeln!(file, "ls -la").unwrap();
        writeln!(file, "#+1680820392").unwrap();
        writeln!(file, "git status").unwrap();
        writeln!(file, "#+1680820393").unwrap();
        writeln!(file, "ls").unwrap();

        let result =
            count_from_file(path.to_str().unwrap(), &[], false).unwrap();
        assert_eq!(result.get("ls"), Some(&2));
        assert_eq!(result.get("git"), Some(&1));

        fs::remove_file(path).ok();
    }

    #[test]
    fn test_invalid_utf8_skipped() {
        let now_nanos =
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!(
            "test_tcsh_utf8_{}_{}.history",
            std::process::id(),
            now_nanos
        ));
        let mut file = File::create(&path).unwrap();
        file.write_all(b"ls -la\n").unwrap();
        file.write_all(b"\xFF\xFE bad\n").unwrap();
        file.write_all(b"git status\n").unwrap();

        let result =
            count_from_file(path.to_str().unwrap(), &[], false).unwrap();
        assert_eq!(result.get("ls"), Some(&1));
        assert_eq!(result.get("git"), Some(&1));

        fs::remove_file(path).ok();
    }
}
