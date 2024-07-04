use crate::lib::{
    local_instances::start::start_all_instances,
    system::{self, poetry::install_deps, python_path},
    utils::{bitwarden, env_vars::set, logging},
};

/// Prepare the environment for the service to run
/// - Start local instances
/// - Get bitwarden environment variables
/// - Set the environment variables
pub async fn prepare(
    dev_mode: bool,
    no_local_instances: bool,
    clean_mode: bool,
    no_bitwarden: bool,
) {
    python_path::check().await;
    install_deps(dev_mode).await;

    let mut env_vars: Vec<(String, String, String)> = Vec::new();
    if dev_mode {
        if !no_local_instances {
            system::check().await;

            logging::nl().await;
            logging::print_color(logging::BG_YELLOW, "Starting local instances").await;
            let start_local_instances_result = start_all_instances(clean_mode).await;

            match start_local_instances_result {
                Ok(li_env_vars) => {
                    for (key, value) in li_env_vars {
                        env_vars.push((key, value, "local instances".to_string()));
                    }
                }
                Err(e) => {
                    let error_message = format!("🛑 Failed to start local instances: {}", e);
                    logging::error(&error_message).await;
                    std::process::exit(1);
                }
            }
        }
    }

    if !no_bitwarden {
        let result = bitwarden::get_env_variables().await;
        match result {
            Ok(bw_env_vars) => {
                for (key, value) in bw_env_vars {
                    env_vars.push((key, value, "bitwarden".to_string()));
                }
            }
            Err(e) => {
                let error_message =
                    format!("🛑 Failed to get bitwarden environment variables: {}", e);
                logging::error(&error_message).await;
                std::process::exit(1);
            }
        }
    }

    set(env_vars).await;
}
