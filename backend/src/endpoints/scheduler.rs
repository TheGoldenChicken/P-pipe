use chrono::Utc;
use rocket::{fairing::AdHoc, http::Status, response::status::Custom};
use sqlx::types::Json;
use std::path::Path;
use std::process::Command;
use tokio::time::{Duration, interval};
use rand;
use rand::seq::IndexedRandom;

use crate::schemas::challenge::ChallengeOptions;
use crate::schemas::common::{AccessBinding, Db, DispatchTarget, TransactionStatus};
use crate::schemas::request::{DataValidationPayload, Request, RequestType};
use crate::schemas::transaction::{CompletedTransaction, Transaction};
use crate::global_rng::global_rng;

// TODO: Move this as a method on Transaction? Should make sense...
pub async fn request_from_transaction(tx: &Transaction) -> Result<Request, Custom<String>> {
    let generated_request_type = match &tx.challenge_options.possible_request_types {
        Some(requests) => {
            let mut rng = global_rng();
            let random_request_type = requests.choose(&mut rng).unwrap(); // TODO: Remove naked unwrap here!
            match random_request_type {
                _ => RequestType::DataValidation(DataValidationPayload::generate_from_transaction(
                    &tx,
                )?),
            }
        }
        None => RequestType::DataValidation(DataValidationPayload::generate_from_transaction(&tx)?),
    };

    let generated_request_string = serde_json::to_string(&generated_request_type)
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;
    let transaction_string = serde_json::to_string(&tx)
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    let python_path = Path::new(".venv/bin/python");

    let output = Command::new(python_path)
        .current_dir("..")
        .arg("-m")
        .arg("py_modules.judgement.expected_response_cli")
        .arg("--request")
        .arg(&generated_request_string)
        .arg("--transaction")
        .arg(&transaction_string)
        .output()
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                format!(
                    "Failed calling Python script to generate expected repsonse with error {:?}",
                    e
                ),
            )
        })?;

    if output.status.code() != Some(0) {
        return Err(Custom(
            Status::InternalServerError,
            format!(
                "Python script failed with error code: {:?} and error: {:?}, and stdout {:?}",
                output.status.code(),
                String::from_utf8_lossy(&output.stderr),
                String::from_utf8_lossy(&output.stdout)
            ),
        ));
    }

    let stdout = &String::from_utf8_lossy(&output.stdout);

    let parsed_stdout = serde_json::from_str::<RequestType>(&stdout).map_err(|e| {
        Custom(
            Status::InternalServerError,
            format!(
                "Failed to parse JSON for expected response from Python output with error: {:?}",
                e
            ),
        )
    })?;

    let deadline = match &tx.challenge_options.requests_deadline {
        Some(deadline) => Some(Utc::now().timestamp_millis() + deadline),
        None => None,
    };

    Ok(Request {
        id: None,
        created_at: None,
        challenge_id: tx.challenge_id,
        type_of_request: Json(generated_request_type),
        expected_response: Json(parsed_stdout),
        deadline: deadline,
    })
}

// TODO: Rename this to something along the lines of "move data as transaction, given that it doesn't process *everything* (not requests)"
pub async fn process_transaction(tx: &Transaction) -> Result<std::process::Output, Custom<String>> {
    // TODO: Move python_path to env variable
    let python_path = Path::new(".venv/bin/python");

    println!("Processing transaction!");
    let transaction_string = serde_json::to_string(&tx)
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    let output = Command::new(python_path)
        .current_dir("..")
        .arg("-m")
        .arg("py_modules.orchestrator")
        .arg("orchestrator-cli")
        .arg("--transaction")
        .arg(transaction_string)
        .output()
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                format!("Failed calling Python script with error {:?}", e),
            )
        })?;
    Ok(output)
}

pub async fn add_request_with_pool(
    pool: &sqlx::PgPool,
    request: Request,
) -> Result<(), Custom<String>> {
    sqlx::query!(
        r#"
        INSERT INTO requests
        (challenge_id, type_of_request, expected_response, deadline)
        VALUES ($1, $2, $3, $4)
        "#,
        request.challenge_id,
        request.type_of_request as _,
        request.expected_response as _,
        request.deadline
    )
    .execute(pool) 
    .await
    .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;
    Ok(())
}

pub async fn process_request_with_transaction(
    pool: &sqlx::PgPool,
    tx: &Transaction,
) -> Result<(), Custom<String>> {
    let transaction_request = request_from_transaction(&tx).await?;
    add_request_with_pool(pool, transaction_request).await
}

