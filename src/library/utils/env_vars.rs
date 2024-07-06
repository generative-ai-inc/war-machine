use std::{collections::HashMap, env};

use regex::Regex;

use crate::library::{secrets, utils::logging};

pub async fn print_variables_box(
    original_env_vars: HashMap<String, String>,
    env_vars: &Vec<(String, String, String)>,
) {
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
                "Secret {} is not set, please set it with `war secret add {}`. Alternatively, you can set it in your environment variables.",
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
            "Secret {} is not set, please set it with `wm secret add {}`. Alternatively, you can set it in your environment variables.",
            name, name,
        ))
        .await;
    std::process::exit(1);
}
