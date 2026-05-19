use std::sync::Arc;

use crate::client::{BucketOperations, OSSClientInner};
use crate::error::{ErrorContext, OssError, OssErrorKind, Result};
use crate::http::client::HttpRequest;
use crate::types::acl::ObjectAcl;
use crate::types::bucket::BucketName;
use crate::types::object::ObjectKey;
use crate::types::storage::{ServerSideEncryption, StorageClass};
use crate::util::uri::oss_endpoint_url;

pub struct PutObjectBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    key: ObjectKey,
    body: Option<bytes::Bytes>,
    content_type: Option<String>,
    content_md5: Option<String>,
    cache_control: Option<String>,
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

impl PutObjectBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName, key: ObjectKey) -> Self {
        Self {
            client,
            bucket,
            key,
            body: None,
            content_type: None,
            content_md5: None,
            cache_control: None,
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

    pub fn body(mut self, body: impl Into<bytes::Bytes>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn content_type(mut self, ct: impl Into<String>) -> Self {
        self.content_type = Some(ct.into());
        self
    }

    pub fn content_md5(mut self, md5: impl Into<String>) -> Self {
        self.content_md5 = Some(md5.into());
        self
    }

    pub fn cache_control(mut self, cc: impl Into<String>) -> Self {
        self.cache_control = Some(cc.into());
        self
    }

    pub fn content_disposition(mut self, cd: impl Into<String>) -> Self {
        self.content_disposition = Some(cd.into());
        self
    }

    pub fn content_encoding(mut self, ce: impl Into<String>) -> Self {
        self.content_encoding = Some(ce.into());
        self
    }

    pub fn expires(mut self, exp: impl Into<String>) -> Self {
        self.expires = Some(exp.into());
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

    pub async fn send(self) -> Result<PutObjectOutput> {
        let body = self.body.ok_or_else(|| OssError {
            kind: OssErrorKind::ValidationError,
            context: Box::new(ErrorContext {
                operation: Some("PutObject: body is required".into()),
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

        let full_uri = uri;

        let mut req = HttpRequest::builder()
            .method(http::Method::PUT)
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

        if let Some(ref md5) = self.content_md5 {
            req = req.header(
                http::HeaderName::from_static("content-md5"),
                http::HeaderValue::from_str(md5).map_err(|e| OssError {
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

        let request = req.body(body).build();

        let response = self
            .client
            .send_signed(request, Some(&self.bucket), Vec::new())
            .await
            .map_err(|e| OssError {
                kind: OssErrorKind::TransportError,
                context: Box::new(ErrorContext {
                    operation: Some("PutObject".into()),
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

            let version_id = response
                .headers
                .get("x-oss-version-id")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            let hash_crc64 = response
                .headers
                .get("x-oss-hash-crc64ecma")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            let result_sse = response
                .headers
                .get("x-oss-server-side-encryption")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            Ok(PutObjectOutput {
                request_id,
                etag,
                version_id,
                hash_crc64,
                sse: result_sse,
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
                    operation: Some("PutObject".into()),
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
pub struct PutObjectOutput {
    pub request_id: String,
    pub etag: String,
    pub version_id: Option<String>,
    pub hash_crc64: Option<String>,
    pub sse: Option<String>,
}

impl BucketOperations {
    pub fn put_object(&self, key: impl Into<String>) -> Result<PutObjectBuilder> {
        let object_key = ObjectKey::new(key.into())?;
        Ok(PutObjectBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            object_key,
        ))
    }

    pub fn get_object(&self, key: impl Into<String>) -> Result<GetObjectBuilder> {
        let object_key = ObjectKey::new(key.into())?;
        Ok(GetObjectBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            object_key,
        ))
    }
}

pub struct GetObjectBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    key: ObjectKey,
    range: Option<String>,
    if_match: Option<String>,
    if_none_match: Option<String>,
    if_modified_since: Option<String>,
    if_unmodified_since: Option<String>,
    response_content_type: Option<String>,
    response_content_encoding: Option<String>,
    response_cache_control: Option<String>,
    response_content_disposition: Option<String>,
    response_content_language: Option<String>,
    response_expires: Option<String>,
    version_id: Option<String>,
    process: Option<String>,
}

impl GetObjectBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName, key: ObjectKey) -> Self {
        Self {
            client,
            bucket,
            key,
            range: None,
            if_match: None,
            if_none_match: None,
            if_modified_since: None,
            if_unmodified_since: None,
            response_content_type: None,
            response_content_encoding: None,
            response_cache_control: None,
            response_content_disposition: None,
            response_content_language: None,
            response_expires: None,
            version_id: None,
            process: None,
        }
    }

    pub fn range(mut self, range: impl Into<String>) -> Self {
        self.range = Some(range.into());
        self
    }

    pub fn if_match(mut self, etag: impl Into<String>) -> Self {
        self.if_match = Some(etag.into());
        self
    }

    pub fn if_none_match(mut self, etag: impl Into<String>) -> Self {
        self.if_none_match = Some(etag.into());
        self
    }

    pub fn if_modified_since(mut self, time: impl Into<String>) -> Self {
        self.if_modified_since = Some(time.into());
        self
    }

    pub fn if_unmodified_since(mut self, time: impl Into<String>) -> Self {
        self.if_unmodified_since = Some(time.into());
        self
    }

    pub fn response_content_type(mut self, ct: impl Into<String>) -> Self {
        self.response_content_type = Some(ct.into());
        self
    }

    pub fn response_content_encoding(mut self, ce: impl Into<String>) -> Self {
        self.response_content_encoding = Some(ce.into());
        self
    }

    pub fn response_cache_control(mut self, cc: impl Into<String>) -> Self {
        self.response_cache_control = Some(cc.into());
        self
    }

    pub fn response_content_disposition(mut self, cd: impl Into<String>) -> Self {
        self.response_content_disposition = Some(cd.into());
        self
    }

    pub fn response_content_language(mut self, cl: impl Into<String>) -> Self {
        self.response_content_language = Some(cl.into());
        self
    }

    pub fn response_expires(mut self, exp: impl Into<String>) -> Self {
        self.response_expires = Some(exp.into());
        self
    }

    pub fn process(mut self, process: impl Into<String>) -> Self {
        self.process = Some(process.into());
        self
    }

    pub fn version_id(mut self, id: impl Into<String>) -> Self {
        self.version_id = Some(id.into());
        self
    }

    pub async fn send(self) -> Result<GetObjectOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = oss_endpoint_url(
            &endpoint,
            Some(self.bucket.as_str()),
            Some(self.key.as_str()),
        );

        let mut query_pairs: Vec<(String, String)> = Vec::new();

        if let Some(ref ct) = self.response_content_type {
            query_pairs.push(("response-content-type".into(), ct.clone()));
        }
        if let Some(ref ce) = self.response_content_encoding {
            query_pairs.push(("response-content-encoding".into(), ce.clone()));
        }
        if let Some(ref cc) = self.response_cache_control {
            query_pairs.push(("response-cache-control".into(), cc.clone()));
        }
        if let Some(ref cd) = self.response_content_disposition {
            query_pairs.push(("response-content-disposition".into(), cd.clone()));
        }
        if let Some(ref cl) = self.response_content_language {
            query_pairs.push(("response-content-language".into(), cl.clone()));
        }
        if let Some(ref exp) = self.response_expires {
            query_pairs.push(("response-expires".into(), exp.clone()));
        }
        if let Some(ref vid) = self.version_id {
            query_pairs.push(("versionId".into(), vid.clone()));
        }
        if let Some(ref proc) = self.process {
            query_pairs.push(("x-oss-process".into(), proc.clone()));
        }

        let query_string = if query_pairs.is_empty() {
            String::new()
        } else {
            let parts: Vec<String> = query_pairs
                .iter()
                .map(|(k, v)| {
                    format!(
                        "{}={}",
                        crate::util::uri::uri_encode(k),
                        crate::util::uri::uri_encode(v)
                    )
                })
                .collect();
            format!("?{}", parts.join("&"))
        };

        let full_uri = format!("{}{}", uri, query_string);

        let mut req = HttpRequest::builder()
            .method(http::Method::GET)
            .uri(&full_uri);

        if let Some(ref range) = self.range {
            req = req.header(
                http::HeaderName::from_static("range"),
                http::HeaderValue::from_str(range).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set range header".into()),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
        }

        if let Some(ref im) = self.if_match {
            req = req.header(
                http::HeaderName::from_static("if-match"),
                http::HeaderValue::from_str(im).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set if-match header".into()),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
        }

        if let Some(ref inm) = self.if_none_match {
            req = req.header(
                http::HeaderName::from_static("if-none-match"),
                http::HeaderValue::from_str(inm).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set if-none-match header".into()),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
        }

        if let Some(ref ims) = self.if_modified_since {
            req = req.header(
                http::HeaderName::from_static("if-modified-since"),
                http::HeaderValue::from_str(ims).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set if-modified-since header".into()),
                        bucket: Some(self.bucket.to_string()),
                        object_key: Some(self.key.to_string()),
                        ..Default::default()
                    }),
                    source: Some(Box::new(e)),
                })?,
            );
        }

        if let Some(ref ius) = self.if_unmodified_since {
            req = req.header(
                http::HeaderName::from_static("if-unmodified-since"),
                http::HeaderValue::from_str(ius).map_err(|e| OssError {
                    kind: OssErrorKind::ValidationError,
                    context: Box::new(ErrorContext {
                        operation: Some("set if-unmodified-since header".into()),
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
            .send_signed(request, Some(&self.bucket), query_pairs)
            .await
            .map_err(|e| OssError {
                kind: OssErrorKind::TransportError,
                context: Box::new(ErrorContext {
                    operation: Some("GetObject".into()),
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

            let content_type = response
                .headers
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            let content_length = response
                .headers
                .get("content-length")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok());

            let etag = response
                .headers
                .get("ETag")
                .or_else(|| response.headers.get("etag"))
                .and_then(|v| v.to_str().ok())
                .map(|s| s.trim_matches('"').to_string());

            let last_modified = response
                .headers
                .get("last-modified")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            let storage_class = response
                .headers
                .get("x-oss-storage-class")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            let object_type = response
                .headers
                .get("x-oss-object-type")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            let mut metadata: Vec<(String, String)> = Vec::new();
            for (name, value) in response.headers.iter() {
                let name_str = name.as_str().to_lowercase();
                if name_str.starts_with("x-oss-meta-")
                    && let Ok(v) = value.to_str()
                {
                    metadata.push((name.as_str().to_string(), v.to_string()));
                }
            }

            Ok(GetObjectOutput {
                request_id,
                body: response.body,
                content_type,
                content_length,
                etag,
                last_modified,
                metadata,
                storage_class,
                object_type,
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
                    operation: Some("GetObject".into()),
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
pub struct GetObjectOutput {
    pub request_id: String,
    pub body: bytes::Bytes,
    pub content_type: Option<String>,
    pub content_length: Option<u64>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub metadata: Vec<(String, String)>,
    pub storage_class: Option<String>,
    pub object_type: Option<String>,
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use std::sync::Mutex;

    use http::HeaderMap;

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
            let mut headers = HeaderMap::new();
            headers.insert(
                "x-oss-request-id",
                http::HeaderValue::from_static("rid-001"),
            );
            headers.insert("ETag", http::HeaderValue::from_static("\"abc123\""));
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

    fn create_test_inner() -> (Arc<OSSClientInner>, Arc<Mutex<Vec<HttpRequest>>>) {
        create_test_inner_with_response(http::StatusCode::OK, bytes::Bytes::new(), vec![])
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

    fn test_bucket() -> BucketName {
        BucketName::new("test-bucket").unwrap()
    }

    #[test]
    fn bucket_operations_put_object_rejects_empty_key() {
        let (inner, _) = create_test_inner();
        let ops = BucketOperations {
            client: inner,
            bucket: test_bucket(),
        };
        assert!(ops.put_object("").is_err());
    }

    #[test]
    fn bucket_operations_put_object_rejects_overlength_key() {
        let (inner, _) = create_test_inner();
        let ops = BucketOperations {
            client: inner,
            bucket: test_bucket(),
        };
        assert!(ops.put_object("a".repeat(1025)).is_err());
    }

    #[test]
    fn bucket_operations_put_object_accepts_valid_key() {
        let (inner, _) = create_test_inner();
        let ops = BucketOperations {
            client: inner,
            bucket: test_bucket(),
        };
        assert!(ops.put_object("valid-key.txt").is_ok());
    }

    #[tokio::test]
    async fn put_object_sends_correct_request() {
        let (inner, requests) = create_test_inner();
        let bucket = BucketName::new("test-bucket").unwrap();
        let builder =
            PutObjectBuilder::new(inner, bucket, ObjectKey::new("test-file.txt").unwrap());

        let output = builder
            .body(bytes::Bytes::from_static(b"hello world"))
            .content_type("text/plain")
            .send()
            .await
            .unwrap();

        assert_eq!(output.request_id, "rid-001");
        assert_eq!(output.etag, "abc123");

        let captured = requests.lock().unwrap();
        assert_eq!(captured.len(), 1);
        assert_eq!(captured[0].method, http::Method::PUT);
        assert!(captured[0].uri.contains("test-bucket"));
        assert!(captured[0].uri.contains("test-file.txt"));

        let ct = captured[0]
            .headers
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(ct, "text/plain");

        assert_eq!(captured[0].body.as_deref(), Some(b"hello world" as &[u8]));
    }

    #[tokio::test]
    async fn put_object_with_custom_metadata() {
        let (inner, requests) = create_test_inner();
        let bucket = BucketName::new("test-bucket").unwrap();
        let builder = PutObjectBuilder::new(inner, bucket, ObjectKey::new("obj.txt").unwrap());

        builder
            .body(bytes::Bytes::from_static(b"data"))
            .metadata("x-oss-meta-author", "test-author")
            .metadata("x-oss-meta-version", "1.0")
            .send()
            .await
            .unwrap();

        let captured = requests.lock().unwrap();
        assert_eq!(
            captured[0]
                .headers
                .get("x-oss-meta-author")
                .unwrap()
                .to_str()
                .unwrap(),
            "test-author"
        );
        assert_eq!(
            captured[0]
                .headers
                .get("x-oss-meta-version")
                .unwrap()
                .to_str()
                .unwrap(),
            "1.0"
        );
    }

    #[tokio::test]
    async fn put_object_with_acl_and_storage_class() {
        let (inner, requests) = create_test_inner();
        let bucket = BucketName::new("test-bucket").unwrap();
        let builder = PutObjectBuilder::new(inner, bucket, ObjectKey::new("obj.txt").unwrap());

        builder
            .body(bytes::Bytes::from_static(b"data"))
            .acl(ObjectAcl::PublicRead)
            .storage_class(StorageClass::IA)
            .send()
            .await
            .unwrap();

        let captured = requests.lock().unwrap();
        assert_eq!(
            captured[0]
                .headers
                .get("x-oss-object-acl")
                .unwrap()
                .to_str()
                .unwrap(),
            "public-read"
        );
        assert_eq!(
            captured[0]
                .headers
                .get("x-oss-storage-class")
                .unwrap()
                .to_str()
                .unwrap(),
            "IA"
        );
    }

    #[tokio::test]
    async fn put_object_requires_body() {
        let (inner, _) = create_test_inner();
        let bucket = BucketName::new("test-bucket").unwrap();
        let builder = PutObjectBuilder::new(inner, bucket, ObjectKey::new("key").unwrap());

        let result = builder.send().await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err().kind,
            OssErrorKind::ValidationError
        ));
    }

    #[tokio::test]
    async fn put_object_uri_encodes_special_chars_in_key() {
        let (inner, requests) = create_test_inner();
        let bucket = BucketName::new("test-bucket").unwrap();
        let builder = PutObjectBuilder::new(inner, bucket, ObjectKey::new("文件 名.txt").unwrap());

        builder
            .body(bytes::Bytes::from_static(b"data"))
            .send()
            .await
            .unwrap();

        let captured = requests.lock().unwrap();
        assert!(
            captured[0]
                .uri
                .contains("%E6%96%87%E4%BB%B6%20%E5%90%8D.txt")
        );
    }

    #[tokio::test]
    async fn put_object_with_sse_aes256() {
        let (inner, requests) = create_test_inner();
        let bucket = BucketName::new("test-bucket").unwrap();
        let builder = PutObjectBuilder::new(inner, bucket, ObjectKey::new("key").unwrap());

        builder
            .body(bytes::Bytes::from_static(b"encrypted"))
            .server_side_encryption("AES256")
            .send()
            .await
            .unwrap();

        let captured = requests.lock().unwrap();
        assert_eq!(
            captured[0]
                .headers
                .get("x-oss-server-side-encryption")
                .unwrap()
                .to_str()
                .unwrap(),
            "AES256"
        );
    }

    #[tokio::test]
    async fn put_object_with_sse_kms_and_key_id() {
        let (inner, requests) = create_test_inner();
        let bucket = BucketName::new("test-bucket").unwrap();
        let builder = PutObjectBuilder::new(inner, bucket, ObjectKey::new("key").unwrap());

        builder
            .body(bytes::Bytes::from_static(b"encrypted"))
            .sse_key_id("cmk-id-123")
            .send()
            .await
            .unwrap();

        let captured = requests.lock().unwrap();
        assert_eq!(
            captured[0]
                .headers
                .get("x-oss-server-side-encryption")
                .unwrap()
                .to_str()
                .unwrap(),
            "KMS"
        );
        assert_eq!(
            captured[0]
                .headers
                .get("x-oss-server-side-encryption-key-id")
                .unwrap()
                .to_str()
                .unwrap(),
            "cmk-id-123"
        );
    }

    #[tokio::test]
    async fn put_object_with_tagging() {
        let (inner, requests) = create_test_inner();
        let bucket = BucketName::new("test-bucket").unwrap();
        let builder = PutObjectBuilder::new(inner, bucket, ObjectKey::new("key").unwrap());

        builder
            .body(bytes::Bytes::from_static(b"data"))
            .tagging("key1=value1&key2=value2")
            .send()
            .await
            .unwrap();

        let captured = requests.lock().unwrap();
        assert_eq!(
            captured[0]
                .headers
                .get("x-oss-tagging")
                .unwrap()
                .to_str()
                .unwrap(),
            "key1=value1&key2=value2"
        );
    }

    #[tokio::test]
    async fn put_object_content_md5_header() {
        let (inner, requests) = create_test_inner();
        let bucket = BucketName::new("test-bucket").unwrap();
        let builder = PutObjectBuilder::new(inner, bucket, ObjectKey::new("key").unwrap());

        builder
            .body(bytes::Bytes::from_static(b"data"))
            .content_md5("dGVzdC1tZDU=")
            .send()
            .await
            .unwrap();

        let captured = requests.lock().unwrap();
        assert_eq!(
            captured[0]
                .headers
                .get("content-md5")
                .unwrap()
                .to_str()
                .unwrap(),
            "dGVzdC1tZDU="
        );
    }

    #[tokio::test]
    async fn get_object_sends_correct_request() {
        let (inner, requests) = create_test_inner_with_response(
            http::StatusCode::OK,
            bytes::Bytes::from_static(b"file content"),
            vec![
                ("content-type", "text/plain"),
                ("content-length", "12"),
                ("last-modified", "Mon, 01 Jan 2024 00:00:00 GMT"),
            ],
        );
        let bucket = BucketName::new("test-bucket").unwrap();
        let builder = GetObjectBuilder::new(inner, bucket, ObjectKey::new("file.txt").unwrap());

        let output = builder.send().await.unwrap();

        assert_eq!(output.body.as_ref(), b"file content");
        assert_eq!(output.content_type.as_deref(), Some("text/plain"));
        assert_eq!(output.content_length, Some(12));
        assert_eq!(output.etag.as_deref(), Some("abc123"));

        let captured = requests.lock().unwrap();
        assert_eq!(captured.len(), 1);
        assert_eq!(captured[0].method, http::Method::GET);
        assert!(captured[0].uri.contains("test-bucket"));
        assert!(captured[0].uri.contains("file.txt"));
    }

    #[tokio::test]
    async fn get_object_returns_custom_metadata() {
        let (inner, _requests) = create_test_inner_with_response(
            http::StatusCode::OK,
            bytes::Bytes::from_static(b"data"),
            vec![
                ("content-type", "text/plain"),
                ("x-oss-meta-author", "echo"),
                ("x-oss-meta-version", "2.0"),
            ],
        );
        let bucket = BucketName::new("test-bucket").unwrap();
        let builder = GetObjectBuilder::new(inner, bucket, ObjectKey::new("obj.txt").unwrap());

        let output = builder.send().await.unwrap();

        let meta_author = output
            .metadata
            .iter()
            .find(|(k, _)| k.to_lowercase() == "x-oss-meta-author");
        assert!(meta_author.is_some());
        assert_eq!(meta_author.unwrap().1, "echo");

        let meta_version = output
            .metadata
            .iter()
            .find(|(k, _)| k.to_lowercase() == "x-oss-meta-version");
        assert!(meta_version.is_some());
        assert_eq!(meta_version.unwrap().1, "2.0");
    }

    #[tokio::test]
    async fn get_object_with_range_header() {
        let (inner, requests) = create_test_inner_with_response(
            http::StatusCode::PARTIAL_CONTENT,
            bytes::Bytes::from_static(b"abc"),
            vec![
                ("content-type", "text/plain"),
                ("content-length", "3"),
                ("content-range", "bytes 0-2/12"),
            ],
        );
        let bucket = BucketName::new("test-bucket").unwrap();
        let builder = GetObjectBuilder::new(inner, bucket, ObjectKey::new("file.txt").unwrap());

        let output = builder.range("bytes=0-2").send().await.unwrap();

        assert_eq!(output.body.as_ref(), b"abc");

        let captured = requests.lock().unwrap();
        assert_eq!(
            captured[0].headers.get("range").unwrap().to_str().unwrap(),
            "bytes=0-2"
        );
    }

    #[tokio::test]
    async fn get_object_with_response_header_overrides() {
        let (inner, requests) = create_test_inner_with_response(
            http::StatusCode::OK,
            bytes::Bytes::from_static(b"image data"),
            vec![("content-type", "image/jpeg")],
        );
        let bucket = BucketName::new("test-bucket").unwrap();
        let builder = GetObjectBuilder::new(inner, bucket, ObjectKey::new("img.jpg").unwrap());

        builder
            .response_content_type("image/png")
            .send()
            .await
            .unwrap();

        let captured = requests.lock().unwrap();
        assert!(captured[0].uri.contains("response-content-type=image/png"));
    }

    #[tokio::test]
    async fn get_object_with_version_id() {
        let (inner, requests) = create_test_inner_with_response(
            http::StatusCode::OK,
            bytes::Bytes::from_static(b"v2 data"),
            vec![("x-oss-version-id", "ver123")],
        );
        let bucket = BucketName::new("test-bucket").unwrap();
        let builder = GetObjectBuilder::new(inner, bucket, ObjectKey::new("file.txt").unwrap());

        builder.version_id("ver123").send().await.unwrap();

        let captured = requests.lock().unwrap();
        assert!(captured[0].uri.contains("versionId=ver123"));
    }

    #[tokio::test]
    async fn get_object_uri_encodes_key() {
        let (inner, requests) = create_test_inner_with_response(
            http::StatusCode::OK,
            bytes::Bytes::from_static(b"data"),
            vec![],
        );
        let bucket = BucketName::new("test-bucket").unwrap();
        let builder = GetObjectBuilder::new(inner, bucket, ObjectKey::new("文件 名.txt").unwrap());

        builder.send().await.unwrap();

        let captured = requests.lock().unwrap();
        assert!(
            captured[0]
                .uri
                .contains("%E6%96%87%E4%BB%B6%20%E5%90%8D.txt")
        );
    }

    #[tokio::test]
    async fn get_object_not_found_returns_error() {
        let (inner, _) = create_test_inner_with_response(
            http::StatusCode::NOT_FOUND,
            bytes::Bytes::new(),
            vec![],
        );
        let bucket = BucketName::new("test-bucket").unwrap();
        let builder = GetObjectBuilder::new(inner, bucket, ObjectKey::new("missing.txt").unwrap());

        let result = builder.send().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn get_object_returning_storage_class_and_object_type() {
        let (inner, _) = create_test_inner_with_response(
            http::StatusCode::OK,
            bytes::Bytes::from_static(b"data"),
            vec![
                ("x-oss-storage-class", "IA"),
                ("x-oss-object-type", "Normal"),
            ],
        );
        let bucket = BucketName::new("test-bucket").unwrap();
        let builder = GetObjectBuilder::new(inner, bucket, ObjectKey::new("obj.txt").unwrap());

        let output = builder.send().await.unwrap();
        assert_eq!(output.storage_class.as_deref(), Some("IA"));
        assert_eq!(output.object_type.as_deref(), Some("Normal"));
    }

    #[tokio::test]
    #[ignore = "requires valid OSS credentials in env vars OSS_ACCESS_KEY_ID, OSS_ACCESS_KEY_SECRET, OSS_REGION, OSS_BUCKET"]
    async fn e2e_get_object_real_oss() {
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

        let key = format!("test-get-object-{}.txt", chrono::Utc::now().timestamp());
        let content = "E2E GetObject test content";

        let _put = client
            .bucket(&bucket_str)
            .unwrap()
            .put_object(&key)
            .unwrap()
            .body(bytes::Bytes::from(content))
            .content_type("text/plain")
            .send()
            .await
            .unwrap();

        let output = client
            .bucket(&bucket_str)
            .unwrap()
            .get_object(&key)
            .unwrap()
            .send()
            .await
            .unwrap();

        assert_eq!(output.body.as_ref(), content.as_bytes());
        assert_eq!(output.content_type.as_deref(), Some("text/plain"));
        assert!(!output.etag.as_ref().unwrap().is_empty());

        eprintln!(
            "GET '{}' succeeded: body={} bytes, etag={}",
            key,
            output.body.len(),
            output.etag.unwrap_or_default()
        );
    }

    #[tokio::test]
    #[ignore = "requires valid OSS credentials"]
    async fn e2e_get_object_range() {
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

        let key = format!("test-range-{}.txt", chrono::Utc::now().timestamp());
        let content = "0123456789";

        let _put = client
            .bucket(&bucket_str)
            .unwrap()
            .put_object(&key)
            .unwrap()
            .body(bytes::Bytes::from(content))
            .send()
            .await
            .unwrap();

        let output = client
            .bucket(&bucket_str)
            .unwrap()
            .get_object(&key)
            .unwrap()
            .range("bytes=0-4")
            .send()
            .await
            .unwrap();

        assert_eq!(output.body.as_ref(), b"01234");

        eprintln!("GET range '{}' succeeded: {} bytes", key, output.body.len());
    }

    #[tokio::test]
    #[ignore = "requires valid OSS credentials"]
    async fn e2e_get_existing_test_file() {
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
            .get_object("test.txt")
            .unwrap()
            .send()
            .await
            .unwrap();

        assert!(!output.body.is_empty());
        eprintln!("GET 'test.txt' succeeded: {} bytes", output.body.len());
    }

    #[tokio::test]
    #[ignore = "requires valid OSS credentials"]
    async fn e2e_put_object_real_oss() {
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

        let key = format!("test-put-object-{}.txt", chrono::Utc::now().timestamp());
        let content = "Hello from aliyun-oss SDK E2E test";

        let output = client
            .bucket(&bucket_str)
            .unwrap()
            .put_object(&key)
            .unwrap()
            .body(bytes::Bytes::from(content))
            .content_type("text/plain")
            .send()
            .await
            .unwrap();

        assert!(!output.request_id.is_empty());
        assert!(!output.etag.is_empty());

        eprintln!(
            "PUT '{}' succeeded: request_id={}, etag={}",
            key, output.request_id, output.etag
        );
    }
}
