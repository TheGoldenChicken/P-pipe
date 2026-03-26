use rocket::{Build, Rocket, fairing};
use std::env;

pub async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    let should_migrate = env::var("RUN_MIGRATIONS")
        .map(|v| v == "true")
        .unwrap_or(false);

    if !should_migrate {
        eprintln!("Skipping migrations due to config");
        return Ok(rocket);
    }

    match rocket.state::<sqlx::PgPool>() {
        Some(pool) => match sqlx::migrate!("src/migrations").run(pool).await {
            Ok(_) => Ok(rocket),
            Err(e) => {
                eprintln!("Failed to initialize SQLx database: {}", e);
                Err(rocket)
            }
        },
        None => Err(rocket),
    }
}
