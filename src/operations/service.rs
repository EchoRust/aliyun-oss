//! Service-level operations (list buckets).

use std::sync::Arc;

use crate::client::OSSClient;
use crate::error::{ErrorContext, OssError, OssErrorKind, Result};
use crate::http::client::HttpRequest;

pub struct ListBucketsBuilder {
    client: Arc<crate::client::OSSClientInner>,
}

impl ListBucketsBuilder {
    pub(crate) fn new(client: Arc<crate::client::OSSClientInner>) -> Self {
        Self { client }
    }

    pub async fn send(self) -> Result<crate::types::response::ListBucketsOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!("https://{}", endpoint);

        let request = HttpRequest::builder()
            .method(http::Method::GET)
            .uri(&uri)
            .build();

        let response = self
            .client
            .send_signed(request, None, Vec::new())
            .await
            .map_err(|e| OssError {
                kind: OssErrorKind::TransportError,
                context: Box::new(ErrorContext {
                    operation: Some("ListBuckets".into()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.is_success() {
            let body_str = response.body_as_str().unwrap_or("");
            Ok(crate::util::xml::from_xml(body_str)?)
        } else {
            Err(OssError {
                kind: OssErrorKind::ServiceError(Box::new(crate::error::OssServiceError {
                    status_code: response.status().as_u16(),
                    code: String::new(),
                    message: String::new(),
                    request_id: String::new(),
                    host_id: String::new(),
                    resource: None,
                    string_to_sign: None,
                })),
                context: Box::new(ErrorContext {
                    operation: Some("ListBuckets".into()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

impl OSSClient {
    pub fn list_buckets(&self) -> ListBucketsBuilder {
        ListBucketsBuilder::new(self.inner().clone())
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
    }

    #[async_trait::async_trait]
    impl HttpClient for RecordingHttpClient {
        async fn send(&self, request: HttpRequest) -> crate::error::Result<HttpResponse> {
            self.requests.lock().unwrap().push(request);
            let mut headers = http::HeaderMap::new();
            headers.insert(
                "x-oss-request-id",
                http::HeaderValue::from_static("rid-svc"),
            );
            Ok(HttpResponse {
                status: http::StatusCode::OK,
                headers,
                body: bytes::Bytes::from(
                    r#"<?xml version="1.0" encoding="UTF-8"?><ListAllMyBucketsResult><Owner><ID>owner-id</ID></Owner><Buckets></Buckets></ListAllMyBucketsResult>"#,
                ),
            })
        }
    }

    fn create_test_inner() -> (Arc<OSSClientInner>, Arc<Mutex<Vec<HttpRequest>>>) {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let http = Arc::new(RecordingHttpClient {
            requests: requests.clone(),
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
    async fn list_buckets_sends_get_request() {
        let (inner, requests) = create_test_inner();
        let builder = ListBucketsBuilder::new(inner);

        builder.send().await.unwrap();

        let captured = requests.lock().unwrap();
        assert_eq!(captured[0].method, http::Method::GET);
    }

    #[tokio::test]
    #[ignore = "requires valid OSS credentials"]
    async fn e2e_list_buckets() {
        let ak = std::env::var("OSS_ACCESS_KEY_ID").expect("OSS_ACCESS_KEY_ID not set");
        let sk = std::env::var("OSS_ACCESS_KEY_SECRET").expect("OSS_ACCESS_KEY_SECRET not set");
        let region_str = std::env::var("OSS_REGION").unwrap_or_else(|_| "cn-wulanchabu".into());

        let region = Region::from_str(&region_str).unwrap_or_else(|_| Region::Custom {
            endpoint: format!("oss-{}.aliyuncs.com", region_str),
            region_id: region_str.clone(),
        });

        let client = crate::client::OSSClient::builder()
            .region(region)
            .credentials(ak, sk)
            .build()
            .unwrap();

        let output = client.list_buckets().send().await.unwrap();
        assert!(!output.owner.id.is_empty());
        eprintln!(
            "ListBuckets: {} buckets, owner={}",
            output.buckets.bucket.len(),
            output.owner.id
        );
    }
}
