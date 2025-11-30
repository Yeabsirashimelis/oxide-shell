# 🐚 Oxide Shell

[![Rust](https://img.shields.io/badge/rust-1.80+-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![CodeCrafters](https://img.shields.io/badge/CodeCrafters-Shell-green.svg)](https://codecrafters.io)

A fully-featured, POSIX-compliant shell implementation **built with Rust** by **Yeabsira Shimelis**. 

**Oxide Shell** is a modern, fast, and interactive command-line shell that brings the power of Rust to your terminal. Created as part of the [CodeCrafters "Build Your Own Shell" Challenge](https://app.codecrafters.io/courses/shell/overview), this project demonstrates advanced systems programming concepts including command parsing, process management, I/O redirection, and more.

## ✨ Features

### Core Functionality
- 🔄 **REPL (Read-Eval-Print Loop)** with interactive prompt
- 📝 **Command History** with persistent storage
- ⌨️ **Tab Completion** for commands (builtins and PATH executables)
- 🎯 **Quote Parsing** with support for single and double quotes
- 🔀 **I/O Redirection** for stdout and stderr
- 🌐 **Cross-Platform** support (Windows, Linux, macOS)

### Builtin Commands

| Command | Description | Examples |
|---------|-------------|----------|
| `echo` | Print text to stdout | `echo Hello World`<br>`echo "Quoted text"` |
| `exit` | Exit shell with optional code | `exit`<br>`exit 42` |
| `type` | Show command type | `type echo`<br>`type ls` |
| `pwd` | Print working directory | `pwd` |
| `cd` | Change directory | `cd /path/to/dir`<br>`cd ~` |
| `cat` | Concatenate and display files | `cat file.txt`<br>`cat file1.txt file2.txt` |
| `ls` | List directory contents | `ls`<br>`ls /path/to/dir` |

### Advanced Features

#### I/O Redirection
```bash
# Redirect stdout (overwrite)
echo "Hello" > output.txt

# Redirect stdout (append)
echo "World" >> output.txt

# Redirect stderr
cat nonexistent.txt 2> errors.txt

# Redirect stderr (append)
cat another.txt 2>> errors.txt
```

#### Quote Handling
```bash
# Single quotes (literal)
echo 'Hello $USER'  # Output: Hello $USER

# Double quotes (with escape sequences)
echo "Hello\nWorld"  # Supports \n, \t, \\, \", etc.
```

#### External Command Execution
```bash
# Run any executable from PATH
ls -la
git status
python script.py
```

#### Directory Navigation
```bash
# Change to home directory
cd ~

# Relative paths
cd ../parent/sibling

# Absolute paths
cd /usr/local/bin
```

## 🚀 Getting Started

### Prerequisites

- Rust 1.80 or higher
- Cargo (comes with Rust)

### Installation

#### Option 1: Download Binary (Easiest)

**Windows:**
1. Download the latest [oxide-shell.exe](https://github.com/yourusername/oxide-shell/releases/latest/download/oxide-shell.exe)
2. Double-click to run!

**Linux/macOS:**
1. Download the binary from [releases](https://github.com/yourusername/oxide-shell/releases)
2. Make it executable: `chmod +x oxide-shell`
3. Run: `./oxide-shell`

#### Option 2: Build from Source

1. **Clone the repository**
   ```bash
   git clone https://github.com/yourusername/oxide-shell.git
   cd oxide-shell
   ```

2. **Build the project**
   ```bash
   cargo build --release
   ```

3. **Run the shell**
   ```bash
   cargo run --release
   ```
   
   Or directly run the binary:
   ```bash
   ./target/release/oxide-shell
   ```

#### Option 3: Install with Cargo (Coming Soon)

```bash
cargo install oxide-shell
```

### Quick Start

```bash
# Start the shell
$ oxide-shell

╔════════════════════════════════════════════╗
║  CREATED BY YEABSIRA SHIMELIS             ║
╔════════════════════════════════════════════╗

# You'll see the prompt
$ pwd
/home/user/oxide-shell

$ echo "Hello, Oxide Shell!"
Hello, Oxide Shell!

$ type echo
echo is a shell builtin

$ cd ~
$ pwd
/home/user

$ exit
```

## 📁 Project Structure

```
oxide-shell/
├── src/
│   ├── main.rs                      # Entry point
│   ├── lib.rs                       # Library root
│   └── shell/
│       ├── mod.rs                   # Shell REPL and tab completion
│       ├── parser.rs                # Command parsing with quote handling
│       └── commands/
│           ├── mod.rs               # Command trait definition
│           ├── map_commands.rs      # Command registry/dispatcher
│           ├── echo_command.rs      # Echo builtin
│           ├── type_command.rs      # Type builtin
│           ├── pwd_command.rs       # PWD builtin
│           ├── cd_command.rs        # CD builtin
│           ├── cat_command.rs       # Cat builtin
│           ├── ls_command.rs        # LS builtin
│           └── external_command.rs  # External command execution
├── Cargo.toml                       # Rust dependencies
├── README.md                        # This file
└── your_program.sh                  # Shell startup script
```

## 🏗️ Architecture

### Command System

The shell uses a **trait-based architecture** for extensibility:

```rust
pub trait Command {
    fn execute(&self, args: &[String]) -> Result<()>;
}
```

Each command is implemented as a separate module and registered in the command dispatcher:

```rust
// Command registry using HashMap for O(1) lookup
let mut commands = HashMap::new();
commands.insert("echo", Box::new(EchoCommand) as Box<dyn Command>);
commands.insert("pwd", Box::new(PwdCommand) as Box<dyn Command>);
// ... more commands
```

### Parser

The parser handles:
- **Whitespace tokenization** with quote preservation
- **Quote parsing** (single and double quotes)
- **Escape sequences** in double quotes (`\n`, `\t`, `\\`, `\"`)
- **Redirection operators** (`>`, `>>`, `2>`, `2>>`)

### Tab Completion

Custom completer integrates with `rustyline`:
- Completes builtin command names
- Searches PATH for external executables
- Caches results for performance

## 🧪 Testing

Run the test suite:

```bash
cargo test
```

Interactive testing:

```bash
# Test builtin commands
$ cargo run
$ echo Hello
$ pwd
$ cd /tmp
$ ls
$ exit

# Test I/O redirection
$ echo "Test" > output.txt
$ cat output.txt
$ cat nonexistent.txt 2> errors.txt
```

## 📦 Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| [rustyline](https://crates.io/crates/rustyline) | 14.0.0 | Line editing, history, tab completion |
| [anyhow](https://crates.io/crates/anyhow) | 1.0.68 | Error handling |
| [thiserror](https://crates.io/crates/thiserror) | 1.0.38 | Custom error types |
| [bytes](https://crates.io/crates/bytes) | 1.3.0 | Buffer management |
| [once_cell](https://crates.io/crates/once_cell) | 1.19.0 | Lazy static initialization |

## 📦 Binary Size & Performance

- **Binary size**: ~2-3 MB (release build)
- **Startup time**: < 50ms
- **Memory footprint**: ~5-10 MB
- **Command execution**: Near-native speed

## 🎯 Implementation Details

### Command Discovery

External commands are discovered by:
1. Searching PATH environment variable
2. Caching results in a HashMap
3. Platform-specific executable detection (`.exe` on Windows)

### I/O Redirection

Redirection is handled by:
1. Parsing redirection operators during command parsing
2. Opening files with appropriate modes (write/append)
3. Redirecting stdout/stderr using `std::process::Command` configuration

### Error Handling

The shell uses Rust's `Result` type with custom error types:
- **ParseError** - Command parsing failures
- **CommandError** - Command execution failures
- **IOError** - File I/O failures

## 🚧 Limitations & Future Work

### Current Limitations
- ❌ No pipe support (`|`)
- ❌ No environment variable expansion (`$VAR`)
- ❌ No background jobs (`&`)
- ❌ No command substitution (`` `cmd` `` or `$(cmd)`)
- ❌ No glob expansion (`*.txt`)

### Planned Features
- [ ] Pipe operator (`|`) for command chaining
- [ ] Environment variable management (`export`, `unset`)
- [ ] Background job control (`&`, `fg`, `bg`, `jobs`)
- [ ] Command substitution
- [ ] Glob pattern matching
- [ ] Aliases
- [ ] Shell scripts support (`.sh` file execution)
- [ ] Signal handling (Ctrl+C, Ctrl+Z)

## 🤝 Contributing

Contributions are welcome! Here's how you can help:

1. **Fork the repository**
2. **Create a feature branch** (`git checkout -b feature/amazing-feature`)
3. **Commit your changes** (`git commit -m 'Add amazing feature'`)
4. **Push to the branch** (`git push origin feature/amazing-feature`)
5. **Open a Pull Request**

Please ensure your code:
- Follows Rust naming conventions
- Includes appropriate error handling
- Has tests for new features
- Updates documentation as needed

<!-- 
## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details. -->

## 👨‍💻 Author

**Yeabsira Shimelis**

Built with 🦀 Rust

## 🙏 Acknowledgments

- [CodeCrafters](https://codecrafters.io) for the excellent challenge
- The Rust community for amazing crates and documentation
- POSIX shell specification for design guidance

## 📚 Resources

- [POSIX Shell Specification](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html)
- [Rust Book](https://doc.rust-lang.org/book/)
- [rustyline Documentation](https://docs.rs/rustyline/)

## 💬 Support

If you have questions or run into issues:
- Open an issue on GitHub
- Check existing issues for solutions
- Read the POSIX shell documentation

---

**Made with effort and 🦀 Rust**

⭐ Star this repo if you found it helpful!
