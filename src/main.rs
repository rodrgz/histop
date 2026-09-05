//! histop - Shell history command frequency analyzer

use std::process;

use histop::app::{self, AppError};

mod interface;

fn main() {
    let config = match interface::cli::from_args().map_err(AppError::Config) {
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
