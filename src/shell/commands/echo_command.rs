pub fn run_echo_command(text: String) {
    let mut cleaned = text.trim().to_string();

    //Remove matching single quotes around the whole string
    if cleaned.starts_with('\'') && cleaned.ends_with('\'') && cleaned.len() >= 2 {
        cleaned = cleaned[1..cleaned.len() - 1].to_string();
    }

    println!("{}", cleaned);
}
