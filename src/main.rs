//! histop - Shell history command frequency analyzer
//!
//! Analyzes your shell history file and presents the most frequently
//! used commands in a visually appealing format.

mod bar;
mod cli;
mod history;

use std::{cmp, process};

use bar::{BarConfig, BarItem};
use cli::Config;

fn main() {
    let config = match Config::from_args() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    let cmd_count = match history::count_from_file(
        &config.file,
        &config.ignore,
        config.no_hist,
        config.verbose,
    ) {
        Ok(counts) => counts,
        Err(e) => {
            eprintln!("Error reading history file: {}", e);
            process::exit(1);
        }
    };

    // Convert to sorted vec
    let mut commands: Vec<_> = cmd_count
        .iter()
        .filter(|(_, &count)| count > config.more_than)
        .collect();

    commands.sort_by_key(|(_, count)| cmp::Reverse(*count));

    // Limit count
    let n = if config.all {
        commands.len()
    } else {
        cmp::min(config.count, commands.len())
    };

    // Create bar items
    let items: Vec<BarItem> = commands
        .iter()
        .take(n)
        .map(|(name, &count)| BarItem::new(name, count))
        .collect();

    // Configure and render bars
    let bar_config = BarConfig {
        size: if config.no_bar { 0 } else { config.bar_size },
        show_percentage: !config.no_perc,
        show_cumulative: !config.no_cumu,
    };

    let rendered = bar::render_bars(&items, &bar_config);
    bar::print_bars(&rendered, !config.no_bar);
}
