#[macro_use]
extern crate rocket;
use tokio_postgres::NoTls;
use rocket_cors::{ CorsOptions, AllowedOrigins };

mod endpoints;
use endpoints::dispatcher::{add_challenge, delete_challenge, get_challenges};
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
        .mount("/", routes![add_challenge, delete_challenge, get_challenges])
        .attach(cors)
}

