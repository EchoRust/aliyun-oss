use std::sync::Arc;

use serde::Deserialize;

use crate::client::{BucketOperations, OSSClientInner};
use crate::error::{ErrorContext, OssError, OssErrorKind, Result};
use crate::http::client::HttpRequest;
use crate::types::acl::ObjectAcl;
use crate::types::bucket::BucketName;
use crate::types::object::ObjectKey;
use crate::types::storage::StorageClass;
use crate::util::uri::oss_endpoint_url;

pub struct CopyObjectBuilder {
    client: Arc<OSSClientInner>,
    dest_bucket: BucketName,
    dest_key: ObjectKey,
    source_bucket: BucketName,
    source_key: ObjectKey,
    source_version_id: Option<String>,
    if_match: Option<String>,
    if_none_match: Option<String>,
    if_modified_since: Option<String>,
    if_unmodified_since: Option<String>,
    metadata_directive: Option<String>,
    acl: Option<ObjectAcl>,
    storage_class: Option<StorageClass>,
    server_side_encryption: Option<String>,
    sse_key_id: Option<String>,
}

impl CopyObjectBuilder {
    pub(crate) fn new(
        client: Arc<OSSClientInner>,
        dest_bucket: BucketName,
        dest_key: ObjectKey,
        source_bucket: BucketName,
        source_key: ObjectKey,
    ) -> Self {
        Self {
            client,
            dest_bucket,
            dest_key,
            source_bucket,
            source_key,
            source_version_id: None,
            if_match: None,
            if_none_match: None,
            if_modified_since: None,
            if_unmodified_since: None,
            metadata_directive: None,
            acl: None,
            storage_class: None,
            server_side_encryption: None,
            sse_key_id: None,
        }
    }

    pub fn source_version_id(mut self, id: impl Into<String>) -> Self {
        self.source_version_id = Some(id.into());
        self
    }

    pub fn if_match(mut self, etag: impl Into<String>) -> Self {
        self.if_match = Some(etag.into());
        self
    }

    pub fn if_none_match(mut self, etag: impl Into<String>) -> Self {
        self.if_none_match = Some(etag.into());
        self
    }

    pub fn if_modified_since(mut self, time: impl Into<String>) -> Self {
        self.if_modified_since = Some(time.into());
        self
    }

    pub fn if_unmodified_since(mut self, time: impl Into<String>) -> Self {
        self.if_unmodified_since = Some(time.into());
        self
    }

    pub fn metadata_directive_copy(mut self) -> Self {
        self.metadata_directive = Some("COPY".into());
        self
    }

    pub fn metadata_directive_replace(mut self) -> Self {
        self.metadata_directive = Some("REPLACE".into());
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
        self.server_side_encryption = Some(sse.into());
        self
    }

    pub fn sse_key_id(mut self, key_id: impl Into<String>) -> Self {
        self.sse_key_id = Some(key_id.into());
        self
    }

    pub async fn send(self) -> Result<CopyObjectOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = oss_endpoint_url(
            &endpoint,
            Some(self.dest_bucket.as_str()),
            Some(self.dest_key.as_str()),
        );

        let source = format!(
            "/{}/{}",
            self.source_bucket.as_str(),
            crate::util::uri::uri_encode(self.source_key.as_str())
        );

        let mut req = HttpRequest::builder().method(http::Method::PUT).uri(&uri);

