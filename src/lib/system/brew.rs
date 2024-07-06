use std::error::Error;
use tokio::{process::Command, sync::watch};

use crate::lib::utils::logging;

async fn install() -> Result<(), Box<dyn Error>> {
    let (tx, mut rx) = watch::channel(false);

    let download_result = Command::new("curl")
        .arg("-fsSL")
        .arg("https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh")
        .output()
        .await;
    let script = match download_result {
        Ok(output) => String::from_utf8_lossy(&output.stdout.to_owned()).to_string(),
        Err(e) => return Err(e.into()),
    };

    let child = Command::new("bash")
        .arg("-c")
        .arg(&script)
        .spawn()
        .expect("Failed to install brew");

    let pid: u32 = child.id().expect("Failed to get brew install pid");
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
                logging::info("🍺 Brew has been installed.").await;
                Ok(())
            } else {
                Err(Box::from("🛑 Brew installation failed"))
            }
        }
        Err(e) => Err(Box::from(format!(
            "🛑 Failed to get brew installation status: {}",
            e
        ))),
    }
}

pub async fn check_installation() {
    let brew_version_result = Command::new("brew").arg("--version").output().await;

    match brew_version_result {
        Ok(output) => {
            logging::info(&format!(
                "Brew version: {}",
                String::from_utf8_lossy(&output.stdout).trim()
            ))
            .await;
            return;
        }
        Err(_) => {
            logging::warn("Brew is not installed, installing...").await;
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
