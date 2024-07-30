use serde_json::json;
use wm::library::utils::bitwarden;

#[tokio::test]
async fn test_bitwarden_parse_env_vars() {
    dotenv::dotenv().ok();

    let env_vars = bitwarden::get_env_variables(&json!({})).await;

    let env_vars_str = serde_json::to_string_pretty(&env_vars).unwrap();
    println!("{}", env_vars_str);
}
