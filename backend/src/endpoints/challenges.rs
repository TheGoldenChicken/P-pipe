use rand::seq::IndexedRandom;
use rocket::serde::json::Json;
use rocket::{delete, get, post}; // Have to do this as long as src/lib.rs contains `pub mod endpoints;`, as it breaks #[macro_use]
use rocket::{http::Status, response::status::Custom};

use rocket_db_pools::Connection;
use sqlx::Arguments; // Even though arguments appears unused, it is used in the background (macros perhaps?)
use sqlx::types::Json as DbJson;

use rand::Rng;

use crate::schemas::challenge::Challenge;
use crate::schemas::common::{AccessBinding, Db, DispatchTarget};
use crate::schemas::transaction::Transaction;

#[post("/api/challenges", data = "<challenge>")]
pub async fn add_challenge(
    mut db: Connection<Db>,
    challenge: Json<Challenge>,
) -> Result<Json<Vec<Challenge>>, Custom<String>> {
    // TODO; Check if we can do this with execute_query?
    let challenge = sqlx::query_as!(
        Challenge,
        r#"
        INSERT INTO challenges
        (challenge_name, init_dataset_location, init_dataset_rows, init_dataset_name,
        init_dataset_description, dispatches_to, time_of_first_release, release_proportions, time_between_releases, access_bindings)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING
            id,
            challenge_name,
            created_at,
            init_dataset_location,
            init_dataset_rows,
            init_dataset_name,
            init_dataset_description,
            dispatches_to as "dispatches_to: Vec<DispatchTarget>",
            time_of_first_release,
            release_proportions,
            time_between_releases,
            access_bindings as "access_bindings: DbJson<Vec<AccessBinding>>"
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
    .fetch_one(&mut **db)
    .await.map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    // Generate transactions and add them to the DB
    let generated_transactions = transactions_from_challenge(challenge)?;
    add_transactions_into_db(&mut db, &generated_transactions).await?;

    get_challenges(db).await
}

// TODO IMPORTANT: Really have a good dig into this one, will fail regularly if we don't find a better way of structuring it, and we'll have no idea why it fails...
async fn add_transactions_into_db(
    db: &mut Connection<Db>,
    transactions: &[Transaction],
) -> Result<u64, Custom<String>> {
    if transactions.is_empty() {
        return Ok(0);
    }

    let mut query = String::from(
        "INSERT INTO transactions (
            challenge_id,
            scheduled_time,
            source_data_location,
            data_intended_location,
            data_intended_name,
            rows_to_push,
            dispatch_location,
            access_bindings
        ) VALUES ",
    );

    let mut args = sqlx::postgres::PgArguments::default();

    for (i, tx) in transactions.iter().enumerate() {
        if i > 0 {
            query.push_str(", ");
        }

        let base = i * 8;
        query.push_str(&format!(
            "(${}, ${}, ${}, ${}, ${}, ${}, ${}, ${})",
            base + 1,
            base + 2,
            base + 3,
            base + 4,
            base + 5,
            base + 6,
            base + 7,
            base + 8
        ));

        args.add(tx.challenge_id);
        args.add(tx.scheduled_time);
        args.add(&tx.source_data_location);
        args.add(&tx.data_intended_location);
        args.add(&tx.data_intended_name);
        args.add(&tx.rows_to_push);
        args.add(&tx.dispatch_location);
        args.add(&tx.access_bindings);
    }

    let affected = sqlx::query_with(&query, args)
        .execute(&mut ***db)
        .await
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?
        .rows_affected();

    Ok(affected)
}

fn transactions_from_challenge(challenge: Challenge) -> Result<Vec<Transaction>, Custom<String>> {
    let mut transactions = Vec::new();

    let mut running_proportion: f64 = 0.;

    for (i, release_proportion) in challenge.release_proportions.iter().enumerate() {
        let scheduled_time =
            challenge.time_of_first_release + challenge.time_between_releases * i as i64;

        // Old implementation, added all data points that should be included... may be useful still...
        // let rows_to_push_count = (release_proportion * challenge.init_dataset_rows as f64).round() as i32;
        // let rows_to_push = (0..rows_to_push_count).collect::<Vec<i32>>();

        // TODO: Consider option to have each portion randomly split between dispatch_locations...
        let rows_from = (running_proportion * challenge.init_dataset_rows as f64).round() as i32;
        let rows_to = ((running_proportion + release_proportion)
            * challenge.init_dataset_rows as f64)
            .round() as i32;
        let rows_to_push = vec![rows_from, rows_to];
        running_proportion += release_proportion;

        // Returns error here if challenge id does not exist.
        let challenge_id = challenge
            .id
            .ok_or_else(|| Custom(Status::BadRequest, "Missing challenge ID".to_string()))?;

        // TODO: Consider if this is safe behavior... can it panic?
        // Random slice of Dispatches to
        let mut rng: rand::prelude::ThreadRng = rand::rng();
        let n = rng.random_range(1..=challenge.dispatches_to.len());
        let dispatch_locations = challenge.dispatches_to.choose_multiple(&mut rng, n);

        // TODO: We can avoid unecessary cloning by using shuffling with .drain(..n)
        for item in dispatch_locations.cloned() {
            let transaction = Transaction {
                id: None,
                challenge_id: challenge_id,
                created_at: None,
                scheduled_time,
                source_data_location: Some(challenge.init_dataset_location.clone()),
                dispatch_location: Some(item),
                data_intended_location: format!(
                    "challenge_{}_{}",
                    challenge_id, challenge.challenge_name
                ),
                data_intended_name: Some(format!("release_{}", i)),
                rows_to_push: Some(rows_to_push.clone()),
                access_bindings: challenge.access_bindings.clone(),
            };
            transactions.push(transaction);
        }
    }

    Ok(transactions)
}

#[get("/api/challenges")]
pub async fn get_challenges(
    mut db: Connection<Db>,
) -> Result<Json<Vec<Challenge>>, Custom<String>> {
    let challenges = sqlx::query_as!(
        Challenge,
        r#"
        SELECT
            id,
            challenge_name,
            created_at,
            init_dataset_location,
            init_dataset_rows,
            init_dataset_name,
            init_dataset_description,
            dispatches_to as "dispatches_to: Vec<DispatchTarget>",
            time_of_first_release,
            release_proportions,
            time_between_releases,
            access_bindings as "access_bindings: DbJson<Vec<AccessBinding>>"
        FROM
            challenges;
        "#
    )
    .fetch_all(&mut **db)
    .await
    .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    Ok(Json(challenges))
}

#[delete("/api/challenges/<id>")]
pub async fn delete_challenge(mut db: Connection<Db>, id: i32) -> Result<Status, Custom<String>> {
    sqlx::query!("DELETE FROM challenges WHERE id = $1", id)
        .execute(&mut **db)
        .await
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    Ok(Status::NoContent)
}

#[delete("/api/challenges")]
pub async fn destroy_challenges(mut db: Connection<Db>) -> Result<(), Custom<String>> {
    sqlx::query!("DELETE FROM challenges")
        .execute(&mut **db)
        .await
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    Ok(())
}
