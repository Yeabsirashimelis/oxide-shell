use std::{
    fs::{self, OpenOptions},
    io::Write,
};

pub fn run_cat_command(args: Vec<String>) {
    let mut files: Vec<String> = args.into_iter().skip(1).collect();

    let mut output_path: Option<String> = None;
    let mut append_stdout = false;
    let mut error_path: Option<String> = None;
    let mut append_stderr = false;

    if let Some(pos) = files
        .iter()
        .position(|a| [" >", "1>", ">>", "1>>", "2>", "2>>"].contains(&a.as_str()))
    {
        if pos + 1 < files.len() {
            match files[pos].as_str() {
                ">" | "1>" => {
                    append_stdout = false;
                    output_path = Some(files[pos + 1].clone());
                }
                ">>" | "1>>" => {
                    append_stdout = true;
                    output_path = Some(files[pos + 1].clone());
                }
                "2>" => {
                    append_stderr = false;
                    error_path = Some(files[pos + 1].clone());
                }
                "2>>" => {
                    append_stderr = true;
                    error_path = Some(files[pos + 1].clone());
                }
                _ => {}
            }
        }
        files.drain(pos..);
    }

    let write_error = |msg: &str| {
        if let Some(path) = &error_path {
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

    let mut total_content = Vec::new();

    for file_path in &files {
        let path = file_path.trim_matches(|c| c == '\'' || c == '"');
        match fs::read_to_string(path) {
            Ok(content) => total_content.push(content),
            Err(_) => write_error(&format!("cat: {}: No such file or directory\n", path)),
        }
    }

    let joined = total_content.join("");

    if let Some(path) = output_path {
        let mut options = OpenOptions::new();
        options.create(true);
        if append_stdout {
            options.append(true);
        } else {
            options.write(true).truncate(true);
        }
        let _ = options
            .open(path)
            .and_then(|mut f| f.write_all(joined.as_bytes()));
    } else {
        print!("{}", joined);
    }
}
