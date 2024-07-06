use crate::{
    lib::{system::bitwarden, utils::env_vars},
    models::config::{Config, Feature},
};

/// Checks that the features have all their prerequisites installed and enabled.
pub async fn check(config: &Config, secrets: &serde_json::Value) {
    for feature in &config.features {
        match feature {
            Feature::PYTHONPATH(_) => {}
            Feature::BITWARDEN => {
                env_vars::make_sure_exists(Some(secrets), "BWS_ACCESS_TOKEN").await;
                bitwarden::check_installation().await;
            }
        }
    }
}
