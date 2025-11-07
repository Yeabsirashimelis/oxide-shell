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

    let mut cmd_to_check = String::from(*cmd);

    #[cfg(windows)]
    {
        if !external_commands.contains_key(&cmd_to_check) {
            let cmd_with_exe = format!("{}.exe", cmd);
            if external_commands.contains_key(&cmd_with_exe) {
                cmd_to_check = cmd_with_exe.clone();
            }
        }
    }

    if external_commands.contains_key(&cmd_to_check) {
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
        "cd" => Some(Command::CD(args)),
        "cat" => {
            let args_vec: Vec<String> = parts.iter().map(|s| s.to_string()).collect();
            return Some(Command::Cat(args_vec));
        }
        _ => Some(Command::Unknown(cmd.to_string())),
    }
}
