use std::collections::HashMap;
use std::env;

use glob::glob;

use super::commands::{ChainOperator, Command};
use crate::shell::commands::{map_external_commands, CommandType};

/// Expands environment variables in the input string.
/// - `$VAR` and `${VAR}` are expanded to their values
/// - Single quotes prevent expansion: `'$VAR'` stays literal
/// - Double quotes allow expansion: `"$VAR"` is expanded
/// - `\$` is a literal dollar sign
/// - Undefined variables expand to empty string
pub fn expand_variables(input: &str, last_exit_code: i32) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;

    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                // Check if escaping a dollar sign
                if let Some(&next) = chars.peek() {
                    if next == '$' && !in_single_quotes {
                        result.push('$');
                        chars.next();
                        continue;
                    }
                }
                result.push(c);
            }
            '\'' if !in_double_quotes => {
                in_single_quotes = !in_single_quotes;
                result.push(c);
            }
            '"' if !in_single_quotes => {
                in_double_quotes = !in_double_quotes;
                result.push(c);
            }
            '$' if !in_single_quotes => {
                // Variable expansion
                let var_value = extract_and_expand_variable(&mut chars, last_exit_code);
                result.push_str(&var_value);
            }
            _ => result.push(c),
        }
    }

    result
}

/// Extracts variable name and returns its value.
fn extract_and_expand_variable(
    chars: &mut std::iter::Peekable<std::str::Chars>,
    last_exit_code: i32,
) -> String {
    // Check for ${VAR} syntax
    if chars.peek() == Some(&'{') {
        chars.next(); // consume '{'
        let mut var_name = String::new();
        while let Some(&c) = chars.peek() {
            if c == '}' {
                chars.next(); // consume '}'
                break;
            }
            var_name.push(c);
            chars.next();
        }
        return get_variable_value(&var_name, last_exit_code);
    }

    // Check for special variables
    if let Some(&c) = chars.peek() {
        // $? - last exit code
        if c == '?' {
            chars.next();
            return last_exit_code.to_string();
        }
        // $$ - process ID (optional, but common)
        if c == '$' {
            chars.next();
            return std::process::id().to_string();
        }
    }

    // Regular $VAR syntax - collect alphanumeric and underscore
    let mut var_name = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_alphanumeric() || c == '_' {
            var_name.push(c);
            chars.next();
        } else {
            break;
        }
    }

    if var_name.is_empty() {
        return "$".to_string(); // Lone $ stays as $
    }

    get_variable_value(&var_name, last_exit_code)
}

/// Gets the value of an environment variable.
/// Escapes backslashes so they survive the parsing stage.
fn get_variable_value(name: &str, _last_exit_code: i32) -> String {
    env::var(name).unwrap_or_default().replace('\\', "\\\\")
}

/// Gets the user's home directory.
fn get_home_dir() -> Option<String> {
    // Try HOME first (Unix), then USERPROFILE (Windows)
    env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .ok()
}

/// Expands tilde (~) at the start of a token to the home directory.
/// - `~` expands to home directory
/// - `~/path` expands to home directory + path
/// - Quoted tokens are not expanded
fn expand_tilde(token: &str, was_quoted: bool) -> String {
    // Don't expand quoted tokens
    if was_quoted {
        return token.to_string();
    }

    // Only expand if token starts with ~
    if !token.starts_with('~') {
        return token.to_string();
    }

    // Get home directory
    let home = match get_home_dir() {
        Some(h) => h,
        None => return token.to_string(), // No home dir, keep original
    };

    // Handle ~ alone or ~/path
    if token == "~" {
        home
    } else if token.starts_with("~/") || token.starts_with("~\\") {
        // Replace ~ with home directory
        format!("{}{}", home, &token[1..])
    } else {
        // ~username or other patterns - keep as-is for now
        token.to_string()
    }
}

