use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;

use crate::shell::commands::cat_command::open_file;

pub fn run_echo_command(input: String) {
    let input = input.trim();

    let mut text_part = input;
    let mut output_path: Option<(&str, bool)> = None;
    let mut error_path: Option<(&str, bool)> = None;

    // ---------- Parse stderr redirection ----------
    if input.contains("2>>") {
        let parts: Vec<&str> = input.splitn(2, "2>>").collect();
        text_part = parts[0].trim();
        error_path = Some((parts[1].trim(), true));
    } else if input.contains("2>") {
        let parts: Vec<&str> = input.splitn(2, "2>").collect();
        text_part = parts[0].trim();
        error_path = Some((parts[1].trim(), false));
    }

    // ---------- Parse stdout redirection ----------
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

    // ---------- Extract echo message (FIXED) ----------
    let mut args = text_part.trim_start_matches("echo").trim().to_string();

    // Remove surrounding quotes ONLY if both match
    if (args.starts_with('"') && args.ends_with('"'))
        || (args.starts_with('\'') && args.ends_with('\''))
    {
        if args.len() >= 2 {
            args = args[1..args.len() - 1].to_string();
        }
    }

    let message = args;

    // ---------- Write to stdout-redirection file ----------
    if let Some((path, append)) = output_path {
        if let Ok(mut f) = open_file(Path::new(path), append) {
            let _ = writeln!(f, "{}", message);
            let _ = f.flush();
        }
    }

    // ---------- Write to stderr-redirection file ----------
    if let Some((path, append)) = error_path {
        if let Ok(mut f) = open_file(Path::new(path), append) {
            let _ = writeln!(f, "{}", message);
            let _ = f.flush();
        }

        // Also print to stderr normally
        let _ = writeln!(io::stderr(), "{}", message);
    }

    // ---------- Print normally if no redirection ----------
    if output_path.is_none() && error_path.is_none() {
        println!("{}", message);
    }
}
