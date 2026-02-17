use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::types::Json as DbJson;

use backend::endpoints::scheduler::{process_request_with_transaction, request_from_transaction};
use backend::schemas::request::{Request, RequestType};
use backend::testing_common::instances::transaction_instance;

// TODO: Maybe this should be a request test? What we're testing isn't whether it's scheduled per-say, but the below isn't relevant if it isn't...
#[sqlx::test(migrations = "src/migrations")]
async fn process_request_basic(
    _: PgPoolOptions,
    pg_connect_options: PgConnectOptions,
) -> sqlx::Result<()> {
    let pool = PgPoolOptions::new()
        .connect_with(pg_connect_options.clone())
        .await?;

    // Temporarily suspend triggers to allow request to exist without existing challenge_id
    sqlx::query!("ALTER TABLE requests DISABLE TRIGGER ALL;")
        .execute(&pool)
        .await?;

    let transaction = transaction_instance();
    let expected_request = request_from_transaction(&transaction)
        .await
        .expect("Could not generate expected request");

    process_request_with_transaction(&pool, &transaction)
        .await
        .unwrap_or_else(|e| panic!("Processing Request failed with error {:#?}", e));

    let recieved_requests = sqlx::query_as!(
        Request,
        r#"
    SELECT
        id,
        challenge_id,
        created_at,
        type_of_request as "type_of_request: DbJson<RequestType>",
        expected_response as "expected_response: DbJson<RequestType>",
        deadline
    FROM requests
    ORDER BY created_at ASC
    LIMIT 1
    "#,
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(
        expected_request, recieved_requests,
        "DID YOU RUN TESTS WITH `--features deterministic` ??? Request from request_from_transaction did not match request from db following process_request_with_transaction"
    );
    Ok(())
}
