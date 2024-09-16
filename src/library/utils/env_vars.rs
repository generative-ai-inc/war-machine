use std::{collections::HashMap, env};

use regex::Regex;

use crate::library::{secrets, utils::logging};

pub async fn print_variables_box(
    original_env_vars: HashMap<String, String>,
    env_vars: &Vec<(String, String, String)>,
) {
    // If there are no environment variables, don't print anything
    if env_vars.is_empty() {
        return;
    }

    logging::nl().await;
    logging::print_color(logging::BG_BLUE, " Environment variables ").await;

    // We need to find the longest key so we can align the table
    let longest_key = env_vars.iter().max_by_key(|(key, _, _)| key.len());
    let longest_key_len = longest_key.map_or(0, |(key, _, _)| key.len());

    let longest_source_len = env_vars.iter().max_by_key(|(_, _, source)| source.len());
    let longest_source_len = longest_source_len.map_or(0, |(_, _, source)| source.len());

    let key_margin = "─".to_string().repeat(longest_key_len);
    let source_margin = "─".to_string().repeat(longest_source_len);

    // Using longer | character for sides: │
    logging::print_color(
        logging::NC,
        &format!("┌─{}─┬─{}─┐", key_margin, source_margin),
    )
    .await;
    logging::print_color(
        logging::NC,
        &format!(
            "│ {:<key_width$} │ {:<source_width$} │",
            "Key",
            "Source",
            key_width = longest_key_len,
            source_width = longest_source_len
        ),
    )
    .await;
    logging::print_color(
        logging::NC,
        &format!("├─{}─┼─{}─┤", key_margin, source_margin),
    )
    .await;

    // Sort by key
    let mut sorted_env_vars = env_vars.clone();
    sorted_env_vars.sort_by_key(|(key, _, _)| key.clone());

    for (key, _, source) in sorted_env_vars {
        let parsed_key = format!("{:<width$}", key, width = longest_key_len);
        let mut parsed_source = format!("{:<width$}", source, width = longest_source_len);

        // Check if the key is already set
        if original_env_vars.contains_key(&key) {
            parsed_source = format!("{:<width$}", "local", width = longest_source_len);
        }

        logging::print_color(
            logging::NC,
            &format!("│ {} │ {} │", parsed_key, parsed_source),
        )
        .await;
    }

    logging::print_color(
        logging::NC,
        &format!("└─{}─┴─{}─┘", key_margin, source_margin),
    )
    .await;
}

pub async fn set(env_vars: &Vec<(String, String, String)>) {
    for (key, value, _) in env_vars {
        // Only set if
        if !env::var(key).is_ok() {
            env::set_var(key, value);
        }
    }
}

pub async fn verify_name(name: String) {
    let regex = Regex::new(r"^[a-zA-Z0-9_]+$").unwrap();
    regex.is_match(&name);

    if !regex.is_match(&name) {
        println!("Invalid variable name: {}", name);
        std::process::exit(1);
    }
}

pub async fn make_sure_exists(secrets: Option<&serde_json::Value>, name: &str) {
    // First check if the secret is set in the environment variables
    if std::env::var(name).is_ok() {
        return;
    }
    // If it's not, check if check the secrets file
    else {
        let secrets_value = if let Some(secrets) = secrets {
            secrets
        } else {
            &secrets::keyring::get_secrets().await
        };

        if secrets_value.get(name).is_none() {
            logging::error(&format!(
                "Secret {} is not set, please set it with `wm secret add {}`. Alternatively, you can set it in your environment variables.",
                name, name,
            ))
            .await;
            std::process::exit(1);
        }
    }
}

/// This function is only needed before the env variables are set.
/// Once the variables are set we can simply use `env_vars::get`
pub async fn get_from_all(secrets: &serde_json::Value, name: &str) -> String {
    // First check if the secret is set in the environment variables
    if let Some(value) = std::env::var(name).ok() {
        return value;
    }

    if let Some(value) = secrets.get(name) {
        return value.as_str().unwrap().to_string();
    }

    logging::error(&format!(
            "Secret {} is not set, please set it with `wm secret add {}`. Alternatively, you can set it in your environment variables.",
            name, name,
        ))
        .await;
    std::process::exit(1);
}

pub async fn get(name: &str) -> String {
    if let Some(value) = std::env::var(name).ok() {
        return value;
    }

    logging::error(&format!(
            "Secret {} is not set, please set it with `wm secret add {}`. Alternatively, you can set it in your .env file.",
            name, name,
        ))
        .await;
    std::process::exit(1);
}

/// Replaces the environment variables in the string with the actual values
pub async fn replace_env_vars(text: &str) -> String {
    // Match $VAR_NAME or ${VAR_NAME}
    let re = Regex::new(r"\$\{?(\w+)\}?").unwrap();

    // Initialize a new String to build the result
    let mut result = String::new();
    let mut last_end = 0;

    // Collect all matches and their positions
    let matches: Vec<_> = re.captures_iter(text).collect();

    for caps in matches {
        let m = caps.get(0).unwrap(); // The full match (e.g., ${VAR_NAME})
        let var_name = &caps[1]; // The captured variable name

        // Append the text before the current match
        result.push_str(&text[last_end..m.start()]);

        // Asynchronously get the environment variable's value
        let value = get(var_name).await;

        // Append the retrieved value (or a default if None)
        result.push_str(&value);

        last_end = m.end();
    }

    // Append any remaining text after the last match
    result.push_str(&text[last_end..]);

    result
}
