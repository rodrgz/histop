//! CLI argument parsing and configuration

use std::{env, fs, io::IsTerminal, path::Path, path::PathBuf, process};

use histop::config::FileConfig;
use histop::output::OutputFormat;
use histop::output::color::ColorMode;

const NO_HIST_INPUT_ERROR: &str = "When using -nh without FILE, provide input through stdin (pipe or \
     redirection), or pass FILE with -f/positional argument";

#[derive(Default)]
struct CliOverrides {
    file: Option<String>,
    count: Option<usize>,
    all: bool,
    more_than: Option<usize>,
    ignore: Option<Vec<String>>,
    bar_size: Option<usize>,
    no_bar: bool,
    no_hist: bool,
    no_cumu: bool,
    no_perc: bool,
    output_format: Option<OutputFormat>,
    color_mode: Option<ColorMode>,
    config_path: Option<String>,
}

/// Application configuration parsed from CLI arguments
pub struct Config {
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

impl Default for Config {
    fn default() -> Self {
        Self {
            file: String::new(),
            count: 25,
            all: false,
            more_than: 0,
            ignore: Vec::new(),
            bar_size: 25,
            no_bar: false,
            no_hist: false,
            no_cumu: false,
            no_perc: false,
            output_format: OutputFormat::Text,
            color_mode: ColorMode::Auto,
        }
    }
}

impl Config {
    /// Parse configuration from command line arguments
    pub fn from_args() -> Result<Self, String> {
        let args: Vec<String> = env::args().collect();
        let mut cli_overrides = CliOverrides::default();

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "-h" | "--help" => {
                    let mut help_config = Config::default();
                    if let Some(file_config) = FileConfig::load_default() {
                        help_config.apply_file_config(&file_config);
                    }
                    print_help_message(help_config.count, help_config.bar_size);
                    process::exit(0);
                }
                "-f" => {
                    let value = require_value_argument(&args, &mut i, "-f")?;
                    if cli_overrides.file.is_some() {
                        return Err(
                            "Conflicting input file arguments: use either -f <FILE> or positional FILE, not both".to_string(),
                        );
                    }
                    cli_overrides.file = Some(value);
                }
                "-c" => {
                    let value = require_value_argument(&args, &mut i, "-c")?;
                    cli_overrides.count =
                        Some(parse_usize_argument(&value, "-c")?);
                }
                "-a" => {
                    cli_overrides.all = true;
                }
                "-m" => {
                    let value = require_value_argument(&args, &mut i, "-m")?;
                    cli_overrides.more_than =
                        Some(parse_non_negative_usize_argument(&value, "-m")?);
                }
                "-i" => {
                    let value = require_value_argument(&args, &mut i, "-i")?;
                    cli_overrides.ignore = Some(
                        value
                            .split('|')
                            .map(|s| s.trim().to_string())
                            .collect(),
                    );
                }
                "-b" => {
                    let value = require_value_argument(&args, &mut i, "-b")?;
                    cli_overrides.bar_size =
                        Some(parse_usize_argument(&value, "-b")?);
                }
                "-n" => {
                    cli_overrides.no_bar = true;
                }
                "-nh" => {
                    cli_overrides.no_hist = true;
                }
                "-np" => {
                    cli_overrides.no_perc = true;
                }
                "-nc" => {
                    cli_overrides.no_cumu = true;
                }
                "-o" => {
                    let value = require_value_argument(&args, &mut i, "-o")?;
                    cli_overrides.output_format = Some(
                        OutputFormat::parse(&value).ok_or_else(|| {
                            format!("Invalid output format: {}. Use text, json, or csv", value)
                        })?,
                    );
                }
                "--color" => {
                    let value =
                        require_value_argument(&args, &mut i, "--color")?;
                    cli_overrides.color_mode = Some(
                        ColorMode::parse(&value).ok_or_else(|| {
                            format!("Invalid color mode: {}. Use auto, always, or never", value)
                        })?,
                    );
                }
                "--config" => {
                    let value =
                        require_value_argument(&args, &mut i, "--config")?;
                    cli_overrides.config_path = Some(value);
                }
                _ => {
                    if args[i].starts_with('-') {
                        return Err(format!("Invalid option: {}", args[i]));
                    }
                    if cli_overrides.file.is_some() {
                        return Err(
                            "Conflicting input file arguments: use either -f <FILE> or positional FILE, not both".to_string(),
                        );
                    }
                    cli_overrides.file = Some(args[i].clone());
                }
            }
            i += 1;
        }

