#[macro_use]
extern crate rocket;
use tokio_postgres::NoTls;
use rocket_cors::{ CorsOptions, AllowedOrigins };

mod endpoints;
use endpoints::dispatcher::{call_file_splitter, call_dispatcher};
use endpoints::table_initialization;

#[launch]
async fn rocket() -> _ {
    let (client, connection) = tokio_postgres
        ::connect("host=localhost user=postgres password=postgres dbname=postgres", NoTls).await
        .expect("Failed to connect to Postgres");

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Failed to connect to Postgres: {}", e);
        }
    });

    table_initialization::create_tables(&client).await;

    let cors: rocket_cors::Cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .to_cors()
        .expect("Error while building CORS");

    rocket
        ::build()
        .manage(client)
        .mount("/", routes![call_file_splitter, call_dispatcher])
        .attach(cors)
}