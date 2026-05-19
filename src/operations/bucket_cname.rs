//! CNAME domain binding operations.

use crate::client::{BucketOperations, OSSClientInner};
use crate::error::{ErrorContext, OssError, OssErrorKind, Result};
use crate::http::client::{HttpRequest, HttpResponse};
use crate::types::bucket::BucketName;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize)]
#[serde(rename = "CnameConfiguration")]
struct CnameConfig {
    #[serde(rename = "Domain")]
    domain: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "ListCnameResult")]
struct ListCnameResult {
    #[serde(rename = "Cname", default)]
    cnames: Vec<CnameInfo>,
}
#[derive(Debug, Clone, Deserialize)]
struct CnameInfo {
    #[serde(rename = "Domain")]
    domain: String,
    #[serde(rename = "Status", default)]
    status: String,
}

pub struct PutBucketCnameBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    domain: String,
}
impl PutBucketCnameBuilder {
    pub(crate) fn new(c: Arc<OSSClientInner>, b: BucketName, d: impl Into<String>) -> Self {
        Self {
            client: c,
            bucket: b,
            domain: d.into(),
        }
    }
    pub async fn send(self) -> Result<PutBucketCnameOutput> {
        put_cname_impl(self.client, self.bucket, self.domain).await
    }
}
#[derive(Debug, Clone)]
pub struct PutBucketCnameOutput {
    pub request_id: String,
}

pub struct ListBucketCnameBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
}
impl ListBucketCnameBuilder {
    pub(crate) fn new(c: Arc<OSSClientInner>, b: BucketName) -> Self {
        Self {
            client: c,
            bucket: b,
        }
    }
    pub async fn send(self) -> Result<ListBucketCnameOutput> {
        list_cname_impl(self.client, self.bucket).await
    }
}
#[derive(Debug, Clone)]
pub struct ListBucketCnameOutput {
    pub cnames: Vec<CnameEntry>,
}
#[derive(Debug, Clone)]
pub struct CnameEntry {
    pub domain: String,
    pub status: String,
}

pub struct DeleteBucketCnameBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    domain: String,
}
impl DeleteBucketCnameBuilder {
    pub(crate) fn new(c: Arc<OSSClientInner>, b: BucketName, d: impl Into<String>) -> Self {
        Self {
            client: c,
            bucket: b,
            domain: d.into(),
        }
    }
    pub async fn send(self) -> Result<DeleteBucketCnameOutput> {
        delete_cname_impl(self.client, self.bucket, self.domain).await
    }
}
#[derive(Debug, Clone)]
pub struct DeleteBucketCnameOutput {
    pub request_id: String,
}

async fn put_cname_impl(
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    domain: String,
) -> Result<PutBucketCnameOutput> {
    let ep = client.endpoint.clone();
    let uri = format!("https://{}.{}?cname", bucket.as_str(), ep);
    let qp = vec![("cname".into(), String::new())];
    let xml = crate::util::xml::to_xml(&CnameConfig { domain })?;
    let req = HttpRequest::builder()
        .method(http::Method::PUT)
        .uri(&uri)
        .body(bytes::Bytes::from(xml))
        .build();
    let r = client
        .send_signed(req, Some(&bucket), qp)
        .await
        .map_err(|e| err("PutBucketCname", &bucket, &ep, e))?;
    if r.status().is_success() {
        Ok(PutBucketCnameOutput {
            request_id: rid(&r),
        })
    } else {
        Err(service_err("PutBucketCname", &bucket, r))
    }
}