        let mut config = Config::default();
        if let Some(file_config) = FileConfig::load_default() {
            config.apply_file_config(&file_config);
        }
        if let Some(ref config_path) = cli_overrides.config_path {
            let file_config = FileConfig::load(Path::new(config_path))
                .map_err(|e| format!("Failed to load config: {}", e))?;
            config.apply_file_config(&file_config);
        }
        config.apply_cli_overrides(&cli_overrides);

        if config.file.is_empty() {
            let stdin_is_terminal = std::io::stdin().is_terminal();
            if config.no_hist {
                config.file = resolve_no_hist_input(stdin_is_terminal)?;
            } else {
                config.file = get_histfile()?;
            }
        }

        Ok(config)
    }

    /// Apply settings from a file config
    fn apply_file_config(
        &mut self,
        file_config: &FileConfig,
    ) {
        if let Some(ref ignore) = file_config.ignore {
            self.ignore = ignore.clone();
        }
        if let Some(bar_size) = file_config.bar_size {
            self.bar_size = bar_size;
        }
        if let Some(count) = file_config.count {
            self.count = count;
        }
        if let Some(color) = file_config.color {
            self.color_mode = color;
        }
        if let Some(more_than) = file_config.more_than {
            self.more_than = more_than;
        }
    }

    fn apply_cli_overrides(
        &mut self,
        overrides: &CliOverrides,
    ) {
        if let Some(ref file) = overrides.file {
            self.file = file.clone();
        }
        if let Some(count) = overrides.count {
            self.count = count;
        }
        if overrides.all {
            self.all = true;
        }
        if let Some(more_than) = overrides.more_than {
            self.more_than = more_than;
        }
        if let Some(ref ignore) = overrides.ignore {
            self.ignore = ignore.clone();
        }
        if let Some(bar_size) = overrides.bar_size {
            self.bar_size = bar_size;
        }
        if overrides.no_bar {
            self.no_bar = true;
        }
        if overrides.no_hist {
            self.no_hist = true;
        }
        if overrides.no_cumu {
            self.no_cumu = true;
        }
        if overrides.no_perc {
            self.no_perc = true;
        }
        if let Some(output_format) = overrides.output_format {
            self.output_format = output_format;
        }
        if let Some(color_mode) = overrides.color_mode {
            self.color_mode = color_mode;
        }
    }
}

fn should_read_stdin(
    no_hist: bool,
    stdin_is_terminal: bool,
) -> bool {
    no_hist && !stdin_is_terminal
}

fn resolve_no_hist_input(stdin_is_terminal: bool) -> Result<String, String> {
    if should_read_stdin(true, stdin_is_terminal) {
        Ok("-".to_string())
    } else {
        Err(NO_HIST_INPUT_ERROR.to_string())
    }
}

fn parse_usize_argument(
    arg: &str,
    flag: &str,
) -> Result<usize, String> {
    match arg.parse::<usize>() {
        Ok(val) if val > 0 => Ok(val),
        _ => Err(format!(
            "Invalid {} argument, must be a positive integer",
            flag
        )),
    }
}

fn parse_non_negative_usize_argument(
    arg: &str,
    flag: &str,
) -> Result<usize, String> {
    match arg.parse::<usize>() {
        Ok(val) => Ok(val),
        _ => Err(format!(
            "Invalid {} argument, must be a non-negative integer",
            flag
        )),
    }
}

fn require_value_argument(
    args: &[String],
    i: &mut usize,
    flag: &str,
) -> Result<String, String> {
    *i += 1;
    if *i >= args.len() {
        return Err(format!("Missing value for {}", flag));
    }
    Ok(args[*i].clone())
}

