//! Bucket server-side encryption operations.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::client::{BucketOperations, OSSClientInner};
use crate::error::{ErrorContext, OssError, OssErrorKind, Result};
use crate::http::client::HttpRequest;
use crate::types::bucket::BucketName;

#[derive(Debug, Clone)]
pub struct ServerSideEncryptionConfiguration {
    pub sse_algorithm: String,
    pub kms_master_key_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename = "ServerSideEncryptionRule")]
struct SSEConfig {
    #[serde(rename = "ApplyServerSideEncryptionByDefault")]
    apply: SSEApply,
}

#[derive(Debug, Clone, Serialize)]
struct SSEApply {
    #[serde(rename = "SSEAlgorithm")]
    sse_algorithm: String,
    #[serde(rename = "KMSMasterKeyID", skip_serializing_if = "Option::is_none")]
    kms_master_key_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "ServerSideEncryptionRule")]
struct SSEConfigResponse {
    #[serde(rename = "ApplyServerSideEncryptionByDefault")]
    apply: SSEApplyResponse,
}

#[derive(Debug, Clone, Deserialize)]
struct SSEApplyResponse {
    #[serde(rename = "SSEAlgorithm")]
    sse_algorithm: String,
    #[serde(rename = "KMSMasterKeyID", default)]
    kms_master_key_id: Option<String>,
}

pub struct PutBucketEncryptionBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    config: ServerSideEncryptionConfiguration,
}

impl PutBucketEncryptionBuilder {
    pub(crate) fn new(
        client: Arc<OSSClientInner>,
        bucket: BucketName,
        config: ServerSideEncryptionConfiguration,
    ) -> Self {
        Self {
            client,
            bucket,
            config,
        }
    }

    pub async fn send(self) -> Result<PutBucketEncryptionOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!("https://{}.{}?encryption", self.bucket.as_str(), endpoint);

        let query_params: Vec<(String, String)> = vec![("encryption".into(), String::new())];

        let config = SSEConfig {
            apply: SSEApply {
                sse_algorithm: self.config.sse_algorithm,
                kms_master_key_id: self.config.kms_master_key_id,
            },
        };

        let body_xml = crate::util::xml::to_xml(&config)?;

        let request = HttpRequest::builder()
            .method(http::Method::PUT)
            .uri(&uri)
            .body(bytes::Bytes::from(body_xml))
            .build();

