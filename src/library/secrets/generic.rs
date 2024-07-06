use std::io::Write;

/// It will ask the user to input a secret in the terminal and return it.
pub async fn ask_for_secret(name: &str) -> String {
    let mut secret = String::new();
    while secret.is_empty() {
        print!("Please enter the value for secret {}: ", name);
        std::io::stdout().flush().unwrap();
        secret = rpassword::read_password().unwrap();
    }
    secret
}