/// Get the history file path
///
/// Uses platform-specific detection:
/// - Linux: reads /proc/self/stat to find parent shell
/// - Other platforms: falls back to $SHELL environment variable
fn get_histfile() -> Result<String, String> {
    let mut checked_paths: Vec<String> = Vec::new();

    if let Ok(histfile) = env::var("HISTFILE") {
        checked_paths.push(histfile.clone());
        if is_regular_file(&histfile) {
            return Ok(histfile);
        }
    }

    let home = env::var("HOME").unwrap_or_default();
    if home.is_empty() {
        return Err("Could not determine shell history file: HOME environment variable is not set".to_string());
    }

    let mut shell_hints: Vec<String> = Vec::new();
    if let Ok(parent_shell) = get_parent_shell() {
        push_unique(&mut shell_hints, parent_shell);
    }
    if let Ok(shell_path) = env::var("SHELL")
        && let Some(shell_name) =
            Path::new(&shell_path).file_name().and_then(|f| f.to_str())
    {
        push_unique(&mut shell_hints, shell_name.to_string());
    }

    let mut candidate_paths: Vec<String> = Vec::new();
    for shell in &shell_hints {
        for candidate in shell_history_candidates(&home, shell) {
            push_unique(&mut candidate_paths, candidate);
        }
    }
    for candidate in default_history_candidates(&home) {
        push_unique(&mut candidate_paths, candidate);
    }

    for candidate in candidate_paths {
        checked_paths.push(candidate.clone());
        if is_regular_file(&candidate) {
            return Ok(candidate);
        }
    }

    Err(format!(
        "Could not determine shell history file. Checked: {}",
        checked_paths.join(", ")
    ))
}

fn is_regular_file(path: &str) -> bool {
    fs::metadata(path).map(|meta| meta.is_file()).unwrap_or(false)
}

