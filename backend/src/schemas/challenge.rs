use rocket::serde::{Deserialize, Serialize};
use sqlx::types::Json;

use super::common::{AccessBinding, DispatchTarget};

#[derive(Serialize, Deserialize, Clone, sqlx::FromRow, Debug, PartialEq)]
pub struct Challenge {
    // "Bookkeeping fields"
    pub id: Option<i32>,
    pub challenge_name: String,
    pub created_at: Option<i64>,
    pub init_dataset_location: String,
    pub init_dataset_rows: i32,
    pub init_dataset_name: Option<String>,
    pub init_dataset_description: Option<String>,

    // Option fields\JsonDb
    pub dispatches_to: Vec<DispatchTarget>,
    pub time_of_first_release: i64,
    pub release_proportions: Vec<f64>,
    pub time_between_releases: i64,

    pub access_bindings: Option<Json<Vec<AccessBinding>>>,
}

impl Default for Challenge {
    fn default() -> Self {
        Challenge {
            id: None,
            challenge_name: "default-challenge".to_string(),
            created_at: None,
            init_dataset_location: "s3://bucket/default.csv".to_string(),
            init_dataset_rows: 100,
            init_dataset_name: None,
            init_dataset_description: None,
            dispatches_to: vec![DispatchTarget::S3],
            time_of_first_release: chrono::Utc::now().timestamp(),
            release_proportions: vec![1.0],
            time_between_releases: 1000, // 1 day in seconds
            access_bindings: None,
        }
    }
}
