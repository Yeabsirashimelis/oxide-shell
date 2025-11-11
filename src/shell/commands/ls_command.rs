use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

pub fn run_ls_command(command: &str) {
    let parts: Vec<&str> = command.trim().split_whitespace().collect();

    let mut dir_path = ".";
    let mut output_path: Option<&str> = None;
    let mut error_path: Option<&str> = None;

    let mut i = 1;
    while i < parts.len() {
        match parts[i] {
            ">" | "1>" => {
                if i + 1 < parts.len() {
                    output_path = Some(parts[i + 1]);
                    i += 1;
                }
            }
            "2>" => {
                if i + 1 < parts.len() {
                    error_path = Some(parts[i + 1]);
                    i += 1;
                }
            }
            _ => dir_path = parts[i],
        }
        i += 1;
    }

    let path_obj = Path::new(dir_path);

    // ✅ check if the path exists first
    if !path_obj.exists() {
        let err_msg = format!("ls: {}: No such file or directory\n", dir_path);
        if let Some(path) = error_path {
            let _ = File::create(path).and_then(|mut f| f.write_all(err_msg.as_bytes()));
        } else {
            eprint!("{}", err_msg);
        }
        return;
    }

    // ✅ then check if it’s a directory
    if !path_obj.is_dir() {
        let err_msg = format!("ls: {}: Not a directory\n", dir_path);
        if let Some(path) = error_path {
            let _ = File::create(path).and_then(|mut f| f.write_all(err_msg.as_bytes()));
        } else {
            eprint!("{}", err_msg);
        }
        return;
    }

    // ✅ list directory
    let mut entries: Vec<String> = vec![];
    match fs::read_dir(path_obj) {
        Ok(dir_entries) => {
            for entry in dir_entries {
                if let Ok(e) = entry {
                    entries.push(e.file_name().to_string_lossy().to_string());
                }
            }
        }
        Err(err) => {
            let err_msg = format!("ls: cannot read directory '{}': {}\n", dir_path, err);
            if let Some(path) = error_path {
                let _ = File::create(path).and_then(|mut f| f.write_all(err_msg.as_bytes()));
            } else {
                eprint!("{}", err_msg);
            }
            return;
        }
    }

    entries.sort();
    let output = entries.join("\n") + "\n";

    if let Some(path) = output_path {
        if let Err(err) = File::create(path).and_then(|mut f| f.write_all(output.as_bytes())) {
            let err_msg = format!("ls: failed to write to '{}': {}\n", path, err);
            if let Some(err_path) = error_path {
                let _ = File::create(err_path).and_then(|mut f| f.write_all(err_msg.as_bytes()));
            } else {
                eprint!("{}", err_msg);
            }
        }
    } else {
        print!("{}", output);
    }
}
