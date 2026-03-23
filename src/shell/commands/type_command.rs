use std::io::Write;

use crate::shell::commands::{load_cmd_and_description, CommandType};

/// Type command with generic writer for pipeline support.
pub fn run_type_command_with_writer(cmd: String, writer: &mut dyn Write) {
    let command_map = load_cmd_and_description();

    // Check for builtin first
    if let Some(CommandType::Builtin) = command_map.get(&cmd) {
        let _ = writeln!(writer, "{} is a shell builtin", cmd);
        return;
    }

    // Check external command
    #[cfg(windows)]
    let cmd_key = if command_map.contains_key(&cmd) {
        cmd.clone()
    } else {
        format!("{}.exe", cmd)
    };

    #[cfg(not(windows))]
    let cmd_key = cmd.clone();

    if let Some(CommandType::External(path)) = command_map.get(&cmd_key) {
        let _ = writeln!(writer, "{} is {}", cmd, path);
    } else {
        eprintln!("{}: not found", cmd);
    }
}

pub fn run_type_command(cmd: String) {
    let command_map = load_cmd_and_description();

    // Check for builtin first
    if let Some(CommandType::Builtin) = command_map.get(&cmd) {
        println!("{} is a shell builtin", cmd);
        return;
    }

    // Check external command
    #[cfg(windows)]
    let cmd_key = if command_map.contains_key(&cmd) {
        cmd.clone()
    } else {
        format!("{}.exe", cmd)
    };

    #[cfg(not(windows))]
    let cmd_key = cmd.clone();

    if let Some(CommandType::External(path)) = command_map.get(&cmd_key) {
        println!("{} is {}", cmd, path);
    } else {
        eprintln!("{}: not found", cmd);
    }
}
