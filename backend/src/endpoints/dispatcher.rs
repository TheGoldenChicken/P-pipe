use aws_config::BehaviorVersion;
use rocket::{Build, Rocket, routes};
use rocket::{fairing::AdHoc, figment::Figment};
use std::env;

use super::challenges::{add_access_type, add_challenge, delete_challenge, destroy_challenges, get_challenges, regenerate_sts};
use super::common::run_migrations;
use super::requests::{
    add_request, answer_request_student, delete_completed_request, delete_request,
    destroy_completed_requests, destroy_requests, get_completed_requests, get_request_student,
    get_requests,
};
use super::scheduler::scheduler_fairing;
use super::transactions::{
    delete_transaction, destroy_transactions, get_completed_transactions, get_transactions,
};

pub fn rocket_from_config(figment: Figment) -> Rocket<Build> {
    let rocket_build = rocket::custom(figment)
        .attach(AdHoc::try_on_ignite("Database Pool", |rocket| async {
            let url = rocket.figment()
                .extract_inner::<String>("database_url")
                .or_else(|_| std::env::var("DATABASE_URL"))
                .expect("DATABASE_URL must be set (via env var or Rocket figment key 'database_url')");
            match sqlx::PgPool::connect(&url).await {
                Ok(pool) => Ok(rocket.manage(pool)),
                Err(e) => {
                    eprintln!("Failed to connect to database: {}", e);
                    Err(rocket)
                }
            }
        }))
        .attach(AdHoc::try_on_ignite("SQLx Migrations", run_migrations))
        .attach(AdHoc::try_on_ignite("AWS STS Client", |rocket| async {
            let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
            let client = aws_sdk_sts::Client::new(&config);
            Ok(rocket.manage(client))
        }))
        .mount(
            "/",
            routes![
                add_challenge,
                get_challenges,
                delete_challenge,
                destroy_challenges,
                add_access_type,
                regenerate_sts,
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
        eprintln!(
            "ATTACH_SCHEDULER either false, not set, or this is a test. No scheduler fairing attached"
        );
        rocket_build
    }
}
