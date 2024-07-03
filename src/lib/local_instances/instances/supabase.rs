use crate::lib::utils::{docker, logging};
use regex::Regex;
use std::error::Error;
use tokio::process::Command;

pub async fn start(clean_mode: bool) -> Result<Vec<(String, String)>, Box<dyn Error>> {
    if clean_mode {
        let remove_containers_result = docker::remove_containers("name=supabase*").await;
        if remove_containers_result.is_err() {
            logging::error(&format!(
                "Failed to remove supabase docker containers: {}",
                remove_containers_result.err().unwrap()
            ))
            .await;

            return Err(Box::<dyn Error>::from(
                "🛑 Supabase docker containers could not be removed, please remove them manually",
            ));
        }
    } else {
        let start_output = Command::new("supabase").arg("start").status().await;

        match start_output {
            Ok(_) => (),
            Err(e) => {
                logging::error(&format!("Failed to start supabase: {}", e)).await;
                return Err(Box::new(e));
            }
        };
    }

    let output = Command::new("supabase")
        .args(&["status", "-o", "env"])
        .output()
        .await;

    match output {
        Ok(output) => {
            let env_vars = String::from_utf8_lossy(&output.stdout);
            logging::info("Retrieved supabase environment variables").await;

            let re = Regex::new(r#"^([A-Z_]+)="(.+)""#).unwrap();

            let mut env_vars_to_return: Vec<(String, String)> = Vec::new();
            for line in env_vars.lines() {
                if let Some(caps) = re.captures(line) {
                    let key = &caps[1];
                    let value = &caps[2];
                    match key {
                        "ANON_KEY" => env_vars_to_return
                            .push(("SUPABASE_ANON_KEY".to_owned(), value.to_owned())),
                        "API_URL" => {
                            env_vars_to_return.push(("SUPABASE_URL".to_owned(), value.to_owned()))
                        }
                        "JWT_SECRET" => env_vars_to_return
                            .push(("SUPABASE_JWT_SECRET".to_owned(), value.to_owned())),
                        "SERVICE_ROLE_KEY" => env_vars_to_return
                            .push(("SUPABASE_SERVICE_ROLE_KEY".to_owned(), value.to_owned())),
                        _ => {}
                    }
                }
            }

            return Ok(env_vars_to_return);
        }
        Err(e) => {
            logging::error(&format!(
                "🛑 Failed to get supabase environment variables: {}",
                e
            ))
            .await;
            return Err(Box::new(e));
        }
    }
}
