use clap::{ArgMatches, Command};
use clap_complete::{generate, Generator, Shell};
use futures::future;
use lazy_static::lazy_static;
use serde_json::{json, Value};
use std::io;
use std::path::PathBuf;
use wm::library::commands::run;
use wm::library::config::{commands, dependencies, features, requirements, services};
use wm::library::machine;
use wm::library::secrets::{generic, keyring};
use wm::library::system::config;
use wm::library::utils::{env_vars, logging, updater};

mod cli;

// Use lazy_static to avoid leaking string in an uncontrolled way
lazy_static! {
    pub static ref WORKERS_STRING: String = 1.to_string();
    pub static ref WORKERS_STR: &'static str = Box::leak(WORKERS_STRING.clone().into_boxed_str());
    pub static ref CONFIG_PATH: PathBuf = PathBuf::from("war_machine.toml");
    pub static ref CONFIG_PATH_STR: &'static str =
        Box::leak(CONFIG_PATH.to_str().unwrap().to_string().into_boxed_str());
    pub static ref BIND_ADDRESS: String = "127.0.0.1:8000".to_string();
    pub static ref BIND_ADDRESS_STR: &'static str =
        Box::leak(BIND_ADDRESS.clone().into_boxed_str());
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}

async fn handle_run_mode(matches: ArgMatches) {
    // Run options
    let mut run_clean_mode = false;
    let mut no_services = false;
    let mut no_features = false;
    let mut no_requirements = false;
    let mut config_path = CONFIG_PATH.clone();

    let mut command_args = "".to_string();

    let mut command_name = None;
    if let Some(run_matches) = matches.subcommand_matches("run") {
        if let Some(passed_command_name) = run_matches.get_one::<String>("command") {
            command_name = Some(passed_command_name.to_owned());
            logging::info(&format!("Command: {}", passed_command_name)).await;
        }

        if let Some(passed_config_path) = run_matches.get_one::<PathBuf>("config") {
            logging::info(&format!("Config file: {}", passed_config_path.display())).await;
            config_path = passed_config_path.to_owned();
        }

        if run_matches.get_flag("no-services") {
            logging::warn("Running without services").await;
            no_services = true;
        }

        if run_matches.get_flag("no-features") {
            logging::warn("Running without features").await;
            no_features = true;
        }

        if run_matches.get_flag("no-requirements") {
            logging::warn("Running without requirements").await;
            no_requirements = true;
        }

        if run_matches.get_flag("clean") {
            logging::warn("Cleaning the docker environment before starting the server").await;
            run_clean_mode = true;
        }

        if let Some(passed_command_args) = run_matches.get_many::<String>("command_args") {
            for arg in passed_command_args {
                command_args = command_args + &arg + " ";
            }
        }
    }

    let config = config::parse(config_path).await;

    let machine_state = machine::state::check(&config, run_clean_mode).await;

    // Check that the command is in the config
    if let Some(ref asserted_command) = command_name {
        commands::check(&config, &asserted_command).await;
    }

    let secrets = keyring::get_secrets().await;

    if !no_features {
        features::check(&config, &secrets).await;
    }
    if !no_requirements {
        requirements::check(&config).await;
    }

    if !no_services {
        dependencies::check(&config).await;
    }

    run(
        machine_state,
        config,
        secrets,
        command_name,
        no_services,
        no_features,
        run_clean_mode,
        command_args,
    )
    .await;
}

async fn handle_clean_mode(matches: ArgMatches) {
    let mut config_path = CONFIG_PATH.clone();
    let mut clean_all = false;

    let mut service_name = None;
    if let Some(run_matches) = matches.subcommand_matches("clean") {
        if run_matches.get_flag("all") {
            logging::warn("🧼🧼🧼 Cleaning all services 🧼🧼🧼").await;
            clean_all = true;
        }
        if let Some(passed_service_name) = run_matches.get_one::<String>("service") {
            service_name = Some(passed_service_name.to_owned());
            logging::info(&format!("🧼 Cleaning service: {}", passed_service_name)).await;
        } else if !clean_all {
            logging::error("No service name provided. Clean all services with the --all flag")
                .await;
            std::process::exit(1);
        }

        if let Some(passed_config_path) = run_matches.get_one::<PathBuf>("config") {
            logging::info(&format!("Config file: {}", passed_config_path.display())).await;
            config_path = passed_config_path.to_owned();
        }
    }

    let config = config::parse(config_path).await;

    let machine_state = machine::state::check(&config, false).await;

    if clean_all {
        let mut tasks = vec![];
        for service in &config.services {
            tasks.push(services::clean(&machine_state, &config, service, true));
        }
        future::join_all(tasks).await;
    } else if let Some(service_name) = service_name {
        let service = config.services.iter().find(|s| s.name == service_name);

        match service {
            Some(service) => services::clean(&machine_state, &config, &service, true).await,
            None => {
                logging::error(&format!("Service {} not found", service_name)).await;
                std::process::exit(1);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let matches = cli::build().get_matches();

    let run_mode = matches.subcommand_matches("run").is_some();
    let update_mode = matches.subcommand_matches("update").is_some();
    let completions_mode = matches.subcommand_matches("completions").is_some();
    let secrets_mode = matches.subcommand_matches("secret").is_some();
    let clean_mode = matches.subcommand_matches("clean").is_some();

    if run_mode {
        handle_run_mode(matches).await;
    } else if clean_mode {
        handle_clean_mode(matches).await;
    } else if update_mode {
        updater::update().await;
    } else if completions_mode {
        if let Some(completions_matches) = matches.subcommand_matches("completions") {
            if let Some(shell) = completions_matches.get_one::<Shell>("shell").copied() {
                let mut cmd = cli::build();
                print_completions(shell, &mut cmd);
            }
        }
    } else if secrets_mode {
        if let Some(secrets_matches) = matches.subcommand_matches("secret") {
            if let Some(add_matches) = secrets_matches.subcommand_matches("add") {
                if let Some(name) = add_matches.get_one::<String>("name") {
                    let upper_name = name.to_uppercase();
                    env_vars::verify_name(upper_name.clone()).await;

                    let mut secrets = keyring::get_secrets().await;
                    let secret;
                    if let Some(value) = add_matches.get_one::<String>("value") {
                        secret = value.to_owned();
                    } else {
                        secret = generic::ask_for_secret(&upper_name).await;
                    }
                    secrets[upper_name] = json!(secret);
                    keyring::set_secret(secrets).await;
                }
            } else if let Some(remove_matches) = secrets_matches.subcommand_matches("remove") {
                if remove_matches.get_flag("all") {
                    let secrets = json!({});
                    keyring::set_secret(secrets).await;
                } else if let Some(name) = remove_matches.get_one::<String>("name") {
                    let upper_name = name.to_uppercase();
                    env_vars::verify_name(upper_name.clone()).await;
                    let mut secrets = keyring::get_secrets().await;
                    if let Value::Object(ref mut map) = secrets {
                        map.remove(&upper_name);
                    }
                    keyring::set_secret(secrets).await;
                }
            } else if secrets_matches.subcommand_matches("list").is_some() {
                let credentials = keyring::get_secrets().await;
                for (key, _) in credentials.as_object().unwrap() {
                    println!("{}", key);
                }
            }
        }
    } else {
        cli::build().print_help().unwrap();
    }
}
