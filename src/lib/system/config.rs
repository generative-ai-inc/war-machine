use std::path::PathBuf;

use tokio::fs::{self};

use crate::{lib::utils::logging, models::config::Config};

/// Checks that the config file is set up correctly
pub async fn parse(config_path: PathBuf) -> Config {
    // Read the TOML file
    let toml_content = fs::read_to_string(config_path).await;

    let toml_content = match toml_content {
        Ok(content) => content,
        Err(e) => {
            logging::error(&format!("Error reading config file: {}", e)).await;
            std::process::exit(1);
        }
    };

    // Parse the TOML content
    let config_result = toml::from_str(&toml_content);

    let config: Config = match config_result {
        Ok(parsed_config) => parsed_config,
        Err(e) => {
            logging::error(&format!("Error parsing config file: {}", e)).await;
            std::process::exit(1);
        }
    };

    config
}
