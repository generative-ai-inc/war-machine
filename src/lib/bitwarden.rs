use crate::lib::utils::logging;
use dotenv::dotenv;
use os_info;
use regex::Regex;
use std::env;
use std::error::Error;
use std::os::unix::fs::PermissionsExt;
use tokio::fs;
use tokio::process::Command;

pub async fn get_env_variables() -> Result<Vec<(String, String)>, Box<dyn Error>> {
    dotenv().ok();

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
                    logging::error("Failed to get architecture").await;
                    return Err(Box::from(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Failed to get architecture",
                    )));
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
                return Err(e.into());
            }
        }
    }

    let output = Command::new("./bws")
        .args(&[
            "secret",
            "list",
            "--output",
            "env",
            "--access-token",
            &env::var("BWS_ACCESS_TOKEN").unwrap(),
        ])
        .output()
        .await
        .expect("Failed to list bitwarden secrets");

    logging::info("Retrieved bitwarden environment variables").await;

    let re = Regex::new(r#"^([A-Z_]+)="(.+)""#).unwrap();

    let env_vars_str = String::from_utf8_lossy(&output.stdout);
    let mut env_vars: Vec<(String, String)> = Vec::new();

    for line in env_vars_str.lines() {
        if let Some(caps) = re.captures(line) {
            let key = &caps[1];
            let value = &caps[2];

            env_vars.push((key.to_string(), value.to_string()));
        }
    }

    Ok(env_vars)
}
