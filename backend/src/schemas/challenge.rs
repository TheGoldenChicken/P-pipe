use rocket::serde::{Deserialize, Serialize};
use sqlx::types::Json;

use super::common::{AccessBinding, DispatchTarget};
use super::request::RequestType;

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
    #[serde(default)]
    pub challenge_options: Json<ChallengeOptions>,
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
            challenge_options: Json(ChallengeOptions::default()),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, sqlx::FromRow, Debug, PartialEq)]
pub struct ChallengeOptions {
    pub possible_request_types: Option<Vec<RequestType>>, // Only allow DataValidation and BatchPrediction
    pub makes_requests_on_transaction_push: Option<bool>,
    // feature_calculation_requests: Option<Vec<DataValidationPayload>>, // An actual list of CalculatedFeaturePayload objects
    pub makes_requests_randomly: Option<bool>,
    pub min_time_between_requests: Option<u64>,
    pub max_time_between_requests: Option<u64>,
    pub requests_deadline: Option<i64>,
    pub validate_request_immediately_on_answer: Option<bool>,
    pub allow_retries_on_request: Option<bool>,
    pub return_completed_request_on_student_answer: Option<bool>,
    // pub random_time_between_releases: Option<bool>,
    // pub min_time_between_releases: Option<u64>,
    // pub max_time_between_releases: Option<u64>,
}

impl Default for ChallengeOptions {
    fn default() -> Self {
        Self {
            possible_request_types: None,
            makes_requests_on_transaction_push: None,
            makes_requests_randomly: None,
            min_time_between_requests: None,
            max_time_between_requests: None,
            requests_deadline: None,
            validate_request_immediately_on_answer: None,
            allow_retries_on_request: None,
            return_completed_request_on_student_answer: None,
        }
    }
}
