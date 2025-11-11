use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

pub fn run_ls_command(command: &str) {
    let parts: Vec<&str> = command.trim().split_whitespace().collect();

    let mut dir_path = ".";
    let mut output_path: Option<&str> = None;
    let mut error_path: Option<&str> = None;
    let mut append_stdout = false;
    let mut append_stderr = false;

    let mut i = 1; // skip "ls"
    while i < parts.len() {
        match parts[i] {
            ">" | "1>" => {
                if i + 1 < parts.len() {
                    output_path = Some(parts[i + 1]);
                    append_stdout = false;
                    i += 1;
                }
            }
            ">>" | "1>>" => {
                if i + 1 < parts.len() {
                    output_path = Some(parts[i + 1]);
                    append_stdout = true;
                    i += 1;
                }
            }
            "2>" => {
                if i + 1 < parts.len() {
                    error_path = Some(parts[i + 1]);
                    append_stderr = false;
                    i += 1;
                }
            }
            "2>>" => {
                if i + 1 < parts.len() {
                    error_path = Some(parts[i + 1]);
                    append_stderr = true;
                    i += 1;
                }
            }
            _ => dir_path = parts[i],
        }
        i += 1;
    }

    let path_obj = Path::new(dir_path);

    // Helper to write to error file or stderr
    let write_error = |msg: &str| {
        if let Some(path) = error_path {
            let mut options = OpenOptions::new();
            options.create(true);
            if append_stderr {
                options.append(true);
            } else {
                options.write(true).truncate(true);
            }
            let _ = options
                .open(path)
                .and_then(|mut f| f.write_all(msg.as_bytes()));
        } else {
            eprint!("{}", msg);
        }
    };

    // Handle missing path
    if !path_obj.exists() {
        write_error(&format!("ls: {}: No such file or directory\n", dir_path));
        return;
    }

    // Handle non-directory
    if !path_obj.is_dir() {
        write_error(&format!("ls: {}: Not a directory\n", dir_path));
        return;
    }

    // Read directory
    let mut entries: Vec<String> = vec![];
    if let Ok(dir_entries) = fs::read_dir(path_obj) {
        for entry in dir_entries.flatten() {
            entries.push(entry.file_name().to_string_lossy().to_string());
        }
    } else {
        write_error(&format!("ls: {}: Cannot read directory\n", dir_path));
        return;
    }

    entries.sort();
    let output = entries.join("\n") + "\n";

    if let Some(path) = output_path {
        let mut options = OpenOptions::new();
        options.create(true);
        if append_stdout {
            options.append(true);
        } else {
            options.write(true).truncate(true);
        }
        let _ = options
            .open(path)
            .and_then(|mut f| f.write_all(output.as_bytes()));
    } else {
        print!("{}", output);
    }
}
