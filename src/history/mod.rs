//! History parsing module.

pub mod detect;
pub mod fish;
pub mod powershell;
pub mod shell;
pub mod tcsh;

pub use detect::{HistoryFormat, detect_history_format};
pub use shell::count_from_file;
