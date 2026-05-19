//! Resource group binding operations.

use crate::client::{BucketOperations, OSSClientInner};
use crate::error::{ErrorContext, OssError, OssErrorKind, Result};
use crate::http::client::HttpRequest;
use crate::types::bucket::BucketName;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize)]
#[serde(rename = "ResourceGroupConfiguration")]
struct ResourceGroupConfig {
    #[serde(rename = "ResourceGroupId")]
    resource_group_id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "ResourceGroupConfiguration")]
struct ResourceGroupConfigResp {
    #[serde(rename = "ResourceGroupId")]
    resource_group_id: String,
}

pub struct PutBucketResourceGroupBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    group_id: String,
}
impl PutBucketResourceGroupBuilder {
    pub(crate) fn new(
        client: Arc<OSSClientInner>,
        bucket: BucketName,
        group_id: impl Into<String>,
    ) -> Self {
        Self {
            client,
            bucket,
            group_id: group_id.into(),
        }
    }
    pub async fn send(self) -> Result<PutBucketResourceGroupOutput> {
        let ep = self.client.endpoint.clone();
        let uri = format!("https://{}.{}?resourceGroup", self.bucket.as_str(), ep);
        let qp = vec![("resourceGroup".into(), String::new())];
        let cfg = ResourceGroupConfig {
            resource_group_id: self.group_id,
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
                    operation: Some("PutBucketResourceGroup".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(ep),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;
        if r.status().is_success() {
            Ok(PutBucketResourceGroupOutput {
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
                    operation: Some("PutBucketResourceGroup".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}
#[derive(Debug, Clone)]
pub struct PutBucketResourceGroupOutput {
    pub request_id: String,
}

pub struct GetBucketResourceGroupBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
}
impl GetBucketResourceGroupBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName) -> Self {
        Self { client, bucket }
    }
    pub async fn send(self) -> Result<GetBucketResourceGroupOutput> {
        let ep = self.client.endpoint.clone();
        let uri = format!("https://{}.{}?resourceGroup", self.bucket.as_str(), ep);
        let qp = vec![("resourceGroup".into(), String::new())];
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
                    operation: Some("GetBucketResourceGroup".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(ep),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;
        if r.is_success() {
            let c: ResourceGroupConfigResp =
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
            Ok(GetBucketResourceGroupOutput {
                resource_group_id: c.resource_group_id,
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
                    operation: Some("GetBucketResourceGroup".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}
#[derive(Debug, Clone)]
pub struct GetBucketResourceGroupOutput {
    pub resource_group_id: String,
}

impl BucketOperations {
    pub fn put_resource_group(&self, group_id: impl Into<String>) -> PutBucketResourceGroupBuilder {
        PutBucketResourceGroupBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            group_id,
        )
    }
    pub fn get_resource_group(&self) -> GetBucketResourceGroupBuilder {
        GetBucketResourceGroupBuilder::new(self.client_inner().clone(), self.bucket_name().clone())
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
    fn resource_group_xml() {
        let c = ResourceGroupConfig {
            resource_group_id: "rg-xxx".into(),
        };
        let x = crate::util::xml::to_xml(&c).unwrap();
        assert!(x.contains("<ResourceGroupId>rg-xxx</ResourceGroupId>"));
    }
    #[tokio::test]
    async fn put_sends_request() {
        let (i, r) = ci();
        PutBucketResourceGroupBuilder::new(i, BucketName::new("test-bucket").unwrap(), "rg-1")
            .send()
            .await
            .unwrap();
        assert_eq!(r.lock().unwrap()[0].method, http::Method::PUT);
    }
}
