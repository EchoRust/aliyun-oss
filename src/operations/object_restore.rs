use std::sync::Arc;

use serde::Serialize;

use crate::client::{BucketOperations, OSSClientInner};
use crate::error::{ErrorContext, OssError, OssErrorKind, Result};
use crate::http::client::HttpRequest;
use crate::types::bucket::BucketName;
use crate::types::object::ObjectKey;
use crate::util::uri::oss_endpoint_url;

#[derive(Debug, Clone, Serialize)]
#[serde(rename = "RestoreRequest")]
struct RestoreRequest {
    #[serde(rename = "Days")]
    days: i32,
    #[serde(rename = "Tier", skip_serializing_if = "Option::is_none")]
    tier: Option<String>,
}

pub struct RestoreObjectBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    key: ObjectKey,
    days: i32,
    tier: Option<String>,
}

impl RestoreObjectBuilder {
    pub(crate) fn new(
        client: Arc<OSSClientInner>,
        bucket: BucketName,
        key: ObjectKey,
        days: i32,
    ) -> Self {
        Self {
            client,
            bucket,
            key,
            days,
            tier: None,
        }
    }

    pub fn tier(mut self, t: impl Into<String>) -> Self {
        self.tier = Some(t.into());
        self
    }

    pub async fn send(self) -> Result<RestoreObjectOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = oss_endpoint_url(
            &endpoint,
            Some(self.bucket.as_str()),
            Some(self.key.as_str()),
        );
        let full_uri = format!("{}?restore", uri);
        let query_params: Vec<(String, String)> = vec![("restore".into(), String::new())];

        let config = RestoreRequest {
            days: self.days,
            tier: self.tier,
        };
        let body_xml = crate::util::xml::to_xml(&config)?;

        let request = HttpRequest::builder()
            .method(http::Method::POST)
            .uri(&full_uri)
            .body(bytes::Bytes::from(body_xml))
            .build();

        let response = self
            .client
            .send_signed(request, Some(&self.bucket), query_params)
            .await
            .map_err(|e| OssError {
                kind: OssErrorKind::TransportError,
                context: Box::new(ErrorContext {
                    operation: Some("RestoreObject".into()),
                    bucket: Some(self.bucket.to_string()),
                    object_key: Some(self.key.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.status().is_success() {
            Ok(RestoreObjectOutput {
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
                    operation: Some("RestoreObject".into()),
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
pub struct RestoreObjectOutput {
    pub request_id: String,
}

impl BucketOperations {
    pub fn restore_object(
        &self,
        key: impl Into<String>,
        days: i32,
    ) -> Result<RestoreObjectBuilder> {
        Ok(RestoreObjectBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            ObjectKey::new(key.into())?,
            days,
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

    struct Rec {
        requests: Arc<Mutex<Vec<HttpRequest>>>,
    }
    #[async_trait::async_trait]
    impl HttpClient for Rec {
        async fn send(&self, request: HttpRequest) -> crate::error::Result<HttpResponse> {
            self.requests.lock().unwrap().push(request);
            let mut h = http::HeaderMap::new();
            h.insert("x-oss-request-id", http::HeaderValue::from_static("rid"));
            Ok(HttpResponse {
                status: http::StatusCode::OK,
                headers: h,
                body: bytes::Bytes::new(),
            })
        }
    }
    fn c() -> (Arc<OSSClientInner>, Arc<Mutex<Vec<HttpRequest>>>) {
        let r = Arc::new(Mutex::new(Vec::new()));
        let h = Arc::new(Rec {
            requests: r.clone(),
        });
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
            r,
        )
    }

    #[test]
    fn restore_xml_generation() {
        let r = RestoreRequest {
            days: 3,
            tier: Some("Standard".into()),
        };
        let x = crate::util::xml::to_xml(&r).unwrap();
        assert!(x.contains("<Days>3</Days>"));
        assert!(x.contains("<Tier>Standard</Tier>"));
    }

    #[tokio::test]
    async fn restore_sends_post() {
        let (i, rq) = c();
        RestoreObjectBuilder::new(
            i,
            BucketName::new("test-bucket").unwrap(),
            ObjectKey::new("k").unwrap(),
            3,
        )
        .send()
        .await
        .unwrap();
        assert_eq!(rq.lock().unwrap()[0].method, http::Method::POST);
    }
}
