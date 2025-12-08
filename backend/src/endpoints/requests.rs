use rocket::serde::json::{Json, Value, json};
use rocket::{delete, get, post, put}; // Have to do this as long as src/lib.rs contains `pub mod endpoints;`, as it breaks #[macro_use]
use rocket::{http::Status, response::status::Custom};
use rocket_db_pools::Connection;
use sqlx::types::Json as DbJson;
use crate::schemas::common::Db;
use crate::schemas::request::{Request, RequestType, CompletedRequest, RequestStatus};
use std::mem::discriminant;
use chrono::Utc;
use std::process::Command;
use std::path::Path;

// For transactions!
use sqlx::Acquire;

// Admin endpoints

#[get("/api/requests")]
pub async fn get_requests(
    mut db: Connection<Db>,
) -> Result<Json<Vec<Request>>, Custom<String>> {
    let requests = sqlx::query_as!(
        Request,
        r#"
        SELECT
            id,
            challenge_id,
            created_at,
            type_of_request as "type_of_request: DbJson<RequestType>",
            expected_response as "expected_response: DbJson<RequestType>",
            deadline
        FROM 
            requests
        "#
    )
    .fetch_all(&mut **db)
    .await
    .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    Ok(Json(requests))
}

// TODO IMPORTANT: Add check to ensure that expected_response type matches type_of_request type!
#[post("/api/requests", data = "<request>")]
pub async fn add_request(
    mut db: Connection<Db>,
    request: Json<Request>,
) -> Result<Json<Vec<Request>>, Custom<String>> {
    let _inserted = sqlx::query_as!(
        Request,
        r#"
        INSERT INTO requests
        (challenge_id, type_of_request, expected_response, deadline)
        VALUES ($1, $2, $3, $4)
        RETURNING
            id,
            challenge_id,
            created_at,
            type_of_request as "type_of_request: DbJson<RequestType>",
            expected_response as "expected_response: DbJson<RequestType>",
            deadline
        "#,
        request.challenge_id,
        request.type_of_request as _,
        request.expected_response as _,
        request.deadline
    )
    .fetch_one(&mut **db)
    .await
    .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    get_requests(db).await
}

#[delete("/api/requests/<id>")]
pub async fn delete_request(mut db: Connection<Db>, id: i32) -> Result<Status, Custom<String>> {
    sqlx::query!("DELETE FROM requests WHERE id = $1", id)
        .execute(&mut **db)
        .await
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    Ok(Status::NoContent)
}

#[delete("/api/requests")]
pub async fn destroy_requests(mut db: Connection<Db>) -> Result<(), Custom<String>> {
    sqlx::query!("DELETE FROM requests")
        .execute(&mut **db)
        .await
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;
    Ok(())
}

#[get("/api/completed_requests")]
pub async fn get_completed_requests(
    mut db: Connection<Db>,
) -> Result<Json<Vec<CompletedRequest>>, Custom<String>> {
    let completed_requests = sqlx::query_as!(
        CompletedRequest,
        r#"
        SELECT
            id,
            challenge_id,
            created_at,
            type_of_request as "type_of_request: DbJson<RequestType>",
            expected_response as "expected_response: DbJson<RequestType>",
            deadline,
            request_status as "request_status: RequestStatus",
            submitted_at,
            submitted_response as "submitted_response: DbJson<RequestType>",
            judgement_message as "judgement_message: DbJson<Value>"
        FROM 
            completed_requests
        "#
    )
    .fetch_all(&mut **db)
    .await
    .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    Ok(Json(completed_requests))
}


#[delete("/api/completed_requests/<id>")]
pub async fn delete_completed_request(mut db: Connection<Db>, id: i32) -> Result<Status, Custom<String>> {
    sqlx::query!("DELETE FROM completed_requests WHERE id = $1", id)
        .execute(&mut **db)
        .await
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    Ok(Status::NoContent)
}

#[delete("/api/completed_requests")]
pub async fn destroy_completed_requests(mut db: Connection<Db>) -> Result<(), Custom<String>> {
    sqlx::query!("DELETE FROM completed_requests")
        .execute(&mut **db)
        .await
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;
    Ok(())
}


// Student / User endpoints

#[get("/api/requests/<challenge_id>")]
pub async fn get_request_student(
    mut db: Connection<Db>,
    challenge_id: i32
) -> Result<Json<rocket::serde::json::Value>, Custom<String>> {
    let requests = sqlx::query_as!(
        Request,
        r#"
        SELECT
            id,
            challenge_id,
            created_at,
            type_of_request as "type_of_request: DbJson<RequestType>",
            expected_response as "expected_response: DbJson<RequestType>",
            deadline
        FROM 
            requests
        WHERE
            challenge_id = $1
        "#,
        &challenge_id
    )
    .fetch_all(&mut **db)
    .await
    .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    let number_requests = requests.len();

    // TODO: Potentially add the TYPE of the expected response here as a help to the students...
    let response: Vec<Value> = requests.into_iter().map(|r| {
        json!({
            "request_id": r.id,
            "challenge_id": r.challenge_id,
            "type_of_request": r.type_of_request,
            "created_at": r.created_at,
            "deadline": r.deadline,
        })
    }).collect();

    let final_response = json!({
        "requests": response,
        "message": format!("Found {} requests for challenge with id {}. You can answer each these by calling the PUT endpoint at /api/requests/<challenge_id>/<request_id> and submitting the answers in a json", number_requests, challenge_id) 
    });

    Ok(Json(final_response))
}

