//! Storage-backend abstraction for dispatch targets.
//!
//! Two orthogonal concerns, split into two traits on purpose:
//!   * `DataSink`      — the DATA plane: write bytes to a location. rclone does
//!                       this today; OpenDAL / fsspec could later.
//!   * `AccessBackend` — the CONTROL plane: grant/revoke a principal's access to
//!                       a location. This is irreducibly per-provider (STS for
//!                       S3, the Drive Permissions API for Drive, ...) and is the
//!                       half rclone fundamentally cannot do.
//!
//! Adding a new backend = one `DispatchTarget` variant + one impl of each trait;
//! the compiler then walks every `match` that needs updating.
//!
//! NOTE: skeleton only. The concrete S3 / Drive impls are deliberately not here
//! yet — this defines the shape they will slot into.

use std::time::Duration;

use async_trait::async_trait;

use crate::schemas::common::{AccessType, DispatchTarget};

pub mod s3;

/// Where data lives / access is granted, abstracted across backends.
///
/// * S3:    `root` = bucket,          `prefix` = "challenge-7" (or ".../group-3")
/// * Drive: `root` = shared-drive id, `prefix` = folder path — the backend
///          resolves this to a folder id, since Drive is id-addressed, not
///          path-addressed.
#[derive(Debug, Clone)]
pub struct Location {
    pub root: String,
    pub prefix: String,
}

/// Who is being granted access, plus enough context to name/scope the grant.
#[derive(Debug, Clone)]
pub struct Principal {
    pub challenge_id: i32,
    pub emails: Vec<String>,
}

/// Errors from the control plane (granting / revoking access).
#[derive(Debug, thiserror::Error)]
pub enum AccessError {
    /// The backend has no way to perform this operation — e.g. STS sessions
    /// cannot be revoked individually.
    #[error("this backend cannot perform that operation")]
    Unsupported,

    #[error("backend returned no credentials")]
    NoCredentials,

    #[error(transparent)]
    Aws(#[from] crate::errors::AwsError),

    #[error("{0}")]
    Other(String),
}

/// Errors from the data plane (writing data).
#[derive(Debug, thiserror::Error)]
pub enum SinkError {
    #[error("this backend cannot perform that operation")]
    Unsupported,

    #[error("{0}")]
    Other(String),
}

/// CONTROL plane: hand a principal time-limited access to a location.
#[async_trait]
pub trait AccessBackend: Send + Sync {
    /// Which dispatch target this backend serves.
    fn target(&self) -> DispatchTarget;

    /// Grant `who` access to `loc` for at most `ttl`, returning the grant to
    /// hand over. `ttl` is a hint — backends clamp to their own ceiling (e.g.
    /// STS caps at 12h).
    async fn grant(
        &self,
        loc: &Location,
        who: &Principal,
        ttl: Duration,
    ) -> Result<AccessType, AccessError>;

    /// Revoke a previously issued grant. Backends that cannot revoke an
    /// individual grant (e.g. STS sessions) return `AccessError::Unsupported`.
    async fn revoke(&self, grant: &AccessType) -> Result<(), AccessError>;
}

/// DATA plane: write data to a location.
#[async_trait]
pub trait DataSink: Send + Sync {
    /// Which dispatch target this backend serves.
    fn target(&self) -> DispatchTarget;

    /// Ensure the destination exists. Creates the directory/folder on backends
    /// that have real directories (Drive, FTP, ...); a no-op for object stores
    /// where prefixes are implicit (S3).
    async fn ensure_location(&self, loc: &Location) -> Result<(), SinkError>;

    /// Write `data` as an object/file named `name` under `loc`.
    async fn put(&self, loc: &Location, name: &str, data: &[u8]) -> Result<(), SinkError>;
}

// Once concrete backends exist, dispatch by target here, e.g.:
//
//   pub fn access_backend_for(target: &DispatchTarget) -> Box<dyn AccessBackend> {
//       match target {
//           DispatchTarget::S3    => Box::new(S3Access::from_env()),
//           DispatchTarget::Drive => Box::new(DriveAccess::from_env()),
//       }
//   }
//
//   pub fn data_sink_for(target: &DispatchTarget) -> Box<dyn DataSink> { ... }
