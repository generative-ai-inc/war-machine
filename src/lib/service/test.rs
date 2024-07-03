use crate::lib::utils::logging::{self};
use std::path::PathBuf;
use tokio::process::Command;

pub async fn test(
    workers: i32,
    path: Option<PathBuf>,
    ignore: Option<PathBuf>,
    with_coverage: bool,
    verbose: bool,
    save_coverage: bool,
) {
    let mut main_command = format!("poetry run pytest -n {} --dist=loadfile", workers,);

    if let Some(path) = path {
        main_command.push_str(&format!(" {}", path.display()));
    }

    if let Some(ignore) = ignore {
        main_command.push_str(&format!(" --ignore {}", ignore.display()));
    }

    if verbose {
        main_command.push_str(" -s -vv");
    }

    if with_coverage {
        main_command.push_str(" --cov=app --cov-report=term-missing");
    }

    if save_coverage {
        main_command.push_str(" --cov-report=xml");
    }

    logging::nl().await;
    logging::print_color(logging::BG_GREEN, "Starting service").await;
    let child = Command::new("sh")
        .arg("-c")
        .arg(&main_command)
        .spawn()
        .expect("Failed to start main command");

    let pid = child.id().expect("Failed to get child pid");
    let handle = child.wait_with_output();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        logging::nl().await;
        logging::info("👍 Shutting down gracefully...").await;
        let result = Command::new("kill").arg(&pid.to_string()).status().await;

        match result {
            Ok(_) => {
                logging::info("✅ All processes have been terminated.").await;
                std::process::exit(0);
            }
            Err(e) => {
                logging::error(&format!("🛑 Failed to kill process: {}", e)).await;
                std::process::exit(1);
            }
        }
    });

    let _output = handle.await.expect("Failed to wait for main command");

    logging::info("✅ Tests completed").await;
    std::process::exit(0);
}