async fn transaction_scheduler(pool: sqlx::PgPool) {
    let mut ticker = interval(Duration::from_secs(5));

    loop {
        ticker.tick().await;

        if let Err(e) = run_scheduler_iteration(&pool).await {
            eprintln!("Scheduler iteration failed: {:?}", e); // TODO: Throw these errors someplace else... Logging perhaps?
            tokio::time::sleep(Duration::from_secs(2)).await; // TODO: see if it makes sense to use a ticker for this?
        }
    }
}

async fn run_scheduler_iteration(pool: &sqlx::PgPool) -> Result<(), Custom<String>> {
    let now = Utc::now().timestamp_millis();

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    // TODO: Move all transactions that are affected to another table - completed transactions or something.
    let transactions = sqlx::query_as!(
        Transaction,
        r#"
        DELETE FROM
            transactions
        WHERE
            scheduled_Time <= $1
        RETURNING
            id,
            challenge_id,
            created_at,
            scheduled_time,
            source_data_location,
            dispatch_location as "dispatch_location: DispatchTarget",
            data_intended_location,
            data_intended_name,
            rows_to_push,
            access_bindings as "access_bindings: Json<Vec<AccessBinding>>",
            challenge_options as "challenge_options: Json<ChallengeOptions>"
        "#,
        now
    )
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| {
        Custom(
            Status::InternalServerError,
            format!("Scheduler to grab transactions from database, {}", e),
        )
    })?;

    for tx in transactions {
        let process_output = process_transaction(&tx).await?;

        let tx_status = match process_output.status.success() {
            true => TransactionStatus::Success,
            false => TransactionStatus::Failed,
        };

        // TODO: We don't even make use of async through this whole thing... do that instead for this one, spawn a new thread? Does it even take long tho?
        // Create db transaction so our transactions are not lost if processing fails halfway through...
        if tx
            .challenge_options
            .makes_requests_on_transaction_push
            .unwrap_or_default()
        {
            process_request_with_transaction(&pool, &tx).await?;
        }

        let completed_tx = CompletedTransaction::from_transaction(
            tx,
            Some(now),
            tx_status,
            Some(
                String::from_utf8(process_output.stdout)
                    .expect("Could not convert py stdout utf8 string!"),
            ),
            Some(
                String::from_utf8(process_output.stderr)
                    .expect("Could not convert py stderr utf8 string!"),
            ),
        );

        insert_completed_transaction(&pool, &completed_tx)
            .await
            .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;
    }
    
    // Commit transaction when we know it is good
    tx.commit()
        .await
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;
    Ok(())
}

pub async fn insert_completed_transaction(
    pool: &sqlx::PgPool,
    tx: &CompletedTransaction,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO completed_transactions (
            challenge_id,
            created_at,
            scheduled_time,
            source_data_location,
            dispatch_location,
            data_intended_location,
            data_intended_name,
            rows_to_push,
            access_bindings,
            challenge_options,
            attempted_at,
            transaction_status,
            stdout,
            stderr
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14
        )
        "#,
        tx.challenge_id,
        tx.created_at,
        tx.scheduled_time,
        tx.source_data_location,
        tx.dispatch_location as _,
        tx.data_intended_location,
        tx.data_intended_name,
        tx.rows_to_push.as_deref(),
        tx.access_bindings as _,
        tx.challenge_options as _,
        tx.attempted_at,
        tx.transaction_status.clone() as TransactionStatus,
        tx.stdout,
        tx.stderr
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub fn scheduler_fairing() -> AdHoc {
    AdHoc::on_ignite("Transaction Scheduler", |rocket| async {
        // We don't use Db<PgPool> here, since connection is only used inside of a request guard
        let db = rocket.state::<Db>().expect("Db not initialized");
        let pool = db.0.clone();
        // TODO: Fix the scheduler completely crapping out on the first failure, it should just move on from it!
        tokio::spawn(async move {
            transaction_scheduler(pool.clone()).await;
        });
        rocket
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing_common::instances::transaction_instance;
    #[tokio::test]
    async fn test_basic_request_from_transaction() {
        let transation = transaction_instance();
        request_from_transaction(&transation)
            .await
            .expect("Could not generate request from transaction");
    }

    // TODO: For future, add test to ensure it also picks different, random RequestTypes for a given transaction
}
