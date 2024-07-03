use crate::lib::utils::{docker, logging};
use std::error::Error;
use tokio::process::Command;

pub async fn start(clean_mode: bool) -> Result<Vec<(String, String)>, Box<dyn Error>> {
    let env_vars_to_return: Vec<(String, String)> = vec![
        (
            "QDRANT_API_URL".to_owned(),
            "http://localhost:6333".to_owned(),
        ),
        ("QDRANT_API_PORT".to_owned(), "6333".to_owned()),
        ("QDRANT_API_KEY".to_owned(), "".to_owned()),
    ];

    if clean_mode {
        let remove_containers_result = docker::remove_containers("name=qdrant*").await;
        if remove_containers_result.is_err() {
            logging::error(&format!(
                "Failed to remove qdrant docker containers: {}",
                remove_containers_result.err().unwrap()
            ))
            .await;

            return Err(Box::<dyn Error>::from(
                "🛑 Qdrant docker containers could not be removed, please remove them manually",
            ));
        }
    } else {
        // Check if redis is already running
        let check_results = Command::new("docker")
            .arg("ps")
            .arg("-a")
            .arg("--filter")
            .arg("name=qdrant")
            .output()
            .await;

        match check_results {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if output_str.contains("qdrant") {
                    logging::info("Qdrant docker container already running").await;
                    return Ok(env_vars_to_return);
                }
            }
            Err(e) => {
                logging::error(&format!("Failed to check if qdrant is running: {}", e)).await;
                return Err(Box::new(e));
            }
        }
    }

    let start_results = Command::new("docker")
        .arg("run")
        .arg("-d")
        .arg("--name")
        .arg("qdrant")
        .arg("-p")
        .arg("6333:6333")
        .arg("qdrant/qdrant")
        .status()
        .await;

    match start_results {
        Ok(status) => {
            if status.success() {
                logging::info("Qdrant docker container started").await;
            } else {
                logging::error("Failed to start qdrant docker container").await;
                return Err(Box::<dyn Error>::from(
                    "Failed to start qdrant docker container",
                ));
            }
        }
        Err(e) => {
            logging::error(&format!("Failed to start qdrant: {}", e)).await;
            return Err(Box::new(e));
        }
    };

    Ok(env_vars_to_return)
}
