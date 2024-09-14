use futures::future::join_all;
use std::error::Error;
use tokio::{process::Command, sync::watch};

use crate::{
    library::{machine, utils::logging},
    models::{
        config::{Config, ContainerSource},
        machine_state::MachineState,
    },
};

use super::command;

pub async fn login(registry: &str, username: &str, password: &str) -> Result<(), Box<dyn Error>> {
    let login_result = command::run(
        format!(
            "echo {} | docker login {} --username {} --password-stdin",
            password, registry, username
        )
        .as_str(),
    )
    .await;

    match login_result {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::from(e)),
    }
}

pub async fn logout(registry: &str) -> Result<(), Box<dyn Error>> {
    let logout_result = command::run(format!("docker logout {}", registry).as_str()).await;

    match logout_result {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::from(e)),
    }
}

pub async fn create_network(name: &str) -> Result<(), Box<dyn Error>> {
    let existing_networks_result = command::run("docker network ls --format '{{.Name}}'").await;

    let network_exists = match existing_networks_result {
        Ok(output) => output.contains(name),
        Err(e) => {
            logging::error(&format!("Failed to get existing networks: {}", e)).await;
            return Err(Box::from(e));
        }
    };

    if network_exists {
        return Ok(());
    }

    let command = format!("docker network create {}", name);

    let create_network_result = command::run(command.as_str()).await;

    match create_network_result {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::from(e)),
    }
}

pub async fn pull_image(registry: &str, image: &str, tag: &str) -> Result<(), Box<dyn Error>> {
    let logout_result =
        command::spawn(format!("docker pull {}/{}:{}", registry, image, tag).as_str()).await;

    match logout_result {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::from(e)),
    }
}

pub async fn start_container(name: &str, source: &ContainerSource) -> Result<(), Box<dyn Error>> {
    let (tx, mut rx) = watch::channel(false);

    let child = Command::new("docker")
        .arg("run")
        .arg("-d")
        .arg("--name")
        .arg(name)
        .arg(&source.image)
        .spawn()
        .expect("Failed to start docker container");

    let pid: u32 = child.id().expect("Failed to get command pid");
    let handle = child.wait_with_output();

    let install_handle = tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    logging::nl().await;
                    logging::info("🟨 Cancelling command...").await;
                    let result = Command::new("kill").arg(&pid.to_string()).status().await;

                    match result {
                        Ok(_) => {
                            logging::info("✅ Command has been terminated.").await;
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
                return Ok(());
            } else {
                return Err(Box::from(String::from_utf8_lossy(&output.stdout)));
            }
        }
        Err(e) => return Err(Box::from(format!("Failed to get command status: {}", e))),
    }
}

pub async fn remove_containers(filter: &str) -> Result<(), Box<dyn Error>> {
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

/// Replaces the placeholders in the command with the actual values
pub async fn replace_placeholders(
    machine_state: &MachineState,
    config: &Config,
    command: &String,
    name: &str,
    source: &ContainerSource,
) -> String {
    let mut new_command = command.to_string();
    new_command = new_command.replace("${machine_name}", &config.machine_name);
    new_command = new_command.replace("${service.name}", name);
    new_command = new_command.replace("${service.source.image}", &source.image);
    new_command = new_command.replace("${service.source.tag}", &source.tag);
    new_command = new_command.replace("${service.source.registry}", &source.registry);
    new_command = machine::ports::replace_ports_in_text(&machine_state, &new_command).await;
    new_command
}

pub async fn clean_service(config: &Config, name: &str, fail_fast: bool) {
    let remove_containers_result =
        remove_containers(&format!("name=^{}-{}$", config.machine_name, name)).await;

    match remove_containers_result {
        Ok(_) => {
            logging::info(&format!("⚠️  {} docker container removed", name)).await;
        }
        Err(e) => {
            logging::error(&format!(
                "Failed to remove {} docker containers: {}",
                name, e
            ))
            .await;

            if fail_fast {
                std::process::exit(1);
            }
        }
    }
}

pub async fn start_service(
    machine_state: &MachineState,
    config: &Config,
    name: &str,
    source: &ContainerSource,
    clean_mode: bool,
    fail_fast: bool,
) {
    let start_command = if let Some(start_command) = &source.start_command {
        Some(replace_placeholders(machine_state, config, start_command, name, source).await)
    } else {
        None
    };

    if clean_mode {
        clean_service(config, name, fail_fast).await;
    } else {
        // Check if redis is already running
        let check_results = Command::new("docker")
            .arg("ps")
            .arg("--filter")
            .arg(&format!("name=^{}-{}$", config.machine_name, name))
            .output()
            .await;

        match check_results {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if output_str.contains(&format!("{}", name)) {
                    logging::info(&format!("✅ {} is running", name)).await;
                    return;
                }
            }
            Err(e) => {
                logging::error(&format!("🛑 Failed to check if {} is running: {}", name, e)).await;

                if fail_fast {
                    std::process::exit(1);
                }
            }
        }
    }

    let pull_image_result = pull_image(&source.registry, &source.image, &source.tag).await;
    if pull_image_result.is_err() {
        logging::error(&format!(
            "🛑 Failed to pull {}/{}:{}",
            source.registry, source.image, source.tag
        ))
        .await;

        logging::error(&pull_image_result.err().unwrap().to_string()).await;

        std::process::exit(1);
    }

    let start_results;

    if let Some(start_command) = start_command {
        start_results = command::spawn(&start_command).await;
    } else {
        start_results = start_container(name, source).await;
    }

    match start_results {
        Ok(_) => {
            logging::info(&format!("🚀 started {}", name)).await;
        }
        Err(e) => {
            logging::error(&format!("🛑 Failed to start {}", name)).await;
            logging::error(&e.to_string()).await;

            if fail_fast {
                std::process::exit(1);
            }
        }
    }
}

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

pub async fn check_installation() {
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
