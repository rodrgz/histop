//! histop - Shell history command frequency analyzer
//!
//! Analyzes your shell history file and presents the most frequently
//! used commands in a visually appealing format.

use std::{cmp, process};

use histop::bar::{BarConfig, BarItem};
use histop::color::Colorizer;
use histop::output::{CommandEntry, OutputFormat};
use histop::{bar, fish, history, output};

mod cli;

fn main() {
    let config = match cli::Config::from_args() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    let cmd_count = if config.fish_format || is_fish_history(&config.file) {
        match fish::count_from_file(
            &config.file,
            &config.ignore,
            config.track_subcommands,
            config.verbose,
        ) {
            Ok(counts) => counts,
            Err(e) => {
                eprintln!("Error reading fish history file: {}", e);
                process::exit(1);
            }
        }
    } else {
        match history::count_from_file(
            &config.file,
            &config.ignore,
            config.no_hist,
            config.track_subcommands,
            config.verbose,
        ) {
            Ok(counts) => counts,
            Err(e) => {
                eprintln!("Error reading history file: {}", e);
                process::exit(1);
            }
        }
    };

    // Convert to sorted vec
    let mut commands: Vec<_> = cmd_count
        .iter()
        .filter(|&(_, &count)| count > config.more_than)
        .collect();

    commands.sort_by_key(|(_, count)| cmp::Reverse(*count));

    // Limit count
    let n = if config.all {
        commands.len()
    } else {
        cmp::min(config.count, commands.len())
    };

    // Calculate total for percentage
    let total: usize = commands.iter().take(n).map(|(_, &c)| c).sum();

    // Handle different output formats
    match config.output_format {
        OutputFormat::Json => {
            let entries: Vec<CommandEntry> = commands
                .iter()
                .take(n)
                .map(|&(name, &count)| CommandEntry::new(name.clone(), count, total))
                .collect();
            println!("{}", output::format_json(&entries));
        }
        OutputFormat::Csv => {
            let entries: Vec<CommandEntry> = commands
                .iter()
                .take(n)
                .map(|&(name, &count)| CommandEntry::new(name.clone(), count, total))
                .collect();
            print!("{}", output::format_csv(&entries));
        }
        OutputFormat::Text => {
            // Create bar items
            let items: Vec<BarItem> = commands
                .iter()
                .take(n)
                .map(|&(name, &count)| BarItem::new(name, count))
                .collect();

            // Configure and render bars
            let bar_config = BarConfig {
                size: if config.no_bar { 0 } else { config.bar_size },
                show_percentage: !config.no_perc,
                show_cumulative: !config.no_cumu,
            };

            let colorizer = Colorizer::new(config.color_mode);
            let rendered = bar::render_bars(&items, &bar_config);
            bar::print_bars(&rendered, !config.no_bar, &colorizer);
        }
    }
}

/// Check if a file path is a fish history file
fn is_fish_history(path: &str) -> bool {
    path.ends_with("fish_history") || path.contains("/fish/")
}
