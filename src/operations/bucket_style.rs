//! Image style operations.

use crate::client::{BucketOperations, OSSClientInner};
use crate::error::{ErrorContext, OssError, OssErrorKind, Result};
use crate::http::client::{HttpRequest, HttpResponse};
use crate::types::bucket::BucketName;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize)]
#[serde(rename = "Style")]
struct StyleConfig {
    #[serde(rename = "Content")]
    content: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "Style")]
struct StyleConfigResp {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Content")]
    content: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "StyleList")]
struct StyleListResp {
    #[serde(rename = "Style", default)]
    styles: Vec<StyleConfigResp>,
}

pub struct PutBucketStyleBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    name: String,
    content: String,
}
impl PutBucketStyleBuilder {
    pub(crate) fn new(
        c: Arc<OSSClientInner>,
        b: BucketName,
        n: impl Into<String>,
        ct: impl Into<String>,
    ) -> Self {
        Self {
            client: c,
            bucket: b,
            name: n.into(),
            content: ct.into(),
        }
    }
    pub async fn send(self) -> Result<PutBucketStyleOutput> {
        put_style_impl(self.client, self.bucket, self.name, self.content).await
    }
}
#[derive(Debug, Clone)]
pub struct PutBucketStyleOutput {
    pub request_id: String,
}

pub struct GetBucketStyleBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    name: String,
}
impl GetBucketStyleBuilder {
    pub(crate) fn new(c: Arc<OSSClientInner>, b: BucketName, n: impl Into<String>) -> Self {
        Self {
            client: c,
            bucket: b,
            name: n.into(),
        }
    }
    pub async fn send(self) -> Result<GetBucketStyleOutput> {
        get_style_impl(self.client, self.bucket, self.name).await
    }
}
#[derive(Debug, Clone)]
pub struct GetBucketStyleOutput {
    pub name: String,
    pub content: String,
}

pub struct ListBucketStyleBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
}
impl ListBucketStyleBuilder {
    pub(crate) fn new(c: Arc<OSSClientInner>, b: BucketName) -> Self {
        Self {
            client: c,
            bucket: b,
        }
    }
    pub async fn send(self) -> Result<ListBucketStyleOutput> {
        list_style_impl(self.client, self.bucket).await
    }
}
#[derive(Debug, Clone)]
pub struct ListBucketStyleOutput {
    pub styles: Vec<StyleEntry>,
}
#[derive(Debug, Clone)]
pub struct StyleEntry {
    pub name: String,
    pub content: String,
}

pub struct DeleteBucketStyleBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    name: String,
}
impl DeleteBucketStyleBuilder {
    pub(crate) fn new(c: Arc<OSSClientInner>, b: BucketName, n: impl Into<String>) -> Self {
        Self {
            client: c,
            bucket: b,
            name: n.into(),
        }
    }
    pub async fn send(self) -> Result<DeleteBucketStyleOutput> {
        delete_style_impl(self.client, self.bucket, self.name).await
    }
}
#[derive(Debug, Clone)]
pub struct DeleteBucketStyleOutput {
    pub request_id: String,
}

async fn put_style_impl(
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    name: String,
    content: String,
) -> Result<PutBucketStyleOutput> {
    let ep = client.endpoint.clone();
    let uri = format!(
        "https://{}.{}?style&styleName={}",
        bucket.as_str(),
        ep,
        name
    );
    let qp = vec![("style".into(), String::new()), ("styleName".into(), name)];
    let xml = crate::util::xml::to_xml(&StyleConfig { content })?;
    let req = HttpRequest::builder()
        .method(http::Method::PUT)
        .uri(&uri)
        .body(bytes::Bytes::from(xml))
        .build();
    let r = client
        .send_signed(req, Some(&bucket), qp)
        .await
        .map_err(|e| terr("PutBucketStyle", &bucket, &ep, e))?;
    if r.status().is_success() {
        Ok(PutBucketStyleOutput {
            request_id: rid(&r),
        })
    } else {
        Err(serr("PutBucketStyle", &bucket, r))
    }
}

