use std::sync::Arc;

use crate::client::{BucketOperations, OSSClientInner};
use crate::error::{ErrorContext, OssError, OssErrorKind, Result};
use crate::http::client::HttpRequest;
use crate::types::acl::ObjectAcl;
use crate::types::bucket::BucketName;
use crate::types::object::ObjectKey;
use crate::types::storage::{ServerSideEncryption, StorageClass};
use crate::util::uri::oss_endpoint_url;

pub struct PutObjectBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    key: ObjectKey,
    body: Option<bytes::Bytes>,
    content_type: Option<String>,
    content_md5: Option<String>,
    cache_control: Option<String>,
    content_disposition: Option<String>,
    content_encoding: Option<String>,
    expires: Option<String>,
    acl: Option<ObjectAcl>,
    storage_class: Option<StorageClass>,
    sse: Option<ServerSideEncryption>,
    sse_key_id: Option<String>,
    tagging: Option<String>,
    metadata: Vec<(String, String)>,
}

impl PutObjectBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName, key: ObjectKey) -> Self {
        Self {
            client,
            bucket,
            key,
            body: None,
            content_type: None,
            content_md5: None,
            cache_control: None,
            content_disposition: None,
            content_encoding: None,
            expires: None,
            acl: None,
            storage_class: None,
            sse: None,
            sse_key_id: None,
            tagging: None,
            metadata: Vec::new(),
        }
    }

    pub fn body(mut self, body: impl Into<bytes::Bytes>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn content_type(mut self, ct: impl Into<String>) -> Self {
        self.content_type = Some(ct.into());
        self
    }

    pub fn content_md5(mut self, md5: impl Into<String>) -> Self {
        self.content_md5 = Some(md5.into());
        self
    }

    pub fn cache_control(mut self, cc: impl Into<String>) -> Self {
        self.cache_control = Some(cc.into());
        self
    }

    pub fn content_disposition(mut self, cd: impl Into<String>) -> Self {
        self.content_disposition = Some(cd.into());
        self
    }

    pub fn content_encoding(mut self, ce: impl Into<String>) -> Self {
        self.content_encoding = Some(ce.into());
        self
    }

    pub fn expires(mut self, exp: impl Into<String>) -> Self {
        self.expires = Some(exp.into());
        self
    }

    pub fn acl(mut self, acl: ObjectAcl) -> Self {
        self.acl = Some(acl);
        self
    }

    pub fn storage_class(mut self, sc: StorageClass) -> Self {
        self.storage_class = Some(sc);
        self
    }

    pub fn server_side_encryption(mut self, sse: impl Into<String>) -> Self {
        match sse.into().as_str() {
            "AES256" => self.sse = Some(ServerSideEncryption::AES256),
            "KMS" => self.sse = Some(ServerSideEncryption::KMS),
            _ => {}
        }
        self
    }

    pub fn sse_key_id(mut self, key_id: impl Into<String>) -> Self {
        self.sse_key_id = Some(key_id.into());
        self
    }

    pub fn tagging(mut self, tag: impl Into<String>) -> Self {
        self.tagging = Some(tag.into());
        self
    }

    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.push((key.into(), value.into()));
        self
    }

    pub async fn send(self) -> Result<PutObjectOutput> {
        let body = self.body.ok_or_else(|| OssError {
            kind: OssErrorKind::ValidationError,
            context: Box::new(ErrorContext {
                operation: Some("PutObject: body is required".into()),
                bucket: Some(self.bucket.to_string()),
                object_key: Some(self.key.to_string()),
                ..Default::default()
            }),
            source: None,
        })?;

        let endpoint = self.client.endpoint.clone();
        let uri = oss_endpoint_url(
            &endpoint,
            Some(self.bucket.as_str()),
            Some(self.key.as_str()),
        );

        let full_uri = uri;

        let mut req = HttpRequest::builder()
            .method(http::Method::PUT)
            .uri(&full_uri);

        if let Some(ref ct) = self.content_type {
            req = req.header(
                http::HeaderName::from_static("content-type"),
                http::HeaderValue::from_str(ct).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set content-type header".into()),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
        }

        if let Some(ref md5) = self.content_md5 {
            req = req.header(
                http::HeaderName::from_static("content-md5"),
                http::HeaderValue::from_str(md5).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set content-md5 header".into()),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
        }

        if let Some(ref cc) = self.cache_control {
            req = req.header(
                http::HeaderName::from_static("cache-control"),
                http::HeaderValue::from_str(cc).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set cache-control header".into()),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
        }

        if let Some(ref cd) = self.content_disposition {
            req = req.header(
                http::HeaderName::from_static("content-disposition"),
                http::HeaderValue::from_str(cd).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set content-disposition header".into()),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
        }

        if let Some(ref ce) = self.content_encoding {
            req = req.header(
                http::HeaderName::from_static("content-encoding"),
                http::HeaderValue::from_str(ce).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set content-encoding header".into()),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
        }

        if let Some(ref exp) = self.expires {
            req = req.header(
                http::HeaderName::from_static("expires"),
                http::HeaderValue::from_str(exp).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set expires header".into()),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
        }

        if let Some(acl) = self.acl {
            req = req.header(
                http::HeaderName::from_static("x-oss-object-acl"),
                http::HeaderValue::from_str(acl.as_str()).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set x-oss-object-acl header".into()),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
        }

        if let Some(sc) = self.storage_class {
            req = req.header(
                http::HeaderName::from_static("x-oss-storage-class"),
                http::HeaderValue::from_str(sc.as_str()).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set x-oss-storage-class header".into()),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
        }

        if let Some(ref sse) = self.sse {
            req = req.header(
                http::HeaderName::from_static("x-oss-server-side-encryption"),
                http::HeaderValue::from_str(sse.as_str()).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set x-oss-server-side-encryption header".into()),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );

            if let Some(key_id) = sse.key_id() {
                req = req.header(
                    http::HeaderName::from_static("x-oss-server-side-encryption-key-id"),
                    http::HeaderValue::from_str(key_id).map_err(|e| OssError {
                        kind: OssErrorKind::ValidationError,
                        context: Box::new(ErrorContext {
                            operation: Some(
                                "set x-oss-server-side-encryption-key-id header".into(),
                            ),
                            bucket: Some(self.bucket.to_string()),
                            object_key: Some(self.key.to_string()),
                            ..Default::default()
                        }),
                        source: Some(Box::new(e)),
                    })?,
                );
            }
        } else if let Some(ref key_id) = self.sse_key_id {
            req = req.header(
                http::HeaderName::from_static("x-oss-server-side-encryption"),
                http::HeaderValue::from_str("KMS").map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set x-oss-server-side-encryption header".into()),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
            req = req.header(
                http::HeaderName::from_static("x-oss-server-side-encryption-key-id"),
                http::HeaderValue::from_str(key_id).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set x-oss-server-side-encryption-key-id header".into()),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
        }

        if let Some(ref tag) = self.tagging {
            req = req.header(
                http::HeaderName::from_static("x-oss-tagging"),
                http::HeaderValue::from_str(tag).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set x-oss-tagging header".into()),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
        }

        for (k, v) in &self.metadata {
            let header_name = http::HeaderName::from_bytes(k.as_bytes()).map_err(|e| OssError {
                kind: OssErrorKind::ValidationError,
                context: Box::new(ErrorContext {
                    operation: Some(format!("set metadata header '{}'", k)),
                    bucket: Some(self.bucket.to_string()),
                    object_key: Some(self.key.to_string()),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;
            req = req.header(
                header_name,
                http::HeaderValue::from_str(v).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some(format!("set metadata header value '{}'", k)),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
        }

        let request = req.body(body).build();

        let response = self
            .client
            .send_signed(request, Some(&self.bucket), Vec::new())
            .await
            .map_err(|e| OssError {
                kind: OssErrorKind::TransportError,
                context: Box::new(ErrorContext {
                    operation: Some("PutObject".into()),
                    bucket: Some(self.bucket.to_string()),
                    object_key: Some(self.key.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.is_success() {
            let request_id = response
                .headers
                .get("x-oss-request-id")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_string();

            let etag = response
                .headers
                .get("ETag")
                .or_else(|| response.headers.get("etag"))
                .and_then(|v| v.to_str().ok())
                .map(|s| s.trim_matches('"').to_string())
                .unwrap_or_default();

            let version_id = response
                .headers
                .get("x-oss-version-id")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            let hash_crc64 = response
                .headers
                .get("x-oss-hash-crc64ecma")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            let result_sse = response
                .headers
                .get("x-oss-server-side-encryption")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            Ok(PutObjectOutput {
                request_id,
                etag,
                version_id,
                hash_crc64,
                sse: result_sse,
            })
        } else {
            Err(OssError {
                kind: OssErrorKind::ServiceError(Box::new(crate::error::OssServiceError {
                    status_code: response.status().as_u16(),
                    code: String::new(),
                    message: String::new(),
                    request_id: String::new(),
                    host_id: String::new(),
                    resource: Some(self.key.to_string()),
                    string_to_sign: None,
                })),
                context: Box::new(ErrorContext {
                    operation: Some("PutObject".into()),
                    bucket: Some(self.bucket.to_string()),
                    object_key: Some(self.key.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct PutObjectOutput {
    pub request_id: String,
    pub etag: String,
    pub version_id: Option<String>,
    pub hash_crc64: Option<String>,
    pub sse: Option<String>,
}

impl BucketOperations {
    pub fn put_object(&self, key: impl Into<String>) -> Result<PutObjectBuilder> {
        let object_key = ObjectKey::new(key.into())?;
        Ok(PutObjectBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            object_key,
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use std::sync::Mutex;

    use http::HeaderMap;

    use crate::client::OSSClientInner;
    use crate::config::credentials::Credentials;
    use crate::http::client::{HttpClient, HttpRequest, HttpResponse};
    use crate::types::region::Region;

    use super::*;

    struct RecordingHttpClient {
        requests: Arc<Mutex<Vec<HttpRequest>>>,
        status_code: http::StatusCode,
    }

    #[async_trait::async_trait]
    impl HttpClient for RecordingHttpClient {
        async fn send(&self, request: HttpRequest) -> crate::error::Result<HttpResponse> {
            self.requests.lock().unwrap().push(request);
            let mut headers = HeaderMap::new();
            headers.insert(
                "x-oss-request-id",
                http::HeaderValue::from_static("rid-001"),
            );
            headers.insert("ETag", http::HeaderValue::from_static("\"abc123\""));
            Ok(HttpResponse {
                status: self.status_code,
                headers,
                body: bytes::Bytes::new(),
            })
        }
    }

    fn create_test_inner() -> (Arc<OSSClientInner>, Arc<Mutex<Vec<HttpRequest>>>) {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let http = Arc::new(RecordingHttpClient {
            requests: requests.clone(),
            status_code: http::StatusCode::OK,
        });
        let credentials = Arc::new(crate::config::credentials::StaticCredentialsProvider::new(
            Credentials::builder()
                .access_key_id("test-ak")
                .access_key_secret("test-sk")
                .build()
                .unwrap(),
        ));
        let inner = Arc::new(OSSClientInner {
            http,
            credentials,
            signer: Arc::from(crate::signer::create_signer(crate::signer::SignVersion::V4)),
            region: Region::CnHangzhou,
            endpoint: "oss-cn-hangzhou.aliyuncs.com".into(),
        });
        (inner, requests)
    }

    fn test_bucket() -> BucketName {
        BucketName::new("test-bucket").unwrap()
    }

    #[test]
    fn bucket_operations_put_object_rejects_empty_key() {
        let (inner, _) = create_test_inner();
        let ops = BucketOperations {
            client: inner,
            bucket: test_bucket(),
        };
        assert!(ops.put_object("").is_err());
    }

    #[test]
    fn bucket_operations_put_object_rejects_overlength_key() {
        let (inner, _) = create_test_inner();
        let ops = BucketOperations {
            client: inner,
            bucket: test_bucket(),
        };
        assert!(ops.put_object("a".repeat(1025)).is_err());
    }

    #[test]
    fn bucket_operations_put_object_accepts_valid_key() {
        let (inner, _) = create_test_inner();
        let ops = BucketOperations {
            client: inner,
            bucket: test_bucket(),
        };
        assert!(ops.put_object("valid-key.txt").is_ok());
    }

    #[tokio::test]
    async fn put_object_sends_correct_request() {
        let (inner, requests) = create_test_inner();
        let bucket = BucketName::new("test-bucket").unwrap();
        let builder =
            PutObjectBuilder::new(inner, bucket, ObjectKey::new("test-file.txt").unwrap());

        let output = builder
            .body(bytes::Bytes::from_static(b"hello world"))
            .content_type("text/plain")
            .send()
            .await
            .unwrap();

        assert_eq!(output.request_id, "rid-001");
        assert_eq!(output.etag, "abc123");

        let captured = requests.lock().unwrap();
        assert_eq!(captured.len(), 1);
        assert_eq!(captured[0].method, http::Method::PUT);
        assert!(captured[0].uri.contains("test-bucket"));
        assert!(captured[0].uri.contains("test-file.txt"));

        let ct = captured[0]
            .headers
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(ct, "text/plain");

        assert_eq!(captured[0].body.as_deref(), Some(b"hello world" as &[u8]));
    }

    #[tokio::test]
    async fn put_object_with_custom_metadata() {
        let (inner, requests) = create_test_inner();
        let bucket = BucketName::new("test-bucket").unwrap();
        let builder = PutObjectBuilder::new(inner, bucket, ObjectKey::new("obj.txt").unwrap());

        builder
            .body(bytes::Bytes::from_static(b"data"))
            .metadata("x-oss-meta-author", "test-author")
            .metadata("x-oss-meta-version", "1.0")
            .send()
            .await
            .unwrap();

        let captured = requests.lock().unwrap();
        assert_eq!(
            captured[0]
                .headers
                .get("x-oss-meta-author")
                .unwrap()
                .to_str()
                .unwrap(),
            "test-author"
        );
        assert_eq!(
            captured[0]
                .headers
                .get("x-oss-meta-version")
                .unwrap()
                .to_str()
                .unwrap(),
            "1.0"
        );
    }

    #[tokio::test]
    async fn put_object_with_acl_and_storage_class() {
        let (inner, requests) = create_test_inner();
        let bucket = BucketName::new("test-bucket").unwrap();
        let builder = PutObjectBuilder::new(inner, bucket, ObjectKey::new("obj.txt").unwrap());

        builder
            .body(bytes::Bytes::from_static(b"data"))
            .acl(ObjectAcl::PublicRead)
            .storage_class(StorageClass::IA)
            .send()
            .await
            .unwrap();

        let captured = requests.lock().unwrap();
        assert_eq!(
            captured[0]
                .headers
                .get("x-oss-object-acl")
                .unwrap()
                .to_str()
                .unwrap(),
            "public-read"
        );
        assert_eq!(
            captured[0]
                .headers
                .get("x-oss-storage-class")
                .unwrap()
                .to_str()
                .unwrap(),
            "IA"
        );
    }

    #[tokio::test]
    async fn put_object_requires_body() {
        let (inner, _) = create_test_inner();
        let bucket = BucketName::new("test-bucket").unwrap();
        let builder = PutObjectBuilder::new(inner, bucket, ObjectKey::new("key").unwrap());

        let result = builder.send().await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err().kind,
            OssErrorKind::ValidationError
        ));
    }

    #[tokio::test]
    async fn put_object_uri_encodes_special_chars_in_key() {
        let (inner, requests) = create_test_inner();
        let bucket = BucketName::new("test-bucket").unwrap();
        let builder = PutObjectBuilder::new(inner, bucket, ObjectKey::new("文件 名.txt").unwrap());

        builder
            .body(bytes::Bytes::from_static(b"data"))
            .send()
            .await
            .unwrap();

        let captured = requests.lock().unwrap();
        assert!(
            captured[0]
                .uri
                .contains("%E6%96%87%E4%BB%B6%20%E5%90%8D.txt")
        );
    }

    #[tokio::test]
    async fn put_object_with_sse_aes256() {
        let (inner, requests) = create_test_inner();
        let bucket = BucketName::new("test-bucket").unwrap();
        let builder = PutObjectBuilder::new(inner, bucket, ObjectKey::new("key").unwrap());

        builder
            .body(bytes::Bytes::from_static(b"encrypted"))
            .server_side_encryption("AES256")
            .send()
            .await
            .unwrap();

        let captured = requests.lock().unwrap();
        assert_eq!(
            captured[0]
                .headers
                .get("x-oss-server-side-encryption")
                .unwrap()
                .to_str()
                .unwrap(),
            "AES256"
        );
    }

    #[tokio::test]
    async fn put_object_with_sse_kms_and_key_id() {
        let (inner, requests) = create_test_inner();
        let bucket = BucketName::new("test-bucket").unwrap();
        let builder = PutObjectBuilder::new(inner, bucket, ObjectKey::new("key").unwrap());

        builder
            .body(bytes::Bytes::from_static(b"encrypted"))
            .sse_key_id("cmk-id-123")
            .send()
            .await
            .unwrap();

        let captured = requests.lock().unwrap();
        assert_eq!(
            captured[0]
                .headers
                .get("x-oss-server-side-encryption")
                .unwrap()
                .to_str()
                .unwrap(),
            "KMS"
        );
        assert_eq!(
            captured[0]
                .headers
                .get("x-oss-server-side-encryption-key-id")
                .unwrap()
                .to_str()
                .unwrap(),
            "cmk-id-123"
        );
    }

    #[tokio::test]
    async fn put_object_with_tagging() {
        let (inner, requests) = create_test_inner();
        let bucket = BucketName::new("test-bucket").unwrap();
        let builder = PutObjectBuilder::new(inner, bucket, ObjectKey::new("key").unwrap());

        builder
            .body(bytes::Bytes::from_static(b"data"))
            .tagging("key1=value1&key2=value2")
            .send()
            .await
            .unwrap();

        let captured = requests.lock().unwrap();
        assert_eq!(
            captured[0]
                .headers
                .get("x-oss-tagging")
                .unwrap()
                .to_str()
                .unwrap(),
            "key1=value1&key2=value2"
        );
    }

    #[tokio::test]
    async fn put_object_content_md5_header() {
        let (inner, requests) = create_test_inner();
        let bucket = BucketName::new("test-bucket").unwrap();
        let builder = PutObjectBuilder::new(inner, bucket, ObjectKey::new("key").unwrap());

        builder
            .body(bytes::Bytes::from_static(b"data"))
            .content_md5("dGVzdC1tZDU=")
            .send()
            .await
            .unwrap();

        let captured = requests.lock().unwrap();
        assert_eq!(
            captured[0]
                .headers
                .get("content-md5")
                .unwrap()
                .to_str()
                .unwrap(),
            "dGVzdC1tZDU="
        );
    }

    #[tokio::test]
    #[ignore = "requires valid OSS credentials in env vars OSS_ACCESS_KEY_ID, OSS_ACCESS_KEY_SECRET, OSS_REGION, OSS_BUCKET"]
    async fn e2e_put_object_real_oss() {
        let ak = std::env::var("OSS_ACCESS_KEY_ID").expect("OSS_ACCESS_KEY_ID not set");
        let sk = std::env::var("OSS_ACCESS_KEY_SECRET").expect("OSS_ACCESS_KEY_SECRET not set");
        let region_str = std::env::var("OSS_REGION").unwrap_or_else(|_| "cn-wulanchabu".into());
        let bucket_str = std::env::var("OSS_BUCKET").expect("OSS_BUCKET not set");

        let region = Region::from_str(&region_str).unwrap_or_else(|_| Region::Custom {
            endpoint: format!("oss-{}.aliyuncs.com", region_str),
            region_id: region_str.clone(),
        });

        let client = crate::client::OSSClient::builder()
            .region(region)
            .credentials(ak, sk)
            .build()
            .unwrap();

        let key = format!("test-put-object-{}.txt", chrono::Utc::now().timestamp());
        let content = "Hello from aliyun-oss SDK E2E test";

        let output = client
            .bucket(&bucket_str)
            .unwrap()
            .put_object(&key)
            .unwrap()
            .body(bytes::Bytes::from(content))
            .content_type("text/plain")
            .send()
            .await
            .unwrap();

        assert!(!output.request_id.is_empty());
        assert!(!output.etag.is_empty());

        eprintln!(
            "PUT '{}' succeeded: request_id={}, etag={}",
            key, output.request_id, output.etag
        );
    }
}
