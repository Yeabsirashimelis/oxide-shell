use std::{
    io::{self, Write},
    process,
    time::Duration,
};

mod commands;
mod parser;

use commands::{handle_command, Command};
use crossterm::event::{poll, read, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use parser::parse_command;

use crate::shell::commands::ls_command::run_ls_command;

pub struct Shell;

impl Shell {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&mut self) {
        let mut input = String::new();
        let available_commands = ["help", "echo", "exit", "ls"];

        enable_raw_mode().unwrap();

        loop {
            // Print prompt
            print!("$ ");
            io::stdout().flush().unwrap();

            input.clear();

            loop {
                if poll(Duration::from_millis(100)).unwrap() {
                    if let Event::Key(key_event) = read().unwrap() {
                        match key_event.code {
                            KeyCode::Enter => {
                                if key_event.kind == KeyEventKind::Press {
                                    println!();
                                    break; // submit command
                                }
                            }
                            KeyCode::Char('c')
                                if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
                            {
                                disable_raw_mode().unwrap();
                                process::exit(0);
                            }
                            KeyCode::Char(c) => {
                                if key_event.kind == KeyEventKind::Press {
                                    input.push(c);
                                    print!("{}", c);
                                    io::stdout().flush().unwrap();
                                }
                            }
                            KeyCode::Backspace => {
                                if key_event.kind == KeyEventKind::Press {
                                    if input.pop().is_some() {
                                        print!("\x08 \x08");
                                        io::stdout().flush().unwrap();
                                    }
                                }
                            }
                            KeyCode::Tab => {
                                if key_event.kind == KeyEventKind::Press {
                                    if let Some(matched) = available_commands
                                        .iter()
                                        .find(|cmd| cmd.starts_with(&input))
                                    {
                                        // autocomplete in place
                                        print!("\r$ {}", matched);
                                        io::stdout().flush().unwrap();
                                        input = matched.to_string();
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            let trimmed = input.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Handle commands
            if trimmed.starts_with("ls") {
                // call your fixed ls_command
                run_ls_command(trimmed);
            } else {
                match parse_command(trimmed) {
                    Some(Command::Exit(code)) => {
                        disable_raw_mode().unwrap();
                        process::exit(code);
                    }
                    Some(cmd) => handle_command(cmd),
                    None => println!("{}: command not found", trimmed),
                }
            }
        }
    }
}
