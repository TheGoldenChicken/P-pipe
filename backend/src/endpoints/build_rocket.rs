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

// #[cfg(test)]
// mod tests {
//     use proptest::prelude::{proptest, prop, prop_assert_eq};
//     // TODO: Remove here, and only import specifcally what is asked for!
//     use super::*;

//     #[test]
//     // TODO: Consider naming convention here, should we really call it basic(), edge_case, invalid_input, etc.?
//     fn test_transactions_from_challenge_basic() {

//         let access_bindings = vec![
//             AccessBinding::S3(S3Binding { identity: "ec2userstuff".to_string(), bucket: "somebucket".to_string() }),
//             AccessBinding::Drive(DriveBinding { identity: "dderpson99@gmail.com".to_string(), folder_id: Some("abcd123".to_string()), user_permissions: "Read".to_string()})
//         ];

//         let challenge = Challenge {
//             id: Some(42),
//             challenge_name: "testingchallenge1".into(),
//             created_at: None,
//             init_dataset_location: "s3://bucket/data.csv".into(),
//             init_dataset_rows: 300,
//             init_dataset_name: Some("dataset".into()),
//             init_dataset_description: Some("desc".into()),
//             dispatches_to: vec![DispatchTarget::S3, DispatchTarget::Drive],
//             time_of_first_release: 1000,
//             release_proportions: vec![0.3, 0.4, 0.3],
//             time_between_releases: 60,
//             access_bindings: Some(sqlx::types::Json(access_bindings))
//         };

//         let transactions = transactions_from_challenge(challenge.clone()).expect("Could not generate transactions from challenge!");

//         assert_eq!(transactions.len(), 3, "Expected 3 transactions");

//         let expected = vec![
//             Transaction {
//                 id: None,
//                 challenge_id: 42,
//                 created_at: None,
//                 scheduled_time: 1000,
//                 source_data_location: challenge.init_dataset_location.clone(),
//                 dispatch_location:
//                 data_intended_location: "release_0".into(),
//                 rows_to_push: Some(vec![0, 30]),
//                 access_bindings: Some(sqlx::types::Json(access_bindings))
//             },
//             Transaction {
//                 id: None,
//                 challenge_id: 42,
//                 created_at: None,
//                 scheduled_time: 1060,
//                 source_data_location: challenge.init_dataset_location.clone(),
//                 data_intended_location: "release_1".into(),
//                 rows_to_push: Some(vec![30, 70]),
//                 access_bindings: Some(sqlx::types::Json(access_bindings))
//             },
//             Transaction {
//                 id: None,
//                 challenge_id: 42,
//                 created_at: None,
//                 scheduled_time: 1120,
//                 source_data_location: challenge.init_dataset_location.clone(),
//                 data_intended_location: "release_2".into(),
//                 rows_to_push: Some(vec![70, 100]),
//                 access_bindings: Some(sqlx::types::Json(access_bindings))
//             },
//         ];

//         assert_eq!(transactions, expected, "Transaction output mismatch");
//     }

//     proptest! {
//         #[test]
//         fn total_rows_pushed_is_100(proportions in prop::collection::vec(0.0..1.0, 1..10)) {
//             // In case of proportion of only 1, will create vector of normalized proportion of [1.0]!
//             let total: f64 = proportions.iter().sum();
//             let normalized: Vec<f64> = if total == 0.0 {
//                 vec![1.0] // fallback to avoid division by zero
//             } else {
//                 proportions.iter().map(|p| p / total).collect()
//             };

//             let challenge = Challenge {
//                 id: Some(1),
//                 name: "test".into(),
//                 created_at: None,
//                 init_dataset_location: "s3://bucket/data.csv".into(),
//                 init_dataset_rows: 100,
//                 init_dataset_name: None,
//                 init_dataset_description: None,
//                 time_of_first_release: 0,
//                 release_proportions: normalized.clone(),
//                 time_between_releases: 1,
//             };

//             let transactions = transactions_from_challenge(challenge).expect("Could not generate transactions from challenge");

//             let total_rows: i32 = transactions.iter()
//                 .map(|t| t.rows_to_push[1] - t.rows_to_push[0])
//                 .sum();
//             prop_assert_eq!(total_rows, 100, "Expected total rows to be 100, got {}", total_rows);
//         }
//     }

// }