fn push_unique(
    values: &mut Vec<String>,
    value: String,
) {
    if !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

fn shell_history_candidates(
    home: &str,
    shell: &str,
) -> Vec<String> {
    match shell {
        "ash" => vec![format!("{}/.ash_history", home)],
        "bash" => vec![format!("{}/.bash_history", home)],
        "fish" => vec![format!("{}/.local/share/fish/fish_history", home)],
        "zsh" => vec![
            format!("{}/.config/zsh/.zsh_history", home),
            format!("{}/.zsh_history", home),
        ],
        "pwsh" => vec![format!(
            "{}/.local/share/powershell/PSReadLine/ConsoleHost_history.txt",
            home
        )],
        "tcsh" | "csh" => vec![
            format!("{}/.history", home),
            format!("{}/.csh_history", home),
            format!("{}/.tcsh_history", home),
        ],
        _ => Vec::new(),
    }
}

fn default_history_candidates(home: &str) -> Vec<String> {
    vec![
        format!("{}/.bash_history", home),
        format!("{}/.zsh_history", home),
        format!("{}/.config/zsh/.zsh_history", home),
        format!("{}/.ash_history", home),
        format!("{}/.local/share/fish/fish_history", home),
        format!(
            "{}/.local/share/powershell/PSReadLine/ConsoleHost_history.txt",
            home
        ),
        format!("{}/.history", home),
    ]
}

/// Get the parent shell name
///
/// Uses platform-specific detection
#[cfg(target_os = "linux")]
fn get_parent_shell() -> Result<String, String> {
    let stat_contents = fs::read_to_string("/proc/self/stat")
        .map_err(|e| format!("Failed to read /proc/self/stat: {}", e))?;
    let fields: Vec<&str> = stat_contents.split_whitespace().collect();
    let ppid = fields.get(3).ok_or("Invalid /proc/self/stat format")?;

    let cmdline_file = PathBuf::from(format!("/proc/{}/cmdline", ppid));
    let cmdline_contents = fs::read(&cmdline_file).map_err(|e| {
        format!("Failed to read {}: {}", cmdline_file.display(), e)
    })?;
    let first_arg = cmdline_contents
        .split(|b| *b == b'\0')
        .next()
        .ok_or_else(|| "Failed to parse parent cmdline".to_string())?;
    let first_arg = std::str::from_utf8(first_arg)
        .map_err(|_| "Failed to decode parent cmdline".to_string())?;

    let parent_cmdline = Path::new(first_arg)
        .file_name()
        .and_then(|f| f.to_str())
        .ok_or_else(|| "Failed to parse parent cmdline".to_string())?;

    Ok(parent_cmdline.to_string())
}

/// Fallback for non-Linux platforms: use $SHELL environment variable
#[cfg(not(target_os = "linux"))]
fn get_parent_shell() -> Result<String, String> {
    env::var("SHELL")
        .map_err(|_| "SHELL environment variable not set".to_string())
        .and_then(|shell| {
            Path::new(&shell)
                .file_name()
                .and_then(|f| f.to_str())
                .map(|s| s.to_string())
                .ok_or_else(|| "Failed to parse SHELL".to_string())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_history_candidates_zsh() {
        let candidates = shell_history_candidates("/tmp/home", "zsh");
        assert_eq!(candidates.len(), 2);
        assert!(candidates[0].ends_with(".config/zsh/.zsh_history"));
        assert!(candidates[1].ends_with(".zsh_history"));
    }

    #[test]
    fn test_shell_history_candidates_powershell() {
        let candidates = shell_history_candidates("/tmp/home", "pwsh");
        assert_eq!(candidates.len(), 1);
        assert!(
            candidates[0]
                .ends_with("powershell/PSReadLine/ConsoleHost_history.txt")
        );
    }

    #[test]
    fn test_shell_history_candidates_tcsh() {
        let candidates = shell_history_candidates("/tmp/home", "tcsh");
        assert_eq!(candidates.len(), 3);
        assert!(candidates.iter().any(|c| c.ends_with("/.history")));
        assert!(candidates.iter().any(|c| c.ends_with("/.csh_history")));
        assert!(candidates.iter().any(|c| c.ends_with("/.tcsh_history")));
    }

    #[test]
    fn test_default_history_candidates_contains_common_shells() {
        let candidates = default_history_candidates("/tmp/home");
        assert!(candidates.iter().any(|c| c.ends_with(".bash_history")));
        assert!(candidates.iter().any(|c| c.ends_with(".zsh_history")));
        assert!(candidates.iter().any(|c| c.ends_with("fish_history")));
        assert!(candidates.iter().any(|c| {
            c.ends_with("powershell/PSReadLine/ConsoleHost_history.txt")
        }));
        assert!(candidates.iter().any(|c| c.ends_with("/.history")));
    }

    #[test]
    fn test_push_unique() {
        let mut values = vec!["bash".to_string()];
        push_unique(&mut values, "bash".to_string());
        push_unique(&mut values, "zsh".to_string());
        assert_eq!(values, vec!["bash".to_string(), "zsh".to_string()]);
    }

    #[test]
    fn test_should_read_stdin_when_no_hist_and_stdin_not_terminal() {
        assert!(should_read_stdin(true, false));
    }

    #[test]
    fn test_should_not_read_stdin_when_history_mode_enabled() {
        assert!(!should_read_stdin(false, false));
    }

    #[test]
    fn test_should_not_read_stdin_when_stdin_is_terminal() {
        assert!(!should_read_stdin(true, true));
    }

    #[test]
    fn test_resolve_no_hist_input_uses_stdin_when_not_terminal() {
        let file = resolve_no_hist_input(false).unwrap();
        assert_eq!(file, "-");
    }

    #[test]
    fn test_resolve_no_hist_input_requires_stdin_or_file_on_terminal() {
        let err = resolve_no_hist_input(true).unwrap_err();
        assert_eq!(err, NO_HIST_INPUT_ERROR);
    }
}

fn print_help_message(
    count: usize,
    bar_size: usize,
) {
    println!(
        "Usage: histop [options] [FILE]\n\
        \u{A0}-h, --help       Print this help message\n\
        \u{A0}-f <FILE>        Path to the history file (or pass FILE positionally)\n\
        \u{A0}-c <COUNT>       Number of commands to print (default: {})\n\
        \u{A0}-a               Print all commands (overrides -c)\n\
        \u{A0}-m <MORE_THAN>   Only consider commands used more than <MORE_THAN> times\n\
        \u{A0}-i <IGNORE>      Ignore specified commands (e.g. \"ls|grep|nvim\")\n\
        \u{A0}-b <BAR_SIZE>    Size of the bar graph (default: {})\n\
        \u{A0}-n               Do not print the bar\n\
        \u{A0}-nh              Disable history mode (requires FILE or piped/redirected stdin)\n\
        \u{A0}-np              Do not print the percentage in the bar\n\
        \u{A0}-nc              Do not print the inverse cumulative percentage in the bar\n\
        \u{A0}-o <FMT>         Output format: text (default), json, csv\n\
        \u{A0}--color <WHEN>   Color output: auto (default), always, never\n\
        \u{A0}--config <PATH>  Path to config file\n\
        \u{A0}██               Percentage\n\
        \u{A0}▓▓               Inverse cumulative percentage",
        count, bar_size
    );
}
