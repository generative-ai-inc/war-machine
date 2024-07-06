use clap::Command;
use clap_complete::{generate, Generator, Shell};
use lazy_static::lazy_static;
use serde_json::{json, Value};
use std::io;
use std::path::PathBuf;
use war_machine::lib::commands::run;
use war_machine::lib::config::{commands, features, requirements};
use war_machine::lib::secrets::{generic, keyring};
use war_machine::lib::system::config;
use war_machine::lib::utils::{env_vars, logging, updater};

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

#[tokio::main]
async fn main() {
    // Run options
    let mut clean_mode = false;
    let mut no_services = false;
    let mut config_path = CONFIG_PATH.clone();

    let matches = cli::build().get_matches();

    let run_mode = matches.subcommand_matches("run").is_some();
    let update_mode = matches.subcommand_matches("update").is_some();
    let completions_mode = matches.subcommand_matches("completions").is_some();
    let secrets_mode = matches.subcommand_matches("secret").is_some();

    let mut command_args = "".to_string();

    if run_mode {
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

            if run_matches.get_flag("clean") {
                logging::warn("Cleaning the docker environment before starting the server").await;
                clean_mode = true;
            }

            if let Some(passed_command_args) = run_matches.get_many::<String>("command_args") {
                for arg in passed_command_args {
                    command_args = command_args + &arg + " ";
                }
            }
        }

        let config = config::parse(config_path).await;

        // Check that the command is in the config
        if let Some(ref asserted_command) = command_name {
            commands::check(&config, &asserted_command).await;
        }

        let secrets = keyring::get_secrets().await;

        features::check(&config, &secrets).await;
        requirements::check(&config).await;

        run(
            config,
            secrets,
            command_name,
            no_services,
            clean_mode,
            command_args,
        )
        .await;
    } else if update_mode {
        updater::update().await;
    } else if completions_mode {
        if let Some(completions_matches) = matches.subcommand_matches("completions") {
            if let Some(shell) = completions_matches.get_one::<Shell>("shell").copied() {
                let mut cmd = cli::build();
                eprintln!("Generating completion file for {}...", shell);
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
