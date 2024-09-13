use std::env;

use wm::library::utils::env_vars;

#[tokio::test]
async fn test_replace_env_vars() {
    env::set_var("TEST_ENV_VAR", "beautiful");
    env::set_var("TEST_ENV_VAR_2", "world");

    let test_string = "Hi there $TEST_ENV_VAR $TEST_ENV_VAR_2";

    let test_string_with_values = env_vars::replace_env_vars(&test_string).await;

    assert_eq!(test_string_with_values, "Hi there beautiful world");
}

#[tokio::test]
async fn test_replace_env_vars_brackets() {
    env::set_var("TEST_ENV_VAR", "beautiful");
    env::set_var("TEST_ENV_VAR_2", "world");

    let test_string = "Hi there ${TEST_ENV_VAR} ${TEST_ENV_VAR_2}";

    let test_string_with_values = env_vars::replace_env_vars(&test_string).await;

    assert_eq!(test_string_with_values, "Hi there beautiful world");
}
