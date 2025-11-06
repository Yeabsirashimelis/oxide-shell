use std::collections::HashMap;

use crate::shell::commands::{map_external_commands, CommandType};

use super::commands::Command;

pub fn parse_command(input: &str) -> Option<Command> {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    let (cmd, args) = parts.split_first().unwrap();
    let args = args.join(" ");

    let mut external_commands: HashMap<String, CommandType> = HashMap::new();
    map_external_commands(&mut external_commands);

    // check if the command is external
    if external_commands.contains_key(*cmd) {
        // Build a Vec<String> including both the command and its arguments
        let args_vec: Vec<String> = parts.iter().map(|s| s.to_string()).collect();
        return Some(Command::External(args_vec));
    }

    match *cmd {
        "exit" => {
            let code = args.parse::<i32>().unwrap_or(0);
            Some(Command::Exit(code))
        }
        "echo" => Some(Command::Echo(args)),
        "type" => Some(Command::Type(args)),
        "pwd" => Some(Command::PWD),
        _ => Some(Command::Unknown(cmd.to_string())),
    }
}
