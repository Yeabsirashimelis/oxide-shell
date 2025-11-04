use std::io::{self, Write};
use std::process;

mod commands;
mod parser;

use commands::{handle_command, Command};
use parser::parse_command;

pub struct Shell;

impl Shell {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&mut self) {
        let mut input = String::new();

        loop {
            print!("$ ");
            io::stdout().flush().unwrap();

            input.clear();
            io::stdin().read_line(&mut input).unwrap();

            if input.trim().is_empty() {
                continue;
            }

            match parse_command(&input) {
                Some(Command::Exit(code)) => process::exit(code),
                Some(cmd) => handle_command(cmd),
                None => println!("{}: command not found", input.trim()),
            }
        }
    }
}
