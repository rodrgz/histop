//! CLI argument parsing and configuration

use std::{env, error::Error, fs, path::Path, path::PathBuf, process};

use histop::color::ColorMode;
use histop::config::FileConfig;
use histop::output::OutputFormat;

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
    pub verbose: bool,
    pub fish_format: bool,
    pub track_subcommands: bool,
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
            verbose: false,
            fish_format: false,
            track_subcommands: false,
            output_format: OutputFormat::Text,
            color_mode: ColorMode::Auto,
        }
    }
}

impl Config {
    /// Parse configuration from command line arguments
    pub fn from_args() -> Result<Self, String> {
        let args: Vec<String> = env::args().collect();
        let mut config = Config::default();

        // Load config file first (CLI args override)
        if let Some(file_config) = FileConfig::load_default() {
            config.apply_file_config(&file_config);
        }

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "-h" | "--help" => {
                    print_help_message(config.count, config.bar_size);
                    process::exit(0);
                }
                "-f" => {
                    i += 1;
                    if i < args.len() {
                        config.file = args[i].clone();
                    }
                }
                "-c" => {
                    i += 1;
                    if i < args.len() {
                        config.count = parse_usize_argument(&args[i], "-c")?;
                    }
                }
                "-a" => {
                    config.all = true;
                }
                "-m" => {
                    i += 1;
                    if i < args.len() {
                        config.more_than = parse_usize_argument(&args[i], "-m")?;
                    }
                }
                "-i" => {
                    i += 1;
                    if i < args.len() {
                        config.ignore = args[i]
                            .split('|')
                            .map(|s| s.trim().to_string())
                            .collect();
                    }
                }
                "-b" => {
                    i += 1;
                    if i < args.len() {
                        config.bar_size = parse_usize_argument(&args[i], "-b")?;
                    }
                }
                "-n" => {
                    config.no_bar = true;
                }
                "-nh" => {
                    config.no_hist = true;
                }
                "-np" => {
                    config.no_perc = true;
                }
                "-nc" => {
                    config.no_cumu = true;
                }
                "-v" => {
                    config.verbose = true;
                }
                "-F" => {
                    config.fish_format = true;
                }
                "-s" | "--subcommands" => {
                    config.track_subcommands = true;
                }
                "-o" | "--output" => {
                    i += 1;
                    if i < args.len() {
                        config.output_format = OutputFormat::parse(&args[i])
                            .ok_or_else(|| format!("Invalid output format: {}. Use text, json, or csv", args[i]))?;
                    }
                }
                "--color" => {
                    i += 1;
                    if i < args.len() {
                        config.color_mode = ColorMode::parse(&args[i])
                            .ok_or_else(|| format!("Invalid color mode: {}. Use auto, always, or never", args[i]))?;
                    }
                }
                "--config" => {
                    i += 1;
                    if i < args.len() {
                        let file_config = FileConfig::load(Path::new(&args[i]))
                            .map_err(|e| format!("Failed to load config: {}", e))?;
                        config.apply_file_config(&file_config);
                    }
                }
                _ => {
                    return Err(format!("Invalid option: {}", args[i]));
                }
            }
            i += 1;
        }

        if config.file.is_empty() {
            config.file = match get_histfile() {
                Ok(s) => s,
                Err(_) => {
                    println!("Could not determine shell history file.");
                    process::exit(1);
                }
            };
        }

        Ok(config)
    }

    /// Apply settings from a file config (file settings don't override CLI)
    fn apply_file_config(&mut self, file_config: &FileConfig) {
        if let Some(ref ignore) = file_config.ignore {
            if self.ignore.is_empty() {
                self.ignore = ignore.clone();
            }
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
        if let Some(subcommands) = file_config.subcommands {
            self.track_subcommands = subcommands;
        }
        if let Some(more_than) = file_config.more_than {
            self.more_than = more_than;
        }
    }
}

fn parse_usize_argument(arg: &str, flag: &str) -> Result<usize, String> {
    match arg.parse::<usize>() {
        Ok(val) if val > 0 => Ok(val),
        _ => Err(format!(
            "Invalid {} argument, must be a positive integer",
            flag
        )),
    }
}

