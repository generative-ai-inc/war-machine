use tokio::process::Command;

use crate::{
    library::{system::command, utils::logging},
    models::{config::Config, machine_state::MachineState},
};

use super::prepare;

pub async fn run(
    machine_state: MachineState,
    config: Config,
    secrets: serde_json::Value,
    command_name: Option<String>,
    no_services: bool,
    clean_mode: bool,
    command_args: String,
) {
    prepare(&machine_state, &config, &secrets, no_services, clean_mode).await;

    if let Some(command_name) = command_name {
        let pre_command_result = config.pre_commands.get(&command_name);
        if let Some(pre_command) = pre_command_result {
            let result = command::run(pre_command).await;
            match result {
                Ok(output) => {
                    logging::info(&format!("Output: {}", output)).await;
                    logging::info("✅ Pre command completed successfully").await;
                }
                Err(e) => {
                    logging::error(e.to_string().as_str().trim()).await;
                    logging::error("🛑 Failed to run pre command").await;
                    std::process::exit(1);
                }
            }
        }

        let command = config.commands.get(&command_name).unwrap();
        let command = format!("{} {}", command, command_args);
        logging::nl().await;
        logging::print_color(logging::BG_GREEN, " Starting service ").await;
        logging::info(&format!("Running: {}", command)).await;
        let child = Command::new("sh")
            .arg("-c")
            .arg(&command)
            .spawn()
            .expect("Failed to start main command");

        let pid = child.id().expect("Failed to get child pid");
        let handle = child.wait_with_output();

        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.unwrap();
            logging::nl().await;
            logging::info("👍 Shutting down gracefully...").await;
            let result = Command::new("kill").arg(&pid.to_string()).status().await;

            match result {
                Ok(_) => {
                    logging::info("✅ All processes have been terminated.").await;
                    std::process::exit(0);
                }
                Err(e) => {
                    logging::error(&format!("🛑 Failed to kill process: {}", e)).await;
                    std::process::exit(1);
                }
            }
        });

        let output = handle.await;

        match output {
            Ok(output) => {
                if output.status.success() {
                    std::process::exit(0);
                } else {
                    std::process::exit(1);
                }
            }
            Err(e) => {
                logging::error(&format!("🛑 Failed to wait for main command: {}", e)).await;
            }
        }
    }
}
