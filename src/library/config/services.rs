use std::collections::HashMap;

use futures::future::join_all;

use crate::library::commands::prepare::get_exposed_variables;
use crate::library::system::{custom_app, docker};
use crate::library::utils::env_vars;
use crate::models::config::{Config, Service, Source};
use crate::models::machine_state::MachineState;

pub async fn start(
    machine_state: &MachineState,
    config: &Config,
    service: &Service,
    clean_mode: bool,
    fail_fast: bool,
) {
    match &service.source {
        Source::CONTAINER(container_source) => {
            docker::start_service(
                machine_state,
                config,
                &service.name,
                container_source,
                clean_mode,
                fail_fast,
            )
            .await;
        }
        Source::APP(app_source) => {
            custom_app::start_service(
                machine_state,
                config,
                &service.name,
                app_source,
                clean_mode,
                fail_fast,
            )
            .await;
        }
    }

    // Set the available_before_start=false variables
    let exposed_values =
        get_exposed_variables(&machine_state, &service.exposed_values, false).await;

    let mut env_vars = vec![];
    for (key, value) in exposed_values {
        env_vars.push((key, value, "war machine".to_string()));
    }

    env_vars::set(&env_vars).await;
}

pub async fn start_all(
    machine_state: &MachineState,
    config: &Config,
    clean_mode: bool,
    fail_fast: bool,
) {
    let mut tasks = vec![];

    let mut leftover_services = HashMap::new();

    // First start the services without any dependencies
    for service in &config.services {
        if service.depends_on.is_empty() {
            let task = start(machine_state, config, &service, clean_mode, fail_fast);
            tasks.push(task);
        } else {
            leftover_services.insert(service.name.clone(), service);
        }
    }

    // We can always start the services without dependencies asynchronously
    join_all(tasks).await;
    // Now we loop over the services that have dependencies and try to start them asynchronously if we can
    // Dependencies configuration should have already checked for:
    // - Circular dependencies
    // - Missing dependencies
    // - Missing services
    // So this should never loop infinitely
    while !leftover_services.is_empty() {
        let mut new_tasks = vec![];
        let mut started_services = vec![];
        for (service_name, service) in &leftover_services {
            // We need to check if all the dependencies are started.
            // If they are not in the leftover_services hashmap, they are already started
            if service
                .depends_on
                .iter()
                .all(|dependency| !leftover_services.contains_key(dependency))
            {
                let task = start(machine_state, config, &service, clean_mode, fail_fast);
                new_tasks.push(task);
                started_services.push(service_name.clone());
            }
        }
        for service_name in started_services {
            leftover_services.remove(&service_name);
        }
        join_all(new_tasks).await;
    }
}
