//! Shared test helpers. Lives in `tests/common/` (a module, not its own test
//! binary), included via `mod common;` in the test files that need it.
//!
//! Most importantly this is where the mock access backend lives — under
//! `tests/`, so it is physically absent from the production binary. There is no
//! build in which the real server can reach it; the only way it is ever used is
//! a test explicitly managing it before ignite (see `client_with_mock_access`).

use std::sync::Once;
use std::time::Duration;

use async_trait::async_trait;
use rocket::local::asynchronous::Client;
use sqlx::postgres::PgConnectOptions;

use backend::dispatch::{AccessBackend, AccessError, Location, Principal};
use backend::schemas::common::{AWSSTS, AccessType, DispatchTarget};
use backend::testing_common::connect::async_client_with_access;

// TODO: Figure out a way to implement this 'MockAccess' for multiple access types without many issues...

/// A fake `AccessBackend` that returns canned credentials and never calls AWS.
pub struct MockAccess;

#[async_trait]
impl AccessBackend for MockAccess {
    fn target(&self) -> DispatchTarget {
        DispatchTarget::S3
    }

    async fn grant(
        &self,
        _loc: &Location,
        _who: &Principal,
        _ttl: Duration,
    ) -> Result<AccessType, AccessError> {
        Ok(AccessType::STS(AWSSTS {
            access_key: "MOCK_ACCESS_KEY".to_string(),
            secret_key: "MOCK_SECRET_KEY".to_string(),
            session_token: "MOCK_SESSION_TOKEN".to_string(),
            expires: 0,
        }))
    }

    async fn revoke(&self, _grant: &AccessType) -> Result<(), AccessError> {
        Ok(())
    }
}

static BUCKET_ONCE: Once = Once::new();


// TODO: This is an ass way of doing this, should be set in another way...
/// `add_challenge` reads `P_PIPE_S3_BUCKET` to build transaction destinations —
/// this is independent of the access backend, so the mock alone isn't enough.
/// Set a dummy so the endpoint works with no real AWS config at all.
fn ensure_test_bucket() {
    BUCKET_ONCE.call_once(|| {
        // SAFETY: `Once` runs this exactly once and completes before returning,
        // so the write happens-before every later `env::var` read; nothing else
        // in the test binary writes the environment concurrently.
        unsafe {
            std::env::set_var("P_PIPE_S3_BUCKET", "mock-test-bucket");
        }
    });
}

/// A Rocket test client whose access backend is the no-AWS [`MockAccess`], with
/// the bucket env var stubbed. Endpoints that mint grants work fully offline.
pub async fn client_with_mock_access(pg_connect_options: PgConnectOptions) -> Client {
    ensure_test_bucket();
    async_client_with_access(pg_connect_options, Box::new(MockAccess)).await
}
