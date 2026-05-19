//! Cross-region replication operations.

use std::sync::Arc;

use crate::client::{BucketOperations, OSSClientInner};
use crate::error::{ErrorContext, OssError, OssErrorKind, Result};
use crate::http::client::HttpRequest;
use crate::types::bucket::BucketName;

pub struct PutBucketReplicationBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    body_xml: String,
}

impl PutBucketReplicationBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName, body_xml: String) -> Self {
        Self {
            client,
            bucket,
            body_xml,
        }
    }

    pub async fn send(self) -> Result<PutBucketReplicationOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!("https://{}.{}?replication", self.bucket.as_str(), endpoint);
        let query_params: Vec<(String, String)> = vec![("replication".into(), String::new())];

        let request = HttpRequest::builder()
            .method(http::Method::PUT)
            .uri(&uri)
            .body(bytes::Bytes::from(self.body_xml))
            .build();

        let response = self
            .client
            .send_signed(request, Some(&self.bucket), query_params)
            .await
            .map_err(|e| OssError {
                kind: OssErrorKind::TransportError,
                context: Box::new(ErrorContext {
                    operation: Some("PutBucketReplication".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.status().is_success() {
            Ok(PutBucketReplicationOutput {
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
                    operation: Some("PutBucketReplication".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct PutBucketReplicationOutput {
    pub request_id: String,
}

pub struct GetBucketReplicationBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
}

impl GetBucketReplicationBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName) -> Self {
        Self { client, bucket }
    }

    pub async fn send(self) -> Result<GetBucketReplicationOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!("https://{}.{}?replication", self.bucket.as_str(), endpoint);
        let query_params: Vec<(String, String)> = vec![("replication".into(), String::new())];

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
                    operation: Some("GetBucketReplication".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.is_success() {
            Ok(GetBucketReplicationOutput {
                body: response.body_as_str().unwrap_or("").to_string(),
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
                    operation: Some("GetBucketReplication".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct GetBucketReplicationOutput {
    pub body: String,
}

pub struct DeleteBucketReplicationBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    replication_rule_id: String,
}

impl DeleteBucketReplicationBuilder {
    pub(crate) fn new(
        client: Arc<OSSClientInner>,
        bucket: BucketName,
        replication_rule_id: String,
    ) -> Self {
        Self {
            client,
            bucket,
            replication_rule_id,
        }
    }

    pub async fn send(self) -> Result<DeleteBucketReplicationOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!(
            "https://{}.{}?replication&replicationRuleId={}",
            self.bucket.as_str(),
            endpoint,
            self.replication_rule_id
        );
        let query_params: Vec<(String, String)> = vec![
            ("replication".into(), String::new()),
            ("replicationRuleId".into(), self.replication_rule_id),
        ];

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
                    operation: Some("DeleteBucketReplication".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.status().is_success() {
            Ok(DeleteBucketReplicationOutput {
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
                    operation: Some("DeleteBucketReplication".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeleteBucketReplicationOutput {
    pub request_id: String,
}

impl BucketOperations {
    pub fn put_replication(&self, body_xml: String) -> PutBucketReplicationBuilder {
        PutBucketReplicationBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            body_xml,
        )
    }

    pub fn get_replication(&self) -> GetBucketReplicationBuilder {
        GetBucketReplicationBuilder::new(self.client_inner().clone(), self.bucket_name().clone())
    }

    pub fn delete_replication(
        &self,
        replication_rule_id: String,
    ) -> DeleteBucketReplicationBuilder {
        DeleteBucketReplicationBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            replication_rule_id,
        )
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
                http::HeaderValue::from_static("rid-repl"),
            );
            Ok(HttpResponse {
                status: self.status_code,
                headers,
                body: self.response_body.clone(),
            })
        }
    }

    fn create_test_inner(
        body: bytes::Bytes,
    ) -> (Arc<OSSClientInner>, Arc<Mutex<Vec<HttpRequest>>>) {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let http = Arc::new(RecordingHttpClient {
            requests: requests.clone(),
            status_code: http::StatusCode::OK,
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
    async fn put_replication_sends_request() {
        let (inner, requests) = create_test_inner(bytes::Bytes::new());
        let builder = PutBucketReplicationBuilder::new(
            inner,
            BucketName::new("test-bucket").unwrap(),
            "<ReplicationConfiguration/>".into(),
        );
        builder.send().await.unwrap();
        let captured = requests.lock().unwrap();
        assert_eq!(captured[0].method, http::Method::PUT);
        assert!(captured[0].uri.contains("?replication"));
    }

    #[tokio::test]
    async fn delete_replication_sends_with_rule_id() {
        let (inner, requests) = create_test_inner(bytes::Bytes::new());
        let builder = DeleteBucketReplicationBuilder::new(
            inner,
            BucketName::new("test-bucket").unwrap(),
            "rule-1".into(),
        );
        builder.send().await.unwrap();
        let captured = requests.lock().unwrap();
        assert_eq!(captured[0].method, http::Method::DELETE);
        assert!(captured[0].uri.contains("replicationRuleId=rule-1"));
    }
}
