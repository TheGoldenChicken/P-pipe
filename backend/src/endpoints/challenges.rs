use rand::Rng;
use rand::seq::IndexedRandom;
use rocket::serde::json::Json;
use rocket::{delete, get, post, put};
use std::mem::discriminant;
use rocket::{http::Status, response::status::Custom};
use sqlx::QueryBuilder;
use sqlx::types::Json as DbJson;

use crate::assigner::assume_role::create_bucket_sts_token;
use crate::schemas::challenge::{Challenge, ChallengeOptions};
use crate::schemas::common::{AccessType, DispatchTarget, AWSSTS};
use crate::schemas::transaction::Transaction;
use rocket::State;
use sqlx::PgPool;
use aws_sdk_sts::Client as StsClient;

#[post("/api/challenges", data = "<challenge>")]
pub async fn add_challenge(
    db: &State<PgPool>,
    challenge: Json<Challenge>,
    sts_client: &State<StsClient>,
) -> Result<Json<Vec<Challenge>>, Custom<String>> {
    // TODO; Check if we can do this with execute_query?
    let challenge = sqlx::query_as!(
        Challenge,
        r#"
        INSERT INTO challenges
        (challenge_name, init_dataset_location, init_dataset_rows, init_dataset_name,
        init_dataset_description, dispatches_to, time_of_first_release, release_proportions, time_between_releases, challenge_options, email_body, recipient_emails, access_types)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
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
            challenge_options as "challenge_options: DbJson<ChallengeOptions>",
            email_body,
            recipient_emails,
            access_types as "access_types: DbJson<Vec<AccessType>>"
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
        challenge.challenge_options as _,
        challenge.email_body,
        &challenge.recipient_emails as _,
        DbJson::<Vec<AccessType>>(vec![]) as _
    )
    .fetch_one(db.inner())
    .await.map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    // Save what we need before transactions_from_challenge consumes challenge by value
    let challenge_id = challenge.id.expect("challenge id missing after INSERT RETURNING");
    let dispatches_to = challenge.dispatches_to.clone();

    for dispatch in &dispatches_to {
        match dispatch {
            DispatchTarget::S3 => {
                // TODO: Potentially read the bucket from env first time this runs so we can rely on a constant or smth instead... 
                let bucket = std::env::var("P_PIPE_S3_BUCKET").map_err(|e| {
                    Custom(Status::InternalServerError, format!("P_PIPE_S3_BUCKET not set: {e}"))
                })?;
                let prefix = format!("challenge-{}", challenge_id);
                let creds = create_bucket_sts_token(sts_client, &bucket, &prefix, None)
                    .await
                    .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;
                let new_sts = AccessType::STS(AWSSTS {
                    access_key: creds.access_key_id().to_string(),
                    secret_key: creds.secret_access_key().to_string(),
                    session_token: creds.session_token().to_string(),
                    expires: creds.expiration().secs() as u64,
                });
                add_access_type(db, challenge_id, Json(new_sts)).await?;
            }
            DispatchTarget::Drive => {}, // TODO: Create Drive credentials
        }
    }

    // TODO: Move this to be after generating credentials
    // Generate transactions and add them to the DB
    let generated_transactions = transactions_from_challenge(challenge)?;
    add_transactions_into_db(db.inner(), &generated_transactions).await?;

    get_challenges(db).await
}


// Despite not being an endpoint, this is tested through integration tests, not unittests!
pub async fn add_transactions_into_db(
    db: &PgPool,
    transactions: &[Transaction],
) -> Result<u64, Custom<String>> {
    if transactions.is_empty() {
        return Ok(0);
    }

    let mut builder = QueryBuilder::new(
        "INSERT INTO transactions (
            challenge_id,
            scheduled_time,
            source_data_location,
            data_intended_location,
            data_intended_name,
            rows_to_push,
            dispatch_location,
            challenge_options
        ) ",
    );

    builder.push_values(transactions, |mut b, tx| {
        b.push_bind(tx.challenge_id)
            .push_bind(tx.scheduled_time)
            .push_bind(&tx.source_data_location)
            .push_bind(&tx.data_intended_location)
            .push_bind(&tx.data_intended_name)
            .push_bind(&tx.rows_to_push)
            .push_bind(&tx.dispatch_location)
            .push_bind(&tx.challenge_options);
    });

    let affected = builder
        .build()
        .execute(db)
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
        // Random slice of dispatches_to
        let mut rng: rand::prelude::ThreadRng = rand::rng();
        let n = rng.random_range(1..=challenge.dispatches_to.len());
        let dispatch_locations = challenge.dispatches_to.choose_multiple(&mut rng, n);

        // TODO: We can avoid unecessary cloning by using shuffling with .drain(..n)
        for item in dispatch_locations.cloned() {
            // For S3 the bucket is embedded here so the transaction is fully
            // self-contained (README's "atomic transaction" goal), and it is
            // built with hyphens so nothing downstream needs to sanitise it.
            let data_intended_location = match &item {
                DispatchTarget::S3 => {
                    let bucket = std::env::var("P_PIPE_S3_BUCKET").map_err(|e| {
                        Custom(Status::InternalServerError, format!("P_PIPE_S3_BUCKET not set: {e}"))
                    })?;
                    format!("{}/challenge-{}", bucket, challenge_id)
                }
                DispatchTarget::Drive => format!("challenge-{}", challenge_id),
            };

            let transaction = Transaction {
                id: None,
                challenge_id: challenge_id,
                created_at: None,
                scheduled_time,
                source_data_location: Some(challenge.init_dataset_location.clone()),
                dispatch_location: Some(item),
                data_intended_location,
                data_intended_name: Some(format!("release_{}", i)),
                rows_to_push: Some(rows_to_push.clone()),
                challenge_options: challenge.challenge_options.clone(),
            };
            transactions.push(transaction);
        }
    }

    Ok(transactions)
}

