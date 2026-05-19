use crate::client::{BucketOperations, OSSClientInner};
use crate::error::{ErrorContext, OssError, OssErrorKind, Result};
use crate::http::client::HttpRequest;
use crate::types::bucket::BucketName;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize)]
#[serde(rename = "TLSConfiguration")]
struct TlsConfig {
    #[serde(rename = "TLSVersion")]
    tls_version: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "TLSConfiguration")]
struct TlsConfigResp {
    #[serde(rename = "TLSVersion", default)]
    tls_version: Vec<String>,
}

pub struct PutBucketTlsBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    versions: Vec<String>,
}
impl PutBucketTlsBuilder {
    pub(crate) fn new(
        client: Arc<OSSClientInner>,
        bucket: BucketName,
        versions: Vec<String>,
    ) -> Self {
        Self {
            client,
            bucket,
            versions,
        }
    }
    pub async fn send(self) -> Result<PutBucketTlsOutput> {
        let ep = self.client.endpoint.clone();
        let uri = format!("https://{}.{}?tls", self.bucket.as_str(), ep);
        let qp = vec![("tls".into(), String::new())];
        let cfg = TlsConfig {
            tls_version: self.versions,
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
                    operation: Some("PutBucketTls".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(ep),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;
        if r.status().is_success() {
            Ok(PutBucketTlsOutput {
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
                    operation: Some("PutBucketTls".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}
#[derive(Debug, Clone)]
pub struct PutBucketTlsOutput {
    pub request_id: String,
}

pub struct GetBucketTlsBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
}
impl GetBucketTlsBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName) -> Self {
        Self { client, bucket }
    }
    pub async fn send(self) -> Result<GetBucketTlsOutput> {
        let ep = self.client.endpoint.clone();
        let uri = format!("https://{}.{}?tls", self.bucket.as_str(), ep);
        let qp = vec![("tls".into(), String::new())];
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
                    operation: Some("GetBucketTls".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(ep),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;
        if r.is_success() {
            let c: TlsConfigResp = crate::util::xml::from_xml(r.body_as_str().unwrap_or(""))
                .map_err(|e| OssError {
                    kind: OssErrorKind::DeserializationError,
                    context: Box::new(ErrorContext {
                        operation: Some("parse XML".into()),
                        bucket: Some(self.bucket.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?;
            Ok(GetBucketTlsOutput {
                tls_version: c.tls_version,
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
                    operation: Some("GetBucketTls".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}
#[derive(Debug, Clone)]
pub struct GetBucketTlsOutput {
    pub tls_version: Vec<String>,
}

impl BucketOperations {
    pub fn put_tls(&self, versions: Vec<String>) -> PutBucketTlsBuilder {
        PutBucketTlsBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            versions,
        )
    }
    pub fn get_tls(&self) -> GetBucketTlsBuilder {
        GetBucketTlsBuilder::new(self.client_inner().clone(), self.bucket_name().clone())
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
    fn tls_xml_generation() {
        let c = TlsConfig {
            tls_version: vec!["TLSv1.2".into(), "TLSv1.3".into()],
        };
        let x = crate::util::xml::to_xml(&c).unwrap();
        assert!(x.contains("<TLSVersion>TLSv1.2</TLSVersion>"));
    }
    #[tokio::test]
    async fn put_sends_request() {
        let (i, r) = ci();
        PutBucketTlsBuilder::new(
            i,
            BucketName::new("test-bucket").unwrap(),
            vec!["TLSv1.2".into()],
        )
        .send()
        .await
        .unwrap();
        assert_eq!(r.lock().unwrap()[0].method, http::Method::PUT);
    }
}
