use crate::client::{BucketOperations, OSSClientInner};
use crate::error::{ErrorContext, OssError, OssErrorKind, Result};
use crate::http::client::HttpRequest;
use crate::types::bucket::BucketName;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize)]
#[serde(rename = "AccessMonitorConfiguration")]
struct AccessMonitorConfig {
    #[serde(rename = "Status")]
    status: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "AccessMonitorConfiguration")]
struct AccessMonitorConfigResp {
    #[serde(rename = "Status")]
    status: String,
}

pub struct PutBucketAccessMonitorBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    status: String,
}
impl PutBucketAccessMonitorBuilder {
    pub(crate) fn new(
        client: Arc<OSSClientInner>,
        bucket: BucketName,
        status: impl Into<String>,
    ) -> Self {
        Self {
            client,
            bucket,
            status: status.into(),
        }
    }
    pub async fn send(self) -> Result<PutBucketAccessMonitorOutput> {
        let ep = self.client.endpoint.clone();
        let uri = format!("https://{}.{}?accessMonitor", self.bucket.as_str(), ep);
        let qp = vec![("accessMonitor".into(), String::new())];
        let cfg = AccessMonitorConfig {
            status: self.status,
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
                    operation: Some("PutBucketAccessMonitor".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(ep),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;
        if r.status().is_success() {
            Ok(PutBucketAccessMonitorOutput {
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
                    operation: Some("PutBucketAccessMonitor".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}
#[derive(Debug, Clone)]
pub struct PutBucketAccessMonitorOutput {
    pub request_id: String,
}

pub struct GetBucketAccessMonitorBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
}
impl GetBucketAccessMonitorBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName) -> Self {
        Self { client, bucket }
    }
    pub async fn send(self) -> Result<GetBucketAccessMonitorOutput> {
        let ep = self.client.endpoint.clone();
        let uri = format!("https://{}.{}?accessMonitor", self.bucket.as_str(), ep);
        let qp = vec![("accessMonitor".into(), String::new())];
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
                    operation: Some("GetBucketAccessMonitor".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(ep),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;
        if r.is_success() {
            let c: AccessMonitorConfigResp =
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
            Ok(GetBucketAccessMonitorOutput { status: c.status })
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
                    operation: Some("GetBucketAccessMonitor".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}
#[derive(Debug, Clone)]
pub struct GetBucketAccessMonitorOutput {
    pub status: String,
}

impl BucketOperations {
    pub fn put_access_monitor(&self, status: impl Into<String>) -> PutBucketAccessMonitorBuilder {
        PutBucketAccessMonitorBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            status,
        )
    }
    pub fn get_access_monitor(&self) -> GetBucketAccessMonitorBuilder {
        GetBucketAccessMonitorBuilder::new(self.client_inner().clone(), self.bucket_name().clone())
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
    fn access_monitor_xml() {
        let c = AccessMonitorConfig {
            status: "Enabled".into(),
        };
        let x = crate::util::xml::to_xml(&c).unwrap();
        assert!(x.contains("<Status>Enabled</Status>"));
    }
    #[tokio::test]
    async fn put_sends_request() {
        let (i, r) = ci();
        PutBucketAccessMonitorBuilder::new(i, BucketName::new("test-bucket").unwrap(), "Enabled")
            .send()
            .await
            .unwrap();
        assert_eq!(r.lock().unwrap()[0].method, http::Method::PUT);
    }
}
