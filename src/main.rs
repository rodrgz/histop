use std::{
    cmp,
    collections::HashMap,
    env, fs,
    io::{BufRead, BufReader},
    process,
};

// Struct to hold command name and count
struct Command<'a> {
    name: &'a str,
    count: usize,
}

fn main() {
    let args = match parse_args() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    let (hist_file, count, all, more_than, ignore, no_bar, bar_size, no_log, no_perc) = args;
    let mut filtered_commands = vec!["sudo", "doas"];
    filtered_commands.extend(ignore.split('|').map(|s| s.trim()).collect::<Vec<_>>());

    // Read history file
    let hist_file = match fs::File::open(&hist_file) {
        Ok(hist_file) => hist_file,
        // Handle file open error
        Err(e) => {
            eprintln!("Error opening history file: {}", e);
            return;
        }
    };
    let reader = BufReader::new(hist_file);

    // Initialize a hashmap to hold command count
    let mut cmd_count: HashMap<String, usize> = HashMap::new();

    // Initialize variables to hold command and skip flag
    let mut skip = false;

    // Iterate over lines in the history file
    for line_result in reader.lines() {
        let line = match line_result {
            Ok(line) => line,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::InvalidData {
                    eprintln!("Non-UTF-8 character detected in input stream, skipping line");
                    continue;
                } else {
                    eprintln!("Error reading history file: {}", e);
                    break;
                }
            }
        };

        match (skip, line.starts_with(": "), line.ends_with("\\")) {
            (false, false, false) => {
                count_commands(&mut cmd_count, &line, &filtered_commands);
            }
            (false, false, true) => {
                count_commands(&mut cmd_count, &line, &filtered_commands);
                skip = true;
            }
            (false, true, _) => {
                skip = true;
            }
            (true, _, true) => {
                skip = true;
            }
            (true, _, false) => {
                skip = false;
            }
        }
    }

    let mut commands = Vec::with_capacity(cmd_count.len());
    for (name, count) in &cmd_count {
        commands.push(Command {
            name,
            count: *count,
        });
    }

    commands.retain(|cmd| cmd.count > more_than);
    commands.sort_by_key(|cmd| cmp::Reverse(cmd.count));

    let n = if all {
        commands.len()
    } else {
        cmp::min(count, commands.len())
    };

    let total_count: usize = commands.iter().map(|cmd| cmd.count).sum();
    let mut total_percentage = 0.0;

    for (cmd, _) in commands.iter().zip(0..n) {
        let percentage = cmd.count as f32 / total_count as f32 * 100.0;
        total_percentage += percentage;

        print!("{: <7}", cmd.count);
        if !no_bar && bar_size > 0 {
            print_bar(percentage, total_percentage, bar_size, no_log, no_perc);
        }
        println!(" {:<9} {:<}", format!("{:.2}%", percentage), cmd.name);
    }
}

fn count_commands(cmd_count: &mut HashMap<String, usize>, line: &str, filtered_commands: &[&str]) {
    if line.contains("|") {
        let cleaned_line = clean_line(line);
        for subcommand in cleaned_line.split('|') {
            let first_word = get_first_word(subcommand, filtered_commands);
            if !first_word.is_empty() {
                cmd_count
                    .entry(first_word.to_string())
                    .and_modify(|count| *count += 1)
                    .or_default();
            }
        }
    } else {
        let first_word = get_first_word(line, filtered_commands);
        if !first_word.is_empty() {
            cmd_count
                .entry(first_word.to_string())
                .and_modify(|count| *count += 1)
                .or_default();
        }
    }
}

fn clean_line(line: &str) -> String {
    let mut cleaned_line = line.clone().to_string();
    let mut in_quotes = false;
    for c in line.chars() {
        if c == '\'' || c == '\"' {
            in_quotes = !in_quotes;
        } else if c == '|' && in_quotes {
            cleaned_line.push(' ');
        }
    }
    cleaned_line
}

fn get_first_word<'a>(subcommand: &'a str, filtered_commands: &[&str]) -> &'a str {
    for w in subcommand.trim().split_whitespace() {
        if filtered_commands.contains(&w) || w.contains('=') {
            continue;
        } else if w.starts_with('\\') && w.len() > 1 {
            if filtered_commands.contains(&&w[1..]) {
                continue;
            } else {
                return &w[1..];
            }
        } else {
            return w;
        }
    }
    ""
}

