use std::{
    fs::{self, File},
    io::{self, Read, Write},
    path::Path,
};

pub fn run_cat_command(args: Vec<String>) {
    let mut files: Vec<String> = args.into_iter().skip(1).collect();

    let mut output_path: Option<String> = None;
    let mut error_path: Option<String> = None;

    // detect stdout redirection
    if let Some(pos) = files.iter().position(|a| a == ">" || a == "1>") {
        if pos + 1 < files.len() {
            output_path = Some(files[pos + 1].clone());
        }
        files.drain(pos..);
    }

    // detect stderr redirection
    if let Some(pos) = files.iter().position(|a| a == "2>") {
        if pos + 1 < files.len() {
            error_path = Some(files[pos + 1].clone());
        }
        files.drain(pos..);
    }

    let mut total_content = Vec::new();

    for file_path in &files {
        let clean_path = unquote_path(file_path);

        match read_file(&clean_path) {
            Ok(content) => total_content.push(content),
            Err(_) => {
                let err_msg = format!("cat: {}: No such file or directory\n", clean_path);

                if let Some(path) = &error_path {
                    // ensure directory exists
                    let path_obj = Path::new(path);
                    if let Some(parent) = path_obj.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    let _ =
                        File::create(path_obj).and_then(|mut f| f.write_all(err_msg.as_bytes()));
                } else {
                    eprint!("{}", err_msg);
                }
            }
        }
    }

    let joined = total_content.join("");

    if let Some(path) = output_path {
        // Write stdout to file
        let path_obj = Path::new(&path);
        if let Some(parent) = path_obj.parent() {
            if !parent.exists() {
                if let Err(err) = fs::create_dir_all(parent) {
                    let msg = format!("cat: could not create directories: {}\n", err);
                    if let Some(err_path) = error_path {
                        let _ =
                            File::create(err_path).and_then(|mut f| f.write_all(msg.as_bytes()));
                    } else {
                        eprint!("{}", msg);
                    }
                    return;
                }
            }
        }

        match File::create(path_obj) {
            Ok(mut file) => {
                if let Err(err) = file.write_all(joined.as_bytes()) {
                    let msg = format!("cat: error writing to {}: {}\n", path, err);
                    if let Some(err_path) = error_path {
                        let _ =
                            File::create(err_path).and_then(|mut f| f.write_all(msg.as_bytes()));
                    } else {
                        eprint!("{}", msg);
                    }
                }
            }
            Err(err) => {
                let msg = format!("cat: cannot create {}: {}\n", path, err);
                if let Some(err_path) = error_path {
                    let _ = File::create(err_path).and_then(|mut f| f.write_all(msg.as_bytes()));
                } else {
                    eprint!("{}", msg);
                }
            }
        }
    } else {
        // print normally to stdout
        print!("{}", joined);
    }
}

fn unquote_path(path: &str) -> String {
    let mut s = path.trim().to_string();

    if (s.starts_with('\'') && s.ends_with('\'')) || (s.starts_with('"') && s.ends_with('"')) {
        s = s[1..s.len() - 1].to_string();
    }

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
