pub fn run_echo_command(text: String) {
    let mut in_single_quote = false;

    for c in text.chars() {
        if c == '\'' {
            in_single_quote = !in_single_quote;
            continue; // skip quotes
        }
        print!("{}", c);
    }
    println!();
}
