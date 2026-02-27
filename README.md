# histop

[![Rust](https://github.com/rodrgz/histop/actions/workflows/rust.yml/badge.svg)](https://github.com/rodrgz/histop/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/histop.svg)](https://crates.io/crates/histop)
[![License](https://img.shields.io/crates/l/histop.svg)](https://github.com/rodrgz/histop/blob/main/LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-blue.svg)](https://github.com/rust-lang/rust)

Uncover the hidden gems of your command history! This program analyzes your shell history file and presents the most frequently used commands in a visually appealing and easy-to-understand format. With powerful options to filter out noise and focus on what matters.

## Supported Shells

- **Ash** (`~/.ash_history`)
- **Bash** (`~/.bash_history`)
- **Fish** (`~/.local/share/fish/fish_history`)
- **PowerShell** (`~/.local/share/powershell/PSReadLine/ConsoleHost_history.txt`)
- **Tcsh** (`~/.history`, `~/.tcsh_history`, `~/.csh_history`)
- **Zsh** (`~/.zsh_history` or `~/.config/zsh/.zsh_history`)

## Usage

```
git clone https://git.sr.ht/~rodrgz/histop
cd histop
cargo build #or nix build
```

```
$ histop -h 
Usage: histop [options] [FILE]
 -h, --help       Print this help message
 -f <FILE>        Path to the history file (or pass FILE positionally)
 -c <COUNT>       Number of commands to print (default: 25)
 -a               Print all commands (overrides -c)
 -m <MORE_THAN>   Only consider commands used more than <MORE_THAN> times
 -i <IGNORE>      Ignore specified commands (e.g. "ls|grep|nvim")
 -b <BAR_SIZE>    Size of the bar graph (default: 25)
 -n               Do not print the bar
 -nh              Disable history mode (can be used for any data)
 -np              Do not print the percentage in the bar
 -nc              Do not print the inverse cumulative percentage in the bar
 -o <FMT>         Output format: text (default), json, csv
 --color <WHEN>   Color output: auto (default), always, never
 --config <PATH>  Path to config file
 ██               Percentage
 ▓▓               Inverse cumulative percentage
```

## Using `-nh` with stdin

When `-nh` is set and no `-f` is provided, `histop` reads from `stdin` if
input is piped or redirected.

```bash
histop -nh < arquivo.txt
cat arquivo.txt | histop -nh
```

If `stdin` is a terminal (not piped), `histop` falls back to shell history
detection as usual.

## Example

```
$ histop -c 10 -i "cd"
1184   │▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓█████│ 19.08%   ls
 943   │░░░░░▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓████│ 15.19%   nvim
 792   │░░░░░░░░░▓▓▓▓▓▓▓▓▓▓▓▓▓███│ 12.76%   git
 670   │░░░░░░░░░░░▓▓▓▓▓▓▓▓▓▓▓███│ 10.80%   nix
 311   │░░░░░░░░░░░░░░░▓▓▓▓▓▓▓▓▓█│  5.01%   rg
 310   │░░░░░░░░░░░░░░░░▓▓▓▓▓▓▓▓█│  5.00%   exit
 255   │░░░░░░░░░░░░░░░░░▓▓▓▓▓▓▓█│  4.11%   dust
 253   │░░░░░░░░░░░░░░░░░░▓▓▓▓▓▓█│  4.08%   histop
 233   │░░░░░░░░░░░░░░░░░░░▓▓▓▓▓█│  3.75%   cargo
 219   │░░░░░░░░░░░░░░░░░░░░▓▓▓▓█│  3.53%   man
```

## Requirements

1. Rust 1.85 or later (edition 2024)
2. Cargo package manager
