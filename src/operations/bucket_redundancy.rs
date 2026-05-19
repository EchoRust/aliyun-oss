use crate::client::{BucketOperations, OSSClientInner};
use crate::error::{ErrorContext, OssError, OssErrorKind, Result};
use crate::http::client::HttpRequest;
use crate::types::bucket::BucketName;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize)]
#[serde(rename = "RedundancyTransitionConfiguration")]
struct RedundancyTransitionConfig {
    #[serde(rename = "TaskId")]
    task_id: String,
    #[serde(rename = "Status", skip_serializing_if = "Option::is_none")]
    status: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "RedundancyTransitionConfiguration")]
struct RedundancyTransitionConfigResp {
    #[serde(rename = "TaskId")]
    task_id: String,
    #[serde(rename = "Status", default)]
    status: String,
}

pub struct PutBucketRedundancyBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    task_id: String,
}
impl PutBucketRedundancyBuilder {
    pub(crate) fn new(
        client: Arc<OSSClientInner>,
        bucket: BucketName,
        task_id: impl Into<String>,
    ) -> Self {
        Self {
            client,
            bucket,
            task_id: task_id.into(),
        }
    }
    pub async fn send(self) -> Result<PutBucketRedundancyOutput> {
        let ep = self.client.endpoint.clone();
        let uri = format!(
            "https://{}.{}?redundancyTransition",
            self.bucket.as_str(),
            ep
        );
        let qp = vec![("redundancyTransition".into(), String::new())];
        let cfg = RedundancyTransitionConfig {
            task_id: self.task_id,
            status: None,
        };
        let xml = crate::util::xml::to_xml(&cfg)?;
        let req = HttpRequest::builder()
            .method(http::Method::PUT)
            .uri(&uri)
            .body(bytes::Bytes::from(xml))
            .build();
        let r = self
            .client
            .send_signed(req, Some(&self.bucket), qp)
            .await
            .map_err(|e| OssError {
                kind: OssErrorKind::TransportError,
                context: Box::new(ErrorContext {
                    operation: Some("PutBucketRedundancy".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(ep),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;
        if r.status().is_success() {
            Ok(PutBucketRedundancyOutput {
                request_id: r
                    .headers
                    .get("x-oss-request-id")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("")
                    .to_string(),
            })
        } else {
            Err(OssError {
                kind: OssErrorKind::ServiceError(Box::new(crate::error::OssServiceError {
                    status_code: r.status().as_u16(),
                    code: String::new(),
                    message: String::new(),
                    request_id: String::new(),
                    host_id: String::new(),
                    resource: Some(self.bucket.to_string()),
                    string_to_sign: None,
                })),
                context: Box::new(ErrorContext {
                    operation: Some("PutBucketRedundancy".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}
#[derive(Debug, Clone)]
pub struct PutBucketRedundancyOutput {
    pub request_id: String,
}

pub struct GetBucketRedundancyBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
}
impl GetBucketRedundancyBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName) -> Self {
        Self { client, bucket }
    }
    pub async fn send(self) -> Result<GetBucketRedundancyOutput> {
        let ep = self.client.endpoint.clone();
        let uri = format!(
            "https://{}.{}?redundancyTransition",
            self.bucket.as_str(),
            ep
        );
        let qp = vec![("redundancyTransition".into(), String::new())];
        let req = HttpRequest::builder()
            .method(http::Method::GET)
            .uri(&uri)
            .build();
        let r = self
            .client
            .send_signed(req, Some(&self.bucket), qp)
            .await
            .map_err(|e| OssError {
                kind: OssErrorKind::TransportError,
                context: Box::new(ErrorContext {
                    operation: Some("GetBucketRedundancy".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(ep),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;
        if r.is_success() {
            let c: RedundancyTransitionConfigResp =
                crate::util::xml::from_xml(r.body_as_str().unwrap_or("")).map_err(|e| {
                    OssError {
                        kind: OssErrorKind::DeserializationError,
                        context: Box::new(ErrorContext {
                            operation: Some("parse XML".into()),
                            bucket: Some(self.bucket.to_string()),
                            ..Default::default()
                        }),
                        source: Some(Box::new(e)),
                    }
                })?;
            Ok(GetBucketRedundancyOutput {
                task_id: c.task_id,
                status: c.status,
            })
        } else {
            Err(OssError {
                kind: OssErrorKind::ServiceError(Box::new(crate::error::OssServiceError {
                    status_code: r.status().as_u16(),
                    code: String::new(),
                    message: String::new(),
                    request_id: String::new(),
                    host_id: String::new(),
                    resource: Some(self.bucket.to_string()),
                    string_to_sign: None,
                })),
                context: Box::new(ErrorContext {
                    operation: Some("GetBucketRedundancy".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}
#[derive(Debug, Clone)]
pub struct GetBucketRedundancyOutput {
    pub task_id: String,
    pub status: String,
}

pub struct DeleteBucketRedundancyBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    task_id: String,
}
impl DeleteBucketRedundancyBuilder {
    pub(crate) fn new(
        client: Arc<OSSClientInner>,
        bucket: BucketName,
        task_id: impl Into<String>,
    ) -> Self {
        Self {
            client,
            bucket,
            task_id: task_id.into(),
        }
    }
    pub async fn send(self) -> Result<DeleteBucketRedundancyOutput> {
        let ep = self.client.endpoint.clone();
        let uri = format!(
            "https://{}.{}?redundancyTransition&taskId={}",
            self.bucket.as_str(),
            ep,
            self.task_id
        );
        let qp = vec![
            ("redundancyTransition".into(), String::new()),
            ("taskId".into(), self.task_id),
        ];
        let req = HttpRequest::builder()
            .method(http::Method::DELETE)
            .uri(&uri)
            .build();
        let r = self
            .client
            .send_signed(req, Some(&self.bucket), qp)
            .await
            .map_err(|e| OssError {
                kind: OssErrorKind::TransportError,
                context: Box::new(ErrorContext {
                    operation: Some("DeleteBucketRedundancy".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(ep),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;
        if r.status().is_success() {
            Ok(DeleteBucketRedundancyOutput {
                request_id: r
                    .headers
                    .get("x-oss-request-id")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("")
                    .to_string(),
            })
        } else {
            Err(OssError {
                kind: OssErrorKind::ServiceError(Box::new(crate::error::OssServiceError {
                    status_code: r.status().as_u16(),
                    code: String::new(),
                    message: String::new(),
                    request_id: String::new(),
                    host_id: String::new(),
                    resource: Some(self.bucket.to_string()),
                    string_to_sign: None,
                })),
                context: Box::new(ErrorContext {
                    operation: Some("DeleteBucketRedundancy".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}
#[derive(Debug, Clone)]
pub struct DeleteBucketRedundancyOutput {
    pub request_id: String,
}

impl BucketOperations {
    pub fn put_redundancy_transition(
        &self,
        task_id: impl Into<String>,
    ) -> PutBucketRedundancyBuilder {
        PutBucketRedundancyBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            task_id,
        )
    }
    pub fn get_redundancy_transition(&self) -> GetBucketRedundancyBuilder {
        GetBucketRedundancyBuilder::new(self.client_inner().clone(), self.bucket_name().clone())
    }
    pub fn delete_redundancy_transition(
        &self,
        task_id: impl Into<String>,
    ) -> DeleteBucketRedundancyBuilder {
        DeleteBucketRedundancyBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            task_id,
        )
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
    fn redundancy_xml() {
        let c = RedundancyTransitionConfig {
            task_id: "task-xxx".into(),
            status: None,
        };
        let x = crate::util::xml::to_xml(&c).unwrap();
        assert!(x.contains("<TaskId>task-xxx</TaskId>"));
    }
    #[tokio::test]
    async fn delete_sends_request() {
        let (i, r) = ci();
        DeleteBucketRedundancyBuilder::new(i, BucketName::new("test-bucket").unwrap(), "task-1")
            .send()
            .await
            .unwrap();
        assert_eq!(r.lock().unwrap()[0].method, http::Method::DELETE);
    }
}
