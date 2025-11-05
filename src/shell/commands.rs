use std::{collections::HashMap, env, fs, path::PathBuf};

pub enum Command {
    Exit(i32),
    Echo(String),
    Unknown(String),
    Type(String),
    External(Vec<String>),
}

#[derive(Debug)]
pub enum CommandType {
    Builtin,
    Alias(String),
    Function(String),
    External(String),
}

fn map_builtin_commands(command_map: &mut HashMap<String, CommandType>) {
    //built in commands
    command_map.insert("echo".to_string(), CommandType::Builtin);
    command_map.insert("exit".to_string(), CommandType::Builtin);
    command_map.insert("type".to_string(), CommandType::Builtin);
}

pub fn map_external_commands(command_map: &mut HashMap<String, CommandType>) {
    // external commands
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
        command_map.insert(
            "my_exe".to_string(),
            CommandType::External("/tmp/bar/my_exe".to_string()),
        );
    } else {
        eprintln!("Warning: PATH enviroment variable not found");
    }
}

fn load_cmd_and_description() -> HashMap<String, CommandType> {
    let mut command_map: HashMap<String, CommandType> = HashMap::new();

    map_builtin_commands(&mut command_map);
    map_external_commands(&mut command_map);
    command_map
}

fn run_type_command(cmd: String) {
    let command_map = load_cmd_and_description();

    let cmd_description = command_map.get(&cmd);

    match cmd_description {
        Option::Some(CommandType::Builtin) => println!("{} is a shell builtin", cmd),
        Option::Some(CommandType::External(path)) => println!("{} is {}", cmd, path),
        _ => eprintln!("{}: not found", cmd),
    }
}

fn run_external_command(args: Vec<String>) {
    if args.is_empty() {
        eprintln!("Error: no command provided");
        return;
    }
    let (cmd, args) = args.split_first().unwrap();

    let process = std::process::Command::new(cmd).args(args).spawn();

    match process {
        Result::Ok(mut process) => {
            // wait the command to finish
            if let Result::Err(error) = process.wait() {
                eprintln!("Error waiting for process: {}", error)
            }
        }
        Result::Err(error) => {
            eprintln!("Failed to execute '{}': {}", cmd, error)
        }
    }

    return;
}

pub fn handle_command(cmd: Command) {
    match cmd {
        Command::Exit(_) => {
            // handled in main loopp
        }
        Command::Echo(text) => println!("{}", text),
        Command::Type(cmd) => run_type_command(cmd),
        Command::External(args) => run_external_command(args),
        Command::Unknown(name) => println!("{}: command not found", name),
    }
}
