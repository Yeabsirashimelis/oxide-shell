use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::Path,
};

pub fn run_cat_command(args: Vec<String>) {
    let mut files: Vec<String> = args.into_iter().skip(1).collect();

    let mut output_path: Option<String> = None;
    let mut append_stdout = false;
    let mut error_path: Option<String> = None;
    let mut append_stderr = false;

    // detect redirection
    if let Some(pos) = files
        .iter()
        .position(|a| ["1>", ">", "1>>", ">>"].contains(&a.as_str()))
    {
        let op = files[pos].as_str();
        if pos + 1 < files.len() {
            output_path = Some(files[pos + 1].clone());
            append_stdout = op.ends_with(">>");
        }
        files.drain(pos..=pos + 1);
    }

    if let Some(pos) = files
        .iter()
        .position(|a| ["2>", "2>>"].contains(&a.as_str()))
    {
        let op = files[pos].as_str();
        if pos + 1 < files.len() {
            error_path = Some(files[pos + 1].clone());
            append_stderr = op.ends_with(">>");
        }
        files.drain(pos..=pos + 1);
    }

    let mut total_content = Vec::new();

    for file_path in &files {
        match File::open(file_path) {
            Ok(mut f) => {
                let mut content = String::new();
                if let Err(_) = f.read_to_string(&mut content) {
                    if let Some(path) = &error_path {
                        let path_obj = Path::new(path);
                        if let Some(parent) = path_obj.parent() {
                            let _ = fs::create_dir_all(parent);
                        }
                        let mut options = OpenOptions::new();
                        options.create(true).append(true).write(true);
                        let _ = options
                            .open(path_obj)
                            .and_then(|mut f| writeln!(f, "cat: {}: Could not read", file_path));
                    }
                    continue;
                }
                total_content.push(content);
            }
            Err(_) => {
                if let Some(path) = &error_path {
                    let path_obj = Path::new(path);
                    if let Some(parent) = path_obj.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    let mut options = OpenOptions::new();
                    options.create(true).append(true).write(true);
                    let _ = options.open(path_obj).and_then(|mut f| {
                        writeln!(f, "cat: {}: No such file or directory", file_path)
                    });
                } else {
                    eprintln!("cat: {}: No such file or directory", file_path);
                }
            }
        }
    }

    let joined = total_content.join("");

    if let Some(path) = output_path {
        let path_obj = Path::new(&path);
        if let Some(parent) = path_obj.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let mut options = OpenOptions::new();
        options.create(true);
        if append_stdout {
            options.append(true);
        } else {
            options.write(true).truncate(true);
        }
        let _ = options
            .open(path_obj)
            .and_then(|mut f| write!(f, "{}", joined));
    } else {
        print!("{}", joined);
    }
}
