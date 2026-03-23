use std::env;

/// Handles the export command.
/// - `export` with no args: list all environment variables
/// - `export VAR=value`: set environment variable
/// - `export VAR`: mark variable for export (just prints current value if set)
pub fn run_export_command(args: String) {
    let args = args.trim();

    // No arguments - list all environment variables
    if args.is_empty() {
        let mut vars: Vec<(String, String)> = env::vars().collect();
        vars.sort_by(|a, b| a.0.cmp(&b.0));
        for (key, value) in vars {
            println!("export {}=\"{}\"", key, value.replace('"', "\\\""));
        }
        return;
    }

    // Check if this is an assignment (contains =)
    if let Some(eq_pos) = args.find('=') {
        let name = &args[..eq_pos];
        let value = &args[eq_pos + 1..];

        // Validate variable name (should not contain spaces)
        let name = name.trim();
        if !is_valid_var_name(name) {
            eprintln!("export: '{}': not a valid identifier", name);
            return;
        }

        // Remove surrounding quotes if present
        let value = strip_quotes(value.trim());

        env::set_var(name, value);
    } else {
        // No '=' found - could be one or more variable names
        for var_name in args.split_whitespace() {
            if !is_valid_var_name(var_name) {
                eprintln!("export: '{}': not a valid identifier", var_name);
                continue;
            }

            if let Ok(value) = env::var(var_name) {
                println!("export {}=\"{}\"", var_name, value.replace('"', "\\\""));
            }
        }
    }
}

/// Checks if a variable name is valid (starts with letter or underscore,
/// followed by alphanumeric or underscore).
fn is_valid_var_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    let mut chars = name.chars();
    let first = chars.next().unwrap();

    if !first.is_alphabetic() && first != '_' {
        return false;
    }

    chars.all(|c| c.is_alphanumeric() || c == '_')
}

/// Strips surrounding quotes from a string.
fn strip_quotes(s: &str) -> &str {
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        if s.len() >= 2 {
            return &s[1..s.len() - 1];
        }
    }
    s
}
