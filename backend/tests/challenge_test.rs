use rocket::http::Status;
use rocket::local::asynchronous::LocalResponse;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::types::Json as DbJson;

use backend::schemas::challenge::Challenge;
use backend::schemas::common::{AccessBinding, DispatchTarget};
use backend::schemas::transaction::Transaction;

use backend::endpoints::challenges::add_transactions_into_db;
use backend::testing_common::connect::async_client_from_pg_connect_options;
use backend::testing_common::instances::{
    challenge_instance, minimal_challenge_instance, transactions_expected_from_challenge_instance,
};

// TODO: To make testing less brittle, have standard common functions for stuff like INSERT, SELECT sql statements...
// ... perhaps also use these in the endpoints themselves, to have some sort of standardization...

async fn unpack_challenge_repsponse(response: LocalResponse<'_>) -> Challenge {
    assert_eq!(
        response.status(),
        Status::Ok,
        "Expected success status, got {:?}",
        response.status()
    );

    response
        .into_json::<Vec<Challenge>>()
        .await
        .expect("Failed to deserialize Challenge response")
        .get(0)
        .cloned()
        .expect("No challenge returned from POST!")
}

#[sqlx::test]
async fn challenge_post_basic(
    _: PgPoolOptions,
    pg_connect_options: PgConnectOptions,
) -> sqlx::Result<()> {
    let client = async_client_from_pg_connect_options(pg_connect_options).await;

    let challenge = challenge_instance();
    let response = client
        .post("/api/challenges")
        .json(&challenge)
        .dispatch()
        .await;

    let _response = unpack_challenge_repsponse(response);
    Ok(())
}

