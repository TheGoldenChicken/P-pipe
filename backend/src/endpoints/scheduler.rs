use chrono::Utc;
use rocket::{fairing::AdHoc, http::Status, response::status::Custom};
use sqlx::types::Json;
use std::path::Path;
use std::process::Command;
use tokio::time::{Duration, interval};

use crate::schemas::common::{AccessBinding, Db, DispatchTarget, TransactionStatus};
use crate::schemas::transaction::{CompletedTransaction, Transaction};
use crate::schemas::challenge::ChallengeOptions;


async fn process_transaction(tx: &Transaction) -> Result<std::process::Output, Custom<String>> {
    // TODO: Move python_path and script_path to env variables
    let python_path = Path::new("../.venv/bin/python");
    let script_path = Path::new("../py_modules/orchestrator.py");

    println!("Processing transaction!");
    let transaction_string = serde_json::to_string(&tx)
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    let output = Command::new(python_path)
        .arg(script_path)
        .arg("orchestrator-cli")
        .arg("--transaction")
        .arg(transaction_string)
        .output();

    // TODO: Ask Hans if there isn't a better way of doing this...
    match output {
        Ok(result) => Ok(result),
        Err(e) => Err(Custom(
            Status::InternalServerError,
            format!("Failed calling Python script with error {:?}", e),
        )),
    }
}

async fn transaction_scheduler(pool: sqlx::PgPool) -> Result<(), Custom<String>> {
    let mut ticker = interval(Duration::from_secs(5));
    loop {
        ticker.tick().await;
        let now = Utc::now().timestamp_millis();

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
        .fetch_all(&pool)
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
    }
}

async fn insert_completed_transaction(
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
        tokio::spawn(async move {
            if let Err(e) = transaction_scheduler(pool.clone()).await {
                eprintln!("Scheduled task failed: {:?}", e);
            }
        });
        rocket
    })
}
