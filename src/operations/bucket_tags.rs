//! Bucket tagging operations.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::client::{BucketOperations, OSSClientInner};
use crate::error::{ErrorContext, OssError, OssErrorKind, Result};
use crate::http::client::HttpRequest;
use crate::types::bucket::BucketName;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "Tagging")]
struct TaggingConfiguration {
    #[serde(rename = "TagSet")]
    tag_set: TagSet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TagSet {
    #[serde(rename = "Tag")]
    tags: Vec<TagEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TagEntry {
    #[serde(rename = "Key")]
    key: String,
    #[serde(rename = "Value")]
    value: String,
}

pub struct PutBucketTagsBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    tags: Vec<(String, String)>,
}

impl PutBucketTagsBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName) -> Self {
        Self {
            client,
            bucket,
            tags: Vec::new(),
        }
    }

    pub fn tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.tags.push((key.into(), value.into()));
        self
    }

    pub async fn send(self) -> Result<PutBucketTagsOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!("https://{}.{}?tagging", self.bucket.as_str(), endpoint);
        let query_params: Vec<(String, String)> = vec![("tagging".into(), String::new())];

        let config = TaggingConfiguration {
            tag_set: TagSet {
                tags: self
                    .tags
                    .into_iter()
                    .map(|(k, v)| TagEntry { key: k, value: v })
                    .collect(),
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
                    operation: Some("PutBucketTags".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.status().is_success() {
            Ok(PutBucketTagsOutput {
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
                    operation: Some("PutBucketTags".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct PutBucketTagsOutput {
    pub request_id: String,
}

pub struct GetBucketTagsBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
}

impl GetBucketTagsBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName) -> Self {
        Self { client, bucket }
    }

    pub async fn send(self) -> Result<GetBucketTagsOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!("https://{}.{}?tagging", self.bucket.as_str(), endpoint);
        let query_params: Vec<(String, String)> = vec![("tagging".into(), String::new())];

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
                    operation: Some("GetBucketTags".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.is_success() {
            let body_str = response.body_as_str().unwrap_or("");
            let config: TaggingConfiguration =
                crate::util::xml::from_xml(body_str).map_err(|e| OssError {
                    kind: OssErrorKind::DeserializationError,
                    context: Box::new(ErrorContext {
                        operation: Some("GetBucketTags: parse XML".into()),
                        bucket: Some(self.bucket.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?;

            Ok(GetBucketTagsOutput {
                tags: config
                    .tag_set
                    .tags
                    .into_iter()
                    .map(|t| (t.key, t.value))
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
                    operation: Some("GetBucketTags".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct GetBucketTagsOutput {
    pub tags: Vec<(String, String)>,
}

pub struct DeleteBucketTagsBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
}

impl DeleteBucketTagsBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName) -> Self {
        Self { client, bucket }
    }

    pub async fn send(self) -> Result<DeleteBucketTagsOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!("https://{}.{}?tagging", self.bucket.as_str(), endpoint);
        let query_params: Vec<(String, String)> = vec![("tagging".into(), String::new())];

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
                    operation: Some("DeleteBucketTags".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.status().is_success() {
            Ok(DeleteBucketTagsOutput {
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
                    operation: Some("DeleteBucketTags".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeleteBucketTagsOutput {
    pub request_id: String,
}

impl BucketOperations {
    pub fn put_tags(&self) -> PutBucketTagsBuilder {
        PutBucketTagsBuilder::new(self.client_inner().clone(), self.bucket_name().clone())
    }

    pub fn get_tags(&self) -> GetBucketTagsBuilder {
        GetBucketTagsBuilder::new(self.client_inner().clone(), self.bucket_name().clone())
    }

    pub fn delete_tags(&self) -> DeleteBucketTagsBuilder {
        DeleteBucketTagsBuilder::new(self.client_inner().clone(), self.bucket_name().clone())
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
                http::HeaderValue::from_static("rid-tags"),
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
    fn tagging_xml_generation() {
        let config = TaggingConfiguration {
            tag_set: TagSet {
                tags: vec![
                    TagEntry {
                        key: "env".into(),
                        value: "prod".into(),
                    },
                    TagEntry {
                        key: "team".into(),
                        value: "backend".into(),
                    },
                ],
            },
        };
        let xml = crate::util::xml::to_xml(&config).unwrap();
        assert!(xml.contains("<Key>env</Key>"));
        assert!(xml.contains("<Value>prod</Value>"));
    }

    #[tokio::test]
    async fn delete_tags_sends_delete_request() {
        let (inner, requests) =
            create_test_inner_with_body(http::StatusCode::NO_CONTENT, bytes::Bytes::new());
        let builder = DeleteBucketTagsBuilder::new(inner, BucketName::new("test-bucket").unwrap());
        builder.send().await.unwrap();
        let captured = requests.lock().unwrap();
        assert_eq!(captured[0].method, http::Method::DELETE);
    }
}
