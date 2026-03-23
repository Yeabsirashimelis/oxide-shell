use std::fs::{File, OpenOptions};
use std::io::Write;

/// Echo command with generic writer for pipeline support.
pub fn run_echo_command_with_writer(raw: String, writer: &mut dyn Write) {
    let tokens: Vec<&str> = raw.split_whitespace().collect();
    let mut echo_parts: Vec<&str> = Vec::new();

    let mut i = 0;
    while i < tokens.len() {
        match tokens[i] {
            ">" | "1>" | ">>" | "1>>" | "2>" | "2>>" => {
                // Skip redirection in pipeline context
                i += 2;
            }
            _ => {
                echo_parts.push(tokens[i]);
                i += 1;
            }
        }
    }

    let message = echo_parts.join(" ");
    let _ = writeln!(writer, "{}", message);
}

pub fn run_echo_command(raw: String) {
    let tokens: Vec<&str> = raw.split_whitespace().collect();

    let mut stdout_path: Option<(&str, bool)> = None; // (path, append)
    let mut stderr_path: Option<(&str, bool)> = None;
    let mut echo_parts: Vec<&str> = Vec::new();

    let mut i = 0;
    while i < tokens.len() {
        match tokens[i] {
            ">" | "1>" => {
                stdout_path = Some((tokens[i + 1], false));
                i += 2;
            }
            ">>" | "1>>" => {
                stdout_path = Some((tokens[i + 1], true));
                i += 2;
            }
            "2>" => {
                stderr_path = Some((tokens[i + 1], false));
                i += 2;
            }
            "2>>" => {
                stderr_path = Some((tokens[i + 1], true));
                i += 2;
            }
            _ => {
                echo_parts.push(tokens[i]);
                i += 1;
            }
        }
    }
    //
    let message = echo_parts.join(" ");

    if let Some((path, append)) = stdout_path {
        let mut file = if append {
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .unwrap()
        } else {
            File::create(path).unwrap()
        };
        writeln!(file, "{}", message).unwrap();
    } else {
        println!("{}", message);
    }

    if let Some((path, append)) = stderr_path {
        let _ = if append {
            OpenOptions::new().create(true).append(true).open(path)
        } else {
            File::create(path)
        };
    }
}
