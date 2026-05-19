//! Append object operations.

use std::sync::Arc;

use crate::client::{BucketOperations, OSSClientInner};
use crate::error::{ErrorContext, OssError, OssErrorKind, Result};
use crate::http::client::HttpRequest;
use crate::types::bucket::BucketName;
use crate::types::object::ObjectKey;
use crate::util::uri::oss_endpoint_url;

pub struct AppendObjectBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    key: ObjectKey,
    position: u64,
    body: bytes::Bytes,
    content_type: Option<String>,
    content_md5: Option<String>,
    content_encoding: Option<String>,
    metadata: Vec<(String, String)>,
}

impl AppendObjectBuilder {
    pub(crate) fn new(
        client: Arc<OSSClientInner>,
        bucket: BucketName,
        key: ObjectKey,
        position: u64,
        body: impl Into<bytes::Bytes>,
    ) -> Self {
        Self {
            client,
            bucket,
            key,
            position,
            body: body.into(),
            content_type: None,
            content_md5: None,
            content_encoding: None,
            metadata: Vec::new(),
        }
    }

    pub fn content_type(mut self, ct: impl Into<String>) -> Self {
        self.content_type = Some(ct.into());
        self
    }

    pub fn content_md5(mut self, md5: impl Into<String>) -> Self {
        self.content_md5 = Some(md5.into());
        self
    }

    pub fn content_encoding(mut self, ce: impl Into<String>) -> Self {
        self.content_encoding = Some(ce.into());
        self
    }

    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.push((key.into(), value.into()));
        self
    }

    pub async fn send(self) -> Result<AppendObjectOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = oss_endpoint_url(
            &endpoint,
            Some(self.bucket.as_str()),
            Some(self.key.as_str()),
        );

        let query_string = format!("?append&position={}", self.position);
        let full_uri = format!("{}{}", uri, query_string);

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

        let query_params = vec![
            ("append".into(), "".into()),
            ("position".into(), self.position.to_string()),
        ];

        let body_len = self.body.len();
        let request = req.body(self.body).build();

        let response = self
            .client
            .send_signed(request, Some(&self.bucket), query_params)
            .await
            .map_err(|e| OssError {
                kind: OssErrorKind::TransportError,
                context: Box::new(ErrorContext {
                    operation: Some("AppendObject".into()),
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

            let next_position = response
                .headers
                .get("x-oss-next-append-position")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(self.position + body_len as u64);

            let hash_crc64 = response
                .headers
                .get("x-oss-hash-crc64ecma")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            Ok(AppendObjectOutput {
                request_id,
                next_position,
                hash_crc64,
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
                    operation: Some("AppendObject".into()),
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
pub struct AppendObjectOutput {
    pub request_id: String,
    pub next_position: u64,
    pub hash_crc64: Option<String>,
}

impl BucketOperations {
    pub fn append_object(
        &self,
        key: impl Into<String>,
        position: u64,
        body: impl Into<bytes::Bytes>,
    ) -> Result<AppendObjectBuilder> {
        let object_key = ObjectKey::new(key.into())?;
        Ok(AppendObjectBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            object_key,
            position,
            body,
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
    }

    #[async_trait::async_trait]
    impl HttpClient for RecordingHttpClient {
        async fn send(&self, request: HttpRequest) -> crate::error::Result<HttpResponse> {
            self.requests.lock().unwrap().push(request);
            let mut headers = HeaderMap::new();
            headers.insert(
                "x-oss-request-id",
                http::HeaderValue::from_static("rid-append"),
            );
            headers.insert(
                "x-oss-next-append-position",
                http::HeaderValue::from_static("11"),
            );
            Ok(HttpResponse {
                status: http::StatusCode::OK,
                headers,
                body: bytes::Bytes::new(),
            })
        }
    }

    fn create_test_inner() -> (Arc<OSSClientInner>, Arc<Mutex<Vec<HttpRequest>>>) {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let http = Arc::new(RecordingHttpClient {
            requests: requests.clone(),
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
    async fn append_object_first_position_zero() {
        let (inner, requests) = create_test_inner();
        let bucket = BucketName::new("test-bucket").unwrap();
        let builder = AppendObjectBuilder::new(
            inner,
            bucket,
            ObjectKey::new("append.txt").unwrap(),
            0,
            bytes::Bytes::from_static(b"hello world"),
        );

        let output = builder.send().await.unwrap();
        assert_eq!(output.next_position, 11);

        let captured = requests.lock().unwrap();
        assert!(captured[0].uri.contains("?append&position=0"));
        assert_eq!(captured[0].method, http::Method::POST);
    }

    #[tokio::test]
    async fn append_object_returns_next_position() {
        let (inner, _) = create_test_inner();
        let bucket = BucketName::new("test-bucket").unwrap();
        let builder = AppendObjectBuilder::new(
            inner,
            bucket,
            ObjectKey::new("append.txt").unwrap(),
            11,
            bytes::Bytes::from_static(b" more data"),
        );

        let output = builder.send().await.unwrap();
        assert_eq!(output.next_position, 11);
        assert!(!output.request_id.is_empty());
    }

    #[tokio::test]
    async fn append_object_with_content_type() {
        let (inner, requests) = create_test_inner();
        let bucket = BucketName::new("test-bucket").unwrap();
        let builder = AppendObjectBuilder::new(
            inner,
            bucket,
            ObjectKey::new("append.txt").unwrap(),
            0,
            bytes::Bytes::from_static(b"data"),
        );

        builder.content_type("text/plain").send().await.unwrap();

        let captured = requests.lock().unwrap();
        assert_eq!(
            captured[0]
                .headers
                .get("content-type")
                .unwrap()
                .to_str()
                .unwrap(),
            "text/plain"
        );
    }

    #[tokio::test]
    #[ignore = "requires valid OSS credentials"]
    async fn e2e_append_object() {
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

        let key = format!("test-append-{}.txt", chrono::Utc::now().timestamp());

        let output1 = client
            .bucket(&bucket_str)
            .unwrap()
            .append_object(&key, 0, bytes::Bytes::from_static(b"hello "))
            .unwrap()
            .send()
            .await
            .unwrap();

        assert_eq!(output1.next_position, 6);

        let output2 = client
            .bucket(&bucket_str)
            .unwrap()
            .append_object(
                &key,
                output1.next_position,
                bytes::Bytes::from_static(b"world"),
            )
            .unwrap()
            .send()
            .await
            .unwrap();

        assert_eq!(output2.next_position, 11);

        let get_output = client
            .bucket(&bucket_str)
            .unwrap()
            .get_object(&key)
            .unwrap()
            .send()
            .await
            .unwrap();
        assert_eq!(
            get_output.body.as_ref(),
            bytes::Bytes::from_static(b"hello world")
        );

        client
            .bucket(&bucket_str)
            .unwrap()
            .delete_object(&key)
            .unwrap()
            .send()
            .await
            .unwrap();

        eprintln!("APPEND E2E '{}' succeeded: 2 appends, total 11 bytes", key);
    }
}
