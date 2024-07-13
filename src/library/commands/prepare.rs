use std::{collections::HashMap, env, path::PathBuf};

use crate::{
    library::{
        config::services,
        machine,
        system::{command, docker, pythonpath},
        utils::{bitwarden, env_vars, logging},
    },
    models::{
        config::{Config, ExposedValueType, Feature},
        machine_state::MachineState,
    },
};

async fn prepare_features(
    config: &Config,
    secrets: &serde_json::Value,
    env_vars: &mut Vec<(String, String, String)>,
) {
    for feature in &config.features {
        match feature {
            Feature::PYTHONPATH(pythonpath_feature) => {
                let env_file_path = PathBuf::from(&pythonpath_feature.env_file_path);
                let pythonpath_value = PathBuf::from(&pythonpath_feature.pythonpath_value);
                pythonpath::check(&env_file_path, &pythonpath_value).await;
            }
            Feature::BITWARDEN => {
                let bw_env_vars = bitwarden::get_env_variables(secrets).await;

                for (key, value) in bw_env_vars {
                    env_vars.push((key, value, "bitwarden".to_string()));
                }
            }
        }
    }
}

pub async fn get_exposed_variables(
    machine_state: &MachineState,
    exposed_values: &Vec<ExposedValueType>,
    available_before_start: bool,
) -> Vec<(String, String)> {
    let mut env_vars_to_return: Vec<(String, String)> = vec![];

    for exposed_value in exposed_values {
        match exposed_value {
            ExposedValueType::LITERAL(exposed_value) => {
                if available_before_start == exposed_value.available_before_start {
                    env_vars_to_return.push((
                        exposed_value.name.trim().to_uppercase(),
                        machine::ports::replace_ports_in_text(machine_state, &exposed_value.value)
                            .await,
                    ));
                }
            }
            ExposedValueType::COMMAND(exposed_value) => {
                if available_before_start == exposed_value.available_before_start {
                    let command_result = command::run(&exposed_value.command).await.unwrap();

                    // Each line should be in the format KEY=VALUE and represent one exposed value.
                    for line in command_result.split('\n') {
                        // Some lines might be empty, so we skip them.
                        if line.is_empty() {
                            continue;
                        }

                        // Some lines might have comments, so we ignore them.
                        if line.starts_with('#') {
                            continue;
                        }

                        // Some lines may not have an equals sign, so we ignore them.
                        if !line.contains('=') {
                            continue;
                        }

                        let (mut key, mut value) = line.split_once('=').unwrap();

                        // Check if the key is in the exclude list
                        if exposed_value.exclude.contains(&key.trim().to_string()) {
                            continue;
                        }

                        if let Some(rename) = exposed_value.rename.get(key.trim()) {
                            key = rename;
                        }

                        // If the value is surrounded by double quotes, remove them
                        if value.starts_with('"') && value.ends_with('"') {
                            value = &value[1..value.len() - 1];
                        }

                        env_vars_to_return
                            .push((key.trim().to_uppercase(), value.trim().to_string()));
                    }
                }
            }
        }
    }

    env_vars_to_return
}

/// Prepare the environment for the service to run.
/// - Add features
/// - Set the environment variables
/// - Create docker networks
/// - Start local instances
pub async fn prepare(
    machine_state: &MachineState,
    config: &Config,
    secrets: &serde_json::Value,
    no_services: bool,
    no_features: bool,
    clean_mode: bool,
) {
    let vars_iter = env::vars();

    let mut original_env_vars: HashMap<String, String> = HashMap::new();

    for (key, value) in vars_iter {
        original_env_vars.insert(key, value);
    }

    let mut env_vars: Vec<(String, String, String)> = Vec::new();

    // Add war machine secrets to the environment variables
    for (key, value) in secrets.as_object().unwrap() {
        env_vars.push((
            key.to_string(),
            value.as_str().unwrap().to_string(),
            "war machine secrets".to_string(),
        ));
    }

    if !no_features {
        prepare_features(config, secrets, &mut env_vars).await;
    }

    if !no_services {
        // Set the available_before_start variables
        for service in &config.services {
            let exposed_values =
                get_exposed_variables(&machine_state, &service.exposed_values, true).await;

            for (key, value) in exposed_values {
                env_vars.push((key, value, "war machine".to_string()));
            }
        }

        env_vars::set(&env_vars).await;

        // Login to all registries
        for registry in &config.registry_credentials {
            let username = env_vars::get(&registry.username).await;
            let password = env_vars::get(&registry.password).await;

            let login_result = docker::login(&registry.registry, &username, &password).await;

            if login_result.is_err() {
                logging::error(&format!("Failed to login to {}", registry.registry)).await;
                std::process::exit(1);
            }
        }

        logging::nl().await;
        logging::print_color(logging::BG_YELLOW, " Starting local instances ").await;

        services::start_all(machine_state, config, clean_mode, true).await;

        // Logout from all registries
        for registry in &config.registry_credentials {
            let logout_result = docker::logout(&registry.registry).await;

            if logout_result.is_err() {
                logging::error(&format!("Failed to logout from {}", registry.registry)).await;
                std::process::exit(1);
            }
        }

        // Set the available_before_start variables
        for service in &config.services {
            let exposed_values =
                get_exposed_variables(&machine_state, &service.exposed_values, false).await;

            for (key, value) in exposed_values {
                env_vars.push((key, value, "war machine".to_string()));
            }
        }
    }

    env_vars::set(&env_vars).await;

    env_vars::print_variables_box(original_env_vars, &env_vars).await;
}
