//! HTTP client abstraction and Reqwest-based implementation.

use async_trait::async_trait;
use http::HeaderMap;

use crate::error::{ErrorContext, OssError, OssErrorKind, Result};

/// A fully-formed HTTP request ready to be sent.
#[derive(Debug)]
pub struct HttpRequest {
    pub method: http::Method,
    pub uri: String,
    pub headers: http::HeaderMap,
    pub body: Option<bytes::Bytes>,
}

impl HttpRequest {
    /// Creates a new `HttpRequestBuilder`.
    pub fn builder() -> HttpRequestBuilder {
        HttpRequestBuilder::default()
    }
}

/// Builder for constructing an `HttpRequest`.
#[derive(Default)]
pub struct HttpRequestBuilder {
    method: Option<http::Method>,
    uri: Option<String>,
    headers: http::HeaderMap,
    body: Option<bytes::Bytes>,
}

impl HttpRequestBuilder {
    /// Sets the HTTP method.
    pub fn method(mut self, method: http::Method) -> Self {
        self.method = Some(method);
        self
    }

    /// Sets the request URI.
    pub fn uri(mut self, uri: impl Into<String>) -> Self {
        self.uri = Some(uri.into());
        self
    }

    /// Adds a header to the request.
    pub fn header(mut self, key: http::HeaderName, value: http::HeaderValue) -> Self {
        self.headers.insert(key, value);
        self
    }

    /// Sets the request body.
    pub fn body(mut self, body: impl Into<bytes::Bytes>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// Builds the `HttpRequest`.
    pub fn build(self) -> HttpRequest {
        HttpRequest {
            method: self.method.unwrap_or(http::Method::GET),
            uri: self.uri.unwrap_or_default(),
            headers: self.headers,
            body: self.body,
        }
    }
}

/// An HTTP response containing status, headers, and body.
#[derive(Debug)]
pub struct HttpResponse {
    pub status: http::StatusCode,
    pub headers: http::HeaderMap,
    pub body: bytes::Bytes,
}

impl HttpResponse {
    /// Creates a new `HttpResponse` with the given status code.
    pub fn new(status: http::StatusCode) -> Self {
        Self {
            status,
            headers: HeaderMap::new(),
            body: bytes::Bytes::new(),
        }
    }

    /// Returns the HTTP status code.
    pub fn status(&self) -> http::StatusCode {
        self.status
    }

    /// Returns `true` if the status is in the 2xx range.
    pub fn is_success(&self) -> bool {
        self.status.is_success()
    }

    /// Returns the response body as a UTF-8 string, if valid.
    pub fn body_as_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.body).ok()
    }
}

/// Trait abstracting the HTTP transport layer.
#[async_trait]
pub trait HttpClient: Send + Sync {
    /// Sends an HTTP request and returns the response.
    async fn send(&self, request: HttpRequest) -> Result<HttpResponse>;
}

/// Reqwest-based implementation of `HttpClient`.
pub struct ReqwestHttpClient {
    inner: reqwest::Client,
}

impl ReqwestHttpClient {
    /// Creates a new `ReqwestHttpClient`.
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder().build().map_err(|e| OssError {
            kind: OssErrorKind::ConfigError,
            context: Box::new(ErrorContext {
                operation: Some("create ReqwestHttpClient".into()),
                ..Default::default()
            }),
            source: Some(Box::new(e)),
        })?;
        Ok(Self { inner: client })
    }
}

impl Default for ReqwestHttpClient {
    fn default() -> Self {
        Self::new().expect("create default ReqwestHttpClient")
    }
}

#[async_trait]
impl HttpClient for ReqwestHttpClient {
    async fn send(&self, request: HttpRequest) -> Result<HttpResponse> {
        let mut req = self.inner.request(request.method, &request.uri);

        for (name, value) in request.headers.iter() {
            req = req.header(name, value);
        }

        if let Some(body) = request.body {
            req = req.body(body);
        }

        let response = req.send().await.map_err(|e| OssError {
            kind: OssErrorKind::TransportError,
            context: Box::new(ErrorContext {
                operation: Some("send HTTP request".into()),
                ..Default::default()
            }),
            source: Some(Box::new(e)),
        })?;

        let status = response.status();
        let headers = response.headers().clone();
        let body = response.bytes().await.map_err(|e| OssError {
            kind: OssErrorKind::TransportError,
            context: Box::new(ErrorContext {
                operation: Some("read HTTP response body".into()),
                ..Default::default()
            }),
            source: Some(Box::new(e)),
        })?;

        Ok(HttpResponse {
            status,
            headers,
            body,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_request_builder_sets_method_and_uri() {
        let request = HttpRequest::builder()
            .method(http::Method::PUT)
            .uri("https://oss-cn-hangzhou.aliyuncs.com/bucket/key")
            .build();
        assert_eq!(request.method, http::Method::PUT);
        assert_eq!(
            request.uri,
            "https://oss-cn-hangzhou.aliyuncs.com/bucket/key"
        );
    }

    #[test]
    fn http_request_builder_sets_headers() {
        let request = HttpRequest::builder()
            .method(http::Method::GET)
            .uri("https://example.com")
            .header(
                http::HeaderName::from_static("content-type"),
                http::HeaderValue::from_static("text/plain"),
            )
            .build();
        assert_eq!(
            request
                .headers
                .get("content-type")
                .unwrap()
                .to_str()
                .unwrap(),
            "text/plain"
        );
    }

    #[test]
    fn http_request_builder_sets_body() {
        let body = bytes::Bytes::from_static(b"hello world");
        let request = HttpRequest::builder()
            .method(http::Method::POST)
            .uri("https://example.com")
            .body(body.clone())
            .build();
        assert_eq!(request.body.as_deref(), Some(b"hello world" as &[u8]));
    }

    #[test]
    fn http_request_builder_defaults() {
        let request = HttpRequest::builder().build();
        assert_eq!(request.method, http::Method::GET);
        assert!(request.body.is_none());
    }

    #[test]
    fn http_response_defaults() {
        let response = HttpResponse::new(http::StatusCode::OK);
        assert_eq!(response.status(), http::StatusCode::OK);
        assert!(response.is_success());
        assert!(response.body.is_empty());
    }

    #[test]
    fn http_response_not_success() {
        let response = HttpResponse::new(http::StatusCode::NOT_FOUND);
        assert!(!response.is_success());
    }

    #[test]
    fn http_client_trait_object_safe() {
        fn _use_client(_client: &dyn HttpClient) {}
    }

    #[tokio::test]
    async fn reqwest_client_send_get_request() {
        let client = ReqwestHttpClient::new().unwrap();
        let request = HttpRequest::builder()
            .method(http::Method::GET)
            .uri("https://httpbin.org/get")
            .build();
        let response = client.send(request).await.unwrap();
        assert!(response.is_success());
        assert_eq!(response.status(), http::StatusCode::OK);
    }

    #[test]
    fn http_request_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<HttpRequest>();
        assert_send_sync::<HttpResponse>();
    }
}