/// Expands tilde and glob patterns in tokens that weren't quoted.
/// - `~` expands to home directory
/// - `*` matches any characters (except path separator)
/// - `?` matches a single character
/// - `[abc]` matches any character in brackets
/// - `[a-z]` matches character ranges
/// Returns expanded tokens, preserving order.
fn expand_globs(tokens: Vec<(String, bool)>) -> Vec<String> {
    let mut result = Vec::new();

    for (token, was_quoted) in tokens {
        // First, expand tilde (before glob expansion so ~/*.txt works)
        let token = expand_tilde(&token, was_quoted);

        // Skip glob expansion for quoted tokens
        if was_quoted {
            result.push(token);
            continue;
        }

        // Check if token contains glob characters
        if !token.contains('*') && !token.contains('?') && !token.contains('[') {
            result.push(token);
            continue;
        }

        // Try to expand the glob pattern
        match glob(&token) {
            Ok(paths) => {
                let mut matches: Vec<String> = paths
                    .filter_map(|p| p.ok())
                    .map(|p| p.to_string_lossy().to_string())
                    .collect();

                if matches.is_empty() {
                    // No matches - keep original (bash behavior)
                    result.push(token);
                } else {
                    // Sort alphabetically
                    matches.sort();
                    result.extend(matches);
                }
            }
            Err(_) => {
                // Invalid pattern - keep original
                result.push(token);
            }
        }
    }

    result
}

/// Splits input by pipe operator `|`, respecting quotes.
/// Returns None if there's no pipe, Some(segments) if there are pipes.
pub fn split_pipeline(input: &str) -> Option<Vec<String>> {
    let mut segments: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;

    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                current.push(c);
                if let Some(next) = chars.next() {
                    current.push(next);
                }
            }
            '\'' if !in_double_quotes => {
                in_single_quotes = !in_single_quotes;
                current.push(c);
            }
            '"' if !in_single_quotes => {
                in_double_quotes = !in_double_quotes;
                current.push(c);
            }
            '|' if !in_single_quotes && !in_double_quotes => {
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() {
                    segments.push(trimmed);
                }
                current.clear();
            }
            _ => current.push(c),
        }
    }

    // Don't forget the last segment
    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        segments.push(trimmed);
    }

    // Only return Some if we actually found pipes (more than one segment)
    if segments.len() > 1 {
        Some(segments)
    } else {
        None
    }
}

/// Splits input by chain operators (&&, ||, ;), respecting quotes and pipes.
/// Returns None if no chain operators found, Some((commands, operators)) otherwise.
/// Pipes have higher precedence than chain operators.
pub fn split_chain(input: &str) -> Option<(Vec<String>, Vec<ChainOperator>)> {
    let mut commands: Vec<String> = Vec::new();
    let mut operators: Vec<ChainOperator> = Vec::new();
    let mut current = String::new();
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;

    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                current.push(c);
                if let Some(next) = chars.next() {
                    current.push(next);
                }
            }
            '\'' if !in_double_quotes => {
                in_single_quotes = !in_single_quotes;
                current.push(c);
            }
            '"' if !in_single_quotes => {
                in_double_quotes = !in_double_quotes;
                current.push(c);
            }
            '&' if !in_single_quotes && !in_double_quotes => {
                // Check for &&
                if chars.peek() == Some(&'&') {
                    chars.next(); // consume second &
                    let trimmed = current.trim().to_string();
                    if !trimmed.is_empty() {
                        commands.push(trimmed);
                        operators.push(ChainOperator::And);
                    }
                    current.clear();
                } else {
                    // Single & (background job - not implemented, treat as literal)
                    current.push(c);
                }
            }
            '|' if !in_single_quotes && !in_double_quotes => {
                // Check for ||
                if chars.peek() == Some(&'|') {
                    chars.next(); // consume second |
                    let trimmed = current.trim().to_string();
                    if !trimmed.is_empty() {
                        commands.push(trimmed);
                        operators.push(ChainOperator::Or);
                    }
                    current.clear();
                } else {
                    // Single | is a pipe, keep it in current segment
                    current.push(c);
                }
            }
            ';' if !in_single_quotes && !in_double_quotes => {
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() {
                    commands.push(trimmed);
                    operators.push(ChainOperator::Sequence);
                }
                current.clear();
            }
            _ => current.push(c),
        }
    }

    // Don't forget the last segment
    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        commands.push(trimmed);
    }

    // Only return Some if we found chain operators
    if operators.is_empty() {
        None
    } else {
        Some((commands, operators))
    }
}

pub fn parse_command(input: &str) -> Option<Command> {
    parse_command_with_exit_code(input, 0)
}

