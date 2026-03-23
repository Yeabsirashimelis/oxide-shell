use super::{handle_command_with_exit, ChainOperator, Command};
use crate::shell::parser::{parse_single_command, split_pipeline};

/// Executes a chain of commands connected by &&, ||, or ;
/// Returns the exit code of the last executed command.
pub fn execute_chain(commands: Vec<String>, operators: Vec<ChainOperator>) -> i32 {
    if commands.is_empty() {
        return 0;
    }

    let mut last_exit_code = 0;

    for (i, cmd_str) in commands.iter().enumerate() {
        // Check if this segment contains a pipeline
        let exit_code = if let Some(segments) = split_pipeline(cmd_str) {
            // Execute as pipeline
            super::pipeline::execute_pipeline(segments);
            0 // Pipelines don't return exit codes yet
        } else {
            // Parse and execute single command
            match parse_single_command(cmd_str) {
                Some(cmd) => execute_single_for_chain(cmd),
                None => {
                    eprintln!("Invalid command: {}", cmd_str);
                    1
                }
            }
        };

        last_exit_code = exit_code;

        // Check if we should continue to next command
        if i < operators.len() {
            let should_continue = match &operators[i] {
                ChainOperator::And => exit_code == 0,      // Continue only if success
                ChainOperator::Or => exit_code != 0,       // Continue only if failure
                ChainOperator::Sequence => true,           // Always continue
            };

            if !should_continue {
                // Skip remaining commands until we find a different operator
                // For && after failure, skip until || or ;
                // For || after success, skip until && or ;
                let mut skip_to = i + 1;
                while skip_to < operators.len() {
                    let next_op = &operators[skip_to];
                    match (&operators[i], next_op) {
                        (ChainOperator::And, ChainOperator::Or) => break,
                        (ChainOperator::And, ChainOperator::Sequence) => break,
                        (ChainOperator::Or, ChainOperator::And) => break,
                        (ChainOperator::Or, ChainOperator::Sequence) => break,
                        _ => skip_to += 1,
                    }
                }
                // For simplicity, just stop execution here
                // A more complete implementation would handle complex chains like: a && b || c
                break;
            }
        }
    }

    last_exit_code
}

/// Execute a single command and return its exit code.
fn execute_single_for_chain(cmd: Command) -> i32 {
    match cmd {
        Command::Exit(code) => {
            // Don't actually exit, just return the code
            code
        }
        Command::Pipeline(segments) => {
            // Execute pipeline and return 0 (pipelines don't track exit codes yet)
            super::pipeline::execute_pipeline(segments);
            0
        }
        Command::Chain { commands, operators } => {
            // Nested chain - execute recursively
            execute_chain(commands, operators)
        }
        other => handle_command_with_exit(other),
    }
}