/// Get the history file path
///
/// Uses platform-specific detection:
/// - Linux: reads /proc/self/stat to find parent shell
/// - Other platforms: falls back to $SHELL environment variable
fn get_histfile() -> Result<String, Box<dyn Error>> {
    // First check HISTFILE environment variable
    if let Ok(histfile) = env::var("HISTFILE") {
        if let Ok(metadata) = fs::metadata(&histfile) {
            if metadata.is_file() {
                return Ok(histfile);
            }
        } else {
            eprintln!("HISTFILE does not exist");
            return Err("HISTFILE does not exist".into());
        }
    }

    let home = env::var("HOME").unwrap_or_default();
    let user = env::var("USER").unwrap_or_default();

    // Try to detect parent shell
    let shell = get_parent_shell()?;

    match shell.as_str() {
        "ash" => Ok(format!("/home/{}/.ash_history", user)),
        "bash" => Ok(format!("/home/{}/.bash_history", user)),
        "fish" => {
            let histfile = format!("{}/.local/share/fish/fish_history", home);
            if fs::metadata(&histfile).is_ok() {
                Ok(histfile)
            } else {
                Err(format!("Fish history not found at {}", histfile).into())
            }
        }
        "zsh" => {
            // Try XDG config location first
            let histfile = format!("/home/{}/.config/zsh/.zsh_history", user);
            if let Ok(metadata) = fs::metadata(&histfile) {
                if metadata.is_file() {
                    return Ok(histfile);
                }
            }
            // Fall back to home directory
            let histfile = format!("/home/{}/.zsh_history", user);
            if fs::metadata(&histfile).is_ok() {
                Ok(histfile)
            } else {
                Err("Zsh history not found".into())
            }
        }
        _ => {
            eprintln!("Unknown shell: {}", shell);
            Err("Unknown shell".into())
        }
    }
}

/// Get the parent shell name
///
/// Uses platform-specific detection
#[cfg(target_os = "linux")]
fn get_parent_shell() -> Result<String, Box<dyn Error>> {
    let stat_contents = fs::read_to_string("/proc/self/stat")?;
    let fields: Vec<&str> = stat_contents.split_whitespace().collect();
    let ppid = fields
        .get(3)
        .ok_or("Invalid /proc/self/stat format")?;

    let cmdline_file = PathBuf::from(format!("/proc/{}/cmdline", ppid));
    let cmdline_contents = fs::read_to_string(&cmdline_file)
        .map_err(|e| format!("Failed to read {}: {}", cmdline_file.display(), e))?;

    let parent_cmdline = Path::new(&cmdline_contents)
        .file_name()
        .and_then(|f| f.to_str())
        .map(|s| s.trim_end_matches('\0'))
        .ok_or("Failed to parse parent cmdline")?;

    Ok(parent_cmdline.to_string())
}

/// Fallback for non-Linux platforms: use $SHELL environment variable
#[cfg(not(target_os = "linux"))]
fn get_parent_shell() -> Result<String, Box<dyn Error>> {
    env::var("SHELL")
        .map_err(|_| "SHELL environment variable not set".into())
        .and_then(|shell| {
            Path::new(&shell)
                .file_name()
                .and_then(|f| f.to_str())
                .map(|s| s.to_string())
                .ok_or_else(|| "Failed to parse SHELL".into())
        })
}

fn print_help_message(count: usize, bar_size: usize) {
    println!(
        "Usage: histop [options]\n\
        \u{A0}-h, --help       Print this help message\n\
        \u{A0}-f <FILE>        Path to the history file\n\
        \u{A0}-c <COUNT>       Number of commands to print (default: {})\n\
        \u{A0}-a               Print all commands (overrides -c)\n\
        \u{A0}-m <MORE_THAN>   Only consider commands used more than <MORE_THAN> times\n\
        \u{A0}-i <IGNORE>      Ignore specified commands (e.g. \"ls|grep|nvim\")\n\
        \u{A0}-b <BAR_SIZE>    Size of the bar graph (default: {})\n\
        \u{A0}-n               Do not print the bar\n\
        \u{A0}-nh              Disable history mode (can be used for any data)\n\
        \u{A0}-np              Do not print the percentage in the bar\n\
        \u{A0}-nc              Do not print the inverse cumulative percentage in the bar\n\
        \u{A0}-v               Verbose\n\
        \u{A0}-F               Force fish history format parsing\n\
        \u{A0}-s, --subcommands  Track subcommands for git, cargo, npm, etc.\n\
        \u{A0}-o, --output <FMT> Output format: text (default), json, csv\n\
        \u{A0}--color <WHEN>   Color output: auto (default), always, never\n\
        \u{A0}--config <PATH>  Path to config file\n\
        \u{A0}██               Percentage\n\
        \u{A0}▓▓               Inverse cumulative percentage",
        count, bar_size
    );
}
