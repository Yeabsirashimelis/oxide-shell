#[allow(unused_imports)]
use std::io::{self, Write};

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

        repl(&command);
        command.clear();
    }
}
