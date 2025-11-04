use rocket::serde::json::Json;
use rocket::{delete, get}; // Have to do this as long as src/lib.rs contains `pub mod endpoints;`, as it breaks #[macro_use]
use rocket::{http::Status, response::status::Custom};
use rocket_db_pools::Connection;
use sqlx::types::Json as DbJson;

use crate::schemas::common::{AccessBinding, Db, DispatchTarget};
use crate::schemas::transaction::Transaction;

#[get("/api/transactions")]
pub async fn get_transactions(
    mut db: Connection<Db>,
) -> Result<Json<Vec<Transaction>>, Custom<String>> {
    let transactions = sqlx::query_as!(
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
    .fetch_all(&mut **db)
    .await
    .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    Ok(Json(transactions))
}

#[delete("/api/transactions/<id>")]
pub async fn delete_transaction(mut db: Connection<Db>, id: i32) -> Result<Status, Custom<String>> {
    sqlx::query!("DELETE FROM transactions WHERE id = $1", id)
        .execute(&mut **db)
        .await
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    Ok(Status::NoContent)
}

#[delete("/api/transactions")]
pub async fn destroy_transactions(mut db: Connection<Db>) -> Result<(), Custom<String>> {
    sqlx::query!("DELETE FROM transactions")
        .execute(&mut **db)
        .await
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;
    Ok(())
}
