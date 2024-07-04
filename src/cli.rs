use clap::{arg, command, value_parser, Command};
use clap::{ArgAction, ValueHint};
use clap_complete::Shell;
use dotenv::dotenv;
use std::path::PathBuf;

use crate::{BIND_ADDRESS_STR, CONFIG_PATH_STR, WORKERS_STR};

pub fn build() -> Command {
    dotenv().ok();

    command!()
    .about("🔥🔫 War Machine is a tool for managing and installing services, tools, and libraries.")
    .subcommand(Command::new("update")
        .about("Update War Machine")
    )
    .subcommand(Command::new("start") // requires `cargo` feature
        .about("Starts the server. Use --dev to enable development mode")
        .arg(
            arg!(
                -d --dev ... "Enable development mode. This will start local instances by default.\nIt will also enable reloading the server on file changes."
            )
            .required(false)
            .action(ArgAction::SetTrue)
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
            .value_parser(value_parser!(PathBuf))
            .value_hint(ValueHint::AnyPath),
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
    ).subcommand(
        Command::new("test")
            .about("Run the tests")
            .arg(
                arg!([path] "Path to the test file or directory")
                .required(false)
                .value_parser(value_parser!(PathBuf))
                .value_hint(ValueHint::AnyPath),
            )
            .arg(
                arg!(-c --coverage ... "Run the tests with coverage")
                .action(ArgAction::SetTrue)
            ).arg(
                arg!(-i --ignore <PATH> "Ignore the specified tests")
                .required(false)
                .value_parser(value_parser!(PathBuf))
                .value_hint(ValueHint::AnyPath),
            )
            .arg(
                arg!(-v --verbose ... "Verbose output")
                .action(ArgAction::SetTrue)
            )
            .arg(
                arg!(
                    -w --workers <INT> "Number of test workers to use"
                )
                .default_value(*WORKERS_STR)
                .required(false)
                .value_parser(value_parser!(i32)),
            )
            .arg(
                arg!(
                    --"save-coverage" ... "Save the coverage report"
                )
                .action(ArgAction::SetTrue)
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
    )
    .subcommand(Command::new("token")
        .about("Add or remove an access token")
        .subcommand_required(true)
        .subcommand(Command::new("add")
            .about("Add an access token")
            .arg_required_else_help(true)
            .arg(
                arg!([name] "Name of the token")
                .required(false)
                .value_parser(value_parser!(String))
                .value_hint(ValueHint::Other),
            )
            .arg(
                arg!([value] "Value of the token")
                .required(false)
                .value_parser(value_parser!(String))
                .value_hint(ValueHint::Other),
            )
        )
        .subcommand(Command::new("remove")
            .arg_required_else_help(true)
            .about("Remove an access token")
            .arg(
                arg!([name] "Name of the token")
                .required(false)
                .value_parser(value_parser!(String))
                .value_hint(ValueHint::Other),
            )
            .arg(
                arg!(-a --all ... "Remove all tokens")
                .action(ArgAction::SetTrue)
            )
        )
        .subcommand(Command::new("list")
            .about("List your access tokens")
        )
    )
    .subcommand(Command::new("completions")
        .about("Generate shell completions. Place the output in your shell's completions directory")
        .arg_required_else_help(true)
        .arg(
            arg!([shell] "Shell to generate completions for.")
            .required(true)
            .value_parser(value_parser!(Shell))
        )
    )
}
