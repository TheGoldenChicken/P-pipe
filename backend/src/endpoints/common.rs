use rocket::{Build, Rocket, fairing}; // Have to do this as long as src/lib.rs contains `pub mod endpoints;`, as it breaks #[macro_use]
use rocket_db_pools::Database;
use std::env;

use crate::schemas::common::Db;

pub async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    let should_migrate = env::var("RUN_MIGRATIONS")
        .map(|v| v == "true")
        .unwrap_or(false);

    if !should_migrate {
        eprintln!("🔧 Skipping migrations due to config");
        return Ok(rocket);
    }

    match Db::fetch(&rocket) {
        Some(db) => match sqlx::migrate!("src/migrations").run(&**db).await {
            Ok(_) => Ok(rocket),
            Err(e) => {
                eprintln!("Failed to initialize SQLx database: {}", e);
                Err(rocket)
            }
        },
        None => Err(rocket),
    }
}
