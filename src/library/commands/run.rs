use tokio::process::Command;

use crate::{
    library::utils::logging,
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

        let _output = handle.await.expect("Failed to wait for main command");

        logging::error("Failed to run service").await;
        std::process::exit(1);
    }
}
