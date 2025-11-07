mod cd_command;
mod external_command;
mod map_commands;
mod pwd_command;
mod type_command;

use std::collections::HashMap;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

pub use crate::shell::commands::map_commands::{map_builtin_commands, map_external_commands};
use crate::shell::commands::{
    cd_command::run_cd_command, external_command::run_external_command,
    pwd_command::run_pwd_command, type_command::run_type_command,
};

pub enum Command {
    Exit(i32),
    Echo(String),
    Unknown(String),
    Type(String),
    PWD,
    CD(String),
    External(Vec<String>),
}

#[derive(Debug)]
pub enum CommandType {
    Builtin,
    Alias(String),
    Function(String),
    External(String),
}

fn load_cmd_and_description() -> HashMap<String, CommandType> {
    let mut command_map: HashMap<String, CommandType> = HashMap::new();

    map_builtin_commands(&mut command_map);
    map_external_commands(&mut command_map);
    command_map
}

pub fn handle_command(cmd: Command) {
    match cmd {
        Command::Exit(_) => {
            // handled in main loop
        }
        Command::Echo(text) => println!("{}", text),
        Command::Type(cmd) => run_type_command(cmd),
        Command::PWD => run_pwd_command(),
        Command::CD(path) => run_cd_command(&path),
        Command::External(args) => run_external_command(args),
        Command::Unknown(name) => println!("{}: command not found", name),
    }
}
