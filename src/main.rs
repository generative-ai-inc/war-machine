use clap::Command;
use clap_complete::{generate, Generator, Shell};
use dotenv::dotenv;
use lazy_static::lazy_static;
use lib::auth::{credentials, generic};
use serde_json::{json, Value};
use std::io;
use std::path::PathBuf;

mod cli;
mod lib {
    pub mod auth;
    pub mod local_instances;
    pub mod service;
    pub mod system;
    pub mod utils;
}

pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

use lib::service::{run, test};
use lib::utils::{env_vars, logging, updater};

// Use lazy_static to avoid leaking string in an uncontrolled way
lazy_static! {
    pub static ref WORKERS_STRING: String = 1.to_string();
    pub static ref WORKERS_STR: &'static str = Box::leak(WORKERS_STRING.clone().into_boxed_str());
    pub static ref CONFIG_PATH: PathBuf = PathBuf::from("hypercorn.toml");
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
    dotenv().ok();

    // Run options
    let mut dev_mode = false;
    let mut clean_mode = false;
    let mut no_local_instances = false;
    let mut no_bitwarden = false;
    let mut workers = WORKERS_STR.parse::<i32>().unwrap();
    let mut config_file = CONFIG_PATH.clone();
    let mut bind_address = BIND_ADDRESS.clone();

    // Test options
    let mut path = None;
    let mut with_coverage = false;
    let mut ignore = None;
    let mut verbose = false;
    let mut save_coverage = false;
    let matches = cli::build().get_matches();

    let start_mode = matches.subcommand_matches("start").is_some();
    let test_mode = matches.subcommand_matches("test").is_some();
    let update_mode = matches.subcommand_matches("update").is_some();
    let completions_mode = matches.subcommand_matches("completions").is_some();
    let token_mode = matches.subcommand_matches("token").is_some();

    if start_mode || test_mode || update_mode {
        credentials::check().await;
    }

    if start_mode {
        if matches.get_flag("dev") {
            logging::info("Running in development mode").await;
            dev_mode = true;
        }

        if let Some(passed_bind_address) = matches.get_one::<String>("bind") {
            logging::info(&format!("Binding to: {}", passed_bind_address)).await;
            bind_address = passed_bind_address.to_owned();
        }

        if let Some(passed_workers) = matches.get_one::<i32>("workers") {
            logging::info(&format!("Workers: {}", passed_workers)).await;
            workers = passed_workers.to_owned();
        }

        if let Some(passed_config_file) = matches.get_one::<PathBuf>("config") {
            logging::info(&format!("Config file: {}", passed_config_file.display())).await;
            config_file = passed_config_file.to_owned();
        }

        if matches.get_flag("no-local-instances") {
            logging::warn("Local tool instances disabled").await;
            no_local_instances = true;
        }

        if matches.get_flag("no-bitwarden") {
            logging::warn("Bitwarden environment variables disabled").await;
            no_bitwarden = true;
        }

        if matches.get_flag("clean") {
            logging::warn("Cleaning the docker environment before starting the server").await;
            clean_mode = true;
        }

        run(
            dev_mode,
            bind_address,
            workers,
            config_file,
            no_local_instances,
            no_bitwarden,
            clean_mode,
        )
        .await;
    } else if test_mode {
        // You can check for the existence of subcommands, and if found use their
        // matches just as you would the top level cmd
        if let Some(matches) = matches.subcommand_matches("test") {
            if let Some(passed_path) = matches.get_one::<PathBuf>("path") {
                logging::info(&format!(
                    "Running tests for path: {}",
                    passed_path.display()
                ))
                .await;
                path = Some(passed_path.to_owned());
            }
            if matches.get_flag("coverage") {
                logging::info("Running tests with coverage").await;
                with_coverage = true;
            }
            if let Some(passed_ignore) = matches.get_one::<PathBuf>("ignore") {
                logging::info(&format!("Ignoring: {}", passed_ignore.display())).await;
                ignore = Some(passed_ignore.to_owned());
            }
            if matches.get_flag("verbose") {
                logging::info("Verbose output").await;
                verbose = true;
            }
            if matches.get_flag("save-coverage") {
                logging::info("Saving coverage report").await;
                save_coverage = true;
            }
            if matches.get_flag("no-local-instances") {
                logging::warn("Local tool instances disabled").await;
                no_local_instances = true;
            }
            if matches.get_flag("no-bitwarden") {
                logging::warn("Bitwarden environment variables disabled").await;
                no_bitwarden = true;
            }
            if matches.get_flag("clean") {
                logging::warn("Cleaning the docker environment before running the tests").await;
                clean_mode = true;
            }
        }

        test(
            workers,
            path,
            ignore,
            with_coverage,
            verbose,
            save_coverage,
            no_local_instances,
            clean_mode,
            no_bitwarden,
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
    } else if token_mode {
        if let Some(token_matches) = matches.subcommand_matches("token") {
            if let Some(add_matches) = token_matches.subcommand_matches("add") {
                if let Some(name) = add_matches.get_one::<String>("name") {
                    let upper_name = name.to_uppercase();
                    env_vars::verify_name(upper_name.clone()).await;
                    let mut credentials = credentials::get_credentials().await;
                    let token = generic::ask_for_token(&upper_name).await;
                    credentials[upper_name] = json!(token);
                    credentials::set_credentials(credentials).await;
                }
            } else if let Some(remove_matches) = token_matches.subcommand_matches("remove") {
                if remove_matches.get_flag("all") {
                    let credentials = json!({});
                    credentials::set_credentials(credentials).await;
                } else if let Some(name) = remove_matches.get_one::<String>("name") {
                    let upper_name = name.to_uppercase();
                    env_vars::verify_name(upper_name.clone()).await;
                    let mut credentials = credentials::get_credentials().await;
                    if let Value::Object(ref mut map) = credentials {
                        map.remove(&upper_name);
                    }
                    credentials::set_credentials(credentials).await;
                }
            } else if token_matches.subcommand_matches("list").is_some() {
                let credentials = credentials::get_credentials().await;
                for (key, _) in credentials.as_object().unwrap() {
                    println!("{}", key);
                }
            }
        }
    }
}
