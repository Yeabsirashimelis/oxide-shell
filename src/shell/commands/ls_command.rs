use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::Path,
};

pub fn run_ls_command(command: &str) {
    let parts: Vec<&str> = command.trim().split_whitespace().collect();

    let mut dir_path = ".";
    let mut output_path: Option<(&str, bool)> = None; // (path, append)
    let mut error_path: Option<(&str, bool)> = None; // (path, append)

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

    let mut write_error = |msg: String| {
        if let Some((path, append)) = error_path {
            let _ = open_file(path, append).and_then(|mut f| f.write_all(msg.as_bytes()));
        } else {
            eprint!("{}", msg);
        }
    };

    if !path_obj.exists() {
        write_error(format!("ls: {}: No such file or directory\n", dir_path));
        return;
    }

    if !path_obj.is_dir() {
        write_error(format!("ls: {}: Not a directory\n", dir_path));
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
            write_error(format!(
                "ls: cannot read directory '{}': {}\n",
                dir_path, err
            ));
            return;
        }
    }

    entries.sort();
    let output = entries.join("\n") + "\n";

    if let Some((path, append)) = output_path {
        let _ = open_file(path, append).and_then(|mut f| f.write_all(output.as_bytes()));
    } else {
        print!("{}", output);
    }
}

fn open_file(path: &str, append: bool) -> std::io::Result<File> {
    if append {
        OpenOptions::new().create(true).append(true).open(path)
    } else {
        File::create(path)
    }
}
