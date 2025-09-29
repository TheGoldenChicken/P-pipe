#[macro_use] extern crate rocket;
mod endpoints;
use endpoints::dispatcher;

#[launch]
fn rocket() -> _ {
    dotenv::dotenv().ok();    
    let figment: rocket::figment::Figment = rocket::Config::figment();
    dispatcher::rocket_from_config(figment)
}


