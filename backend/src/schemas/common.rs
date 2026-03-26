use rocket::serde::{Deserialize, Serialize};

use strum_macros::{EnumCount, EnumIter};

#[derive(sqlx::Type, Serialize, Deserialize, Debug, Clone, PartialEq, EnumCount, EnumIter)]
#[sqlx(type_name = "dispatch_target", rename_all = "snake_case")]
pub enum DispatchTarget {
    S3,
    Drive,
}


#[derive(Debug, Clone)]
pub struct S3BucketCredentials {
    pub bucket_name: String,
    pub access_key: String,
    pub secret_key: String,
    pub session_token: String,
    pub expiry: i64, // use creds.expiration().unwrap().secs() to get to fill this...
}


#[derive(sqlx::Type, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[sqlx(type_name = "transaction_status_enum", rename_all = "snake_case")]
pub enum TransactionStatus {
    Success,
    SuccessWithStdout,
    Failed,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")] // This tells serde to use the "type" field to determine the variant
pub enum AccessType {
    STS(AWSSTS),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AWSSTS {
    pub access_key: String,
    pub secret_key: String,
    pub session_token: String,
    pub expires: u64
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
