use std::env;

use tokio::{fs::OpenOptions, io::AsyncWriteExt};

/// Checks that the PYTHONPATH is set, if not it will add it to the .env file
pub async fn check() {
    dotenv::dotenv().ok();
    // Check that PYTHONPATH is set
    if !env::var("PYTHONPATH").is_ok() {
        let current_path = env::current_dir().unwrap();
        let python_path = format!("PYTHONPATH={}/app", current_path.display());

        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(".env")
            .await
            .unwrap();

        // Append to .env file
        file.write_all(python_path.as_bytes()).await.unwrap();
        file.flush().await.unwrap();
        env::set_var(
            "PYTHONPATH",
            format!("{}/app", current_path.display().to_string()),
        );
    }
}
