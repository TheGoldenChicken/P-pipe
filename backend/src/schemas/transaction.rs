use rocket::serde::{Deserialize, Serialize};
use sqlx::types::Json;

use super::common::{AccessBinding, DispatchTarget, TransactionStatus};

#[derive(Serialize, Deserialize, Clone, Debug, sqlx::FromRow)]
pub struct Transaction {
    // "Bookkeeping fields"
    pub id: Option<i32>,
    pub challenge_id: i32,
    pub created_at: Option<i64>,

    // Transaction info fields
    pub scheduled_time: i64,
    pub source_data_location: Option<String>,
    pub dispatch_location: Option<DispatchTarget>,
    pub data_intended_location: String,
    pub data_intended_name: Option<String>,
    pub rows_to_push: Option<Vec<i32>>,

    pub access_bindings: Option<Json<Vec<AccessBinding>>>,
}

// Custom PartialEq function so we can test if transactions from the db (with id and created_at) match those we expect to create (from transactions_from_challenge)
impl PartialEq for Transaction {
    fn eq(&self, other: &Self) -> bool {
        self.challenge_id == other.challenge_id
            && self.scheduled_time == other.scheduled_time
            && self.source_data_location == other.source_data_location
            && self.data_intended_location == other.data_intended_location
            && self.rows_to_push == other.rows_to_push
            && self.access_bindings == other.access_bindings
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, sqlx::FromRow)]
pub struct CompletedTransaction {
    pub id: Option<i32>,
    pub challenge_id: i32,
    pub created_at: Option<i64>,

    pub scheduled_time: i64,
    pub source_data_location: Option<String>,
    pub dispatch_location: Option<DispatchTarget>,
    pub data_intended_location: String,
    pub data_intended_name: Option<String>,
    pub rows_to_push: Option<Vec<i32>>,

    pub access_bindings: Option<Json<Vec<AccessBinding>>>,

    // Status fields - for completed transactions
    pub attempted_at: Option<i64>,
    pub transaction_status: TransactionStatus,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

impl CompletedTransaction {
    pub fn from_transaction(
        tx: Transaction,
        attempted_at: Option<i64>,
        transaction_status: TransactionStatus,
        stdout: Option<String>,
        stderr: Option<String>,
    ) -> Self {
        CompletedTransaction {
            id: tx.id,
            challenge_id: tx.challenge_id,
            created_at: tx.created_at,
            scheduled_time: tx.scheduled_time,
            source_data_location: tx.source_data_location,
            dispatch_location: tx.dispatch_location,
            data_intended_location: tx.data_intended_location,
            data_intended_name: tx.data_intended_name,
            rows_to_push: tx.rows_to_push,
            access_bindings: tx.access_bindings,
            attempted_at,
            transaction_status,
            stdout,
            stderr,
        }
    }
}
