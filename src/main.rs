//! histop - Shell history command frequency analyzer

use std::process;

use histop::app::{self, AppError, RunConfig};

mod interface;

fn main() {
    let config = match interface::cli::Config::from_args()
        .map(to_run_config)
        .map_err(AppError::Config)
    {
        Ok(value) => value,
        Err(err) => {
            eprintln!("Error: {}", err);
            process::exit(1);
        }
    };

    match app::run(&config) {
        Ok(()) => {}
        Err(AppError::BrokenPipe) => process::exit(0),
        Err(err) => {
            eprintln!("Error: {}", err);
            process::exit(1);
        }
    }
}

fn to_run_config(config: interface::cli::Config) -> RunConfig {
    RunConfig {
        file: config.file,
        count: config.count,
        all: config.all,
        more_than: config.more_than,
        ignore: config.ignore,
        bar_size: config.bar_size,
        no_bar: config.no_bar,
        no_hist: config.no_hist,
        no_cumu: config.no_cumu,
        no_perc: config.no_perc,
        output_format: config.output_format,
        color_mode: config.color_mode,
    }
}
