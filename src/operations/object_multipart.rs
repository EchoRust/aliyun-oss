use std::sync::Arc;

use serde::Deserialize;

use crate::client::{BucketOperations, OSSClientInner};
use crate::error::{ErrorContext, OssError, OssErrorKind, Result};
use crate::http::client::HttpRequest;
use crate::types::acl::ObjectAcl;
use crate::types::bucket::BucketName;
use crate::types::object::ObjectKey;
use crate::types::storage::{ServerSideEncryption, StorageClass};
use crate::util::uri::oss_endpoint_url;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "InitiateMultipartUploadResult")]
struct InitiateMultipartUploadResult {
    #[serde(rename = "Bucket")]
    bucket: String,
    #[serde(rename = "Key")]
    key: String,
    #[serde(rename = "UploadId")]
    upload_id: String,
}

pub struct InitiateMultipartUploadBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    key: ObjectKey,
    cache_control: Option<String>,
    content_type: Option<String>,
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

impl InitiateMultipartUploadBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName, key: ObjectKey) -> Self {
        Self {
            client,
            bucket,
            key,
            cache_control: None,
            content_type: None,
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

    pub fn cache_control(mut self, v: impl Into<String>) -> Self {
        self.cache_control = Some(v.into());
        self
    }

    pub fn content_type(mut self, v: impl Into<String>) -> Self {
        self.content_type = Some(v.into());
        self
    }

    pub fn content_disposition(mut self, v: impl Into<String>) -> Self {
        self.content_disposition = Some(v.into());
        self
    }

    pub fn content_encoding(mut self, v: impl Into<String>) -> Self {
        self.content_encoding = Some(v.into());
        self
    }

    pub fn expires(mut self, v: impl Into<String>) -> Self {
        self.expires = Some(v.into());
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

    pub async fn send(self) -> Result<InitiateMultipartUploadOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = oss_endpoint_url(
            &endpoint,
            Some(self.bucket.as_str()),
            Some(self.key.as_str()),
        );
        let full_uri = format!("{}?uploads", uri);

        let mut req = HttpRequest::builder()
            .method(http::Method::POST)
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

        let query_params: Vec<(String, String)> = vec![("uploads".into(), String::new())];
        let request = req.build();

        let response = self
            .client
            .send_signed(request, Some(&self.bucket), query_params)
            .await
            .map_err(|e| OssError {
                kind: OssErrorKind::TransportError,
                context: Box::new(ErrorContext {
                    operation: Some("InitiateMultipartUpload".into()),
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

            let body_str = response.body_as_str().unwrap_or("");

            let result: InitiateMultipartUploadResult = crate::util::xml::from_xml(body_str)
                .map_err(|e| OssError {
                    kind: OssErrorKind::DeserializationError,
                    context: Box::new(ErrorContext {
                        operation: Some("InitiateMultipartUpload: parse XML".into()),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?;

            Ok(InitiateMultipartUploadOutput {
                request_id,
                bucket: result.bucket,
                key: result.key,
                upload_id: result.upload_id,
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
                    operation: Some("InitiateMultipartUpload".into()),
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
pub struct InitiateMultipartUploadOutput {
    pub request_id: String,
    pub bucket: String,
    pub key: String,
    pub upload_id: String,
}

impl BucketOperations {
    pub fn initiate_multipart_upload(
        &self,
        key: impl Into<String>,
    ) -> Result<InitiateMultipartUploadBuilder> {
        let object_key = ObjectKey::new(key.into())?;
        Ok(InitiateMultipartUploadBuilder::new(
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

    use crate::client::OSSClientInner;
    use crate::config::credentials::Credentials;
    use crate::http::client::{HttpClient, HttpRequest, HttpResponse};
    use crate::types::region::Region;

    use super::*;

    struct RecordingHttpClient {
        requests: Arc<Mutex<Vec<HttpRequest>>>,
        status_code: http::StatusCode,
        response_body: bytes::Bytes,
        response_headers: Vec<(&'static str, &'static str)>,
    }

    #[async_trait::async_trait]
    impl HttpClient for RecordingHttpClient {
        async fn send(&self, request: HttpRequest) -> crate::error::Result<HttpResponse> {
            self.requests.lock().unwrap().push(request);
            let mut headers = http::HeaderMap::new();
            headers.insert(
                "x-oss-request-id",
                http::HeaderValue::from_static("rid-multipart"),
            );
            for (name, value) in &self.response_headers {
                if let (Ok(n), Ok(v)) = (
                    http::HeaderName::from_bytes(name.as_bytes()),
                    http::HeaderValue::from_str(value),
                ) {
                    headers.insert(n, v);
                }
            }
            Ok(HttpResponse {
                status: self.status_code,
                headers,
                body: self.response_body.clone(),
            })
        }
    }

    fn create_test_inner_with_response(
        status_code: http::StatusCode,
        response_body: bytes::Bytes,
        response_headers: Vec<(&'static str, &'static str)>,
    ) -> (Arc<OSSClientInner>, Arc<Mutex<Vec<HttpRequest>>>) {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let http = Arc::new(RecordingHttpClient {
            requests: requests.clone(),
            status_code,
            response_body,
            response_headers,
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

    #[test]
    fn initiate_multipart_builder_has_bucket_and_key() {
        let (inner, _) =
            create_test_inner_with_response(http::StatusCode::OK, bytes::Bytes::new(), vec![]);
        let _builder = InitiateMultipartUploadBuilder::new(
            inner,
            BucketName::new("test-bucket").unwrap(),
            ObjectKey::new("test-key.txt").unwrap(),
        );
    }

    #[tokio::test]
    async fn initiate_multipart_builder_sends_post_with_uploads_query() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<InitiateMultipartUploadResult>
  <Bucket>oss-bucket</Bucket>
  <Key>test-key.txt</Key>
  <UploadId>upload-id-123</UploadId>
</InitiateMultipartUploadResult>"#;
        let (inner, requests) =
            create_test_inner_with_response(http::StatusCode::OK, bytes::Bytes::from(xml), vec![]);
        let builder = InitiateMultipartUploadBuilder::new(
            inner,
            BucketName::new("test-bucket").unwrap(),
            ObjectKey::new("test-key.txt").unwrap(),
        );

        let output = builder.send().await.unwrap();
        assert_eq!(output.upload_id, "upload-id-123");
        assert_eq!(output.bucket, "oss-bucket");
        assert_eq!(output.key, "test-key.txt");

        let captured = requests.lock().unwrap();
        assert_eq!(captured[0].method, http::Method::POST);
        assert!(captured[0].uri.contains("?uploads"));
    }

    #[tokio::test]
    async fn initiate_multipart_builder_sets_optional_headers() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<InitiateMultipartUploadResult>
  <Bucket>b</Bucket>
  <Key>k</Key>
  <UploadId>uid</UploadId>
</InitiateMultipartUploadResult>"#;
        let (inner, requests) =
            create_test_inner_with_response(http::StatusCode::OK, bytes::Bytes::from(xml), vec![]);
        let builder = InitiateMultipartUploadBuilder::new(
            inner,
            BucketName::new("test-bucket").unwrap(),
            ObjectKey::new("test-key.txt").unwrap(),
        )
        .cache_control("max-age=300")
        .content_type("application/octet-stream")
        .acl(ObjectAcl::Private)
        .storage_class(StorageClass::Standard);

        builder.send().await.unwrap();

        let captured = requests.lock().unwrap();
        let has_header = |name: &str, val: &str| -> bool {
            captured[0]
                .headers
                .get(http::HeaderName::from_bytes(name.as_bytes()).unwrap())
                .map(|v| v.to_str().ok() == Some(val))
                .unwrap_or(false)
        };
        assert!(has_header("cache-control", "max-age=300"));
        assert!(has_header("content-type", "application/octet-stream"));
        assert!(has_header("x-oss-object-acl", "private"));
        assert!(has_header("x-oss-storage-class", "Standard"));
    }

    #[tokio::test]
    async fn initiate_multipart_returns_error_on_failure() {
        let (inner, _) = create_test_inner_with_response(
            http::StatusCode::BAD_REQUEST,
            bytes::Bytes::from(""),
            vec![],
        );
        let builder = InitiateMultipartUploadBuilder::new(
            inner,
            BucketName::new("test-bucket").unwrap(),
            ObjectKey::new("test-key.txt").unwrap(),
        );

        let result = builder.send().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[ignore = "requires valid OSS credentials"]
    async fn e2e_initiate_multipart_upload() {
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

        let key = format!("test-multipart-{}.bin", chrono::Utc::now().timestamp());
        let output = client
            .bucket(&bucket_str)
            .unwrap()
            .initiate_multipart_upload(&key)
            .unwrap()
            .send()
            .await
            .unwrap();

        assert!(!output.upload_id.is_empty());
        assert_eq!(output.key, key);
        eprintln!(
            "InitiateMultipartUpload: key={}, upload_id={}",
            output.key, output.upload_id
        );
    }
}
