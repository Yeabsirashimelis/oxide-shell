use std::env;

use super::handle_command_with_exit;
use crate::shell::parser::{expand_variables, parse_command_with_exit_code};

/// Execute a single command string and return its exit code.
fn run_command(cmd_str: &str, last_exit_code: i32) -> i32 {
    let cmd_str = cmd_str.trim();
    if cmd_str.is_empty() {
        return 0;
    }

    match parse_command_with_exit_code(cmd_str, last_exit_code) {
        Some(cmd) => handle_command_with_exit(cmd),
        None => 0,
    }
}

/// Execute a list of command strings sequentially.
fn run_body(commands: &[String], last_exit_code: i32) -> i32 {
    let mut exit_code = last_exit_code;
    for cmd_str in commands {
        exit_code = run_command(cmd_str, exit_code);
    }
    exit_code
}

/// Executes if/elif/else/fi
pub fn execute_if(
    branches: Vec<(String, Vec<String>)>,
    else_body: Option<Vec<String>>,
) -> i32 {
    let mut last_exit_code = 0;

    for (condition, body) in &branches {
        // Execute the condition
        let cond_result = run_command(condition, last_exit_code);

        if cond_result == 0 {
            // Condition succeeded, execute body
            return run_body(body, last_exit_code);
        }
        last_exit_code = cond_result;
    }

    // No condition matched, try else
    if let Some(else_cmds) = else_body {
        return run_body(&else_cmds, last_exit_code);
    }

    last_exit_code
}

/// Executes for var in words; do body; done
pub fn execute_for(var: &str, words: Vec<String>, body: Vec<String>) -> i32 {
    let mut last_exit_code = 0;

    for word in &words {
        // Set the loop variable
        env::set_var(var, word);

        // Execute body
        last_exit_code = run_body(&body, last_exit_code);
    }

    last_exit_code
}

/// Executes while/until condition; do body; done
pub fn execute_while_until(
    is_until: bool,
    condition: &str,
    body: Vec<String>,
) -> i32 {
    let mut last_exit_code = 0;
    let max_iterations = 10000; // Safety limit

    for _ in 0..max_iterations {
        let cond_result = run_command(condition, last_exit_code);

        let should_continue = if is_until {
            cond_result != 0 // until: continue while condition FAILS
        } else {
            cond_result == 0 // while: continue while condition SUCCEEDS
        };

        if !should_continue {
            break;
        }

        last_exit_code = run_body(&body, last_exit_code);
    }

    last_exit_code
}

/// Executes case word in pattern) body;; esac
pub fn execute_case(word: &str, arms: Vec<(Vec<String>, Vec<String>)>) -> i32 {
    let last_exit_code = 0;
    // Expand variables in the word (e.g., $val -> its value)
    let expanded_word = expand_variables(word, last_exit_code);

    for (patterns, body) in &arms {
        for pattern in patterns {
            if matches_pattern(&expanded_word, pattern) {
                return run_body(body, last_exit_code);
            }
        }
    }

    last_exit_code
}

/// Simple glob-style pattern matching for case statements.
fn matches_pattern(text: &str, pattern: &str) -> bool {
    let pattern = pattern.trim();

    // Wildcard matches everything
    if pattern == "*" {
        return true;
    }

    // Try glob-style matching
    match glob::Pattern::new(pattern) {
        Ok(p) => p.matches(text),
        Err(_) => text == pattern, // Fall back to exact match
    }
}
