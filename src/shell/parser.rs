use std::collections::HashMap;

use super::commands::Command;
use crate::shell::commands::{map_external_commands, CommandType};

pub fn parse_command(input: &str) -> Option<Command> {
    if input.trim().is_empty() {
        return None;
    }

    // --- TOKENIZER (kept the same) ---
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
                        current.push('\\');
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

    let (cmd, _) = parts.split_first().unwrap();

    let mut external_commands: HashMap<String, CommandType> = HashMap::new();
    map_external_commands(&mut external_commands);

    let mut cmd_to_check = cmd.to_string();

    #[cfg(windows)]
    {
        if !external_commands.contains_key(&cmd_to_check) {
            let exe_cmd = format!("{}.exe", cmd);
            if external_commands.contains_key(&exe_cmd) {
                cmd_to_check = exe_cmd;
            }
        }
    }

    // -------- ECHO FIX BELOW --------
    if cmd == "echo" {
        let has_redirect = parts.iter().any(|p| p.contains('>'));

        if has_redirect {
            // Use RUST echo implementation
            return Some(Command::Echo(parts.clone()));
        } else {
            // Use external /bin/echo
            return Some(Command::External(parts.clone()));
        }
    }
    // --------------------------------

    match cmd.as_str() {
        "exit" => {
            let args = parts.get(1).cloned().unwrap_or_default();
            let code = args.parse::<i32>().unwrap_or(0);
            Some(Command::Exit(code))
        }
        "type" => {
            let args = parts[1..].join(" ");
            Some(Command::Type(args))
        }
        "pwd" => Some(Command::PWD),
        "cd" => {
            let args = parts.get(1).cloned().unwrap_or_default();
            Some(Command::CD(args))
        }
        "cat" => Some(Command::Cat(parts.clone())),
        "ls" => {
            let args = parts[1..].join(" ");
            Some(Command::Ls(args))
        }
        _ => {
            if external_commands.contains_key(&cmd_to_check) {
                Some(Command::External(parts.clone()))
            } else {
                Some(Command::Unknown(cmd.to_string()))
            }
        }
    }
}
