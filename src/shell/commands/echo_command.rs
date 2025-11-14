use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;

pub fn run_echo_command(raw: String) {
    // --- Proper shell-style tokenization ---
    let mut parts = split_tokens(&raw);

    let mut stdout_path: Option<(String, bool)> = None;
    let mut stderr_path: Option<(String, bool)> = None;

    // --- Parse stderr first ---
    if let Some(pos) = parts.iter().position(|a| a == "2>>") {
        stderr_path = Some((parts[pos + 1].clone(), true));
        parts.drain(pos..=pos + 1);
    } else if let Some(pos) = parts.iter().position(|a| a == "2>") {
        stderr_path = Some((parts[pos + 1].clone(), false));
        parts.drain(pos..=pos + 1);
    }

    // --- Parse stdout ---
    if let Some(pos) = parts.iter().position(|a| a == "1>>" || a == ">>") {
        stdout_path = Some((parts[pos + 1].clone(), true));
        parts.drain(pos..=pos + 1);
    } else if let Some(pos) = parts.iter().position(|a| a == "1>" || a == ">") {
        stdout_path = Some((parts[pos + 1].clone(), false));
        parts.drain(pos..=pos + 1);
    }

    // --- Reconstruct echo output text ---
    let text = parts.join(" ");
    let cleaned = strip_outer_quotes(&text);
    let output = format!("{}\n", cleaned);

    // --- Handle stderr redirection ---
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

    // --- Handle stdout redirection ---
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

    // --- Normal echo ---
    print!("{}", output);
}

/// Proper shell-like tokenizer that preserves quoted strings as single tokens.
fn split_tokens(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    let mut chars = input.chars().peekable();
    let mut in_quotes = false;
    let mut quote_char = ' ';

    while let Some(c) = chars.next() {
        if in_quotes {
            current.push(c);
            if c == quote_char {
                in_quotes = false;
            }
        } else {
            match c {
                '"' | '\'' => {
                    in_quotes = true;
                    quote_char = c;
                    current.push(c);
                }
                ' ' | '\t' => {
                    if !current.is_empty() {
                        tokens.push(current.clone());
                        current.clear();
                    }
                }
                _ => current.push(c),
            }
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

/// Removes outer quotes ONLY if the entire string is quoted
fn strip_outer_quotes(s: &str) -> String {
    if s.len() >= 2 {
        let first = s.chars().next().unwrap();
        let last = s.chars().last().unwrap();
        if (first == '"' && last == '"') || (first == '\'' && last == '\'') {
            return s[1..s.len() - 1].to_string();
        }
    }
    s.to_string()
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
