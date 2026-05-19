//! Bucket lifecycle rule operations.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::client::{BucketOperations, OSSClientInner};
use crate::error::{ErrorContext, OssError, OssErrorKind, Result};
use crate::http::client::HttpRequest;
use crate::types::bucket::BucketName;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "LifecycleConfiguration")]
struct LifecycleConfiguration {
    #[serde(rename = "Rule")]
    rules: Vec<LifecycleRuleData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LifecycleRuleData {
    #[serde(rename = "ID", skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(rename = "Prefix", skip_serializing_if = "Option::is_none")]
    prefix: Option<String>,
    #[serde(rename = "Status")]
    status: String,
    #[serde(rename = "Expiration", skip_serializing_if = "Option::is_none")]
    expiration: Option<LifecycleExpiration>,
    #[serde(
        rename = "AbortMultipartUpload",
        skip_serializing_if = "Option::is_none"
    )]
    abort_multipart_upload: Option<LifecycleAbortMultipartUpload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LifecycleExpiration {
    #[serde(rename = "Days", skip_serializing_if = "Option::is_none")]
    days: Option<i32>,
    #[serde(rename = "CreatedBeforeDate", skip_serializing_if = "Option::is_none")]
    created_before_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LifecycleAbortMultipartUpload {
    #[serde(rename = "Days", skip_serializing_if = "Option::is_none")]
    days: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct LifecycleRule {
    pub id: Option<String>,
    pub prefix: Option<String>,
    pub status: LifecycleRuleStatus,
    pub expiration_days: Option<i32>,
    pub expiration_date: Option<String>,
    pub abort_multipart_upload_days: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleRuleStatus {
    Enabled,
    Disabled,
}

impl LifecycleRuleStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Enabled => "Enabled",
            Self::Disabled => "Disabled",
        }
    }
}

pub struct PutBucketLifecycleBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    rules: Vec<LifecycleRule>,
}

impl PutBucketLifecycleBuilder {
    pub(crate) fn new(
        client: Arc<OSSClientInner>,
        bucket: BucketName,
        rules: Vec<LifecycleRule>,
    ) -> Self {
        Self {
            client,
            bucket,
            rules,
        }
    }

    pub async fn send(self) -> Result<PutBucketLifecycleOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!("https://{}.{}?lifecycle", self.bucket.as_str(), endpoint);

        let query_params: Vec<(String, String)> = vec![("lifecycle".into(), String::new())];

