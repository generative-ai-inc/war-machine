use tokio::join;

use super::{brew, docker_cli, pipx, supabase_cli};

/// Takes care of checking if the system is setup correctly for the runner to run.
/// This includes checking the following:
/// - Docker is installed and running
/// - Supabase CLI is installed
pub async fn check() {
    brew::check().await;
    docker_cli::check().await;

    // We can run these in parallel
    join!(supabase_cli::check(), pipx::check());
}
