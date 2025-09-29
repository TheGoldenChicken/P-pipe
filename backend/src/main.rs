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

// use rocket::fairing::AdHoc;
// use rocket::local::blocking::Client;
// use rocket::serde::{Serialize, Deserialize};
// use rocket::http::Status;

// fn test(base: &str, stage: AdHoc) {
//     // TODO; remove this, we should just add base through the fucking tests written below...
//     let challenges_base  = "/api/challenges";
//     // NOTE: If we had more than one test running concurrently that dispatches
//     // DB-accessing requests, we'd need transactions or to serialize all tests.
//     let client = Client::tracked(rocket::build().attach(stage)).unwrap();
    
//     // Clear everything from the database.
//     assert_eq!(client.delete(challenges_base).dispatch().status(), Status::Ok);




//     // assert_eq!(client.get(base).dispatch().into_json::<Vec<i64>>(), Some(vec![]));
// }



