use std::{
    fs::{self},
    io::Write,
    path::Path,
};

use crate::shell::commands::echo_command::open_file;

pub fn run_ls_command(command: &str) {
    let parts: Vec<&str> = command.trim().split_whitespace().collect();

    let mut dir_path = ".";
    let mut output_path: Option<(&str, bool)> = None; // (path, append)
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
            _ => dir_path = parts[i],
        }
        i += 1;
    }

    if let Some((path, append)) = output_path {
        let _ = open_file(Path::new(path), append);
    }
    if let Some((path, append)) = error_path {
        let _ = open_file(Path::new(path), append);
    }

    let path_obj = Path::new(dir_path);

    if !path_obj.exists() {
        let err_msg = format!("ls: {}: No such file or directory\n", dir_path);
        if let Some((path, append)) = error_path {
            let _ = open_file(Path::new(path), append)
                .and_then(|mut f| f.write_all(err_msg.as_bytes()));
        } else {
            eprint!("{}", err_msg);
        }
        return;
    }

    if !path_obj.is_dir() {
        let err_msg = format!("ls: {}: Not a directory\n", dir_path);
        if let Some((path, append)) = error_path {
            let _ = open_file(Path::new(path), append)
                .and_then(|mut f| f.write_all(err_msg.as_bytes()));
        } else {
            eprint!("{}", err_msg);
        }
        return;
    }

    let mut entries: Vec<String> = vec![];
    match fs::read_dir(path_obj) {
        Ok(dir_entries) => {
            for entry in dir_entries.flatten() {
                entries.push(entry.file_name().to_string_lossy().to_string());
            }
        }
        Err(err) => {
            let err_msg = format!("ls: cannot read directory '{}': {}\n", dir_path, err);
            if let Some((path, append)) = error_path {
                let _ = open_file(Path::new(path), append)
                    .and_then(|mut f| f.write_all(err_msg.as_bytes()));
            } else {
                eprint!("{}", err_msg);
            }
            return;
        }
    }

    entries.sort();
    let output = entries.join("\n") + "\n";

    if let Some((path, append)) = output_path {
        let _ = open_file(Path::new(path), append).and_then(|mut f| f.write_all(output.as_bytes()));
    } else {
        // Always print to stdout if no stdout redirection
        // (stderr redirection doesn't affect stdout)
        print!("{}", output);
    }
}