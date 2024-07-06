use futures::future::join_all;

use crate::library::system::{custom_app, docker};
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
}

pub async fn start_all(
    machine_state: &MachineState,
    config: &Config,
    clean_mode: bool,
    fail_fast: bool,
) {
    let mut tasks = vec![];

    for service in &config.services {
        let task = start(machine_state, config, &service, clean_mode, fail_fast);
        tasks.push(task);
    }
    join_all(tasks).await;
}
