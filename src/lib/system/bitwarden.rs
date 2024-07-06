use crate::lib::utils::logging;
use os_info;
use std::os::unix::fs::PermissionsExt;
use tokio::fs;
use tokio::process::Command;

pub async fn check_installation() {
    if !std::path::Path::new("bws").exists() {
        logging::info("Downloading bitwarden cli. This will only happen once.").await;

        // Check operating system to download correct binary

        let info = os_info::get();

        let url;

        if info.os_type() == os_info::Type::Macos {
            url = "https://github.com/bitwarden/sdk/releases/download/bws-v0.5.0/bws-macos-universal-0.5.0.zip".to_string();
        } else {
            let arch = info.architecture();

            match arch {
                Some(arch) => {
                    if info.os_type() == os_info::Type::Windows {
                        url = format!("https://github.com/bitwarden/sdk/releases/download/bws-v0.5.0/bws-{}-pc-windows-0.5.0.zip", arch);
                    } else {
                        url = format!("https://github.com/bitwarden/sdk/releases/download/bws-v0.5.0/bws-{}-unknown-linux-gnu-0.5.0.zip", arch);
                    }

                    logging::info("Downloading bitwarden cli for Linux").await;
                }
                None => {
                    logging::error("Failed to install bitwarden cli: Failed to get architecture")
                        .await;
                    std::process::exit(1);
                }
            }
        }

        // Print full information:
        logging::info(&format!("OS information: {info}")).await;

        let response = reqwest::get(&url).await;

        match response {
            Ok(response) => {
                let bytes = response.bytes().await.expect("Failed to read bytes");
                fs::write("bws.zip", &bytes)
                    .await
                    .expect("Failed to write file");
                Command::new("unzip")
                    .arg("bws.zip")
                    .status()
                    .await
                    .expect("Failed to unzip file");
                fs::remove_file("bws.zip")
                    .await
                    .expect("Failed to remove zip file");
                fs::set_permissions("bws", PermissionsExt::from_mode(0o755))
                    .await
                    .expect("Failed to set permissions");
                logging::info("Bitwarden cli downloaded").await;
            }
            Err(e) => {
                logging::error(&format!("Failed to download bitwarden cli: {}", e)).await;
                std::process::exit(1);
            }
        }
    }
}
