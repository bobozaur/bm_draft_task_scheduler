mod config;
mod error;

use std::time::Duration;

use rocket::{
    config::LogLevel,
    fairing::{self, Fairing, Info, Kind},
    figment::{providers::Serialized, Figment},
    log::error_,
    Build, Rocket,
};
use sqlx::{pool::PoolOptions, postgres::PgConnectOptions, ConnectOptions, PgPool};

use self::{config::PoolConfig, error::InitPoolError};

/// Fairing for setting up database access in API endpoints.
//
// Manually implementing this because,
// while the `sqlx` dependency was recently updated in `rocket_db_pools`,
// there's not a new crate version released yet.
pub struct Database;

impl Database {
    /// Init database pool from configuration
    ///
    /// # Errors
    /// An error is returned if the connection pool could not be initialized.
    async fn init(rocket: &Rocket<Build>) -> Result<PgPool, InitPoolError> {
        // taken from rocket_db_pools::database
        let conf = rocket.figment();
        let workers: usize = conf
            .extract_inner(rocket::Config::WORKERS)
            .unwrap_or_else(|_| rocket::Config::default().workers);

        let max_connections = workers * 4;

        Self::connect(conf, rocket::Config::LOG_LEVEL, max_connections).await
    }

    pub async fn connect(
        conf: &Figment,
        log_level: &str,
        max_connections: usize,
    ) -> Result<PgPool, InitPoolError> {
        let enriched_conf = conf
            .focus("databases.devdb")
            .merge(Serialized::default("max_connections", max_connections))
            .merge(Serialized::default("acquire_timeout", 5));

        // taken from rocket_db_pools::pool
        let config = enriched_conf.extract::<PoolConfig>()?;
        let mut opts = config.url.parse::<PgConnectOptions>()?;

        let _ = opts.disable_statement_logging();
        if let Ok(level) = enriched_conf.extract_inner::<LogLevel>(log_level) {
            if !matches!(level, LogLevel::Normal | LogLevel::Off) {
                let _ = opts
                    .log_statements(level.into())
                    .log_slow_statements(level.into(), Duration::default());
            }
        }

        PoolOptions::new()
            .max_connections(config.max_connections)
            .acquire_timeout(Duration::from_secs(config.acquire_timeout))
            .idle_timeout(config.idle_timeout.map(Duration::from_secs))
            .min_connections(config.min_connections.unwrap_or_default())
            .connect_with(opts)
            .await
            .map_err(From::from)
    }
}

#[rocket::async_trait]
impl Fairing for Database {
    fn info(&self) -> Info {
        Info {
            name: stringify!(Database),
            kind: Kind::Ignite,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
        match Self::init(&rocket).await {
            Ok(pool) => Ok(rocket.manage(pool)),
            Err(err) => {
                error_!("failed to initialize database: {}", err);
                Err(rocket)
            }
        }
    }
}
