use std::collections::HashMap;
use std::{env, process};

use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::{
    highlight::Highlighter,
    hint::Hinter,
    history::History,
    validate::{ValidationContext, ValidationResult, Validator},
    Context, Editor, Helper,
};

mod commands;
mod parser;

use commands::{handle_command, Command};
use parser::parse_command;

use crate::shell::commands::map_external_commands;
use crate::shell::commands::CommandType;

const BUILTIN_COMMANDS: [&str; 4] = ["help", "echo", "exit", "ls"];

fn path_executables_for_tabcompletiion() -> Vec<String> {
    let mut external_commands: HashMap<String, CommandType> = HashMap::new();
    map_external_commands(&mut external_commands);

    let external_commands: Vec<String> = external_commands
        .iter()
        .map(|(key, _)| key.split(".").collect::<Vec<_>>()[0].to_string())
        .collect();

    external_commands
}

#[derive(Clone)]
struct ShellCompleter {
    external_commands: Vec<String>,
}

impl Hinter for ShellCompleter {
    type Hint = String;
}

impl Highlighter for ShellCompleter {}

impl Validator for ShellCompleter {
    fn validate(&self, _: &mut ValidationContext) -> Result<ValidationResult, ReadlineError> {
        Ok(ValidationResult::Valid(None))
    }
}

impl Helper for ShellCompleter {}

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
        let mut matches: Vec<Pair> = Vec::new();

        for name in BUILTIN_COMMANDS.iter() {
            if name.starts_with(prefix) {
                matches.push(Pair {
                    display: name.to_string(),
                    replacement: format!("{} ", name),
                });
            }
        }

        for name in self.external_commands.iter() {
            if name.starts_with(prefix) {
                matches.push(Pair {
                    display: name.clone(),
                    replacement: format!("{} ", name),
                });
            }
        }

        Ok((0, matches))
    }
}

pub struct Shell;

impl Shell {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&mut self) {
        println!("\x1b[32m╔════════════════════════════════════════════╗\x1b[0m");
        println!("\x1b[32m║  CREATED BY YEABSIRA SHIMELIS             ║\x1b[0m");
        println!("\x1b[32m╔════════════════════════════════════════════╗\x1b[0m");
        println!();

        let external_commands = path_executables_for_tabcompletiion();
        let completer = ShellCompleter { external_commands };

        let mut rl = Editor::<ShellCompleter, _>::new().expect("Failed to create rustyline Editor");

        rl.set_helper(Some(completer));

        if rl.load_history("history.txt").is_err() {}

        loop {
            let current_dir =
                env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("<unknown>"));
            let prompt = format!("[🦀 yeabshell {}]$ ", current_dir.display());
            let readline = rl.readline(&prompt);

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
