use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::client::{BucketOperations, OSSClientInner};
use crate::error::{ErrorContext, OssError, OssErrorKind, Result};
use crate::http::client::HttpRequest;
use crate::types::bucket::BucketName;

#[derive(Debug, Clone, Serialize)]
#[serde(rename = "WebsiteConfiguration")]
struct WebsiteConfiguration {
    #[serde(rename = "IndexDocument")]
    index_document: IndexDocumentConfig,
    #[serde(rename = "ErrorDocument", skip_serializing_if = "Option::is_none")]
    error_document: Option<ErrorDocumentConfig>,
}

#[derive(Debug, Clone, Serialize)]
struct IndexDocumentConfig {
    #[serde(rename = "Suffix")]
    suffix: String,
}

#[derive(Debug, Clone, Serialize)]
struct ErrorDocumentConfig {
    #[serde(rename = "Key")]
    key: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "WebsiteConfiguration")]
struct WebsiteConfigurationResponse {
    #[serde(rename = "IndexDocument")]
    index_document: IndexDocumentResponse,
    #[serde(rename = "ErrorDocument", default)]
    error_document: Option<ErrorDocumentResponse>,
}

#[derive(Debug, Clone, Deserialize)]
struct IndexDocumentResponse {
    #[serde(rename = "Suffix")]
    suffix: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ErrorDocumentResponse {
    #[serde(rename = "Key")]
    key: String,
}

pub struct PutBucketWebsiteBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    index_suffix: String,
    error_key: Option<String>,
}

impl PutBucketWebsiteBuilder {
    pub(crate) fn new(
        client: Arc<OSSClientInner>,
        bucket: BucketName,
        index_suffix: impl Into<String>,
    ) -> Self {
        Self {
            client,
            bucket,
            index_suffix: index_suffix.into(),
            error_key: None,
        }
    }

    pub fn error_document(mut self, key: impl Into<String>) -> Self {
        self.error_key = Some(key.into());
        self
    }

    pub async fn send(self) -> Result<PutBucketWebsiteOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!("https://{}.{}?website", self.bucket.as_str(), endpoint);
        let query_params: Vec<(String, String)> = vec![("website".into(), String::new())];

        let config = WebsiteConfiguration {
            index_document: IndexDocumentConfig {
                suffix: self.index_suffix,
            },
            error_document: self.error_key.map(|key| ErrorDocumentConfig { key }),
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
                    operation: Some("PutBucketWebsite".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.status().is_success() {
            Ok(PutBucketWebsiteOutput {
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
                    operation: Some("PutBucketWebsite".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct PutBucketWebsiteOutput {
    pub request_id: String,
}

pub struct GetBucketWebsiteBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
}

impl GetBucketWebsiteBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName) -> Self {
        Self { client, bucket }
    }

    pub async fn send(self) -> Result<GetBucketWebsiteOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!("https://{}.{}?website", self.bucket.as_str(), endpoint);
        let query_params: Vec<(String, String)> = vec![("website".into(), String::new())];

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
                    operation: Some("GetBucketWebsite".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.is_success() {
            let body_str = response.body_as_str().unwrap_or("");
            let config: WebsiteConfigurationResponse = crate::util::xml::from_xml(body_str)
                .map_err(|e| OssError {
                    kind: OssErrorKind::DeserializationError,
                    context: Box::new(ErrorContext {
                        operation: Some("GetBucketWebsite: parse XML".into()),
                        bucket: Some(self.bucket.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?;

            Ok(GetBucketWebsiteOutput {
                index_suffix: config.index_document.suffix,
                error_key: config.error_document.map(|e| e.key),
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
                    operation: Some("GetBucketWebsite".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct GetBucketWebsiteOutput {
    pub index_suffix: String,
    pub error_key: Option<String>,
}

pub struct DeleteBucketWebsiteBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
}

impl DeleteBucketWebsiteBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName) -> Self {
        Self { client, bucket }
    }

    pub async fn send(self) -> Result<DeleteBucketWebsiteOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!("https://{}.{}?website", self.bucket.as_str(), endpoint);
        let query_params: Vec<(String, String)> = vec![("website".into(), String::new())];

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
                    operation: Some("DeleteBucketWebsite".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.status().is_success() {
            Ok(DeleteBucketWebsiteOutput {
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
                    operation: Some("DeleteBucketWebsite".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeleteBucketWebsiteOutput {
    pub request_id: String,
}

impl BucketOperations {
    pub fn put_website(&self, index_suffix: impl Into<String>) -> PutBucketWebsiteBuilder {
        PutBucketWebsiteBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            index_suffix,
        )
    }

    pub fn get_website(&self) -> GetBucketWebsiteBuilder {
        GetBucketWebsiteBuilder::new(self.client_inner().clone(), self.bucket_name().clone())
    }

    pub fn delete_website(&self) -> DeleteBucketWebsiteBuilder {
        DeleteBucketWebsiteBuilder::new(self.client_inner().clone(), self.bucket_name().clone())
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
                http::HeaderValue::from_static("rid-website"),
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
    fn website_xml_generation() {
        let config = WebsiteConfiguration {
            index_document: IndexDocumentConfig {
                suffix: "index.html".into(),
            },
            error_document: Some(ErrorDocumentConfig {
                key: "error.html".into(),
            }),
        };
        let xml = crate::util::xml::to_xml(&config).unwrap();
        assert!(xml.contains("<Suffix>index.html</Suffix>"));
        assert!(xml.contains("<Key>error.html</Key>"));
    }

    #[tokio::test]
    async fn delete_website_sends_delete_request() {
        let (inner, requests) =
            create_test_inner_with_body(http::StatusCode::NO_CONTENT, bytes::Bytes::new());
        let builder =
            DeleteBucketWebsiteBuilder::new(inner, BucketName::new("test-bucket").unwrap());
        builder.send().await.unwrap();
        let captured = requests.lock().unwrap();
        assert_eq!(captured[0].method, http::Method::DELETE);
    }
}
