use super::commands::Command;
use crate::shell::commands::{map_external_commands, CommandType};
use std::collections::HashMap;

pub fn parse_command(input: &str) -> Option<Command> {
    if input.trim().is_empty() {
        return None;
    }

    // Tokenizer
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

    // --- ECHO FIX ---
    if cmd == "echo" {
        let has_redirect = parts.iter().any(|p| p.contains('>'));
        if has_redirect {
            return Some(Command::Echo(parts.clone())); // Rust echo
        } else {
            return Some(Command::External(parts.clone())); // external /bin/echo
        }
    }

    match cmd.as_str() {
        "exit" => {
            let code = parts
                .get(1)
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(0);
            Some(Command::Exit(code))
        }
        "type" => Some(Command::Type(parts[1..].join(" "))),
        "pwd" => Some(Command::PWD),
        "cd" => Some(Command::CD(parts.get(1).cloned().unwrap_or_default())),
        "cat" => Some(Command::Cat(parts.clone())),
        "ls" => Some(Command::Ls(parts[1..].join(" "))),
        _ => {
            if external_commands.contains_key(&cmd_to_check) {
                Some(Command::External(parts.clone()))
            } else {
                Some(Command::Unknown(cmd.to_string()))
            }
        }
    }
}
