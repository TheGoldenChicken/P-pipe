#[macro_use] extern crate rocket;
mod endpoints;
use endpoints::dispatcher;


// fn scheduler_fairing() -> AdHoc {
//     AdHoc::on_ignite("Transaction Scheduler", |rocket| async {
//         // We don't use Db<PgPool> here, since connection is only used inside of a request guard
//         let db = rocket.state::<dispatcher::Db>().expect("Db not initialized");
//         let pool = db.0.clone();
//         tokio::spawn(dispatcher::transaction_scheduler(pool.clone()));
//         rocket
//     })
// }


#[launch]
fn rocket() -> _ {
    dotenv::dotenv().ok();    
    let figment: rocket::figment::Figment = rocket::Config::figment();
    // TODO: Attach scheduler fairing here?
    dispatcher::rocket_from_config(figment)
}


