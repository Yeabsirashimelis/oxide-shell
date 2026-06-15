use std::collections::HashMap;
use std::env;
use std::process::{Command as ProcessCommand, Stdio};

use glob::glob;

use super::commands::{get_alias, ChainOperator, Command};
use crate::shell::commands::{map_external_commands, CommandType};

/// Expands environment variables and command substitutions in the input string.
/// - `$VAR` and `${VAR}` are expanded to their values
/// - `$(command)` is replaced with command output
/// - `` `command` `` is replaced with command output (backtick syntax)
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
                // Check if escaping a dollar sign or backtick
                if let Some(&next) = chars.peek() {
                    if (next == '$' || next == '`') && !in_single_quotes {
                        result.push(next);
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
                // Check for arithmetic expansion $((...)) or command substitution $(...)
                if chars.peek() == Some(&'(') {
                    chars.next(); // consume first '('
                    if chars.peek() == Some(&'(') {
                        // Arithmetic expansion $((...))
                        chars.next(); // consume second '('
                        let arith_result =
                            extract_and_eval_arithmetic(&mut chars, last_exit_code);
                        result.push_str(&arith_result);
                    } else {
                        let cmd_output =
                            extract_and_run_command_substitution(&mut chars, last_exit_code);
                        result.push_str(&cmd_output);
                    }
                } else {
                    // Variable expansion
                    let var_value = extract_and_expand_variable(&mut chars, last_exit_code);
                    result.push_str(&var_value);
                }
            }
            '`' if !in_single_quotes => {
                // Backtick command substitution
                let cmd_output = extract_and_run_backtick_substitution(&mut chars, last_exit_code);
                result.push_str(&cmd_output);
            }
            _ => result.push(c),
        }
    }

    result
}

/// Extracts command from $(...) and executes it, returning the output.
fn extract_and_run_command_substitution(
    chars: &mut std::iter::Peekable<std::str::Chars>,
    last_exit_code: i32,
) -> String {
    let mut command = String::new();
    let mut depth = 1;

    while let Some(c) = chars.next() {
        match c {
            '(' => {
                depth += 1;
                command.push(c);
            }
            ')' => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
                command.push(c);
            }
            _ => command.push(c),
        }
    }

    execute_command_substitution(&command, last_exit_code)
}

/// Extracts arithmetic expression from $((...)) and evaluates it.
fn extract_and_eval_arithmetic(
    chars: &mut std::iter::Peekable<std::str::Chars>,
    last_exit_code: i32,
) -> String {
    let mut expr = String::new();
    let mut paren_depth = 0; // extra parens inside the expression

    while let Some(c) = chars.next() {
        match c {
            '(' => {
                paren_depth += 1;
                expr.push(c);
            }
            ')' => {
                if paren_depth > 0 {
                    paren_depth -= 1;
                    expr.push(c);
                } else {
                    // This is one of the closing )) — check for the second
                    if chars.peek() == Some(&')') {
                        chars.next(); // consume second ')'
                    }
                    break;
                }
            }
            _ => expr.push(c),
        }
    }

    // Expand variables in the expression
    let expanded = expand_variables(&expr, last_exit_code);
    eval_arithmetic_expr(&expanded)
}

/// Evaluates a simple arithmetic expression.
/// Supports: +, -, *, /, % with integer arithmetic.
fn eval_arithmetic_expr(expr: &str) -> String {
    let expr = expr.trim();

    // Try to handle simple binary operations
    // We parse with operator precedence: first handle +/-, then *//%
    match eval_add_sub(expr) {
        Some(val) => val.to_string(),
        None => String::new(),
    }
}

fn eval_add_sub(expr: &str) -> Option<i64> {
    let expr = expr.trim();
    // Find the last + or - that's not inside parentheses
    let mut depth = 0;
    let mut last_op: Option<(usize, char)> = None;
    for (i, c) in expr.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            '+' | '-' if depth == 0 && i > 0 => {
                last_op = Some((i, c));
            }
            _ => {}
        }
    }
    if let Some((pos, op)) = last_op {
        let left = eval_add_sub(&expr[..pos])?;
        let right = eval_mul_div(&expr[pos + 1..])?;
        return Some(if op == '+' { left + right } else { left - right });
    }
    eval_mul_div(expr)
}

