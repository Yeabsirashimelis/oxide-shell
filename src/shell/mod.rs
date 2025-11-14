use std::io::{self, Write};
use std::process;

mod commands;
mod parser;

use commands::{handle_command, Command};
use parser::parse_command;

use crate::shell::commands::echo_command::run_echo_command;

pub struct Shell;

impl Shell {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&mut self) {
        let mut input = String::new();

        loop {
            input.clear();
            print!("$ ");
            io::stdout().flush().unwrap();

            if io::stdin().read_line(&mut input).is_err() {
                continue;
            }
            let trimmed = input.trim();
            if trimmed.is_empty() {
                continue;
            }

            if let Some(cmd) = parse_command(trimmed) {
                match &cmd {
                    Command::Exit(code) => process::exit(*code),
                    Command::Echo(parts) => run_echo_command(parts.clone()),
                    _ => handle_command(cmd),
                }
            } else {
                println!("{}: command not found", trimmed);
            }
        }
    }
}
