use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;

use crate::shell::commands::cat_command::open_file;

pub fn run_echo_command(input: String) {
    let input = input.trim();

    let mut text_part = input;
    let mut output_path: Option<(&str, bool)> = None;
    let mut error_path: Option<(&str, bool)> = None;

    // Parse stderr redirection first
    if input.contains("2>>") {
        let parts: Vec<&str> = input.splitn(2, "2>>").collect();
        text_part = parts[0].trim();
        error_path = Some((parts[1].trim(), true));
    } else if input.contains("2>") {
        let parts: Vec<&str> = input.splitn(2, "2>").collect();
        text_part = parts[0].trim();
        error_path = Some((parts[1].trim(), false));
    }

    // Parse stdout redirection
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

    // Extract the message
    let text_part = text_part.trim_start_matches("echo").trim();
    let message = text_part
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .or_else(|| {
            text_part
                .strip_prefix('\'')
                .and_then(|s| s.strip_suffix('\''))
        })
        .unwrap_or(text_part);

    // Write to stdout file if needed
    if let Some((path, append)) = output_path {
        if let Ok(mut f) = open_file(Path::new(path), append) {
            let _ = writeln!(f, "{}", message);
            let _ = f.flush();
        }
    }

    // Write to stderr file if needed (and also emit to stderr)
    if let Some((path, append)) = error_path {
        if let Ok(mut f) = open_file(Path::new(path), append) {
            let _ = writeln!(f, "{}", message);
            let _ = f.flush();
        }
        // Output to stderr for the shell
        let _ = writeln!(io::stderr(), "{}", message);
    }

    // Print normally if no redirection
    if output_path.is_none() && error_path.is_none() {
        println!("{}", message);
    }
}
