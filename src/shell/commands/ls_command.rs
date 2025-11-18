use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use crate::shell::commands::cat_command::open_file;

pub fn run_ls_command(command: &str) {
    // The command is already trimmed before being passed here
    let parts: Vec<&str> = command.split_whitespace().collect();

    // Check for required elements. If no command parts, skip.
    if parts.is_empty() {
        return;
    }

    let mut dir_path = ".";
    // Using None for options, path stored as string slice, bool for append
    let mut output_path: Option<(&str, bool)> = None;
    let mut error_path: Option<(&str, bool)> = None;

    // Use a Vec to collect non-redirection arguments (path, flags, etc.)
    let mut args: Vec<&str> = Vec::new();

    let mut i = 1; // Start after the command name "ls"
    while i < parts.len() {
        match parts[i] {
            ">" | "1>" => {
                if i + 1 < parts.len() {
                    output_path = Some((parts[i + 1], false));
                    i += 1; // Skip the path part
                }
            }
            ">>" | "1>>" => {
                if i + 1 < parts.len() {
                    output_path = Some((parts[i + 1], true));
                    i += 1; // Skip the path part
                }
            }
            "2>" => {
                if i + 1 < parts.len() {
                    error_path = Some((parts[i + 1], false));
                    i += 1; // Skip the path part
                }
            }
            "2>>" => {
                if i + 1 < parts.len() {
                    error_path = Some((parts[i + 1], true));
                    i += 1; // Skip the path part
                }
            }
            // All other arguments are collected
            _ => {
                args.push(parts[i]);
            }
        }
        i += 1;
    }

    // Determine the directory path: It should be the last non-redirection argument.
    // Assuming the only non-flag argument is the directory path (e.g., ls -1 /tmp/dir)
    // If multiple paths are given, this shell simplifies to only use the last one found.
    // For this specific test case, we only expect one path: "nonexistent".
    // For Stage #UN3, the arguments are "-1" and "nonexistent".
    if let Some(path_or_flag) = args.last() {
        if !path_or_flag.starts_with('-') {
            dir_path = path_or_flag;
        }
    }

    let path_obj = Path::new(dir_path);

    // Initial output/error file creation to handle truncation/append flags correctly
    // Since the test fails on an error (nonexistent file), we must make sure we open the
    // error file first to capture the error message.
    let mut error_file: Option<File> = None;
    if let Some((path, append)) = error_path {
        if let Ok(file) = open_file(Path::new(path), append) {
            error_file = Some(file);
        }
    }

    // Handle path not found
    if !path_obj.exists() {
        let err_msg = format!("ls: {}: No such file or directory\n", dir_path);

        if let Some(mut f) = error_file {
            let _ = f.write_all(err_msg.as_bytes());
        } else {
            // Write to default stderr
            let _ = std::io::stderr().write_all(err_msg.as_bytes());
            let _ = std::io::stderr().flush();
        }
        return;
    }

    // Handle not a directory
    if !path_obj.is_dir() {
        let err_msg = format!("ls: {}: Not a directory\n", dir_path);

        if let Some(mut f) = error_file {
            let _ = f.write_all(err_msg.as_bytes());
        } else {
            let _ = std::io::stderr().write_all(err_msg.as_bytes());
            let _ = std::io::stderr().flush();
        }
        return;
    }

    // Read directory entries
    let mut entries: Vec<String> = Vec::new();
    match fs::read_dir(path_obj) {
        Ok(dir_entries) => {
            for entry in dir_entries.flatten() {
                entries.push(entry.file_name().to_string_lossy().to_string());
            }
        }
        Err(err) => {
            let err_msg = format!("ls: cannot read directory '{}': {}\n", dir_path, err);

            if let Some(mut f) = error_file {
                let _ = f.write_all(err_msg.as_bytes());
            } else {
                let _ = std::io::stderr().write_all(err_msg.as_bytes());
                let _ = std::io::stderr().flush();
            }
            return;
        }
    }

    // Output handling (for Stage 1: only output non-hidden files in current directory)
    // The test requires the output to be sorted.
    entries.sort();
    let output = entries.join("\n") + "\n";

    if let Some((path, append)) = output_path {
        let _ = open_file(Path::new(path), append).and_then(|mut f| f.write_all(output.as_bytes()));
    } else {
        print!("{}", output);
    }
}
