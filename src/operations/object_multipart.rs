use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::client::{BucketOperations, OSSClientInner};
use crate::error::{ErrorContext, OssError, OssErrorKind, Result};
use crate::http::client::HttpRequest;
use crate::types::acl::ObjectAcl;
use crate::types::bucket::BucketName;
use crate::types::object::ObjectKey;
use crate::types::storage::{ServerSideEncryption, StorageClass};
use crate::util::uri::oss_endpoint_url;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "InitiateMultipartUploadResult")]
struct InitiateMultipartUploadResult {
    #[serde(rename = "Bucket")]
    bucket: String,
    #[serde(rename = "Key")]
    key: String,
    #[serde(rename = "UploadId")]
    upload_id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "ListMultipartUploadsResult")]
struct ListMultipartUploadsResult {
    #[serde(rename = "Bucket")]
    bucket: String,
    #[serde(rename = "Upload", default)]
    uploads: Vec<MultipartUpload>,
    #[serde(rename = "IsTruncated")]
    is_truncated: bool,
    #[serde(rename = "NextKeyMarker", default)]
    next_key_marker: String,
    #[serde(rename = "NextUploadIdMarker", default)]
    next_upload_id_marker: String,
    #[serde(rename = "MaxUploads")]
    max_uploads: i32,
}

#[derive(Debug, Clone, Deserialize)]
struct MultipartUpload {
    #[serde(rename = "Key")]
    key: String,
    #[serde(rename = "UploadId")]
    upload_id: String,
    #[serde(rename = "Initiated")]
    initiated: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "ListPartsResult")]
struct ListPartsResult {
    #[serde(rename = "Bucket")]
    bucket: String,
    #[serde(rename = "Key")]
    key: String,
    #[serde(rename = "UploadId")]
    upload_id: String,
    #[serde(rename = "Part", default)]
    parts: Vec<PartSummary>,
    #[serde(rename = "MaxParts")]
    max_parts: i32,
    #[serde(rename = "IsTruncated")]
    is_truncated: bool,
    #[serde(rename = "NextPartNumberMarker", default)]
    next_part_number_marker: String,
}

#[derive(Debug, Clone, Deserialize)]
struct PartSummary {
    #[serde(rename = "PartNumber")]
    part_number: i32,
    #[serde(rename = "LastModified")]
    last_modified: String,
    #[serde(rename = "ETag")]
    etag: String,
    #[serde(rename = "Size")]
    size: u64,
}

pub struct InitiateMultipartUploadBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    key: ObjectKey,
    cache_control: Option<String>,
    content_type: Option<String>,
    content_disposition: Option<String>,
    content_encoding: Option<String>,
    expires: Option<String>,
    acl: Option<ObjectAcl>,
    storage_class: Option<StorageClass>,
    sse: Option<ServerSideEncryption>,
    sse_key_id: Option<String>,
    tagging: Option<String>,
    metadata: Vec<(String, String)>,
}

