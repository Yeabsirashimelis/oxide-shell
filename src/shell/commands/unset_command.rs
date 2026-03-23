use std::env;

/// Handles the unset command.
/// Removes one or more environment variables.
pub fn run_unset_command(args: String) {
    let args = args.trim();

    if args.is_empty() {
        // Nothing to unset
        return;
    }

    // Unset each variable
    for var_name in args.split_whitespace() {
        if !is_valid_var_name(var_name) {
            eprintln!("unset: '{}': not a valid identifier", var_name);
            continue;
        }

        env::remove_var(var_name);
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
