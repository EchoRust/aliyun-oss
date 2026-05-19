use std::sync::Arc;

use crate::client::{BucketOperations, OSSClientInner};
use crate::error::{ErrorContext, OssError, OssErrorKind, Result};
use crate::http::client::HttpRequest;
use crate::types::bucket::BucketName;
use crate::util::uri::oss_endpoint_url;

pub struct ListObjectsBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    prefix: Option<String>,
    delimiter: Option<String>,
    marker: Option<String>,
    max_keys: Option<i32>,
    encoding_type: Option<String>,
}

impl ListObjectsBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName) -> Self {
        Self {
            client,
            bucket,
            prefix: None,
            delimiter: None,
            marker: None,
            max_keys: None,
            encoding_type: None,
        }
    }

    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    pub fn delimiter(mut self, delimiter: impl Into<String>) -> Self {
        self.delimiter = Some(delimiter.into());
        self
    }

    pub fn marker(mut self, marker: impl Into<String>) -> Self {
        self.marker = Some(marker.into());
        self
    }

    pub fn max_keys(mut self, max_keys: i32) -> Self {
        self.max_keys = Some(max_keys);
        self
    }

    pub fn encoding_type(mut self, et: impl Into<String>) -> Self {
        self.encoding_type = Some(et.into());
        self
    }

    pub async fn send(self) -> Result<crate::types::response::ListObjectsOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = oss_endpoint_url(&endpoint, Some(self.bucket.as_str()), None);

        let mut query_pairs: Vec<(String, String)> = Vec::new();
        if let Some(ref p) = self.prefix {
            query_pairs.push(("prefix".into(), crate::util::uri::uri_encode(p)));
        }
        if let Some(ref d) = self.delimiter {
            query_pairs.push(("delimiter".into(), d.clone()));
        }
        if let Some(ref m) = self.marker {
            query_pairs.push(("marker".into(), m.clone()));
        }
        if let Some(mk) = self.max_keys {
            query_pairs.push(("max-keys".into(), mk.to_string()));
        }
        if let Some(ref et) = self.encoding_type {
            query_pairs.push(("encoding-type".into(), et.clone()));
        }

        let query_string = if query_pairs.is_empty() {
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
                    operation: Some("ListObjects".into()),
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
                    operation: Some("ListObjects".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

pub struct ListObjectsV2Builder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    prefix: Option<String>,
    delimiter: Option<String>,
    start_after: Option<String>,
    continuation_token: Option<String>,
    max_keys: Option<i32>,
    encoding_type: Option<String>,
    fetch_owner: Option<bool>,
}

impl ListObjectsV2Builder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName) -> Self {
        Self {
            client,
            bucket,
            prefix: None,
            delimiter: None,
            start_after: None,
            continuation_token: None,
            max_keys: None,
            encoding_type: None,
            fetch_owner: None,
        }
    }

    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    pub fn delimiter(mut self, delimiter: impl Into<String>) -> Self {
        self.delimiter = Some(delimiter.into());
        self
    }

    pub fn start_after(mut self, start_after: impl Into<String>) -> Self {
        self.start_after = Some(start_after.into());
        self
    }

    pub fn continuation_token(mut self, token: impl Into<String>) -> Self {
        self.continuation_token = Some(token.into());
        self
    }

    pub fn max_keys(mut self, max_keys: i32) -> Self {
        self.max_keys = Some(max_keys);
        self
    }

    pub fn encoding_type(mut self, et: impl Into<String>) -> Self {
        self.encoding_type = Some(et.into());
        self
    }

    pub fn fetch_owner(mut self, fetch: bool) -> Self {
        self.fetch_owner = Some(fetch);
        self
    }

    pub async fn send(self) -> Result<crate::types::response::ListObjectsV2Output> {
        let endpoint = self.client.endpoint.clone();
        let uri = oss_endpoint_url(&endpoint, Some(self.bucket.as_str()), None);

        let mut query_pairs: Vec<(String, String)> = Vec::new();
        query_pairs.push(("list-type".into(), "2".into()));
        if let Some(ref p) = self.prefix {
            query_pairs.push(("prefix".into(), crate::util::uri::uri_encode(p)));
        }
        if let Some(ref d) = self.delimiter {
            query_pairs.push(("delimiter".into(), d.clone()));
        }
        if let Some(ref s) = self.start_after {
            query_pairs.push(("start-after".into(), s.clone()));
        }
        if let Some(ref ct) = self.continuation_token {
            query_pairs.push(("continuation-token".into(), ct.clone()));
        }
        if let Some(mk) = self.max_keys {
            query_pairs.push(("max-keys".into(), mk.to_string()));
        }
        if let Some(ref et) = self.encoding_type {
            query_pairs.push(("encoding-type".into(), et.clone()));
        }
        if let Some(fo) = self.fetch_owner {
            query_pairs.push(("fetch-owner".into(), fo.to_string()));
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
                    operation: Some("ListObjectsV2".into()),
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
                    operation: Some("ListObjectsV2".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

pub struct ListObjectVersionsBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    prefix: Option<String>,
    delimiter: Option<String>,
    key_marker: Option<String>,
    version_id_marker: Option<String>,
    max_keys: Option<i32>,
    encoding_type: Option<String>,
}

impl ListObjectVersionsBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName) -> Self {
        Self {
            client,
            bucket,
            prefix: None,
            delimiter: None,
            key_marker: None,
            version_id_marker: None,
            max_keys: None,
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

    pub fn key_marker(mut self, v: impl Into<String>) -> Self {
        self.key_marker = Some(v.into());
        self
    }

    pub fn version_id_marker(mut self, v: impl Into<String>) -> Self {
        self.version_id_marker = Some(v.into());
        self
    }

    pub fn max_keys(mut self, v: i32) -> Self {
        self.max_keys = Some(v);
        self
    }

    pub fn encoding_type(mut self, v: impl Into<String>) -> Self {
        self.encoding_type = Some(v.into());
        self
    }

    pub async fn send(self) -> Result<crate::types::response::ListVersionsOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = oss_endpoint_url(&endpoint, Some(self.bucket.as_str()), None);

        let mut query_pairs: Vec<(String, String)> = Vec::new();
        query_pairs.push(("versions".into(), String::new()));
        if let Some(ref p) = self.prefix {
            query_pairs.push(("prefix".into(), crate::util::uri::uri_encode(p)));
        }
        if let Some(ref d) = self.delimiter {
            query_pairs.push(("delimiter".into(), d.clone()));
        }
        if let Some(ref km) = self.key_marker {
            query_pairs.push(("key-marker".into(), km.clone()));
        }
        if let Some(ref vim) = self.version_id_marker {
            query_pairs.push(("version-id-marker".into(), vim.clone()));
        }
        if let Some(mk) = self.max_keys {
            query_pairs.push(("max-keys".into(), mk.to_string()));
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
                    operation: Some("ListObjectVersions".into()),
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
                    operation: Some("ListObjectVersions".into()),
                    bucket: Some(self.bucket.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

impl BucketOperations {
    pub fn list_objects(&self) -> ListObjectsBuilder {
        ListObjectsBuilder::new(self.client_inner().clone(), self.bucket_name().clone())
    }

    pub fn list_objects_v2(&self) -> ListObjectsV2Builder {
        ListObjectsV2Builder::new(self.client_inner().clone(), self.bucket_name().clone())
    }

    pub fn list_object_versions(&self) -> ListObjectVersionsBuilder {
        ListObjectVersionsBuilder::new(self.client_inner().clone(), self.bucket_name().clone())
    }
}
