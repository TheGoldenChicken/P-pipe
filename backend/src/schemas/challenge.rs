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
