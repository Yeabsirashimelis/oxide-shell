use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

pub fn run_ls_command(command: &str) {
    // Example command:
    // "ls /tmp/baz > /tmp/foo/baz.md"

    let parts: Vec<&str> = command.split_whitespace().collect();
    let mut output_path: Option<&str> = None;
    let mut dir_path = "."; // default to current directory

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
    let mut output = String::new();

    if path_obj.is_dir() {
        match fs::read_dir(path_obj) {
            Ok(entries) => {
                for entry in entries {
                    if let Ok(e) = entry {
                        output.push_str(&format!("{}\n", e.file_name().to_string_lossy()));
                    }
                }
            }
            Err(err) => output.push_str(&format!("ls: cannot open '{}': {}\n", dir_path, err)),
        }
    } else {
        output.push_str(&format!(
            "ls: cannot access '{}': Not a directory\n",
            dir_path
        ));
    }

    // Now handle redirection
    if let Some(path) = output_path {
        if let Some(parent) = Path::new(path).parent() {
            if !parent.exists() {
                if let Err(err) = fs::create_dir_all(parent) {
                    eprintln!("ls: failed to create parent dirs: {}", err);
                    return;
                }
            }
        }

        match File::create(path) {
            Ok(mut file) => {
                if let Err(err) = file.write_all(output.as_bytes()) {
                    eprintln!("ls: failed to write to '{}': {}", path, err);
                }
            }
            Err(err) => eprintln!("ls: failed to open '{}': {}", path, err),
        }
    } else {
        // No redirection, print to stdout
        print!("{}", output);
    }
}
