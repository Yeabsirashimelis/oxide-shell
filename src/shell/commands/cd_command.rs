use std::{env, path::PathBuf};

pub fn run_cd_command(path_str: &str) {
    let target_path = if path_str.trim().is_empty() || path_str == "~" {
        // Try HOME first (Linux)
        if let Ok(home) = env::var("HOME") {
            PathBuf::from(home)
            //THEN USERPROFILE (windows)
        } else if let Ok(home) = env::var("USERPROFILE") {
            PathBuf::from(home)
        } else {
            PathBuf::from("/")
        }
    } else if path_str.starts_with("~/") {
        let mut home = if let Ok(home) = env::var("HOME") {
            PathBuf::from(home)
        } else if let Ok(home) = env::var("USERPROFILE") {
            PathBuf::from(home)
        } else {
            PathBuf::from("/")
        };
        home.push(&path_str[2..]);
        home
    } else {
        PathBuf::from(path_str)
    };

    if let Err(_) = env::set_current_dir(&target_path) {
        eprintln!("cd: {}: No such file or directory", path_str);
    }
}
