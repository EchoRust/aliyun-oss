//! Bucket referer (hotlink protection) operations.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::client::{BucketOperations, OSSClientInner};
use crate::error::{ErrorContext, OssError, OssErrorKind, Result};
use crate::http::client::HttpRequest;
use crate::types::bucket::BucketName;

#[derive(Debug, Clone, Serialize)]
#[serde(rename = "RefererConfiguration")]
struct RefererConfiguration {
    #[serde(rename = "AllowEmptyReferer")]
    allow_empty_referer: bool,
    #[serde(rename = "RefererList", skip_serializing_if = "Option::is_none")]
    referer_list: Option<RefererList>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RefererList {
    #[serde(rename = "Referer")]
    referer: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "RefererConfiguration")]
struct RefererConfigurationResponse {
    #[serde(rename = "AllowEmptyReferer")]
    allow_empty_referer: bool,
    #[serde(rename = "RefererList", default)]
    referer_list: Option<RefererList>,
}

pub struct PutBucketRefererBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    allow_empty_referer: bool,
    referers: Vec<String>,
}

impl PutBucketRefererBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName) -> Self {
        Self {
            client,
            bucket,
            allow_empty_referer: true,
            referers: Vec::new(),
        }
    }

    pub fn allow_empty_referer(mut self, allow: bool) -> Self {
        self.allow_empty_referer = allow;
        self
    }

    pub fn add_referer(mut self, referer: impl Into<String>) -> Self {
        self.referers.push(referer.into());
        self
    }

    pub async fn send(self) -> Result<PutBucketRefererOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!("https://{}.{}?referer", self.bucket.as_str(), endpoint);
        let query_params: Vec<(String, String)> = vec![("referer".into(), String::new())];

        let config = RefererConfiguration {
            allow_empty_referer: self.allow_empty_referer,
            referer_list: if self.referers.is_empty() {
                None
            } else {
                Some(RefererList {
                    referer: self.referers,
                })
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
                    operation: Some("PutBucketReferer".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.status().is_success() {
            Ok(PutBucketRefererOutput {
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
                    operation: Some("PutBucketReferer".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct PutBucketRefererOutput {
    pub request_id: String,
}

pub struct GetBucketRefererBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
}

impl GetBucketRefererBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName) -> Self {
        Self { client, bucket }
    }

    pub async fn send(self) -> Result<GetBucketRefererOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!("https://{}.{}?referer", self.bucket.as_str(), endpoint);
        let query_params: Vec<(String, String)> = vec![("referer".into(), String::new())];

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
                    operation: Some("GetBucketReferer".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.is_success() {
            let body_str = response.body_as_str().unwrap_or("");
            let config: RefererConfigurationResponse = crate::util::xml::from_xml(body_str)
                .map_err(|e| OssError {
                    kind: OssErrorKind::DeserializationError,
                    context: Box::new(ErrorContext {
                        operation: Some("GetBucketReferer: parse XML".into()),
                        bucket: Some(self.bucket.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?;

            Ok(GetBucketRefererOutput {
                allow_empty_referer: config.allow_empty_referer,
                referers: config.referer_list.map(|rl| rl.referer).unwrap_or_default(),
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
                    operation: Some("GetBucketReferer".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct GetBucketRefererOutput {
    pub allow_empty_referer: bool,
    pub referers: Vec<String>,
}

impl BucketOperations {
    pub fn put_referer(&self) -> PutBucketRefererBuilder {
        PutBucketRefererBuilder::new(self.client_inner().clone(), self.bucket_name().clone())
    }

    pub fn get_referer(&self) -> GetBucketRefererBuilder {
        GetBucketRefererBuilder::new(self.client_inner().clone(), self.bucket_name().clone())
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
                http::HeaderValue::from_static("rid-referer"),
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
    fn referer_xml_generation() {
        let config = RefererConfiguration {
            allow_empty_referer: true,
            referer_list: Some(RefererList {
                referer: vec!["https://example.com".into(), "https://*.example.com".into()],
            }),
        };
        let xml = crate::util::xml::to_xml(&config).unwrap();
        assert!(xml.contains("<AllowEmptyReferer>true</AllowEmptyReferer>"));
        assert!(xml.contains("<Referer>https://example.com</Referer>"));
    }

    #[tokio::test]
    async fn put_referer_sends_request() {
        let (inner, requests) =
            create_test_inner_with_body(http::StatusCode::OK, bytes::Bytes::new());
        let builder = PutBucketRefererBuilder::new(inner, BucketName::new("test-bucket").unwrap())
            .add_referer("https://example.com");
        builder.send().await.unwrap();
        let captured = requests.lock().unwrap();
        assert_eq!(captured[0].method, http::Method::PUT);
        assert!(captured[0].uri.contains("?referer"));
    }
}
