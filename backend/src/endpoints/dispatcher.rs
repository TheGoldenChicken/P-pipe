use rocket::{Build, Rocket, routes}; // Have to do this as long as src/lib.rs contains `pub mod endpoints;`, as it breaks #[macro_use]
use rocket::{fairing::AdHoc, figment::Figment};
use rocket_db_pools::Database;
use std::env;

use crate::schemas::common::Db;

use super::challenges::{add_challenge, delete_challenge, destroy_challenges, get_challenges};
use super::common::run_migrations;
use super::scheduler::scheduler_fairing;
use super::transactions::{delete_transaction, destroy_transactions, get_transactions, get_completed_transactions};
use super::requests::{add_request, get_requests, delete_request, destroy_requests, get_request_student, answer_request_student, get_completed_requests, delete_completed_request, destroy_completed_requests};


pub fn rocket_from_config(figment: Figment) -> Rocket<Build> {
    let rocket_build = rocket::custom(figment)
        .attach(Db::init())
        .attach(AdHoc::try_on_ignite("SQLx Migrations", run_migrations))
        .mount(
            "/",
            routes![
                add_challenge,
                get_challenges,
                delete_challenge,
                destroy_challenges,
                get_transactions,
                get_completed_transactions,
                delete_transaction,
                destroy_transactions,
                add_request,
                get_requests,
                delete_request,
                destroy_requests,
                get_request_student,
                answer_request_student,
                get_completed_requests,
                delete_completed_request,
                destroy_completed_requests
            ],
        );

    let attach_scheduler = env::var("ATTACH_SCHEDULER")
        .map(|v| v == "true")
        .unwrap_or(false);
    
    // attaching a scheduler during testing usually breaks the testing process
    // TODO: Find a new way of attaching the scheduler fairing when running tests...
    if cfg!(not(test)) && attach_scheduler {
        println!("Attaching scheduler fairing");
        rocket_build.attach(scheduler_fairing())
    } else {
        eprintln!("ATTACH_SCHEDULER either false, not set, or this is a test. No scheduler fairing attached");
        rocket_build
    }
}