pub fn parse_command_with_exit_code(input: &str, last_exit_code: i32) -> Option<Command> {
    if input.trim().is_empty() {
        return None;
    }

    // Expand environment variables first
    let expanded = expand_variables(input, last_exit_code);

    // Check for command chaining first (&&, ||, ;)
    if let Some((commands, operators)) = split_chain(&expanded) {
        return Some(Command::Chain { commands, operators });
    }

    // Check for pipeline
    if let Some(segments) = split_pipeline(&expanded) {
        return Some(Command::Pipeline(segments));
    }

    parse_single_command(&expanded)
}

/// Parses a single command (no pipeline detection).
/// Used internally and by the pipeline executor.
pub fn parse_single_command(input: &str) -> Option<Command> {
    if input.trim().is_empty() {
        return None;
    }

    let mut tokens_with_quote_info: Vec<(String, bool)> = Vec::new();
    let mut current = String::new();
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;
    let mut token_has_quoted_chars = false;

    let mut chars = input.trim().chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if let Some(next_char) = chars.next() {
                    if in_single_quotes {
                        current.push('\\'); // literal in single quotes
                        token_has_quoted_chars = true;
                    }
                    if in_double_quotes && !"\\\"$`".contains(next_char) {
                        current.push('\\');
                    }
                    if in_single_quotes || in_double_quotes {
                        token_has_quoted_chars = true;
                    }
                    current.push(next_char);
                }
            }
            '\'' if !in_double_quotes => {
                in_single_quotes = !in_single_quotes;
                continue;
            }
            '"' if !in_single_quotes => {
                in_double_quotes = !in_double_quotes;
                continue;
            }
            ' ' if !in_single_quotes && !in_double_quotes => {
                if !current.is_empty() {
                    tokens_with_quote_info.push((current.clone(), token_has_quoted_chars));
                    current.clear();
                    token_has_quoted_chars = false;
                }
            }
            _ => {
                if in_single_quotes || in_double_quotes {
                    token_has_quoted_chars = true;
                }
                current.push(c);
            }
        }
    }

    if !current.is_empty() {
        tokens_with_quote_info.push((current, token_has_quoted_chars));
    }

    if tokens_with_quote_info.is_empty() {
        return None;
    }

    // Expand glob patterns (only for unquoted tokens)
    let parts = expand_globs(tokens_with_quote_info);

    if parts.is_empty() {
        return None;
    }

    let (cmd, args_vec) = parts.split_first().unwrap();
    let args = args_vec.join(" ");

    let mut external_commands: HashMap<String, CommandType> = HashMap::new();
    map_external_commands(&mut external_commands);

    let mut cmd_to_check = String::from(cmd.as_str());

    #[cfg(windows)]
    {
        if !external_commands.contains_key(&cmd_to_check) {
            let cmd_with_exe = format!("{}.exe", cmd);
            if external_commands.contains_key(&cmd_with_exe) {
                cmd_to_check = cmd_with_exe.clone();
            }
        }
    }

    match cmd.as_str() {
        "exit" => {
            let code = args.parse::<i32>().unwrap_or(0);
            Some(Command::Exit(code))
        }
        "echo" => {
            // Check if the input contains redirection symbols
            if cmd == "echo"
                && args_vec.iter().any(|s| {
                    s == ">" || s == ">>" || s == "1>" || s == "1>>" || s == "2>" || s == "2>>"
                })
            {
                // Use custom echo for redirection
                Some(Command::Echo(args))
            } else if cmd == "echo" {
                // Plain echo, check if external exists
                if external_commands.contains_key(cmd) {
                    // Treat as external command
                    let full_args: Vec<String> = parts.iter().map(|s| s.to_string()).collect();
                    Some(Command::External(full_args))
                } else {
                    // Builtin echo
                    Some(Command::Echo(args))
                }
            } else {
                // Builtin echo
                Some(Command::Echo(args))
            }
        }
        "type" => Some(Command::Type(args)),
        "pwd" => Some(Command::PWD),
        "cd" => Some(Command::CD(args)),
        "export" => Some(Command::Export(args)),
        "unset" => Some(Command::Unset(args)),
        "cat" => {
            let args_vec: Vec<String> = parts.iter().map(|s| s.to_string()).collect();
            Some(Command::Cat(args_vec))
        }
        "ls" => Some(Command::Ls(args)),
        _ => {
            if external_commands.contains_key(&cmd_to_check) {
                let args_vec: Vec<String> = parts.iter().map(|s| s.to_string()).collect();
                Some(Command::External(args_vec))
            } else {
                Some(Command::Unknown(cmd.to_string()))
            }
        }
    }
}
