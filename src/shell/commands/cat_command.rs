use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;

// Open a file for redirection
pub fn open_file(path: &Path, append: bool) -> io::Result<File> {
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

pub fn run_cat_command(
    args: Vec<String>,
    stdout_redir: Option<(&str, bool)>,
    stderr_redir: Option<(&str, bool)>,
) {
    let mut files: Vec<String> = args.into_iter().skip(1).collect();
    let mut total_content = Vec::new();

    for file_path in &files {
        let clean_path = unquote_path(file_path);

        match read_file(&clean_path) {
            Ok(content) => total_content.push(content),
            Err(_) => {
                let err_msg = format!("cat: {}: No such file or directory\n", clean_path);
                if let Some((path, append)) = &stderr_redir {
                    if let Ok(mut f) = open_file(Path::new(path), *append) {
                        let _ = f.write_all(err_msg.as_bytes());
                        let _ = f.flush();
                    }
                } else {
                    eprint!("{}", err_msg);
                }
            }
        }
    }

    let joined = total_content.join("");

    if let Some((path, append)) = stdout_redir {
        if let Ok(mut f) = open_file(Path::new(path), append) {
            let _ = f.write_all(joined.as_bytes());
            let _ = f.flush();
        }
    } else if !joined.is_empty() {
        print!("{}", joined);
    }
}
