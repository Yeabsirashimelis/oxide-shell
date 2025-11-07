use std::{collections::HashMap, env, fs, path::PathBuf};

use crate::shell::commands::CommandType;

pub fn map_builtin_commands(command_map: &mut HashMap<String, CommandType>) {
    //built in commands
    command_map.insert("echo".to_string(), CommandType::Builtin);
    command_map.insert("exit".to_string(), CommandType::Builtin);
    command_map.insert("type".to_string(), CommandType::Builtin);
    command_map.insert("pwd".to_string(), CommandType::Builtin);
    command_map.insert("cd".to_string(), CommandType::Builtin);
}

pub fn map_external_commands(command_map: &mut HashMap<String, CommandType>) {
    #[cfg(windows)]
    let valid_extensions = ["exe", "bat", "cmd"];
    #[cfg(not(windows))]
    let valid_extensions: [&str; 0] = [];

    if let Ok(paths) = env::var("PATH") {
        #[cfg(windows)]
        let separator = ';';
        #[cfg(not(windows))]
        let separator = ':';

        for dir in paths.split(separator) {
            let dir = dir.trim();
            if dir.is_empty() {
                continue;
            }

            let path = PathBuf::from(dir);
            if path.is_dir() {
                if let Ok(entries) = fs::read_dir(path) {
                    for entry in entries.flatten() {
                        let file_path = entry.path();

                        if file_path.is_file() {
                            if let Some(file_name) = file_path.file_name().and_then(|n| n.to_str())
                            {
                                #[cfg(windows)]
                                {
                                    if let Some(ext) =
                                        file_path.extension().and_then(|e| e.to_str())
                                    {
                                        if !valid_extensions.contains(&ext.to_lowercase().as_str())
                                        {
                                            continue;
                                        }
                                    } else {
                                        continue;
                                    }
                                }

                                //Check if executable before inserting
                                #[cfg(unix)]
                                {
                                    if let Ok(metadata) = fs::metadata(&file_path) {
                                        let mode = metadata.permissions().mode();
                                        let is_executable = mode & 0o111 != 0;
                                        if !is_executable {
                                            continue; // skip non-executable
                                        }
                                    } else {
                                        continue;
                                    }
                                }

                                //Now insert — first executable wins
                                command_map.entry(file_name.to_string()).or_insert(
                                    CommandType::External(file_path.to_string_lossy().to_string()),
                                );
                            }
                        }
                    }
                }
            }
        }
    } else {
        eprintln!("Warning: PATH environment variable not found");
    }
}
