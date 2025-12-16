use std::collections::HashMap;
use std::fmt;
use rocket::{http::Status, response::status::Custom};
use rocket::serde::{Deserialize, Serialize};
use sqlx::types::Json;
use rocket::serde::json::Value;

use crate::schemas::transaction::Transaction;
use crate::global_rng::global_rng;

use rand::seq::IteratorRandom;
use rand::Rng;

// TODO: Potentially add ColumnValue enum to control "accepted" types in requests...
// pub enum ColumnValue {
//     Str(String),
//     Num(i64),
//     Bool(bool),
// }



#[derive(sqlx::Type, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[sqlx(type_name = "request_status", rename_all = "snake_case")]
pub enum RequestStatus {
    Pending,
    Correct,
    PartialCorrect,
    Incorrect,
    SyntaxError,
    DeadlineExceeded
}

// Cannot map to a postgres type because it is a tuple kinda type! Be aware!
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum RequestType {
    DataValidation(DataValidationPayload),
    BatchPrediction(BatchPredictionPayload),
    CalculatedFeature(CalculatedFeaturePayload)
}

impl fmt::Display for RequestType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RequestType::DataValidation(_) => write!(f, "DataValidation"),
            RequestType::BatchPrediction(_) => write!(f, "BatchPrediction"),
            RequestType::CalculatedFeature(_) => write!(f, "CalculatedFeature"),
        }
    }
}

// TODO: Consider if we need to make it a kind of Vec<HashMap<i32, HashMap<String, serde_json::Value>>>
// To handle the case where users write "2: {data...}", or how they will indicate which part of the DataValidation
// goes to which part
// We can perhaps supply them an API
// Or we can always just have them include the ID of the data point or something...
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DataValidationPayload {
    pub items: Vec<i32>,
    pub count: Option<i32>,
}

impl DataValidationPayload {
    pub fn generate_from_transaction(tx: &Transaction) -> Result<Self, Custom<String>> {
        if let Some(range) = &tx.rows_to_push {
            let mut rng = global_rng();
            let request_size = rng.random_range(1..=(range[1] - range[0] + 1));
            let rows_to_check = (range[0]..=range[1]).choose_multiple(&mut rng, request_size as usize);
            Ok(Self {
                items: rows_to_check,
                count: Some(request_size)
            })
        } else {
            Err(Custom(Status::InternalServerError, "Given transaction has no rows to push!".to_string()))
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BatchPredictionPayload {
    pub items: Vec<HashMap<String, serde_json::Value>>,
    pub count: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CalculatedFeaturePayload {
    pub features_name: String,
    pub feature_information: String, // Information on how to calculate the features
    pub items: Vec<i32>,
    pub count: Option<i32>,
}


#[derive(Serialize, Deserialize, Clone, Debug, sqlx::FromRow)]
pub struct Request {
    pub id: Option<i32>,
    pub challenge_id: i32,
    pub created_at: Option<i64>,

    // Transaction info fields
    pub type_of_request: Json<RequestType>,
    pub expected_response: Json<RequestType>, // Kind of a monke solution, but technically we can just remap these...
    pub deadline: Option<i64>,
}

impl PartialEq for Request {
    fn eq(&self, other: &Self) -> bool {
        self.challenge_id == other.challenge_id
            && self.type_of_request == other.type_of_request
            && self.expected_response == other.expected_response
    }
}


#[derive(Serialize, Deserialize, Clone, Debug, sqlx::FromRow)]
pub struct CompletedRequest {
    pub id: Option<i32>,
    pub challenge_id: i32,
    pub created_at: Option<i64>,

    // Transaction info fields
    pub type_of_request: Json<RequestType>,
    pub expected_response: Json<RequestType>,
    pub deadline: Option<i64>,

    pub request_status: RequestStatus,
    pub submitted_at: Option<i64>,
    pub submitted_response: Option<Json<RequestType>>,
    pub judgement_message: Option<Json<Value>>
}

impl CompletedRequest {
    pub fn from_request(
        req: Request,
        status: RequestStatus,
        submitted_at: Option<i64>,
        submitted_response: Json<RequestType>,
        judgement_message: Option<Json<Value>>
    ) -> Self {
        CompletedRequest {
            id: req.id,
            challenge_id: req.challenge_id,
            created_at: req.created_at,
            type_of_request: req.type_of_request,
            expected_response: req.expected_response,
            deadline: req.deadline,
            request_status: status,
            submitted_at: submitted_at,
            submitted_response: Some(submitted_response),
            judgement_message,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use crate::schemas::challenge::ChallengeOptions;
    proptest! {
        #[test]
        fn test_data_val_payload_generation_range(start in any::<i32>(), len in 1..1000) {
            let end = start + len;
            let rows_to_push = vec![start, end];
            
            let transaction = Transaction {
                id: None,
                challenge_id: 0,
                created_at: None,
                scheduled_time: 1120,
                source_data_location: Some("../py_modules/tests/test_data/iris.csv".into()),
                dispatch_location: None,
                data_intended_location: "challenge_42_testingchallenge1".into(),
                data_intended_name: None,
                rows_to_push: Some(rows_to_push),
                access_bindings: None,
                challenge_options: Json(ChallengeOptions::default())
            };

            let generated_payload = DataValidationPayload::generate_from_transaction(&transaction)
                .expect("Could not generate DataValidationPayload from transaction");

            prop_assert!(
                generated_payload.items.iter().all(|&x| (start..=end).contains(&x)),
                "Items {:?} not all within {}..={}", generated_payload.items, start, end
            );
            
            let generated_payload_count = generated_payload.count.unwrap();
            let generated_payload_len = generated_payload.items.len() as i32;

            prop_assert!(
                generated_payload_count == generated_payload_len,
                "Generated payload count value: {} does not match its len of: {}", generated_payload_count, generated_payload_len
            )
        }
    }
    
}