        req = req.header(
            http::HeaderName::from_static("x-oss-copy-source"),
            http::HeaderValue::from_str(&source).map_err(|e| OssError {
                kind: OssErrorKind::ValidationError,
                context: Box::new(ErrorContext {
                    operation: Some("set x-oss-copy-source header".into()),
                    bucket: Some(self.dest_bucket.to_string()),
                    object_key: Some(self.dest_key.to_string()),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?,
        );

        if let Some(ref im) = self.if_match {
            req = req.header(
                http::HeaderName::from_static("x-oss-copy-source-if-match"),
                http::HeaderValue::from_str(im).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set x-oss-copy-source-if-match header".into()),
                        bucket: Some(self.dest_bucket.to_string()),
                        object_key: Some(self.dest_key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
        }

        if let Some(ref inm) = self.if_none_match {
            req = req.header(
                http::HeaderName::from_static("x-oss-copy-source-if-none-match"),
                http::HeaderValue::from_str(inm).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set x-oss-copy-source-if-none-match header".into()),
                        bucket: Some(self.dest_bucket.to_string()),
                        object_key: Some(self.dest_key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
        }

        if let Some(ref ims) = self.if_modified_since {
            req = req.header(
                http::HeaderName::from_static("x-oss-copy-source-if-modified-since"),
                http::HeaderValue::from_str(ims).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set x-oss-copy-source-if-modified-since header".into()),
                        bucket: Some(self.dest_bucket.to_string()),
                        object_key: Some(self.dest_key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
        }

        if let Some(ref ius) = self.if_unmodified_since {
            req = req.header(
                http::HeaderName::from_static("x-oss-copy-source-if-unmodified-since"),
                http::HeaderValue::from_str(ius).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set x-oss-copy-source-if-unmodified-since header".into()),
                        bucket: Some(self.dest_bucket.to_string()),
                        object_key: Some(self.dest_key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
        }

        if let Some(ref md) = self.metadata_directive {
            req = req.header(
                http::HeaderName::from_static("x-oss-metadata-directive"),
                http::HeaderValue::from_str(md).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set x-oss-metadata-directive header".into()),
                        bucket: Some(self.dest_bucket.to_string()),
                        object_key: Some(self.dest_key.to_string()),
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
                        bucket: Some(self.dest_bucket.to_string()),
                        object_key: Some(self.dest_key.to_string()),
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
                        bucket: Some(self.dest_bucket.to_string()),
                        object_key: Some(self.dest_key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
        }

        if let Some(ref sse) = self.server_side_encryption {
            req = req.header(
                http::HeaderName::from_static("x-oss-server-side-encryption"),
                http::HeaderValue::from_str(sse).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set x-oss-server-side-encryption header".into()),
                        bucket: Some(self.dest_bucket.to_string()),
                        object_key: Some(self.dest_key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
        }

        if let Some(ref key_id) = self.sse_key_id {
            req = req.header(
                http::HeaderName::from_static("x-oss-server-side-encryption-key-id"),
                http::HeaderValue::from_str(key_id).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set x-oss-server-side-encryption-key-id header".into()),
                        bucket: Some(self.dest_bucket.to_string()),
                        object_key: Some(self.dest_key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
        }

        let request = req.build();

        let response = self
            .client
            .send_signed(request, Some(&self.dest_bucket), Vec::new())
            .await
            .map_err(|e| OssError {
                kind: OssErrorKind::TransportError,
                context: Box::new(ErrorContext {
                    operation: Some("CopyObject".into()),
                    bucket: Some(self.dest_bucket.to_string()),
                    object_key: Some(self.dest_key.to_string()),
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

            let result: CopyObjectResult =
                crate::util::xml::from_xml(response.body_as_str().unwrap_or(""))?;

            Ok(CopyObjectOutput {
                request_id,
                etag: result.etag.trim_matches('"').to_string(),
                last_modified: result.last_modified,
            })
        } else {
            Err(OssError {
                kind: OssErrorKind::ServiceError(Box::new(crate::error::OssServiceError {
                    status_code: response.status().as_u16(),
                    code: String::new(),
                    message: String::new(),
                    request_id: String::new(),
                    host_id: String::new(),
                    resource: Some(self.dest_key.to_string()),
                    string_to_sign: None,
                })),
                context: Box::new(ErrorContext {
                    operation: Some("CopyObject".into()),
                    bucket: Some(self.dest_bucket.to_string()),
                    object_key: Some(self.dest_key.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename = "CopyObjectResult")]
struct CopyObjectResult {
    #[serde(rename = "ETag")]
    pub(crate) etag: String,
    #[serde(rename = "LastModified")]
    pub(crate) last_modified: String,
}

#[derive(Debug, Clone)]
pub struct CopyObjectOutput {
    pub request_id: String,
    pub etag: String,
    pub last_modified: String,
}

impl BucketOperations {
    pub fn copy_object(
        &self,
        dest_key: impl Into<String>,
        source_bucket: &BucketName,
        source_key: &ObjectKey,
    ) -> Result<CopyObjectBuilder> {
        let dest_key = ObjectKey::new(dest_key.into())?;
        Ok(CopyObjectBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            dest_key,
            source_bucket.clone(),
            source_key.clone(),
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
        response_body: bytes::Bytes,
    }

    #[async_trait::async_trait]
    impl HttpClient for RecordingHttpClient {
        async fn send(&self, request: HttpRequest) -> crate::error::Result<HttpResponse> {
            self.requests.lock().unwrap().push(request);
            let mut headers = HeaderMap::new();
            headers.insert(
                "x-oss-request-id",
                http::HeaderValue::from_static("rid-copy"),
            );
            Ok(HttpResponse {
                status: http::StatusCode::OK,
                headers,
                body: self.response_body.clone(),
            })
        }
    }

    fn create_test_inner(
        response_body: bytes::Bytes,
    ) -> (Arc<OSSClientInner>, Arc<Mutex<Vec<HttpRequest>>>) {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let http = Arc::new(RecordingHttpClient {
            requests: requests.clone(),
            response_body,
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

    #[tokio::test]
    async fn copy_object_sends_correct_request() {
        let xml_body = r#"<?xml version="1.0" encoding="UTF-8"?>
<CopyObjectResult>
  <ETag>"abc789"</ETag>
  <LastModified>2024-06-01T00:00:00.000Z</LastModified>
</CopyObjectResult>"#;

        let (inner, requests) = create_test_inner(bytes::Bytes::from(xml_body));
        let source_bucket = BucketName::new("source-bucket").unwrap();
        let source_key = ObjectKey::new("source/file.txt").unwrap();
        let builder = CopyObjectBuilder::new(
            inner,
            BucketName::new("dest-bucket").unwrap(),
            ObjectKey::new("dest/file.txt").unwrap(),
            source_bucket,
            source_key,
        );

        let output = builder.send().await.unwrap();

        assert_eq!(output.etag, "abc789");
        assert!(!output.request_id.is_empty());

        let captured = requests.lock().unwrap();
        assert_eq!(captured[0].method, http::Method::PUT);
        assert!(captured[0].uri.contains("dest-bucket"));
        assert!(captured[0].uri.contains("dest/file.txt"));

        let copy_source = captured[0]
            .headers
            .get("x-oss-copy-source")
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(copy_source, "/source-bucket/source/file.txt");
    }

    #[tokio::test]
    async fn copy_object_with_metadata_directive() {
        let xml_body = r#"<?xml version="1.0" encoding="UTF-8"?>
<CopyObjectResult>
  <ETag>"etag"</ETag>
  <LastModified>2024-01-01T00:00:00.000Z</LastModified>
</CopyObjectResult>"#;

        let (inner, requests) = create_test_inner(bytes::Bytes::from(xml_body));
        let builder = CopyObjectBuilder::new(
            inner,
            BucketName::new("dest-bucket").unwrap(),
            ObjectKey::new("dest.txt").unwrap(),
            BucketName::new("src-bucket").unwrap(),
            ObjectKey::new("src.txt").unwrap(),
        );

        builder.metadata_directive_replace().send().await.unwrap();

        let captured = requests.lock().unwrap();
        assert_eq!(
            captured[0]
                .headers
                .get("x-oss-metadata-directive")
                .unwrap()
                .to_str()
                .unwrap(),
            "REPLACE"
        );
    }

    #[tokio::test]
    async fn copy_object_source_encoding() {
        let xml_body = r#"<?xml version="1.0" encoding="UTF-8"?>
<CopyObjectResult>
  <ETag>"etag"</ETag>
  <LastModified>2024-01-01T00:00:00.000Z</LastModified>
</CopyObjectResult>"#;

        let (inner, requests) = create_test_inner(bytes::Bytes::from(xml_body));
        let builder = CopyObjectBuilder::new(
            inner,
            BucketName::new("dest-bucket").unwrap(),
            ObjectKey::new("dest.txt").unwrap(),
            BucketName::new("src-bucket").unwrap(),
            ObjectKey::new("文件 名.txt").unwrap(),
        );

        builder.send().await.unwrap();

        let captured = requests.lock().unwrap();
        let copy_source = captured[0]
            .headers
            .get("x-oss-copy-source")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(copy_source.contains("%E6%96%87%E4%BB%B6%20%E5%90%8D.txt"));
    }

    #[tokio::test]
    #[ignore = "requires valid OSS credentials"]
    async fn e2e_copy_object() {
        let ak = std::env::var("OSS_ACCESS_KEY_ID").expect("OSS_ACCESS_KEY_ID not set");
        let sk = std::env::var("OSS_ACCESS_KEY_SECRET").expect("OSS_ACCESS_KEY_SECRET not set");
        let region_str = std::env::var("OSS_REGION").unwrap_or_else(|_| "cn-hangzhou".into());
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

        let src_key = format!("test-copy-src-{}.txt", chrono::Utc::now().timestamp());
        let dest_key = format!("test-copy-dest-{}.txt", chrono::Utc::now().timestamp());
        let content = "CopyObject E2E test content";

        client
            .bucket(&bucket_str)
            .unwrap()
            .put_object(&src_key)
            .unwrap()
            .body(bytes::Bytes::from(content))
            .content_type("text/plain")
            .send()
            .await
            .unwrap();

        let bucket = BucketName::new(&bucket_str).unwrap();
        let source_key = ObjectKey::new(&src_key).unwrap();

        let output = client
            .bucket(&bucket_str)
            .unwrap()
            .copy_object(&dest_key, &bucket, &source_key)
            .unwrap()
            .send()
            .await
            .unwrap();

        assert!(!output.etag.is_empty());
        assert!(!output.last_modified.is_empty());
        eprintln!(
            "COPY '{}' -> '{}' succeeded: etag={}",
            src_key, dest_key, output.etag
        );

        let get_output = client
            .bucket(&bucket_str)
            .unwrap()
            .get_object(&dest_key)
            .unwrap()
            .send()
            .await
            .unwrap();
        assert_eq!(get_output.body.as_ref(), content.as_bytes());

        client
            .bucket(&bucket_str)
            .unwrap()
            .delete_object(&src_key)
            .unwrap()
            .send()
            .await
            .unwrap();
        client
            .bucket(&bucket_str)
            .unwrap()
            .delete_object(&dest_key)
            .unwrap()
            .send()
            .await
            .unwrap();
    }
}
