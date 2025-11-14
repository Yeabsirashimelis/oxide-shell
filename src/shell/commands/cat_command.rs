use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;

/// Run the `cat` command with stdout/stderr redirection support
pub fn run_cat_command(args: Vec<String>) {
    // args[0] = "cat", remaining are actual filenames and redirections
    let mut files: Vec<String> = args.into_iter().skip(1).collect();

    let mut output_path: Option<(String, bool)> = None;
    let mut error_path: Option<(String, bool)> = None;

    // ---- Parse stderr redirection first ----
    if let Some(pos) = files.iter().position(|a| a == "2>>") {
        if pos + 1 < files.len() {
            error_path = Some((files[pos + 1].clone(), true)); // true for append
            files.drain(pos..=pos + 1);
        }
    } else if let Some(pos) = files.iter().position(|a| a == "2>") {
        if pos + 1 < files.len() {
            error_path = Some((files[pos + 1].clone(), false)); // false for overwrite
            files.drain(pos..=pos + 1);
        }
    }

    // ---- Parse stdout redirection second ----
    if let Some(pos) = files.iter().position(|a| a == "1>>" || a == ">>") {
        if pos + 1 < files.len() {
            output_path = Some((files[pos + 1].clone(), true)); // true for append
            files.drain(pos..=pos + 1);
        }
    } else if let Some(pos) = files.iter().position(|a| a == "1>" || a == ">") {
        if pos + 1 < files.len() {
            output_path = Some((files[pos + 1].clone(), false)); // false for overwrite
            files.drain(pos..=pos + 1);
        }
    }

    let mut all_content: Vec<String> = Vec::new();

    for file_arg in &files {
        let clean_path = unquote_path(file_arg);

        match read_file(&clean_path) {
            Ok(content) => all_content.push(content),
            Err(_) => {
                let msg = format!("cat: {}: No such file or directory\n", clean_path);

                if let Some((err_file, append)) = &error_path {
                    if let Ok(mut f) = open_file(Path::new(err_file), *append) {
                        let _ = f.write_all(msg.as_bytes());
                        let _ = f.flush();
                    }
                } else {
                    eprint!("{}", msg);
                }
            }
        }
    }

    let final_output = all_content.join("");

    // Write to stdout if redirected, else print normally
    if let Some((out_file, append)) = output_path {
        if let Ok(mut f) = open_file(Path::new(&out_file), append) {
            let _ = f.write_all(final_output.as_bytes());
            let _ = f.flush();
        } else {
            print!("{}", final_output);
        }
    } else {
        print!("{}", final_output);
    }
}

pub fn open_file(path: &Path, append: bool) -> std::io::Result<File> {
    let mut opts = OpenOptions::new();
    opts.create(true);
    if append {
        opts.append(true);
    } else {
        opts.write(true).truncate(true);
    }
    opts.open(path)
}

fn unquote_path(path: &str) -> String {
    let mut p = path.trim().to_string();
    if (p.starts_with('"') && p.ends_with('"')) || (p.starts_with('\'') && p.ends_with('\'')) {
        p = p[1..p.len() - 1].to_string();
    }
    p
}

fn read_file(path: &str) -> Result<String, io::Error> {
    let mut f = File::open(path)?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    Ok(s)
}
