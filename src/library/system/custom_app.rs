use crate::{
    library::{machine, utils::logging},
    models::{
        config::{AppSource, Config},
        machine_state::MachineState,
    },
};

use super::command;

/// Replaces the placeholders in the command with the actual values
pub async fn replace_placeholders(
    machine_state: &MachineState,
    config: &Config,
    command: &String,
    name: &str,
) -> String {
    let mut new_command = command.to_string();
    new_command = new_command.replace("${machine_name}", &config.machine_name);
    new_command = new_command.replace("${service.name}", name);
    new_command = machine::ports::replace_ports_in_text(&machine_state, &new_command).await;
    new_command
}

pub async fn start_service(
    machine_state: &MachineState,
    config: &Config,
    name: &str,
    source: &AppSource,
    clean_mode: bool,
    fail_fast: bool,
) {
    let start_command =
        replace_placeholders(&machine_state, config, &source.start_command, name).await;
    let health_check_command =
        replace_placeholders(&machine_state, config, &source.health_check_command, name).await;
    let clean_command = if let Some(clean_command) = &source.clean_command {
        Some(replace_placeholders(&machine_state, config, clean_command, name).await)
    } else {
        None
    };

    if clean_mode {
        if let Some(clean_command) = clean_command.clone() {
            let clean_results = command::spawn(&clean_command).await;

            match clean_results {
                Ok(_) => {}
                Err(_) => {
                    logging::error(&format!("Failed to clean {}", name)).await;

                    if fail_fast {
                        std::process::exit(1);
                    }
                }
            }
        }
    } else {
        let health_check_results = command::run(&health_check_command).await;

        match health_check_results {
            Ok(_) => {
                logging::info(&format!("✅ {} is running", name)).await;
                return;
            }
            Err(e) => {
                logging::error(&format!("Failed to check if {} is running: {}", name, e)).await;

                if fail_fast {
                    std::process::exit(1);
                }
            }
        }
    }

    let start_results = command::spawn(&start_command).await;

    match start_results {
        Ok(_) => {
            logging::info(&format!("🚀 Started {}", name)).await;
        }
        Err(e) => {
            logging::error(&format!("🛑 Failed to start {}", name)).await;
            logging::error(&e.to_string()).await;

            if fail_fast {
                std::process::exit(1);
            }
        }
    }
}
