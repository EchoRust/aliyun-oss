//! WORM (Write Once Read Many) operations.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::client::{BucketOperations, OSSClientInner};
use crate::error::{ErrorContext, OssError, OssErrorKind, Result};
use crate::http::client::HttpRequest;
use crate::types::bucket::BucketName;

#[derive(Debug, Clone, Serialize)]
#[serde(rename = "InitiateWormConfiguration")]
struct InitiateWormConfiguration {
    #[serde(rename = "RetentionPeriodInDays")]
    retention_period_days: i32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename = "ExtendWormConfiguration")]
struct ExtendWormConfiguration {
    #[serde(rename = "RetentionPeriodInDays")]
    retention_period_days: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "WormConfiguration")]
struct WormConfigurationResponse {
    #[serde(rename = "WormId")]
    worm_id: String,
    #[serde(rename = "State")]
    state: String,
    #[serde(rename = "CreationDate")]
    creation_date: String,
    #[serde(rename = "RetentionPeriodInDays")]
    retention_period_days: i32,
}

pub struct InitiateBucketWormBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    retention_days: i32,
}

impl InitiateBucketWormBuilder {
    pub(crate) fn new(
        client: Arc<OSSClientInner>,
        bucket: BucketName,
        retention_days: i32,
    ) -> Self {
        Self {
            client,
            bucket,
            retention_days,
        }
    }

    pub async fn send(self) -> Result<InitiateBucketWormOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!("https://{}.{}?worm", self.bucket.as_str(), endpoint);
        let query_params: Vec<(String, String)> = vec![("worm".into(), String::new())];

        let config = InitiateWormConfiguration {
            retention_period_days: self.retention_days,
        };
        let body_xml = crate::util::xml::to_xml(&config)?;

