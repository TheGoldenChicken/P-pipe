#[macro_use] extern crate rocket;
use std::env;
mod endpoints;
use endpoints::dispatcher;

// #[cfg(test)] mod tests;

// use rocket::response::Redirect;

// #[get("/")]
// fn index() -> Redirect {
//     Redirect::to(uri!("/sqlx", sqlx::list()))
// }

#[launch]
fn rocket() -> _ {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("Expected DATABASE_URL in the environment"); 
    rocket::build()
        // .mount("/", routes![index])
        .attach(dispatcher::stage())
}