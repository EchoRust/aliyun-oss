//! Bucket inventory configuration operations.

use std::sync::Arc;

use crate::client::{BucketOperations, OSSClientInner};
use crate::error::{ErrorContext, OssError, OssErrorKind, Result};
use crate::http::client::HttpRequest;
use crate::types::bucket::BucketName;

pub struct PutBucketInventoryBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    inventory_id: String,
    body_xml: String,
}

impl PutBucketInventoryBuilder {
    pub(crate) fn new(
        client: Arc<OSSClientInner>,
        bucket: BucketName,
        inventory_id: String,
        body_xml: String,
    ) -> Self {
        Self {
            client,
            bucket,
            inventory_id,
            body_xml,
        }
    }

    pub async fn send(self) -> Result<PutBucketInventoryOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!(
            "https://{}.{}?inventory&inventoryId={}",
            self.bucket.as_str(),
            endpoint,
            self.inventory_id
        );
        let query_params: Vec<(String, String)> = vec![
            ("inventory".into(), String::new()),
            ("inventoryId".into(), self.inventory_id.clone()),
        ];

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
                    operation: Some("PutBucketInventory".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.status().is_success() {
            Ok(PutBucketInventoryOutput {
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
                    operation: Some("PutBucketInventory".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct PutBucketInventoryOutput {
    pub request_id: String,
}

pub struct GetBucketInventoryBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    inventory_id: String,
}

impl GetBucketInventoryBuilder {
    pub(crate) fn new(
        client: Arc<OSSClientInner>,
        bucket: BucketName,
        inventory_id: String,
    ) -> Self {
        Self {
            client,
            bucket,
            inventory_id,
        }
    }

    pub async fn send(self) -> Result<GetBucketInventoryOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!(
            "https://{}.{}?inventory&inventoryId={}",
            self.bucket.as_str(),
            endpoint,
            self.inventory_id
        );
        let query_params: Vec<(String, String)> = vec![
            ("inventory".into(), String::new()),
            ("inventoryId".into(), self.inventory_id),
        ];

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
                    operation: Some("GetBucketInventory".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.is_success() {
            Ok(GetBucketInventoryOutput {
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
                    operation: Some("GetBucketInventory".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct GetBucketInventoryOutput {
    pub body: String,
}

pub struct DeleteBucketInventoryBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    inventory_id: String,
}

impl DeleteBucketInventoryBuilder {
    pub(crate) fn new(
        client: Arc<OSSClientInner>,
        bucket: BucketName,
        inventory_id: String,
    ) -> Self {
        Self {
            client,
            bucket,
            inventory_id,
        }
    }

    pub async fn send(self) -> Result<DeleteBucketInventoryOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!(
            "https://{}.{}?inventory&inventoryId={}",
            self.bucket.as_str(),
            endpoint,
            self.inventory_id
        );
        let query_params: Vec<(String, String)> = vec![
            ("inventory".into(), String::new()),
            ("inventoryId".into(), self.inventory_id),
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
                    operation: Some("DeleteBucketInventory".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.status().is_success() {
            Ok(DeleteBucketInventoryOutput {
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
                    operation: Some("DeleteBucketInventory".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeleteBucketInventoryOutput {
    pub request_id: String,
}

pub struct ListBucketInventoryBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
}

impl ListBucketInventoryBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName) -> Self {
        Self { client, bucket }
    }

    pub async fn send(self) -> Result<ListBucketInventoryOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!("https://{}.{}?inventory", self.bucket.as_str(), endpoint);
        let query_params: Vec<(String, String)> = vec![("inventory".into(), String::new())];

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
                    operation: Some("ListBucketInventory".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.is_success() {
            Ok(ListBucketInventoryOutput {
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
                    operation: Some("ListBucketInventory".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct ListBucketInventoryOutput {
    pub body: String,
}

impl BucketOperations {
    pub fn put_inventory(
        &self,
        inventory_id: String,
        body_xml: String,
    ) -> PutBucketInventoryBuilder {
        PutBucketInventoryBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            inventory_id,
            body_xml,
        )
    }

    pub fn get_inventory(&self, inventory_id: String) -> GetBucketInventoryBuilder {
        GetBucketInventoryBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            inventory_id,
        )
    }

    pub fn delete_inventory(&self, inventory_id: String) -> DeleteBucketInventoryBuilder {
        DeleteBucketInventoryBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            inventory_id,
        )
    }

    pub fn list_inventory(&self) -> ListBucketInventoryBuilder {
        ListBucketInventoryBuilder::new(self.client_inner().clone(), self.bucket_name().clone())
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
                http::HeaderValue::from_static("rid-inv"),
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
    async fn put_inventory_sends_with_inventory_id() {
        let (inner, requests) = create_test_inner(bytes::Bytes::new());
        let builder = PutBucketInventoryBuilder::new(
            inner,
            BucketName::new("test-bucket").unwrap(),
            "report1".into(),
            "<InventoryConfiguration/>".into(),
        );
        builder.send().await.unwrap();
        let captured = requests.lock().unwrap();
        assert_eq!(captured[0].method, http::Method::PUT);
        assert!(captured[0].uri.contains("?inventory"));
    }

    #[tokio::test]
    async fn list_inventory_sends_get() {
        let (inner, requests) =
            create_test_inner(bytes::Bytes::from("<ListInventoryConfigurationsResult/>"));
        let builder =
            ListBucketInventoryBuilder::new(inner, BucketName::new("test-bucket").unwrap());
        let _output = builder.send().await.unwrap();
        let captured = requests.lock().unwrap();
        assert_eq!(captured[0].method, http::Method::GET);
    }
}
