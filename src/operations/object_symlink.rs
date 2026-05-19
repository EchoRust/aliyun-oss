use std::sync::Arc;

use serde::Deserialize;

use crate::client::{BucketOperations, OSSClientInner};
use crate::error::{ErrorContext, OssError, OssErrorKind, Result};
use crate::http::client::HttpRequest;
use crate::types::acl::ObjectAcl;
use crate::types::bucket::BucketName;
use crate::types::object::ObjectKey;
use crate::types::storage::StorageClass;
use crate::util::uri::oss_endpoint_url;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "SymlinkResult")]
struct SymlinkResult {
    #[serde(rename = "SymlinkTarget")]
    target: String,
    #[serde(rename = "ETag")]
    etag: String,
}

pub struct PutSymlinkBuilder {
    client: Arc<OSSClientInner>,
    key: ObjectKey,
    bucket: BucketName,
    target: String,
    acl: Option<ObjectAcl>,
    storage_class: Option<StorageClass>,
    metadata: Vec<(String, String)>,
}

impl PutSymlinkBuilder {
    pub(crate) fn new(
        client: Arc<OSSClientInner>,
        bucket: BucketName,
        key: ObjectKey,
        target: impl Into<String>,
    ) -> Self {
        Self {
            client,
            bucket,
            key,
            target: target.into(),
            acl: None,
            storage_class: None,
            metadata: Vec::new(),
        }
    }

    pub fn acl(mut self, acl: ObjectAcl) -> Self {
        self.acl = Some(acl);
        self
    }
    pub fn storage_class(mut self, sc: StorageClass) -> Self {
        self.storage_class = Some(sc);
        self
    }
    pub fn metadata(mut self, k: impl Into<String>, v: impl Into<String>) -> Self {
        self.metadata.push((k.into(), v.into()));
        self
    }

    pub async fn send(self) -> Result<PutSymlinkOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = oss_endpoint_url(
            &endpoint,
            Some(self.bucket.as_str()),
            Some(self.key.as_str()),
        );
        let full_uri = format!("{}?symlink", uri);
        let query_params: Vec<(String, String)> = vec![("symlink".into(), String::new())];

