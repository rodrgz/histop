use std::cmp::min;
use std::collections::HashMap;
use std::fs;

// Struct to hold command name and count
struct Command<'a> {
    name: &'a str,
    count: usize,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut file = &get_histfile();
    let mut count = 25;
    let mut all = false;
    let mut more_than = 1;
    let mut ignore = String::new();
    let mut no_bar = false;
    let mut bar_size = 25;

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
                    count = args[i].parse().unwrap_or(25);
                }
            }
            "-a" => {
                all = true;
            }
            "-m" => {
                i += 1;
                if i < args.len() {
                    more_than = args[i].parse().unwrap_or(1);
                }
            }
            "-i" => {
                i += 1;
                if i < args.len() {
                    ignore = args[i].clone();
                }
            }
            "-n" => {
                no_bar = true;
            }
            "-b" => {
                i += 1;
                if i < args.len() {
                    bar_size = args[i].parse().unwrap_or(25);
                }
            }
            "-h" | "--help" => {
                println!("Usage: cmdtop [OPTIONS]");
                println!("-f <FILE>           Path to history file");
                println!("-c <COUNT>          Number of commands to print [default: 25]");
                println!("-a                  Print all commands");
                println!("-m <MORE_THAN>      Only consider commands used more than <MORE_THAN> times [default: 1]");
                println!("-i <IGNORE>         Ignore specified commands, e.g. \"ls|grep|nvim\"");
                println!("-n                  Do not print bar graph");
                println!("-b <BAR_SIZE>       Size of bar graph [default: 25]");
                println!("-h, --help          Print this help message");
                return;
            }
            _ => {
                println!("Invalid option: {}", args[i]);
                return;
            }
        }
        i += 1;
    }

    let mut filtered_commands = vec!["sudo", "doas"];
    filtered_commands.extend(ignore.split('|').map(|s| s.trim()).collect::<Vec<_>>());

    // Initialize a hashmap to hold command count
    let mut cmd_count: HashMap<String, usize> = HashMap::new();

    // Read history file
    let history = match fs::read_to_string(file) {
        Ok(file) => file,
        // Handle file read error
        Err(e) => {
            eprintln!("Error reading history file: {}", e);
            return;
        }
    };
    if history.is_empty() {
        println!("History file is empty");
        return;
    }

    // Initialize variables to hold command and skip flag
    let mut skip = false;

    // Iterate over lines in the history file
    for line in history.lines() {
        match (skip, line.starts_with(": "), line.ends_with("\\")) {
            (false, false, false) => {
                count_commands(&mut cmd_count, line, &filtered_commands);
            }
            (false, false, true) => {
                count_commands(&mut cmd_count, line, &filtered_commands);
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

    commands.retain(|cmd| cmd.count >= more_than);
    commands.sort_by(|a, b| b.count.cmp(&a.count));

    let n = if all {
        commands.len()
    } else {
        min(count, commands.len())
    };

    let total_count = commands.iter().fold(0, |acc, cmd| acc + cmd.count);
    let mut total_percentage = 0.0;

    for (cmd, _) in commands.iter().zip(0..n) {
        let percentage = cmd.count as f32 / total_count as f32 * 100.0;
        total_percentage += percentage;
        let normalized_percentages = percentage * 100.0 / total_percentage;

        print!("{: <7}", cmd.count);
        if !no_bar {
            print_bar(normalized_percentages, percentage, bar_size);
        }
        println!(" {:<9} {:<}", format!("{:.2}%", percentage), cmd.name);
    }
}

fn get_histfile() -> String {
    std::env::var("HISTFILE").unwrap_or_else(|_| {
        let user = std::env::var("USER").unwrap_or_default();
        let shell = std::env::var("SHELL").unwrap_or_default();
        if shell.ends_with("zsh") {
            match std::fs::metadata(format!("/home/{}/.config/zsh/.zsh_history", user)) {
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

fn count_commands(cmd_count: &mut HashMap<String, usize>, line: &str, filtered_commands: &[&str]) {
    let cleaned_line = clean_line(line);
    for subcommand in cleaned_line.split('|') {
        let first_word = get_first_word(subcommand, filtered_commands);
        if !first_word.is_empty() {
            *cmd_count.entry(first_word.to_string()).or_default() += 1;
        }
    }
}

fn clean_line(line: &str) -> String {
    let mut cleaned_line = String::new();
    let mut in_quotes = false;
    for c in line.chars() {
        if c == '\'' || c == '\"' {
            in_quotes = !in_quotes;
        } else if c == '|' && in_quotes {
            cleaned_line.push(' ');
        } else {
            cleaned_line.push(c);
        }
    }
    cleaned_line
}

fn get_first_word<'a>(subcommand: &'a str, filtered_commands: &[&str]) -> &'a str {
    for w in subcommand.trim().split_whitespace() {
        if filtered_commands.contains(&w) {
            break;
        }
        if w.contains('=') {
            continue;
        } else if w.starts_with('\\') {
            if w.len() > 1 {
                if filtered_commands.contains(&&w[1..]) {
                    continue;
                } else {
                    return &w[1..];
                }
            }
        } else {
            if filtered_commands.contains(&w) || w.contains('=') {
                continue;
            } else {
                return w;
            }
        }
    }
    ""
}

fn print_bar(normalized_percentage: f32, percentage: f32, bar_size: usize) {
    let log_normalized_percentage = (normalized_percentage + 1.0).ln() / 101.0_f32.ln();

    let filled_count = (percentage / 100.0 * bar_size as f32).round() as usize;
    let semifilled_count =
        ((log_normalized_percentage - percentage / 100.0) * bar_size as f32).round() as usize;
    let unfilled_count = bar_size.saturating_sub(filled_count + semifilled_count);

    print!(
        "│{}{}{}│",
        "░".repeat(unfilled_count),
        "▓".repeat(semifilled_count),
        "█".repeat(filled_count)
    );
}
