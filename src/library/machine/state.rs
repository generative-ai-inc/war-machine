use std::collections::HashMap;
use std::path::PathBuf;

use tokio::fs::OpenOptions;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::models::config::Config;
use crate::models::machine_state::MachineState;

use super::ports::produce_port_map;

pub async fn get_machine_state() -> MachineState {
    let file_read_result = OpenOptions::new()
        .read(true)
        .open(PathBuf::from(".war_machine/state.json"))
        .await;

    let state = match file_read_result {
        Ok(mut file) => {
            let mut buffer = String::new();
            file.read_to_string(&mut buffer).await.unwrap();
            let state: MachineState = serde_json::from_str(&buffer).unwrap();
            file.flush().await.unwrap();
            state
        }
        Err(_) => MachineState {
            containers: HashMap::new(),
            ports: HashMap::new(),
        },
    };

    state
}

pub async fn save_machine_state(state: &MachineState) {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(PathBuf::from(".war_machine/state.json"))
        .await
        .unwrap();

    let contents = serde_json::to_string(&state).unwrap();
    file.write_all(contents.as_bytes()).await.unwrap();
    file.flush().await.unwrap();
}

pub async fn create_war_machine_dir(path: &PathBuf) {
    std::fs::create_dir_all(path).unwrap();

    // Create a .gitignore file to ignore the .war_machine dir
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(path.join(".gitignore"))
        .await
        .unwrap();

    // Ignore all files in the .war_machine dir
    file.write_all(b"*\n").await.unwrap();
    file.flush().await.unwrap();
}

/// Checks if the .war_machine dir exists, if not it creates it
pub async fn check(config: &Config, clean_mode: bool) -> MachineState {
    let war_machine_dir = std::env::current_dir().unwrap().join(".war_machine");
    if !war_machine_dir.exists() {
        create_war_machine_dir(&war_machine_dir).await;
    }

    let mut machine_state = get_machine_state().await;

    if clean_mode {
        machine_state.ports.clear();
    }

    produce_port_map(&mut machine_state, config).await;

    save_machine_state(&machine_state).await;
    machine_state
}
