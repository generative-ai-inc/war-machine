use crate::lib::utils::{docker, logging};
use std::error::Error;
use tokio::process::Command;

pub async fn start(clean_mode: bool) -> Result<Vec<(String, String)>, Box<dyn Error>> {
    let meili_master_key = "a_UTF-8_string_of_at_least_16_bytes";

    let env_vars_to_return: Vec<(String, String)> = vec![
        (
            "MEILISEARCH_URL".to_owned(),
            "http://localhost:7700".to_owned(),
        ),
        ("MEILISEARCH_KEY".to_owned(), meili_master_key.to_owned()),
    ];

    if clean_mode {
        let remove_containers_result = docker::remove_containers("name=meili*").await;
        if remove_containers_result.is_err() {
            logging::error(&format!(
                "Failed to remove meilisearch docker containers: {}",
                remove_containers_result.err().unwrap()
            ))
            .await;

            return Err(Box::<dyn Error>::from(
                "🛑 Meilisearch docker containers could not be removed, please remove them manually",
            ));
        }
    } else {
        // Check if redis is already running
        let check_results = Command::new("docker")
            .arg("ps")
            .arg("-a")
            .arg("--filter")
            .arg("name=meilisearch")
            .output()
            .await;

        match check_results {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if output_str.contains("meilisearch") {
                    logging::info("Meilisearch docker container already running").await;
                    return Ok(env_vars_to_return);
                }
            }
            Err(e) => {
                logging::error(&format!("Failed to check if meilisearch is running: {}", e)).await;
                return Err(Box::new(e));
            }
        }
    }

    let current_dir = std::env::current_dir().unwrap();

    let start_results = Command::new("docker")
        .arg("run")
        .arg("-d")
        .arg("--name")
        .arg("meilisearch")
        .arg("-p")
        .arg("7700:7700")
        .arg("-v")
        .arg(&format!("{}/meili_data:/meili_data", current_dir.display()))
        .arg("-e")
        .arg(format!("MEILI_MASTER_KEY={}", meili_master_key))
        .arg("getmeili/meilisearch")
        .status()
        .await;

    match start_results {
        Ok(status) => {
            if status.success() {
                logging::info("Meilisearch docker container started").await;
            } else {
                logging::error("Failed to start meilisearch docker container").await;
                return Err(Box::<dyn Error>::from(
                    "Failed to start meilisearch docker container",
                ));
            }
        }
        Err(e) => {
            logging::error(&format!("Failed to start meilisearch: {}", e)).await;
            return Err(Box::new(e));
        }
    };

    Ok(env_vars_to_return)
}
