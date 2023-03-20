use std::{
    cmp,
    collections::HashMap,
    env, fs,
    io::{BufRead, BufReader},
    process,
};

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
    let (
        hist_file,
        count,
        all,
        more_than,
        ignore,
        bar_size,
        no_bar,
        no_cumu,
        no_perc,
    ) = args;

    let hist_file = match fs::File::open(&hist_file) {
        Ok(hist_file) => hist_file,
        Err(e) => {
            eprintln!("Error opening history file: {}", e);
            return;
        }
    };
    let reader = BufReader::new(hist_file);

    let mut filtered_commands = vec!["sudo", "doas"];
    filtered_commands
        .extend(ignore.split('|').map(|s| s.trim()).collect::<Vec<_>>());

    let mut skip = false;
    let mut non_utf8 = false;
    let mut cmd_count: HashMap<String, usize> = HashMap::new();

    for line_result in reader.lines() {
        let line = match line_result {
            Ok(line) => line,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::InvalidData {
                    if !non_utf8 {
                        eprintln!("Non-UTF-8 character detected in input stream, skipping line");
                        non_utf8 = true;
                    }
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
        commands.push(Command { name, count: *count });
    }

    commands.retain(|cmd| cmd.count > more_than);
    commands.sort_by_key(|cmd| cmp::Reverse(cmd.count));

    let total_count: usize = commands.iter().map(|cmd| cmd.count).sum();
    let padding_str = " ".repeat(3);
    let mut padding_count = 0;
    let mut padding_perc = 0;
    let mut old_count_len = 0;
    let mut old_perc_len = 0;
    let mut inv_cum_perc = 100.0;
    let mut first_loop = true;

    let n = if all { commands.len() } else { cmp::min(count, commands.len()) };

    for (cmd, _) in commands.iter().zip(0..n) {
        let perc = cmd.count as f32 / total_count as f32 * 100.0;
        let perc_formated = format!("{:.2}%", perc);

        if first_loop {
            old_count_len = cmd.count.to_string().len();
            old_perc_len = perc_formated.len();
            first_loop = false;
        }

        let count_len = cmd.count.to_string().len();
        let perc_len = perc_formated.len();
        let diff_count = old_count_len - count_len;
        let diff_perc = old_perc_len - perc_len;

        padding_count = padding_count.max(diff_count);
        padding_perc = padding_perc.max(diff_perc);
        old_count_len = count_len + diff_count;
        old_perc_len = perc_len;

        print!("{}{}{}", " ".repeat(padding_count), cmd.count, padding_str,);

        if !no_bar && bar_size > 0 {
            print_bar(perc, inv_cum_perc, bar_size, no_cumu, no_perc);
            inv_cum_perc -= perc;
        }

        print!(
            "{}{}{}{}",
            " ".repeat(padding_perc),
            perc_formated,
            padding_str,
            cmd.name,
        );
        println!();
    }
}

fn count_commands(
    cmd_count: &mut HashMap<String, usize>,
    line: &str,
    filtered_commands: &[&str],
) {
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
    let mut in_quotes = false;
    let mut cleaned_line = line.clone().to_string();
    for c in line.chars() {
        if c == '\'' || c == '\"' {
            in_quotes = !in_quotes;
        } else if c == '|' && in_quotes {
            cleaned_line.push(' ');
        }
    }
    cleaned_line
}

fn get_first_word<'a>(
    subcommand: &'a str,
    filtered_commands: &[&str],
) -> &'a str {
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

fn print_bar(
    perc: f32,
    inv_cumu_perc: f32,
    bar_size: usize,
    no_cumu: bool,
    no_perc: bool,
) {
    let dec = perc / 100.0;
    let inv_cumu_dec = inv_cumu_perc / 100.0;
    let mut semifilled_count = 0;
    let mut filled_count = 0;
    let mut unfilled_count = 0;

    match (!no_cumu, !no_perc) {
        (true, true) => {
            semifilled_count =
                ((inv_cumu_dec - dec) * bar_size as f32).round() as usize;
            filled_count = (dec * bar_size as f32).round() as usize;
            if filled_count + semifilled_count > bar_size {
                semifilled_count = bar_size - filled_count;
            };
            unfilled_count =
                (bar_size - filled_count - semifilled_count).min(bar_size);
        }
        (false, true) => {
            filled_count = (dec * bar_size as f32).round() as usize;
            unfilled_count = (bar_size - filled_count).min(bar_size);
        }
        (true, false) => {
            semifilled_count =
                (inv_cumu_dec * bar_size as f32).round() as usize;
            unfilled_count = (bar_size - semifilled_count).min(bar_size);
        }
        (_, _) => {}
    }

    if unfilled_count + semifilled_count + filled_count > 0 {
        print!(
            "│{}{}{}│ ",
            "░".repeat(unfilled_count),
            "▓".repeat(semifilled_count),
            "█".repeat(filled_count)
        );
    }
}

fn parse_args(
) -> Result<(String, usize, bool, usize, String, usize, bool, bool, bool), String>
{
    let args: Vec<String> = env::args().collect();
    let mut all = false;
    let mut no_bar = false;
    let mut no_cumu = false;
    let mut no_perc = false;
    let mut bar_size: usize = 25;
    let mut count: usize = 25;
    let mut more_than: usize = 0;
    let mut file = &get_histfile();
    let mut ignore = String::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_help_message(count, bar_size);
                process::exit(0);
            }
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
            "-b" => {
                i += 1;
                if i < args.len() {
                    bar_size = parse_usize_argument(&args[i], "-b")?;
                }
            }
            "-n" => {
                if i < args.len() {
                    no_bar = true;
                }
            }
            "-np" => {
                if i < args.len() {
                    no_perc = true;
                }
            }
            "-nc" => {
                if i < args.len() {
                    no_cumu = true;
                }
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
        bar_size,
        no_bar,
        no_cumu,
        no_perc,
    ))
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

fn get_histfile() -> String {
    env::var("HISTFILE").unwrap_or_else(|_| {
        let user = env::var("USER").unwrap_or_default();
        let shell = env::var("SHELL").unwrap_or_default();
        if shell.ends_with("zsh") {
            match fs::metadata(format!(
                "/home/{}/.config/zsh/.zsh_history",
                user
            )) {
                Ok(_) => format!("/home/{}/.config/zsh/.zsh_history", user),
                Err(_) => {
                    format!("/home/{}/.zsh_history", user)
                }
            }
        } else if shell.ends_with("bash") {
            format!("/home/{}/.bash_history", user)
        } else {
            String::new()
        }
    })
}

fn print_help_message(
    count: usize,
    bar_size: usize,
) {
    println!( "Usage: histop [options]\n\
        \u{A0}-h, --help       Print this help message\n\
        \u{A0}-f <FILE>        Path to the history file\n\
        \u{A0}-c <COUNT>       Number of commands to print (default: {})\n\
        \u{A0}-a               Print all commands (overrides -c)\n\
        \u{A0}-m <MORE_THAN>   Only consider commands used more than <MORE_THAN> times\n\
        \u{A0}-i <IGNORE>      Ignore specified commands (e.g. \"ls|grep|nvim\")\n\
        \u{A0}-b <BAR_SIZE>    Size of the bar graph (default: {})\n\
        \u{A0}-n               Do not print the bar\n\
        \u{A0}-np              Do not print the percentage in the bar\n\
        \u{A0}-nc              Do not print the inverse cumulative percentage in the bar\n\
        \u{A0}██               Percentage\n\
        \u{A0}▓▓               Inverse cumulative percentage",
        count, bar_size);
}
