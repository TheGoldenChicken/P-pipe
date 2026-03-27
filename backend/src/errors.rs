use thiserror::Error;
use aws_sdk_sts::operation::assume_role::AssumeRoleError;
use aws_sdk_sts::error::SdkError;

#[derive(Debug, Error)]
pub enum AwsError {
    #[error("Missing environment variable: {0}")]
    VarError(#[from] std::env::VarError),

    #[error("AssumeRole SDK error: {0}")]
    AssumeRoleSdkError(#[from] SdkError<AssumeRoleError>),

    #[error("AssumeRole failed: {0}")]
    AssumeRoleError(String),
}
