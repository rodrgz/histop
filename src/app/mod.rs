//! Application orchestration layer.

use std::collections::HashMap;
use std::{cmp, fmt, io};

use crate::color::ColorMode;
use crate::output::OutputFormat;

mod parser;
mod render;

#[derive(Debug)]
pub enum AppError {
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
pub struct RunConfig {
    pub file: String,
    pub count: usize,
    pub all: bool,
    pub more_than: usize,
    pub ignore: Vec<String>,
    pub bar_size: usize,
    pub no_bar: bool,
    pub no_hist: bool,
    pub no_cumu: bool,
    pub no_perc: bool,
    pub output_format: OutputFormat,
    pub color_mode: ColorMode,
}

#[derive(Debug, Clone)]
pub(crate) struct RankedCommand {
    pub(crate) name: String,
    pub(crate) count: usize,
}

pub fn run(config: &RunConfig) -> Result<(), AppError> {
    let command_counts = parser::load_command_counts(&config.file, &config.ignore, config.no_hist)?;
    let commands = filter_and_sort_commands(command_counts, config.more_than);
    let n = output_limit(commands.len(), config.all, config.count);
    render::write_output(&commands, n, config)
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
}
