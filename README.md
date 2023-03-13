# histop

<dl>
  <dt>Linux (x86_64)</dt><dd><a href="https://builds.sr.ht/~rodrgz/histop/commits/main/alpine.yml"><img src="https://builds.sr.ht/~rodrgz/histop/commits/main/alpine.yml.svg" alt="Build status for Linux" /></a></dd>
  <dt>FreeBSD (x86_64)</dt><dd><a href="https://builds.sr.ht/~rodrgz/histop/commits/main/freebsd.yml"><img src="https://builds.sr.ht/~rodrgz/histop/commits/main/freebsd.yml.svg" alt="Build status for FreeBSD" /></a></dd>
</dl>

Uncover the hidden gems of your command history! This nifty program analyzes your zsh/bash history file and presents the most frequently used commands in a visually appealing and easy-to-understand format. With powerful options to filter out noise and focus on what matters.

## Usage

```
git clone https://git.sr.ht/~rodrgz/histop
cd histop
cargo build #or nix build
```

```
$ histop -h 
Usage: histop [OPTIONS]
-f <FILE>           Path to history file
-c <COUNT>          Number of commands to print [default: 25]
-a                  Print all commands
-m <MORE_THAN>      Only consider commands used more than <MORE_THAN> times
-i <IGNORE>         Ignore specified commands, e.g. "ls|grep|nvim"
-n                  Do not print bar graph
-np                 Do not print percentage
-nc                 Do not print cumulative percentage
-b <BAR_SIZE>       Size of bar graph [default: 25]
-h, --help          Print this help message
▓▓                  Cumulative Percentage
██                  Percentage
```

## Example

```
$ histop -f ~/.zsh_history -c 10
1441   │▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓███│ 13.79%    exa
995    │░░░░░▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓██│ 9.52%     z
910    │░░░░░░░▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓██│ 8.71%     hx
507    │░░░░░░░░░░░▓▓▓▓▓▓▓▓▓▓▓▓▓█│ 4.85%     rg
280    │░░░░░░░░░░░░░░▓▓▓▓▓▓▓▓▓▓█│ 2.68%     cargo
256    │░░░░░░░░░░░░░░░▓▓▓▓▓▓▓▓▓█│ 2.45%     nix
219    │░░░░░░░░░░░░░░░▓▓▓▓▓▓▓▓▓█│ 2.10%     dust
201    │░░░░░░░░░░░░░░░░░▓▓▓▓▓▓▓▓│ 1.92%     :q
190    │░░░░░░░░░░░░░░░░░▓▓▓▓▓▓▓▓│ 1.82%     git
188    │░░░░░░░░░░░░░░░░░▓▓▓▓▓▓▓▓│ 1.80%     histop
```

## Requirements

1. Rust 1.46 or later
2. Cargo package manager
