pub fn run_echo_command(text: String) {
    let mut in_single_quote = false;

    for c in text.chars() {
        if c == '\'' {
            in_single_quote = !in_single_quote;
            continue;
        }

        if in_single_quote {
            print!("{}", c);
        } else {
            if !c.is_whitespace() {
                print!("{}", c);
            } else {
                print!(" ")
            }
        }
    }
    println!();
}
