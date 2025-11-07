use std::collections::HashMap;

use crate::shell::commands::{map_external_commands, CommandType};

use super::commands::Command;

fn split_command(input: &str) -> (String, Vec<String>) {
    let mut in_quotes = false;
    let mut current = String::new();
    let mut parts = Vec::new();

    for c in input.chars() {
        match c {
            '\'' => in_quotes = !in_quotes,
            ' ' if !in_quotes => {
                if !current.is_empty() {
                    parts.push(current.clone());
                    current.clear();
                }
            }
            _ => current.push(c),
        }
    }

    if !current.is_empty() {
        parts.push(current);
    }

    let empty = String::new();

    let (cmd, args) = parts.split_first().unwrap_or((&empty, &[]));
    (cmd.clone(), args.iter().cloned().collect())
}

pub fn parse_command(input: &str) -> Option<Command> {
    let (cmd, args_vec) = split_command(input);

    if cmd.is_empty() {
        return None;
    }

    let args = args_vec.join(" ");

    let mut external_commands: HashMap<String, CommandType> = HashMap::new();
    map_external_commands(&mut external_commands);

    // check if the command is external
    if external_commands.contains_key(cmd.as_str()) {
        // Build a Vec<String> including both the command and its arguments

        return Some(Command::External(args_vec.clone()));
    }

    match cmd.as_str() {
        "exit" => {
            let code = args.parse::<i32>().unwrap_or(0);
            Some(Command::Exit(code))
        }
        "echo" => Some(Command::Echo(args)),
        "type" => Some(Command::Type(args)),
        "pwd" => Some(Command::PWD),
        "cd" => Some(Command::CD(args)),
        "cat" => Some(Command::Cat(args_vec)),
        _ => Some(Command::Unknown(cmd.to_string())),
    }
}
