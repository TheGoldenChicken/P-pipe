use aws_config::{BehaviorVersion, Region};
use aws_sdk_sts::Client;
use crate::errors::AwsError;

// TODO: Place these in some kind of env file that user sets up in the beginning
// ... makes self hosting easier.
// const AWS_ACCOUNT_ID: &str = "739275472501";
// const AWS_ACCESS_ROLE_NAME: &str = "S3TempAccessRole";

pub async fn create_bucket_sts_token(
    client: &Client,
    bucket_name: &str,
    duration_secs: Option<i32>,
) -> Result<aws_sdk_sts::types::Credentials, AwsError> {
    // Scoped to just this one bucket
    // TODO: Potentially remove Delete and Put, students might not need this... or add it as options...
    let session_policy = format!(r#"{{
        "Version": "2012-10-17",
        "Statement": [{{
            "Effect": "Allow",
            "Action": ["s3:GetObject", "s3:PutObject", "s3:ListBucket", "s3:DeleteObject"],
            "Resource": [
                "arn:aws:s3:::{bucket}",
                "arn:aws:s3:::{bucket}/*"
            ]
        }}]
    }}"#, bucket = bucket_name);

    // TODO: See if this cannot be moved to another place... I mean we already do it in main.rs
    dotenv::dotenv().ok();
    let aws_account_id = std::env::var("AWS_ACCOUNT_ID")?;
    let aws_access_role_name = std::env::var("AWS_ACCESS_ROLE_NAME")?;

    let resp = client
        .assume_role()
        .role_arn(format!("arn:aws:iam::{aws_account_id}:role/{aws_access_role_name}"))
        .role_session_name(format!("access-{}", bucket_name))
        .duration_seconds(duration_secs.unwrap_or(43200))
        .policy(session_policy)  // <-- narrows permissions to this bucket only
        .send()
        .await?;

    resp.credentials()
        .ok_or_else(|| AwsError::AssumeRoleError("AssumeRole response contained no credentials".to_string()))
        .map(|c| c.clone())
}

// TODO: Make logs available to user, not as println!
// STS credentials cannot be extended — this issues a fresh set for the same bucket,
// which the caller should use to replace the expiring ones.
async fn renew_session(
    client: &Client,
    existing: &aws_sdk_sts::types::Credentials,
    bucket_name: &str,
    duration_secs: i32,
) -> Result<aws_sdk_sts::types::Credentials, AwsError> {
    let expiry = existing.expiration().secs();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    if expiry - now > 3600 {
        println!("Session still has {}s (more than one hour) remaining, no renewal needed.", expiry - now);
        return Ok(existing.clone());
    }

    println!("Renewing session for bucket '{}'...", bucket_name);
    create_bucket_sts_token(client, bucket_name, Some(duration_secs)).await
}

// TODO: Consider if this is actually necessary, or we can do without it...
pub async fn create_aws_client() -> aws_sdk_sts::Client {
    dotenv::dotenv().ok();
    let region = std::env::var("AWS_DEFAULT_REGION")
        .expect("AWS_DEFAULT_REGION must be set in .env");
    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(Region::new(region))
        .load()
        .await;
    Client::new(&config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_bucket_sts_token() {
        let client = create_aws_client().await;
        let result = create_bucket_sts_token(&client, "test-bucket", None).await;
        assert!(result.is_ok(), "Failed to create STS token: {:?}", result.err());
    }
}