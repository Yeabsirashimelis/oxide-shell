use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::Path,
};

pub fn run_cat_command(args: Vec<String>) {
    let mut files: Vec<String> = args.into_iter().skip(1).collect();

    let mut output_path: Option<(String, bool)> = None;
    let mut error_path: Option<(String, bool)> = None;

    //detect stdout append
    if let Some(pos) = files.iter().position(|a| a == ">>" || a == "1>>") {
        if pos + 1 < files.len() {
            output_path = Some((files[pos + 1].clone(), true));
        }
        files.drain(pos..);
    }

    //detect stdout overwrite
    if let Some(pos) = files.iter().position(|a| a == ">" || a == "1>") {
        if pos + 1 < files.len() {
            output_path = Some((files[pos + 1].clone(), false));
        }
        files.drain(pos..);
    }

    //detect stderr append
    if let Some(pos) = files.iter().position(|a| a == "2>>") {
        if pos + 1 < files.len() {
            error_path = Some((files[pos + 1].clone(), true));
        }
        files.drain(pos..);
    }

    //detect stderr overwrite
    if let Some(pos) = files.iter().position(|a| a == "2>") {
        if pos + 1 < files.len() {
            error_path = Some((files[pos + 1].clone(), false));
        }
        files.drain(pos..);
    }

    if let Some((path, append)) = &output_path {
        let p = Path::new(path);
        if let Some(parent) = p.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = open_file(p, *append);
    }
    if let Some((path, append)) = &error_path {
        let p = Path::new(path);
        if let Some(parent) = p.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = open_file(p, *append);
    }

    let mut total_content = Vec::new();

    for file_path in &files {
        let clean_path = unquote_path(file_path);

        match read_file(&clean_path) {
            Ok(content) => total_content.push(content),
            Err(_) => {
                let err_msg = format!("cat: {}: No such file or directory\n", clean_path);

                if let Some((path, append)) = &error_path {
                    let path_obj = Path::new(path);
                    let _ = open_file(path_obj, *append)
                        .and_then(|mut f| f.write_all(err_msg.as_bytes()));
                } else {
                    eprint!("{}", err_msg);
                }
            }
        }
    }

    let joined = total_content.join("");

    if let Some((path, append)) = output_path {
        let path_obj = Path::new(&path);
        if let Some(parent) = path_obj.parent() {
            let _ = fs::create_dir_all(parent);
        }

        match open_file(path_obj, append) {
            Ok(mut file) => {
                let _ = file.write_all(joined.as_bytes());
            }
            Err(err) => {
                let msg = format!("cat: cannot open {}: {}\n", path, err);
                if let Some((err_path, append_err)) = error_path {
                    let _ = open_file(Path::new(&err_path), append_err)
                        .and_then(|mut f| f.write_all(msg.as_bytes()));
                } else {
                    eprint!("{}", msg);
                }
            }
        }
    } else {
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
