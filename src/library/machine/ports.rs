use regex::Regex;

use crate::{
    library::{system::command, utils::logging},
    models::{
        config::{Config, Source},
        machine_state::MachineState,
    },
};
use std::collections::HashSet;

pub async fn replace_ports_in_text(machine_state: &MachineState, text: &str) -> String {
    let mut text = text.to_string();

    for (key, value) in machine_state.ports.iter() {
        text = text.replace(&format!("${{port.{}}}", key), value.to_string().as_str());
    }

    text
}

fn get_ports_needed_from_str(input: &str) -> Vec<String> {
    // Find any instances of '${port.<word>}' in the commands
    let mut ports_needed: Vec<String> = vec![];
    let regex = Regex::new(r"\$\{port\.\w+\}").unwrap();
    let ports = regex.find_iter(input);
    for port in ports {
        // Remove the ${port. and the }
        let port_name = port.as_str().replace("${port.", "").replace("}", "");
        ports_needed.push(port_name.to_string());
    }
    ports_needed
}

async fn get_ports_needed(config: &Config) -> Vec<String> {
    let mut ports_needed: Vec<String> = vec![];
    for service in &config.services {
        match &service.source {
            Source::CONTAINER(docker_service) => {
                if let Some(start_command) = &docker_service.start_command {
                    let ports = get_ports_needed_from_str(&start_command);
                    ports_needed.extend(ports);
                }

                if let Some(stop_command) = &docker_service.stop_command {
                    let ports = get_ports_needed_from_str(&stop_command);
                    ports_needed.extend(ports);
                }
            }
            Source::APP(app_service) => {
                let install_ports = get_ports_needed_from_str(&app_service.install_command);
                ports_needed.extend(install_ports);

                let health_check_ports =
                    get_ports_needed_from_str(&app_service.health_check_command);
                ports_needed.extend(health_check_ports);

                let start_ports = get_ports_needed_from_str(&app_service.start_command);
                ports_needed.extend(start_ports);

                if let Some(stop_command) = &app_service.stop_command {
                    let stop_ports = get_ports_needed_from_str(&stop_command);
                    ports_needed.extend(stop_ports);
                }

                if let Some(clean_command) = &app_service.clean_command {
                    let clean_ports = get_ports_needed_from_str(&clean_command);
                    ports_needed.extend(clean_ports);
                }
            }
        }
    }

    // Drop all duplicate port names
    let ports_needed: Vec<String> = ports_needed
        .into_iter()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    ports_needed
}

pub async fn get_ports(length: usize) -> Vec<i32> {
    // Start at 49000 and increment
    let mut i: i32 = 49000;
    let mut ports: Vec<i32> = vec![];
    while ports.len() < length as usize {
        let nc_result = command::run(format!("nc -zv 127.0.0.1 {}", i).as_str()).await;
        match nc_result {
            Ok(_) => {
                // Connection successful, port is in use
            }
            Err(_) => {
                // Connection failed, port is not in use
                ports.push(i);
            }
        }
        i += 1;
    }
    ports
}

pub async fn print_port_map_box(machine_state: &MachineState) {
    logging::nl().await;
    logging::print_color(logging::BG_MAGENTA, " Port map ").await;

    // We need to find the longest key so we can align the table
    let longest_name = machine_state.ports.keys().max_by_key(|key| key.len());
    let longest_name_len = longest_name.map_or(0, |key| key.len());

    let longest_port = machine_state
        .ports
        .values()
        .max_by_key(|value| value.to_string().len());
    let longest_port_len = longest_port.map_or(0, |value| value.to_string().len());

    let name_margin = "─".to_string().repeat(longest_name_len);
    let port_margin = "─".to_string().repeat(longest_port_len);

    // Using longer | character for sides: │
    logging::print_color(
        logging::NC,
        &format!("┌─{}─┬─{}─┐", name_margin, port_margin),
    )
    .await;
    logging::print_color(
        logging::NC,
        &format!(
            "│ {:<name_width$} │ {:<port_width$} │",
            "Name",
            "Port",
            name_width = longest_name_len,
            port_width = longest_port_len
        ),
    )
    .await;
    logging::print_color(
        logging::NC,
        &format!("├─{}─┼─{}─┤", name_margin, port_margin),
    )
    .await;

    // Sort by name
    let mut sorted_ports: Vec<_> = machine_state.ports.iter().collect();
    sorted_ports.sort_by_key(|(key, _)| *key);

    for (key, value) in sorted_ports {
        let parsed_name = format!("{:<width$}", key, width = longest_name_len);
        let parsed_port = format!("{:<width$}", value, width = longest_port_len);

        logging::print_color(
            logging::NC,
            &format!("│ {} │ {} │", parsed_name, parsed_port),
        )
        .await;
    }

    logging::print_color(
        logging::NC,
        &format!("└─{}─┴─{}─┘", name_margin, port_margin),
    )
    .await;
}

pub async fn produce_port_map(machine_state: &mut MachineState, config: &Config) {
    let mut ports_needed = get_ports_needed(config).await;

    // Drop any ports in the state that are not needed
    machine_state
        .ports
        .retain(|port, _| ports_needed.contains(port));

    // Drop any ports_needed that the state already has
    ports_needed.retain(|port| !machine_state.ports.contains_key(port));

    let ports = get_ports(ports_needed.len()).await;
    for (i, port) in ports_needed.iter().zip(ports.iter()) {
        machine_state.ports.insert(i.to_string(), *port);
    }

    print_port_map_box(machine_state).await;
}
