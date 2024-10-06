use crate::{
    library::{
        system::{brew, docker, pipx, poetry, python},
        utils::logging,
    },
    models::config::{Config, Requirement},
};

pub async fn check(config: &Config) {
    for requirement in &config.requirements {
        match requirement {
            Requirement::Brew => {
                brew::check_installation().await;
            }
            Requirement::Docker => {
                docker::check_installation().await;

                // Create docker networks
                for network in &config.networks {
                    let result = docker::create_network(network).await;
                    match result {
                        Ok(_) => (),
                        Err(e) => {
                            logging::error(&format!("Failed to create network {}: {}", network, e))
                                .await;
                            std::process::exit(1);
                        }
                    }
                }
            }
            Requirement::Pipx => {
                pipx::check_installation().await;
            }
            Requirement::Python => {
                python::check_installation().await;
            }
            Requirement::Poetry => {
                poetry::check_installation().await;
            }
        }
    }
}
