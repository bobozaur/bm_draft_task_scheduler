use database::Database;
use rocket::{tokio::main, Config};
use sqlx::{pool::PoolOptions, PgPool};

#[main]
async fn main() {
    let conf = Config::figment();
    let pool = Database::connect(&conf, rocket::Config::LOG_LEVEL, 1)
        .await
        .unwrap();
}
