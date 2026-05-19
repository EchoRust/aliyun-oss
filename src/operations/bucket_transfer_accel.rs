use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::client::{BucketOperations, OSSClientInner};
use crate::error::{ErrorContext, OssError, OssErrorKind, Result};
use crate::http::client::HttpRequest;
use crate::types::bucket::BucketName;

#[derive(Debug, Clone, Serialize)]
#[serde(rename = "TransferAccelerationConfiguration")]
struct TransferAccelConfiguration {
    #[serde(rename = "Enabled")]
    enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "TransferAccelerationConfiguration")]
struct TransferAccelConfigurationResponse {
    #[serde(rename = "Enabled")]
    enabled: bool,
}

pub struct PutBucketTransferAccelBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    enabled: bool,
}

impl PutBucketTransferAccelBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName, enabled: bool) -> Self {
        Self {
            client,
            bucket,
            enabled,
        }
    }

    pub async fn send(self) -> Result<PutBucketTransferAccelOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!(
            "https://{}.{}?transferAcceleration",
            self.bucket.as_str(),
            endpoint
        );
        let query_params: Vec<(String, String)> =
            vec![("transferAcceleration".into(), String::new())];

        let config = TransferAccelConfiguration {
            enabled: self.enabled,
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
                    operation: Some("PutBucketTransferAccel".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.status().is_success() {
            Ok(PutBucketTransferAccelOutput {
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
                    operation: Some("PutBucketTransferAccel".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct PutBucketTransferAccelOutput {
    pub request_id: String,
}

pub struct GetBucketTransferAccelBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
}

impl GetBucketTransferAccelBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName) -> Self {
        Self { client, bucket }
    }

    pub async fn send(self) -> Result<GetBucketTransferAccelOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!(
            "https://{}.{}?transferAcceleration",
            self.bucket.as_str(),
            endpoint
        );
        let query_params: Vec<(String, String)> =
            vec![("transferAcceleration".into(), String::new())];

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
                    operation: Some("GetBucketTransferAccel".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.is_success() {
            let body_str = response.body_as_str().unwrap_or("");
            let config: TransferAccelConfigurationResponse = crate::util::xml::from_xml(body_str)
                .map_err(|e| OssError {
                kind: OssErrorKind::DeserializationError,
                context: Box::new(ErrorContext {
                    operation: Some("GetBucketTransferAccel: parse XML".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

            Ok(GetBucketTransferAccelOutput {
                enabled: config.enabled,
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
                    operation: Some("GetBucketTransferAccel".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct GetBucketTransferAccelOutput {
    pub enabled: bool,
}

impl BucketOperations {
    pub fn put_transfer_acceleration(&self, enabled: bool) -> PutBucketTransferAccelBuilder {
        PutBucketTransferAccelBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            enabled,
        )
    }

    pub fn get_transfer_acceleration(&self) -> GetBucketTransferAccelBuilder {
        GetBucketTransferAccelBuilder::new(self.client_inner().clone(), self.bucket_name().clone())
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
            headers.insert("x-oss-request-id", http::HeaderValue::from_static("rid-ta"));
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
    fn transfer_accel_xml_enabled() {
        let config = TransferAccelConfiguration { enabled: true };
        let xml = crate::util::xml::to_xml(&config).unwrap();
        assert!(xml.contains("<Enabled>true</Enabled>"));
    }

    #[tokio::test]
    async fn get_transfer_accel_parses_response() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<TransferAccelerationConfiguration><Enabled>true</Enabled></TransferAccelerationConfiguration>"#;
        let (inner, _) = create_test_inner(bytes::Bytes::from(xml));
        let builder =
            GetBucketTransferAccelBuilder::new(inner, BucketName::new("test-bucket").unwrap());
        let output = builder.send().await.unwrap();
        assert!(output.enabled);
    }
}
