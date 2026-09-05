//! History parsing module.

pub mod detect;
pub mod fish;
pub mod powershell;
pub mod shell;
pub mod simple_history;
pub mod tcsh;

mod load;

pub use detect::{HistoryFormat, detect_history_format};
pub use shell::count_from_file;
pub(crate) use load::{HistoryLoadError, load_command_frequencies};