        let request = HttpRequest::builder()
            .method(http::Method::POST)
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
                    operation: Some("InitiateBucketWorm".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.status().is_success() {
            Ok(InitiateBucketWormOutput {
                request_id: response
                    .headers
                    .get("x-oss-request-id")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("")
                    .to_string(),
                worm_id: response
                    .headers
                    .get("x-oss-worm-id")
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
                    operation: Some("InitiateBucketWorm".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct InitiateBucketWormOutput {
    pub request_id: String,
    pub worm_id: String,
}

pub struct AbortBucketWormBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
}

impl AbortBucketWormBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName) -> Self {
        Self { client, bucket }
    }

    pub async fn send(self) -> Result<AbortBucketWormOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!("https://{}.{}?worm", self.bucket.as_str(), endpoint);
        let query_params: Vec<(String, String)> = vec![("worm".into(), String::new())];

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
                    operation: Some("AbortBucketWorm".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.status().is_success() {
            Ok(AbortBucketWormOutput {
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
                    operation: Some("AbortBucketWorm".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct AbortBucketWormOutput {
    pub request_id: String,
}

pub struct CompleteBucketWormBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    worm_id: String,
}

impl CompleteBucketWormBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName, worm_id: String) -> Self {
        Self {
            client,
            bucket,
            worm_id,
        }
    }

    pub async fn send(self) -> Result<CompleteBucketWormOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!(
            "https://{}.{}?wormId={}",
            self.bucket.as_str(),
            endpoint,
            self.worm_id
        );
        let query_params: Vec<(String, String)> = vec![("wormId".into(), self.worm_id)];

        let request = HttpRequest::builder()
            .method(http::Method::PUT)
            .uri(&uri)
            .build();

        let response = self
            .client
            .send_signed(request, Some(&self.bucket), query_params)
            .await
            .map_err(|e| OssError {
                kind: OssErrorKind::TransportError,
                context: Box::new(ErrorContext {
                    operation: Some("CompleteBucketWorm".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.status().is_success() {
            Ok(CompleteBucketWormOutput {
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
                    operation: Some("CompleteBucketWorm".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompleteBucketWormOutput {
    pub request_id: String,
}

pub struct ExtendBucketWormBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    worm_id: String,
    extension_days: i32,
}

impl ExtendBucketWormBuilder {
    pub(crate) fn new(
        client: Arc<OSSClientInner>,
        bucket: BucketName,
        worm_id: String,
        extension_days: i32,
    ) -> Self {
        Self {
            client,
            bucket,
            worm_id,
            extension_days,
        }
    }

    pub async fn send(self) -> Result<ExtendBucketWormOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!(
            "https://{}.{}?wormId={}&wormExtend",
            self.bucket.as_str(),
            endpoint,
            self.worm_id
        );
        let query_params: Vec<(String, String)> = vec![
            ("wormId".into(), self.worm_id),
            ("wormExtend".into(), String::new()),
        ];

        let config = ExtendWormConfiguration {
            retention_period_days: self.extension_days,
        };
        let body_xml = crate::util::xml::to_xml(&config)?;

        let request = HttpRequest::builder()
            .method(http::Method::POST)
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
                    operation: Some("ExtendBucketWorm".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.status().is_success() {
            Ok(ExtendBucketWormOutput {
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
                    operation: Some("ExtendBucketWorm".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExtendBucketWormOutput {
    pub request_id: String,
}

pub struct GetBucketWormBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
}

impl GetBucketWormBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName) -> Self {
        Self { client, bucket }
    }

    pub async fn send(self) -> Result<GetBucketWormOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!("https://{}.{}?worm", self.bucket.as_str(), endpoint);
        let query_params: Vec<(String, String)> = vec![("worm".into(), String::new())];

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
                    operation: Some("GetBucketWorm".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.is_success() {
            let body_str = response.body_as_str().unwrap_or("");
            let config: WormConfigurationResponse =
                crate::util::xml::from_xml(body_str).map_err(|e| OssError {
                    kind: OssErrorKind::DeserializationError,
                    context: Box::new(ErrorContext {
                        operation: Some("GetBucketWorm: parse XML".into()),
                        bucket: Some(self.bucket.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?;

            Ok(GetBucketWormOutput {
                worm_id: config.worm_id,
                state: config.state,
                creation_date: config.creation_date,
                retention_period_days: config.retention_period_days,
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
                    operation: Some("GetBucketWorm".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct GetBucketWormOutput {
    pub worm_id: String,
    pub state: String,
    pub creation_date: String,
    pub retention_period_days: i32,
}

impl BucketOperations {
    pub fn initiate_worm(&self, retention_days: i32) -> InitiateBucketWormBuilder {
        InitiateBucketWormBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            retention_days,
        )
    }

    pub fn abort_worm(&self) -> AbortBucketWormBuilder {
        AbortBucketWormBuilder::new(self.client_inner().clone(), self.bucket_name().clone())
    }

    pub fn complete_worm(&self, worm_id: String) -> CompleteBucketWormBuilder {
        CompleteBucketWormBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            worm_id,
        )
    }

    pub fn extend_worm(&self, worm_id: String, extension_days: i32) -> ExtendBucketWormBuilder {
        ExtendBucketWormBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            worm_id,
            extension_days,
        )
    }

    pub fn get_worm(&self) -> GetBucketWormBuilder {
        GetBucketWormBuilder::new(self.client_inner().clone(), self.bucket_name().clone())
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
                http::HeaderValue::from_static("rid-worm"),
            );
            Ok(HttpResponse {
                status: self.status_code,
                headers,
                body: self.response_body.clone(),
            })
        }
    }

    fn create_test_inner(
        body: bytes::Bytes,
    ) -> (Arc<OSSClientInner>, Arc<Mutex<Vec<HttpRequest>>>) {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let http = Arc::new(RecordingHttpClient {
            requests: requests.clone(),
            status_code: http::StatusCode::OK,
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
    fn worm_initiate_xml() {
        let config = InitiateWormConfiguration {
            retention_period_days: 365,
        };
        let xml = crate::util::xml::to_xml(&config).unwrap();
        assert!(xml.contains("<RetentionPeriodInDays>365</RetentionPeriodInDays>"));
    }

    #[tokio::test]
    async fn get_worm_parses_response() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<WormConfiguration>
  <WormId>worm-123</WormId>
  <State>Locked</State>
  <CreationDate>2024-01-01T00:00:00.000Z</CreationDate>
  <RetentionPeriodInDays>365</RetentionPeriodInDays>
</WormConfiguration>"#;
        let (inner, _) = create_test_inner(bytes::Bytes::from(xml));
        let builder = GetBucketWormBuilder::new(inner, BucketName::new("test-bucket").unwrap());
        let output = builder.send().await.unwrap();
        assert_eq!(output.worm_id, "worm-123");
        assert_eq!(output.state, "Locked");
    }
}
