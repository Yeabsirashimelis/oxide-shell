mod cat_command;
mod cd_command;
mod chain;
mod echo_command;
mod export_command;
mod external_command;
pub mod ls_command;
pub mod map_commands;
pub mod pipeline;
pub mod pwd_command;
mod type_command;
mod unset_command;

use std::collections::HashMap;

pub use crate::shell::commands::map_commands::{map_builtin_commands, map_external_commands};
use crate::shell::commands::{
    cat_command::run_cat_command, cd_command::run_cd_command, chain::execute_chain,
    echo_command::run_echo_command, export_command::run_export_command,
    external_command::run_external_command, ls_command::run_ls_command,
    pipeline::execute_pipeline, pwd_command::run_pwd_command, type_command::run_type_command,
    unset_command::run_unset_command,
};

/// Operators for command chaining
#[derive(Debug, Clone, PartialEq)]
pub enum ChainOperator {
    And,      // && - run next only if previous succeeds
    Or,       // || - run next only if previous fails
    Sequence, // ;  - run next regardless of previous result
}

pub enum Command {
    Exit(i32),
    Echo(String),
    Unknown(String),
    Type(String),
    PWD,
    CD(String),
    Cat(Vec<String>),
    Ls(String),
    External(Vec<String>),
    Pipeline(Vec<String>),
    Export(String),
    Unset(String),
    /// Chained commands with operators between them
    Chain {
        commands: Vec<String>,
        operators: Vec<ChainOperator>,
    },
    Clear,
    History,
}

#[derive(Debug)]
#[allow(dead_code)]
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

/// Handles a command and returns its exit code.
pub fn handle_command_with_exit(cmd: Command) -> i32 {
    match cmd {
        Command::Exit(code) => code,
        Command::Echo(text) => {
            run_echo_command(text);
            0
        }
        Command::Type(cmd) => {
            run_type_command(cmd);
            0
        }
        Command::PWD => {
            run_pwd_command();
            0
        }
        Command::CD(path) => run_cd_command(&path),
        Command::Cat(paths) => run_cat_command(paths),
        Command::Ls(path) => {
            run_ls_command(&path);
            0
        }
        Command::External(args) => run_external_command(args),
        Command::Pipeline(segments) => {
            execute_pipeline(segments);
            0 // TODO: return last command's exit code
        }
        Command::Export(args) => {
            run_export_command(args);
            0
        }
        Command::Unset(args) => {
            run_unset_command(args);
            0
        }
        Command::Chain { commands, operators } => execute_chain(commands, operators),
        Command::Clear => {
            // ANSI escape: clear screen and move cursor to home
            print!("\x1b[2J\x1b[H");
            0
        }
        Command::History => {
            run_history_command();
            0
        }
        Command::Unknown(name) => {
            eprintln!("{}: command not found", name);
            127
        }
    }
}

/// Displays command history from history.txt
fn run_history_command() {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let history_path = "history.txt";
    match File::open(history_path) {
        Ok(file) => {
            let reader = BufReader::new(file);
            for (i, line) in reader.lines().enumerate() {
                if let Ok(cmd) = line {
                    println!("{:5}  {}", i + 1, cmd);
                }
            }
        }
        Err(_) => {
            println!("No history available");
        }
    }
}

/// Handles a command (legacy wrapper, doesn't return exit code).
pub fn handle_command(cmd: Command) {
    handle_command_with_exit(cmd);
}
