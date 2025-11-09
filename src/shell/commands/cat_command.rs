use std::{
    fs::{self, File},
    io::{self, Read, Write},
    path::Path,
};

pub fn run_cat_command(args: Vec<String>) {
    // kip the first element (the "cat" command itself)
    let mut files: Vec<String> = args.into_iter().skip(1).collect();

    // Detect redirection symbol if passed from the user
    let mut output_path: Option<String> = None;
    if let Some(pos) = files.iter().position(|a| a == ">" || a == "1>") {
        if pos + 1 < files.len() {
            output_path = Some(files[pos + 1].clone());
        }
        files.drain(pos..);
    }

    let mut total_content = Vec::new();

    for file_path in &files {
        let clean_path = unquote_path(file_path);

        match read_file(&clean_path) {
            Ok(content) => total_content.push(content),
            Err(_) => eprintln!("cat: {}: No such file or directory", clean_path),
        }
    }

    let joined = total_content.join(""); // preserves formatting exactly

    if let Some(path) = output_path {
        let path_obj = Path::new(&path);
        if let Some(parent) = path_obj.parent() {
            if !parent.exists() {
                if let Err(err) = fs::create_dir_all(parent) {
                    eprintln!("cat: could not create directories: {}", err);
                    return;
                }
            }
        }

        match File::create(path_obj) {
            Ok(mut file) => {
                if let Err(err) = file.write_all(joined.as_bytes()) {
                    eprintln!("cat: error writing to {}: {}", path, err);
                }
            }
            Err(err) => eprintln!("cat: cannot create {}: {}", path, err),
        }
    } else {
        print!("{}", joined);
    }
}

//Handle quoted paths and escape sequences
fn unquote_path(path: &str) -> String {
    let mut s = path.trim().to_string();

    //Remove outer single or double quotes
    if (s.starts_with('\'') && s.ends_with('\'')) || (s.starts_with('"') && s.ends_with('"')) {
        s = s[1..s.len() - 1].to_string();
    }

    //Handle escaped sequences if double-quoted
    if path.starts_with('"') && path.ends_with('"') {
        s = s
            .replace(r"\n", "\n")
            .replace(r"\\", "\\")
            .replace(r#"\""#, "\"");
    }

    s
}

fn read_file(path: &str) -> Result<String, io::Error> {
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}
