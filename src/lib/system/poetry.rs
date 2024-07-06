use std::error::Error;
use tokio::{process::Command, sync::watch};

use crate::lib::utils::logging;

async fn install() -> Result<(), Box<dyn Error>> {
    let (tx, mut rx) = watch::channel(false);

    let child = Command::new("pipx")
        .arg("install")
        .arg("poetry")
        .spawn()
        .expect("Failed to install poetry");

    let pid: u32 = child.id().expect("Failed to get poetry install pid");
    let handle = child.wait_with_output();

    let install_handle = tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    logging::nl().await;
                    logging::info("🟨 Cancelling installation...").await;
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
                logging::info("📝 Poetry has been installed.").await;
                Ok(())
            } else {
                Err(Box::from("🛑 Poetry installation failed"))
            }
        }
        Err(e) => Err(Box::from(format!(
            "🛑 Failed to get poetry install status: {}",
            e
        ))),
    }
}

pub async fn check_installation() {
    let poetry_version_result = Command::new("poetry").arg("--version").output().await;

    match poetry_version_result {
        Ok(output) => {
            logging::info(&format!(
                "Poetry version: {}",
                String::from_utf8_lossy(&output.stdout).trim()
            ))
            .await;
            return;
        }
        Err(_) => {
            logging::warn("Poetry is not installed, installing...").await;
            let install_result = install().await;

            match install_result {
                Ok(()) => return,
                Err(e) => {
                    logging::error(&e.to_string()).await;
                    std::process::exit(1);
                }
            }
        }
    }
}
