# Directory setup
PGO_DATA="pgo-data"
rm -rf "$PGO_DATA"
mkdir -p "$PGO_DATA"

BASH_HISTORY="bash_history_pgo"
FISH_HISTORY="fish_history_pgo"

echo "Generating synthetic workload..."

# Generate bash history
# 50k lines of random commands including corner cases
python3 - <<EOF
import random
BASH_HISTORY = "$BASH_HISTORY"
base_commands = ["ls -la", "cd ..", "git status", "cargo build", "vim src/main.rs", "grep -r foo .", "find . -name \"*.rs\"", "cat Cargo.toml", "echo hello", "rm -rf target", "docker ps", "kubectl get pods"]
wrappers = ["sudo", "doas", "time", "watch"]
env_vars = ["FOO=bar", "Debug=1", "RUST_BACKTRACE=1"]
complex_patterns = [
    "{} | grep foo", 
    "{} | xargs echo",
    "echo \"{} | pipe inside quotes\"",
    "echo '{} | pipe inside single quotes'",
    "\\\\{}", # Escaped command
    "{} --flag",
    "{} -- --separated args" # Double dash
]

with open(BASH_HISTORY, "w") as f:
    for _ in range(50000):
        cmd = random.choice(base_commands)
        
        # 30% chance to wrap with sudo/doas/etc
        if random.random() < 0.3:
            cmd = f"{random.choice(wrappers)} {cmd}"
            
        # 20% chance to add env vars
        if random.random() < 0.2:
            cmd = f"{random.choice(env_vars)} {cmd}"
            
        # 20% chance to make it complex (pipes, quotes, etc)
        if random.random() < 0.2:
            pattern = random.choice(complex_patterns)
            # Simple format replacement
            cmd = pattern.format(cmd)
            
        f.write(cmd + "\n")
EOF

# Generate fish history
# 50k entries in YAML-like format
python3 - <<EOF
import random
import time
FISH_HISTORY = "$FISH_HISTORY"
commands = ["ls -la", "cd ..", "git status", "cargo build", "vim src/main.rs", "grep -r foo .", "find . -name \"*.rs\"", "cat Cargo.toml", "echo hello", "nix develop", "htop"]
paths = ["/home/user/project", "/etc", "/var/log", "/tmp", "/usr/bin"]

# Fish also supports wrappers and pipes, let's mix them in too
wrappers = ["sudo", "doas"]
complex_patterns = ["{} | grep foo", "{} --flag"]

with open(FISH_HISTORY, "w") as f:
    for _ in range(50000):
        cmd = random.choice(commands)
        
        if random.random() < 0.2:
             cmd = f"{random.choice(wrappers)} {cmd}"
             
        if random.random() < 0.2:
            pattern = random.choice(complex_patterns)
            cmd = pattern.format(cmd)

        when = int(time.time()) - random.randint(0, 1000000)
        path = random.choice(paths)
        f.write(f"- cmd: {cmd}\n  when: {when}\n  paths:\n    - {path}\n")
EOF

echo "Building instrumented binary..."
cargo clean
# profile-generate requires absolute path usually, or reliable relative. $(pwd) is safe.
RUSTFLAGS="-Cprofile-generate=$(pwd)/$PGO_DATA" cargo build --release

INSTRUMENTED_BINARY="./target/release/histop"

echo "Running workload..."
# Run against bash history
# We assume histop autodetects or handles these files.
$INSTRUMENTED_BINARY -f "$BASH_HISTORY" > /dev/null

# Run against fish history
$INSTRUMENTED_BINARY -f "$FISH_HISTORY" > /dev/null

# Run with some flags to exercise different paths
$INSTRUMENTED_BINARY -f "$BASH_HISTORY" -a -c 100 > /dev/null
$INSTRUMENTED_BINARY -f "$FISH_HISTORY" -m 5 > /dev/null
$INSTRUMENTED_BINARY -f "$BASH_HISTORY" -o json > /dev/null
$INSTRUMENTED_BINARY -f "$FISH_HISTORY" -o csv > /dev/null

echo "Merging profiles..."
# Use nix shell to run llvm-profdata
nix shell nixpkgs#llvmPackages.bintools-unwrapped --command llvm-profdata merge -o "$PGO_DATA/merged.profdata" "$PGO_DATA"

echo "Building optimized binary..."
cargo clean
RUSTFLAGS="-Cprofile-use=$(pwd)/$PGO_DATA/merged.profdata" cargo build --release

echo "PGO Build complete!"
ls -lh target/release/histop

# Cleanup
rm "$BASH_HISTORY" "$FISH_HISTORY"
rm -rf "$PGO_DATA"
