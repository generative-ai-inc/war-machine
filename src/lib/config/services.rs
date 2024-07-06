use futures::future::join_all;

use crate::lib::system::{custom_app, docker};
use crate::models::config::{Config, Service, Source};

pub async fn start(service: &Service, clean_mode: bool, fail_fast: bool) {
    match &service.source {
        Source::CONTAINER(container_source) => {
            docker::start_service(&service.name, container_source, clean_mode, fail_fast).await;
        }
        Source::APP(app_source) => {
            custom_app::start_service(&service.name, app_source, clean_mode, fail_fast).await;
        }
    }
}

pub async fn start_all(config: &Config, clean_mode: bool, fail_fast: bool) {
    let mut tasks = vec![];

    for service in &config.services {
        let task = start(&service, clean_mode, fail_fast);
        tasks.push(task);
    }
    join_all(tasks).await;
}
