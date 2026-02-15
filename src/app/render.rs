use std::io::{self, BufWriter, Write};

use crate::app::{AppError, RankedCommand, RunConfig};
use crate::output::bar::{self, BarConfig, BarItem};
use crate::output::color::Colorizer;
use crate::output::{self, CommandEntry, OutputFormat};

pub(super) fn write_output(
    commands: &[RankedCommand],
    n: usize,
    config: &RunConfig,
) -> Result<(), AppError> {
    match config.output_format {
        OutputFormat::Json => write_json_output(commands, n),
        OutputFormat::Csv => write_csv_output(commands, n),
        OutputFormat::Text => write_text_output(commands, n, config),
    }
}

fn build_command_entries(
    commands: &[RankedCommand],
    n: usize,
) -> Vec<CommandEntry> {
    let total: usize = commands.iter().take(n).map(|entry| entry.count).sum();

    commands
        .iter()
        .take(n)
        .map(|entry| CommandEntry::new(entry.name.clone(), entry.count, total))
        .collect()
}

fn write_json_output(
    commands: &[RankedCommand],
    n: usize,
) -> Result<(), AppError> {
    let entries = build_command_entries(commands, n);
    write_stdout(&(output::format_json(&entries) + "\n"))
}

fn write_csv_output(
    commands: &[RankedCommand],
    n: usize,
) -> Result<(), AppError> {
    let entries = build_command_entries(commands, n);
    write_stdout(&output::format_csv(&entries))
}

fn write_text_output(
    commands: &[RankedCommand],
    n: usize,
    config: &RunConfig,
) -> Result<(), AppError> {
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
    fn test_build_command_entries_uses_top_n_total() {
        let commands = vec![
            RankedCommand { name: "ls".to_string(), count: 6 },
            RankedCommand { name: "git".to_string(), count: 4 },
            RankedCommand { name: "cd".to_string(), count: 2 },
        ];

        let entries = build_command_entries(&commands, 2);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].command, "ls");
        assert_eq!(entries[1].command, "git");
        assert!((entries[0].percentage - 60.0).abs() < f64::EPSILON);
        assert!((entries[1].percentage - 40.0).abs() < f64::EPSILON);
    }
}
