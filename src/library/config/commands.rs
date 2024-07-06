use crate::{library::utils::logging, models::config::Config};

/// Check that the command is in the configuration
pub async fn check(config: &Config, command: &String) {
    if !config.commands.contains_key(command) {
        logging::error(&format!("Command {} not found in the config", command)).await;
        std::process::exit(1);
    }
}
