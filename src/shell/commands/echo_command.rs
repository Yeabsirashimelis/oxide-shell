use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::Path,
};

pub fn run_echo_command(input: &str) {
    let input = input.trim();

    let mut append_stdout = false;
    let mut append_stderr = false;
    let mut output_path: Option<&str> = None;
    let mut error_path: Option<&str> = None;

    let mut text_part = input;

    // Detect operators
    for op in ["1>>", ">>", "2>>", "2>", "1>", ">"] {
        if input.contains(op) {
            let parts: Vec<&str> = input.splitn(2, op).collect();
            text_part = parts[0].trim_start_matches("echo").trim();
            let path = parts[1].trim();
            match op {
                ">" | "1>" => {
                    output_path = Some(path);
                    append_stdout = false;
                }
                ">>" | "1>>" => {
                    output_path = Some(path);
                    append_stdout = true;
                }
                "2>" => {
                    error_path = Some(path);
                    append_stderr = false;
                }
                "2>>" => {
                    error_path = Some(path);
                    append_stderr = true;
                }
                _ => {}
            }
            break;
        }
    }

    let write_error = |msg: &str| {
        if let Some(path) = error_path {
            let mut options = OpenOptions::new();
            options.create(true);
            if append_stderr {
                options.append(true);
            } else {
                options.write(true).truncate(true);
            }
            let _ = options
                .open(path)
                .and_then(|mut f| f.write_all(msg.as_bytes()));
        } else {
            eprint!("{}", msg);
        }
    };

    if let Some(path) = output_path {
        let mut options = OpenOptions::new();
        options.create(true);
        if append_stdout {
            options.append(true);
        } else {
            options.write(true).truncate(true);
        }
        if let Err(err) = options
            .open(path)
            .and_then(|mut f| f.write_all(text_part.as_bytes()))
        {
            write_error(&format!("echo: failed to write: {}\n", err));
        }
    } else {
        print!("{}", text_part);
    }
}
