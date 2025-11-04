pub enum Command {
    Exit(i32),
    Echo(String),
    Unknown(String),
}

pub fn handle_command(cmd: Command) {
    match cmd {
        Command::Exit(_) => {
            // handled in main loop
        }
        Command::Echo(text) => println!("{}", text),
        Command::Unknown(name) => println!("{}: command not found", name),
    }
}
