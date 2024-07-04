use std::io::Write;

/// It will ask the user to input a token in the terminal and return it.
pub async fn ask_for_token(name: &str) -> String {
    let mut token = String::new();
    while token.is_empty() {
        print!("Please enter the value for {}: ", name);
        std::io::stdout().flush().unwrap();
        token = rpassword::read_password().unwrap();
    }
    token
}
