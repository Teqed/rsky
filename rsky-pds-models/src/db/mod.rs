use anyhow::Result;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use dotenvy::dotenv;
use rocket_sync_db_pools::database;
use std::env;
use std::fmt::{Debug, Formatter};

// const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

#[database("pg_db")]
pub struct DbConn(PgConnection);

impl Debug for DbConn {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[tracing::instrument(skip_all)]
pub fn establish_connection_for_sequencer() -> Result<PgConnection> {
    dotenv().ok();
    tracing::debug!("Establishing database connection for Sequencer");
    let database_url = env::var("DATABASE_URL").unwrap_or("".into());
    let mut result = PgConnection::establish(&database_url).map_err(|error| {
        let context = format!("Error connecting to {database_url:?}");
        anyhow::Error::new(error).context(context)
    })?;
    // result.run_pending_migrations(MIGRATIONS).unwrap();

    Ok(result)
}
