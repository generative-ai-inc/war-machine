use std::path::PathBuf;

use war_machine::lib::system::config;

#[tokio::test]
async fn test_parse_config() {
    let config_path = PathBuf::from("tests/assets/war_machine.toml");
    let config = config::parse(config_path).await;
    assert!(config.features.len() > 0);
    assert!(config.services.len() > 0);
    assert!(config.services[0].exposed_values.len() > 0);
}
