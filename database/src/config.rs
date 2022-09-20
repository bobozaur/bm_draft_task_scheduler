use rocket::serde::{Deserialize, Serialize};

/// Shamelessly copied from `rocket_db_pools` to work around the
/// semver issue between `rocket_db_pools` and `sqlx`.
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(crate = "rocket::serde")]
pub struct PoolConfig {
    pub url: String,
    pub min_connections: Option<u32>,
    pub max_connections: u32,
    pub acquire_timeout: u64,
    pub idle_timeout: Option<u64>,
}
