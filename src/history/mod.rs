//! History parsing module.

mod detect;
pub mod fish;
pub mod shell;

pub use detect::{HistoryFormat, detect_history_format};
pub use shell::count_from_file;
