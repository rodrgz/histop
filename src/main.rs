//! histop - Shell history command frequency analyzer
//!
//! Analyzes your shell history file and presents the most frequently
//! used commands in a visually appealing format.

use std::collections::HashMap;
use std::io::{self, BufWriter, Write};
use std::{cmp, fmt, process};

use histop::bar::{BarConfig, BarItem};
use histop::color::Colorizer;
use histop::output::{CommandEntry, OutputFormat};
use histop::{bar, fish, history, output};

mod cli;

#[derive(Debug)]
enum AppError {
    Config(String),
    HistoryRead {
        parser: &'static str,
        path: String,
        source: io::Error,
    },
    Output(io::Error),
    BrokenPipe,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(msg) => write!(f, "{}", msg),
            Self::HistoryRead {
                parser,
                path,
                source,
            } => {
                write!(f, "Error reading {} history file {}: {}", parser, path, source)
            }
            Self::Output(source) => write!(f, "Error writing output: {}", source),
            Self::BrokenPipe => write!(f, "Broken pipe"),
        }
    }
}

#[derive(Debug, Clone)]
struct RankedCommand {
    name: String,
    count: usize,
}

fn main() {
    let config = match cli::Config::from_args().map_err(AppError::Config) {
        Ok(value) => value,
        Err(err) => {
            eprintln!("Error: {}", err);
            process::exit(1);
        }
    };

    match run(&config) {
        Ok(()) => {}
        Err(AppError::BrokenPipe) => process::exit(0),
        Err(err) => {
            eprintln!("Error: {}", err);
            process::exit(1);
        }
    }
}

fn run(config: &cli::Config) -> Result<(), AppError> {
    let command_counts = load_command_counts(config)?;
    let commands = filter_and_sort_commands(command_counts, config.more_than);
    let n = output_limit(commands.len(), config.all, config.count);

    match config.output_format {
        OutputFormat::Json => write_json_output(&commands, n),
        OutputFormat::Csv => write_csv_output(&commands, n),
        OutputFormat::Text => write_text_output(&commands, n, config),
    }
}

fn load_command_counts(config: &cli::Config) -> Result<HashMap<String, usize>, AppError> {
    if is_fish_history(&config.file) {
        fish::count_from_file(&config.file, &config.ignore).map_err(|source| AppError::HistoryRead {
            parser: "fish",
            path: config.file.clone(),
            source,
        })
    } else {
        history::count_from_file(&config.file, &config.ignore, config.no_hist).map_err(|source| {
            AppError::HistoryRead {
                parser: "shell",
                path: config.file.clone(),
                source,
            }
        })
    }
}

fn filter_and_sort_commands(
    command_counts: HashMap<String, usize>,
    more_than: usize,
) -> Vec<RankedCommand> {
    let mut commands: Vec<RankedCommand> = command_counts
        .into_iter()
        .filter(|(_, count)| *count > more_than)
        .map(|(name, count)| RankedCommand { name, count })
        .collect();

    commands.sort_unstable_by(|a, b| {
        b.count
            .cmp(&a.count)
            .then_with(|| a.name.cmp(&b.name))
    });
    commands
}

fn output_limit(total_commands: usize, all: bool, count: usize) -> usize {
    if all {
        total_commands
    } else {
        cmp::min(count, total_commands)
    }
}

fn build_command_entries(commands: &[RankedCommand], n: usize) -> Vec<CommandEntry> {
    let total: usize = commands.iter().take(n).map(|entry| entry.count).sum();

    commands
        .iter()
        .take(n)
        .map(|entry| CommandEntry::new(entry.name.clone(), entry.count, total))
        .collect()
}

fn write_json_output(commands: &[RankedCommand], n: usize) -> Result<(), AppError> {
    let entries = build_command_entries(commands, n);
    write_stdout(&(output::format_json(&entries) + "\n"))
}

fn write_csv_output(commands: &[RankedCommand], n: usize) -> Result<(), AppError> {
    let entries = build_command_entries(commands, n);
    write_stdout(&output::format_csv(&entries))
}

fn write_text_output(commands: &[RankedCommand], n: usize, config: &cli::Config) -> Result<(), AppError> {
    let items: Vec<BarItem> = commands
        .iter()
        .take(n)
        .map(|entry| BarItem::new(entry.name.as_str(), entry.count))
        .collect();

    let bar_config = BarConfig {
        size: if config.no_bar { 0 } else { config.bar_size },
        show_percentage: !config.no_perc,
        show_cumulative: !config.no_cumu,
    };

    let colorizer = Colorizer::new(config.color_mode);
    let rendered = bar::render_bars(&items, &bar_config);

    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());
    bar::write_bars(&mut writer, &rendered, !config.no_bar, &colorizer)
        .and_then(|_| writer.flush())
        .map_err(map_output_error)
}

/// Check if a file path is a fish history file
fn is_fish_history(path: &str) -> bool {
    path.ends_with("fish_history") || path.contains("/fish/")
}

fn write_stdout(output: &str) -> Result<(), AppError> {
    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());
    writer
        .write_all(output.as_bytes())
        .and_then(|_| writer.flush())
        .map_err(map_output_error)
}

fn map_output_error(e: io::Error) -> AppError {
    if e.kind() == io::ErrorKind::BrokenPipe {
        AppError::BrokenPipe
    } else {
        AppError::Output(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_and_sort_commands() {
        let mut counts = HashMap::new();
        counts.insert("ls".to_string(), 4);
        counts.insert("git".to_string(), 2);
        counts.insert("cd".to_string(), 1);

        let commands = filter_and_sort_commands(counts, 1);
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].name, "ls");
        assert_eq!(commands[0].count, 4);
        assert_eq!(commands[1].name, "git");
        assert_eq!(commands[1].count, 2);
    }

    #[test]
    fn test_filter_and_sort_commands_deterministic_tie_break() {
        let mut counts = HashMap::new();
        counts.insert("zsh".to_string(), 2);
        counts.insert("bash".to_string(), 2);

        let commands = filter_and_sort_commands(counts, 0);
        assert_eq!(commands[0].name, "bash");
        assert_eq!(commands[1].name, "zsh");
    }

    #[test]
    fn test_output_limit() {
        assert_eq!(output_limit(10, false, 3), 3);
        assert_eq!(output_limit(2, false, 3), 2);
        assert_eq!(output_limit(2, true, 1), 2);
    }

    #[test]
    fn test_build_command_entries_uses_top_n_total() {
        let commands = vec![
            RankedCommand {
                name: "ls".to_string(),
                count: 6,
            },
            RankedCommand {
                name: "git".to_string(),
                count: 4,
            },
            RankedCommand {
                name: "cd".to_string(),
                count: 2,
            },
        ];

        let entries = build_command_entries(&commands, 2);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].command, "ls");
        assert_eq!(entries[1].command, "git");
        assert!((entries[0].percentage - 60.0).abs() < f64::EPSILON);
        assert!((entries[1].percentage - 40.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_load_command_counts_with_fixture() {
        let mut config = cli::Config::default();
        config.file = format!("{}/tests/fixtures/bash_history", env!("CARGO_MANIFEST_DIR"));
        config.no_hist = false;

        let counts = load_command_counts(&config).unwrap();
        assert_eq!(counts.get("git"), Some(&6));
        assert_eq!(counts.get("ls"), Some(&5));
    }
}
