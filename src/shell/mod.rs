// src/shell/mod.rs (or where your ShellCompleter is defined)
use std::io; // The compiler warns this is unused. You can safely remove it.
use std::process;

use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
// We need to import all traits required by Helper and History from its specific path
use rustyline::{
    highlight::Highlighter,
    hint::Hinter,
    // FIX 1: Use the compiler's suggestion for History (E0603: trait `History` is private)
    history::History,
    validate::{ValidationContext, ValidationResult, Validator},
    Context,
    Editor,
    Helper,
};

// Command placeholders
mod commands;
mod parser;

use commands::{handle_command, Command};
use parser::parse_command;

// Your list of commands
const AVAILABLE_COMMANDS: [&str; 4] = ["help", "echo", "exit", "ls"];

// --- RUSTYLINE HELPER / COMPLETER IMPLEMENTATION ---
#[derive(Clone)]
struct ShellCompleter;

// FIX 2: Implement all required sub-traits (E0277 errors)
// Hinter (Required by Helper)
impl Hinter for ShellCompleter {
    // Provide no hints
    type Hint = String;
}

// Highlighter (Required by Helper)
impl Highlighter for ShellCompleter {}

// Validator (Required by Helper)
impl Validator for ShellCompleter {
    fn validate(&self, _: &mut ValidationContext) -> Result<ValidationResult, ReadlineError> {
        // Always consider the line valid immediately
        Ok(ValidationResult::Valid(None))
    }
}

// Helper (Requires Completer, Hinter, Highlighter, Validator)
// Now this implementation is valid because all dependencies are met.
impl Helper for ShellCompleter {}

// Completer (which you already implemented)
impl Completer for ShellCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        let parts: Vec<&str> = line[..pos].split_whitespace().collect();
        if parts.len() > 1 {
            return Ok((pos, vec![]));
        }

        let prefix = parts.first().unwrap_or(&"");

        let matches = AVAILABLE_COMMANDS
            .iter()
            .filter(|&name| name.starts_with(prefix))
            .map(|name| Pair {
                display: name.to_string(),
                replacement: name.to_string(),
            })
            .collect();

        Ok((0, matches))
    }
}

// --- SHELL IMPLEMENTATION ---

pub struct Shell;

impl Shell {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&mut self) {
        let completer = ShellCompleter {};

        // The second generic argument `_` now successfully infers the default history type
        // because we correctly imported `History` via `rustyline::history::History`.
        let mut rl = Editor::<ShellCompleter, _>::new().expect("Failed to create rustyline Editor");

        rl.set_helper(Some(completer));

        if rl.load_history("history.txt").is_err() {
            // ...
        }

        loop {
            // ... (Your loop logic remains the same and will now compile)
            let prompt = "$ ";
            let readline = rl.readline(prompt);

            match readline {
                Ok(line) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    rl.add_history_entry(trimmed).unwrap();

                    match parse_command(trimmed) {
                        Some(Command::Exit(code)) => {
                            rl.save_history("history.txt").unwrap();
                            process::exit(code);
                        }
                        Some(cmd) => handle_command(cmd),
                        None => {
                            eprintln!("{}: command not found", trimmed);
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    eprintln!("^C");
                }
                Err(ReadlineError::Eof) => {
                    println!("exit");
                    rl.save_history("history.txt").unwrap();
                    process::exit(0);
                }
                Err(err) => {
                    eprintln!("Error reading input: {:?}", err);
                    break;
                }
            }
        }
    }
}