#[get("/api/challenges")]
pub async fn get_challenges(
    db: &State<PgPool>,
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
            challenge_options as "challenge_options: DbJson<ChallengeOptions>",
            email_body,
            recipient_emails,
            access_types as "access_types: DbJson<Vec<AccessType>>"
        FROM
            challenges;
        "#
    )
    .fetch_all(db.inner())
    .await
    .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    Ok(Json(challenges))
}

#[delete("/api/challenges/<id>")]
pub async fn delete_challenge(db: &State<PgPool>, id: i32) -> Result<Status, Custom<String>> {
    sqlx::query!("DELETE FROM challenges WHERE id = $1", id)
        .execute(db.inner())
        .await
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    Ok(Status::NoContent)
}

#[delete("/api/challenges")]
pub async fn destroy_challenges(db: &State<PgPool>) -> Result<Status, Custom<String>> {
    sqlx::query!("DELETE FROM challenges")
        .execute(db.inner())
        .await
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    Ok(Status::NoContent)
}

#[put("/api/challenges/<id>/access_type", data = "<access_type>")]
pub async fn add_access_type(
    db: &State<PgPool>,
    id: i32,
    access_type: Json<AccessType>,
) -> Result<Json<Challenge>, Custom<String>> {
    let row = sqlx::query!(
        r#"SELECT access_types as "access_types: DbJson<Vec<AccessType>>" FROM challenges WHERE id = $1"#,
        id
    )
    .fetch_one(db.inner())
    .await
    .map_err(|e| Custom(Status::NotFound, e.to_string()))?;

    let mut access_types = row.access_types.0;
    access_types.retain(|existing| discriminant(existing) != discriminant(&*access_type));
    access_types.push(access_type.into_inner());
    let access_types = DbJson(access_types);

    let challenge = sqlx::query_as!(
        Challenge,
        r#"
        UPDATE challenges SET access_types = $1 WHERE id = $2
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
            challenge_options as "challenge_options: DbJson<ChallengeOptions>",
            email_body,
            recipient_emails,
            access_types as "access_types: DbJson<Vec<AccessType>>"
        "#,
        access_types as _,
        id
    )
    .fetch_one(db.inner())
    .await
    .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    Ok(Json(challenge))
}

