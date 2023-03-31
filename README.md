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
Usage: histop [options]
 -h, --help       Print this help message
 -f <FILE>        Path to the history file
 -c <COUNT>       Number of commands to print (default: 25)
 -a               Print all commands (overrides -c)
 -m <MORE_THAN>   Only consider commands used more than <MORE_THAN> times
 -i <IGNORE>      Ignore specified commands (e.g. "ls|grep|nvim")
 -b <BAR_SIZE>    Size of the bar graph (default: 25)
 -n               Do not print the bar
 -np              Do not print the percentage in the bar
 -nc              Do not print the inverse cumulative percentage in the bar
 -v               Verbose
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

## Requirements

1. Rust 1.46 or later
2. Cargo package manager
