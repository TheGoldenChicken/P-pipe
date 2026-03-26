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