#[post("/api/challenges/<id>/regenerate_sts")]
pub async fn regenerate_sts(
    db: &State<PgPool>,
    id: i32,
    sts_client: &State<StsClient>,
) -> Result<Json<Challenge>, Custom<String>> {
    let challenge = sqlx::query_as!(
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
            challenge_options as "challenge_options: DbJson<ChallengeOptions>",
            email_body,
            recipient_emails,
            access_types as "access_types: DbJson<Vec<AccessType>>"
        FROM challenges WHERE id = $1
        "#,
        id
    )
    .fetch_one(db.inner())
    .await
    .map_err(|e| Custom(Status::NotFound, e.to_string()))?;

    if !challenge.dispatches_to.contains(&DispatchTarget::S3) {
        return Err(Custom(
            Status::BadRequest,
            "Challenge has no AWS dispatch locations; cannot regenerate STS credentials".to_string(),
        ));
    }

    let bucket = std::env::var("P_PIPE_S3_BUCKET").map_err(|e| {
        Custom(Status::InternalServerError, format!("P_PIPE_S3_BUCKET not set: {e}"))
    })?;
    let prefix = format!("challenge-{}", id);
    let creds = create_bucket_sts_token(sts_client, &bucket, &prefix, None)
        .await
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    let new_sts = AccessType::STS(AWSSTS {
        access_key: creds.access_key_id().to_string(),
        secret_key: creds.secret_access_key().to_string(),
        session_token: creds.session_token().to_string(),
        expires: creds.expiration().secs() as u64,
    });

    let mut access_types = challenge.access_types.0;
    access_types.retain(|existing| discriminant(existing) != discriminant(&new_sts));
    access_types.push(new_sts);
    let access_types = DbJson(access_types);

    let updated = sqlx::query_as!(
        Challenge,
        r#"
        UPDATE challenges SET access_types = $1 WHERE id = $2
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
            challenge_options as "challenge_options: DbJson<ChallengeOptions>",
            email_body,
            recipient_emails,
            access_types as "access_types: DbJson<Vec<AccessType>>"
        "#,
        access_types as _,
        id
    )
    .fetch_one(db.inner())
    .await
    .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    Ok(Json(updated))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing_common::instances::{
        challenge_instance, transactions_expected_from_challenge_instance,
    };
    use proptest::prelude::*;
    // Wish I could be without this, but it also lets me iterate over them...
    use proptest::sample::subsequence;
    use strum::{EnumCount, IntoEnumIterator};


    #[test]
    fn test_transactions_from_challenge_basic() {
        let challenge = challenge_instance(); // 3 release proportions

        let transactions = transactions_from_challenge(challenge)
            .expect("Could not generate transactions from challenge!");

        assert_eq!(
            transactions.len(),
            3,
            "Expected 3 transactions, got {}",
            transactions.len()
        );
    }

    #[test]
    fn test_transactions_created_correctly() {
        let challenge = challenge_instance();
        let created_transactions = transactions_from_challenge(challenge)
            .expect("Could not generate transactions from challenge!");
        let expected_transactions = transactions_expected_from_challenge_instance();

        assert_eq!(
            created_transactions, expected_transactions,
            "Expected created transactions to match expected transactions!"
        )
    }

    proptest! {
        #[test]
        fn total_rows_pushed_equals_init_dataset_rows(
            proportions in prop::collection::vec(0.0..1.0, 1..10),
            init_rows in 50..2000i32
        ) {

            // Normalize proportions so they sum to 1.0
            let total: f64 = proportions.iter().sum();
            let normalized: Vec<f64> = if total == 0.0 {
                vec![1.0] // fallback to avoid division by zero
            } else {
                proportions.iter().map(|p| p / total).collect()
            };

            let challenge = Challenge {
                id: Some(1),
                challenge_name: "test".into(),
                created_at: None,
                init_dataset_location: "s3://bucket/data.csv".into(),
                init_dataset_rows: init_rows,
                init_dataset_name: None,
                init_dataset_description: None,
                dispatches_to: vec![DispatchTarget::S3],
                time_of_first_release: 0,
                release_proportions: normalized.clone(),
                time_between_releases: 1,
                challenge_options: DbJson(ChallengeOptions::default()),
                email_body: None,
                recipient_emails: vec![],
                access_types: DbJson(vec![]),
            };

            let transactions = transactions_from_challenge(challenge)
                .expect("Could not generate transactions from challenge");

            // Sum up all rows pushed
            let total_rows: i32 = transactions.iter()
                .map(|t| {
                    let rows = t.rows_to_push.as_ref().expect("rows_to_push missing");
                    rows[1] - rows[0]
                })
                .sum();

            // Assert that the sum equals init_dataset_rows
            prop_assert_eq!(
                total_rows,
                init_rows,
                "Expected total rows to equal {}, got {}",
                init_rows,
                total_rows
            );
        }
    }

    fn subset_strategy() -> impl Strategy<Value = Vec<DispatchTarget>> {
        let all = DispatchTarget::iter().collect::<Vec<DispatchTarget>>();
        subsequence(all, 1..=DispatchTarget::COUNT)
    }

    proptest! {
        #[test]
        fn multiple_dispatch_trans_between_minmax(
            proportions in prop::collection::vec(0.0..1.0, 1..10),
            dispatch_locations in subset_strategy()
        ) {

        let expected_min_transactions = proportions.len();
        let expected_max_transactions = expected_min_transactions * dispatch_locations.len();

        println!("{:?}", proportions.clone());
        println!("{:?}", dispatch_locations.clone());
        
        let challenge = Challenge {
            id: Some(1),
            challenge_name: "testingchallenge1".into(),
            created_at: None,
            init_dataset_location: "s3://bucket/data.csv".into(),
            init_dataset_rows: 300,
            init_dataset_name: None,
            init_dataset_description: None,
            dispatches_to: dispatch_locations,
            time_of_first_release: 1000,
            release_proportions: proportions,
            time_between_releases: 60,
            challenge_options: DbJson(ChallengeOptions::default()),
            email_body: None,
            recipient_emails: vec![],
            access_types: DbJson(vec![]),
        };

        let transactions = transactions_from_challenge(challenge)
        .expect("Could not generate transactions!");

        prop_assert!(
                    transactions.len() >= expected_min_transactions &&
                    transactions.len() <= expected_max_transactions,
                    "Expected number of generated transactions between min ({}) and max ({}), got: {}",
                    expected_min_transactions,
                    expected_max_transactions,
                    transactions.len()
                );
        }
    }
}
