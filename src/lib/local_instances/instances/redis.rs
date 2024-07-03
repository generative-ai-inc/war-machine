use crate::lib::utils::{docker, logging};
use std::error::Error;
use tokio::process::Command;

pub async fn start(clean_mode: bool) -> Result<Vec<(String, String)>, Box<dyn Error>> {
    let env_vars_to_return: Vec<(String, String)> = vec![
        ("REDIS_URL".to_owned(), "localhost".to_owned()),
        ("REDIS_PORT".to_owned(), "6379".to_owned()),
        ("REDIS_PASSWORD".to_owned(), "".to_owned()),
    ];

    if clean_mode {
        let remove_containers_result = docker::remove_containers("name=redis*").await;
        if remove_containers_result.is_err() {
            logging::error(&format!(
                "Failed to remove redis docker containers: {}",
                remove_containers_result.err().unwrap()
            ))
            .await;

            return Err(Box::<dyn Error>::from(
                "🛑 Redis docker containers could not be removed, please remove them manually",
            ));
        }
    } else {
        // Check if redis is already running
        let check_results = Command::new("docker")
            .arg("ps")
            .arg("-a")
            .arg("--filter")
            .arg("name=redis")
            .output()
            .await;

        match check_results {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if output_str.contains("redis") {
                    logging::info("Redis docker container already running").await;
                    return Ok(env_vars_to_return);
                }
            }
            Err(e) => {
                logging::error(&format!("Failed to check if redis is running: {}", e)).await;
                return Err(Box::new(e));
            }
        }
    }

    let start_results = Command::new("docker")
        .arg("run")
        .arg("-d")
        .arg("--name")
        .arg("redis")
        .arg("-p")
        .arg("6379:6379")
        .arg("--health-cmd=redis-cli ping")
        .arg("--health-interval=10s")
        .arg("--health-timeout=5s")
        .arg("--health-retries=3")
        .arg("--health-start-period=10s")
        .arg("redis")
        .arg("--notify-keyspace-events Ex")
        .status()
        .await;

    match start_results {
        Ok(status) => {
            if status.success() {
                logging::info("Redis docker container started").await;
            } else {
                logging::error("Failed to start redis docker container").await;
                return Err(Box::<dyn Error>::from(
                    "Failed to start redis docker container",
                ));
            }
        }
        Err(e) => {
            logging::error(&format!("Failed to start redis: {}", e)).await;
            return Err(Box::new(e));
        }
    };

    Ok(env_vars_to_return)
}
