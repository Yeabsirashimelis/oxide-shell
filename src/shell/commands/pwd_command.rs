use std::env;
use std::io::Write;

/// PWD command with generic writer for pipeline support.
pub fn run_pwd_command_with_writer(writer: &mut dyn Write) {
    let path = env::current_dir();

    match path {
        Result::Ok(working_dir) => {
            let _ = writeln!(writer, "{}", working_dir.display());
        }
        Result::Err(error) => eprintln!("failed to get working directory, {}", error),
    }
}

pub fn run_pwd_command() {
    run_pwd_command_with_writer(&mut std::io::stdout());
}
