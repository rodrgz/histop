//! Application orchestration layer.

use std::{cmp, fmt, io};

use crate::history::{self, HistoryLoadError};
use crate::output::OutputFormat;
use crate::output::color::ColorMode;

mod render;

#[derive(Debug)]
pub enum AppError {
    Config(String),
    HistoryRead { parser: &'static str, path: String, source: io::Error },
    Output(io::Error),
    BrokenPipe,
}

impl fmt::Display for AppError {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            Self::Config(msg) => write!(f, "{}", msg),
            Self::HistoryRead { parser, path, source } => {
                write!(
                    f,
                    "Error reading {} history file {}: {}",
                    parser, path, source
                )
            }
            Self::Output(source) => {
                write!(f, "Error writing output: {}", source)
            }
            Self::BrokenPipe => write!(f, "Broken pipe"),
        }
    }
}

impl From<HistoryLoadError> for AppError {
    fn from(error: HistoryLoadError) -> Self {
        Self::HistoryRead {
            parser: error.parser,
            path: error.path,
            source: error.source,
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

pub fn run(config: &RunConfig) -> Result<(), AppError> {
    let frequencies = history::load_command_frequencies(
        &config.file,
        &config.ignore,
        config.no_hist,
    )?;
    let commands = frequencies.into_ranked(config.more_than);
    let n = output_limit(commands.len(), config.all, config.count);
    render::write_output(&commands, n, config)
}

fn output_limit(
    total_commands: usize,
    all: bool,
    count: usize,
) -> usize {
    if all { total_commands } else { cmp::min(count, total_commands) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_limit() {
        assert_eq!(output_limit(10, false, 3), 3);
        assert_eq!(output_limit(2, false, 3), 2);
        assert_eq!(output_limit(2, true, 1), 2);
    }
}
