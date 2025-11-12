use std::io::Write;
use std::path::Path;

use crate::shell::commands::cat_command::open_file;

pub fn run_echo_command(input: String) {
    let mut input = input.trim().to_string();
    let mut output_path: Option<(&str, bool)> = None;
    let mut error_path: Option<(&str, bool)> = None;

    // Parse stderr redirection first
    if let Some(idx) = input.find("2>>") {
        let (before, after) = input.split_at(idx);
        input = before.trim().to_string();
        error_path = Some((after.trim_start_matches("2>>").trim(), true));
    } else if let Some(idx) = input.find("2>") {
        let (before, after) = input.split_at(idx);
        input = before.trim().to_string();
        error_path = Some((after.trim_start_matches("2>").trim(), false));
    }

    // Parse stdout redirection
    if let Some(idx) = input.find("1>>") {
        let (before, after) = input.split_at(idx);
        input = before.trim().to_string();
        output_path = Some((after.trim_start_matches("1>>").trim(), true));
    } else if let Some(idx) = input.find(">>") {
        let (before, after) = input.split_at(idx);
        input = before.trim().to_string();
        output_path = Some((after.trim_start_matches(">>").trim(), true));
    } else if let Some(idx) = input.find("1>") {
        let (before, after) = input.split_at(idx);
        input = before.trim().to_string();
        output_path = Some((after.trim_start_matches("1>").trim(), false));
    } else if let Some(idx) = input.find('>') {
        let (before, after) = input.split_at(idx);
        input = before.trim().to_string();
        output_path = Some((after.trim_start_matches('>').trim(), false));
    }

    // Extract the echo message
    let text_part = input.trim_start_matches("echo").trim();
    let message = text_part
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .or_else(|| {
            text_part
                .strip_prefix('\'')
                .and_then(|s| s.strip_suffix('\''))
        })
        .unwrap_or(text_part);

    // Write to stderr if specified
    if let Some((path, append)) = error_path {
        if let Ok(mut file) = open_file(Path::new(path), append) {
            let _ = writeln!(file, "{}", message);
            let _ = file.flush();
            return; // don't print anything to stdout
        }
    }

    // Write to stdout file if specified
    if let Some((path, append)) = output_path {
        if let Ok(mut file) = open_file(Path::new(path), append) {
            let _ = writeln!(file, "{}", message);
            let _ = file.flush();
            return;
        }
    }

    // Default to console output
    println!("{}", message);
}
