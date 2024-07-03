use crate::lib::utils::logging::{self};
use std::path::PathBuf;
use tokio::process::Command;

pub async fn run(dev_mode: bool, bind_address: String, workers: i32, config_file: PathBuf) {
    let mut main_command = format!(
        "poetry run hypercorn app.main:app --bind {} --workers {} --config {}",
        bind_address,
        workers,
        config_file.display()
    );

    if dev_mode {
        main_command.push_str(" --reload");
    }

    logging::nl().await;
    logging::print_color(logging::BG_GREEN, "Starting service").await;
    let child = Command::new("sh")
        .arg("-c")
        .arg(&main_command)
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
