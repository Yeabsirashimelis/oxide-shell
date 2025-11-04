#[allow(unused_imports)]
use std::io::{self, Write};
use std::process;

use codecrafters_shell::shell::repl;

fn main() {
    // TODO: Uncomment the code below to pass the first stage

    let mut command = String::new();

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let _ = io::stdin().read_line(&mut command).unwrap();

        if command.clone().is_empty() {
            continue;
        }
        if &command.trim() == &"exit 0" {
            process::exit(0);
        }

        if &command.trim() == &"exit 1" {
            process::exit(1);
        }
        repl(&command);
        command.clear();
    }
}
