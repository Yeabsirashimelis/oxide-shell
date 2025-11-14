use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;

pub fn run_echo_command(args: Vec<String>) {
    // args[0] = "echo", rest are actual content + redirections
    let mut parts: Vec<String> = args.into_iter().skip(1).collect();

    let mut stdout_path: Option<(String, bool)> = None; // (path, append)
    let mut stderr_path: Option<(String, bool)> = None; // (path, append)

    // ----------------------
    // Parse stderr redirection FIRST
    // ----------------------
    if let Some(pos) = parts.iter().position(|a| a == "2>>") {
        if pos + 1 < parts.len() {
            stderr_path = Some((parts[pos + 1].clone(), true));
            parts.drain(pos..=pos + 1);
        }
    } else if let Some(pos) = parts.iter().position(|a| a == "2>") {
        if pos + 1 < parts.len() {
            stderr_path = Some((parts[pos + 1].clone(), false));
            parts.drain(pos..=pos + 1);
        }
    }

    // ----------------------
    // Parse stdout redirection SECOND
    // ----------------------
    if let Some(pos) = parts.iter().position(|a| a == "1>>" || a == ">>") {
        if pos + 1 < parts.len() {
            stdout_path = Some((parts[pos + 1].clone(), true));
            parts.drain(pos..=pos + 1);
        }
    } else if let Some(pos) = parts.iter().position(|a| a == "1>" || a == ">") {
        if pos + 1 < parts.len() {
            stdout_path = Some((parts[pos + 1].clone(), false));
            parts.drain(pos..=pos + 1);
        }
    }

    // ----------------------
    // Join remaining parts into one echo output (preserve spaces)
    // ----------------------
    let raw_output = parts.join(" ");
    let output = remove_quotes(&raw_output) + "\n";

    // ----------------------
    // Handle stderr redirection
    // ----------------------
    if let Some((path, append)) = stderr_path {
        if let Ok(mut file) = open_file(Path::new(&path), append) {
            let _ = file.write_all(output.as_bytes());
            let _ = file.flush();
            return;
        } else {
            eprint!("{}", output);
            return;
        }
    }

    // ----------------------
    // Handle stdout redirection
    // ----------------------
    if let Some((path, append)) = stdout_path {
        if let Ok(mut file) = open_file(Path::new(&path), append) {
            let _ = file.write_all(output.as_bytes());
            let _ = file.flush();
            return;
        } else {
            print!("{}", output);
            return;
        }
    }

    // ----------------------
    // No redirection -> normal echo
    // ----------------------
    print!("{}", output);
}

fn open_file(path: &Path, append: bool) -> std::io::Result<File> {
    let mut opts = OpenOptions::new();
    opts.create(true);
    if append {
        opts.append(true);
    } else {
        opts.write(true).truncate(true);
    }
    opts.open(path)
}

// Removes wrapping "quotes" or 'quotes'
fn remove_quotes(text: &str) -> String {
    if (text.starts_with('"') && text.ends_with('"'))
        || (text.starts_with('\'') && text.ends_with('\''))
    {
        text[1..text.len() - 1].to_string()
    } else {
        text.to_string()
    }
}