async fn list_cname_impl(
    client: Arc<OSSClientInner>,
    bucket: BucketName,
) -> Result<ListBucketCnameOutput> {
    let ep = client.endpoint.clone();
    let uri = format!("https://{}.{}?cname", bucket.as_str(), ep);
    let qp = vec![("cname".into(), String::new())];
    let req = HttpRequest::builder()
        .method(http::Method::GET)
        .uri(&uri)
        .build();
    let r = client
        .send_signed(req, Some(&bucket), qp)
        .await
        .map_err(|e| err("ListBucketCname", &bucket, &ep, e))?;
    if r.is_success() {
        let result: ListCnameResult = crate::util::xml::from_xml(r.body_as_str().unwrap_or(""))
            .map_err(|e| OssError {
                kind: OssErrorKind::DeserializationError,
                context: Box::new(ErrorContext {
                    operation: Some("parse XML".into()),
                    bucket: Some(bucket.to_string()),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;
        Ok(ListBucketCnameOutput {
            cnames: result
                .cnames
                .into_iter()
                .map(|c| CnameEntry {
                    domain: c.domain,
                    status: c.status,
                })
                .collect(),
        })
    } else {
        Err(service_err("ListBucketCname", &bucket, r))
    }
}

async fn delete_cname_impl(
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    domain: String,
) -> Result<DeleteBucketCnameOutput> {
    let ep = client.endpoint.clone();
    let uri = format!("https://{}.{}?cname&comp=delete", bucket.as_str(), ep);
    let qp = vec![
        ("cname".into(), String::new()),
        ("comp".into(), "delete".into()),
    ];
    let xml = crate::util::xml::to_xml(&CnameConfig { domain })?;
    let req = HttpRequest::builder()
        .method(http::Method::POST)
        .uri(&uri)
        .body(bytes::Bytes::from(xml))
        .build();
    let r = client
        .send_signed(req, Some(&bucket), qp)
        .await
        .map_err(|e| err("DeleteBucketCname", &bucket, &ep, e))?;
    if r.status().is_success() {
        Ok(DeleteBucketCnameOutput {
            request_id: rid(&r),
        })
    } else {
        Err(service_err("DeleteBucketCname", &bucket, r))
    }
}

fn rid(r: &HttpResponse) -> String {
    r.headers
        .get("x-oss-request-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string()
}
fn err(op: &str, b: &BucketName, ep: &str, e: OssError) -> OssError {
    OssError {
        kind: OssErrorKind::TransportError,
        context: Box::new(ErrorContext {
            operation: Some(op.into()),
            bucket: Some(b.to_string()),
            endpoint: Some(ep.into()),
            ..Default::default()
        }),
        source: Some(Box::new(e)),
    }
}
fn service_err(op: &str, b: &BucketName, r: HttpResponse) -> OssError {
    OssError {
        kind: OssErrorKind::ServiceError(Box::new(crate::error::OssServiceError {
            status_code: r.status().as_u16(),
            code: String::new(),
            message: String::new(),
            request_id: String::new(),
            host_id: String::new(),
            resource: Some(b.to_string()),
            string_to_sign: None,
        })),
        context: Box::new(ErrorContext {
            operation: Some(op.into()),
            bucket: Some(b.to_string()),
            ..Default::default()
        }),
        source: None,
    }
}

impl BucketOperations {
    pub fn put_cname(&self, domain: impl Into<String>) -> PutBucketCnameBuilder {
        PutBucketCnameBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            domain,
        )
    }
    pub fn list_cname(&self) -> ListBucketCnameBuilder {
        ListBucketCnameBuilder::new(self.client_inner().clone(), self.bucket_name().clone())
    }
    pub fn delete_cname(&self, domain: impl Into<String>) -> DeleteBucketCnameBuilder {
        DeleteBucketCnameBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            domain,
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
    fn cname_xml() {
        let c = CnameConfig {
            domain: "cdn.example.com".into(),
        };
        let x = crate::util::xml::to_xml(&c).unwrap();
        assert!(x.contains("<Domain>cdn.example.com</Domain>"));
    }
    #[tokio::test]
    async fn put_cname_sends_request() {
        let (i, r) = ci();
        PutBucketCnameBuilder::new(
            i,
            BucketName::new("test-bucket").unwrap(),
            "cdn.example.com",
        )
        .send()
        .await
        .unwrap();
        assert_eq!(r.lock().unwrap()[0].method, http::Method::PUT);
    }
}
