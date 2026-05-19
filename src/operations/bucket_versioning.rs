use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::client::{BucketOperations, OSSClientInner};
use crate::error::{ErrorContext, OssError, OssErrorKind, Result};
use crate::http::client::HttpRequest;
use crate::types::bucket::BucketName;

#[derive(Debug, Clone, Serialize)]
#[serde(rename = "VersioningConfiguration")]
struct VersioningConfiguration {
    #[serde(rename = "Status")]
    status: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "VersioningConfiguration")]
struct VersioningConfigurationResponse {
    #[serde(rename = "Status")]
    status: String,
}

pub struct PutBucketVersioningBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    status: String,
}

impl PutBucketVersioningBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName, status: String) -> Self {
        Self {
            client,
            bucket,
            status,
        }
    }

    pub async fn send(self) -> Result<PutBucketVersioningOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!("https://{}.{}?versioning", self.bucket.as_str(), endpoint);

        let query_params: Vec<(String, String)> = vec![("versioning".into(), String::new())];

        let config = VersioningConfiguration {
            status: self.status,
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
                    operation: Some("PutBucketVersioning".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.status().is_success() {
            Ok(PutBucketVersioningOutput {
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
                    operation: Some("PutBucketVersioning".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct PutBucketVersioningOutput {
    pub request_id: String,
}

pub struct GetBucketVersioningBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
}

impl GetBucketVersioningBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName) -> Self {
        Self { client, bucket }
    }

    pub async fn send(self) -> Result<GetBucketVersioningOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!("https://{}.{}?versioning", self.bucket.as_str(), endpoint);

        let query_params: Vec<(String, String)> = vec![("versioning".into(), String::new())];

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
                    operation: Some("GetBucketVersioning".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.is_success() {
            let body_str = response.body_as_str().unwrap_or("");
            let config: VersioningConfigurationResponse = crate::util::xml::from_xml(body_str)
                .map_err(|e| OssError {
                    kind: OssErrorKind::DeserializationError,
                    context: Box::new(ErrorContext {
                        operation: Some("GetBucketVersioning: parse XML".into()),
                        bucket: Some(self.bucket.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?;

            Ok(GetBucketVersioningOutput {
                status: config.status,
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
                    operation: Some("GetBucketVersioning".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct GetBucketVersioningOutput {
    pub status: String,
}

impl BucketOperations {
    pub fn put_versioning(&self, status: impl Into<String>) -> PutBucketVersioningBuilder {
        PutBucketVersioningBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            status.into(),
        )
    }

    pub fn get_versioning(&self) -> GetBucketVersioningBuilder {
        GetBucketVersioningBuilder::new(self.client_inner().clone(), self.bucket_name().clone())
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
    }

    #[async_trait::async_trait]
    impl HttpClient for RecordingHttpClient {
        async fn send(&self, request: HttpRequest) -> crate::error::Result<HttpResponse> {
            self.requests.lock().unwrap().push(request);
            let mut headers = http::HeaderMap::new();
            headers.insert(
                "x-oss-request-id",
                http::HeaderValue::from_static("rid-versioning"),
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
    fn put_versioning_xml_enabled() {
        let config = VersioningConfiguration {
            status: "Enabled".into(),
        };
        let xml = crate::util::xml::to_xml(&config).unwrap();
        assert!(xml.contains("<Status>Enabled</Status>"));
    }

    #[test]
    fn put_versioning_xml_suspended() {
        let config = VersioningConfiguration {
            status: "Suspended".into(),
        };
        let xml = crate::util::xml::to_xml(&config).unwrap();
        assert!(xml.contains("<Status>Suspended</Status>"));
    }

    #[tokio::test]
    async fn put_versioning_sends_request() {
        let (inner, requests) =
            create_test_inner_with_body(http::StatusCode::OK, bytes::Bytes::new());
        let builder = PutBucketVersioningBuilder::new(
            inner,
            BucketName::new("test-bucket").unwrap(),
            "Enabled".into(),
        );
        builder.send().await.unwrap();
        let captured = requests.lock().unwrap();
        assert_eq!(captured[0].method, http::Method::PUT);
        assert!(captured[0].uri.contains("?versioning"));
    }

    #[tokio::test]
    async fn get_versioning_parses_response() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<VersioningConfiguration>
  <Status>Enabled</Status>
</VersioningConfiguration>"#;
        let (inner, _) = create_test_inner_with_body(http::StatusCode::OK, bytes::Bytes::from(xml));
        let builder =
            GetBucketVersioningBuilder::new(inner, BucketName::new("test-bucket").unwrap());
        let output = builder.send().await.unwrap();
        assert_eq!(output.status, "Enabled");
    }

    #[tokio::test]
    #[ignore = "requires valid OSS credentials"]
    async fn e2e_bucket_versioning() {
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

        let bucket = client.bucket(&bucket_str).unwrap();
        bucket.put_versioning("Enabled").send().await.unwrap();
        let output = bucket.get_versioning().send().await.unwrap();
        assert!(!output.status.is_empty());
        eprintln!("GetBucketVersioning: status={}", output.status);
    }
}
