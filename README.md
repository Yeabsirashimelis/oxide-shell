# Oxide Shell

[![Rust](https://img.shields.io/badge/rust-1.80+-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Release](https://img.shields.io/github/v/release/Yeabsirashimelis/oxide-shell)](https://github.com/Yeabsirashimelis/oxide-shell/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/Yeabsirashimelis/oxide-shell/total)](https://github.com/Yeabsirashimelis/oxide-shell/releases)

A shell built from scratch in Rust. Started as a [CodeCrafters challenge](https://app.codecrafters.io/courses/shell/overview), grew into something with real features — pipes, control flow, variable expansion, the works.

```
  ___       _     _        ____  _          _ _
 / _ \__  _(_) __| | ___  / ___|| |__   ___| | |
| | | \ \/ / |/ _` |/ _ \ \___ \| '_ \ / _ \ | |
| |_| |>  <| | (_| |  __/  ___) | | | |  __/ | |
 \___//_/\_\_|\__,_|\___| |____/|_| |_|\___|_|_|
                                        v0.2.0
                              by Yeabsira Shimelis
```

## Get It

**Download the binary:** [oxide-shell.exe](https://github.com/Yeabsirashimelis/oxide-shell/releases/latest/download/oxide-shell.exe)

Or build from source:

```bash
git clone https://github.com/Yeabsirashimelis/oxide-shell.git
cd oxide-shell
cargo build --release
./target/release/oxide-shell
```

## What It Can Do

### Builtins

| Command | What it does |
|---------|-------------|
| `echo` | Print text, supports redirection (`>`, `>>`, `2>`) |
| `cd` | Change directory (`cd ~`, `cd ..`, `cd /path`) |
| `pwd` | Print working directory |
| `ls` | List directory contents |
| `cat` | Read files, supports input redirection (`<`) |
| `export` | Set environment variables |
| `unset` | Remove environment variables |
| `alias` / `unalias` | Create and remove command aliases |
| `history` | Show command history |
| `clear` | Clear the screen |
| `type` | Show whether a command is builtin or external |
| `exit` | Quit the shell |

### Pipes and Chaining

```bash
echo hello world | grep hello
cat file.txt | sort | uniq

# Run second command only if first succeeds
mkdir newdir && cd newdir

# Run second command only if first fails
cat config.txt 2>/dev/null || echo "no config found"

# Run both regardless
echo starting ; ls ; echo done
```

### Variables

```bash
# Set and use variables
name=world
echo $name          # world
echo ${name}        # world

# Export for child processes
export PATH="/my/bin:$PATH"

# Special variables
echo $?             # last exit code
echo $$             # shell process ID
```

### Arithmetic

```bash
echo $((2 + 3))         # 5
echo $((10 * 4 - 3))    # 37
echo $((10 / 3))        # 3
echo $((10 % 3))        # 1

x=5
echo $((x + 1))         # 6
```

### Command Substitution

```bash
echo "I'm in $(pwd)"
files=$(ls)
today=`date`
```

### Globbing and Expansion

```bash
# Wildcards
ls *.txt
cat src/*.rs

# Brace expansion
echo {a,b,c}             # a b c
echo file{1..5}.txt      # file1.txt file2.txt ... file5.txt
touch test_{a,b,c}.log

# Tilde
cd ~
ls ~/Documents
```

### Control Flow

```bash
# if / elif / else
if [ -f config.txt ]; then
    echo "config exists"
elif [ -f config.default ]; then
    echo "using default"
else
    echo "no config"
fi

# for loops
for f in *.txt; do
    echo "found: $f"
done

for i in {1..5}; do
    echo $i
done

# while loops
x=0
while [ $x -lt 5 ]; do
    echo $x
    x=$((x + 1))
done

# case
case $1 in
    start)  echo "starting" ;;
    stop)   echo "stopping" ;;
    *)      echo "unknown: $1" ;;
esac
```

### Here Documents

```bash
cat <<EOF
Hello $name,
This is a multi-line message.
Today is $(date).
EOF
```

### I/O Redirection

```bash
echo "hello" > output.txt       # overwrite
echo "world" >> output.txt      # append
cat < input.txt                 # read from file
cmd 2> errors.log               # redirect stderr
cmd 2>> errors.log              # append stderr
```

### Aliases

```bash
alias ll="ls -la"
alias gs="git status"
ll                    # runs ls -la
unalias ll
```

### Other Stuff

- **Tab completion** for commands (builtins + everything on PATH)
- **Persistent history** across sessions
- **Quote handling** — single quotes are literal, double quotes expand variables
- **Cross-platform** — works on Windows, Linux, macOS

## Project Structure

```
oxide-shell/
├── src/
│   ├── main.rs
│   ├── lib.rs
│   └── shell/
│       ├── mod.rs                  # REPL loop, tab completion, heredoc/control flow collection
│       ├── parser.rs               # Tokenizer, variable/arithmetic/glob/brace/tilde expansion
│       └── commands/
│           ├── mod.rs              # Command enum, dispatch, aliases, heredoc execution
│           ├── map_commands.rs     # Builtin + PATH command discovery
│           ├── echo_command.rs     # echo with redirection
│           ├── cat_command.rs      # cat with redirection
│           ├── cd_command.rs       # cd
│           ├── pwd_command.rs      # pwd
│           ├── ls_command.rs       # ls
│           ├── type_command.rs     # type
│           ├── export_command.rs   # export
│           ├── unset_command.rs    # unset
│           ├── external_command.rs # External command execution with redirection
│           ├── pipeline.rs         # Pipe execution
│           ├── chain.rs            # &&, ||, ; chaining
│           └── control_flow.rs     # if/for/while/until/case execution
├── Cargo.toml
└── README.md
```

## Not Yet Implemented

- Background jobs (`&`, `fg`, `bg`, `jobs`)
- Signal handling (`Ctrl+Z` to suspend)
- Shell scripting (running `.sh` files)
- Functions
- `source` / `.` command
- Prompt customization

## Dependencies

| Crate | Purpose |
|-------|---------|
| [rustyline](https://crates.io/crates/rustyline) | Line editing, history, tab completion |
| [glob](https://crates.io/crates/glob) | Wildcard pattern matching |
| [os_pipe](https://crates.io/crates/os_pipe) | Cross-platform pipe support |
| [once_cell](https://crates.io/crates/once_cell) | Lazy statics |
| [anyhow](https://crates.io/crates/anyhow) / [thiserror](https://crates.io/crates/thiserror) | Error handling |

## Contributing

PRs welcome. Fork it, branch it, open a PR.

## Author

**Yeabsira Shimelis** — built with Rust
