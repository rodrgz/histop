# histop

<dl>
  <dt>Linux (x86_64)</dt><dd><a href="https://builds.sr.ht/~rodrgz/histop/commits/main/alpine.yml"><img src="https://builds.sr.ht/~rodrgz/histop/commits/main/alpine.yml.svg" alt="Build status for Linux" /></a></dd>
  <dt>FreeBSD (x86_64)</dt><dd><a href="https://builds.sr.ht/~rodrgz/histop/commits/main/freebsd.yml"><img src="https://builds.sr.ht/~rodrgz/histop/commits/main/freebsd.yml.svg" alt="Build status for FreeBSD" /></a></dd>
</dl>

Uncover the hidden gems of your command history! This nifty program analyzes your shell history file and presents the most frequently used commands in a visually appealing and easy-to-understand format. With powerful options to filter out noise and focus on what matters.

## Supported Shells

- **Bash** (`~/.bash_history`)
- **Zsh** (`~/.zsh_history` or `~/.config/zsh/.zsh_history`)
- **Ash** (`~/.ash_history`)
- **Fish** (`~/.local/share/fish/fish_history`) — native support!

## Usage

```
git clone https://git.sr.ht/~rodrgz/histop
cd histop
cargo build #or nix build
```

```
$ histop -h 
Usage: histop [options]
 -h, --help       Print this help message
 -f <FILE>        Path to the history file
 -c <COUNT>       Number of commands to print (default: 25)
 -a               Print all commands (overrides -c)
 -m <MORE_THAN>   Only consider commands used more than <MORE_THAN> times
 -i <IGNORE>      Ignore specified commands (e.g. "ls|grep|nvim")
 -b <BAR_SIZE>    Size of the bar graph (default: 25)
 -n               Do not print the bar
 -nh              Disable history mode (can be used for any data)
 -np              Do not print the percentage in the bar
 -nc              Do not print the inverse cumulative percentage in the bar
 ██               Percentage
 ▓▓               Inverse cumulative percentage
```

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

## Fish Shell

Fish is now natively supported! Just run `histop` from a Fish shell, or use `-f` to point to your fish_history:

```
$ histop -f ~/.local/share/fish/fish_history
```

## Testing

### Running All Tests

```bash
cargo test
```

This runs all tests:
- **Unit tests** (47 tests) — test individual functions in each module
- **Integration tests** (10 tests) — test history parsing with fixture files
- **CLI tests** (50 tests) — test all command-line options end-to-end

### Running Specific Test Suites

```bash
# Unit tests only (in src/)
cargo test --lib

# Integration tests only
cargo test --test integration_test

# CLI tests only (requires binary to be built first)
cargo build && cargo test --test cli_test
```

### Test Fixtures

Test fixtures are located in `tests/fixtures/`:
- `bash_history` — sample bash history file
- `zsh_history` — sample zsh extended history file  
- `fish_history` — sample fish history file (YAML format)

## Requirements

1. Rust 1.85 or later (edition 2024)
2. Cargo package manager

## Common Errors

- `Could not determine shell history file`: use `-f <FILE>` explicitly or configure `HISTFILE`.
- `Missing value for ...`: one of the options requiring a parameter was passed sem valor.
- `Invalid ... argument, must be a positive integer`: revise numeric values for `-c`, `-b` e `-m`.
