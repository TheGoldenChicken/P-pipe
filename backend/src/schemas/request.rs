use std::collections::HashMap;

use rocket::serde::{Deserialize, Serialize};
use sqlx::types::Json;

// TODO: Potentially add ColumnValue enum to control "accepted" types in requests...

// pub enum ColumnValue {
//     Str(String),
//     Num(i64),
//     Bool(bool),
// }



#[derive(sqlx::Type, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[sqlx(type_name = "request_status", rename_all = "snake_case")]
pub enum RequestStatus {
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

// #[derive(Serialize, Deserialize, Clone, Debug, sqlx::FromRow)]
// pub struct CompletedRequest {
//     pub id: Option<i32>,
//     pub challenge_id: i32,
//     pub created_at: Option<i64>,

//     // Transaction info fields
//     pub type_of_request: RequestType,
//     pub request_payload: ,
//     pub expected_response: ,
//     pub deadline: Option<i64>,

//     pub request_status: RequestStatus,
//     pub submitted_at: Option<i64>,
//     pub submitted_response: ,
// }

// TODO: Potentially implement from_request for CompletedRequest...