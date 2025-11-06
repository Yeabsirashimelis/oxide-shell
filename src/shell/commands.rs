use std::{collections::HashMap, env, fs, path::PathBuf};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

pub enum Command {
    Exit(i32),
    Echo(String),
    Unknown(String),
    Type(String),
    PWD,
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

                                // ✅ Check if executable before inserting
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

                                // ✅ Now insert — first executable wins
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

fn run_pwd_command() {
    let path = env::current_dir();

    match path {
        Result::Ok(working_dir) => println!("{}", working_dir.display()),
        Result::Err(error) => eprintln!("failed to get working directory, {}", error),
    }
}

pub fn handle_command(cmd: Command) {
    match cmd {
        Command::Exit(_) => {
            // handled in main loop
        }
        Command::Echo(text) => println!("{}", text),
        Command::Type(cmd) => run_type_command(cmd),
        Command::PWD => run_pwd_command(),
        Command::External(args) => run_external_command(args),
        Command::Unknown(name) => println!("{}: command not found", name),
    }
}
