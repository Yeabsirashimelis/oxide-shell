use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;

/// Run echo command with support for stdout and stderr redirection
pub fn run_echo_command(parts: Vec<String>) {
    if parts.is_empty() {
        return;
    }

    let mut stdout_path: Option<(String, bool)> = None;
    let mut stderr_path: Option<(String, bool)> = None;

    let mut filtered_parts = Vec::new();
    let mut i = 0;

    while i < parts.len() {
        match parts[i].as_str() {
            ">" | "1>" => {
                if i + 1 < parts.len() {
                    stdout_path = Some((parts[i + 1].clone(), false));
                    i += 1;
                }
            }
            ">>" | "1>>" => {
                if i + 1 < parts.len() {
                    stdout_path = Some((parts[i + 1].clone(), true));
                    i += 1;
                }
            }
            "2>" => {
                if i + 1 < parts.len() {
                    stderr_path = Some((parts[i + 1].clone(), false));
                    i += 1;
                }
            }
            "2>>" => {
                if i + 1 < parts.len() {
                    stderr_path = Some((parts[i + 1].clone(), true));
                    i += 1;
                }
            }
            _ => filtered_parts.push(parts[i].clone()),
        }
        i += 1;
    }

    // Reconstruct message (skip "echo" command itself)
    let message = filtered_parts[1..]
        .iter()
        .map(|s| strip_outer_quotes(s))
        .collect::<Vec<_>>()
        .join(" ")
        + "\n";

    // Handle stderr redirection first
    if let Some((path, append)) = stderr_path {
        if let Ok(mut f) = open_file(Path::new(&path), append) {
            let _ = f.write_all(message.as_bytes());
            let _ = f.flush();
        } else {
            let _ = eprint!("{}", message);
        }
        return; // do NOT print to stdout
    }

    // Handle stdout redirection
    if let Some((path, append)) = stdout_path {
        if let Ok(mut f) = open_file(Path::new(&path), append) {
            let _ = f.write_all(message.as_bytes());
            let _ = f.flush();
        } else {
            print!("{}", message);
        }
        return;
    }

    // Normal echo
    print!("{}", message);
}

fn strip_outer_quotes(s: &str) -> String {
    if s.len() >= 2 {
        let first = s.chars().next().unwrap();
        let last = s.chars().last().unwrap();
        if (first == '"' && last == '"') || (first == '\'' && last == '\'') {
            return s[1..s.len() - 1].to_string();
        }
    }
    s.to_string()
}

fn open_file(path: &Path, append: bool) -> std::io::Result<File> {
    let mut opts = OpenOptions::new();
    opts.create(true);
    if append {
        opts.append(true);
    } else {
        opts.write(true).truncate(true);
    }
    opts.open(path)
}
