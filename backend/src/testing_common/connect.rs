use crate::dispatch::AccessBackend;
use crate::endpoints::dispatcher::rocket_from_config;
use rocket::local::asynchronous::Client;
use sqlx::postgres::PgConnectOptions;

// Below not needed; is automatically gotten by sqlx...
// TODO: IMPORTANT: Have a default kinda pg_connect_options returner for test scripts so they don't need to define it...

pub async fn async_client_from_pg_connect_options(pg_connect_options: PgConnectOptions) -> Client {
    let db_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        pg_connect_options.get_username(),
        "postgres",
        pg_connect_options.get_host(),
        pg_connect_options.get_port(),
        pg_connect_options.get_database().unwrap()
    );

    let figment = rocket::Config::figment().merge(("database_url", db_url));

    let client = Client::tracked(rocket_from_config(figment))
        .await
        .expect("invalid rocket instance");

    return client;
}

/// Like `async_client_from_pg_connect_options`, but with a caller-provided
/// access backend managed before ignite. The `S3 access backend` fairing yields
/// to it, so this never touches AWS — used by tests to inject a mock. The mock
/// type itself lives under `tests/` and is never part of this binary.
pub async fn async_client_with_access(
    pg_connect_options: PgConnectOptions,
    access: Box<dyn AccessBackend>,
) -> Client {
    let db_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        pg_connect_options.get_username(),
        "postgres",
        pg_connect_options.get_host(),
        pg_connect_options.get_port(),
        pg_connect_options.get_database().unwrap()
    );

    let figment = rocket::Config::figment().merge(("database_url", db_url));

    Client::tracked(rocket_from_config(figment).manage(access))
        .await
        .expect("invalid rocket instance")
}
