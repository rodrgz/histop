# cmdtop


<dl>
  <dt>Linux (x86_64)</dt><dd><a href="https://builds.sr.ht/~rodrgz/cmdtop/commits/main/alpine.yml"><img src="https://builds.sr.ht/~rodrgz/cmdtop/commits/main/alpine.yml.svg" alt="Build status for Linux" /></a></dd>
  <dt>FreeBSD (x86_64)</dt><dd><a href="https://builds.sr.ht/~rodrgz/cmdtop/commits/main/freebsd.yml"><img src="https://builds.sr.ht/~rodrgz/cmdtop/commits/main/freebsd.yml.svg" alt="Build status for FreeBSD" /></a></dd>
</dl>

This program reads a zsh/bash history file and counts the number of occurrences of each command, ignoring environment variables, the "doas" and "sudo" commands. The output is sorted by the number of occurrences in descending order and can be limited by the number of commands to display. The output is sorted by the number of occurrences in descending order. The program takes two optional command line arguments:

1. The path to the zsh/bash history file (default: $HISTFILE)
2. The number of commands to display (default: 25)

On my computer, it took 3ms to process 20000 commands!

## Usage

```
git clone https://git.sr.ht/~rodrgz/cmdtop
cd cmdtop
cargo build #or nix build
```

## Example

```
$ cmdtop -f ~/.zsh_history -c 10
  COUNT     PERC        CMD
  294       19.50%      nvim
  184       12.20%      nix
  92        6.10%       ls
  89        5.90%       cd
  79        5.24%       git
  58        3.85%       ./result/bin/cmdtop
  56        3.71%       hyperfine
  52        3.45%       gcc
  41        2.72%       ,
  36        2.39%       lazygit
```

This command runs the program with the specified zsh history file and displays the top 10 commands.

## Requirements

1. Rust 1.46 or later
2. Cargo package manager

## Note

This program assumes that the zsh/bash history file is formatted with one command per line and commands that span multiple lines are continued with a "\\" at the end of each line, except for the last line. It ignores environment variables, as well as the words "doas" and "sudo."
