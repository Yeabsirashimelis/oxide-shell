use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    path::Path,
};

pub fn run_cat_command(args: Vec<String>) {
    let mut files: Vec<String> = args.into_iter().skip(1).collect();

    let mut output_path: Option<(String, bool)> = None;
    let mut error_path: Option<(String, bool)> = None;

    // Process ALL redirections first
    let mut i = 0;
    while i < files.len() {
        match files[i].as_str() {
            ">" | "1>" => {
                if i + 1 < files.len() {
                    output_path = Some((files[i + 1].clone(), false));
                    files.drain(i..=i + 1);
                    continue; // Don't increment i since we removed elements
                }
            }
            ">>" | "1>>" => {
                if i + 1 < files.len() {
                    output_path = Some((files[i + 1].clone(), true));
                    files.drain(i..=i + 1);
                    continue;
                }
            }
            "2>" => {
                if i + 1 < files.len() {
                    error_path = Some((files[i + 1].clone(), false));
                    files.drain(i..=i + 1);
                    continue;
                }
            }
            "2>>" => {
                if i + 1 < files.len() {
                    error_path = Some((files[i + 1].clone(), true));
                    files.drain(i..=i + 1);
                    continue;
                }
            }
            _ => {}
        }
        i += 1;
    }

    let mut total_content = Vec::new();

    for file_path in &files {
        let clean_path = unquote_path(file_path);

        match read_file(&clean_path) {
            Ok(content) => total_content.push(content),
            Err(_) => {
                let err_msg = format!("cat: {}: No such file or directory\n", clean_path);

                if let Some((path, append)) = &error_path {
                    // Clean the path and open file
                    let clean_error_path = unquote_path(path);
                    match open_file(Path::new(&clean_error_path), *append) {
                        Ok(mut file) => {
                            if let Err(_) = file.write_all(err_msg.as_bytes()) {
                                // If writing to redirected stderr fails, fall back to terminal
                                eprint!("{}", err_msg);
                            }
                        }
                        Err(_) => {
                            // If opening redirected file fails, fall back to terminal
                            eprint!("{}", err_msg);
                        }
                    }
                } else {
                    eprint!("{}", err_msg);
                }
            }
        }
    }

    let joined = total_content.join("");

    // Handle stdout output
    if let Some((path, append)) = output_path {
        let clean_output_path = unquote_path(&path);
        match open_file(Path::new(&clean_output_path), append) {
            Ok(mut file) => {
                let _ = file.write_all(joined.as_bytes());
            }
            Err(_) => {
                // If stdout redirection fails, print to terminal
                if !joined.is_empty() {
                    print!("{}", joined);
                }
            }
        }
    } else if !joined.is_empty() {
        print!("{}", joined);
    }
}

fn open_file(path: &Path, append: bool) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.create(true);

    if append {
        options.append(true);
    } else {
        options.write(true).truncate(true);
    }
    options.open(path)
}

fn unquote_path(path: &str) -> String {
    let mut s = path.trim().to_string();
    if (s.starts_with('\'') && s.ends_with('\'')) || (s.starts_with('"') && s.ends_with('"')) {
        s = s[1..s.len() - 1].to_string();
    }
    s
}

fn read_file(path: &str) -> Result<String, io::Error> {
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}
