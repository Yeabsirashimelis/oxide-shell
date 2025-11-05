use std::collections::HashMap;

pub enum Command {
    Exit(i32),
    Echo(String),
    Unknown(String),
    Type(String),
}

enum CommandType {
    Builtin,
    Alias(String),
    Function(String),
    External(String),
}

fn load_cmd_and_description() -> HashMap<String, CommandType> {
    let mut command_map: HashMap<String, CommandType> = HashMap::new();

    command_map.insert("echo".to_string(), CommandType::Builtin);
    command_map.insert("exit".to_string(), CommandType::Builtin);
    command_map.insert("type".to_string(), CommandType::Builtin);

    command_map
}

fn handle_type_command(cmd: String) {
    let command_map = load_cmd_and_description();

    let cmd_description = command_map.get(&cmd);

    match cmd_description {
        Option::Some(CommandType::Builtin) => println!("{} is a shell builtin", cmd),
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
