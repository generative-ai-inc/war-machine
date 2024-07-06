use crate::library::utils::logging;
use keyring::Entry;
use serde_json::json;

pub async fn get_secrets() -> serde_json::Value {
    // Attempt to get the secrets from the keyring
    let entry = Entry::new("war-machine", "secrets").expect("Could not create keyring entry");
    let password = entry.get_password();

    match password {
        Ok(password) => {
            let secrets: serde_json::Value = serde_json::from_str(&password).unwrap();
            secrets
        }
        Err(_) => {
            return json!({});
        }
    }
}

pub async fn set_secret(secrets: serde_json::Value) {
    let secrets_str = serde_json::to_string(&secrets).unwrap();

    let entry = Entry::new("war-machine", "secrets").expect("Could not create keyring entry");
    let res = entry.set_password(&secrets_str);

    if res.is_err() {
        logging::error("Could not save secrets in keyring").await;
    }
}