fn eval_mul_div(expr: &str) -> Option<i64> {
    let expr = expr.trim();
    let mut depth = 0;
    let mut last_op: Option<(usize, char)> = None;
    for (i, c) in expr.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            '*' | '/' | '%' if depth == 0 && i > 0 => {
                last_op = Some((i, c));
            }
            _ => {}
        }
    }
    if let Some((pos, op)) = last_op {
        let left = eval_mul_div(&expr[..pos])?;
        let right = eval_atom(expr[pos + 1..].trim())?;
        return match op {
            '*' => Some(left * right),
            '/' => {
                if right == 0 { None } else { Some(left / right) }
            }
            '%' => {
                if right == 0 { None } else { Some(left % right) }
            }
            _ => None,
        };
    }
    eval_atom(expr)
}

fn eval_atom(expr: &str) -> Option<i64> {
    let expr = expr.trim();
    if expr.is_empty() {
        return None;
    }
    // Handle parenthesized expression
    if expr.starts_with('(') && expr.ends_with(')') {
        return eval_add_sub(&expr[1..expr.len() - 1]);
    }
    // Handle unary minus
    if expr.starts_with('-') {
        return eval_atom(&expr[1..]).map(|v| -v);
    }
    // Handle unary plus
    if expr.starts_with('+') {
        return eval_atom(&expr[1..]);
    }
    // Try parsing as integer
    if let Ok(val) = expr.parse::<i64>() {
        return Some(val);
    }
    // Try as a variable name (bash allows bare names in arithmetic)
    if expr
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_')
    {
        let val = env::var(expr).unwrap_or_default();
        return val.parse::<i64>().ok().or(Some(0));
    }
    None
}

/// Extracts command from `...` and executes it, returning the output.
fn extract_and_run_backtick_substitution(
    chars: &mut std::iter::Peekable<std::str::Chars>,
    last_exit_code: i32,
) -> String {
    let mut command = String::new();

    while let Some(c) = chars.next() {
        match c {
            '`' => break,
            '\\' => {
                // In backticks, \` is an escaped backtick
                if chars.peek() == Some(&'`') {
                    command.push('`');
                    chars.next();
                } else {
                    command.push(c);
                }
            }
            _ => command.push(c),
        }
    }

    execute_command_substitution(&command, last_exit_code)
}

