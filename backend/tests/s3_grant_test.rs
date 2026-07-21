//! Live S3 integration tests for the access-granting path.
//!
//! These hit REAL AWS: they create a throwaway bucket, upload a dummy object,
//! mint an STS grant scoped to its prefix, prove the scoped credentials can read
//! that object, and tear the bucket back down. They do NOT test delivery of the
//! token to a student — only that a grant, once made, works against S3.
//!
//! Opt-in: set `P_PIPE_RUN_LIVE_S3=1` (plus AWS credentials in the environment
//! and the bucket's region in `AWS_DEFAULT_REGION`). Without the flag they skip,
//! so the normal suite is unaffected.
//!
//! `grant_opens_its_own_prefix` provisions its own uniquely-named bucket and
//! needs nothing else. `endpoint_grant_opens_challenge_prefix` also creates and
//! destroys its bucket, but takes the *name* from `P_PIPE_S3_BUCKET` because that
//! is where the endpoint (`add_challenge`) reads it; it deletes the bucket only
//! if it was the one that created it, so pointing it at a real bucket is safe.

use aws_config::BehaviorVersion;
use aws_sdk_s3::config::{Credentials, Region};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::{BucketLocationConstraint, CreateBucketConfiguration};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use std::time::Duration;
use uuid::Uuid;

use backend::dispatch::s3::S3Access;
use backend::dispatch::{AccessBackend, Location, Principal};
use backend::schemas::challenge::Challenge;
use backend::schemas::common::{AWSSTS, AccessType};
use backend::testing_common::connect::async_client_from_pg_connect_options;
use backend::testing_common::instances::challenge_instance;

// TODO: Ensure that the tests here can only run on actual testing buckets created by the functions here - Don't want this deleting 
// Buckets that are *actually* in production

// TODO: Potentially have a flag which is set during testing to determine whether these tests run or not, this is kinda silly
fn live_enabled() -> bool {
    if std::env::var("P_PIPE_RUN_LIVE_S3").is_err() {
        eprintln!("skipping live S3 test — set P_PIPE_RUN_LIVE_S3=1 (and AWS creds) to run");
        return false;
    }
    true
}

// TODO: No need to have own function
async fn admin_s3() -> aws_sdk_s3::Client {
    aws_sdk_s3::Client::new(&aws_config::load_defaults(BehaviorVersion::latest()).await)
}

// TODO: No need to have own function
async fn admin_sts() -> aws_sdk_sts::Client {
    aws_sdk_sts::Client::new(&aws_config::load_defaults(BehaviorVersion::latest()).await)
}

/// An S3 client authenticating with the temporary, prefix-scoped creds a grant
/// produced — i.e. what a student would use.
fn scoped_s3(creds: &AWSSTS) -> aws_sdk_s3::Client {
    let temp = Credentials::new(
        creds.access_key.clone(),
        creds.secret_key.clone(),
        Some(creds.session_token.clone()),
        None,
        "p-pipe-live-test",
    );
    let conf = aws_sdk_s3::Config::builder()
        .region(Region::new("eu-central-1".to_string())) // Less robust than having use env variable that gives its own, but usually we just want eu-central-1 tbh
        .credentials_provider(temp)
        .behavior_version(BehaviorVersion::latest())
        .build();
    aws_sdk_s3::Client::from_conf(conf)
}

/// Create `bucket`. Returns `true` if we created it (caller owns teardown),
/// `false` if it already existed and is ours (leave it alone).
async fn create_bucket(client: &aws_sdk_s3::Client, bucket: &str) -> bool {
    let mut req = client.create_bucket().bucket(bucket);
    // us-east-1 must NOT carry a LocationConstraint; every other region must.
    let region = "eu-central-1".to_string();
    if region != "us-east-1" {
        req = req.create_bucket_configuration(
            CreateBucketConfiguration::builder()
                .location_constraint(BucketLocationConstraint::from(region.as_str()))
                .build(),
        );
    }
    match req.send().await {
        Ok(_) => true,
        Err(err) => {
            if err.as_service_error().is_some_and(|e| e.is_bucket_already_owned_by_you()) {
                false
            } else {
                panic!("create_bucket({bucket}) failed: {err:?}");
            }
        }
    }
}

/// Empty a bucket (single unpaginated page is plenty for these tests) and delete it.
async fn empty_and_delete_bucket(client: &aws_sdk_s3::Client, bucket: &str) {
    if let Ok(list) = client.list_objects_v2().bucket(bucket).send().await {
        for obj in list.contents() {
            if let Some(key) = obj.key() {
                let _ = client.delete_object().bucket(bucket).key(key).send().await;
            }
        }
    }
    let _ = client.delete_bucket().bucket(bucket).send().await;
}