        let response = self
            .client
            .send_signed(request, Some(&self.bucket), query_params)
            .await
            .map_err(|e| OssError {
                kind: OssErrorKind::TransportError,
                context: Box::new(ErrorContext {
                    operation: Some("PutBucketEncryption".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.status().is_success() {
            Ok(PutBucketEncryptionOutput {
                request_id: response
                    .headers
                    .get("x-oss-request-id")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("")
                    .to_string(),
            })
        } else {
            Err(OssError {
                kind: OssErrorKind::ServiceError(Box::new(crate::error::OssServiceError {
                    status_code: response.status().as_u16(),
                    code: String::new(),
                    message: String::new(),
                    request_id: String::new(),
                    host_id: String::new(),
                    resource: Some(self.bucket.to_string()),
                    string_to_sign: None,
                })),
                context: Box::new(ErrorContext {
                    operation: Some("PutBucketEncryption".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct PutBucketEncryptionOutput {
    pub request_id: String,
}

pub struct GetBucketEncryptionBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
}

impl GetBucketEncryptionBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName) -> Self {
        Self { client, bucket }
    }

    pub async fn send(self) -> Result<GetBucketEncryptionOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!("https://{}.{}?encryption", self.bucket.as_str(), endpoint);

        let query_params: Vec<(String, String)> = vec![("encryption".into(), String::new())];

        let request = HttpRequest::builder()
            .method(http::Method::GET)
            .uri(&uri)
            .build();

        let response = self
            .client
            .send_signed(request, Some(&self.bucket), query_params)
            .await
            .map_err(|e| OssError {
                kind: OssErrorKind::TransportError,
                context: Box::new(ErrorContext {
                    operation: Some("GetBucketEncryption".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.is_success() {
            let body_str = response.body_as_str().unwrap_or("");
            let config: SSEConfigResponse =
                crate::util::xml::from_xml(body_str).map_err(|e| OssError {
                    kind: OssErrorKind::DeserializationError,
                    context: Box::new(ErrorContext {
                        operation: Some("GetBucketEncryption: parse XML".into()),
                        bucket: Some(self.bucket.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?;

            Ok(GetBucketEncryptionOutput {
                sse_algorithm: config.apply.sse_algorithm,
                kms_master_key_id: config.apply.kms_master_key_id,
            })
        } else {
            Err(OssError {
                kind: OssErrorKind::ServiceError(Box::new(crate::error::OssServiceError {
                    status_code: response.status().as_u16(),
                    code: String::new(),
                    message: String::new(),
                    request_id: String::new(),
                    host_id: String::new(),
                    resource: Some(self.bucket.to_string()),
                    string_to_sign: None,
                })),
                context: Box::new(ErrorContext {
                    operation: Some("GetBucketEncryption".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct GetBucketEncryptionOutput {
    pub sse_algorithm: String,
    pub kms_master_key_id: Option<String>,
}

pub struct DeleteBucketEncryptionBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
}

impl DeleteBucketEncryptionBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName) -> Self {
        Self { client, bucket }
    }

    pub async fn send(self) -> Result<DeleteBucketEncryptionOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!("https://{}.{}?encryption", self.bucket.as_str(), endpoint);

        let query_params: Vec<(String, String)> = vec![("encryption".into(), String::new())];

        let request = HttpRequest::builder()
            .method(http::Method::DELETE)
            .uri(&uri)
            .build();

        let response = self
            .client
            .send_signed(request, Some(&self.bucket), query_params)
            .await
            .map_err(|e| OssError {
                kind: OssErrorKind::TransportError,
                context: Box::new(ErrorContext {
                    operation: Some("DeleteBucketEncryption".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.status().is_success() {
            Ok(DeleteBucketEncryptionOutput {
                request_id: response
                    .headers
                    .get("x-oss-request-id")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("")
                    .to_string(),
            })
        } else {
            Err(OssError {
                kind: OssErrorKind::ServiceError(Box::new(crate::error::OssServiceError {
                    status_code: response.status().as_u16(),
                    code: String::new(),
                    message: String::new(),
                    request_id: String::new(),
                    host_id: String::new(),
                    resource: Some(self.bucket.to_string()),
                    string_to_sign: None,
                })),
                context: Box::new(ErrorContext {
                    operation: Some("DeleteBucketEncryption".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeleteBucketEncryptionOutput {
    pub request_id: String,
}

impl BucketOperations {
    pub fn put_encryption(
        &self,
        config: ServerSideEncryptionConfiguration,
    ) -> PutBucketEncryptionBuilder {
        PutBucketEncryptionBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            config,
        )
    }

    pub fn get_encryption(&self) -> GetBucketEncryptionBuilder {
        GetBucketEncryptionBuilder::new(self.client_inner().clone(), self.bucket_name().clone())
    }

    pub fn delete_encryption(&self) -> DeleteBucketEncryptionBuilder {
        DeleteBucketEncryptionBuilder::new(self.client_inner().clone(), self.bucket_name().clone())
    }
}

#[cfg(test)]
mod tests {
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
    }

    #[async_trait::async_trait]
    impl HttpClient for RecordingHttpClient {
        async fn send(&self, request: HttpRequest) -> crate::error::Result<HttpResponse> {
            self.requests.lock().unwrap().push(request);
            let mut headers = http::HeaderMap::new();
            headers.insert(
                "x-oss-request-id",
                http::HeaderValue::from_static("rid-encryption"),
            );
            Ok(HttpResponse {
                status: self.status_code,
                headers,
                body: self.response_body.clone(),
            })
        }
    }

    fn create_test_inner_with_body(
        status: http::StatusCode,
        body: bytes::Bytes,
    ) -> (Arc<OSSClientInner>, Arc<Mutex<Vec<HttpRequest>>>) {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let http = Arc::new(RecordingHttpClient {
            requests: requests.clone(),
            status_code: status,
            response_body: body,
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
    fn encryption_xml_aes256() {
        let config = SSEConfig {
            apply: SSEApply {
                sse_algorithm: "AES256".into(),
                kms_master_key_id: None,
            },
        };
        let xml = crate::util::xml::to_xml(&config).unwrap();
        assert!(xml.contains("<SSEAlgorithm>AES256</SSEAlgorithm>"));
        assert!(!xml.contains("KMSMasterKeyID"));
    }

    #[test]
    fn encryption_xml_kms() {
        let config = SSEConfig {
            apply: SSEApply {
                sse_algorithm: "KMS".into(),
                kms_master_key_id: Some("key-id-123".into()),
            },
        };
        let xml = crate::util::xml::to_xml(&config).unwrap();
        assert!(xml.contains("<SSEAlgorithm>KMS</SSEAlgorithm>"));
        assert!(xml.contains("<KMSMasterKeyID>key-id-123</KMSMasterKeyID>"));
    }

    #[tokio::test]
    async fn get_bucket_encryption_parses_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ServerSideEncryptionRule>
  <ApplyServerSideEncryptionByDefault>
    <SSEAlgorithm>KMS</SSEAlgorithm>
    <KMSMasterKeyID>key-id</KMSMasterKeyID>
  </ApplyServerSideEncryptionByDefault>
</ServerSideEncryptionRule>"#;
        let (inner, _) = create_test_inner_with_body(http::StatusCode::OK, bytes::Bytes::from(xml));
        let builder =
            GetBucketEncryptionBuilder::new(inner, BucketName::new("test-bucket").unwrap());
        let output = builder.send().await.unwrap();
        assert_eq!(output.sse_algorithm, "KMS");
        assert_eq!(output.kms_master_key_id.as_deref(), Some("key-id"));
    }

    #[tokio::test]
    async fn delete_bucket_encryption_sends_delete_request() {
        let (inner, requests) =
            create_test_inner_with_body(http::StatusCode::NO_CONTENT, bytes::Bytes::new());
        let builder =
            DeleteBucketEncryptionBuilder::new(inner, BucketName::new("test-bucket").unwrap());
        builder.send().await.unwrap();
        let captured = requests.lock().unwrap();
        assert_eq!(captured[0].method, http::Method::DELETE);
    }
}
