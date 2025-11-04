use super::commands::Command;

pub fn parse_command(input: &str) -> Option<Command> {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    let (cmd, args) = parts.split_first().unwrap();
    let args = args.join(" ");

    match *cmd {
        "exit" => {
            let code = args.parse::<i32>().unwrap_or(0);
            Some(Command::Exit(code))
        }
        "echo" => Some(Command::Echo(args)),
        _ => Some(Command::Unknown(cmd.to_string())),
    }
}
