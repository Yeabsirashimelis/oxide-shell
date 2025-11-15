use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;

// --- Utility Functions ---
// These should be accessible by all command modules (like ls)

pub fn open_file(path: &Path, append: bool) -> std::io::Result<File> {
    let mut options = OpenOptions::new();
    options.create(true);
    if append {
        options.append(true);
    } else {
        options.write(true).truncate(true);
    }
    options.open(path)
}

fn unquote_path(path: &str) -> String {
    let mut s = path.trim().to_string();
    if (s.starts_with('\'') && s.ends_with('\'')) || (s.starts_with('"') && s.ends_with('"')) {
        s = s[1..s.len() - 1].to_string();
    }
    s
}

fn read_file(path: &str) -> Result<String, io::Error> {
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

// --- Main Command Logic ---

pub fn run_cat_command(args: Vec<String>) {
    let mut files: Vec<String> = args.into_iter().skip(1).collect();

    let mut output_path: Option<(String, bool)> = None;
    let mut error_path: Option<(String, bool)> = None;

    // Parse stdout redirection
    if let Some(pos) = files.iter().position(|a| a == ">>" || a == "1>>") {
        if pos + 1 < files.len() {
            output_path = Some((files[pos + 1].clone(), true));
            files.drain(pos..=pos + 1);
        }
    }
    if let Some(pos) = files.iter().position(|a| a == ">" || a == "1>") {
        if pos + 1 < files.len() {
            output_path = Some((files[pos + 1].clone(), false));
            files.drain(pos..=pos + 1);
        }
    }

    // Parse stderr redirection
    if let Some(pos) = files.iter().position(|a| a == "2>>") {
        if pos + 1 < files.len() {
            error_path = Some((files[pos + 1].clone(), true));
            files.drain(pos..=pos + 1);
        }
    }
    if let Some(pos) = files.iter().position(|a| a == "2>") {
        if pos + 1 < files.len() {
            error_path = Some((files[pos + 1].clone(), false));
            files.drain(pos..=pos + 1);
        }
    }

    let mut total_content = Vec::new();

    for file_path in &files {
        let clean_path = unquote_path(file_path);

        match read_file(&clean_path) {
            Ok(content) => total_content.push(content),
            Err(_) => {
                let err_msg = format!("cat: {}: No such file or directory\n", clean_path);

                if let Some((path, append)) = &error_path {
                    // Write to redirected stderr file ONLY
                    if let Ok(mut file) = open_file(Path::new(path), *append) {
                        let _ = file.write_all(err_msg.as_bytes());
                        let _ = file.flush();
                    }
                } else {
                    // Write to actual stderr (console)
                    let _ = eprint!("{}", err_msg);
                }
            }
        }
    }

    let joined = total_content.join("");

    // Write to stdout file if redirected
    if let Some((path, append)) = output_path {
        if let Ok(mut file) = open_file(Path::new(&path), append) {
            let _ = file.write_all(joined.as_bytes());
            let _ = file.flush();
        } else if !joined.is_empty() {
            // fallback to normal stdout if file fails
            print!("{}", joined);
        }
    } else if !joined.is_empty() {
        // no redirection → print to console
        print!("{}", joined);
    }
}
