use std::fs;
use std::io::{self, BufRead, BufReader};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryFormat {
    Shell,
    Fish,
}

pub fn detect_history_format(path: &str) -> Result<HistoryFormat, io::Error> {
    let file = fs::File::open(path)?;
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
                return Err(e);
            }
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        inspected += 1;
        if trimmed.starts_with("- cmd: ") {
            fish_score += 4;
        } else if trimmed.starts_with("when: ") || trimmed.starts_with("paths:")
        {
            fish_score += 2;
        } else if line.starts_with("  when: ")
            || line.starts_with("  paths: ")
            || line.starts_with("  - ")
        {
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
