use std::error::Error;

use tokio::{process::Command, sync::watch};

use crate::library::utils::logging;

pub async fn run(command_str: &str) -> Result<String, Box<dyn Error>> {
    let run_results = Command::new("sh").arg("-c").arg(command_str).output().await;

    match run_results {
        Ok(output) => {
            if output.status.success() {
                return Ok(String::from_utf8_lossy(&output.stdout).to_string());
            } else {
                return Err(Box::from(String::from_utf8_lossy(&output.stderr)));
            }
        }
        Err(e) => {
            return Err(Box::from(format!("🛑 Failed to check result: {}", e)));
        }
    }
}

pub async fn spawn(command_str: &str) -> Result<(), Box<dyn Error>> {
    let (tx, mut rx) = watch::channel(false);

    let child = Command::new("sh")
        .arg("-c")
        .arg(&command_str)
        .spawn()
        .expect("Failed to run command");

    let pid: u32 = child.id().expect("Failed to get command pid");
    let handle = child.wait_with_output();

    let install_handle = tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    logging::nl().await;
                    logging::info("🟨 Cancelling command...").await;
                    let result = Command::new("kill").arg(&pid.to_string()).status().await;

                    match result {
                        Ok(_) => {
                            logging::info("✅ Command has been terminated.").await;
                            std::process::exit(0);
                        }
                        Err(e) => {
                            logging::error(&format!("🛑 Failed to kill process: {}", e)).await;
                            std::process::exit(1);
                        }
                    }
                }
                _ = rx.changed() => {
                    if *rx.borrow() {
                        break; // Command completed, exit the task
                    }
                }
            }
        }
    });

    let output = handle.await;

    match output {
        Ok(output) => {
            if output.status.success() {
                tx.send(true).unwrap();
                install_handle.await.unwrap();
                return Ok(());
            } else {
                return Err(Box::from("🛑 Command failed"));
            }
        }
        Err(e) => return Err(Box::from(format!("🛑 Failed to get command status: {}", e))),
    }
}
