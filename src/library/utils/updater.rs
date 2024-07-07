use std::{os::unix::fs::PermissionsExt, path::PathBuf};

use tokio::process::Command;

use crate::built_info;

use super::logging;

use tokio::fs;

async fn schedule_replace_and_restart(
    current_binary_path: &std::path::Path,
    new_binary_path: &std::path::Path,
) {
    // Use a shell script to handle the replacement after the current process exits
    let script = format!(
        r#"
        #!/bin/bash
        sleep 1
        mv "{new}" "{current}"
        chmod +x "{current}"
        "#,
        new = new_binary_path.display(),
        current = current_binary_path.display()
    );

    let rand_num = fastrand::i32(..);
    let script_path = PathBuf::from(format!("/tmp/wm-{}.sh", rand_num));
    fs::write(&script_path, script)
        .await
        .expect("Failed to write update script");
    fs::set_permissions(&script_path, PermissionsExt::from_mode(0o755))
        .await
        .expect("Failed to set script permissions");

    // Execute the script in a new process and exit the current process
    Command::new("sh")
        .arg(script_path)
        .spawn()
        .expect("Failed to execute update script");

    // Exit the current process
    std::process::exit(0);
}

pub async fn update() {
    let github_token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN must be set");

    logging::info("Updating War Machine...").await;

    let current_version: String = built_info::PKG_VERSION.to_string();

    logging::info(&format!("Current version: {}", current_version)).await;

    // Find where the binary is located
    let binary_path = std::env::current_exe().unwrap();

    let os;
    let arch;

    // Find os and arch
    let os_results = Command::new("uname").args(&["-s"]).output().await;
    match os_results {
        Ok(os_output) => {
            os = String::from_utf8(os_output.stdout).unwrap();
        }
        Err(e) => {
            logging::error(&format!("Failed to find os: {}", e)).await;
            return;
        }
    }

    let arch_results = Command::new("uname").args(&["-m"]).output().await;
    match arch_results {
        Ok(arch_output) => {
            arch = String::from_utf8(arch_output.stdout).unwrap();
        }
        Err(e) => {
            logging::error(&format!("Failed to find arch: {}", e)).await;
            return;
        }
    }

    let asset_name = format!(
        "war-machine-{}-{}",
        arch.to_lowercase().trim(),
        os.to_lowercase().trim()
    );

    let client = reqwest::Client::new();

    // Find the latest version
    let response = client
        .get("https://api.github.com/repos/generative-ai-inc/war-machine/releases/latest")
        .header("Accept", "application/json")
        .header("Authorization", &format!("token {}", github_token))
        .header("User-Agent", "wm")
        .send()
        .await;

    let response = match response {
        Ok(res) => res,
        Err(e) => {
            logging::error(&format!("Failed to get latest version: {}", e)).await;
            std::process::exit(1);
        }
    };

    if !response.status().is_success() {
        logging::error(&format!(
            "Failed to get latest version: {}",
            response.status()
        ))
        .await;
        std::process::exit(1);
    }

    let data = response.json::<serde_json::Value>().await.unwrap();

    let latest_version = data["tag_name"].as_str().unwrap();
    let latest_version = latest_version.replace("v", "");

    logging::info(&format!("Latest version: {}", latest_version)).await;

    if current_version == latest_version {
        logging::info("You are already on the latest version").await;
        std::process::exit(0);
    }

    let assets = data["assets"].as_array().unwrap();

    let asset = assets
        .iter()
        .find(|a| a["name"].as_str().unwrap().contains(&asset_name));

    let download_url;
    match asset {
        Some(a) => {
            download_url = a["url"].as_str().unwrap();
        }
        None => {
            logging::error(&format!("No asset found for {}", asset_name)).await;
            std::process::exit(1);
        }
    }

    let response = client
        .get(download_url)
        .header("Authorization", &format!("token {}", github_token))
        .header("Accept", "application/octet-stream")
        .header("User-Agent", "wm")
        .send()
        .await
        .unwrap();

    let bytes = response.bytes().await.expect("Failed to get bytes");

    let rand_num = fastrand::i32(..);
    let tmp_binary_path = PathBuf::from(format!("/tmp/wm-{}", rand_num));
    fs::write(&tmp_binary_path, bytes)
        .await
        .expect("Failed to write bytes");

    logging::info(&format!(
        "Updated War Machine to version {}",
        latest_version
    ))
    .await;

    // Schedule the replacement and restart
    schedule_replace_and_restart(&binary_path, &tmp_binary_path).await;
}
