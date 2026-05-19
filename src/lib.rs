//! Alibaba Cloud OSS (Object Storage Service) Rust SDK.
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use aliyun_oss::client::OSSClient;
//! use aliyun_oss::types::region::Region;
//!
//! # async fn example() -> aliyun_oss::error::Result<()> {
//! let client = OSSClient::builder()
//!     .region(Region::CnHangzhou)
//!     .credentials("your-access-key-id", "your-access-key-secret")
//!     .build()?;
//!
//! let output = client
//!     .bucket("my-bucket")?
//!     .put_object("hello.txt")?
//!     .body("Hello OSS")
//!     .send()
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Features
//!
//! - V4 (HMAC-SHA256) and V1 (HMAC-SHA1) request signing
//! - Full Object CRUD with metadata, ACL, tagging, and server-side encryption
//! - Multipart upload with automatic Content-MD5 computation
//! - Bucket configuration (lifecycle, CORS, policy, encryption, versioning, etc.)
//! - Pre-signed URL generation (V1)
//! - Credentials chain (environment variables, static, custom providers)

pub mod client;
pub mod config;
pub mod error;
pub mod http;
pub mod operations;
pub mod signer;
pub mod types;
pub mod util;
