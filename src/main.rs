use clap::ArgAction;
use clap::{arg, command, value_parser, Command};
use dotenv::dotenv;
use lazy_static::lazy_static;
use lib::bitwarden;
use lib::local_instances::start::start_all_instances;
use lib::utils::env_vars::set_env_vars;
use std::path::PathBuf;

mod lib {
    pub mod bitwarden;
    pub mod local_instances;
    pub mod service;
    pub mod utils;
}

use lib::service::{run, test};
use lib::utils::logging;

// Use lazy_static to avoid leaking string in an uncontrolled way
lazy_static! {
    static ref WORKERS_STRING: String = 1.to_string();
    static ref WORKERS_STR: &'static str = Box::leak(WORKERS_STRING.clone().into_boxed_str());
    static ref CONFIG_PATH: PathBuf = PathBuf::from("hypercorn.toml");
    static ref CONFIG_PATH_STR: &'static str =
        Box::leak(CONFIG_PATH.to_str().unwrap().to_string().into_boxed_str());
    static ref BIND_ADDRESS: String = "127.0.0.1:8000".to_string();
    static ref BIND_ADDRESS_STR: &'static str = Box::leak(BIND_ADDRESS.clone().into_boxed_str());
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

    let matches = command!() // requires `cargo` feature
        .about("Starts the server. Use --test to run the tests.")
        .arg(
            arg!(
                -d --dev ... "Enable development mode. This will start local instances by default.\nIt will also enable reloading the server on file changes."
            )
            .required(false)
            .action(ArgAction::SetTrue),
        )
        .arg(
            arg!(
                -b --bind <STR> "Bind to the specified address."
            )
            .default_value(*BIND_ADDRESS_STR)
            .required(false)
            .value_parser(value_parser!(String)),
        )
        .arg(
            arg!(
                -w --workers <INT> "Number of workers to use."
            )
            .default_value(*WORKERS_STR)
            .required(false)
            .value_parser(value_parser!(i32)),
        )
        .arg(
            arg!(
                -c --config <FILE> "Configuration file to use."
            )
            .default_value(*CONFIG_PATH_STR)
            .required(false)
            .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            arg!(
                --"no-local-instances" ... "Disable starting local tool instances like Redis, Supabase, Qdrant, etc."
            )
            .required(false)
            .action(ArgAction::SetTrue),
        )
        .arg(
            arg!(
                --"no-bitwarden" ... "Disable fetching Bitwarden environment variables"
            )
            .required(false)
            .action(ArgAction::SetTrue),
        )
        .arg(
            arg!(
                --clean ... "Clean the docker environment before starting the server"
            )
            .required(false)
            .action(ArgAction::SetTrue),
        )
        .subcommand(
            Command::new("test")
                .about("Run the tests")
                .arg(
                    arg!([path] "Path to the test file or directory")
                    .required(false)
                    .value_parser(value_parser!(PathBuf)),
                )
                .arg(
                    arg!(-c --coverage ... "Run the tests with coverage")
                    .action(ArgAction::SetTrue)
                ).arg(
                    arg!(-i --ignore <PATH> "Ignore the specified tests")
                    .required(false)
                    .value_parser(value_parser!(PathBuf)),
                )
                .arg(
                    arg!(-v --verbose ... "Verbose output")
                    .action(ArgAction::SetTrue)
                )
                .arg(
                    arg!(
                        --"save-coverage" ... "Save the coverage report"
                    )
                    .action(ArgAction::SetTrue)
                ),
        )
        .get_matches();

    if matches.get_flag("dev") {
        logging::info("Running in development mode").await;
        dev_mode = true;
    }

    if let Some(passed_bind_address) = matches.get_one::<String>("bind") {
        logging::info(&format!("Binding to: {}", passed_bind_address)).await;
        bind_address = passed_bind_address.to_string().to_owned();
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

    let mut env_vars: Vec<(String, String, String)> = Vec::new();
    if dev_mode {
        if !no_local_instances {
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

    set_env_vars(env_vars).await;

    let test_mode = matches.subcommand_matches("test").is_some();

    if test_mode {
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
        }

        test(workers, path, ignore, with_coverage, verbose, save_coverage).await;
    } else {
        run(dev_mode, bind_address, workers, config_file).await;
    }
}