async fn get_style_impl(
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    name: String,
) -> Result<GetBucketStyleOutput> {
    let ep = client.endpoint.clone();
    let uri = format!(
        "https://{}.{}?style&styleName={}",
        bucket.as_str(),
        ep,
        name
    );
    let qp = vec![("style".into(), String::new()), ("styleName".into(), name)];
    let req = HttpRequest::builder()
        .method(http::Method::GET)
        .uri(&uri)
        .build();
    let r = client
        .send_signed(req, Some(&bucket), qp)
        .await
        .map_err(|e| terr("GetBucketStyle", &bucket, &ep, e))?;
    if r.is_success() {
        let s: StyleConfigResp = crate::util::xml::from_xml(r.body_as_str().unwrap_or(""))
            .map_err(|e| OssError {
                kind: OssErrorKind::DeserializationError,
                context: Box::new(ErrorContext {
                    operation: Some("parse XML".into()),
                    bucket: Some(bucket.to_string()),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;
        Ok(GetBucketStyleOutput {
            name: s.name,
            content: s.content,
        })
    } else {
        Err(serr("GetBucketStyle", &bucket, r))
    }
}

async fn list_style_impl(
    client: Arc<OSSClientInner>,
    bucket: BucketName,
) -> Result<ListBucketStyleOutput> {
    let ep = client.endpoint.clone();
    let uri = format!("https://{}.{}?style", bucket.as_str(), ep);
    let qp = vec![("style".into(), String::new())];
    let req = HttpRequest::builder()
        .method(http::Method::GET)
        .uri(&uri)
        .build();
    let r = client
        .send_signed(req, Some(&bucket), qp)
        .await
        .map_err(|e| terr("ListBucketStyle", &bucket, &ep, e))?;
    if r.is_success() {
        let l: StyleListResp =
            crate::util::xml::from_xml(r.body_as_str().unwrap_or("")).map_err(|e| OssError {
                kind: OssErrorKind::DeserializationError,
                context: Box::new(ErrorContext {
                    operation: Some("parse XML".into()),
                    bucket: Some(bucket.to_string()),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;
        Ok(ListBucketStyleOutput {
            styles: l
                .styles
                .into_iter()
                .map(|s| StyleEntry {
                    name: s.name,
                    content: s.content,
                })
                .collect(),
        })
    } else {
        Err(serr("ListBucketStyle", &bucket, r))
    }
}

async fn delete_style_impl(
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    name: String,
) -> Result<DeleteBucketStyleOutput> {
    let ep = client.endpoint.clone();
    let uri = format!(
        "https://{}.{}?style&styleName={}",
        bucket.as_str(),
        ep,
        name
    );
    let qp = vec![("style".into(), String::new()), ("styleName".into(), name)];
    let req = HttpRequest::builder()
        .method(http::Method::DELETE)
        .uri(&uri)
        .build();
    let r = client
        .send_signed(req, Some(&bucket), qp)
        .await
        .map_err(|e| terr("DeleteBucketStyle", &bucket, &ep, e))?;
    if r.status().is_success() {
        Ok(DeleteBucketStyleOutput {
            request_id: rid(&r),
        })
    } else {
        Err(serr("DeleteBucketStyle", &bucket, r))
    }
}

fn rid(r: &HttpResponse) -> String {
    r.headers
        .get("x-oss-request-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string()
}
fn terr(op: &str, b: &BucketName, ep: &str, e: OssError) -> OssError {
    OssError {
        kind: OssErrorKind::TransportError,
        context: Box::new(ErrorContext {
            operation: Some(op.into()),
            bucket: Some(b.to_string()),
            endpoint: Some(ep.into()),
            ..Default::default()
        }),
        source: Some(Box::new(e)),
    }
}
fn serr(op: &str, b: &BucketName, r: HttpResponse) -> OssError {
    OssError {
        kind: OssErrorKind::ServiceError(Box::new(crate::error::OssServiceError {
            status_code: r.status().as_u16(),
            code: String::new(),
            message: String::new(),
            request_id: String::new(),
            host_id: String::new(),
            resource: Some(b.to_string()),
            string_to_sign: None,
        })),
        context: Box::new(ErrorContext {
            operation: Some(op.into()),
            bucket: Some(b.to_string()),
            ..Default::default()
        }),
        source: None,
    }
}

impl BucketOperations {
    pub fn put_style(
        &self,
        name: impl Into<String>,
        content: impl Into<String>,
    ) -> PutBucketStyleBuilder {
        PutBucketStyleBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            name,
            content,
        )
    }
    pub fn get_style(&self, name: impl Into<String>) -> GetBucketStyleBuilder {
        GetBucketStyleBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            name,
        )
    }
    pub fn list_style(&self) -> ListBucketStyleBuilder {
        ListBucketStyleBuilder::new(self.client_inner().clone(), self.bucket_name().clone())
    }
    pub fn delete_style(&self, name: impl Into<String>) -> DeleteBucketStyleBuilder {
        DeleteBucketStyleBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            name,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn style_xml() {
        let c = StyleConfig {
            content: "image/resize,m_fixed,w_100".into(),
        };
        let x = crate::util::xml::to_xml(&c).unwrap();
        assert!(x.contains("<Content>image/resize,m_fixed,w_100</Content>"));
    }
}
