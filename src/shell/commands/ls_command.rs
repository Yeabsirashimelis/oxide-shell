use std::fs;
use std::io::Write;
use std::path::Path;

use crate::shell::commands::cat_command::open_file;

pub fn run_ls_command(
    command: &str,
    stdout_redir: Option<(&str, bool)>,
    stderr_redir: Option<(&str, bool)>,
) {
    let parts: Vec<&str> = command.trim().split_whitespace().collect();
    let mut dir_path = ".";

    // parse the path argument (ignore redirections, they are handled externally)
    if parts.len() > 1 && !parts[1].starts_with('>') && !parts[1].starts_with("2") {
        dir_path = parts[1];
    }

    let path_obj = Path::new(dir_path);

    if !path_obj.exists() {
        let err_msg = format!("ls: {}: No such file or directory\n", dir_path);
        if let Some((path, append)) = stderr_redir {
            if let Ok(mut f) = open_file(Path::new(path), append) {
                let _ = f.write_all(err_msg.as_bytes());
            }
        } else {
            eprint!("{}", err_msg);
        }
        return;
    }

    if !path_obj.is_dir() {
        let err_msg = format!("ls: {}: Not a directory\n", dir_path);
        if let Some((path, append)) = stderr_redir {
            if let Ok(mut f) = open_file(Path::new(path), append) {
                let _ = f.write_all(err_msg.as_bytes());
            }
        } else {
            eprint!("{}", err_msg);
        }
        return;
    }

    let mut entries: Vec<String> = fs::read_dir(path_obj)
        .map(|dir| {
            dir.filter_map(|e| e.ok())
                .map(|entry| entry.file_name().to_string_lossy().to_string())
                .collect()
        })
        .unwrap_or_default();

    entries.sort();
    let output = entries.join("\n") + "\n";

    if let Some((path, append)) = stdout_redir {
        if let Ok(mut f) = open_file(Path::new(path), append) {
            let _ = f.write_all(output.as_bytes());
        }
    } else {
        print!("{}", output);
    }
}
