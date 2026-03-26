#[macro_use]
extern crate rocket;
use backend::endpoints::dispatcher::rocket_from_config;

#[launch]
fn rocket() -> _ {
    // Should include managing a client, possibly for each 3rd location
    // rocket::build()
        // .manage(client)  // stored once, available to all endpoints via &State<Client>

    dotenv::dotenv().ok();
    let figment: rocket::figment::Figment = rocket::Config::figment();
    rocket_from_config(figment)
}
