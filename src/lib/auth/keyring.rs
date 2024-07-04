use crate::lib::utils::logging;
use keyring::Entry;
use serde_json::json;

use super::generic;

pub async fn get_credentials() -> serde_json::Value {
    // Attempt to get the credentials from the keyring
    let entry = Entry::new("war-machine", "credentials").expect("Could not create keyring entry");
    let password = entry.get_password();

    match password {
        Ok(password) => {
            let credentials: serde_json::Value = serde_json::from_str(&password).unwrap();
            credentials
        }
        Err(_) => {
            return json!({});
        }
    }
}

pub async fn set_credentials(credentials: serde_json::Value) {
    let credentials_str = serde_json::to_string(&credentials).unwrap();

    let entry = Entry::new("war-machine", "credentials").expect("Could not create keyring entry");
    let res = entry.set_password(&credentials_str);

    if res.is_err() {
        logging::error("Could not save credentials in keyring").await;
    }
}

pub async fn check() {
    let mut credentials = get_credentials().await;

    if credentials.get("GITHUB_TOKEN").is_none() {
        let token = generic::ask_for_token("GITHUB_TOKEN").await;
        credentials["GITHUB_TOKEN"] = json!(token);
    }

    if credentials.get("BWS_TOKEN").is_none() {
        let token = generic::ask_for_token("BWS_TOKEN").await;
        credentials["BWS_TOKEN"] = json!(token);
    }

    set_credentials(credentials).await;
}
