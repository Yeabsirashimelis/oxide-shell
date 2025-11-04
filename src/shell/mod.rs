use std::io;

pub fn repl(input_command: &str) {
    println!("{}: command not found", input_command.trim());
}
