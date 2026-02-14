//! History parsing module.

pub mod fish;
pub mod shell;
mod detect;

pub use detect::{detect_history_format, HistoryFormat};
pub use shell::count_from_file;
