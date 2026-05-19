//! Object tagging operations.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::client::{BucketOperations, OSSClientInner};
use crate::error::{ErrorContext, OssError, OssErrorKind, Result};
use crate::http::client::HttpRequest;
use crate::types::bucket::BucketName;
use crate::types::object::ObjectKey;
use crate::util::uri::oss_endpoint_url;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "Tagging")]
struct TaggingConfig {
    #[serde(rename = "TagSet")]
    tag_set: TagSetConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TagSetConfig {
    #[serde(rename = "Tag")]
    tags: Vec<TagPair>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TagPair {
    #[serde(rename = "Key")]
    key: String,
    #[serde(rename = "Value")]
    value: String,
}

pub struct PutObjectTaggingBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    key: ObjectKey,
    tags: Vec<(String, String)>,
}

impl PutObjectTaggingBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName, key: ObjectKey) -> Self {
        Self {
            client,
            bucket,
            key,
            tags: Vec::new(),
        }
    }

    pub fn tag(mut self, k: impl Into<String>, v: impl Into<String>) -> Self {
        self.tags.push((k.into(), v.into()));
        self
    }

    pub async fn send(self) -> Result<PutObjectTaggingOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = oss_endpoint_url(
            &endpoint,
            Some(self.bucket.as_str()),
            Some(self.key.as_str()),
        );
        let full_uri = format!("{}?tagging", uri);
        let qp: Vec<(String, String)> = vec![("tagging".into(), String::new())];

        let config = TaggingConfig {
            tag_set: TagSetConfig {
                tags: self
                    .tags
                    .into_iter()
                    .map(|(k, v)| TagPair { key: k, value: v })
                    .collect(),
            },
        };
        let body_xml = crate::util::xml::to_xml(&config)?;

        let request = HttpRequest::builder()
            .method(http::Method::PUT)
            .uri(&full_uri)
            .body(bytes::Bytes::from(body_xml))
            .build();
        let response = self
            .client
            .send_signed(request, Some(&self.bucket), qp)
            .await
            .map_err(|e| OssError {
                kind: OssErrorKind::TransportError,
                context: Box::new(ErrorContext {
                    operation: Some("PutObjectTagging".into()),
                    bucket: Some(self.bucket.to_string()),
                    object_key: Some(self.key.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.is_success() {
            Ok(PutObjectTaggingOutput {
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
                    resource: Some(self.key.to_string()),
                    string_to_sign: None,
                })),
                context: Box::new(ErrorContext {
                    operation: Some("PutObjectTagging".into()),
                    bucket: Some(self.bucket.to_string()),
                    object_key: Some(self.key.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct PutObjectTaggingOutput {
    pub request_id: String,
}

pub struct GetObjectTaggingBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    key: ObjectKey,
}

impl GetObjectTaggingBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName, key: ObjectKey) -> Self {
        Self {
            client,
            bucket,
            key,
        }
    }

    pub async fn send(self) -> Result<GetObjectTaggingOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = oss_endpoint_url(
            &endpoint,
            Some(self.bucket.as_str()),
            Some(self.key.as_str()),
        );
        let full_uri = format!("{}?tagging", uri);
        let qp: Vec<(String, String)> = vec![("tagging".into(), String::new())];

        let request = HttpRequest::builder()
            .method(http::Method::GET)
            .uri(&full_uri)
            .build();
        let response = self
            .client
            .send_signed(request, Some(&self.bucket), qp)
            .await
            .map_err(|e| OssError {
                kind: OssErrorKind::TransportError,
                context: Box::new(ErrorContext {
                    operation: Some("GetObjectTagging".into()),
                    bucket: Some(self.bucket.to_string()),
                    object_key: Some(self.key.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.is_success() {
            let body_str = response.body_as_str().unwrap_or("");
            let config: TaggingConfig =
                crate::util::xml::from_xml(body_str).map_err(|e| OssError {
                    kind: OssErrorKind::DeserializationError,
                    context: Box::new(ErrorContext {
                        operation: Some("GetObjectTagging: parse XML".into()),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?;
            Ok(GetObjectTaggingOutput {
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
                    resource: Some(self.key.to_string()),
                    string_to_sign: None,
                })),
                context: Box::new(ErrorContext {
                    operation: Some("GetObjectTagging".into()),
                    bucket: Some(self.bucket.to_string()),
                    object_key: Some(self.key.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct GetObjectTaggingOutput {
    pub tags: Vec<(String, String)>,
}

pub struct DeleteObjectTaggingBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    key: ObjectKey,
}

impl DeleteObjectTaggingBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName, key: ObjectKey) -> Self {
        Self {
            client,
            bucket,
            key,
        }
    }
    pub async fn send(self) -> Result<DeleteObjectTaggingOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = oss_endpoint_url(
            &endpoint,
            Some(self.bucket.as_str()),
            Some(self.key.as_str()),
        );
        let full_uri = format!("{}?tagging", uri);
        let qp: Vec<(String, String)> = vec![("tagging".into(), String::new())];

        let request = HttpRequest::builder()
            .method(http::Method::DELETE)
            .uri(&full_uri)
            .build();
        let response = self
            .client
            .send_signed(request, Some(&self.bucket), qp)
            .await
            .map_err(|e| OssError {
                kind: OssErrorKind::TransportError,
                context: Box::new(ErrorContext {
                    operation: Some("DeleteObjectTagging".into()),
                    bucket: Some(self.bucket.to_string()),
                    object_key: Some(self.key.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.status().is_success() {
            Ok(DeleteObjectTaggingOutput {
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
                    resource: Some(self.key.to_string()),
                    string_to_sign: None,
                })),
                context: Box::new(ErrorContext {
                    operation: Some("DeleteObjectTagging".into()),
                    bucket: Some(self.bucket.to_string()),
                    object_key: Some(self.key.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeleteObjectTaggingOutput {
    pub request_id: String,
}

impl BucketOperations {
    pub fn put_object_tagging(&self, key: impl Into<String>) -> Result<PutObjectTaggingBuilder> {
        Ok(PutObjectTaggingBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            ObjectKey::new(key.into())?,
        ))
    }
    pub fn get_object_tagging(&self, key: impl Into<String>) -> Result<GetObjectTaggingBuilder> {
        Ok(GetObjectTaggingBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            ObjectKey::new(key.into())?,
        ))
    }
    pub fn delete_object_tagging(
        &self,
        key: impl Into<String>,
    ) -> Result<DeleteObjectTaggingBuilder> {
        Ok(DeleteObjectTaggingBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            ObjectKey::new(key.into())?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::OSSClientInner;
    use crate::config::credentials::Credentials;
    use crate::http::client::{HttpClient, HttpRequest, HttpResponse};
    use crate::types::region::Region;
    use std::sync::Mutex;

    struct Rc {
        r: Arc<Mutex<Vec<HttpRequest>>>,
    }
    #[async_trait::async_trait]
    impl HttpClient for Rc {
        async fn send(&self, req: HttpRequest) -> crate::error::Result<HttpResponse> {
            self.r.lock().unwrap().push(req);
            let mut h = http::HeaderMap::new();
            h.insert("x-oss-request-id", http::HeaderValue::from_static("rid"));
            Ok(HttpResponse {
                status: http::StatusCode::OK,
                headers: h,
                body: bytes::Bytes::new(),
            })
        }
    }
    fn ci() -> (Arc<OSSClientInner>, Arc<Mutex<Vec<HttpRequest>>>) {
        let rq = Arc::new(Mutex::new(Vec::new()));
        let h = Arc::new(Rc { r: rq.clone() });
        let cr = Arc::new(crate::config::credentials::StaticCredentialsProvider::new(
            Credentials::builder()
                .access_key_id("ak")
                .access_key_secret("sk")
                .build()
                .unwrap(),
        ));
        (
            Arc::new(OSSClientInner {
                http: h,
                credentials: cr,
                signer: Arc::from(crate::signer::create_signer(crate::signer::SignVersion::V4)),
                region: Region::CnHangzhou,
                endpoint: "oss-cn-hangzhou.aliyuncs.com".into(),
            }),
            rq,
        )
    }

    #[test]
    fn tagging_xml() {
        let c = TaggingConfig {
            tag_set: TagSetConfig {
                tags: vec![TagPair {
                    key: "env".into(),
                    value: "prod".into(),
                }],
            },
        };
        let xml = crate::util::xml::to_xml(&c).unwrap();
        assert!(xml.contains("<Key>env</Key>"));
        assert!(xml.contains("<Value>prod</Value>"));
    }

    #[tokio::test]
    async fn delete_tagging_sends_delete() {
        let (i, r) = ci();
        DeleteObjectTaggingBuilder::new(
            i,
            BucketName::new("test-bucket").unwrap(),
            ObjectKey::new("k").unwrap(),
        )
        .send()
        .await
        .unwrap();
        assert_eq!(r.lock().unwrap()[0].method, http::Method::DELETE);
    }
}
