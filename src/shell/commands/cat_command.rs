use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;

pub fn run_cat_command(args: Vec<String>) {
    let mut files: Vec<String> = args.into_iter().skip(1).collect();

    let mut output_path: Option<(String, bool)> = None;
    let mut error_path: Option<(String, bool)> = None;

    // Parse stdout
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

    // Parse stderr
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

    for file_path in &files {
        let clean_path = unquote_path(file_path);

        match read_file(&clean_path) {
            Ok(content) => {
                if let Some((path, append)) = &output_path {
                    if let Ok(mut f) = open_file(Path::new(path), *append) {
                        let _ = f.write_all(content.as_bytes());
                        let _ = f.flush();
                        continue;
                    }
                } else {
                    print!("{}", content);
                    let _ = io::stdout().flush();
                }
            }
            Err(_) => {
                let err_msg = format!("cat: {}: No such file or directory\n", clean_path);
                if let Some((path, append)) = &error_path {
                    if let Ok(mut f) = open_file(Path::new(path), *append) {
                        let _ = f.write_all(err_msg.as_bytes());
                        let _ = f.flush();
                        continue;
                    }
                }
                let _ = eprint!("{}", err_msg);
                let _ = io::stderr().flush();
            }
        }
    }
}

pub fn open_file(path: &Path, append: bool) -> io::Result<File> {
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
    let s = path.trim();
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

fn read_file(path: &str) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}
