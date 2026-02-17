use rocket::http::Status;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::types::Json as DbJson;

use backend::endpoints::scheduler::process_request_with_transaction;
use backend::schemas::request::{Request, RequestType};
use backend::testing_common::connect::async_client_from_pg_connect_options;
use backend::testing_common::instances::{batch_prediction_instance, transaction_instance};
use backend::schemas::request::{CompletedRequest, RequestStatus};

// AVAST!
// MOST TESTS WILL FAIL IF YOU DO NOT USE --features deterministic

// TODO: See if we can't make a way to resuse functionality... right now both tests use a lot of the same code...
#[sqlx::test(migrations = "src/migrations")]
async fn answer_request_correct(
    _: PgPoolOptions,
    pg_connect_options: PgConnectOptions,
) -> sqlx::Result<()> {
    let client = async_client_from_pg_connect_options(pg_connect_options.clone()).await;
    let pool: sqlx::Pool<sqlx::Postgres> = PgPoolOptions::new()
        .connect_with(pg_connect_options.clone())
        .await?;

    sqlx::query!("ALTER TABLE requests DISABLE TRIGGER ALL;")
        .execute(&pool)
        .await?;
    sqlx::query!("ALTER TABLE completed_requests DISABLE TRIGGER ALL;")
        .execute(&pool)
        .await?;

    let transaction = transaction_instance(); // challenge_id is 42!

    process_request_with_transaction(&pool, &transaction)
        .await
        .unwrap_or_else(|e| panic!("Processing Request failed with error {:#?}", e));

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
            id = 1 AND challenge_id = 42
        "#
    )
    .fetch_one(&pool)
    .await
    .expect("Could not retrieve requests from database");

    // answer_request_student
    let response = client
        .put("/api/requests/42/1")
        .json(&request.expected_response)
        .dispatch()
        .await;

    assert_eq!(
        response.status(),
        Status::Ok,
        "Expected success status, got {:?}, {:?}",
        response.status(),
        response.into_string().await.unwrap()
    );

    let completed_request_response = response
        .into_json::<CompletedRequest>()
        .await
        .expect("Failed to deserialize CompletedRequest response");

    assert_eq!(
        completed_request_response.request_status,
        RequestStatus::Correct,
        "Expected request to be marked Correct, got {:?}",
        completed_request_response.request_status
    );

    Ok(())
}

#[sqlx::test(migrations = "src/migrations")]
async fn answer_request_incorrect(
    _: PgPoolOptions,
    pg_connect_options: PgConnectOptions,
) -> sqlx::Result<()> {
    let client = async_client_from_pg_connect_options(pg_connect_options.clone()).await;
    let pool: sqlx::Pool<sqlx::Postgres> = PgPoolOptions::new()
        .connect_with(pg_connect_options.clone())
        .await?;

    sqlx::query!("ALTER TABLE requests DISABLE TRIGGER ALL;")
        .execute(&pool)
        .await?;
    sqlx::query!("ALTER TABLE completed_requests DISABLE TRIGGER ALL;")
        .execute(&pool)
        .await?;

    let transaction = transaction_instance(); // challenge_id is 42!

    process_request_with_transaction(&pool, &transaction)
        .await
        .unwrap_or_else(|e| panic!("Processing Request failed with error {:#?}", e));

    let wrong_request_type = batch_prediction_instance();

    // answer_request_student
    let response = client
        .put("/api/requests/42/1")
        .json(&wrong_request_type)
        .dispatch()
        .await;

    assert_eq!(
        response.status(),
        Status::Ok,
        "Expected success status, got {:?}, {:?}",
        response.status(),
        response.into_string().await.unwrap()
    );

    let completed_request_response = response
        .into_json::<CompletedRequest>()
        .await
        .expect("Failed to deserialize CompletedRequest response");

    assert_eq!(
        completed_request_response.request_status,
        RequestStatus::Incorrect,
        "Expected request to be marked Incorrect, got {:?}",
        completed_request_response.request_status
    );

    Ok(())
}
