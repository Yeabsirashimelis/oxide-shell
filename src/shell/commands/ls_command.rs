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

    // Parse redirection
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

    // Determine writers
    let mut stdout_writer: Box<dyn Write> = if let Some((path, append)) = output_path {
        Box::new(open_file(Path::new(path), append).unwrap())
    } else {
        Box::new(stdout())
    };

    let mut stderr_writer: Box<dyn Write> = if let Some((path, append)) = error_path {
        Box::new(open_file(Path::new(path), append).unwrap())
    } else {
        // if no explicit error_path, write to same as stdout
        stdout_writer.as_mut()
    };

    let path_obj = Path::new(dir_path);

    if !path_obj.exists() {
        let err_msg = format!("ls: {}: No such file or directory\n", dir_path);
        let _ = stderr_writer.write_all(err_msg.as_bytes());
        let _ = stderr_writer.flush();
        return;
    }

    if !path_obj.is_dir() {
        let err_msg = format!("ls: {}: Not a directory\n", dir_path);
        let _ = stderr_writer.write_all(err_msg.as_bytes());
        let _ = stderr_writer.flush();
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
            let err_msg = format!("ls: cannot read directory '{}': {}\n", dir_path, err);
            let _ = stderr_writer.write_all(err_msg.as_bytes());
            let _ = stderr_writer.flush();
            return;
        }
    }

    entries.sort();
    let output = entries.join("\n") + "\n";

    let _ = stdout_writer.write_all(output.as_bytes());
    let _ = stdout_writer.flush();
}
