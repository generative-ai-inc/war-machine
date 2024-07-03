use std::error::Error;

use futures::try_join;

use crate::lib::local_instances::instances::redis;
use crate::lib::local_instances::instances::supabase;

use super::instances::meilisearch;
use super::instances::qdrant;

pub async fn start_all_instances(
    clean_mode: bool,
) -> Result<Vec<(String, String)>, Box<dyn Error>> {
    let supabase_task = supabase::start(clean_mode);
    let redis_task = redis::start(clean_mode);
    let qdrant_task = qdrant::start(clean_mode);
    let meilisearch_task = meilisearch::start(clean_mode);

    let results = try_join!(supabase_task, redis_task, qdrant_task, meilisearch_task);

    match results {
        Ok((supabase_env_vars, redis_env_vars, qdrant_env_vars, meilisearch_env_vars)) => {
            // Merge into one vector
            let mut merged_env_vars: Vec<(String, String)> = Vec::new();
            merged_env_vars.extend(supabase_env_vars);
            merged_env_vars.extend(redis_env_vars);
            merged_env_vars.extend(qdrant_env_vars);
            merged_env_vars.extend(meilisearch_env_vars);

            Ok(merged_env_vars)
        }
        Err(e) => Err(e),
    }
}
