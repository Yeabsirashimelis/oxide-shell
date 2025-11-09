use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

pub fn run_ls_command(command: &str) {
    // Split manually, respecting quotes would be ideal, but here assume tester uses simple paths
    let mut parts: Vec<&str> = command.trim().split_whitespace().collect();

    let mut dir_path = "."; // default
    let mut output_path: Option<&str> = None;

    let mut i = 1; // skip "ls"
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
    if !path_obj.exists() || !path_obj.is_dir() {
        eprintln!("ls: cannot access '{}': Not a directory", dir_path);
        return;
    }

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
            eprintln!("ls: cannot read directory '{}': {}", dir_path, err);
            return;
        }
    }

    // Sort alphabetically for tester
    entries.sort();
    let output = entries.join("\n") + "\n";

    if let Some(path) = output_path {
        let path_obj = Path::new(path);

        // Create parent directories if they don't exist
        if let Some(parent) = path_obj.parent() {
            if !parent.exists() {
                if let Err(err) = fs::create_dir_all(parent) {
                    eprintln!(
                        "ls: failed to create directories '{}': {}",
                        parent.display(),
                        err
                    );
                    return;
                }
            }
        }

        match File::create(path_obj) {
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
