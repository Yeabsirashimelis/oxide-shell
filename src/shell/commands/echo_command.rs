use std::io::Write;
use std::path::Path;

use crate::shell::commands::cat_command::open_file;

pub fn run_echo_command(input: String) {
    let input = input.trim();

    let mut text_part = input;
    let mut output_path: Option<(&str, bool)> = None;
    let mut error_path: Option<(&str, bool)> = None;

    // parse stderr redirection first
    if input.contains("2>>") {
        let parts: Vec<&str> = input.splitn(2, "2>>").collect();
        text_part = parts[0].trim();
        error_path = Some((parts[1].trim(), true));
    } else if input.contains("2>") {
        let parts: Vec<&str> = input.splitn(2, "2>").collect();
        text_part = parts[0].trim();
        error_path = Some((parts[1].trim(), false));
    }

    // parse stdout redirection
    if text_part.contains("1>>") {
        let parts: Vec<&str> = text_part.splitn(2, "1>>").collect();
        text_part = parts[0].trim();
        output_path = Some((parts[1].trim(), true));
    } else if text_part.contains(">>") {
        let parts: Vec<&str> = text_part.splitn(2, ">>").collect();
        text_part = parts[0].trim();
        output_path = Some((parts[1].trim(), true));
    } else if text_part.contains("1>") {
        let parts: Vec<&str> = text_part.splitn(2, "1>").collect();
        text_part = parts[0].trim();
        output_path = Some((parts[1].trim(), false));
    } else if text_part.contains('>') {
        let parts: Vec<&str> = text_part.splitn(2, '>').collect();
        text_part = parts[0].trim();
        output_path = Some((parts[1].trim(), false));
    }
    let text_part = text_part
        .strip_prefix("echo")
        .map(|s| s.trim_start())
        .unwrap_or(text_part)
        .trim();

    let message = text_part
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .or_else(|| {
            text_part
                .strip_prefix('\'')
                .and_then(|s| s.strip_suffix('\''))
        })
        .unwrap_or(text_part);

    // Ensure files exist first
    if let Some((path, append)) = output_path {
        let _ = open_file(Path::new(path), append);
    }
    if let Some((path, append)) = error_path {
        let _ = open_file(Path::new(path), append);
    }

    // Write to stdout file if exists
    if let Some((path, append)) = output_path {
        if let Ok(mut f) = open_file(Path::new(path), append) {
            let _ = writeln!(f, "{}", message);
            let _ = f.flush();
        }
    }

    // Write to stderr file if exists
    if let Some((path, append)) = error_path {
        if let Ok(mut f) = open_file(Path::new(path), append) {
            let _ = writeln!(f, "{}", message);
            let _ = f.flush();
        }
    }

    // Only print to stdout if no redirection exists
    if output_path.is_none() && error_path.is_none() {
        println!("{}", message);
    }
}
