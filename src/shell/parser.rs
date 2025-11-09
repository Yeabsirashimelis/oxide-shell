use std::collections::HashMap;

use super::commands::Command;
use crate::shell::commands::{map_external_commands, CommandType};

pub fn parse_command(input: &str) -> Option<Command> {
    if input.trim().is_empty() {
        return None;
    }

    let mut parts: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;

    let mut chars = input.trim().chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if let Some(next_char) = chars.next() {
                    if in_single_quotes {
                        current.push('\\'); // literal in single quotes
                    }
                    if in_double_quotes && !"\\\"$`".contains(next_char) {
                        current.push('\\');
                    }
                    current.push(next_char);
                }
            }
            '\'' if !in_double_quotes => {
                in_single_quotes = !in_single_quotes;
                continue;
            }
            '"' if !in_single_quotes => {
                in_double_quotes = !in_double_quotes;
                continue;
            }
            ' ' if !in_single_quotes && !in_double_quotes => {
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

    if parts.is_empty() {
        return None;
    }

    let (cmd, args_vec) = parts.split_first().unwrap();
    let args = args_vec.join(" ");

    let mut external_commands: HashMap<String, CommandType> = HashMap::new();
    map_external_commands(&mut external_commands);

    let mut cmd_to_check = String::from(cmd.as_str());

    #[cfg(windows)]
    {
        if !external_commands.contains_key(&cmd_to_check) {
            let cmd_with_exe = format!("{}.exe", cmd);
            if external_commands.contains_key(&cmd_with_exe) {
                cmd_to_check = cmd_with_exe.clone();
            }
        }
    }

    match cmd.as_str() {
        "exit" => {
            let code = args.parse::<i32>().unwrap_or(0);
            Some(Command::Exit(code))
        }
        "echo" => {
            if args.contains('>') || args.contains("1>") {
                Some(Command::Echo(args))
            } else if external_commands.contains_key(&cmd_to_check) {
                let args_vec: Vec<String> = parts.iter().map(|s| s.to_string()).collect();
                Some(Command::External(args_vec))
            } else {
                // Fallback to Rust echo
                Some(Command::Echo(args))
            }
        }
        "type" => Some(Command::Type(args)),
        "pwd" => Some(Command::PWD),
        "cd" => Some(Command::CD(args)),
        "cat" => {
            let args_vec: Vec<String> = parts.iter().map(|s| s.to_string()).collect();
            Some(Command::Cat(args_vec))
        }
        "ls" => Some(Command::Ls(args)),
        _ => {
            if external_commands.contains_key(&cmd_to_check) {
                let args_vec: Vec<String> = parts.iter().map(|s| s.to_string()).collect();
                Some(Command::External(args_vec))
            } else {
                Some(Command::Unknown(cmd.to_string()))
            }
        }
    }
}
