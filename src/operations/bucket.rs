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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::xml::to_xml;

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
}
