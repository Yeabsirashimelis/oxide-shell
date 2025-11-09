use std::{fs, io::Write, path::Path};

pub fn run_ls_command(command: &str) {
    let parts: Vec<&str> = command.split_whitespace().collect();
    let mut output_path: Option<&str> = None;
    let mut dir_path = ".";

    let mut i = 1;
    while i < parts.len() {
        match parts[i] {
            ">" | "1>" => {
                if i + 1 < parts.len() {
                    output_path = Some(parts[i + 1]);
                    break;
                }
            }
            _ => dir_path = parts[i],
        }
        i += 1;
    }

    let path_obj = Path::new(dir_path);
    let mut entries_vec = Vec::new();

    if path_obj.is_dir() {
        match fs::read_dir(path_obj) {
            Ok(entries) => {
                for entry in entries {
                    if let Ok(e) = entry {
                        entries_vec.push(e.file_name().to_string_lossy().to_string());
                    }
                }
                entries_vec.sort(); // SORT alphabetically
            }
            Err(err) => {
                eprintln!("ls: cannot open '{}': {}", dir_path, err);
                return;
            }
        }
    } else {
        eprintln!("ls: cannot access '{}': Not a directory", dir_path);
        return;
    }

    // Combine entries into output string
    let output = entries_vec.join("\n") + "\n";

    // Handle redirection
    if let Some(path) = output_path {
        if let Some(parent) = Path::new(path).parent() {
            if !parent.exists() {
                if let Err(err) = fs::create_dir_all(parent) {
                    eprintln!("ls: failed to create parent dirs: {}", err);
                    return;
                }
            }
        }
        match fs::File::create(path) {
            Ok(mut file) => {
                if let Err(err) = file.write_all(output.as_bytes()) {
                    eprintln!("ls: failed to write to '{}': {}", path, err);
                }
            }
            Err(err) => eprintln!("ls: failed to open '{}': {}", path, err),
        }
    } else {
        print!("{}", output);
    }
}
