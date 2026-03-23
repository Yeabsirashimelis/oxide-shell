use std::io::{Read, Write};
use std::process::{Command as SysCommand, Stdio};
use std::thread;

use super::{
    cat_command::run_cat_command_with_io,
    echo_command::run_echo_command_with_writer,
    ls_command::run_ls_command_with_writer,
    pwd_command::run_pwd_command_with_writer,
    type_command::run_type_command_with_writer,
    Command,
};
use crate::shell::parser::parse_single_command;

/// Executes a pipeline of commands connected by pipes.
pub fn execute_pipeline(segments: Vec<String>) {
    if segments.is_empty() {
        return;
    }

    if segments.len() == 1 {
        // Single command, execute normally
        if let Some(cmd) = parse_single_command(&segments[0]) {
            super::handle_command(cmd);
        }
        return;
    }

    // Create pipes for each connection between commands
    let num_pipes = segments.len() - 1;
    let mut pipes: Vec<(os_pipe::PipeReader, os_pipe::PipeWriter)> = Vec::with_capacity(num_pipes);

    for _ in 0..num_pipes {
        match os_pipe::pipe() {
            Ok(pipe) => pipes.push(pipe),
            Err(e) => {
                eprintln!("Failed to create pipe: {}", e);
                return;
            }
        }
    }

    let mut handles: Vec<thread::JoinHandle<()>> = Vec::new();
    let mut children: Vec<std::process::Child> = Vec::new();

    for (i, segment) in segments.iter().enumerate() {
        let is_first = i == 0;
        let is_last = i == segments.len() - 1;

        let cmd = match parse_single_command(segment) {
            Some(c) => c,
            None => {
                eprintln!("Invalid command in pipeline: {}", segment);
                continue;
            }
        };

        match cmd {
            Command::Exit(_) => {
                eprintln!("exit cannot be used in a pipeline");
                return;
            }
            Command::CD(_) => {
                eprintln!("cd cannot be used in a pipeline");
                return;
            }
            Command::Unknown(name) => {
                eprintln!("{}: command not found", name);
                return;
            }
            Command::Pipeline(_) => {
                eprintln!("Nested pipelines are not supported");
                return;
            }
            Command::External(args) => {
                // External command - use process with piped I/O
                let stdin_cfg = if is_first {
                    Stdio::inherit()
                } else {
                    let reader = std::mem::replace(
                        &mut pipes[i - 1].0,
                        os_pipe::pipe().unwrap().0,
                    );
                    Stdio::from(reader)
                };

                let stdout_cfg = if is_last {
                    Stdio::inherit()
                } else {
                    let writer = std::mem::replace(
                        &mut pipes[i].1,
                        os_pipe::pipe().unwrap().1,
                    );
                    Stdio::from(writer)
                };

                match spawn_external_command(args, stdin_cfg, stdout_cfg) {
                    Ok(child) => children.push(child),
                    Err(e) => eprintln!("Failed to spawn command: {}", e),
                }
            }
            // Built-in commands - run in threads
            builtin_cmd => {
                // Get stdin reader for this command (if not first)
                let stdin_reader: Option<os_pipe::PipeReader> = if is_first {
                    None
                } else {
                    Some(std::mem::replace(
                        &mut pipes[i - 1].0,
                        os_pipe::pipe().unwrap().0,
                    ))
                };

                // Get stdout writer for this command (if not last)
                let stdout_writer: Option<os_pipe::PipeWriter> = if is_last {
                    None
                } else {
                    Some(std::mem::replace(
                        &mut pipes[i].1,
                        os_pipe::pipe().unwrap().1,
                    ))
                };

                let handle = thread::spawn(move || {
                    execute_builtin_with_io(builtin_cmd, stdin_reader, stdout_writer);
                });
                handles.push(handle);
            }
        }
    }

    // Wait for all external processes
    for mut child in children {
        let _ = child.wait();
    }

    // Wait for all builtin threads
    for handle in handles {
        let _ = handle.join();
    }
}

/// Spawns an external command with configured stdin/stdout.
fn spawn_external_command(
    args: Vec<String>,
    stdin: Stdio,
    stdout: Stdio,
) -> std::io::Result<std::process::Child> {
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

    let mut command = SysCommand::new(&cmd_name);
    if args.len() > 1 {
        command.args(&args[1..]);
    }

    command.stdin(stdin).stdout(stdout).spawn()
}

/// Executes a built-in command with optional stdin reader and stdout writer.
fn execute_builtin_with_io(
    cmd: Command,
    stdin_reader: Option<os_pipe::PipeReader>,
    stdout_writer: Option<os_pipe::PipeWriter>,
) {
    // Create a boxed writer - either the pipe or stdout
    let mut writer: Box<dyn Write> = match stdout_writer {
        Some(w) => Box::new(w),
        None => Box::new(std::io::stdout()),
    };

    match cmd {
        Command::Echo(text) => {
            run_echo_command_with_writer(text, &mut writer);
        }
        Command::PWD => {
            run_pwd_command_with_writer(&mut writer);
        }
        Command::Type(cmd_name) => {
            run_type_command_with_writer(cmd_name, &mut writer);
        }
        Command::Ls(path) => {
            run_ls_command_with_writer(&path, &mut writer);
        }
        Command::Cat(args) => {
            let reader: Option<Box<dyn Read>> = stdin_reader.map(|r| Box::new(r) as Box<dyn Read>);
            run_cat_command_with_io(args, reader, &mut writer);
        }
        _ => {
            // Other commands shouldn't reach here
        }
    }

    // Flush to ensure all output is written before pipe closes
    let _ = writer.flush();
}
