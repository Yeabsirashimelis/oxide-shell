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

    // Write to stderr first if redirected
    if let Some((path, append)) = error_path {
        if let Ok(mut file) = open_file(Path::new(path), append) {
            let _ = writeln!(file, "{}", message);
            let _ = file.flush();
            return; // Do not write to stdout if stderr redirection exists
        }
    }

    // Write to stdout if redirected
    if let Some((path, append)) = output_path {
        if let Ok(mut file) = open_file(Path::new(path), append) {
            let _ = writeln!(file, "{}", message);
            let _ = file.flush();
            return; // Done
        }
    }

    // Only print to actual stdout if no redirection
    print!("{}", message);
    let _ = std::io::stdout().flush();
}