/// Executes a command and returns its stdout output.
fn execute_command_substitution(command: &str, last_exit_code: i32) -> String {
    let command = command.trim();
    if command.is_empty() {
        return String::new();
    }

    // Expand variables in the command first (recursive)
    let expanded_cmd = expand_variables(command, last_exit_code);

    // Try to execute as external command
    let output = if cfg!(windows) {
        // On Windows, try cmd.exe for built-in commands or direct execution
        ProcessCommand::new("cmd")
            .args(["/C", &expanded_cmd])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
    } else {
        // On Unix, use sh -c
        ProcessCommand::new("sh")
            .args(["-c", &expanded_cmd])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
    };

    match output {
        Ok(output) => {
            let mut stdout = String::from_utf8_lossy(&output.stdout).to_string();
            // Remove trailing newline (common shell behavior)
            if stdout.ends_with('\n') {
                stdout.pop();
                if stdout.ends_with('\r') {
                    stdout.pop();
                }
            }
            stdout
        }
        Err(_) => String::new(),
    }
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
    env::var("HOME").or_else(|_| env::var("USERPROFILE")).ok()
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

/// Expands brace patterns in a token.
/// - `{a,b,c}` expands to multiple tokens: `a`, `b`, `c`
/// - `file{1,2,3}.txt` expands to: `file1.txt`, `file2.txt`, `file3.txt`
/// - `{a,b}{1,2}` expands to: `a1`, `a2`, `b1`, `b2`
/// Returns a vector of expanded tokens.
fn expand_braces(token: &str) -> Vec<String> {
    // Find the first brace pair
    let mut depth = 0;
    let mut brace_start: Option<usize> = None;
    let mut brace_end: Option<usize> = None;

    for (i, c) in token.char_indices() {
        match c {
            '{' => {
                if depth == 0 {
                    brace_start = Some(i);
                }
                depth += 1;
            }
            '}' => {
                depth -= 1;
                if depth == 0 && brace_start.is_some() {
                    brace_end = Some(i);
                    break;
                }
            }
            _ => {}
        }
    }

    // If no valid brace pair found, return the token as-is
    let (start, end) = match (brace_start, brace_end) {
        (Some(s), Some(e)) => (s, e),
        _ => return vec![token.to_string()],
    };

    let prefix = &token[..start];
    let suffix = &token[end + 1..];
    let inner = &token[start + 1..end];

    // Check if it's a range pattern like {1..5} or {a..z}
    if let Some(expanded) = try_expand_range(inner) {
        let mut result = Vec::new();
        for item in expanded {
            let combined = format!("{}{}{}", prefix, item, suffix);
            // Recursively expand any remaining braces
            result.extend(expand_braces(&combined));
        }
        return result;
    }

    // Split by commas (respecting nested braces)
    let alternatives = split_brace_alternatives(inner);

    if alternatives.len() <= 1 && !inner.contains(',') {
        // No comma found, not a valid brace expansion
        return vec![token.to_string()];
    }

    let mut result = Vec::new();
    for alt in alternatives {
        let combined = format!("{}{}{}", prefix, alt, suffix);
        // Recursively expand any remaining braces
        result.extend(expand_braces(&combined));
    }

    result
}

/// Try to expand a range pattern like "1..5" or "a..e"
fn try_expand_range(inner: &str) -> Option<Vec<String>> {
    let parts: Vec<&str> = inner.split("..").collect();
    if parts.len() != 2 {
        return None;
    }

    let start = parts[0].trim();
    let end = parts[1].trim();

    // Try numeric range
    if let (Ok(start_num), Ok(end_num)) = (start.parse::<i32>(), end.parse::<i32>()) {
        let range: Vec<String> = if start_num <= end_num {
            (start_num..=end_num).map(|n| n.to_string()).collect()
        } else {
            (end_num..=start_num).rev().map(|n| n.to_string()).collect()
        };
        return Some(range);
    }

    // Try alphabetic range (single characters)
    if start.len() == 1 && end.len() == 1 {
        let start_char = start.chars().next().unwrap();
        let end_char = end.chars().next().unwrap();

        if start_char.is_ascii_alphabetic() && end_char.is_ascii_alphabetic() {
            let range: Vec<String> = if start_char <= end_char {
                (start_char..=end_char).map(|c| c.to_string()).collect()
            } else {
                (end_char..=start_char)
                    .rev()
                    .map(|c| c.to_string())
                    .collect()
            };
            return Some(range);
        }
    }

    None
}

/// Split brace content by commas, respecting nested braces
fn split_brace_alternatives(inner: &str) -> Vec<String> {
    let mut alternatives = Vec::new();
    let mut current = String::new();
    let mut depth = 0;

    for c in inner.chars() {
        match c {
            '{' => {
                depth += 1;
                current.push(c);
            }
            '}' => {
                depth -= 1;
                current.push(c);
            }
            ',' if depth == 0 => {
                alternatives.push(current.clone());
                current.clear();
            }
            _ => current.push(c),
        }
    }

    if !current.is_empty() || inner.ends_with(',') {
        alternatives.push(current);
    }

    alternatives
}

/// Expands tilde, braces, and glob patterns in tokens that weren't quoted.
/// - `~` expands to home directory
/// - `{a,b,c}` expands to multiple tokens
/// - `{1..5}` expands to numeric range
/// - `*` matches any characters (except path separator)
/// - `?` matches a single character
/// - `[abc]` matches any character in brackets
/// - `[a-z]` matches character ranges
/// Returns expanded tokens, preserving order.
fn expand_globs(tokens: Vec<(String, bool)>) -> Vec<String> {
    let mut result = Vec::new();

    for (token, was_quoted) in tokens {
        // Skip all expansion for quoted tokens
        if was_quoted {
            result.push(token);
            continue;
        }

        // 1. Expand tilde
        let token = expand_tilde(&token, false);

        // 2. Expand braces (can produce multiple tokens)
        let brace_expanded = if token.contains('{') && token.contains('}') {
            expand_braces(&token)
        } else {
            vec![token]
        };

        // 3. Expand globs for each brace-expanded token
        for token in brace_expanded {
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

/// Checks if input contains a here document (<<DELIMITER).
/// Returns Some((delimiter, is_quoted)) if found, None otherwise.
/// is_quoted indicates if the delimiter was quoted (no expansion).
pub fn detect_heredoc(input: &str) -> Option<(String, bool)> {
    let mut chars = input.chars().peekable();
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;

    while let Some(c) = chars.next() {
        match c {
            '\'' if !in_double_quotes => in_single_quotes = !in_single_quotes,
            '"' if !in_single_quotes => in_double_quotes = !in_double_quotes,
            '<' if !in_single_quotes && !in_double_quotes => {
                if chars.peek() == Some(&'<') {
                    chars.next(); // consume second <

                    // Skip whitespace
                    while chars.peek() == Some(&' ') || chars.peek() == Some(&'\t') {
                        chars.next();
                    }

                    // Check if delimiter is quoted
                    let mut delimiter = String::new();
                    let mut is_quoted = false;
                    let quote_char = chars.peek().copied();

                    if quote_char == Some('\'') || quote_char == Some('"') {
                        is_quoted = true;
                        chars.next(); // consume opening quote

                        // Read until closing quote
                        while let Some(ch) = chars.next() {
                            if ch == quote_char.unwrap() {
                                break;
                            }
                            delimiter.push(ch);
                        }
                    } else {
                        // Unquoted delimiter - read until whitespace or end
                        while let Some(&ch) = chars.peek() {
                            if ch.is_whitespace() {
                                break;
                            }
                            delimiter.push(ch);
                            chars.next();
                        }
                    }

                    if !delimiter.is_empty() {
                        return Some((delimiter, is_quoted));
                    }
                }
            }
            _ => {}
        }
    }

    None
}

/// Removes the heredoc part (<<DELIMITER) from the command, leaving the base command.
pub fn remove_heredoc_from_command(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;

    while let Some(c) = chars.next() {
        match c {
            '\'' if !in_double_quotes => {
                in_single_quotes = !in_single_quotes;
                result.push(c);
            }
            '"' if !in_single_quotes => {
                in_double_quotes = !in_double_quotes;
                result.push(c);
            }
            '<' if !in_single_quotes && !in_double_quotes => {
                if chars.peek() == Some(&'<') {
                    // Skip the heredoc part
                    chars.next(); // consume second <

                    // Skip whitespace
                    while chars.peek() == Some(&' ') || chars.peek() == Some(&'\t') {
                        chars.next();
                    }

                    // Skip the delimiter (quoted or unquoted)
                    let quote_char = chars.peek().copied();
                    if quote_char == Some('\'') || quote_char == Some('"') {
                        chars.next(); // consume opening quote
                        while let Some(ch) = chars.next() {
                            if ch == quote_char.unwrap() {
                                break;
                            }
                        }
                    } else {
                        // Skip unquoted delimiter
                        while let Some(&ch) = chars.peek() {
                            if ch.is_whitespace() {
                                break;
                            }
                            chars.next();
                        }
                    }
                } else {
                    result.push(c);
                }
            }
            _ => result.push(c),
        }
    }

    result.trim().to_string()
}

/// Expands aliases in the input.
/// Only the first word is checked for alias expansion.
/// Alias expansion is recursive but with a depth limit to prevent infinite loops.
fn expand_alias(input: &str, depth: usize) -> String {
    if depth > 10 {
        // Prevent infinite recursion
        return input.to_string();
    }

    let trimmed = input.trim();
    if trimmed.is_empty() {
        return input.to_string();
    }

    // Find the first word (command name)
    let first_word_end = trimmed
        .find(|c: char| c.is_whitespace())
        .unwrap_or(trimmed.len());
    let first_word = &trimmed[..first_word_end];
    let rest = &trimmed[first_word_end..];

    // Check if first word is an alias
    if let Some(expansion) = get_alias(first_word) {
        // Replace the command with its expansion
        let expanded = format!("{}{}", expansion, rest);
        // Recursively expand in case the alias expands to another alias
        expand_alias(&expanded, depth + 1)
    } else {
        input.to_string()
    }
}

/// Checks if input starts with a control flow keyword.
pub fn is_control_flow(input: &str) -> bool {
    let first_word = input.trim().split_whitespace().next().unwrap_or("");
    matches!(first_word, "if" | "for" | "while" | "until" | "case")
}

/// Checks if a control flow block is complete (all opening keywords have matching closing keywords).
pub fn is_control_flow_complete(input: &str) -> bool {
    let mut if_depth = 0i32;
    let mut for_while_depth = 0i32;
    let mut case_depth = 0i32;
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;

    // Extract all words, respecting quotes
    let mut words: Vec<String> = Vec::new();
    let mut current = String::new();

    for c in input.chars() {
        match c {
            '\'' if !in_double_quotes => {
                in_single_quotes = !in_single_quotes;
                current.push(c);
            }
            '"' if !in_single_quotes => {
                in_double_quotes = !in_double_quotes;
                current.push(c);
            }
            ' ' | '\t' | '\n' | ';' if !in_single_quotes && !in_double_quotes => {
                if !current.is_empty() {
                    words.push(current.clone());
                    current.clear();
                }
            }
            _ => current.push(c),
        }
    }
    if !current.is_empty() {
        words.push(current);
    }

    for word in &words {
        match word.as_str() {
            "if" => if_depth += 1,
            "fi" => if_depth -= 1,
            "for" | "while" | "until" => for_while_depth += 1,
            "done" => for_while_depth -= 1,
            "case" => case_depth += 1,
            "esac" => case_depth -= 1,
            _ => {}
        }
    }

    if_depth == 0 && for_while_depth == 0 && case_depth == 0
}

/// Splits input into statements by ; and \n, respecting quotes.
fn split_into_statements(input: &str) -> Vec<String> {
    let mut statements = Vec::new();
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
            ';' if !in_single_quotes && !in_double_quotes => {
                // Check for ;; (case separator)
                if chars.peek() == Some(&';') {
                    chars.next();
                    let trimmed = current.trim().to_string();
                    if !trimmed.is_empty() {
                        statements.push(trimmed);
                    }
                    statements.push(";;".to_string());
                    current.clear();
                } else {
                    let trimmed = current.trim().to_string();
                    if !trimmed.is_empty() {
                        statements.push(trimmed);
                    }
                    current.clear();
                }
            }
            '\n' if !in_single_quotes && !in_double_quotes => {
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() {
                    statements.push(trimmed);
                }
                current.clear();
            }
            _ => current.push(c),
        }
    }

    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        statements.push(trimmed);
    }

    statements
}

/// Parse a control flow construct from the input.
pub fn parse_control_flow(input: &str) -> Option<Command> {
    let trimmed = input.trim();
    let first_word = trimmed.split_whitespace().next().unwrap_or("");

    match first_word {
        "for" => parse_for(trimmed),
        "if" => parse_if(trimmed),
        "while" => parse_while_until(trimmed, false),
        "until" => parse_while_until(trimmed, true),
        "case" => parse_case(trimmed),
        _ => None,
    }
}

/// Parse: for VAR in WORDS; do BODY; done
fn parse_for(input: &str) -> Option<Command> {
    let statements = split_into_statements(input);

    // Flatten into a token stream of words and keywords
    let mut all_words: Vec<String> = Vec::new();
    for stmt in &statements {
        for word in stmt.split_whitespace() {
            all_words.push(word.to_string());
        }
    }

    // Expected: for VAR in WORD... do COMMAND... done
    if all_words.len() < 5 || all_words[0] != "for" {
        return None;
    }

    let var = all_words[1].clone();

    // Find "in"
    if all_words.get(2).map(|s| s.as_str()) != Some("in") {
        return None;
    }

    // Find "do"
    let do_pos = all_words.iter().position(|w| w == "do")?;
    // Expand the word list: variables, then braces, then globs
    let raw_words: Vec<String> = all_words[3..do_pos].to_vec();
    let expanded_words_str = expand_variables(&raw_words.join(" "), 0);
    let word_tokens: Vec<(String, bool)> = expanded_words_str
        .split_whitespace()
        .map(|s| (s.to_string(), false))
        .collect();
    let words = expand_globs(word_tokens);

    // Body is between do and done - reconstruct from statements
    let body = extract_body_between_keywords(&statements, "do", "done");

    Some(Command::For { var, words, body })
}

/// Parse: if COND; then BODY; [elif COND; then BODY;] [else BODY;] fi
fn parse_if(input: &str) -> Option<Command> {
    let statements = split_into_statements(input);
    let mut branches: Vec<(String, Vec<String>)> = Vec::new();
    let mut else_body: Option<Vec<String>> = None;

    // Skip "if" keyword from first statement
    if statements.is_empty() {
        return None;
    }

    // First condition: everything after "if" until "then"
    let mut state = "expect_condition"; // expect_condition, collect_body, expect_elif_else_fi
    let mut current_condition = String::new();
    let mut current_body: Vec<String> = Vec::new();
    let mut in_else = false;

    // Track nesting to handle inner if/for/while
    let mut nest_depth = 0;

    for stmt in &statements {
        let trimmed = stmt.trim();
        let first_word = trimmed.split_whitespace().next().unwrap_or("");

        match state {
            "expect_condition" => {
                if first_word == "if" || first_word == "elif" {
                    // Extract condition: everything after if/elif, removing trailing "then" if present
                    let rest = trimmed
                        .strip_prefix("if")
                        .or_else(|| trimmed.strip_prefix("elif"))
                        .unwrap_or("")
                        .trim();

                    if rest.ends_with("then") {
                        current_condition =
                            rest[..rest.len() - 4].trim().to_string();
                        state = "collect_body";
                    } else {
                        current_condition = rest.to_string();
                        state = "expect_then";
                    }
                }
            }
            "expect_then" => {
                if trimmed == "then" {
                    state = "collect_body";
                } else if first_word == "then" {
                    // "then BODY" on the same line
                    let rest = trimmed.strip_prefix("then").unwrap_or("").trim();
                    if !rest.is_empty() {
                        // Check if rest starts with nested control flow
                        let rest_first = rest.split_whitespace().next().unwrap_or("");
                        if matches!(rest_first, "if" | "for" | "while" | "until" | "case") {
                            nest_depth += 1;
                        }
                        current_body.push(rest.to_string());
                    }
                    state = "collect_body";
                } else {
                    // Condition continues
                    if !current_condition.is_empty() {
                        current_condition.push(' ');
                    }
                    current_condition.push_str(trimmed);
                    if trimmed.ends_with("then") {
                        current_condition = current_condition
                            [..current_condition.len() - 4]
                            .trim()
                            .to_string();
                        state = "collect_body";
                    }
                }
            }
            "collect_body" => {
                // Track nesting
                if matches!(first_word, "if" | "for" | "while" | "until" | "case") {
                    nest_depth += 1;
                }
                if matches!(first_word, "fi" | "done" | "esac") && nest_depth > 0 {
                    nest_depth -= 1;
                    // Append to last body entry (part of nested construct)
                    if let Some(last) = current_body.last_mut() {
                        last.push_str("; ");
                        last.push_str(trimmed);
                    } else {
                        current_body.push(trimmed.to_string());
                    }
                    continue;
                }

                if nest_depth > 0 {
                    // Append to last body entry (part of nested construct)
                    if let Some(last) = current_body.last_mut() {
                        last.push_str("; ");
                        last.push_str(trimmed);
                    } else {
                        current_body.push(trimmed.to_string());
                    }
                    continue;
                }

                if first_word == "elif" {
                    // Save current branch
                    if !in_else {
                        branches.push((current_condition.clone(), current_body.clone()));
                    }
                    current_body.clear();
                    current_condition.clear();

                    let rest = trimmed.strip_prefix("elif").unwrap_or("").trim();
                    if rest.ends_with("then") {
                        current_condition =
                            rest[..rest.len() - 4].trim().to_string();
                        state = "collect_body";
                    } else {
                        current_condition = rest.to_string();
                        state = "expect_then";
                    }
                } else if first_word == "else" {
                    // Save current branch
                    branches.push((current_condition.clone(), current_body.clone()));
                    current_body.clear();
                    in_else = true;
                    // If there's content after "else" on the same line
                    let rest = trimmed.strip_prefix("else").unwrap_or("").trim();
                    if !rest.is_empty() {
                        current_body.push(rest.to_string());
                    }
                } else if first_word == "fi" {
                    if in_else {
                        else_body = Some(current_body.clone());
                    } else {
                        branches.push((current_condition.clone(), current_body.clone()));
                    }
                    break;
                } else {
                    current_body.push(trimmed.to_string());
                }
            }
            _ => {}
        }
    }

    if branches.is_empty() {
        return None;
    }

    Some(Command::If {
        branches,
        else_body,
    })
}

/// Parse: while/until COND; do BODY; done
fn parse_while_until(input: &str, is_until: bool) -> Option<Command> {
    let statements = split_into_statements(input);
    let keyword = if is_until { "until" } else { "while" };

    let mut condition = String::new();
    let mut body: Vec<String> = Vec::new();
    let mut state = "expect_condition";
    let mut nest_depth = 0;

    for stmt in &statements {
        let trimmed = stmt.trim();
        let first_word = trimmed.split_whitespace().next().unwrap_or("");

        match state {
            "expect_condition" => {
                let rest = trimmed.strip_prefix(keyword).unwrap_or(trimmed).trim();
                if rest.ends_with("do") {
                    condition = rest[..rest.len() - 2].trim().to_string();
                    state = "collect_body";
                } else {
                    condition = rest.to_string();
                    state = "expect_do";
                }
            }
            "expect_do" => {
                if trimmed == "do" {
                    state = "collect_body";
                } else {
                    if !condition.is_empty() {
                        condition.push(' ');
                    }
                    condition.push_str(trimmed);
                }
            }
            "collect_body" => {
                if matches!(first_word, "if" | "for" | "while" | "until" | "case") {
                    nest_depth += 1;
                }
                if matches!(first_word, "fi" | "done" | "esac") && nest_depth > 0 {
                    nest_depth -= 1;
                    body.push(trimmed.to_string());
                    continue;
                }

                if first_word == "done" && nest_depth == 0 {
                    break;
                } else {
                    body.push(trimmed.to_string());
                }
            }
            _ => {}
        }
    }

    if condition.is_empty() {
        return None;
    }

    Some(Command::WhileUntil {
        is_until,
        condition,
        body,
    })
}

/// Parse: case WORD in PATTERN) BODY;; ... esac
fn parse_case(input: &str) -> Option<Command> {
    let statements = split_into_statements(input);
    let mut word = String::new();
    let mut arms: Vec<(Vec<String>, Vec<String>)> = Vec::new();
    let mut state = "expect_word";
    let mut current_patterns: Vec<String> = Vec::new();
    let mut current_body: Vec<String> = Vec::new();

    for stmt in &statements {
        let trimmed = stmt.trim();

        match state {
            "expect_word" => {
                // case WORD in [PATTERN)...]
                let rest = trimmed.strip_prefix("case").unwrap_or(trimmed).trim();

                // Find " in " to separate word from potential patterns
                if let Some(in_pos) = find_keyword_in(rest) {
                    word = rest[..in_pos].trim().to_string();
                    let after_in = rest[in_pos + 3..].trim(); // skip " in "

                    if after_in.is_empty() {
                        state = "expect_pattern";
                    } else {
                        // Pattern follows on same line
                        state = "expect_pattern";
                        // Process the pattern part
                        if after_in.contains(')') {
                            let paren_pos = after_in.find(')').unwrap();
                            let pattern_str = &after_in[..paren_pos];
                            current_patterns = pattern_str.split('|').map(|s| s.trim().to_string()).collect();
                            let body_rest = after_in[paren_pos + 1..].trim();
                            if !body_rest.is_empty() {
                                current_body.push(body_rest.to_string());
                            }
                            state = "collect_body";
                        }
                    }
                } else {
                    word = rest.to_string();
                    state = "expect_in";
                }
            }
            "expect_in" => {
                if trimmed == "in" {
                    state = "expect_pattern";
                }
            }
            "expect_pattern" => {
                if trimmed == "esac" {
                    break;
                }
                // Pattern ends with )
                if trimmed.ends_with(')') {
                    let pattern_str = &trimmed[..trimmed.len() - 1];
                    current_patterns = pattern_str.split('|').map(|s| s.trim().to_string()).collect();
                    state = "collect_body";
                } else if trimmed.contains(')') {
                    // Pattern and body on same line: pattern) body
                    let paren_pos = trimmed.find(')').unwrap();
                    let pattern_str = &trimmed[..paren_pos];
                    current_patterns = pattern_str.split('|').map(|s| s.trim().to_string()).collect();
                    let rest = trimmed[paren_pos + 1..].trim();
                    if !rest.is_empty() {
                        current_body.push(rest.to_string());
                    }
                    state = "collect_body";
                }
            }
            "collect_body" => {
                if trimmed == ";;" {
                    arms.push((current_patterns.clone(), current_body.clone()));
                    current_patterns.clear();
                    current_body.clear();
                    state = "expect_pattern";
                } else if trimmed == "esac" {
                    // Push final arm if there's content
                    if !current_patterns.is_empty() {
                        arms.push((current_patterns.clone(), current_body.clone()));
                    }
                    break;
                } else {
                    current_body.push(trimmed.to_string());
                }
            }
            _ => {}
        }
    }

    if word.is_empty() {
        return None;
    }

    Some(Command::Case { word, arms })
}

/// Find the position of the " in " keyword in a case statement.
/// Returns the byte position of " in " or None if not found.
fn find_keyword_in(input: &str) -> Option<usize> {
    // Look for " in " as a word boundary
    let mut search_from = 0;
    while let Some(pos) = input[search_from..].find(" in") {
        let abs_pos = search_from + pos;
        let after = abs_pos + 3;
        // Check it's followed by whitespace or end of string
        if after >= input.len() || input.as_bytes()[after] == b' ' || input.as_bytes()[after] == b'\t' {
            return Some(abs_pos);
        }
        search_from = abs_pos + 1;
    }
    None
}

/// Extract body commands between two keywords from a list of statements.
fn extract_body_between_keywords(
    statements: &[String],
    start_keyword: &str,
    end_keyword: &str,
) -> Vec<String> {
    let mut body = Vec::new();
    let mut found_start = false;
    let mut nest_depth = 0;

    for stmt in statements {
        let trimmed = stmt.trim();
        let first_word = trimmed.split_whitespace().next().unwrap_or("");

        if !found_start {
            if trimmed == start_keyword || first_word == start_keyword {
                found_start = true;
                // If there's content after "do" on the same line
                let rest = trimmed.strip_prefix(start_keyword).unwrap_or("").trim();
                if !rest.is_empty() {
                    body.push(rest.to_string());
                }
                continue;
            }
            continue;
        }

        // Track nesting
        if matches!(first_word, "if" | "for" | "while" | "until" | "case") {
            nest_depth += 1;
        }
        if matches!(first_word, "fi" | "done" | "esac") {
            if nest_depth > 0 {
                nest_depth -= 1;
            } else if first_word == end_keyword {
                break;
            }
        }

        body.push(trimmed.to_string());
    }

    body
}

pub fn parse_command_with_exit_code(input: &str, last_exit_code: i32) -> Option<Command> {
    if input.trim().is_empty() {
        return None;
    }

    // Expand aliases first (before variable expansion)
    let aliased = expand_alias(input, 0);

    // Check for control flow BEFORE variable expansion
    // (body commands like $i must be preserved as templates)
    if is_control_flow(&aliased) {
        if let Some(cf) = parse_control_flow(&aliased) {
            return Some(cf);
        }
    }

    // Expand environment variables
    let expanded = expand_variables(&aliased, last_exit_code);

    // Check for command chaining (&&, ||, ;)
    if let Some((commands, operators)) = split_chain(&expanded) {
        return Some(Command::Chain {
            commands,
            operators,
        });
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

    // Check for bare variable assignment: VAR=value
    if let Some(eq_pos) = cmd.find('=') {
        let name = &cmd[..eq_pos];
        if !name.is_empty()
            && name
                .chars()
                .next()
                .map_or(false, |c| c.is_alphabetic() || c == '_')
            && name.chars().all(|c| c.is_alphanumeric() || c == '_')
        {
            let value = &cmd[eq_pos + 1..];
            env::set_var(name, value);
            return Some(Command::Noop);
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
        "clear" => Some(Command::Clear),
        "history" => Some(Command::History),
        "alias" => Some(Command::Alias(args)),
        "unalias" => Some(Command::Unalias(args)),
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
