#[macro_use]
extern crate rocket;
use backend::endpoints::dispatcher::rocket_from_config;

#[launch]
fn rocket() -> _ {
    dotenv::dotenv().ok();
    let figment: rocket::figment::Figment = rocket::Config::figment();
    // TODO: Attach scheduler fairing here?
    rocket_from_config(figment)
}
