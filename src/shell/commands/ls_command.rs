use std::{
    fs,
    io::{stdout, Write},
    path::Path,
};

use crate::shell::commands::cat_command::open_file;

pub fn run_ls_command(command: &str) {
    let parts: Vec<&str> = command.trim().split_whitespace().collect();

    let mut dir_path = ".";
    let mut output_path: Option<(&str, bool)> = None; // (path, append)
    let mut error_path: Option<(&str, bool)> = None;

    // Parse command arguments
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
            _ => dir_path = parts[i],
        }
        i += 1;
    }

    let path_obj = Path::new(dir_path);

    // Helper to write errors
    let write_error = |msg: &str| {
        if let Some((path, append)) = error_path {
            if let Ok(mut f) = open_file(Path::new(path), append) {
                let _ = f.write_all(msg.as_bytes());
            }
        } else if let Some((path, append)) = output_path {
            // If no 2> but output redirected, write errors to output file
            if let Ok(mut f) = open_file(Path::new(path), append) {
                let _ = f.write_all(msg.as_bytes());
            }
        } else {
            // Default: print to stderr (new line, no extra spaces)
            eprint!("{}", msg);
            let _ = stdout().flush();
        }
    };

    // File/directory existence checks
    if !path_obj.exists() {
        let err_msg = format!("ls: {}: No such file or directory\n", dir_path);
        write_error(&err_msg);
        return;
    }

    if !path_obj.is_dir() {
        let err_msg = format!("ls: {}: Not a directory\n", dir_path);
        write_error(&err_msg);
        return;
    }

    // Read directory
    let mut entries: Vec<String> = Vec::new();
    match fs::read_dir(path_obj) {
        Ok(dir_entries) => {
            for entry in dir_entries.flatten() {
                entries.push(entry.file_name().to_string_lossy().to_string());
            }
        }
        Err(err) => {
            let err_msg = format!("ls: cannot read directory '{}': {}\n", dir_path, err);
            write_error(&err_msg);
            return;
        }
    }

    entries.sort();
    let output = entries.join("\n") + "\n";

    // Write to output or stdout
    if let Some((path, append)) = output_path {
        if let Ok(mut f) = open_file(Path::new(path), append) {
            let _ = f.write_all(output.as_bytes());
        }
    } else {
        print!("{}", output);
        let _ = stdout().flush();
    }
}
