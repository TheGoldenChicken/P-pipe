use rocket::serde::{Deserialize, Serialize};
use rocket_db_pools::Database;

#[derive(Database)]
#[database("postgres_db")]
pub struct Db(pub sqlx::PgPool);

// Why did we make this Result type??
// type Result<T, E = rocket::response::Debug<sqlx::Error>> = std::result::Result<T, E>;

#[derive(sqlx::Type, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[sqlx(type_name = "dispatch_target", rename_all = "snake_case")] // TODO: CHANGE TO SNAKE CASE, NOT LOWERCASE
pub enum DispatchTarget {
    S3,
    Drive,
}

use sqlx::postgres::{PgHasArrayType, PgTypeInfo};

impl PgHasArrayType for DispatchTarget {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("dispatch_target[]")
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")] // This tells serde to use the "type" field to determine the variant
pub enum AccessBinding {
    S3(S3Binding),
    Drive(DriveBinding),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct S3Binding {
    pub identity: String,
    pub bucket: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DriveBinding {
    pub identity: String,
    pub folder_id: Option<String>,
    pub user_permissions: String, // TODO: Change this to be an enum of all roles in Drive
}

#[derive(sqlx::Type, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[sqlx(type_name = "transaction_status_enum", rename_all = "snake_case")]
pub enum TransactionStatus {
    Success,
    SuccessWithStdout,
    Failed,
}
