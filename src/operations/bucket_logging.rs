//! Bucket access logging operations.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::client::{BucketOperations, OSSClientInner};
use crate::error::{ErrorContext, OssError, OssErrorKind, Result};
use crate::http::client::HttpRequest;
use crate::types::bucket::BucketName;

#[derive(Debug, Clone, Serialize)]
#[serde(rename = "BucketLoggingStatus")]
struct BucketLoggingConfiguration {
    #[serde(rename = "LoggingEnabled", skip_serializing_if = "Option::is_none")]
    logging_enabled: Option<LoggingEnabled>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LoggingEnabled {
    #[serde(rename = "TargetBucket")]
    target_bucket: String,
    #[serde(rename = "TargetPrefix")]
    target_prefix: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "BucketLoggingStatus")]
struct BucketLoggingConfigurationResponse {
    #[serde(rename = "LoggingEnabled", default)]
    logging_enabled: Option<LoggingEnabled>,
}

pub struct PutBucketLoggingBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    target_bucket: Option<String>,
    target_prefix: Option<String>,
}

impl PutBucketLoggingBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName) -> Self {
        Self {
            client,
            bucket,
            target_bucket: None,
            target_prefix: None,
        }
    }

    pub fn target_bucket(mut self, v: impl Into<String>) -> Self {
        self.target_bucket = Some(v.into());
        self
    }

    pub fn target_prefix(mut self, v: impl Into<String>) -> Self {
        self.target_prefix = Some(v.into());
        self
    }

    pub async fn send(self) -> Result<PutBucketLoggingOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!("https://{}.{}?logging", self.bucket.as_str(), endpoint);
        let query_params: Vec<(String, String)> = vec![("logging".into(), String::new())];

        let config = BucketLoggingConfiguration {
            logging_enabled: match (&self.target_bucket, &self.target_prefix) {
                (Some(tb), Some(tp)) => Some(LoggingEnabled {
                    target_bucket: tb.clone(),
                    target_prefix: tp.clone(),
                }),
                _ => None,
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
                    operation: Some("PutBucketLogging".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.status().is_success() {
            Ok(PutBucketLoggingOutput {
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
                    operation: Some("PutBucketLogging".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct PutBucketLoggingOutput {
    pub request_id: String,
}

pub struct GetBucketLoggingBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
}

impl GetBucketLoggingBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName) -> Self {
        Self { client, bucket }
    }

    pub async fn send(self) -> Result<GetBucketLoggingOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!("https://{}.{}?logging", self.bucket.as_str(), endpoint);
        let query_params: Vec<(String, String)> = vec![("logging".into(), String::new())];

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
                    operation: Some("GetBucketLogging".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.is_success() {
            let body_str = response.body_as_str().unwrap_or("");
            let config: BucketLoggingConfigurationResponse = crate::util::xml::from_xml(body_str)
                .map_err(|e| OssError {
                kind: OssErrorKind::DeserializationError,
                context: Box::new(ErrorContext {
                    operation: Some("GetBucketLogging: parse XML".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

            Ok(GetBucketLoggingOutput {
                target_bucket: config
                    .logging_enabled
                    .as_ref()
                    .map(|l| l.target_bucket.clone()),
                target_prefix: config.logging_enabled.map(|l| l.target_prefix),
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
                    operation: Some("GetBucketLogging".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct GetBucketLoggingOutput {
    pub target_bucket: Option<String>,
    pub target_prefix: Option<String>,
}

pub struct DeleteBucketLoggingBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
}

impl DeleteBucketLoggingBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName) -> Self {
        Self { client, bucket }
    }

    pub async fn send(self) -> Result<DeleteBucketLoggingOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!("https://{}.{}?logging", self.bucket.as_str(), endpoint);
        let query_params: Vec<(String, String)> = vec![("logging".into(), String::new())];

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
                    operation: Some("DeleteBucketLogging".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.status().is_success() {
            Ok(DeleteBucketLoggingOutput {
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
                    operation: Some("DeleteBucketLogging".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeleteBucketLoggingOutput {
    pub request_id: String,
}

impl BucketOperations {
    pub fn put_logging(&self) -> PutBucketLoggingBuilder {
        PutBucketLoggingBuilder::new(self.client_inner().clone(), self.bucket_name().clone())
    }

    pub fn get_logging(&self) -> GetBucketLoggingBuilder {
        GetBucketLoggingBuilder::new(self.client_inner().clone(), self.bucket_name().clone())
    }

    pub fn delete_logging(&self) -> DeleteBucketLoggingBuilder {
        DeleteBucketLoggingBuilder::new(self.client_inner().clone(), self.bucket_name().clone())
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
                http::HeaderValue::from_static("rid-logging"),
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
    fn logging_xml_generation() {
        let config = BucketLoggingConfiguration {
            logging_enabled: Some(LoggingEnabled {
                target_bucket: "logs-bucket".into(),
                target_prefix: "access-log/".into(),
            }),
        };
        let xml = crate::util::xml::to_xml(&config).unwrap();
        assert!(xml.contains("<TargetBucket>logs-bucket</TargetBucket>"));
        assert!(xml.contains("<TargetPrefix>access-log/</TargetPrefix>"));
    }

    #[tokio::test]
    async fn delete_logging_sends_delete_request() {
        let (inner, requests) =
            create_test_inner_with_body(http::StatusCode::NO_CONTENT, bytes::Bytes::new());
        let builder =
            DeleteBucketLoggingBuilder::new(inner, BucketName::new("test-bucket").unwrap());
        builder.send().await.unwrap();
        let captured = requests.lock().unwrap();
        assert_eq!(captured[0].method, http::Method::DELETE);
    }
}
