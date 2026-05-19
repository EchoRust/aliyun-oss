//! Requester-pays configuration operations.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::client::{BucketOperations, OSSClientInner};
use crate::error::{ErrorContext, OssError, OssErrorKind, Result};
use crate::http::client::HttpRequest;
use crate::types::bucket::BucketName;

#[derive(Debug, Clone, Serialize)]
#[serde(rename = "RequestPaymentConfiguration")]
struct RequestPaymentConfiguration {
    #[serde(rename = "Payer")]
    payer: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "RequestPaymentConfiguration")]
struct RequestPaymentConfigurationResponse {
    #[serde(rename = "Payer")]
    payer: String,
}

pub struct PutBucketRequestPaymentBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    payer: String,
}

impl PutBucketRequestPaymentBuilder {
    pub(crate) fn new(
        client: Arc<OSSClientInner>,
        bucket: BucketName,
        payer: impl Into<String>,
    ) -> Self {
        Self {
            client,
            bucket,
            payer: payer.into(),
        }
    }

    pub async fn send(self) -> Result<PutBucketRequestPaymentOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!(
            "https://{}.{}?requestPayment",
            self.bucket.as_str(),
            endpoint
        );
        let query_params: Vec<(String, String)> = vec![("requestPayment".into(), String::new())];

        let config = RequestPaymentConfiguration { payer: self.payer };
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
                    operation: Some("PutBucketRequestPayment".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.status().is_success() {
            Ok(PutBucketRequestPaymentOutput {
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
                    operation: Some("PutBucketRequestPayment".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct PutBucketRequestPaymentOutput {
    pub request_id: String,
}

pub struct GetBucketRequestPaymentBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
}

impl GetBucketRequestPaymentBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName) -> Self {
        Self { client, bucket }
    }

    pub async fn send(self) -> Result<GetBucketRequestPaymentOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!(
            "https://{}.{}?requestPayment",
            self.bucket.as_str(),
            endpoint
        );
        let query_params: Vec<(String, String)> = vec![("requestPayment".into(), String::new())];

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
                    operation: Some("GetBucketRequestPayment".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.is_success() {
            let body_str = response.body_as_str().unwrap_or("");
            let config: RequestPaymentConfigurationResponse = crate::util::xml::from_xml(body_str)
                .map_err(|e| OssError {
                    kind: OssErrorKind::DeserializationError,
                    context: Box::new(ErrorContext {
                        operation: Some("GetBucketRequestPayment: parse XML".into()),
                        bucket: Some(self.bucket.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?;

            Ok(GetBucketRequestPaymentOutput {
                payer: config.payer,
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
                    operation: Some("GetBucketRequestPayment".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct GetBucketRequestPaymentOutput {
    pub payer: String,
}

impl BucketOperations {
    pub fn put_request_payment(&self, payer: impl Into<String>) -> PutBucketRequestPaymentBuilder {
        PutBucketRequestPaymentBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            payer,
        )
    }

    pub fn get_request_payment(&self) -> GetBucketRequestPaymentBuilder {
        GetBucketRequestPaymentBuilder::new(self.client_inner().clone(), self.bucket_name().clone())
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
            headers.insert("x-oss-request-id", http::HeaderValue::from_static("rid-rp"));
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
    fn request_payment_xml() {
        let config = RequestPaymentConfiguration {
            payer: "BucketOwner".into(),
        };
        let xml = crate::util::xml::to_xml(&config).unwrap();
        assert!(xml.contains("<Payer>BucketOwner</Payer>"));
    }

    #[tokio::test]
    async fn get_request_payment_parses_response() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<RequestPaymentConfiguration><Payer>Requester</Payer></RequestPaymentConfiguration>"#;
        let (inner, _) = create_test_inner(bytes::Bytes::from(xml));
        let builder =
            GetBucketRequestPaymentBuilder::new(inner, BucketName::new("test-bucket").unwrap());
        let output = builder.send().await.unwrap();
        assert_eq!(output.payer, "Requester");
    }
}
