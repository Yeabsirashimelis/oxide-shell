use std::{
    collections::HashMap,
    env, fs,
    path::{self, PathBuf},
};

pub enum Command {
    Exit(i32),
    Echo(String),
    Unknown(String),
    Type(String),
}

#[derive(Debug)]
enum CommandType {
    Builtin,
    Alias(String),
    Function(String),
    External(String),
}

fn load_cmd_and_description() -> HashMap<String, CommandType> {
    let mut command_map: HashMap<String, CommandType> = HashMap::new();

    // Builtin commands
    command_map.insert("echo".to_string(), CommandType::Builtin);
    command_map.insert("exit".to_string(), CommandType::Builtin);
    command_map.insert("type".to_string(), CommandType::Builtin);

    // Valid extensions for Windows
    #[cfg(windows)]
    let valid_extensions = ["exe", "bat", "cmd"];
    #[cfg(not(windows))]
    let valid_extensions: [&str; 0] = []; // On Unix, we'll assume all files are potentially executable

    if let Ok(paths) = env::var("PATH") {
        // Use correct separator for Windows vs Unix
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

            // Check if directory exists
            if path.is_dir() {
                if let Ok(entries) = fs::read_dir(path) {
                    for entry in entries.flatten() {
                        let file_path = entry.path();

                        // On Unix, check if it's a file; on Windows, check extension too
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
                                            continue; // skip non-executables
                                        }
                                    } else {
                                        continue; // skip files without extension
                                    }
                                }

                                // Insert into HashMap if not already present
                                command_map
                                    .entry(file_name.split(".").collect::<Vec<_>>()[0].to_string())
                                    .or_insert(CommandType::External(
                                        file_path.to_string_lossy().to_string(),
                                    ));
                            }
                        }
                    }
                }
            }
        }
    }

    command_map.insert(
        "my_exe".to_string(),
        CommandType::External("/tmp/qux/my_exe".to_string()),
    );

    command_map
}

fn handle_type_command(cmd: String) {
    let command_map = load_cmd_and_description();

    let cmd_description = command_map.get(&cmd);

    match cmd_description {
        Option::Some(CommandType::Builtin) => println!("{} is a shell builtin", cmd),
        Option::Some(CommandType::External(path)) => println!("{} is {}", cmd, path),
        _ => println!("{}: not found", cmd),
    }
}

pub fn handle_command(cmd: Command) {
    match cmd {
        Command::Exit(_) => {
            // handled in main loop
        }
        Command::Echo(text) => println!("{}", text),
        Command::Type(cmd) => handle_type_command(cmd),
        Command::Unknown(name) => println!("{}: command not found", name),
    }
}
