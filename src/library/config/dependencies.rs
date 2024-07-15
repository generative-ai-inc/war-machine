use crate::{library::utils::logging, models::config::Config};

pub async fn check(config: &Config) {
    for service in &config.services {
        for dependency in &service.depends_on {
            // Find the services this depends on
            let dependency_service = config.services.iter().find(|s| &s.name == dependency);

            match dependency_service {
                Some(dependency_service) => {
                    // Check that this service is not in the dependency_service's depends_on
                    if dependency_service.depends_on.contains(&service.name) {
                        logging::error(&format!(
                            "Circular dependency detected: {} -> {}",
                            service.name, dependency
                        ))
                        .await;
                    }
                }
                None => {
                    logging::error(&format!(
                        "Dependency {} not found for service {}",
                        dependency, service.name
                    ))
                    .await;
                }
            }
        }
    }
}
