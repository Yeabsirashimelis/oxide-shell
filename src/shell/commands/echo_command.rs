use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;

pub fn run_echo_command(raw: String) {
    // --------- Split into tokens ---------
    let parts = split_tokens(&raw);

    let mut stdout_path: Option<(String, bool)> = None;
    let mut stderr_path: Option<(String, bool)> = None;

    let mut args = parts.clone();

    // --------- Parse stderr redirection ---------
    if let Some(pos) = args.iter().position(|a| a == "2>>") {
        if pos + 1 < args.len() {
            stderr_path = Some((args[pos + 1].clone(), true));
            args.drain(pos..=pos + 1);
        }
    } else if let Some(pos) = args.iter().position(|a| a == "2>") {
        if pos + 1 < args.len() {
            stderr_path = Some((args[pos + 1].clone(), false));
            args.drain(pos..=pos + 1);
        }
    }

    // --------- Parse stdout redirection ---------
    if let Some(pos) = args.iter().position(|a| a == "1>>" || a == ">>") {
        if pos + 1 < args.len() {
            stdout_path = Some((args[pos + 1].clone(), true));
            args.drain(pos..=pos + 1);
        }
    } else if let Some(pos) = args.iter().position(|a| a == "1>" || a == ">") {
        if pos + 1 < args.len() {
            stdout_path = Some((args[pos + 1].clone(), false));
            args.drain(pos..=pos + 1);
        }
    }

    // --------- Build the output ---------
    let joined = args.join(" ");
    let output = remove_quotes(&joined) + "\n";

    // --------- Handle stderr redirection ---------
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

    // --------- Handle stdout redirection ---------
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

    // --------- Normal echo ---------
    print!("{}", output);
}

fn split_tokens(input: &str) -> Vec<String> {
    let mut parts = vec![];
    let mut current = String::new();
    let mut in_quotes = false;
    let mut quote_char = ' ';

    for c in input.chars() {
        if (c == '"' || c == '\'') && !in_quotes {
            in_quotes = true;
            quote_char = c;
            current.push(c);
        } else if in_quotes && c == quote_char {
            in_quotes = false;
            current.push(c);
        } else if c.is_whitespace() && !in_quotes {
            if !current.is_empty() {
                parts.push(current.clone());
                current.clear();
            }
        } else {
            current.push(c);
        }
    }

    if !current.is_empty() {
        parts.push(current);
    }

    parts
}

fn remove_quotes(s: &str) -> String {
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
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
