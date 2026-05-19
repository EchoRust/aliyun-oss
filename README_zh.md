# aliyun-oss

[![Crates.io](https://img.shields.io/crates/v/aliyun-oss.svg)](https://crates.io/crates/aliyun-oss)
[![Docs.rs](https://docs.rs/aliyun-oss/badge.svg)](https://docs.rs/aliyun-oss)
[![License](https://img.shields.io/crates/l/aliyun-oss.svg)](LICENSE)

[阿里云对象存储 OSS](https://www.aliyun.com/product/oss) 的 Rust 原生异步 SDK。类型安全、全异步、覆盖 80+ API 操作。

## 特性

- **全异步** — 基于 `tokio` 和 `reqwest`，所有网络 I/O 非阻塞
- **V4 签名** — HMAC-SHA256 请求签名，通过官方已知答案测试验证
- **V1 签名** — HMAC-SHA1 签名，兼容旧版
- **预签名 URL** — 生成有时效的下载链接，无需暴露凭证
- **对象 CRUD** — Put、Get、Head、Delete、Copy、Append，支持元数据、ACL、服务端加密
- **分片上传** — 初始化、上传分片（自动 Content-MD5）、拷贝分片、完成、中止
- **Bucket 配置** — 生命周期、CORS、Policy、加密、版本控制、日志、网站、防盗链、标签、复制、WORM、TLS 等
- **类型安全** — Bucket 名称、Object Key、ETag、Region、存储类型均用 newtype 封装
- **凭证链** — 支持环境变量、静态凭证、自定义 Provider
- **结构化错误** — 包含 OSS 服务端错误解析和上下文信息的错误类型

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
aliyun-oss = "0.2"
tokio = { version = "1", features = ["full"] }
```

## 快速开始

```rust
use aliyun_oss::client::OSSClient;
use aliyun_oss::types::region::Region;

#[tokio::main]
async fn main() -> aliyun_oss::error::Result<()> {
    let client = OSSClient::builder()
        .region(Region::CnHangzhou)
        .credentials("your-access-key-id", "your-access-key-secret")
        .build()?;

    // 上传对象
    client
        .bucket("my-bucket")?
        .put_object("hello.txt")?
        .body("Hello, OSS!")
        .content_type("text/plain")
        .send()
        .await?;

    // 下载对象
    let output = client
        .bucket("my-bucket")?
        .get_object("hello.txt")?
        .send()
        .await?;

    println!("下载了 {} 字节", output.body.len());
    Ok(())
}
```

## 认证方式

```rust
use aliyun_oss::config::credentials::{
    Credentials, EnvironmentCredentialsProvider, CredentialsChain,
};

// 从环境变量读取 (OSS_ACCESS_KEY_ID, OSS_ACCESS_KEY_SECRET)
let provider = EnvironmentCredentialsProvider::new();

// 链式回退：先尝试环境变量，再尝试静态凭证
let chain = CredentialsChain::builder()
    .with(EnvironmentCredentialsProvider::new())
    .with(StaticCredentialsProvider::new(
        Credentials::builder()
            .access_key_id("ak")
            .access_key_secret("sk")
            .build()?
    ))
    .build();

// 或直接在 Builder 中指定
let client = OSSClient::builder()
    .region(Region::CnHangzhou)
    .credentials("your-ak", "your-sk")
    .build()?;
```

STS 临时凭证：

```rust
let creds = Credentials::builder()
    .access_key_id("sts-ak")
    .access_key_secret("sts-sk")
    .security_token("sts-token")
    .build()?;
```

## 对象操作

### 基本 CRUD

```rust
let bucket = client.bucket("my-bucket")?;

// 上传
let put = bucket.put_object("data.json")?
    .body(json_bytes)
    .content_type("application/json")
    .acl(ObjectAcl::Private)
    .storage_class(StorageClass::Standard)
    .metadata("x-oss-meta-author", "echo")
    .send().await?;

// 下载
let get = bucket.get_object("data.json")?
    .range("bytes=0-1023")
    .send().await?;

// 获取元数据（不含 Body）
let head = bucket.head_object("data.json")?.send().await?;

// 删除
bucket.delete_object("data.json")?.send().await?;
```

### 拷贝

```rust
bucket.put_object("dest.txt")?
    .copy_source("/source-bucket/source-key")
    .send().await?;
```

### 追加上传

```rust
bucket.append_object("log.txt", 0)?   // position = 0 表示首次追加
    .body("first chunk")
    .send().await?;

bucket.append_object("log.txt", 11)?  // position = 对象当前大小
    .body("second chunk")
    .send().await?;
```

### 列举对象

```rust
// V1 列举
let objects = bucket.list_objects()
    .prefix("photos/")
    .delimiter("/")
    .max_keys(100)
    .send().await?;

// V2 列举（支持 continuation token）
let objects = bucket.list_objects_v2()
    .prefix("photos/")
    .start_after("photos/img_001.jpg")
    .max_keys(50)
    .send().await?;

// 列举所有版本（需先启用版本控制）
let versions = bucket.list_object_versions()
    .max_keys(100)
    .send().await?;
```

### 标签

```rust
bucket.put_object_tagging("file.txt")?
    .tag("env", "production")
    .tag("region", "cn-hangzhou")
    .send().await?;

let tags = bucket.get_object_tagging("file.txt")?.send().await?;
bucket.delete_object_tagging("file.txt")?.send().await?;
```

### 对象 ACL

```rust
let acl = bucket.get_object_acl("file.txt")?.send().await?;
bucket.put_object_acl("file.txt", ObjectAcl::PublicRead)?.send().await?;
```

### 软链接

```rust
bucket.put_symlink("link.txt", "target.txt")?.send().await?;
let sym = bucket.get_symlink("link.txt")?.send().await?;
```

### 归档解冻

```rust
bucket.restore_object("archive.bin", 3)?  // 解冻 3 天
    .tier("Standard")
    .send().await?;
```

### 批量删除

```rust
bucket.delete_multiple_objects(vec![
    "file1.txt".into(),
    "file2.txt".into(),
]).send().await?;
```

### 图片处理

```rust
let img = bucket.process_object("photo.jpg", "image/resize,m_fixed,w_200")?
    .send().await?;
```

## 分片上传

适用于大于 5 GiB 的文件或需要断点续传的场景：

```rust
// 初始化
let init = bucket.initiate_multipart_upload("large-file.bin")?
    .content_type("application/octet-stream")
    .send().await?;

// 上传分片（Content-MD5 自动计算）
let part1 = bucket.upload_part("large-file.bin", &init.upload_id, 1)?
    .body(chunk1)
    .send().await?;

let part2 = bucket.upload_part("large-file.bin", &init.upload_id, 2)?
    .body(chunk2)
    .send().await?;

// 从其他对象拷贝分片
let copied = bucket.upload_part_copy("large-file.bin", &init.upload_id, 3)?
    .copy_source("/other-bucket/source-key")
    .send().await?;

// 完成上传
bucket.complete_multipart_upload("large-file.bin", &init.upload_id)?
    .part(1, &part1.etag)
    .part(2, &part2.etag)
    .part(3, &copied.etag)
    .send().await?;

// 列举分片
let parts = bucket.list_parts("large-file.bin", &init.upload_id)?
    .send().await?;

// 列举所有分片上传任务
let uploads = bucket.list_multipart_uploads().send().await?;

// 中止上传
bucket.abort_multipart_upload("large-file.bin", &init.upload_id)?
    .send().await?;
```

## Bucket 操作

### 创建与管理

```rust
bucket.create()
    .acl(BucketAcl::Private)
    .storage_class(StorageClass::Standard)
    .data_redundancy(DataRedundancyType::LRS)
    .send().await?;

let info = bucket.get_info().send().await?;
let stat = bucket.get_stat().send().await?;
bucket.delete().send().await?;
```

### 访问控制

```rust
bucket.put_acl(BucketAcl::PublicRead).send().await?;
let acl = bucket.get_acl().send().await?;
```

### 版本控制

```rust
bucket.put_versioning("Enabled").send().await?;
let status = bucket.get_versioning().send().await?;
```

### 生命周期

```rust
bucket.put_lifecycle(vec![LifecycleRule {
    id: Some("expire-logs".into()),
    prefix: Some("logs/".into()),
    status: LifecycleRuleStatus::Enabled,
    expiration_days: Some(30),
    expiration_date: None,
    abort_multipart_upload_days: None,
}]).send().await?;

let rules = bucket.get_lifecycle().send().await?;
bucket.delete_lifecycle().send().await?;
```

### CORS、Policy、加密、网站、日志、防盗链、标签

```rust
bucket.put_cors(vec![CorsRule {
    allowed_origins: vec!["*".into()],
    allowed_methods: vec!["GET".into()],
    allowed_headers: vec!["*".into()],
    expose_headers: vec![],
    max_age_seconds: Some(3600),
}]).send().await?;

bucket.put_policy(r#"{"Version":"1","Statement":[]}"#.into()).send().await?;

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
    .send().await?;
```

## 预签名 URL

生成有时效的下载链接，无需暴露 AccessKey：

```rust
// V4 预签名 GET URL（有效期 1 小时）
let url = client
    .presign("my-bucket", "secret-file.pdf")
    .method("GET")
    .expires(std::time::Duration::from_secs(3600))
    .generate_v4()?;

// V1 预签名 URL
let url = client
    .presign("my-bucket", "secret-file.pdf")
    .generate_v1()?;
```

## 服务级操作

```rust
let buckets = client.list_buckets()?
    .prefix("my-project-")
    .max_keys(100)
    .send().await?;
```

## 错误处理

所有可能失败的操作返回 `aliyun_oss::error::Result<T>`（等价于 `std::result::Result<T, OssError>`）。

```rust
use aliyun_oss::error::{OssError, OssErrorKind};

match client.bucket("my-bucket")?.get_object("key.txt")?.send().await {
    Ok(output) => println!("获取 {} 字节", output.body.len()),
    Err(err) => match err.kind {
        OssErrorKind::ServiceError(ref se) => {
            eprintln!("OSS 错误 {}: {}", se.status_code, se.message);
        }
        OssErrorKind::ValidationError => eprintln!("输入参数无效"),
        _ => eprintln!("{}", err),
    },
}
```

## 支持的地域

通过 `Region` 枚举支持 37+ 个地域：

```rust
use aliyun_oss::types::region::Region;

Region::CnHangzhou      // oss-cn-hangzhou.aliyuncs.com
Region::CnShanghai       // oss-cn-shanghai.aliyuncs.com
Region::CnBeijing        // oss-cn-beijing.aliyuncs.com
Region::CnShenzhen       // oss-cn-shenzhen.aliyuncs.com
Region::ApSingapore      // oss-ap-southeast-1.aliyuncs.com

// 自定义 endpoint
Region::Custom {
    endpoint: "oss-cn-wulanchabu.aliyuncs.com".into(),
    region_id: "cn-wulanchabu".into(),
}
```

## 最低 Rust 版本

Rust **1.85+** (Edition 2024)。

## 许可证

本项目采用 [MIT License](LICENSE)。
