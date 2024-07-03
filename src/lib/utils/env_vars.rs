use std::env;

use crate::lib::utils::logging;

pub async fn set_env_vars(env_vars: Vec<(String, String, String)>) {
    dotenv::dotenv().ok();

    logging::nl().await;
    logging::print_color(logging::BG_BLUE, "Environment variables").await;

    // Using longer | character for sides: │
    logging::print_color(
        logging::NC,
        "┌───────────────────────────┬──────────────────────┐",
    )
    .await;
    logging::print_color(
        logging::NC,
        "│ Key                       │ Source               │",
    )
    .await;
    logging::print_color(
        logging::NC,
        "├───────────────────────────┼──────────────────────┤",
    )
    .await;

    for (key, value, source) in &env_vars {
        let parsed_key = format!("{:<25}", key);
        let mut parsed_source = format!("{:<20}", source);

        // Check if the key is already set
        if env::var(key).is_ok() {
            parsed_source = format!("{:<20}", "local");
        } else {
            env::set_var(key, value);
        }

        logging::print_color(
            logging::NC,
            &format!("│ {} │ {} │", parsed_key, parsed_source),
        )
        .await;
    }

    logging::print_color(
        logging::NC,
        "└───────────────────────────┴──────────────────────┘",
    )
    .await;
}
