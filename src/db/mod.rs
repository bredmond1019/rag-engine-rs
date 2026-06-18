// File: src/db/mod.rs

use std::env;

use diesel::r2d2::{self, ConnectionManager, Pool};
use diesel::RunQueryDsl;
use diesel::{sql_query, PgConnection};

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub fn init_pool() -> DbPool {
    let database_url: String = get_database_url();
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder()
        .build(manager)
        // Startup-only: fail fast if the DB is unreachable or DATABASE_URL is malformed.
        .expect("Failed to create database connection pool; check DATABASE_URL and that Postgres is running")
}

pub fn get_database_url() -> String {
    // Startup-only: the service cannot run without a database.
    env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (see .env.example for the expected format)")
}

pub fn clear_all_tables(conn: &mut PgConnection) -> Result<(), diesel::result::Error> {
    sql_query("TRUNCATE TABLE articles, collections CASCADE").execute(conn)?;
    Ok(())
}
