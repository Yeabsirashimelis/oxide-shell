use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;

use crate::shell::commands::cat_command::open_file;
pub fn run_echo_command(input: String) {
    let input = input.trim();

    let mut text_part = input;
    let mut output_path: Option<(&str, bool)> = None;
    let mut error_path: Option<(&str, bool)> = None;

    if input.contains("2>>") {
        let parts: Vec<&str> = input.splitn(2, "2>>").collect();
        text_part = parts[0].trim();
        error_path = Some((parts[1].trim(), true));
    } else if input.contains("2>") {
        let parts: Vec<&str> = input.splitn(2, "2>").collect();
        text_part = parts[0].trim();
        error_path = Some((parts[1].trim(), false));
    }

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

    if let Some((path, append)) = output_path {
        let _ = open_file(Path::new(path), append);
    }
    if let Some((path, append)) = error_path {
        let _ = open_file(Path::new(path), append);
    }

    // Write to stdout file if it exists
    if let Some((path, append)) = output_path {
        let _ = open_file(Path::new(path), append).and_then(|mut f| writeln!(f, "{}", message));
    }

    // Write to stderr file if it exists
    if let Some((path, append)) = error_path {
        let _ = open_file(Path::new(path), append).and_then(|mut f| writeln!(f, "{}", message));
    }

    // Print to stdout only if NO stderr redirection
    if output_path.is_none() && error_path.is_none() {
        println!("{}", message);
    }
}