impl InitiateMultipartUploadBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName, key: ObjectKey) -> Self {
        Self {
            client,
            bucket,
            key,
            cache_control: None,
            content_type: None,
            content_disposition: None,
            content_encoding: None,
            expires: None,
            acl: None,
            storage_class: None,
            sse: None,
            sse_key_id: None,
            tagging: None,
            metadata: Vec::new(),
        }
    }

    pub fn cache_control(mut self, v: impl Into<String>) -> Self {
        self.cache_control = Some(v.into());
        self
    }

    pub fn content_type(mut self, v: impl Into<String>) -> Self {
        self.content_type = Some(v.into());
        self
    }

    pub fn content_disposition(mut self, v: impl Into<String>) -> Self {
        self.content_disposition = Some(v.into());
        self
    }

    pub fn content_encoding(mut self, v: impl Into<String>) -> Self {
        self.content_encoding = Some(v.into());
        self
    }

    pub fn expires(mut self, v: impl Into<String>) -> Self {
        self.expires = Some(v.into());
        self
    }

    pub fn acl(mut self, acl: ObjectAcl) -> Self {
        self.acl = Some(acl);
        self
    }

    pub fn storage_class(mut self, sc: StorageClass) -> Self {
        self.storage_class = Some(sc);
        self
    }

    pub fn server_side_encryption(mut self, sse: impl Into<String>) -> Self {
        match sse.into().as_str() {
            "AES256" => self.sse = Some(ServerSideEncryption::AES256),
            "KMS" => self.sse = Some(ServerSideEncryption::KMS),
            _ => {}
        }
        self
    }

    pub fn sse_key_id(mut self, key_id: impl Into<String>) -> Self {
        self.sse_key_id = Some(key_id.into());
        self
    }

    pub fn tagging(mut self, tag: impl Into<String>) -> Self {
        self.tagging = Some(tag.into());
        self
    }

    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.push((key.into(), value.into()));
        self
    }

    pub async fn send(self) -> Result<InitiateMultipartUploadOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = oss_endpoint_url(
            &endpoint,
            Some(self.bucket.as_str()),
            Some(self.key.as_str()),
        );
        let full_uri = format!("{}?uploads", uri);

        let mut req = HttpRequest::builder()
            .method(http::Method::POST)
            .uri(&full_uri);

        if let Some(ref ct) = self.content_type {
            req = req.header(
                http::HeaderName::from_static("content-type"),
                http::HeaderValue::from_str(ct).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set content-type header".into()),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
        }

        if let Some(ref cc) = self.cache_control {
            req = req.header(
                http::HeaderName::from_static("cache-control"),
                http::HeaderValue::from_str(cc).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set cache-control header".into()),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
        }

        if let Some(ref cd) = self.content_disposition {
            req = req.header(
                http::HeaderName::from_static("content-disposition"),
                http::HeaderValue::from_str(cd).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set content-disposition header".into()),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
        }

        if let Some(ref ce) = self.content_encoding {
            req = req.header(
                http::HeaderName::from_static("content-encoding"),
                http::HeaderValue::from_str(ce).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set content-encoding header".into()),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
        }

        if let Some(ref exp) = self.expires {
            req = req.header(
                http::HeaderName::from_static("expires"),
                http::HeaderValue::from_str(exp).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set expires header".into()),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
        }

        if let Some(acl) = self.acl {
            req = req.header(
                http::HeaderName::from_static("x-oss-object-acl"),
                http::HeaderValue::from_str(acl.as_str()).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set x-oss-object-acl header".into()),
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
                        operation: Some("set x-oss-storage-class header".into()),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
        }

        if let Some(ref sse) = self.sse {
            req = req.header(
                http::HeaderName::from_static("x-oss-server-side-encryption"),
                http::HeaderValue::from_str(sse.as_str()).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set x-oss-server-side-encryption header".into()),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
            if let Some(key_id) = sse.key_id() {
                req = req.header(
                    http::HeaderName::from_static("x-oss-server-side-encryption-key-id"),
                    http::HeaderValue::from_str(key_id).map_err(|e| OssError {
                        kind: OssErrorKind::ValidationError,
                        context: Box::new(ErrorContext {
                            operation: Some(
                                "set x-oss-server-side-encryption-key-id header".into(),
                            ),
                            bucket: Some(self.bucket.to_string()),
                            object_key: Some(self.key.to_string()),
                            ..Default::default()
                        }),
                        source: Some(Box::new(e)),
                    })?,
                );
            }
        } else if let Some(ref key_id) = self.sse_key_id {
            req = req.header(
                http::HeaderName::from_static("x-oss-server-side-encryption"),
                http::HeaderValue::from_str("KMS").map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set x-oss-server-side-encryption header".into()),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
            req = req.header(
                http::HeaderName::from_static("x-oss-server-side-encryption-key-id"),
                http::HeaderValue::from_str(key_id).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set x-oss-server-side-encryption-key-id header".into()),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
        }

        if let Some(ref tag) = self.tagging {
            req = req.header(
                http::HeaderName::from_static("x-oss-tagging"),
                http::HeaderValue::from_str(tag).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set x-oss-tagging header".into()),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
        }

        for (k, v) in &self.metadata {
            let header_name = http::HeaderName::from_bytes(k.as_bytes()).map_err(|e| OssError {
                kind: OssErrorKind::ValidationError,
                context: Box::new(ErrorContext {
                    operation: Some(format!("set metadata header '{}'", k)),
                    bucket: Some(self.bucket.to_string()),
                    object_key: Some(self.key.to_string()),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;
            req = req.header(
                header_name,
                http::HeaderValue::from_str(v).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some(format!("set metadata header value '{}'", k)),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
        }

        let query_params: Vec<(String, String)> = vec![("uploads".into(), String::new())];
        let request = req.build();

        let response = self
            .client
            .send_signed(request, Some(&self.bucket), query_params)
            .await
            .map_err(|e| OssError {
                kind: OssErrorKind::TransportError,
                context: Box::new(ErrorContext {
                    operation: Some("InitiateMultipartUpload".into()),
                    bucket: Some(self.bucket.to_string()),
                    object_key: Some(self.key.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.is_success() {
            let request_id = response
                .headers
                .get("x-oss-request-id")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_string();

            let body_str = response.body_as_str().unwrap_or("");

            let result: InitiateMultipartUploadResult = crate::util::xml::from_xml(body_str)
                .map_err(|e| OssError {
                    kind: OssErrorKind::DeserializationError,
                    context: Box::new(ErrorContext {
                        operation: Some("InitiateMultipartUpload: parse XML".into()),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?;

            Ok(InitiateMultipartUploadOutput {
                request_id,
                bucket: result.bucket,
                key: result.key,
                upload_id: result.upload_id,
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
                    operation: Some("InitiateMultipartUpload".into()),
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
pub struct InitiateMultipartUploadOutput {
    pub request_id: String,
    pub bucket: String,
    pub key: String,
    pub upload_id: String,
}

pub struct UploadPartBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    key: ObjectKey,
    upload_id: String,
    part_number: u32,
    body: Option<bytes::Bytes>,
    content_md5: Option<String>,
}

impl UploadPartBuilder {
    pub(crate) fn new(
        client: Arc<OSSClientInner>,
        bucket: BucketName,
        key: ObjectKey,
        upload_id: impl Into<String>,
        part_number: u32,
    ) -> Self {
        Self {
            client,
            bucket,
            key,
            upload_id: upload_id.into(),
            part_number,
            body: None,
            content_md5: None,
        }
    }

    pub fn body(mut self, body: impl Into<bytes::Bytes>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn content_md5(mut self, md5: impl Into<String>) -> Self {
        self.content_md5 = Some(md5.into());
        self
    }

    pub(crate) fn compute_md5(body: &[u8]) -> String {
        use md5::{Digest, Md5};
        let digest = Md5::digest(body);
        base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            digest.as_slice(),
        )
    }

    pub async fn send(self) -> Result<UploadPartOutput> {
        let body = self.body.ok_or_else(|| OssError {
            kind: OssErrorKind::ValidationError,
            context: Box::new(ErrorContext {
                operation: Some("UploadPart: body is required".into()),
                bucket: Some(self.bucket.to_string()),
                object_key: Some(self.key.to_string()),
                ..Default::default()
            }),
            source: None,
        })?;

        if self.part_number < 1 || self.part_number > 10000 {
            return Err(OssError {
                kind: OssErrorKind::ValidationError,
                context: Box::new(ErrorContext {
                    operation: Some("UploadPart: part_number must be 1..=10000".into()),
                    bucket: Some(self.bucket.to_string()),
                    object_key: Some(self.key.to_string()),
                    ..Default::default()
                }),
                source: None,
            });
        }

        let endpoint = self.client.endpoint.clone();
        let uri = oss_endpoint_url(
            &endpoint,
            Some(self.bucket.as_str()),
            Some(self.key.as_str()),
        );
        let full_uri = format!(
            "{}?partNumber={}&uploadId={}",
            uri, self.part_number, self.upload_id
        );

        let query_params: Vec<(String, String)> = vec![
            ("partNumber".into(), self.part_number.to_string()),
            ("uploadId".into(), self.upload_id.clone()),
        ];

        let mut req = HttpRequest::builder()
            .method(http::Method::PUT)
            .uri(&full_uri);

        let md5_value = self.content_md5.unwrap_or_else(|| Self::compute_md5(&body));
        req = req.header(
            http::HeaderName::from_static("content-md5"),
            http::HeaderValue::from_str(&md5_value).map_err(|e| OssError {
                kind: OssErrorKind::ValidationError,
                context: Box::new(ErrorContext {
                    operation: Some("set content-md5 header".into()),
                    bucket: Some(self.bucket.to_string()),
                    object_key: Some(self.key.to_string()),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?,
        );

        let request = req.body(body).build();

        let response = self
            .client
            .send_signed(request, Some(&self.bucket), query_params)
            .await
            .map_err(|e| OssError {
                kind: OssErrorKind::TransportError,
                context: Box::new(ErrorContext {
                    operation: Some("UploadPart".into()),
                    bucket: Some(self.bucket.to_string()),
                    object_key: Some(self.key.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.is_success() {
            let request_id = response
                .headers
                .get("x-oss-request-id")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_string();

            let etag = response
                .headers
                .get("ETag")
                .or_else(|| response.headers.get("etag"))
                .and_then(|v| v.to_str().ok())
                .map(|s| s.trim_matches('"').to_string())
                .unwrap_or_default();

            Ok(UploadPartOutput {
                request_id,
                etag,
                part_number: self.part_number,
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
                    operation: Some("UploadPart".into()),
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
pub struct UploadPartOutput {
    pub request_id: String,
    pub etag: String,
    pub part_number: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "CopyPartResult")]
struct CopyPartResult {
    #[serde(rename = "ETag")]
    etag: String,
    #[serde(rename = "LastModified")]
    last_modified: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename = "CompleteMultipartUpload")]
struct CompleteMultipartUpload {
    #[serde(rename = "Part")]
    parts: Vec<UploadPartItem>,
}

#[derive(Debug, Clone, Serialize)]
struct UploadPartItem {
    #[serde(rename = "PartNumber")]
    part_number: i32,
    #[serde(rename = "ETag")]
    etag: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "CompleteMultipartUploadResult")]
struct CompleteMultipartUploadResult {
    #[serde(rename = "Location")]
    location: String,
    #[serde(rename = "Bucket")]
    bucket: String,
    #[serde(rename = "Key")]
    key: String,
    #[serde(rename = "ETag")]
    etag: String,
}

pub struct AbortMultipartUploadBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    key: ObjectKey,
    upload_id: String,
}

impl AbortMultipartUploadBuilder {
    pub(crate) fn new(
        client: Arc<OSSClientInner>,
        bucket: BucketName,
        key: ObjectKey,
        upload_id: impl Into<String>,
    ) -> Self {
        Self {
            client,
            bucket,
            key,
            upload_id: upload_id.into(),
        }
    }

    pub async fn send(self) -> Result<AbortMultipartUploadOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = oss_endpoint_url(
            &endpoint,
            Some(self.bucket.as_str()),
            Some(self.key.as_str()),
        );
        let full_uri = format!("{}?uploadId={}", uri, self.upload_id);

        let query_params: Vec<(String, String)> = vec![("uploadId".into(), self.upload_id.clone())];

        let request = HttpRequest::builder()
            .method(http::Method::DELETE)
            .uri(&full_uri)
            .build();

        let response = self
            .client
            .send_signed(request, Some(&self.bucket), query_params)
            .await
            .map_err(|e| OssError {
                kind: OssErrorKind::TransportError,
                context: Box::new(ErrorContext {
                    operation: Some("AbortMultipartUpload".into()),
                    bucket: Some(self.bucket.to_string()),
                    object_key: Some(self.key.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.status().is_success() {
            Ok(AbortMultipartUploadOutput {
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
                    operation: Some("AbortMultipartUpload".into()),
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
pub struct AbortMultipartUploadOutput {
    pub request_id: String,
}

pub struct CompleteMultipartUploadBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    key: ObjectKey,
    upload_id: String,
    parts: Vec<(i32, String)>,
}

impl CompleteMultipartUploadBuilder {
    pub(crate) fn new(
        client: Arc<OSSClientInner>,
        bucket: BucketName,
        key: ObjectKey,
        upload_id: impl Into<String>,
    ) -> Self {
        Self {
            client,
            bucket,
            key,
            upload_id: upload_id.into(),
            parts: Vec::new(),
        }
    }

    pub fn part(mut self, part_number: i32, etag: impl Into<String>) -> Self {
        self.parts.push((part_number, etag.into()));
        self
    }

    pub async fn send(self) -> Result<CompleteMultipartUploadOutput> {
        if self.parts.is_empty() {
            return Err(OssError {
                kind: OssErrorKind::ValidationError,
                context: Box::new(ErrorContext {
                    operation: Some("CompleteMultipartUpload: at least one part required".into()),
                    bucket: Some(self.bucket.to_string()),
                    object_key: Some(self.key.to_string()),
                    ..Default::default()
                }),
                source: None,
            });
        }

        let endpoint = self.client.endpoint.clone();
        let uri = oss_endpoint_url(
            &endpoint,
            Some(self.bucket.as_str()),
            Some(self.key.as_str()),
        );
        let full_uri = format!("{}?uploadId={}", uri, self.upload_id);

        let query_params: Vec<(String, String)> = vec![("uploadId".into(), self.upload_id.clone())];

        let complete_xml = CompleteMultipartUpload {
            parts: self
                .parts
                .iter()
                .map(|(n, e)| UploadPartItem {
                    part_number: *n,
                    etag: e.clone(),
                })
                .collect(),
        };
        let body_xml = crate::util::xml::to_xml(&complete_xml)?;

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
                    operation: Some("CompleteMultipartUpload".into()),
                    bucket: Some(self.bucket.to_string()),
                    object_key: Some(self.key.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.is_success() {
            let request_id = response
                .headers
                .get("x-oss-request-id")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_string();

            let body_str = response.body_as_str().unwrap_or("");
            let result: CompleteMultipartUploadResult = crate::util::xml::from_xml(body_str)
                .map_err(|e| OssError {
                    kind: OssErrorKind::DeserializationError,
                    context: Box::new(ErrorContext {
                        operation: Some("CompleteMultipartUpload: parse XML".into()),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?;

            Ok(CompleteMultipartUploadOutput {
                request_id,
                bucket: result.bucket,
                key: result.key,
                location: result.location,
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
                    operation: Some("CompleteMultipartUpload".into()),
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
pub struct CompleteMultipartUploadOutput {
    pub request_id: String,
    pub bucket: String,
    pub key: String,
    pub location: String,
    pub etag: String,
}

pub struct ListMultipartUploadsBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    prefix: Option<String>,
    delimiter: Option<String>,
    max_uploads: Option<i32>,
    key_marker: Option<String>,
    upload_id_marker: Option<String>,
    encoding_type: Option<String>,
}

impl ListMultipartUploadsBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName) -> Self {
        Self {
            client,
            bucket,
            prefix: None,
            delimiter: None,
            max_uploads: None,
            key_marker: None,
            upload_id_marker: None,
            encoding_type: None,
        }
    }

    pub fn prefix(mut self, v: impl Into<String>) -> Self {
        self.prefix = Some(v.into());
        self
    }

    pub fn delimiter(mut self, v: impl Into<String>) -> Self {
        self.delimiter = Some(v.into());
        self
    }

    pub fn max_uploads(mut self, v: i32) -> Self {
        self.max_uploads = Some(v);
        self
    }

    pub fn key_marker(mut self, v: impl Into<String>) -> Self {
        self.key_marker = Some(v.into());
        self
    }

    pub fn upload_id_marker(mut self, v: impl Into<String>) -> Self {
        self.upload_id_marker = Some(v.into());
        self
    }

    pub fn encoding_type(mut self, v: impl Into<String>) -> Self {
        self.encoding_type = Some(v.into());
        self
    }

    pub async fn send(self) -> Result<ListMultipartUploadsOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = oss_endpoint_url(&endpoint, Some(self.bucket.as_str()), None);

        let mut query_pairs: Vec<(String, String)> = Vec::new();
        query_pairs.push(("uploads".into(), String::new()));
        if let Some(ref p) = self.prefix {
            query_pairs.push(("prefix".into(), crate::util::uri::uri_encode(p)));
        }
        if let Some(ref d) = self.delimiter {
            query_pairs.push(("delimiter".into(), d.clone()));
        }
        if let Some(mu) = self.max_uploads {
            query_pairs.push(("max-uploads".into(), mu.to_string()));
        }
        if let Some(ref km) = self.key_marker {
            query_pairs.push(("key-marker".into(), km.clone()));
        }
        if let Some(ref uim) = self.upload_id_marker {
            query_pairs.push(("upload-id-marker".into(), uim.clone()));
        }
        if let Some(ref et) = self.encoding_type {
            query_pairs.push(("encoding-type".into(), et.clone()));
        }

        let query_string: String = if query_pairs.is_empty() {
            String::new()
        } else {
            let parts: Vec<String> = query_pairs
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            format!("?{}", parts.join("&"))
        };
        let full_uri = format!("{}{}", uri, query_string);

        let request = HttpRequest::builder()
            .method(http::Method::GET)
            .uri(&full_uri)
            .build();

        let response = self
            .client
            .send_signed(request, Some(&self.bucket), query_pairs)
            .await
            .map_err(|e| OssError {
                kind: OssErrorKind::TransportError,
                context: Box::new(ErrorContext {
                    operation: Some("ListMultipartUploads".into()),
                    bucket: Some(self.bucket.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.is_success() {
            let body_str = response.body_as_str().unwrap_or("");
            let result: ListMultipartUploadsResult =
                crate::util::xml::from_xml(body_str).map_err(|e| OssError {
                    kind: OssErrorKind::DeserializationError,
                    context: Box::new(ErrorContext {
                        operation: Some("ListMultipartUploads: parse XML".into()),
                        bucket: Some(self.bucket.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?;

            Ok(ListMultipartUploadsOutput {
                bucket: result.bucket,
                uploads: result
                    .uploads
                    .into_iter()
                    .map(|u| MultipartUploadInfo {
                        key: u.key,
                        upload_id: u.upload_id,
                        initiated: u.initiated,
                    })
                    .collect(),
                is_truncated: result.is_truncated,
                next_key_marker: result.next_key_marker,
                next_upload_id_marker: result.next_upload_id_marker,
                max_uploads: result.max_uploads,
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
                    operation: Some("ListMultipartUploads".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct ListMultipartUploadsOutput {
    pub bucket: String,
    pub uploads: Vec<MultipartUploadInfo>,
    pub is_truncated: bool,
    pub next_key_marker: String,
    pub next_upload_id_marker: String,
    pub max_uploads: i32,
}

#[derive(Debug, Clone)]
pub struct MultipartUploadInfo {
    pub key: String,
    pub upload_id: String,
    pub initiated: String,
}

pub struct ListPartsBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    key: ObjectKey,
    upload_id: String,
    max_parts: Option<i32>,
    part_number_marker: Option<i32>,
    encoding_type: Option<String>,
}

impl ListPartsBuilder {
    pub(crate) fn new(
        client: Arc<OSSClientInner>,
        bucket: BucketName,
        key: ObjectKey,
        upload_id: impl Into<String>,
    ) -> Self {
        Self {
            client,
            bucket,
            key,
            upload_id: upload_id.into(),
            max_parts: None,
            part_number_marker: None,
            encoding_type: None,
        }
    }

    pub fn max_parts(mut self, v: i32) -> Self {
        self.max_parts = Some(v);
        self
    }

    pub fn part_number_marker(mut self, v: i32) -> Self {
        self.part_number_marker = Some(v);
        self
    }

    pub fn encoding_type(mut self, v: impl Into<String>) -> Self {
        self.encoding_type = Some(v.into());
        self
    }

    pub async fn send(self) -> Result<ListPartsOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = oss_endpoint_url(
            &endpoint,
            Some(self.bucket.as_str()),
            Some(self.key.as_str()),
        );
        let full_uri = format!("{}?uploadId={}", uri, self.upload_id);

        let mut query_pairs: Vec<(String, String)> = Vec::new();
        query_pairs.push(("uploadId".into(), self.upload_id.clone()));
        if let Some(mp) = self.max_parts {
            query_pairs.push(("max-parts".into(), mp.to_string()));
        }
        if let Some(pnm) = self.part_number_marker {
            query_pairs.push(("part-number-marker".into(), pnm.to_string()));
        }
        if let Some(ref et) = self.encoding_type {
            query_pairs.push(("encoding-type".into(), et.clone()));
        }

        let request = HttpRequest::builder()
            .method(http::Method::GET)
            .uri(&full_uri)
            .build();

        let response = self
            .client
            .send_signed(request, Some(&self.bucket), query_pairs)
            .await
            .map_err(|e| OssError {
                kind: OssErrorKind::TransportError,
                context: Box::new(ErrorContext {
                    operation: Some("ListParts".into()),
                    bucket: Some(self.bucket.to_string()),
                    object_key: Some(self.key.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.is_success() {
            let body_str = response.body_as_str().unwrap_or("");
            let result: ListPartsResult =
                crate::util::xml::from_xml(body_str).map_err(|e| OssError {
                    kind: OssErrorKind::DeserializationError,
                    context: Box::new(ErrorContext {
                        operation: Some("ListParts: parse XML".into()),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?;

            Ok(ListPartsOutput {
                bucket: result.bucket,
                key: result.key,
                upload_id: result.upload_id,
                parts: result
                    .parts
                    .into_iter()
                    .map(|p| PartInfo {
                        part_number: p.part_number,
                        last_modified: p.last_modified,
                        etag: p.etag.trim_matches('"').to_string(),
                        size: p.size,
                    })
                    .collect(),
                max_parts: result.max_parts,
                is_truncated: result.is_truncated,
                next_part_number_marker: result.next_part_number_marker,
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
                    operation: Some("ListParts".into()),
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
pub struct ListPartsOutput {
    pub bucket: String,
    pub key: String,
    pub upload_id: String,
    pub parts: Vec<PartInfo>,
    pub max_parts: i32,
    pub is_truncated: bool,
    pub next_part_number_marker: String,
}

#[derive(Debug, Clone)]
pub struct PartInfo {
    pub part_number: i32,
    pub last_modified: String,
    pub etag: String,
    pub size: u64,
}

pub struct UploadPartCopyBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    key: ObjectKey,
    upload_id: String,
    part_number: u32,
    copy_source: Option<String>,
    copy_source_range: Option<String>,
    copy_source_if_match: Option<String>,
    copy_source_if_none_match: Option<String>,
    copy_source_if_modified_since: Option<String>,
    copy_source_if_unmodified_since: Option<String>,
}

impl UploadPartCopyBuilder {
    pub(crate) fn new(
        client: Arc<OSSClientInner>,
        bucket: BucketName,
        key: ObjectKey,
        upload_id: impl Into<String>,
        part_number: u32,
    ) -> Self {
        Self {
            client,
            bucket,
            key,
            upload_id: upload_id.into(),
            part_number,
            copy_source: None,
            copy_source_range: None,
            copy_source_if_match: None,
            copy_source_if_none_match: None,
            copy_source_if_modified_since: None,
            copy_source_if_unmodified_since: None,
        }
    }

    pub fn copy_source(mut self, source: impl Into<String>) -> Self {
        self.copy_source = Some(source.into());
        self
    }

    pub fn copy_source_range(mut self, range: impl Into<String>) -> Self {
        self.copy_source_range = Some(range.into());
        self
    }

    pub fn copy_source_if_match(mut self, etag: impl Into<String>) -> Self {
        self.copy_source_if_match = Some(etag.into());
        self
    }

    pub fn copy_source_if_none_match(mut self, etag: impl Into<String>) -> Self {
        self.copy_source_if_none_match = Some(etag.into());
        self
    }

    pub fn copy_source_if_modified_since(mut self, time: impl Into<String>) -> Self {
        self.copy_source_if_modified_since = Some(time.into());
        self
    }

    pub fn copy_source_if_unmodified_since(mut self, time: impl Into<String>) -> Self {
        self.copy_source_if_unmodified_since = Some(time.into());
        self
    }

    pub async fn send(self) -> Result<UploadPartCopyOutput> {
        let copy_source = self.copy_source.ok_or_else(|| OssError {
            kind: OssErrorKind::ValidationError,
            context: Box::new(ErrorContext {
                operation: Some("UploadPartCopy: copy_source is required".into()),
                bucket: Some(self.bucket.to_string()),
                object_key: Some(self.key.to_string()),
                ..Default::default()
            }),
            source: None,
        })?;

        let endpoint = self.client.endpoint.clone();
        let uri = oss_endpoint_url(
            &endpoint,
            Some(self.bucket.as_str()),
            Some(self.key.as_str()),
        );
        let full_uri = format!(
            "{}?partNumber={}&uploadId={}",
            uri, self.part_number, self.upload_id
        );

        let query_params: Vec<(String, String)> = vec![
            ("partNumber".into(), self.part_number.to_string()),
            ("uploadId".into(), self.upload_id.clone()),
        ];

        let mut req = HttpRequest::builder()
            .method(http::Method::PUT)
            .uri(&full_uri);

        req = req.header(
            http::HeaderName::from_static("x-oss-copy-source"),
            http::HeaderValue::from_str(&copy_source).map_err(|e| OssError {
                kind: OssErrorKind::ValidationError,
                context: Box::new(ErrorContext {
                    operation: Some("set x-oss-copy-source header".into()),
                    bucket: Some(self.bucket.to_string()),
                    object_key: Some(self.key.to_string()),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?,
        );

        if let Some(ref r) = self.copy_source_range {
            req = req.header(
                http::HeaderName::from_static("x-oss-copy-source-range"),
                http::HeaderValue::from_str(r).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set x-oss-copy-source-range header".into()),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
        }

        if let Some(ref im) = self.copy_source_if_match {
            req = req.header(
                http::HeaderName::from_static("x-oss-copy-source-if-match"),
                http::HeaderValue::from_str(im).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set x-oss-copy-source-if-match header".into()),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
        }

        if let Some(ref inm) = self.copy_source_if_none_match {
            req = req.header(
                http::HeaderName::from_static("x-oss-copy-source-if-none-match"),
                http::HeaderValue::from_str(inm).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set x-oss-copy-source-if-none-match header".into()),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
        }

        if let Some(ref ims) = self.copy_source_if_modified_since {
            req = req.header(
                http::HeaderName::from_static("x-oss-copy-source-if-modified-since"),
                http::HeaderValue::from_str(ims).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set x-oss-copy-source-if-modified-since header".into()),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
        }

        if let Some(ref ius) = self.copy_source_if_unmodified_since {
            req = req.header(
                http::HeaderName::from_static("x-oss-copy-source-if-unmodified-since"),
                http::HeaderValue::from_str(ius).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set x-oss-copy-source-if-unmodified-since header".into()),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
        }

        let request = req.build();

        let response = self
            .client
            .send_signed(request, Some(&self.bucket), query_params)
            .await
            .map_err(|e| OssError {
                kind: OssErrorKind::TransportError,
                context: Box::new(ErrorContext {
                    operation: Some("UploadPartCopy".into()),
                    bucket: Some(self.bucket.to_string()),
                    object_key: Some(self.key.to_string()),
                    endpoint: Some(endpoint),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.is_success() {
            let request_id = response
                .headers
                .get("x-oss-request-id")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_string();

            let body_str = response.body_as_str().unwrap_or("");
            let result: CopyPartResult =
                crate::util::xml::from_xml(body_str).map_err(|e| OssError {
                    kind: OssErrorKind::DeserializationError,
                    context: Box::new(ErrorContext {
                        operation: Some("UploadPartCopy: parse XML".into()),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?;

            Ok(UploadPartCopyOutput {
                request_id,
                etag: result.etag.trim_matches('"').to_string(),
                part_number: self.part_number,
                last_modified: result.last_modified,
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
                    operation: Some("UploadPartCopy".into()),
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
pub struct UploadPartCopyOutput {
    pub request_id: String,
    pub etag: String,
    pub part_number: u32,
    pub last_modified: String,
}

impl BucketOperations {
    pub fn initiate_multipart_upload(
        &self,
        key: impl Into<String>,
    ) -> Result<InitiateMultipartUploadBuilder> {
        let object_key = ObjectKey::new(key.into())?;
        Ok(InitiateMultipartUploadBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            object_key,
        ))
    }

    pub fn upload_part(
        &self,
        key: impl Into<String>,
        upload_id: impl Into<String>,
        part_number: u32,
    ) -> Result<UploadPartBuilder> {
        let object_key = ObjectKey::new(key.into())?;
        Ok(UploadPartBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            object_key,
            upload_id,
            part_number,
        ))
    }

    pub fn upload_part_copy(
        &self,
        key: impl Into<String>,
        upload_id: impl Into<String>,
        part_number: u32,
    ) -> Result<UploadPartCopyBuilder> {
        let object_key = ObjectKey::new(key.into())?;
        Ok(UploadPartCopyBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            object_key,
            upload_id,
            part_number,
        ))
    }

    pub fn complete_multipart_upload(
        &self,
        key: impl Into<String>,
        upload_id: impl Into<String>,
    ) -> Result<CompleteMultipartUploadBuilder> {
        let object_key = ObjectKey::new(key.into())?;
        Ok(CompleteMultipartUploadBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            object_key,
            upload_id,
        ))
    }

    pub fn abort_multipart_upload(
        &self,
        key: impl Into<String>,
        upload_id: impl Into<String>,
    ) -> Result<AbortMultipartUploadBuilder> {
        let object_key = ObjectKey::new(key.into())?;
        Ok(AbortMultipartUploadBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            object_key,
            upload_id,
        ))
    }

    pub fn list_multipart_uploads(&self) -> ListMultipartUploadsBuilder {
        ListMultipartUploadsBuilder::new(self.client_inner().clone(), self.bucket_name().clone())
    }

    pub fn list_parts(
        &self,
        key: impl Into<String>,
        upload_id: impl Into<String>,
    ) -> Result<ListPartsBuilder> {
        let object_key = ObjectKey::new(key.into())?;
        Ok(ListPartsBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            object_key,
            upload_id,
        ))
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
        response_headers: Vec<(&'static str, &'static str)>,
    }

    #[async_trait::async_trait]
    impl HttpClient for RecordingHttpClient {
        async fn send(&self, request: HttpRequest) -> crate::error::Result<HttpResponse> {
            self.requests.lock().unwrap().push(request);
            let mut headers = http::HeaderMap::new();
            headers.insert(
                "x-oss-request-id",
                http::HeaderValue::from_static("rid-multipart"),
            );
            for (name, value) in &self.response_headers {
                if let (Ok(n), Ok(v)) = (
                    http::HeaderName::from_bytes(name.as_bytes()),
                    http::HeaderValue::from_str(value),
                ) {
                    headers.insert(n, v);
                }
            }
            Ok(HttpResponse {
                status: self.status_code,
                headers,
                body: self.response_body.clone(),
            })
        }
    }

    fn create_test_inner_with_response(
        status_code: http::StatusCode,
        response_body: bytes::Bytes,
        response_headers: Vec<(&'static str, &'static str)>,
    ) -> (Arc<OSSClientInner>, Arc<Mutex<Vec<HttpRequest>>>) {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let http = Arc::new(RecordingHttpClient {
            requests: requests.clone(),
            status_code,
            response_body,
            response_headers,
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
    fn initiate_multipart_builder_has_bucket_and_key() {
        let (inner, _) =
            create_test_inner_with_response(http::StatusCode::OK, bytes::Bytes::new(), vec![]);
        let _builder = InitiateMultipartUploadBuilder::new(
            inner,
            BucketName::new("test-bucket").unwrap(),
            ObjectKey::new("test-key.txt").unwrap(),
        );
    }

    #[tokio::test]
    async fn initiate_multipart_builder_sends_post_with_uploads_query() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<InitiateMultipartUploadResult>
  <Bucket>oss-bucket</Bucket>
  <Key>test-key.txt</Key>
  <UploadId>upload-id-123</UploadId>
</InitiateMultipartUploadResult>"#;
        let (inner, requests) =
            create_test_inner_with_response(http::StatusCode::OK, bytes::Bytes::from(xml), vec![]);
        let builder = InitiateMultipartUploadBuilder::new(
            inner,
            BucketName::new("test-bucket").unwrap(),
            ObjectKey::new("test-key.txt").unwrap(),
        );

        let output = builder.send().await.unwrap();
        assert_eq!(output.upload_id, "upload-id-123");
        assert_eq!(output.bucket, "oss-bucket");
        assert_eq!(output.key, "test-key.txt");

        let captured = requests.lock().unwrap();
        assert_eq!(captured[0].method, http::Method::POST);
        assert!(captured[0].uri.contains("?uploads"));
    }

    #[tokio::test]
    async fn initiate_multipart_builder_sets_optional_headers() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<InitiateMultipartUploadResult>
  <Bucket>b</Bucket>
  <Key>k</Key>
  <UploadId>uid</UploadId>
</InitiateMultipartUploadResult>"#;
        let (inner, requests) =
            create_test_inner_with_response(http::StatusCode::OK, bytes::Bytes::from(xml), vec![]);
        let builder = InitiateMultipartUploadBuilder::new(
            inner,
            BucketName::new("test-bucket").unwrap(),
            ObjectKey::new("test-key.txt").unwrap(),
        )
        .cache_control("max-age=300")
        .content_type("application/octet-stream")
        .acl(ObjectAcl::Private)
        .storage_class(StorageClass::Standard);

        builder.send().await.unwrap();

        let captured = requests.lock().unwrap();
        let has_header = |name: &str, val: &str| -> bool {
            captured[0]
                .headers
                .get(http::HeaderName::from_bytes(name.as_bytes()).unwrap())
                .map(|v| v.to_str().ok() == Some(val))
                .unwrap_or(false)
        };
        assert!(has_header("cache-control", "max-age=300"));
        assert!(has_header("content-type", "application/octet-stream"));
        assert!(has_header("x-oss-object-acl", "private"));
        assert!(has_header("x-oss-storage-class", "Standard"));
    }

    #[tokio::test]
    async fn initiate_multipart_returns_error_on_failure() {
        let (inner, _) = create_test_inner_with_response(
            http::StatusCode::BAD_REQUEST,
            bytes::Bytes::from(""),
            vec![],
        );
        let builder = InitiateMultipartUploadBuilder::new(
            inner,
            BucketName::new("test-bucket").unwrap(),
            ObjectKey::new("test-key.txt").unwrap(),
        );

        let result = builder.send().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[ignore = "requires valid OSS credentials"]
    async fn e2e_initiate_multipart_upload() {
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

        let key = format!("test-multipart-{}.bin", chrono::Utc::now().timestamp());
        let output = client
            .bucket(&bucket_str)
            .unwrap()
            .initiate_multipart_upload(&key)
            .unwrap()
            .send()
            .await
            .unwrap();

        assert!(!output.upload_id.is_empty());
        assert_eq!(output.key, key);
        eprintln!(
            "InitiateMultipartUpload: key={}, upload_id={}",
            output.key, output.upload_id
        );
    }

    #[tokio::test]
    async fn upload_part_builder_rejects_part_number_zero() {
        let (inner, _) =
            create_test_inner_with_response(http::StatusCode::OK, bytes::Bytes::new(), vec![]);
        let builder = UploadPartBuilder::new(
            inner,
            BucketName::new("test-bucket").unwrap(),
            ObjectKey::new("k").unwrap(),
            "upload-id",
            0,
        )
        .body(bytes::Bytes::from("data"));
        let result = builder.send().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn upload_part_builder_rejects_part_number_over_10000() {
        let (inner, _) =
            create_test_inner_with_response(http::StatusCode::OK, bytes::Bytes::new(), vec![]);
        let builder = UploadPartBuilder::new(
            inner,
            BucketName::new("test-bucket").unwrap(),
            ObjectKey::new("k").unwrap(),
            "upload-id",
            10001,
        )
        .body(bytes::Bytes::from("data"));
        let result = builder.send().await;
        assert!(result.is_err());
    }

    #[test]
    fn content_md5_computation_known_answer() {
        let body = b"hello world";
        let md5 = UploadPartBuilder::compute_md5(body);
        assert_eq!(md5, "XrY7u+Ae7tCTyyK7j1rNww==");
    }

    #[tokio::test]
    async fn upload_part_returns_etag_from_response() {
        let (inner, requests) = create_test_inner_with_response(
            http::StatusCode::OK,
            bytes::Bytes::new(),
            vec![("ETag", "\"abc123\"")],
        );
        let builder = UploadPartBuilder::new(
            inner,
            BucketName::new("test-bucket").unwrap(),
            ObjectKey::new("test-key.txt").unwrap(),
            "upload-123",
            1,
        )
        .body(bytes::Bytes::from("part data"));

        let output = builder.send().await.unwrap();
        assert_eq!(output.etag, "abc123");
        assert_eq!(output.part_number, 1);

        let captured = requests.lock().unwrap();
        assert_eq!(captured[0].method, http::Method::PUT);
        assert!(captured[0].uri.contains("partNumber=1"));
        assert!(captured[0].uri.contains("uploadId=upload-123"));
    }

    #[tokio::test]
    #[ignore = "requires valid OSS credentials"]
    async fn e2e_upload_part() {
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

        let key = format!("test-part-{}.bin", chrono::Utc::now().timestamp());
        let upload_id = client
            .bucket(&bucket_str)
            .unwrap()
            .initiate_multipart_upload(&key)
            .unwrap()
            .send()
            .await
            .unwrap()
            .upload_id;

        let output = client
            .bucket(&bucket_str)
            .unwrap()
            .upload_part(&key, &upload_id, 1)
            .unwrap()
            .body(bytes::Bytes::from("hello part"))
            .send()
            .await
            .unwrap();

        assert!(!output.etag.is_empty());
        eprintln!(
            "UploadPart: part_number={}, etag={}",
            output.part_number, output.etag
        );
    }

    #[tokio::test]
    async fn upload_part_copy_requires_copy_source() {
        let (inner, _) =
            create_test_inner_with_response(http::StatusCode::OK, bytes::Bytes::new(), vec![]);
        let builder = UploadPartCopyBuilder::new(
            inner,
            BucketName::new("test-bucket").unwrap(),
            ObjectKey::new("k").unwrap(),
            "upload-id",
            1,
        );
        let result = builder.send().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn upload_part_copy_parses_etag_from_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<CopyPartResult>
  <ETag>"abc123"</ETag>
  <LastModified>2024-01-01T00:00:00.000Z</LastModified>
</CopyPartResult>"#;
        let (inner, requests) =
            create_test_inner_with_response(http::StatusCode::OK, bytes::Bytes::from(xml), vec![]);
        let builder = UploadPartCopyBuilder::new(
            inner,
            BucketName::new("test-bucket").unwrap(),
            ObjectKey::new("dest-key.txt").unwrap(),
            "upload-123",
            1,
        )
        .copy_source("/src-bucket/src-key");

        let output = builder.send().await.unwrap();
        assert_eq!(output.etag, "abc123");
        assert_eq!(output.part_number, 1);

        let captured = requests.lock().unwrap();
        assert!(
            captured[0]
                .headers
                .get("x-oss-copy-source")
                .map(|v| v.to_str().ok() == Some("/src-bucket/src-key"))
                .unwrap_or(false)
        );
    }

    #[tokio::test]
    #[ignore = "requires valid OSS credentials"]
    async fn e2e_upload_part_copy() {
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

        let src_key = format!("test-pc-src-{}.txt", chrono::Utc::now().timestamp());
        let dst_key = format!("test-pc-dst-{}.bin", chrono::Utc::now().timestamp());

        client
            .bucket(&bucket_str)
            .unwrap()
            .put_object(&src_key)
            .unwrap()
            .body(bytes::Bytes::from("source content for copy"))
            .send()
            .await
            .unwrap();

        let upload_id = client
            .bucket(&bucket_str)
            .unwrap()
            .initiate_multipart_upload(&dst_key)
            .unwrap()
            .send()
            .await
            .unwrap()
            .upload_id;

        let copy_source = format!("/{}/{}", bucket_str, src_key);
        let output = client
            .bucket(&bucket_str)
            .unwrap()
            .upload_part_copy(&dst_key, &upload_id, 1)
            .unwrap()
            .copy_source(&copy_source)
            .send()
            .await
            .unwrap();

        assert!(!output.etag.is_empty());
        eprintln!("UploadPartCopy: etag={}", output.etag);
    }

    #[test]
    fn complete_multipart_xml_contains_parts() {
        let complete = CompleteMultipartUpload {
            parts: vec![
                UploadPartItem {
                    part_number: 1,
                    etag: "etag1".into(),
                },
                UploadPartItem {
                    part_number: 2,
                    etag: "etag2".into(),
                },
            ],
        };
        let xml = crate::util::xml::to_xml(&complete).unwrap();
        assert!(xml.contains("<PartNumber>1</PartNumber>"));
        assert!(xml.contains("<ETag>etag1</ETag>"));
        assert!(xml.contains("<PartNumber>2</PartNumber>"));
        assert!(xml.contains("<ETag>etag2</ETag>"));
    }

    #[tokio::test]
    async fn complete_multipart_requires_at_least_one_part() {
        let (inner, _) =
            create_test_inner_with_response(http::StatusCode::OK, bytes::Bytes::new(), vec![]);
        let builder = CompleteMultipartUploadBuilder::new(
            inner,
            BucketName::new("test-bucket").unwrap(),
            ObjectKey::new("k").unwrap(),
            "upload-id",
        );
        let result = builder.send().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn complete_multipart_parses_result_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<CompleteMultipartUploadResult>
  <Location>http://bucket.oss-cn-hangzhou.aliyuncs.com/key</Location>
  <Bucket>bucket</Bucket>
  <Key>key</Key>
  <ETag>"final-etag"</ETag>
</CompleteMultipartUploadResult>"#;
        let (inner, requests) =
            create_test_inner_with_response(http::StatusCode::OK, bytes::Bytes::from(xml), vec![]);
        let builder = CompleteMultipartUploadBuilder::new(
            inner,
            BucketName::new("test-bucket").unwrap(),
            ObjectKey::new("key").unwrap(),
            "upload-id",
        )
        .part(1, "etag1");

        let output = builder.send().await.unwrap();
        assert_eq!(output.etag, "final-etag");
        assert_eq!(output.bucket, "bucket");
        assert_eq!(output.key, "key");

        let captured = requests.lock().unwrap();
        assert_eq!(captured[0].method, http::Method::POST);
        assert!(captured[0].uri.contains("uploadId=upload-id"));
    }

    #[tokio::test]
    #[ignore = "requires valid OSS credentials"]
    async fn e2e_complete_multipart_upload() {
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

        let key = format!("test-complete-{}.bin", chrono::Utc::now().timestamp());
        let bucket = client.bucket(&bucket_str).unwrap();

        let upload_id = bucket
            .initiate_multipart_upload(&key)
            .unwrap()
            .send()
            .await
            .unwrap()
            .upload_id;

        let part = bucket
            .upload_part(&key, &upload_id, 1)
            .unwrap()
            .body(bytes::Bytes::from("hello"))
            .send()
            .await
            .unwrap();

        let output = bucket
            .complete_multipart_upload(&key, &upload_id)
            .unwrap()
            .part(1, &part.etag)
            .send()
            .await
            .unwrap();

        assert!(!output.etag.is_empty());
        assert_eq!(output.key, key);
        eprintln!(
            "CompleteMultipart: key={}, etag={}",
            output.key, output.etag
        );
    }

    #[tokio::test]
    async fn abort_multipart_sends_delete_request() {
        let (inner, requests) = create_test_inner_with_response(
            http::StatusCode::NO_CONTENT,
            bytes::Bytes::new(),
            vec![],
        );
        let builder = AbortMultipartUploadBuilder::new(
            inner,
            BucketName::new("test-bucket").unwrap(),
            ObjectKey::new("k").unwrap(),
            "upload-123",
        );

        let output = builder.send().await.unwrap();
        assert!(!output.request_id.is_empty());

        let captured = requests.lock().unwrap();
        assert_eq!(captured[0].method, http::Method::DELETE);
        assert!(captured[0].uri.contains("uploadId=upload-123"));
    }

    #[tokio::test]
    #[ignore = "requires valid OSS credentials"]
    async fn e2e_abort_multipart_upload() {
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

        let key = format!("test-abort-{}.bin", chrono::Utc::now().timestamp());
        let bucket = client.bucket(&bucket_str).unwrap();

        let upload_id = bucket
            .initiate_multipart_upload(&key)
            .unwrap()
            .send()
            .await
            .unwrap()
            .upload_id;

        let output = bucket
            .abort_multipart_upload(&key, &upload_id)
            .unwrap()
            .send()
            .await
            .unwrap();

        assert!(!output.request_id.is_empty());
        eprintln!("AbortMultipartUpload: key={}", key);
    }

    #[tokio::test]
    async fn list_multipart_uploads_parses_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ListMultipartUploadsResult>
  <Bucket>bucket</Bucket>
  <Upload>
    <Key>obj1</Key>
    <UploadId>upload-id-1</UploadId>
    <Initiated>2024-01-01T00:00:00.000Z</Initiated>
  </Upload>
  <IsTruncated>false</IsTruncated>
  <MaxUploads>100</MaxUploads>
</ListMultipartUploadsResult>"#;
        let (inner, _) =
            create_test_inner_with_response(http::StatusCode::OK, bytes::Bytes::from(xml), vec![]);
        let builder =
            ListMultipartUploadsBuilder::new(inner, BucketName::new("test-bucket").unwrap());
        let output = builder.send().await.unwrap();
        assert_eq!(output.uploads.len(), 1);
        assert_eq!(output.uploads[0].key, "obj1");
        assert!(!output.is_truncated);
    }

    #[tokio::test]
    async fn list_parts_parses_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ListPartsResult>
  <Bucket>bucket</Bucket>
  <Key>key</Key>
  <UploadId>upload-123</UploadId>
  <Part>
    <PartNumber>1</PartNumber>
    <LastModified>2024-01-01T00:00:00.000Z</LastModified>
    <ETag>"etag1"</ETag>
    <Size>1024</Size>
  </Part>
  <MaxParts>1000</MaxParts>
  <IsTruncated>false</IsTruncated>
</ListPartsResult>"#;
        let (inner, _) =
            create_test_inner_with_response(http::StatusCode::OK, bytes::Bytes::from(xml), vec![]);
        let builder = ListPartsBuilder::new(
            inner,
            BucketName::new("test-bucket").unwrap(),
            ObjectKey::new("key").unwrap(),
            "upload-123",
        );
        let output = builder.send().await.unwrap();
        assert_eq!(output.parts.len(), 1);
        assert_eq!(output.parts[0].part_number, 1);
        assert_eq!(output.parts[0].etag, "etag1");
    }
}
