use crate::lib::utils::logging;
use futures::future::join_all;
use std::error::Error;
use tokio::process::Command;

pub async fn remove_containers(filter: &str) -> Result<(), Box<dyn Error>> {
    logging::info("Cleaning up docker containers").await;

    let containers_to_remove_vec = Command::new("docker")
        .args(&["ps", "-a", "--filter", filter, "-q"])
        .output()
        .await
        .expect("Failed to get docker container ids")
        .stdout;

    let containers_to_remove = String::from_utf8_lossy(&containers_to_remove_vec);

    let mut tasks = Vec::new();
    for container in containers_to_remove.lines() {
        let task = Command::new("docker")
            .args(&["rm", "-f", &container, "--force"])
            .status();
        tasks.push(task);
    }

    let results = join_all(tasks).await;
    for result in results {
        match result {
            Ok(_) => {}
            Err(e) => {
                logging::error(&format!("Failed to remove container: {}", e)).await;
                return Err(Box::new(e));
            }
        }
    }

    Ok(())
}
