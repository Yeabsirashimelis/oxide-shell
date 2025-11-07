pub fn run_external_command(args: Vec<String>) {
    if args.is_empty() {
        eprintln!("Error: no command provided");
        return;
    }
    let (cmd, args) = args.split_first().unwrap();

    let process = std::process::Command::new(cmd).args(args).spawn();

    match process {
        Result::Ok(mut process) => {
            // wait the command to finish
            if let Result::Err(error) = process.wait() {
                eprintln!("Error waiting for process: {}", error)
            }
        }
        Result::Err(error) => {
            eprintln!("Failed to execute '{}': {}", cmd, error)
        }
    }

    return;
}
