# cmdtop


<dl>
  <dt>Linux (x86_64)</dt><dd><a href="https://builds.sr.ht/~rodrgz/cmdtop/commits/main/alpine.yml"><img src="https://builds.sr.ht/~rodrgz/cmdtop/commits/main/alpine.yml.svg" alt="Build status for Linux" /></a></dd>
  <dt>FreeBSD (x86_64)</dt><dd><a href="https://builds.sr.ht/~rodrgz/cmdtop/commits/main/freebsd.yml"><img src="https://builds.sr.ht/~rodrgz/cmdtop/commits/main/freebsd.yml.svg" alt="Build status for FreeBSD" /></a></dd>
</dl>

This program analyzes a zsh/bash history file and tallies the frequency of each command, taking into account piped commands, while disregarding occurrences of environment variables, as well as the use of the "doas" and "sudo" commands.

## Usage

```
git clone https://git.sr.ht/~rodrgz/cmdtop
cd cmdtop
cargo build #or nix build
```

```
$ cmdtop -h 
Usage: cmdtop [OPTIONS]
-f <FILE>           Path to history file
-c <COUNT>          Number of commands to print [default: 25]
-a                  Print all commands
-m <MORE_THAN>      Only consider commands used more than <MORE_THAN> times [default: 1]
-i <IGNORE>         Ignore specified commands, e.g. "ls|grep|nvim"
-n                  Do not print bar graph
-b <BAR_SIZE>       Size of bar graph [default: 25]
-h, --help          Print this help message
```

## Example

```
$ cmdtop -f ~/.zsh_history -c 10
1441   │▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓███│ 13.79%    exa
995    │░░░░░▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓██│ 9.52%     z
910    │░░░░░░░▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓██│ 8.71%     hx
507    │░░░░░░░░░░░▓▓▓▓▓▓▓▓▓▓▓▓▓█│ 4.85%     rg
280    │░░░░░░░░░░░░░░▓▓▓▓▓▓▓▓▓▓█│ 2.68%     cargo
256    │░░░░░░░░░░░░░░░▓▓▓▓▓▓▓▓▓█│ 2.45%     nix
219    │░░░░░░░░░░░░░░░▓▓▓▓▓▓▓▓▓█│ 2.10%     dust
201    │░░░░░░░░░░░░░░░░░▓▓▓▓▓▓▓▓│ 1.92%     :q
190    │░░░░░░░░░░░░░░░░░▓▓▓▓▓▓▓▓│ 1.82%     git
188    │░░░░░░░░░░░░░░░░░▓▓▓▓▓▓▓▓│ 1.80%     cmdtop
```

## Requirements

1. Rust 1.46 or later
2. Cargo package manager
