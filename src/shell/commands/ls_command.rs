use crate::shell::commands::cat_command::open_file;
use std::fs::{self};
use std::io::Write;
use std::path::Path;

pub fn run_ls_command(command: &str) {
    let parts: Vec<&str> = command.trim().split_whitespace().collect();

    let mut dir_path = ".";
    let mut output_path: Option<(&str, bool)> = None;
    let mut error_path: Option<(&str, bool)> = None;

    let mut i = 1;
    while i < parts.len() {
        match parts[i] {
            ">" | "1>" => {
                if i + 1 < parts.len() {
                    output_path = Some((parts[i + 1], false));
                    i += 1;
                }
            }
            ">>" | "1>>" => {
                if i + 1 < parts.len() {
                    output_path = Some((parts[i + 1], true));
                    i += 1;
                }
            }
            "2>" => {
                if i + 1 < parts.len() {
                    error_path = Some((parts[i + 1], false));
                    i += 1;
                }
            }
            "2>>" => {
                if i + 1 < parts.len() {
                    error_path = Some((parts[i + 1], true));
                    i += 1;
                }
            }
            _ => {
                // Only set dir_path if it's not a redirection operator and we haven't set it yet
                if dir_path == "." && !parts[i].starts_with('>') {
                    dir_path = parts[i];
                }
            }
        }
        i += 1;
    }

    // Open files if redirected (creates empty files if missing)
    if let Some((path, append)) = output_path {
        let _ = open_file(Path::new(path), append);
    }
    if let Some((path, append)) = error_path {
        let _ = open_file(Path::new(path), append);
    }

    let path_obj = Path::new(dir_path);

    if !path_obj.exists() {
        let msg = format!("ls: {}: No such file or directory\n", dir_path);
        if let Some((path, append)) = error_path {
            if let Ok(mut f) = open_file(Path::new(path), append) {
                let _ = f.write_all(msg.as_bytes());
            }
        } else {
            eprint!("{}", msg);
        }
        return;
    }

    if !path_obj.is_dir() {
        let msg = format!("ls: {}: Not a directory\n", dir_path);
        if let Some((path, append)) = error_path {
            if let Ok(mut f) = open_file(Path::new(path), append) {
                let _ = f.write_all(msg.as_bytes());
            }
        } else {
            eprint!("{}", msg);
        }
        return;
    }

    let mut entries: Vec<String> = Vec::new();
    match fs::read_dir(path_obj) {
        Ok(dir_entries) => {
            for entry in dir_entries.flatten() {
                entries.push(entry.file_name().to_string_lossy().to_string());
            }
        }
        Err(err) => {
            let msg = format!("ls: cannot read directory '{}': {}\n", dir_path, err);
            if let Some((path, append)) = error_path {
                if let Ok(mut f) = open_file(Path::new(path), append) {
                    let _ = f.write_all(msg.as_bytes());
                }
            } else {
                eprint!("{}", msg);
            }
            return;
        }
    }

    entries.sort();
    let output = entries.join("\n") + "\n";

    if let Some((path, append)) = output_path {
        let _ = open_file(Path::new(path), append).and_then(|mut f| f.write_all(output.as_bytes()));
    } else {
        print!("{}", output);
    }
}
