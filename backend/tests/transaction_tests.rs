use rocket::http::Status;
use rocket::local::asynchronous::LocalResponse;
use sqlx::{Executor, Postgres};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

use backend::schemas::transaction::Transaction;
use backend::schemas::challenge::Challenge;
use backend::testing_common::connect::async_client_from_pg_connect_options;
use backend::testing_common::instances::{challenge_instance, transaction_instance};

// AVAST!
// GET TEST WILL POSSIBLY FAIL WITH ATTACH_SCHEDULER=true

async fn unpack_transaction_response(response: LocalResponse<'_>) -> Vec<Transaction> {
    assert_eq!(
        response.status(),
        Status::Ok,
        "Expected success status, got {:?}",
        response.status()
    );

    response
        .into_json::<Vec<Transaction>>()
        .await
        .expect("Failed to deserialize Transaction response")
}


pub async fn add_challenge_to_db<'a, E>(
    executor: E,
    challenge: &Challenge,
) -> sqlx::Result<()>
where
    E: Executor<'a, Database = Postgres>,
{
    sqlx::query!(
        r#"
        INSERT INTO challenges
        (id, challenge_name, init_dataset_location, init_dataset_rows, init_dataset_name,
        init_dataset_description, dispatches_to, time_of_first_release, release_proportions,
        time_between_releases, access_bindings)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#,
        challenge.id,
        challenge.challenge_name,
        challenge.init_dataset_location,
        challenge.init_dataset_rows,
        challenge.init_dataset_name,
        challenge.init_dataset_description,
        challenge.dispatches_to as _,
        challenge.time_of_first_release,
        &challenge.release_proportions,
        challenge.time_between_releases,
        challenge.access_bindings as _
    )
    .execute(executor)
    .await?;

    Ok(())
}

#[sqlx::test(migrations = "src/migrations")]
async fn transactions_get_basic(
    _: PgPoolOptions,
    pg_connect_options: PgConnectOptions,
) -> sqlx::Result<()> {
    let pool = PgPoolOptions::new()
        .connect_with(pg_connect_options.clone())
        .await?;

    // Gotta do this so the insert does not violate foreign key constraints!
    let challenge = challenge_instance();
    add_challenge_to_db(&pool, &challenge).await
    .expect("Could not insert challenge to db!");

    let client = async_client_from_pg_connect_options(pg_connect_options).await;

    let tx = transaction_instance();

    sqlx::query!(
        r#"
        INSERT INTO transactions
        (challenge_id, scheduled_time, source_data_location, dispatch_location,
        data_intended_location, data_intended_name, rows_to_push, access_bindings)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
        tx.challenge_id,
        tx.scheduled_time,
        tx.source_data_location,
        tx.dispatch_location as _,
        tx.data_intended_location,
        tx.data_intended_name,
        tx.rows_to_push.as_ref().unwrap(),
        tx.access_bindings as _
    )
    .execute(&pool)
    .await?;

    let response = client.get("/api/transactions").dispatch().await;
    let transactions = unpack_transaction_response(response).await;

    assert_eq!(transactions.len(), 1, "Expected exactly one transaction");
    Ok(())
}

#[sqlx::test(migrations = "src/migrations")]
async fn transaction_delete_basic(
    _: PgPoolOptions,
    pg_connect_options: PgConnectOptions,
) -> sqlx::Result<()> {
    let pool = PgPoolOptions::new()
        .connect_with(pg_connect_options.clone())
        .await?;

    let challenge = challenge_instance();
    add_challenge_to_db(&pool, &challenge).await
    .expect("Could not insert challenge to db!");

    let client = async_client_from_pg_connect_options(pg_connect_options).await;

    let tx = transaction_instance();

    let rec = sqlx::query!(
        r#"
        INSERT INTO transactions
        (challenge_id, scheduled_time, source_data_location, dispatch_location,
        data_intended_location, data_intended_name, rows_to_push, access_bindings)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id
        "#,
        tx.challenge_id,
        tx.scheduled_time,
        tx.source_data_location,
        tx.dispatch_location as _,
        tx.data_intended_location,
        tx.data_intended_name,
        tx.rows_to_push.as_ref().unwrap(),
        tx.access_bindings as _
    )
    .fetch_one(&pool)
    .await?;

    let response = client
        .delete(format!("/api/transactions/{}", rec.id))
        .dispatch()
        .await;
    assert_eq!(response.status(), Status::NoContent);

    let remaining = sqlx::query!(
        "SELECT COUNT(*) as count FROM transactions WHERE id = $1",
        rec.id
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(remaining.count.unwrap_or(0), 0);
    Ok(())
}

#[sqlx::test(migrations = "src/migrations")]
async fn transactions_destroy_basic(
    _: PgPoolOptions,
    pg_connect_options: PgConnectOptions,
) -> sqlx::Result<()> {
    let pool = PgPoolOptions::new()
        .connect_with(pg_connect_options.clone())
        .await?;

    let challenge = challenge_instance();
    add_challenge_to_db(&pool, &challenge).await
    .expect("Could not insert challenge to db!");

    let client = async_client_from_pg_connect_options(pg_connect_options).await;

    let tx = transaction_instance();

    for _ in 0..3 {
        sqlx::query!(
            r#"
            INSERT INTO transactions
            (challenge_id, scheduled_time, source_data_location, dispatch_location,
            data_intended_location, data_intended_name, rows_to_push, access_bindings)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            tx.challenge_id,
            tx.scheduled_time,
            tx.source_data_location,
            tx.dispatch_location as _,
            tx.data_intended_location,
            tx.data_intended_name,
            tx.rows_to_push.as_ref().unwrap(),
            tx.access_bindings as _
        )
        .execute(&pool)
        .await?;
    }

    let response = client.delete("/api/transactions").dispatch().await;
    assert_eq!(response.status(), Status::Ok);

    let remaining = sqlx::query!("SELECT COUNT(*) as count FROM transactions")
        .fetch_one(&pool)
        .await?;
    assert_eq!(remaining.count.unwrap_or(0), 0);
    Ok(())
}