/// Read `bucket/key` with the scoped creds, returning the bytes or an error string.
async fn scoped_read(creds: &AWSSTS, bucket: &str, key: &str) -> Result<Vec<u8>, String> {
    match scoped_s3(creds).get_object().bucket(bucket).key(key).send().await {
        Ok(out) => out
            .body
            .collect()
            .await
            .map(|b| b.into_bytes().to_vec())
            .map_err(|e| e.to_string()),
        // TODO: Potentially have a this error for this error
        Err(e) => Err(format!("{e:?}")),
    }
}

/// Grant, in isolation: create a bucket, drop dummy data, mint an STS token for
/// its prefix, prove the token reads it, then destroy the bucket.
#[tokio::test]
async fn grant_opens_its_own_prefix() {
    if !live_enabled() {
        return;
    }

    let admin = admin_s3().await;
    let access = S3Access::new(admin_sts().await);
    let bucket = format!("p-pipe-livetest-{}", Uuid::new_v4());
    create_bucket(&admin, &bucket).await;

    // TODO: Change the map_err txts here to be a bit more informative
    let outcome: Result<Vec<u8>, String> = async {
        let prefix = "challenge-1".to_string();
        let key = format!("{prefix}/dummy.txt");
        admin
            .put_object()
            .bucket(&bucket)
            .key(&key)
            .body(ByteStream::from_static(b"hello"))
            .send()
            .await
            .map_err(|e| format!("upload dummy: {e:?}"))?;

        let loc = Location { root: bucket.clone(), prefix };
        let who = Principal { challenge_id: 1, emails: vec![] };
        let AccessType::STS(creds) = access
            .grant(&loc, &who, Duration::from_secs(900))
            .await
            .map_err(|e| format!("grant: {e:?}"))?;

        scoped_read(&creds, &bucket, &key).await
    }
    .await;

    empty_and_delete_bucket(&admin, &bucket).await;

    assert_eq!(
        outcome.expect("scoped credentials should grant access to their own prefix"),
        b"hello",
        "read-back content did not match upload",
    );
}

/// The same, but through the endpoint: POST a challenge, take the grant the
/// endpoint minted, and prove it opens that challenge's prefix. The bucket name
/// comes from `P_PIPE_S3_BUCKET` (what the endpoint reads); we create it and, if
/// we created it, destroy it afterward.
#[sqlx::test(migrations = "src/migrations")]
async fn endpoint_grant_opens_challenge_prefix(
    _: PgPoolOptions,
    pg_connect_options: PgConnectOptions,
) -> sqlx::Result<()> {
    if !live_enabled() {
        return Ok(());
    }

    let bucket = std::env::var("P_PIPE_S3_BUCKET")
        .expect("live endpoint test needs P_PIPE_S3_BUCKET set to a bucket name to use/create");
    let admin = admin_s3().await;
    let we_created_it = create_bucket(&admin, &bucket).await;
    let client = async_client_from_pg_connect_options(pg_connect_options).await;

    let mut uploaded_key: Option<String> = None;

    let outcome: Result<Vec<u8>, String> = async {
        let response = client
            .post("/api/challenges")
            .json(&challenge_instance())
            .dispatch()
            .await;
        let challenges: Vec<Challenge> = response
            .into_json()
            .await
            .ok_or("deserialize challenge response failed")?;
        let created = challenges.first().ok_or("no challenge returned from POST")?;
        let challenge_id = created.id.ok_or("created challenge has no id")?;
        let AccessType::STS(creds) = created
            .access_types
            .0
            .first()
            .cloned()
            .ok_or("endpoint stored no access grant")?;

        // The endpoint scoped it to "challenge-{id}"; drop dummy data there.
        let key = format!("challenge-{challenge_id}/dummy.txt");
        admin
            .put_object()
            .bucket(&bucket)
            .key(&key)
            .body(ByteStream::from_static(b"hello"))
            .send()
            .await
            .map_err(|e| format!("upload dummy: {e:?}"))?;
        uploaded_key = Some(key.clone());

        scoped_read(&creds, &bucket, &key).await
    }
    .await;

    // Only remove bucket if the test was the one that craeted it
    if let Some(key) = &uploaded_key {
        let _ = admin.delete_object().bucket(&bucket).key(key).send().await;
    }
    if we_created_it {
        empty_and_delete_bucket(&admin, &bucket).await;
    }

    assert_eq!(
        outcome.expect("endpoint-minted credentials should grant access to their prefix"),
        b"hello",
        "read-back content did not match upload",
    );
    Ok(())
}