        let config = LifecycleConfiguration {
            rules: self
                .rules
                .into_iter()
                .map(|r| LifecycleRuleData {
                    id: r.id,
                    prefix: r.prefix,
                    status: r.status.as_str().to_string(),
                    expiration: if r.expiration_days.is_some() || r.expiration_date.is_some() {
                        Some(LifecycleExpiration {
                            days: r.expiration_days,
                            created_before_date: r.expiration_date,
                        })
                    } else {
                        None
                    },
                    abort_multipart_upload: r
                        .abort_multipart_upload_days
                        .map(|d| LifecycleAbortMultipartUpload { days: Some(d) }),
                })
                .collect(),
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
                    operation: Some("PutBucketLifecycle".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.status().is_success() {
            Ok(PutBucketLifecycleOutput {
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
                    operation: Some("PutBucketLifecycle".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct PutBucketLifecycleOutput {
    pub request_id: String,
}

pub struct GetBucketLifecycleBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
}

impl GetBucketLifecycleBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName) -> Self {
        Self { client, bucket }
    }

    pub async fn send(self) -> Result<GetBucketLifecycleOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!("https://{}.{}?lifecycle", self.bucket.as_str(), endpoint);

        let query_params: Vec<(String, String)> = vec![("lifecycle".into(), String::new())];

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
                    operation: Some("GetBucketLifecycle".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.is_success() {
            let body_str = response.body_as_str().unwrap_or("");
            let config: LifecycleConfiguration =
                crate::util::xml::from_xml(body_str).map_err(|e| OssError {
                    kind: OssErrorKind::DeserializationError,
                    context: Box::new(ErrorContext {
                        operation: Some("GetBucketLifecycle: parse XML".into()),
                        bucket: Some(self.bucket.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?;

            Ok(GetBucketLifecycleOutput {
                rules: config
                    .rules
                    .into_iter()
                    .map(|r| LifecycleRule {
                        id: r.id,
                        prefix: r.prefix,
                        status: if r.status == "Enabled" {
                            LifecycleRuleStatus::Enabled
                        } else {
                            LifecycleRuleStatus::Disabled
                        },
                        expiration_days: r.expiration.as_ref().and_then(|e| e.days),
                        expiration_date: r.expiration.and_then(|e| e.created_before_date),
                        abort_multipart_upload_days: r.abort_multipart_upload.and_then(|a| a.days),
                    })
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
                    operation: Some("GetBucketLifecycle".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct GetBucketLifecycleOutput {
    pub rules: Vec<LifecycleRule>,
}

pub struct DeleteBucketLifecycleBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
}

impl DeleteBucketLifecycleBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName) -> Self {
        Self { client, bucket }
    }

    pub async fn send(self) -> Result<DeleteBucketLifecycleOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!("https://{}.{}?lifecycle", self.bucket.as_str(), endpoint);

        let query_params: Vec<(String, String)> = vec![("lifecycle".into(), String::new())];

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
                    operation: Some("DeleteBucketLifecycle".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.status().is_success() {
            Ok(DeleteBucketLifecycleOutput {
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
                    operation: Some("DeleteBucketLifecycle".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeleteBucketLifecycleOutput {
    pub request_id: String,
}

impl BucketOperations {
    pub fn put_lifecycle(&self, rules: Vec<LifecycleRule>) -> PutBucketLifecycleBuilder {
        PutBucketLifecycleBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            rules,
        )
    }

    pub fn get_lifecycle(&self) -> GetBucketLifecycleBuilder {
        GetBucketLifecycleBuilder::new(self.client_inner().clone(), self.bucket_name().clone())
    }

    pub fn delete_lifecycle(&self) -> DeleteBucketLifecycleBuilder {
        DeleteBucketLifecycleBuilder::new(self.client_inner().clone(), self.bucket_name().clone())
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
                http::HeaderValue::from_static("rid-lifecycle"),
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
    fn lifecycle_xml_generation_contains_rule() {
        let config = LifecycleConfiguration {
            rules: vec![LifecycleRuleData {
                id: Some("rule1".into()),
                prefix: Some("logs/".into()),
                status: "Enabled".into(),
                expiration: Some(LifecycleExpiration {
                    days: Some(30),
                    created_before_date: None,
                }),
                abort_multipart_upload: None,
            }],
        };
        let xml = crate::util::xml::to_xml(&config).unwrap();
        assert!(xml.contains("<ID>rule1</ID>"));
        assert!(xml.contains("<Prefix>logs/</Prefix>"));
        assert!(xml.contains("<Days>30</Days>"));
    }

    #[tokio::test]
    async fn get_bucket_lifecycle_parses_rules_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<LifecycleConfiguration>
  <Rule>
    <ID>exp-rule</ID>
    <Prefix>logs/</Prefix>
    <Status>Enabled</Status>
    <Expiration><Days>30</Days></Expiration>
  </Rule>
</LifecycleConfiguration>"#;
        let (inner, _) = create_test_inner_with_body(http::StatusCode::OK, bytes::Bytes::from(xml));
        let builder =
            GetBucketLifecycleBuilder::new(inner, BucketName::new("test-bucket").unwrap());
        let output = builder.send().await.unwrap();
        assert_eq!(output.rules.len(), 1);
        assert_eq!(output.rules[0].expiration_days, Some(30));
    }

    #[tokio::test]
    async fn delete_bucket_lifecycle_sends_delete_request() {
        let (inner, requests) =
            create_test_inner_with_body(http::StatusCode::NO_CONTENT, bytes::Bytes::new());
        let builder =
            DeleteBucketLifecycleBuilder::new(inner, BucketName::new("test-bucket").unwrap());
        builder.send().await.unwrap();
        let captured = requests.lock().unwrap();
        assert_eq!(captured[0].method, http::Method::DELETE);
    }

    #[tokio::test]
    #[ignore = "requires valid OSS credentials"]
    async fn e2e_bucket_lifecycle() {
        let ak = std::env::var("OSS_ACCESS_KEY_ID").expect("OSS_ACCESS_KEY_ID not set");
        let sk = std::env::var("OSS_ACCESS_KEY_SECRET").expect("OSS_ACCESS_KEY_SECRET not set");
        let region_str = std::env::var("OSS_REGION").unwrap_or_else(|_| "cn-wulanchabu".into());
        let bucket_str = std::env::var("OSS_BUCKET").expect("OSS_BUCKET not set");

        use std::str::FromStr;
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
        bucket
            .put_lifecycle(vec![LifecycleRule {
                id: Some("test-rule".into()),
                prefix: Some("test-lc/".into()),
                status: LifecycleRuleStatus::Enabled,
                expiration_days: Some(1),
                expiration_date: None,
                abort_multipart_upload_days: None,
            }])
            .send()
            .await
            .unwrap();

        let output = bucket.get_lifecycle().send().await.unwrap();
        assert!(!output.rules.is_empty());
        eprintln!("GetBucketLifecycle: {} rules", output.rules.len());

        bucket.delete_lifecycle().send().await.unwrap();
        eprintln!("DeleteBucketLifecycle: ok");
    }
}
