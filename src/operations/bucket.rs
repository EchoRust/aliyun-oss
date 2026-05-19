use std::sync::Arc;

use serde::Serialize;

use crate::client::{BucketOperations, OSSClientInner};
use crate::error::{ErrorContext, OssError, OssErrorKind, Result};
use crate::http::client::HttpRequest;
use crate::types::acl::BucketAcl;
use crate::types::bucket::BucketName;
use crate::types::region::Region;
use crate::types::storage::{DataRedundancyType, StorageClass};
use crate::util::xml::to_xml;

#[derive(Debug, Clone, Serialize)]
#[serde(rename = "CreateBucketConfiguration")]
struct CreateBucketConfiguration {
    #[serde(rename = "StorageClass", skip_serializing_if = "Option::is_none")]
    storage_class: Option<String>,
    #[serde(rename = "DataRedundancyType", skip_serializing_if = "Option::is_none")]
    data_redundancy_type: Option<String>,
}

pub struct PutBucketBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    acl: Option<BucketAcl>,
    storage_class: Option<StorageClass>,
    data_redundancy: Option<DataRedundancyType>,
}

impl PutBucketBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName) -> Self {
        Self {
            client,
            bucket,
            acl: None,
            storage_class: None,
            data_redundancy: None,
        }
    }

    pub fn acl(mut self, acl: BucketAcl) -> Self {
        self.acl = Some(acl);
        self
    }

    pub fn storage_class(mut self, sc: StorageClass) -> Self {
        self.storage_class = Some(sc);
        self
    }

    pub fn data_redundancy(mut self, dr: DataRedundancyType) -> Self {
        self.data_redundancy = Some(dr);
        self
    }

    pub async fn send(self) -> Result<PutBucketOutput> {
        let region: Region = self.client.region.clone();
        let endpoint = self.client.endpoint.clone();
        let uri = format!("https://{}/{}/", endpoint, self.bucket.as_str());

        let mut req = HttpRequest::builder().method(http::Method::PUT).uri(&uri);

        if let Some(acl) = &self.acl {
            req = req.header(
                http::HeaderName::from_static("x-oss-acl"),
                http::HeaderValue::from_str(acl.as_str()).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set x-oss-acl header".into()),
                        bucket: Some(self.bucket.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
        }

        let config = CreateBucketConfiguration {
            storage_class: self.storage_class.map(|sc| sc.as_str().to_string()),
            data_redundancy_type: self.data_redundancy.map(|dr| dr.as_str().to_string()),
        };

        if config.storage_class.is_some() || config.data_redundancy_type.is_some() {
            let body_xml = to_xml(&config)?;
            req = req.body(bytes::Bytes::from(body_xml));
        }

        let request = req.build();

        let response = self.client.http.send(request).await.map_err(|e| OssError {
            kind: OssErrorKind::TransportError,
            context: Box::new(ErrorContext {
                operation: Some("PutBucket".into()),
                bucket: Some(self.bucket.to_string()),
                endpoint: Some(endpoint),
                ..Default::default()
            }),
            source: Some(Box::new(e)),
        })?;

        if response.is_success() {
            Ok(PutBucketOutput {
                request_id: response
                    .headers
                    .get("x-oss-request-id")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("")
                    .to_string(),
                region,
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
                    operation: Some("PutBucket".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct PutBucketOutput {
    pub request_id: String,
    pub region: Region,
}

impl BucketOperations {
    pub fn create(&self) -> PutBucketBuilder {
        PutBucketBuilder::new(self.client_inner().clone(), self.bucket_name().clone())
    }

    pub fn delete(&self) -> DeleteBucketBuilder {
        DeleteBucketBuilder::new(self.client_inner().clone(), self.bucket_name().clone())
    }

    pub fn get_info(&self) -> GetBucketInfoBuilder {
        GetBucketInfoBuilder::new(self.client_inner().clone(), self.bucket_name().clone())
    }
}

pub struct DeleteBucketBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
}

impl DeleteBucketBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName) -> Self {
        Self { client, bucket }
    }

    pub async fn send(self) -> Result<DeleteBucketOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!("https://{}.{}", self.bucket.as_str(), endpoint);

        let request = HttpRequest::builder()
            .method(http::Method::DELETE)
            .uri(&uri)
            .build();

        let response = self
            .client
            .send_signed(request, Some(&self.bucket), Vec::new())
            .await
            .map_err(|e| OssError {
                kind: OssErrorKind::TransportError,
                context: Box::new(ErrorContext {
                    operation: Some("DeleteBucket".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.status().is_success() {
            Ok(DeleteBucketOutput {
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
                    operation: Some("DeleteBucket".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeleteBucketOutput {
    pub request_id: String,
}

pub struct GetBucketInfoBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
}

impl GetBucketInfoBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName) -> Self {
        Self { client, bucket }
    }

    pub async fn send(self) -> Result<crate::types::response::GetBucketInfoOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = format!("https://{}.{}?bucketInfo", self.bucket.as_str(), endpoint);

        let request = HttpRequest::builder()
            .method(http::Method::GET)
            .uri(&uri)
            .build();

        let query_params = vec![("bucketInfo".into(), "".into())];

        let response = self
            .client
            .send_signed(request, Some(&self.bucket), query_params)
            .await
            .map_err(|e| OssError {
                kind: OssErrorKind::TransportError,
                context: Box::new(ErrorContext {
                    operation: Some("GetBucketInfo".into()),
                    bucket: Some(self.bucket.to_string()),
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
                    resource: Some(self.bucket.to_string()),
                    string_to_sign: None,
                })),
                context: Box::new(ErrorContext {
                    operation: Some("GetBucketInfo".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use std::sync::Mutex;

    use crate::client::OSSClientInner;
    use crate::config::credentials::Credentials;
    use crate::http::client::{HttpClient, HttpRequest, HttpResponse};

    use super::*;
    use crate::util::xml::to_xml;

    struct RecordingHttpClient {
        requests: Arc<Mutex<Vec<HttpRequest>>>,
        response_status: http::StatusCode,
        response_body: bytes::Bytes,
    }

    #[async_trait::async_trait]
    impl HttpClient for RecordingHttpClient {
        async fn send(&self, request: HttpRequest) -> crate::error::Result<HttpResponse> {
            self.requests.lock().unwrap().push(request);
            let mut headers = http::HeaderMap::new();
            headers.insert(
                "x-oss-request-id",
                http::HeaderValue::from_static("rid-bucket"),
            );
            Ok(HttpResponse {
                status: self.response_status,
                headers,
                body: self.response_body.clone(),
            })
        }
    }

    fn create_test_inner(
        status: http::StatusCode,
        body: bytes::Bytes,
    ) -> (Arc<OSSClientInner>, Arc<Mutex<Vec<HttpRequest>>>) {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let http = Arc::new(RecordingHttpClient {
            requests: requests.clone(),
            response_status: status,
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
            region: crate::types::region::Region::CnHangzhou,
            endpoint: "oss-cn-hangzhou.aliyuncs.com".into(),
        });
        (inner, requests)
    }

    #[test]
    fn put_bucket_builder_generates_correct_xml_body() {
        let config = CreateBucketConfiguration {
            storage_class: Some("Standard".into()),
            data_redundancy_type: Some("LRS".into()),
        };
        let xml = to_xml(&config).unwrap();
        assert!(xml.contains("<StorageClass>Standard</StorageClass>"));
        assert!(xml.contains("<DataRedundancyType>LRS</DataRedundancyType>"));
    }

    #[test]
    fn put_bucket_builder_xml_omits_none_fields() {
        let config = CreateBucketConfiguration {
            storage_class: Some("IA".into()),
            data_redundancy_type: None,
        };
        let xml = to_xml(&config).unwrap();
        assert!(xml.contains("<StorageClass>IA</StorageClass>"));
        assert!(!xml.contains("DataRedundancyType"));
    }

    #[test]
    fn put_bucket_builder_empty_config() {
        let config = CreateBucketConfiguration {
            storage_class: None,
            data_redundancy_type: None,
        };
        let xml = to_xml(&config).unwrap();
        assert!(xml.contains("CreateBucketConfiguration"));
    }

    #[tokio::test]
    async fn delete_bucket_sends_delete_request() {
        let (inner, requests) =
            create_test_inner(http::StatusCode::NO_CONTENT, bytes::Bytes::new());
        let builder = DeleteBucketBuilder::new(inner, BucketName::new("test-bucket").unwrap());

        builder.send().await.unwrap();

        let captured = requests.lock().unwrap();
        assert_eq!(captured[0].method, http::Method::DELETE);
    }

    #[tokio::test]
    async fn get_bucket_info_parses_response() {
        let info_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<BucketInfo>
  <Bucket>
    <Name>test-bucket</Name>
    <CreationDate>2024-01-01T00:00:00.000Z</CreationDate>
    <Location>oss-cn-hangzhou</Location>
    <StorageClass>Standard</StorageClass>
    <ExtranetEndpoint>test-bucket.oss-cn-hangzhou.aliyuncs.com</ExtranetEndpoint>
    <IntranetEndpoint>test-bucket.oss-cn-hangzhou-internal.aliyuncs.com</IntranetEndpoint>
    <Owner>
      <ID>owner-id</ID>
    </Owner>
  </Bucket>
</BucketInfo>"#;

        let (inner, requests) =
            create_test_inner(http::StatusCode::OK, bytes::Bytes::from(info_xml));
        let builder = GetBucketInfoBuilder::new(inner, BucketName::new("test-bucket").unwrap());

        let output = builder.send().await.unwrap();
        assert_eq!(output.bucket.name, "test-bucket");

        let captured = requests.lock().unwrap();
        assert!(captured[0].uri.contains("bucketInfo"));
    }

    #[tokio::test]
    #[ignore = "requires valid OSS credentials"]
    async fn e2e_get_bucket_info() {
        let ak = std::env::var("OSS_ACCESS_KEY_ID").expect("OSS_ACCESS_KEY_ID not set");
        let sk = std::env::var("OSS_ACCESS_KEY_SECRET").expect("OSS_ACCESS_KEY_SECRET not set");
        let region_str = std::env::var("OSS_REGION").unwrap_or_else(|_| "cn-wulanchabu".into());
        let bucket_str = std::env::var("OSS_BUCKET").expect("OSS_BUCKET not set");

        let region = crate::types::region::Region::from_str(&region_str).unwrap_or_else(|_| {
            crate::types::region::Region::Custom {
                endpoint: format!("oss-{}.aliyuncs.com", region_str),
                region_id: region_str.clone(),
            }
        });

        let client = crate::client::OSSClient::builder()
            .region(region)
            .credentials(ak, sk)
            .build()
            .unwrap();

        let output = client
            .bucket(&bucket_str)
            .unwrap()
            .get_info()
            .send()
            .await
            .unwrap();

        assert_eq!(output.bucket.name, bucket_str);
        eprintln!(
            "GetBucketInfo: name={}, location={}, storage={}",
            output.bucket.name, output.bucket.location, output.bucket.storage_class
        );
    }
}
