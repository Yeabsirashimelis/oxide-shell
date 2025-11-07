use std::env;

pub fn run_pwd_command() {
    let path = env::current_dir();

    match path {
        Result::Ok(working_dir) => println!("{}", working_dir.display()),
        Result::Err(error) => eprintln!("failed to get working directory, {}", error),
    }
}
