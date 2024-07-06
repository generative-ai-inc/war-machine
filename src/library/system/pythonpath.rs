use std::{env, path::PathBuf};

use tokio::{fs::OpenOptions, io::AsyncWriteExt};

use crate::library::utils::logging;

/// Checks that the PYTHONPATH is set, if not it will add it to the .env file
pub async fn check(env_file_path: &PathBuf, pythonpath_value: &PathBuf) {
    let absolute_pythonpath_value_result = pythonpath_value.canonicalize();

    let absolute_pythonpath_value = match absolute_pythonpath_value_result {
        Ok(absolute_pythonpath_value) => absolute_pythonpath_value,
        Err(_) => {
            logging::error(
                "Error setting PYTHONPATH through feature \"pythonpath\", please check your pythonpath_value in war_machine.toml actually exists",
            )
            .await;
            return;
        }
    };

    // Check that PYTHONPATH is set
    if !env::var("PYTHONPATH").is_ok() {
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(env_file_path)
            .await
            .unwrap();

        // Append to .env file
        file.write_all(format!("PYTHONPATH={}", absolute_pythonpath_value.display()).as_bytes())
            .await
            .unwrap();
        file.flush().await.unwrap();
        env::set_var(
            "PYTHONPATH",
            absolute_pythonpath_value.display().to_string(),
        );
    }
}
