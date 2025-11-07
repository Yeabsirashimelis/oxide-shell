use crate::shell::commands::{load_cmd_and_description, CommandType};

pub fn run_type_command(cmd: String) {
    let command_map = load_cmd_and_description();

    let cmd_description = command_map.get(&cmd);

    match cmd_description {
        Option::Some(CommandType::Builtin) => println!("{} is a shell builtin", cmd),
        Option::Some(CommandType::External(path)) => println!("{} is {}", cmd, path),
        _ => eprintln!("{}: not found", cmd),
    }
}