// TODO: Could potentially also be done easier... just check if the RequestType of request matches RequestType of expected_response...
// Potentially also add helper function to give information on how the response schema is supposed to be for a given request 
fn _check_request_type_match(
    request: &RequestType,
    response: &RequestType
) -> Result<(), String> {
    match (request, response) {
        (RequestType::DataValidation(_), RequestType::BatchPrediction(_)) => Ok(()),
        (RequestType::BatchPrediction(_),RequestType::DataValidation(_)) => Ok(()),
        // Do not support any other types of answered requests right now
        _ => Err(format!("Wrong response to request_type {}. ", request)) 
    }
}

// Student answer request from their end:
// 1. Validate that the json they have submitted fits with RequestType
// 2. Select relevant request from id

#[put("/api/requests/<challenge_id>/<request_id>", data = "<response>")]
pub async fn answer_request_student(
    mut db: Connection<Db>,
    challenge_id: i32,
    request_id: i32,
    // TODO: Potentially use Json<Value> and manually deserialize to return more... holistic error messages to users...
    response: Json<RequestType>
) -> Result<Json<CompletedRequest>, Custom<String>> {

    let submitted_at = Utc::now().timestamp_millis();
    let request = sqlx::query_as!(
        Request,
        r#"
        SELECT
            id,
            challenge_id,
            created_at,
            type_of_request as "type_of_request: DbJson<RequestType>",
            expected_response as "expected_response: DbJson<RequestType>",
            deadline
        FROM 
            requests
        WHERE
            id = $1 AND challenge_id = $2
        "#,
        &request_id,
        &challenge_id
    )
    .fetch_one(&mut **db)
    .await
    .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    // Ensure the response type user submitted matches expected type
    // check_request_type_match(&*request.type_of_request, &*response)
    // TODO: Ensure the correct error is mapped here, it should be a user-fucked-up kind of error
    // .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;
    if discriminant(&request.expected_response.0) != discriminant(&response.0) {
        return Err(
                Custom(Status::NotAcceptable,
                format!("Wrong response {} to request_type {}. Expected of type {}", *response, *request.type_of_request, *request.expected_response))
            )
    }

    // Assuming correct type is there, we go on to creating a CompletedRequest
    let completed_request = CompletedRequest::from_request(request, RequestStatus::Pending, Some(submitted_at), DbJson(response.clone().into_inner()), None);

    let completed_request_string = serde_json::to_string(&completed_request)
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    // TODO: Async this python bitch!
    // TODO: Move these (should be constant variables) into places that make sense?
    let python_path = Path::new("../.venv/bin/python");
    let script_path = Path::new("../py_modules/judgement/judge_cli.py");

    // TODO: Add catcher in case exit code is 1 or 2, in this case WE (not the student), fucked up in some way
    let output = Command::new(python_path)
        .arg(script_path)
        .arg("--completed_request")
        .arg(completed_request_string)
        .output()
        .map_err(|e| Custom(
        Status::InternalServerError,
        format!("Failed calling Python script with error {:?}", e),
    ))?;  

    let code = output.status.code().unwrap_or(-1);
    let stdout = &String::from_utf8_lossy(&output.stdout);
    
    // TODO: remove Debug info here...
    // println!("Exit code: {}, stdout: {}", code, stdout);
    // let stderr = &String::from_utf8_lossy(&output.stderr);
    // println!("Stderr {}", stderr);
    
    let parsed_stdout = serde_json::from_str::<Value>(&stdout)
        .map_err(|e| Custom(
            Status::InternalServerError,
            format!("Failed to parse JSON from Python output: {:?}", e),
        ))?;

    let completed_request_status: RequestStatus = match code {
        0 => RequestStatus::Correct,
        4 => RequestStatus::PartialCorrect,
        5 => RequestStatus::Incorrect,
        _ => RequestStatus::SyntaxError, // Should never happen here, fr fr
    };

    // We use transactions to prevent us from removing requests that don't go through
    let mut tx = db.begin()
        .await
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    sqlx::query!("DELETE FROM requests WHERE id = $1", request_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;


    let inserted = sqlx::query_as!(
        CompletedRequest,
        r#"
        INSERT INTO completed_requests
        (challenge_id, type_of_request, expected_response, deadline, request_status, submitted_at, submitted_response, judgement_message)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING
            id,
            challenge_id,
            created_at,
            type_of_request as "type_of_request: DbJson<RequestType>",
            expected_response as "expected_response: DbJson<RequestType>",
            deadline,
            request_status as "request_status: RequestStatus",
            submitted_at,
            submitted_response as "submitted_response: DbJson<RequestType>",
            judgement_message as "judgement_message: DbJson<Value>"
        "#,
        completed_request.challenge_id,
        completed_request.type_of_request as _,
        completed_request.expected_response as _,
        completed_request.deadline,
        completed_request_status as RequestStatus,
        completed_request.submitted_at,
        completed_request.submitted_response as _, 
        parsed_stdout,
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    // Only commit when we know both have gone through...
    tx.commit().await.map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    // Finally, tell user the result of the response...
    // Potentially return the full response they created if correct, and an error if it is not...
    Ok(Json(inserted))
}