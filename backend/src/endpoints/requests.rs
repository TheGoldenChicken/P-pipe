use rocket::serde::json::Json;
use rocket::{delete, get, post}; // Have to do this as long as src/lib.rs contains `pub mod endpoints;`, as it breaks #[macro_use]
use rocket::{http::Status, response::status::Custom};
use rocket_db_pools::Connection;
use sqlx::types::Json as DbJson;

use crate::schemas::common::Db;
use crate::schemas::request::{Request, RequestType};


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
pub async fn delete_requests(mut db: Connection<Db>, id: i32) -> Result<Status, Custom<String>> {
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