fn print_bar(percentage: f32, total_percentage: f32, bar_size: usize, no_log: bool, no_perc: bool) {
    let mut semifilled_count = 0;
    let mut filled_count = 0;
    let mut unfilled_count = 0;

    let decimal = percentage / 100.0;

    match (!no_log, !no_perc) {
        (true, true) => {
            let scaled_percentage = percentage * 100.0 / total_percentage;
            let scaled_log_decimal = (scaled_percentage + 1.0).ln() / 100_f32.ln();
            semifilled_count = ((scaled_log_decimal - decimal) * bar_size as f32).round() as usize;
            filled_count = (decimal * bar_size as f32).round() as usize;
            if filled_count + semifilled_count > bar_size {
                semifilled_count = bar_size - filled_count;
            };
            unfilled_count = (bar_size - filled_count - semifilled_count).min(bar_size);
        }
        (false, true) => {
            filled_count = (decimal * bar_size as f32).round() as usize;
            unfilled_count = (bar_size - filled_count).min(bar_size);
            semifilled_count = 0;
        }
        (true, false) => {
            let scaled_percentage = percentage * 100.0 / total_percentage;
            let scaled_log_decimal = (scaled_percentage + 1.0).ln() / 100_f32.ln();
            filled_count = (scaled_log_decimal * bar_size as f32).round() as usize;
            unfilled_count = (bar_size - filled_count).min(bar_size);
        }
        (_, _) => {}
    }

    if unfilled_count + semifilled_count + filled_count > 0 {
        print!(
            "│{}{}{}│",
            "░".repeat(unfilled_count),
            "▓".repeat(semifilled_count),
            "█".repeat(filled_count)
        );
    }
}

fn parse_args() -> Result<(String, usize, bool, usize, String, bool, usize, bool, bool), String> {
    let args: Vec<String> = env::args().collect();

    let mut file = &get_histfile();
    let mut ignore = String::new();
    let mut all = false;
    let mut no_bar = false;
    let mut no_log = false;
    let mut no_perc = false;
    let mut bar_size: usize = 25;
    let mut count: usize = 25;
    let mut more_than: usize = 0;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-f" => {
                i += 1;
                if i < args.len() {
                    file = &args[i];
                }
            }
            "-c" => {
                i += 1;
                if i < args.len() {
                    count = parse_usize_argument(&args[i], "-c")?;
                }
            }
            "-a" => {
                all = true;
            }
            "-m" => {
                i += 1;
                if i < args.len() {
                    more_than = parse_usize_argument(&args[i], "-m")?;
                }
            }
            "-i" => {
                i += 1;
                if i < args.len() {
                    ignore = args[i].clone();
                }
            }
            "-n" => {
                if i < args.len() {
                    no_bar = true;
                }
            }
            "-nl" => {
                if i < args.len() {
                    no_log = true;
                }
            }
            "-np" => {
                if i < args.len() {
                    no_perc = true;
                }
            }

            "-b" => {
                i += 1;
                if i < args.len() {
                    bar_size = parse_usize_argument(&args[i], "-b")?;
                }
            }
            "-h" | "--help" => {
                print_help_message(count, bar_size);
                process::exit(0);
            }
            _ => {
                return Err(format!("Invalid option: {}", args[i]));
            }
        }
        i += 1;
    }

    Ok((
        file.to_string(),
        count,
        all,
        more_than,
        ignore,
        no_bar,
        bar_size,
        no_log,
        no_perc,
    ))
}

fn parse_usize_argument(arg: &str, flag: &str) -> Result<usize, String> {
    match arg.parse() {
        Ok(val) if val >= 1 || flag == "-m" => Ok(val),
        Ok(_) => Err(format!("Error: {} needs to be positive", flag)),
        Err(_) => Err(format!("Error: Invalid {} argument", flag)),
    }
}

fn get_histfile() -> String {
    env::var("HISTFILE").unwrap_or_else(|_| {
        let user = env::var("USER").unwrap_or_default();
        let shell = env::var("SHELL").unwrap_or_default();
        if shell.ends_with("zsh") {
            match fs::metadata(format!("/home/{}/.config/zsh/.zsh_history", user)) {
                Ok(_) => format!("/home/{}/.config/zsh/.zsh_history", user),
                Err(_) => format!("/home/{}/.zsh_history", user),
            }
        } else if shell.ends_with("bash") {
            format!("/home/{}/.bash_history", user)
        } else {
            String::new()
        }
    })
}

fn print_help_message(count: usize, bar_size: usize) {
    println!("Usage: histop [OPTIONS]");
    println!("-f <FILE>           Path to history file");
    println!(
        "-c <COUNT>          Number of commands to print [default: {}]",
        count
    );
    println!("-a                  Print all commands");
    println!("-m <MORE_THAN>      Only consider commands used more than <MORE_THAN> times");
    println!("-i <IGNORE>         Ignore specified commands, e.g. \"ls|grep|nvim\"");
    println!("-n                  Do not print bar graph");
    println!("-np                 Do not print percentage");
    println!("-nl                 Do not print logarithmically scaled percentage");
    println!(
        "-b <BAR_SIZE>       Size of bar graph [default: {}]",
        bar_size
    );
    println!("-h, --help          Print this help message");
    println!("▓▓                  Logarithmically Scaled Percentage");
    println!("██                  Percentage");
}
