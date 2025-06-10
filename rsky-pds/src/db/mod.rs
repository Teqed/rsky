use anyhow::Result;
use deadpool_diesel::sqlite::{Manager, Pool, Runtime};
use diesel::*;
use dotenvy::dotenv;
use std::env;

pub type DbConn = deadpool_diesel::Pool<
    deadpool_diesel::Manager<SqliteConnection>,
    deadpool_diesel::sqlite::Object,
>;

#[tracing::instrument(skip_all)]
/// Establish a connection to the database
/// Takes a database URL as an argument (like "sqlite://data/sqlite.db")
pub(crate) fn establish_pool(database_url: &str) -> Result<Pool> {
    tracing::debug!("Establishing database connection");
    let manager = Manager::new(database_url, Runtime::Tokio1);
    let pool = Pool::builder(manager)
        .max_size(8)
        .build()
        .expect("should be able to create connection pool");
    tracing::debug!("Database connection established");
    Ok(pool)
}

#[tracing::instrument(skip_all)]
pub fn establish_connection_for_sequencer() -> Result<Pool> {
    dotenv().ok();
    tracing::debug!("Establishing database connection for Sequencer");
    let database_url = env::var("DATABASE_URL").unwrap_or("".into());
    let result = establish_pool(&database_url)
        .map_err(|e| anyhow::anyhow!("Failed to establish connection pool: {}", e))?;

    Ok(result)
}
