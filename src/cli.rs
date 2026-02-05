//! CLI argument parsing and configuration

use std::{env, error::Error, fs, path::Path, path::PathBuf, process};

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
        }
    }
}

impl Config {
    /// Parse configuration from command line arguments
    pub fn from_args() -> Result<Self, String> {
        let args: Vec<String> = env::args().collect();
        let mut config = Config::default();

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

fn get_histfile() -> Result<String, Box<dyn Error>> {
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

    match parent_cmdline {
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
            let histfile = format!("/home/{}/.config/zsh/.zsh_history", user);
            if let Ok(metadata) = fs::metadata(&histfile) {
                if metadata.is_file() {
                    Ok(histfile)
                } else {
                    Ok(format!("/home/{}/.zsh_history", user))
                }
            } else {
                Err(format!("Could not read metadata for {}", histfile).into())
            }
        }
        _ => {
            eprintln!("Unknown shell: {}", parent_cmdline);
            Err("Unknown shell".into())
        }
    }
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
        \u{A0}██               Percentage\n\
        \u{A0}▓▓               Inverse cumulative percentage",
        count, bar_size
    );
}
