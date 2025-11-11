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

    let write_error = |msg: &str| {
        if let Some(path) = error_path {
            let path_obj = Path::new(path);
            if let Some(parent) = path_obj.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let mut options = OpenOptions::new();
            options.create(true);
            if append_stderr {
                options.append(true);
            } else {
                options.write(true).truncate(true);
            }
            let _ = options
                .open(path_obj)
                .and_then(|mut f| writeln!(f, "{}", msg));
        } else {
            eprint!("{}", msg);
        }
    };

    if !path_obj.exists() {
        write_error(&format!("ls: {}: No such file or directory", dir_path));
        return;
    }

    if !path_obj.is_dir() {
        write_error(&format!("ls: {}: Not a directory", dir_path));
        return;
    }

    let mut entries: Vec<String> = vec![];
    if let Ok(dir_entries) = fs::read_dir(path_obj) {
        for entry in dir_entries.flatten() {
            entries.push(entry.file_name().to_string_lossy().to_string());
        }
    } else {
        write_error(&format!("ls: {}: Cannot read directory", dir_path));
        return;
    }

    entries.sort();
    let output = entries.join("\n") + "\n";

    if let Some(path) = output_path {
        let path_obj = Path::new(path);
        if let Some(parent) = path_obj.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let mut options = OpenOptions::new();
        options.create(true);
        if append_stdout {
            options.append(true);
        } else {
            options.write(true).truncate(true);
        }
        if let Err(err) = options
            .open(path_obj)
            .and_then(|mut f| f.write_all(output.as_bytes()))
        {
            write_error(&format!("ls: failed to write to {}: {}", path, err));
        }
    } else {
        print!("{}", output);
    }
}
