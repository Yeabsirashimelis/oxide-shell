#[allow(unused_imports)]
use std::io::{self, Write};

use codecrafters_shell::shell::repl;

fn main() {
    // TODO: Uncomment the code below to pass the first stage
    print!("$ ");
    io::stdout().flush().unwrap();

    let mut command = String::new();

    loop {
        let _ = io::stdin().read_line(&mut command).unwrap();
        repl(&command);
    }
}
