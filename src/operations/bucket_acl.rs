//! Bucket ACL (Access Control List) operations.

use std::sync::Arc;

use serde::Deserialize;

use crate::client::{BucketOperations, OSSClientInner};
use crate::error::{ErrorContext, OssError, OssErrorKind, Result};
use crate::http::client::HttpRequest;
use crate::types::acl::BucketAcl;
use crate::types::bucket::BucketName;
use crate::types::response::OwnerInfo;

pub struct PutBucketAclBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    acl: BucketAcl,
}

impl PutBucketAclBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName, acl: BucketAcl) -> Self {
        Self {
            client,
            bucket,
            acl,
        }
    }

    pub async fn send(self) -> Result<PutBucketAclOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!("https://{}.{}?acl", self.bucket.as_str(), endpoint);

        let query_params: Vec<(String, String)> = vec![("acl".into(), String::new())];

        let request = HttpRequest::builder()
            .method(http::Method::PUT)
            .uri(&uri)
            .header(
                http::HeaderName::from_static("x-oss-acl"),
                http::HeaderValue::from_str(self.acl.as_str()).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set x-oss-acl header".into()),
                        bucket: Some(self.bucket.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            )
            .build();

        let response = self
            .client
            .send_signed(request, Some(&self.bucket), query_params)
            .await
            .map_err(|e| OssError {
                kind: OssErrorKind::TransportError,
                context: Box::new(ErrorContext {
                    operation: Some("PutBucketAcl".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.status().is_success() {
            Ok(PutBucketAclOutput {
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
                    operation: Some("PutBucketAcl".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct PutBucketAclOutput {
    pub request_id: String,
}

pub struct GetBucketAclBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
}

impl GetBucketAclBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName) -> Self {
        Self { client, bucket }
    }

    pub async fn send(self) -> Result<GetBucketAclOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!("https://{}.{}?acl", self.bucket.as_str(), endpoint);

        let query_params: Vec<(String, String)> = vec![("acl".into(), String::new())];

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
                    operation: Some("GetBucketAcl".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.is_success() {
            let body_str = response.body_as_str().unwrap_or("");
            let policy: AccessControlPolicy =
                crate::util::xml::from_xml(body_str).map_err(|e| OssError {
                    kind: OssErrorKind::DeserializationError,
                    context: Box::new(ErrorContext {
                        operation: Some("GetBucketAcl: parse XML".into()),
                        bucket: Some(self.bucket.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?;

            Ok(GetBucketAclOutput {
                owner: policy.owner,
                grant: policy.acl.grant,
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
                    operation: Some("GetBucketAcl".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "AccessControlPolicy")]
struct AccessControlPolicy {
    #[serde(rename = "Owner")]
    owner: OwnerInfo,
    #[serde(rename = "AccessControlList")]
    acl: AclGrant,
}

#[derive(Debug, Clone, Deserialize)]
struct AclGrant {
    #[serde(rename = "Grant")]
    grant: String,
}

#[derive(Debug, Clone)]
pub struct GetBucketAclOutput {
    pub owner: OwnerInfo,
    pub grant: String,
}

impl BucketOperations {
    pub fn put_acl(&self, acl: BucketAcl) -> PutBucketAclBuilder {
        PutBucketAclBuilder::new(self.client_inner().clone(), self.bucket_name().clone(), acl)
    }

    pub fn get_acl(&self) -> GetBucketAclBuilder {
        GetBucketAclBuilder::new(self.client_inner().clone(), self.bucket_name().clone())
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
                http::HeaderValue::from_static("rid-acl"),
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

    #[tokio::test]
    async fn put_bucket_acl_sets_acl_header() {
        let (inner, requests) =
            create_test_inner_with_body(http::StatusCode::OK, bytes::Bytes::new());
        let builder = PutBucketAclBuilder::new(
            inner,
            BucketName::new("test-bucket").unwrap(),
            BucketAcl::PublicRead,
        );

        builder.send().await.unwrap();
        let captured = requests.lock().unwrap();
        assert_eq!(captured[0].method, http::Method::PUT);
        assert!(captured[0].uri.contains("?acl"));
    }

    #[tokio::test]
    async fn get_bucket_acl_parses_xml_response() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<AccessControlPolicy>
  <Owner><ID>owner-id</ID></Owner>
  <AccessControlList>
    <Grant>private</Grant>
  </AccessControlList>
</AccessControlPolicy>"#;
        let (inner, _) = create_test_inner_with_body(http::StatusCode::OK, bytes::Bytes::from(xml));
        let builder = GetBucketAclBuilder::new(inner, BucketName::new("test-bucket").unwrap());

        let output = builder.send().await.unwrap();
        assert_eq!(output.owner.id, "owner-id");
        assert_eq!(output.grant, "private");
    }

    #[tokio::test]
    #[ignore = "requires valid OSS credentials"]
    async fn e2e_bucket_acl() {
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

        let output = client
            .bucket(&bucket_str)
            .unwrap()
            .get_acl()
            .send()
            .await
            .unwrap();

        assert!(!output.grant.is_empty());
        eprintln!(
            "GetBucketAcl: grant={}, owner={}",
            output.grant, output.owner.id
        );
    }
}