#[sqlx::test]
async fn challenge_get_basic(
    _: PgPoolOptions,
    pg_connect_options: PgConnectOptions,
) -> sqlx::Result<()> {
    let pool = PgPoolOptions::new()
        .connect_with(pg_connect_options.clone())
        .await?;
    let client = async_client_from_pg_connect_options(pg_connect_options).await;

    let challenge = challenge_instance();

    sqlx::query!(
        r#"
        INSERT INTO challenges
        (challenge_name, init_dataset_location, init_dataset_rows, init_dataset_name,
        init_dataset_description, dispatches_to, time_of_first_release, release_proportions,
        time_between_releases, access_bindings)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#,
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
    .execute(&pool)
    .await?;

    let response = client.get("/api/challenges").dispatch().await;

    let _response = unpack_challenge_repsponse(response);
    Ok(())
}

#[sqlx::test]
async fn post_minimal_challenge(
    _: PgPoolOptions,
    pg_connect_options: PgConnectOptions,
) -> sqlx::Result<()> {
    let client = async_client_from_pg_connect_options(pg_connect_options).await;

    let challenge = minimal_challenge_instance();
    let response = client
        .post("/api/challenges")
        .json(&challenge)
        .dispatch()
        .await;
    let _response = unpack_challenge_repsponse(response);
    Ok(())
}

#[sqlx::test]
async fn challenge_delete_basic(
    _: PgPoolOptions,
    pg_connect_options: PgConnectOptions,
) -> sqlx::Result<()> {
    let pool = PgPoolOptions::new()
        .connect_with(pg_connect_options.clone())
        .await?;
    let client = async_client_from_pg_connect_options(pg_connect_options).await;

    let challenge = challenge_instance();

    sqlx::query!(
        r#"
        INSERT INTO challenges
        (id, challenge_name, init_dataset_location, init_dataset_rows, init_dataset_name,
        init_dataset_description, dispatches_to, time_of_first_release, release_proportions,
        time_between_releases, access_bindings)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#,
        1,
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
    .execute(&pool)
    .await?;

    let response = client.delete("/api/challenges/1").dispatch().await;
    assert_eq!(
        response.status(),
        Status::NoContent, // Remember, Delete returns NoContent, not OK
        "Expected NoContent status on DELETE, got {:?}",
        response.status()
    );

    let deleted = sqlx::query("SELECT 1 FROM challenges WHERE id = $1")
        .bind(1)
        .fetch_optional(&pool)
        .await?;

    assert!(
        deleted.is_none(),
        "Challenge with id 1 should have been deleted"
    );

    Ok(())
}

#[sqlx::test]
async fn challenge_destroy_all(
    _: PgPoolOptions,
    pg_connect_options: PgConnectOptions,
) -> sqlx::Result<()> {
    let pool = PgPoolOptions::new()
        .connect_with(pg_connect_options.clone())
        .await?;
    let client = async_client_from_pg_connect_options(pg_connect_options).await;

    // Insert multiple challenges
    for i in 1..=3 {
        let mut challenge = challenge_instance();
        challenge.challenge_name = format!("Challenge {}", i);

        sqlx::query(
            r#"
            INSERT INTO challenges
            (id, challenge_name, init_dataset_location, init_dataset_rows, init_dataset_name,
            init_dataset_description, dispatches_to, time_of_first_release, release_proportions,
            time_between_releases, access_bindings)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(i)
        .bind(&challenge.challenge_name)
        .bind(&challenge.init_dataset_location)
        .bind(challenge.init_dataset_rows)
        .bind(&challenge.init_dataset_name)
        .bind(&challenge.init_dataset_description)
        .bind(&challenge.dispatches_to)
        .bind(challenge.time_of_first_release)
        .bind(&challenge.release_proportions)
        .bind(challenge.time_between_releases)
        .bind(&challenge.access_bindings)
        .execute(&pool)
        .await?;
    }

    // Call destroy endpoint (assumed to delete all challenges)
    let response = client.delete("/api/challenges").dispatch().await;
    assert_eq!(
        response.status(),
        Status::NoContent,
        "Expected NoContent status on destroy, got {:?}",
        response.status()
    );

    // Verify all challenges are gone
    let remaining = sqlx::query("SELECT 1 FROM challenges")
        .fetch_optional(&pool)
        .await?;

    assert!(
        remaining.is_none(),
        "Expected no remaining challenges after destroy"
    );

    Ok(())
}

// TODO: Really would like to proptest this, so I can check if it even works when pushing n transactions to the db...
// TODO: Figure out why this fails if I don't specify src/migrations, but the others don't...
#[sqlx::test(migrations = "src/migrations")]
async fn add_transactions_into_db_basic(pool: sqlx::PgPool) {
    let mut conn = pool.acquire().await.expect("Could not acquire pool!");

    let challenge = challenge_instance();

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
    .execute(&mut *conn)
    .await
    .expect("Could not add challenge to db!");

    let transactions = transactions_expected_from_challenge_instance();
    // TODO: Figure out why the compiler lets us use &mut PoolConnection<Postgres> in place of &mut sqlx::PgConnection....

    let affected = add_transactions_into_db(&mut conn, &transactions)
        .await
        .expect("Error while trying to add transactions into db!");

    assert_eq!(
        transactions.len() as u64,
        affected,
        "Expected transactions of len {} to match number affected rows: {}",
        transactions.len(),
        affected
    );
}

#[sqlx::test(migrations = "src/migrations")]
async fn add_transactions_into_db_expected_output(pool: sqlx::PgPool) {
    let mut conn = pool.acquire().await.expect("Could not acquire pool!");

    let challenge = challenge_instance();

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
    .execute(&mut *conn)
    .await
    .expect("Could not add challenge to db!");

    let transactions = transactions_expected_from_challenge_instance();
    // TODO: Figure out why the compiler lets us use &mut PoolConnection<Postgres> in place of &mut sqlx::PgConnection....

    add_transactions_into_db(&mut conn, &transactions)
        .await
        .expect("Error while trying to add transactions into db!");

    let transactions_from_db = sqlx::query_as!(
        Transaction,
        r#"
        SELECT
            id,
            challenge_id,
            created_at,
            scheduled_time,
            source_data_location,
            dispatch_location as "dispatch_location: DispatchTarget",
            data_intended_location,
            data_intended_name,
            rows_to_push,
            access_bindings as "access_bindings: DbJson<Vec<AccessBinding>>"
        FROM 
            transactions
        "#
    )
    .fetch_all(&mut *conn)
    .await
    .expect("Could not retrieve transactions from db!");

    assert_eq!(
        transactions, transactions_from_db,
        "Transactions to function did not match transctions from db afterwards!"
    )
}

// #[sqlx::test]
// async fn challenge_post_posts_to_transactions(
//     _: PgPoolOptions,
//     pg_connect_options: PgConnectOptions,
// ) -> sqlx::Result<()> {
//     let pool = PgPoolOptions::new()
//         .connect_with(pg_connect_options.clone())
//         .await?;

//     let client = async_client_from_pg_connect_options(pg_connect_options).await;
//     let base_challenges = "/api/challenges";

//     let post = Challenge {
//         id: None,
//         created_at: None,
//         name: String::from("testing_challenge"),
//         init_dataset_location: String::from("/home/cicero/ppipe/tests/test_data/iris.csv"),
//         init_dataset_rows: 300,
//         init_dataset_name: Some(String::from("iris")),
//         init_dataset_description: Some(String::from(
//             "a .csv collection of flowers, classification task",
//         )),
//         time_of_first_release: 5000,
//         release_proportions: vec![0.50, 0.25],
//         time_between_releases: 100,
//     };

//     let response = client.post(base_challenges).json(&post).dispatch().await;

//     assert_eq!(
//         response.status(),
//         Status::Ok,
//         "Expected success status, got {:?}",
//         response.status()
//     );

//     let response = response
//         .into_json::<Vec<Challenge>>()
//         .await
//         .expect("Failed to deserialize Challenge response")
//         .get(0)
//         .cloned()
//         .expect("No challenge returned from POST!");

//     let expected_transactions = transactions_from_challenge(response)
//         .expect("Could not generate expected transactions from challenge");

//     let db_transactions: Vec<Transaction> =
//         sqlx::query_as::<_, Transaction>("SELECT * FROM transactions")
//             .fetch_all(&pool)
//             .await?;

//     if db_transactions.is_empty() {
//         panic!("No transactions found in db after POST api/challenges")
//     }

//     assert_eq!(
//         db_transactions, expected_transactions,
//         "Expected db transactions after POST api/challenges to match expected transactions from transactions_from_challenge!"
//     );

//     Ok(())
// }
