use crate::shell::commands::cat_command::open_file;
use std::io::Write;
use std::path::Path;
use std::process::{Child, Command as SysCommand, Stdio};

/// External command with configurable stdin/stdout for pipeline support.
/// Returns the child process for the caller to wait on.
#[allow(dead_code)]
pub fn run_external_command_with_io(
    args: Vec<String>,
    stdin: Stdio,
    stdout: Stdio,
) -> std::io::Result<Child> {
    if args.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "No command provided",
        ));
    }

    let mut cmd_name = args[0].clone();

    #[cfg(windows)]
    {
        if !cmd_name.ends_with(".exe") {
            cmd_name = format!("{}.exe", cmd_name);
        }
    }

    let final_args: Vec<String> = args.into_iter().skip(1).collect();

    let mut command = SysCommand::new(&cmd_name);
    if !final_args.is_empty() {
        command.args(&final_args);
    }

    command.stdin(stdin).stdout(stdout).spawn()
}

pub fn run_external_command(args: Vec<String>) -> i32 {
    if args.is_empty() {
        eprintln!("Error: no command provided");
        return 1;
    }

    let mut args_iter = args.into_iter();
    let mut cmd_name = args_iter.next().unwrap();

    #[cfg(windows)]
    {
        if !cmd_name.ends_with(".exe") {
            cmd_name = format!("{}.exe", cmd_name);
        }
    }

    let mut stdout_path: Option<(String, bool)> = None; // (path, append)
    let mut stderr_path: Option<(String, bool)> = None;
    let mut final_args: Vec<String> = Vec::new();

    // --- Parse arguments for redirection ---
    while let Some(arg) = args_iter.next() {
        match arg.as_str() {
            ">" | "1>" => {
                if let Some(path) = args_iter.next() {
                    stdout_path = Some((path, false));
                }
            }
            ">>" | "1>>" => {
                if let Some(path) = args_iter.next() {
                    stdout_path = Some((path, true));
                }
            }
            "2>" => {
                if let Some(path) = args_iter.next() {
                    stderr_path = Some((path, false));
                }
            }
            "2>>" => {
                if let Some(path) = args_iter.next() {
                    stderr_path = Some((path, true));
                }
            }
            _ => final_args.push(arg),
        }
    }

    let mut command = SysCommand::new(&cmd_name);
    if !final_args.is_empty() {
        command.args(&final_args);
    }

    // --- Redirect stdout ---
    if let Some((ref path, append)) = stdout_path {
        if let Ok(file) = open_file(Path::new(path), append) {
            command.stdout(Stdio::from(file));
        }
    }

    // --- Redirect stderr ---
    if let Some((ref path, append)) = stderr_path {
        if let Ok(file) = open_file(Path::new(path), append) {
            command.stderr(Stdio::from(file));
        }
    }

    // --- Spawn process ---
    match command.spawn() {
        Ok(mut child) => match child.wait() {
            Ok(status) => status.code().unwrap_or(1),
            Err(e) => {
                if stderr_path.is_none() {
                    eprintln!("Error waiting for process: {}", e);
                }
                1
            }
        },
        Err(e) => {
            let msg = format!("Failed to execute '{}': {}\n", cmd_name, e);
            if let Some((ref path, append)) = stderr_path {
                if let Ok(mut f) = open_file(Path::new(path), append) {
                    let _ = f.write_all(msg.as_bytes());
                    let _ = f.flush();
                }
            } else {
                eprint!("{}", msg);
            }
            127
        }
    }
}
