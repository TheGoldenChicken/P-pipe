use aws_config::BehaviorVersion;
use aws_sdk_sts::Client;

// TODO: Place these in some kind of env file that user sets up in the beginning
// ... makes self hosting easier.
// const AWS_ACCOUNT_ID: &str = "739275472501";
// const AWS_ACCESS_ROLE_NAME: &str = "S3TempAccessRole";

pub async fn create_bucket_STS_token(
    client: &Client,
    bucket_name: &str,
    duration_secs: Option<i32>,
) -> Result<aws_sdk_sts::types::Credentials, String> {
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

    let AWS_account_id = std::env::var("AWS_ACCOUNT_ID").map_err(|e| e.to_string())?;
    let AWS_access_role_name = std::env::var("AWS_ACCESS_ROLE_NAME").map_err(|e| e.to_string())?;

    let resp = client
        .assume_role()
        .role_arn(format!("arn:aws:iam::{AWS_account_id}:role/{AWS_access_role_name}"))
        .role_session_name(format!("access-{}", bucket_name))
        .duration_seconds(duration_secs.unwrap_or(36000))
        .policy(session_policy)  // <-- narrows permissions to this bucket only
        .send()
        .await
        .map_err(|e| e.to_string())?;


    Ok(resp.credentials().unwrap().clone())
}

// STS credentials cannot be extended — this issues a fresh set for the same bucket,
// which the caller should use to replace the expiring ones.
async fn renew_session(
    client: &Client,
    existing: &aws_sdk_sts::types::Credentials,
    bucket_name: &str,
    duration_secs: i32,
) -> Result<aws_sdk_sts::types::Credentials, String> {
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
    create_bucket_STS_token(client, bucket_name, Some(duration_secs)).await
}

// TODO: Consider if this is actually necessary, or we can do without it...
pub async fn create_AWS_client() -> aws_sdk_sts::Client {
    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    Client::new(&config)
}

// #[tokio::main]
// async fn main() -> Result<(), aws_sdk_sts::Error> {
//     let client = create_AWS_client();
//     grant_bucket_access(&client, "challenge-1-iris-classification", 3600).await?;
//     Ok(())
// }
