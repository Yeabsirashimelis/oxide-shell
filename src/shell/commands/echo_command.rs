use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

pub fn run_echo_command(input: String) {
    let input = input.trim();

    let mut text_part = input;
    let mut output_path: Option<&str> = None;
    let mut error_path: Option<&str> = None;

    //Detect 2> (stderr redirection)
    if input.contains("2>") {
        let parts: Vec<&str> = input.splitn(2, "2>").collect();
        text_part = parts[0].trim();
        error_path = Some(parts[1].trim());
    }

    //Detect > or 1> (stdout redirection)
    if text_part.contains("1>") {
        let parts: Vec<&str> = text_part.splitn(2, "1>").collect();
        text_part = parts[0].trim();
        output_path = Some(parts[1].trim());
    } else if text_part.contains('>') {
        let parts: Vec<&str> = text_part.splitn(2, '>').collect();
        text_part = parts[0].trim();
        output_path = Some(parts[1].trim());
    }

    //Remove the "echo" keyword
    let text_part = text_part.trim_start_matches("echo").trim();

    // Trim quotes if present
    let message = text_part.trim_matches('\'');

    //Handle stdout redirection (>)
    if let Some(path) = output_path {
        let output_path = Path::new(path);
        if let Some(parent) = output_path.parent() {
            if !parent.exists() {
                if let Err(err) = fs::create_dir_all(parent) {
                    eprintln!("echo: could not create directories: {}", err);
                    return;
                }
            }
        }

        match File::create(output_path) {
            Ok(mut file) => {
                if let Err(err) = writeln!(file, "{}", message) {
                    eprintln!("echo: failed to write: {}", err);
                }
            }
            Err(err) => eprintln!("echo: failed to create {}: {}", path, err),
        }

        return;
    }

    //Handle stderr redirection (2>)
    if let Some(path) = error_path {
        let error_path = Path::new(path);
        if let Some(parent) = error_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let _ = File::create(error_path);
    }

    println!("{}", message);
}
