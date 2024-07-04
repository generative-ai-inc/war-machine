use std::error::Error;
use tokio::{process::Command, sync::watch};

use crate::lib::utils::logging;

async fn install() -> Result<(), Box<dyn Error>> {
    let (tx, mut rx) = watch::channel(false);

    let child = Command::new("brew")
        .arg("install")
        .arg("docker")
        .arg("--cask")
        .spawn()
        .expect("Failed to install docker");

    let pid: u32 = child.id().expect("Failed to get docker install pid");
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
                logging::info("🐳 Docker has been installed.").await;
                Ok(())
            } else {
                Err(Box::from("🛑 Docker installation failed"))
            }
        }
        Err(e) => Err(Box::from(format!(
            "🛑 Failed to get docker installation status: {}",
            e
        ))),
    }
}

pub async fn check() {
    let docker_version_result = Command::new("docker").arg("--version").output().await;

    match docker_version_result {
        Ok(output) => {
            logging::info(&format!(
                "Docker version: {}",
                String::from_utf8_lossy(&output.stdout).trim()
            ))
            .await;
            return;
        }
        Err(_) => {
            logging::warn("Docker is not installed, installing...").await;
            let install_result = install().await;

            match install_result {
                Ok(_) => {
                    logging::warn(
                        "ℹ️ Docker was installed but needs GUI actions to continue the installation. Open the Docker application and follow the instructions."
                    )
                    .await;
                    std::process::exit(0);
                }
                Err(e) => {
                    logging::error(&e.to_string()).await;
                    std::process::exit(1);
                }
            }
        }
    }
}
