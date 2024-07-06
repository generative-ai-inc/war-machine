use clap::{arg, command, value_parser, Command};
use clap::{ArgAction, ValueHint};
use clap_complete::Shell;
use dotenv::dotenv;
use std::path::PathBuf;

use crate::CONFIG_PATH_STR;

pub fn build() -> Command {
    dotenv().ok();

    command!()
    .about("🔥🔫 War Machine is a tool for managing and installing services, tools, and libraries.")
    .subcommand(Command::new("run")
        .about("Run a command")
        .arg(
            arg!([command] "Command to run")
            .required(false)
            .value_parser(value_parser!(String))
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
                --"no-services" "Does not start the services defined in the configuration file"
            )
            .required(false)
            .action(ArgAction::SetTrue),
        )
        .arg(
            arg!(
                --clean "Clean the docker environment before starting the server"
            )
            .required(false)
            .action(ArgAction::SetTrue),
        )
        // Allow passing direct args to the command
        .arg(
            arg!(
                [command_args] ... "Arguments passed after --"
            )
            .required(false)
            .value_parser(value_parser!(String))
            .value_hint(ValueHint::Other)
            .allow_hyphen_values(true)
            .last(true)
        )
    )
    .subcommand(Command::new("secret")
        .about("Add or remove a secret")
        .subcommand_required(true)
        .subcommand(Command::new("add")
            .about("Add a secret")
            .arg_required_else_help(true)
            .arg(
                arg!([name] "Name of the secret")
                .required(false)
                .value_parser(value_parser!(String))
                .value_hint(ValueHint::Other),
            )
            .arg(
                arg!([value] "Value of the secret")
                .required(false)
                .value_parser(value_parser!(String))
                .value_hint(ValueHint::Other),
            )
        )
        .subcommand(Command::new("remove")
            .arg_required_else_help(true)
            .about("Remove a secret")
            .arg(
                arg!([name] "Name of the secret")
                .required(false)
                .value_parser(value_parser!(String))
                .value_hint(ValueHint::Other),
            )
            .arg(
                arg!(-a --all "Remove all secrets")
                .action(ArgAction::SetTrue)
            )
        )
        .subcommand(Command::new("list")
            .about("List your secrets")
        )
    )
    .subcommand(Command::new("update")
        .about("Update War Machine")
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
