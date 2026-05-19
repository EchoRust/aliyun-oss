# aliyun-oss

[![Crates.io](https://img.shields.io/crates/v/aliyun-oss.svg)](https://crates.io/crates/aliyun-oss)
[![Docs.rs](https://docs.rs/aliyun-oss/badge.svg)](https://docs.rs/aliyun-oss)
[![License](https://img.shields.io/crates/l/aliyun-oss.svg)](LICENSE)

A fully async, type-safe Rust SDK for [Alibaba Cloud Object Storage Service (OSS)](https://help.aliyun.com/zh/oss).

## Features

- **Async-first** — Built on `tokio` and `reqwest`, all network I/O is non-blocking
- **V4 Sigining** — HMAC-SHA256 request signing with known-answer test verification
- **V1 Sigining** — HMAC-SHA1 signing for legacy compatibility
- **Pre-signed URLs** — Generate time-limited download URLs without sharing credentials
- **Full Object CRUD** — Put, Get, Head, Delete, Copy, Append with metadata, ACL, and SSE support
- **Multipart Upload** — Initiate, upload parts (with automatic Content-MD5), copy parts, complete, and abort
- **Bucket Configuration** — Lifecycle, CORS, policy, encryption, versioning, logging, website, referer, tagging, replication, WORM, TLS, and more
- **Type-safe** — Newtype wrappers for bucket names, object keys, ETags, regions, and storage classes
- **Credentials Chain** — Environment variables, static credentials, and custom provider support
- **Rich Error Handling** — Structured error types with OSS service error parsing and contextual information

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
aliyun-oss = "0.2"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use aliyun_oss::client::OSSClient;
use aliyun_oss::types::region::Region;

#[tokio::main]
async fn main() -> aliyun_oss::error::Result<()> {
    let client = OSSClient::builder()
        .region(Region::CnHangzhou)
        .credentials("your-access-key-id", "your-access-key-secret")
        .build()?;

    // Upload an object
    client
        .bucket("my-bucket")?
        .put_object("hello.txt")?
        .body("Hello, OSS!")
        .content_type("text/plain")
        .send()
        .await?;

    // Download an object
    let output = client
        .bucket("my-bucket")?
        .get_object("hello.txt")?
        .send()
        .await?;

    println!("Downloaded {} bytes", output.body.len());
    Ok(())
}
```

## Authentication

Credentials can be provided in several ways:

```rust
use aliyun_oss::config::credentials::{
    Credentials, EnvironmentCredentialsProvider, CredentialsChain,
};

// From environment variables (OSS_ACCESS_KEY_ID, OSS_ACCESS_KEY_SECRET)
let provider = EnvironmentCredentialsProvider::new();

// Chained fallback: try env first, then static
let chain = CredentialsChain::builder()
    .with(EnvironmentCredentialsProvider::new())
    .with(StaticCredentialsProvider::new(
        Credentials::builder()
            .access_key_id("ak")
            .access_key_secret("sk")
            .build()?
    ))
    .build();

// Or directly in the builder
let client = OSSClient::builder()
    .region(Region::CnHangzhou)
    .credentials("your-ak", "your-sk")
    .build()?;
```

For STS temporary credentials:

```rust
let creds = Credentials::builder()
    .access_key_id("sts-ak")
    .access_key_secret("sts-sk")
    .security_token("sts-token")
    .build()?;
```

## Object Operations

### Basic CRUD

```rust
let bucket = client.bucket("my-bucket")?;

// Put
let put = bucket.put_object("data.json")?
    .body(json_bytes)
    .content_type("application/json")
    .acl(ObjectAcl::Private)
    .storage_class(StorageClass::Standard)
    .metadata("x-oss-meta-author", "echo")
    .send().await?;

// Get
let get = bucket.get_object("data.json")?
    .range("bytes=0-1023")
    .if_match("\"etag-value\"")
    .send().await?;

// Head (metadata only, no body)
let head = bucket.head_object("data.json")?
    .send().await?;

// Delete
bucket.delete_object("data.json")?.send().await?;
```

### Copy

```rust
bucket.put_object("dest.txt")?
    .copy_source("/source-bucket/source-key")
    .send().await?;
```

### Append

```rust
bucket.append_object("log.txt", 0)?  // position = 0 for first append
    .body("first chunk")
    .send().await?;

bucket.append_object("log.txt", 11)? // position = current object size
    .body("second chunk")
    .send().await?;
```

### List Objects

```rust
// V1 listing
let objects = bucket.list_objects()
    .prefix("photos/")
    .delimiter("/")
    .max_keys(100)
    .send().await?;

// V2 listing (supports continuation token)
let objects = bucket.list_objects_v2()
    .prefix("photos/")
    .start_after("photos/img_001.jpg")
    .max_keys(50)
    .send().await?;

// List all versions (requires versioning enabled)
let versions = bucket.list_object_versions()
    .prefix("data/")
    .max_keys(100)
    .send().await?;
```

### Tagging

```rust
bucket.put_object_tagging("file.txt")?
    .tag("env", "production")
    .tag("region", "cn-hangzhou")
    .send().await?;

let tags = bucket.get_object_tagging("file.txt")?.send().await?;
bucket.delete_object_tagging("file.txt")?.send().await?;
```

### Object ACL

```rust
// Get ACL
let acl = bucket.get_object_acl("file.txt")?.send().await?;

// Set ACL
bucket.put_object_acl("file.txt", ObjectAcl::PublicRead)?.send().await?;
```

### Symlink

```rust
bucket.put_symlink("link.txt", "target.txt")?.send().await?;
let sym = bucket.get_symlink("link.txt")?.send().await?;
```

### Restore (Archive/ColdArchive)

```rust
bucket.restore_object("archive-file.bin", 3)?  // restore for 3 days
    .tier("Standard")
    .send().await?;
```

### Batch Delete

```rust
let result = bucket.delete_multiple_objects(vec![
    "file1.txt".into(),
    "file2.txt".into(),
]).quiet(true).send().await?;
```

### Image Processing

```rust
let processed = bucket.process_object("photo.jpg", "image/resize,m_fixed,w_200")?
    .send().await?;
```

## Multipart Upload

For files larger than 5 GiB or when resumable upload is needed:

```rust
// Initiate
let init = bucket.initiate_multipart_upload("large-file.bin")?
    .content_type("application/octet-stream")
    .send().await?;

// Upload parts (Content-MD5 is computed automatically)
let part1 = bucket.upload_part("large-file.bin", &init.upload_id, 1)?
    .body(chunk1)
    .send().await?;

let part2 = bucket.upload_part("large-file.bin", &init.upload_id, 2)?
    .body(chunk2)
    .send().await?;

// Copy a part from another object
let copied = bucket.upload_part_copy("large-file.bin", &init.upload_id, 3)?
    .copy_source("/other-bucket/source-key")
    .send().await?;

// Complete
let complete = bucket.complete_multipart_upload("large-file.bin", &init.upload_id)?
    .part(1, &part1.etag)
    .part(2, &part2.etag)
    .part(3, &copied.etag)
    .send().await?;

// List parts
let parts = bucket.list_parts("large-file.bin", &init.upload_id)?
    .max_parts(100)
    .send().await?;

// List all multipart uploads
let uploads = bucket.list_multipart_uploads()
    .max_uploads(50)
    .send().await?;

// Abort if needed
bucket.abort_multipart_upload("large-file.bin", &init.upload_id)?
    .send().await?;
```

## Bucket Operations

### Create and manage

```rust
// Create a bucket
bucket.create()
    .acl(BucketAcl::Private)
    .storage_class(StorageClass::Standard)
    .data_redundancy(DataRedundancyType::LRS)
    .send().await?;

// Get bucket info
let info = bucket.get_info().send().await?;

// Get bucket statistics
let stat = bucket.get_stat().send().await?;

// Delete bucket
bucket.delete().send().await?;
```

### Access Control

```rust
bucket.put_acl(BucketAcl::PublicRead).send().await?;
let acl = bucket.get_acl().send().await?;
```

### Versioning

```rust
bucket.put_versioning("Enabled").send().await?;
let status = bucket.get_versioning().send().await?;
```

### Lifecycle

```rust
let rules = vec![LifecycleRule {
    id: Some("expire-old-logs".into()),
    prefix: Some("logs/".into()),
    status: LifecycleRuleStatus::Enabled,
    expiration_days: Some(30),
    expiration_date: None,
    abort_multipart_upload_days: Some(7),
}];
bucket.put_lifecycle(rules).send().await?;
let rules = bucket.get_lifecycle().send().await?;
bucket.delete_lifecycle().send().await?;
```

### CORS

```rust
bucket.put_cors(vec![CorsRule {
    allowed_origins: vec!["*".into()],
    allowed_methods: vec!["GET".into(), "PUT".into()],
    allowed_headers: vec!["*".into()],
    expose_headers: vec![],
    max_age_seconds: Some(3600),
}]).send().await?;

let rules = bucket.get_cors().send().await?;
bucket.delete_cors().send().await?;
```

### Policy, Encryption, Website, Logging, Referer, Tags

```rust
bucket.put_policy(r#"{"Version":"1","Statement":[]}"#.into()).send().await?;
let policy = bucket.get_policy().send().await?;

bucket.put_encryption(ServerSideEncryptionConfiguration {
    sse_algorithm: "AES256".into(),
    kms_master_key_id: None,
}).send().await?;

bucket.put_website("index.html")?
    .error_document("error.html")
    .send().await?;

bucket.put_logging()
    .target_bucket("log-bucket")
    .target_prefix("access-log/")
    .send().await?;

bucket.put_referer()
    .add_referer("https://example.com")
    .send().await?;

bucket.put_tags()
    .tag("project", "aliyun-oss")
    .tag("env", "production")
    .send().await?;
```

## Pre-signed URLs

Generate time-limited URLs for sharing objects without exposing credentials:

```rust
// Build a pre-signed GET URL (valid for 1 hour)
let url = client
    .presign("my-bucket", "secret-file.pdf").await
    .method("GET")
    .expires(std::time::Duration::from_secs(3600))
    .query_param("response-content-disposition", "attachment")
    .generate()
    .await?;

println!("Download URL: {}", url);

// V4 pre-signed URL
let v4_url = client
    .presign("my-bucket", "secret-file.pdf")
    .generate_v4()?;
```

## Service Operations

```rust
// List all buckets
let buckets = client.list_buckets()?
    .prefix("my-project-")
    .max_keys(100)
    .send()
    .await?;
```

## Error Handling

All fallible operations return `aliyun_oss::error::Result<T>`, which is an alias for `std::result::Result<T, OssError>`.

```rust
use aliyun_oss::error::{OssError, OssErrorKind};

match client.bucket("my-bucket")?.get_object("key.txt")?.send().await {
    Ok(output) => println!("Got {} bytes", output.body.len()),
    Err(err) => match err.kind {
        OssErrorKind::ServiceError(ref se) => {
            eprintln!("OSS error {}: {}", se.status_code, se.message);
        }
        OssErrorKind::ValidationError => eprintln!("Invalid input"),
        _ => eprintln!("{}", err),
    },
}
```

## Regions

Supported regions via the `Region` enum:

```rust
use aliyun_oss::types::region::Region;

Region::CnHangzhou      // oss-cn-hangzhou.aliyuncs.com
Region::CnShanghai       // oss-cn-shanghai.aliyuncs.com
Region::CnBeijing        // oss-cn-beijing.aliyuncs.com
Region::CnShenzhen       // oss-cn-shenzhen.aliyuncs.com
Region::ApSingapore      // oss-ap-southeast-1.aliyuncs.com
// ... 37+ regions supported

// Custom endpoint
Region::Custom {
    endpoint: "oss-cn-wulanchabu.aliyuncs.com".into(),
    region_id: "cn-wulanchabu".into(),
}
```

## Minimum Supported Rust Version

Rust **1.85+** (Edition 2024).

## License

This project is licensed under the [MIT License](LICENSE).
