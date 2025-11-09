use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

pub fn run_echo_command(input: String) {
    // Example input: echo 'Hello James' 1> /tmp/foo/foo.md
    let input = input.trim();

    // Detect redirection operator
    let (text_part, output_path) = if input.contains("1>") {
        let parts: Vec<&str> = input.splitn(2, "1>").collect();
        (parts[0].trim(), Some(parts[1].trim()))
    } else if input.contains('>') {
        let parts: Vec<&str> = input.splitn(2, '>').collect();
        (parts[0].trim(), Some(parts[1].trim()))
    } else {
        (input, None)
    };

    // Remove the "echo" keyword if it’s still there
    let text_part = text_part.trim_start_matches("echo").trim();

    // Handle redirection case
    if let Some(path) = output_path {
        let output_path = Path::new(path);

        // Ensure parent directories exist
        if let Some(parent) = output_path.parent() {
            if !parent.exists() {
                if let Err(err) = fs::create_dir_all(parent) {
                    eprintln!("echo: could not create directories: {}", err);
                    return;
                }
            }
        }

        // Create or overwrite file and write text
        match File::create(output_path) {
            Ok(mut file) => {
                if let Err(err) = writeln!(file, "{}", text_part.trim_matches('\'')) {
                    eprintln!("echo: failed to write: {}", err);
                }
            }
            Err(err) => eprintln!("echo: failed to create {}: {}", path, err),
        }

        return;
    }

    // Normal echo (no redirection)
    println!("{}", text_part.trim_matches('\''));
}
