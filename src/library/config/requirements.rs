use crate::{
    library::system::{brew, docker, pipx, poetry, python},
    models::config::{Config, Requirement},
};

pub async fn check(config: &Config) {
    for requirement in &config.requirements {
        match requirement {
            Requirement::BREW => {
                brew::check_installation().await;
            }
            Requirement::DOCKER => {
                docker::check_installation().await;
            }
            Requirement::PIPX => {
                pipx::check_installation().await;
            }
            Requirement::PYTHON => {
                python::check_installation().await;
            }
            Requirement::POETRY => {
                poetry::check_installation().await;
            }
        }
    }
}