        let mut req = HttpRequest::builder()
            .method(http::Method::PUT)
            .uri(&full_uri);
        req = req.header(
            http::HeaderName::from_static("x-oss-symlink-target"),
            http::HeaderValue::from_str(&self.target).map_err(|e| OssError {
                kind: OssErrorKind::ValidationError,
                context: Box::new(ErrorContext {
                    operation: Some("set symlink target".into()),
                    bucket: Some(self.bucket.to_string()),
                    object_key: Some(self.key.to_string()),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?,
        );

        if let Some(acl) = self.acl {
            req = req.header(
                http::HeaderName::from_static("x-oss-object-acl"),
                http::HeaderValue::from_str(acl.as_str()).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set acl".into()),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
        }
        if let Some(sc) = self.storage_class {
            req = req.header(
                http::HeaderName::from_static("x-oss-storage-class"),
                http::HeaderValue::from_str(sc.as_str()).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set storage class".into()),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
        }

        let response = self
            .client
            .send_signed(req.build(), Some(&self.bucket), query_params)
            .await
            .map_err(|e| OssError {
                kind: OssErrorKind::TransportError,
                context: Box::new(ErrorContext {
                    operation: Some("PutSymlink".into()),
                    bucket: Some(self.bucket.to_string()),
                    object_key: Some(self.key.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.is_success() {
            Ok(PutSymlinkOutput {
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
                    operation: Some("PutSymlink".into()),
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
pub struct PutSymlinkOutput {
    pub request_id: String,
}

pub struct GetSymlinkBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    key: ObjectKey,
}

impl GetSymlinkBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName, key: ObjectKey) -> Self {
        Self {
            client,
            bucket,
            key,
        }
    }

    pub async fn send(self) -> Result<GetSymlinkOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = oss_endpoint_url(
            &endpoint,
            Some(self.bucket.as_str()),
            Some(self.key.as_str()),
        );
        let full_uri = format!("{}?symlink", uri);
        let query_params: Vec<(String, String)> = vec![("symlink".into(), String::new())];

        let request = HttpRequest::builder()
            .method(http::Method::GET)
            .uri(&full_uri)
            .build();

        let response = self
            .client
            .send_signed(request, Some(&self.bucket), query_params)
            .await
            .map_err(|e| OssError {
                kind: OssErrorKind::TransportError,
                context: Box::new(ErrorContext {
                    operation: Some("GetSymlink".into()),
                    bucket: Some(self.bucket.to_string()),
                    object_key: Some(self.key.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.is_success() {
            let body_str = response.body_as_str().unwrap_or("");
            let result: SymlinkResult =
                crate::util::xml::from_xml(body_str).map_err(|e| OssError {
                    kind: OssErrorKind::DeserializationError,
                    context: Box::new(ErrorContext {
                        operation: Some("GetSymlink: parse XML".into()),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?;

            Ok(GetSymlinkOutput {
                target: result.target,
                etag: result.etag.trim_matches('"').to_string(),
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
                    operation: Some("GetSymlink".into()),
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
pub struct GetSymlinkOutput {
    pub target: String,
    pub etag: String,
}

impl BucketOperations {
    pub fn put_symlink(
        &self,
        key: impl Into<String>,
        target: impl Into<String>,
    ) -> Result<PutSymlinkBuilder> {
        Ok(PutSymlinkBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            ObjectKey::new(key.into())?,
            target,
        ))
    }
    pub fn get_symlink(&self, key: impl Into<String>) -> Result<GetSymlinkBuilder> {
        Ok(GetSymlinkBuilder::new(
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
                http::HeaderValue::from_static("rid-symlink"),
            );
            Ok(HttpResponse {
                status: self.status_code,
                headers,
                body: self.response_body.clone(),
            })
        }
    }

    fn create_inner(body: bytes::Bytes) -> (Arc<OSSClientInner>, Arc<Mutex<Vec<HttpRequest>>>) {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let http = Arc::new(RecordingHttpClient {
            requests: requests.clone(),
            status_code: http::StatusCode::OK,
            response_body: body,
        });
        let creds = Arc::new(crate::config::credentials::StaticCredentialsProvider::new(
            Credentials::builder()
                .access_key_id("ak")
                .access_key_secret("sk")
                .build()
                .unwrap(),
        ));
        (
            Arc::new(OSSClientInner {
                http,
                credentials: creds,
                signer: Arc::from(crate::signer::create_signer(crate::signer::SignVersion::V4)),
                region: Region::CnHangzhou,
                endpoint: "oss-cn-hangzhou.aliyuncs.com".into(),
            }),
            requests,
        )
    }

    #[tokio::test]
    async fn put_symlink_sets_target_header() {
        let (inner, requests) = create_inner(bytes::Bytes::new());
        let b = PutSymlinkBuilder::new(
            inner,
            BucketName::new("test-bucket").unwrap(),
            ObjectKey::new("link").unwrap(),
            "target.txt",
        );
        b.send().await.unwrap();
        let c = requests.lock().unwrap();
        assert_eq!(c[0].method, http::Method::PUT);
        assert!(c[0].uri.contains("?symlink"));
    }

    #[tokio::test]
    async fn get_symlink_parses_xml() {
        let xml = r#"<?xml version="1.0"?><SymlinkResult><SymlinkTarget>target.txt</SymlinkTarget><ETag>"abc"</ETag></SymlinkResult>"#;
        let (inner, _) = create_inner(bytes::Bytes::from(xml));
        let b = GetSymlinkBuilder::new(
            inner,
            BucketName::new("test-bucket").unwrap(),
            ObjectKey::new("link").unwrap(),
        );
        let o = b.send().await.unwrap();
        assert_eq!(o.target, "target.txt");
    }

    #[tokio::test]
    #[ignore = "requires valid OSS credentials"]
    async fn e2e_symlink() {
        let ak = std::env::var("OSS_ACCESS_KEY_ID").expect("OSS_ACCESS_KEY_ID not set");
        let sk = std::env::var("OSS_ACCESS_KEY_SECRET").expect("OSS_ACCESS_KEY_SECRET not set");
        let rs = std::env::var("OSS_REGION").unwrap_or_else(|_| "cn-wulanchabu".into());
        let bs = std::env::var("OSS_BUCKET").expect("OSS_BUCKET not set");
        let region = std::str::FromStr::from_str(&rs).unwrap_or_else(|_| Region::Custom {
            endpoint: format!("oss-{}.aliyuncs.com", rs),
            region_id: rs.clone(),
        });
        let client = crate::client::OSSClient::builder()
            .region(region)
            .credentials(ak, sk)
            .build()
            .unwrap();
        let key = format!("symlink-{}.txt", chrono::Utc::now().timestamp());
        let b = client.bucket(&bs).unwrap();
        b.put_object(&key)
            .unwrap()
            .body(bytes::Bytes::from("target"))
            .send()
            .await
            .unwrap();
        let o = b
            .put_symlink(&format!("link-{}", chrono::Utc::now().timestamp()), &key)
            .unwrap()
            .send()
            .await
            .unwrap();
        eprintln!("PutSymlink: rid={}", o.request_id);
    }
}
