//! Bucket CORS configuration operations.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::client::{BucketOperations, OSSClientInner};
use crate::error::{ErrorContext, OssError, OssErrorKind, Result};
use crate::http::client::HttpRequest;
use crate::types::bucket::BucketName;

#[derive(Debug, Clone)]
pub struct CorsRule {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub expose_headers: Vec<String>,
    pub max_age_seconds: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "CORSConfiguration")]
struct CORSConfiguration {
    #[serde(rename = "CORSRule")]
    rules: Vec<CorsRuleData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CorsRuleData {
    #[serde(rename = "AllowedOrigin")]
    allowed_origin: Vec<String>,
    #[serde(rename = "AllowedMethod")]
    allowed_method: Vec<String>,
    #[serde(
        rename = "AllowedHeader",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    allowed_header: Vec<String>,
    #[serde(
        rename = "ExposeHeader",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    expose_header: Vec<String>,
    #[serde(
        rename = "MaxAgeSeconds",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    max_age_seconds: Option<i32>,
}

pub struct PutBucketCorsBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    rules: Vec<CorsRule>,
}

impl PutBucketCorsBuilder {
    pub(crate) fn new(
        client: Arc<OSSClientInner>,
        bucket: BucketName,
        rules: Vec<CorsRule>,
    ) -> Self {
        Self {
            client,
            bucket,
            rules,
        }
    }

    pub async fn send(self) -> Result<PutBucketCorsOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!("https://{}.{}?cors", self.bucket.as_str(), endpoint);

        let query_params: Vec<(String, String)> = vec![("cors".into(), String::new())];

        let config = CORSConfiguration {
            rules: self
                .rules
                .into_iter()
                .map(|r| CorsRuleData {
                    allowed_origin: r.allowed_origins,
                    allowed_method: r.allowed_methods,
                    allowed_header: r.allowed_headers,
                    expose_header: r.expose_headers,
                    max_age_seconds: r.max_age_seconds,
                })
                .collect(),
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
                    operation: Some("PutBucketCors".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.status().is_success() {
            Ok(PutBucketCorsOutput {
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
                    operation: Some("PutBucketCors".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct PutBucketCorsOutput {
    pub request_id: String,
}

pub struct GetBucketCorsBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
}

impl GetBucketCorsBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName) -> Self {
        Self { client, bucket }
    }

    pub async fn send(self) -> Result<GetBucketCorsOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!("https://{}.{}?cors", self.bucket.as_str(), endpoint);

        let query_params: Vec<(String, String)> = vec![("cors".into(), String::new())];

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
                    operation: Some("GetBucketCors".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.is_success() {
            let body_str = response.body_as_str().unwrap_or("");
            let config: CORSConfiguration =
                crate::util::xml::from_xml(body_str).map_err(|e| OssError {
                    kind: OssErrorKind::DeserializationError,
                    context: Box::new(ErrorContext {
                        operation: Some("GetBucketCors: parse XML".into()),
                        bucket: Some(self.bucket.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?;

            Ok(GetBucketCorsOutput {
                rules: config
                    .rules
                    .into_iter()
                    .map(|r| CorsRule {
                        allowed_origins: r.allowed_origin,
                        allowed_methods: r.allowed_method,
                        allowed_headers: r.allowed_header,
                        expose_headers: r.expose_header,
                        max_age_seconds: r.max_age_seconds,
                    })
                    .collect(),
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
                    operation: Some("GetBucketCors".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct GetBucketCorsOutput {
    pub rules: Vec<CorsRule>,
}

pub struct DeleteBucketCorsBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
}

impl DeleteBucketCorsBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName) -> Self {
        Self { client, bucket }
    }

    pub async fn send(self) -> Result<DeleteBucketCorsOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!("https://{}.{}?cors", self.bucket.as_str(), endpoint);

        let query_params: Vec<(String, String)> = vec![("cors".into(), String::new())];

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
                    operation: Some("DeleteBucketCors".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.status().is_success() {
            Ok(DeleteBucketCorsOutput {
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
                    operation: Some("DeleteBucketCors".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeleteBucketCorsOutput {
    pub request_id: String,
}

impl BucketOperations {
    pub fn put_cors(&self, rules: Vec<CorsRule>) -> PutBucketCorsBuilder {
        PutBucketCorsBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            rules,
        )
    }

    pub fn get_cors(&self) -> GetBucketCorsBuilder {
        GetBucketCorsBuilder::new(self.client_inner().clone(), self.bucket_name().clone())
    }

    pub fn delete_cors(&self) -> DeleteBucketCorsBuilder {
        DeleteBucketCorsBuilder::new(self.client_inner().clone(), self.bucket_name().clone())
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
                http::HeaderValue::from_static("rid-cors"),
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
    fn cors_xml_generation() {
        let config = CORSConfiguration {
            rules: vec![CorsRuleData {
                allowed_origin: vec!["*".into()],
                allowed_method: vec!["GET".into(), "PUT".into()],
                allowed_header: vec!["*".into()],
                expose_header: vec![],
                max_age_seconds: Some(3600),
            }],
        };
        let xml = crate::util::xml::to_xml(&config).unwrap();
        assert!(xml.contains("<AllowedOrigin>*</AllowedOrigin>"));
        assert!(xml.contains("<AllowedMethod>GET</AllowedMethod>"));
        assert!(xml.contains("<MaxAgeSeconds>3600</MaxAgeSeconds>"));
    }

    #[tokio::test]
    async fn get_bucket_cors_parses_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<CORSConfiguration>
  <CORSRule>
    <AllowedOrigin>*</AllowedOrigin>
    <AllowedMethod>GET</AllowedMethod>
  </CORSRule>
</CORSConfiguration>"#;
        let (inner, _) = create_test_inner_with_body(http::StatusCode::OK, bytes::Bytes::from(xml));
        let builder = GetBucketCorsBuilder::new(inner, BucketName::new("test-bucket").unwrap());
        let output = builder.send().await.unwrap();
        assert_eq!(output.rules.len(), 1);
    }

    #[tokio::test]
    async fn delete_bucket_cors_sends_delete_request() {
        let (inner, requests) =
            create_test_inner_with_body(http::StatusCode::NO_CONTENT, bytes::Bytes::new());
        let builder = DeleteBucketCorsBuilder::new(inner, BucketName::new("test-bucket").unwrap());
        builder.send().await.unwrap();
        let captured = requests.lock().unwrap();
        assert_eq!(captured[0].method, http::Method::DELETE);
    }
}
