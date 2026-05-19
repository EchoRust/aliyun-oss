//! HTTP method and status code wrappers.

use std::fmt;
use std::str::FromStr;

use crate::error::{OssError, OssErrorKind};

/// Supported HTTP methods for OSS API requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Put,
    Post,
    Delete,
    Head,
    Options,
}

impl HttpMethod {
    /// Returns the uppercase HTTP method string (e.g. "GET").
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Put => "PUT",
            Self::Post => "POST",
            Self::Delete => "DELETE",
            Self::Head => "HEAD",
            Self::Options => "OPTIONS",
        }
    }
}

impl FromStr for HttpMethod {
    type Err = OssError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Self::Get),
            "PUT" => Ok(Self::Put),
            "POST" => Ok(Self::Post),
            "DELETE" => Ok(Self::Delete),
            "HEAD" => Ok(Self::Head),
            "OPTIONS" => Ok(Self::Options),
            other => Err(OssError {
                kind: OssErrorKind::ValidationError,
                context: Box::new(crate::error::ErrorContext {
                    operation: Some(format!("parse HttpMethod from '{}'", other)),
                    ..Default::default()
                }),
                source: None,
            }),
        }
    }
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A collection of HTTP headers backed by `http::HeaderMap`.
pub struct HttpHeaders {
    pub(crate) inner: http::HeaderMap,
}

impl HttpHeaders {
    /// Creates an empty `HttpHeaders` collection.
    pub fn new() -> Self {
        Self {
            inner: http::HeaderMap::new(),
        }
    }

    /// Inserts a header name/value pair, returning any old value.
    pub fn insert<K, V>(&mut self, key: K, value: V) -> Option<http::HeaderValue>
    where
        K: TryInto<http::HeaderName>,
        K::Error: fmt::Debug,
        V: TryInto<http::HeaderValue>,
        V::Error: fmt::Debug,
    {
        let name = key.try_into().expect("valid header name");
        let val = value.try_into().expect("valid header value");
        self.inner.insert(name, val)
    }

    /// Gets a header value by name (case-insensitive).
    pub fn get(&self, key: impl AsRef<str>) -> Option<&http::HeaderValue> {
        self.inner.get(key.as_ref())
    }

    /// Returns `true` if the header collection contains the given key.
    pub fn contains_key(&self, key: impl AsRef<str>) -> bool {
        self.inner.contains_key(key.as_ref())
    }

    /// Returns `true` if the header collection is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl Default for HttpHeaders {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for HttpHeaders {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(self.inner.iter().map(|(k, v)| (k.as_str(), v)))
            .finish()
    }
}

/// A MIME content type for HTTP requests and responses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentType {
    inner: String,
}

impl ContentType {
    /// Common content type constants.
    pub const TEXT_PLAIN: &'static str = "text/plain";
    pub const TEXT_HTML: &'static str = "text/html";
    pub const TEXT_CSS: &'static str = "text/css";
    pub const TEXT_JAVASCRIPT: &'static str = "text/javascript";
    pub const APPLICATION_JSON: &'static str = "application/json";
    pub const APPLICATION_XML: &'static str = "application/xml";
    pub const APPLICATION_OCTET_STREAM: &'static str = "application/octet-stream";
    pub const APPLICATION_PDF: &'static str = "application/pdf";
    pub const IMAGE_JPEG: &'static str = "image/jpeg";
    pub const IMAGE_PNG: &'static str = "image/png";
    pub const IMAGE_GIF: &'static str = "image/gif";
    pub const IMAGE_WEBP: &'static str = "image/webp";
    pub const IMAGE_SVG: &'static str = "image/svg+xml";
    pub const VIDEO_MP4: &'static str = "video/mp4";
    pub const AUDIO_MPEG: &'static str = "audio/mpeg";

    /// Resolves a content type from a file extension.
    pub fn from_extension(ext: &str) -> Self {
        let mime = match ext.to_lowercase().as_str() {
            "html" | "htm" => Self::TEXT_HTML,
            "css" => Self::TEXT_CSS,
            "js" | "mjs" => Self::TEXT_JAVASCRIPT,
            "json" => Self::APPLICATION_JSON,
            "xml" => Self::APPLICATION_XML,
            "pdf" => Self::APPLICATION_PDF,
            "jpg" | "jpeg" => Self::IMAGE_JPEG,
            "png" => Self::IMAGE_PNG,
            "gif" => Self::IMAGE_GIF,
            "webp" => Self::IMAGE_WEBP,
            "svg" => Self::IMAGE_SVG,
            "mp4" => Self::VIDEO_MP4,
            "mp3" => Self::AUDIO_MPEG,
            "txt" | "text" => Self::TEXT_PLAIN,
            _ => Self::APPLICATION_OCTET_STREAM,
        };
        ContentType {
            inner: mime.to_string(),
        }
    }

    /// Returns the MIME type string (e.g. "application/json").
    pub fn as_str(&self) -> &str {
        &self.inner
    }
}

impl Default for ContentType {
    fn default() -> Self {
        ContentType {
            inner: Self::APPLICATION_OCTET_STREAM.to_string(),
        }
    }
}

impl From<&str> for ContentType {
    fn from(s: &str) -> Self {
        ContentType {
            inner: s.to_string(),
        }
    }
}

impl From<String> for ContentType {
    fn from(s: String) -> Self {
        ContentType { inner: s }
    }
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_method_converts_to_str_correctly() {
        assert_eq!(HttpMethod::Get.as_str(), "GET");
        assert_eq!(HttpMethod::Put.as_str(), "PUT");
        assert_eq!(HttpMethod::Post.as_str(), "POST");
        assert_eq!(HttpMethod::Delete.as_str(), "DELETE");
        assert_eq!(HttpMethod::Head.as_str(), "HEAD");
        assert_eq!(HttpMethod::Options.as_str(), "OPTIONS");
    }

    #[test]
    fn http_method_from_str_case_sensitive() {
        assert_eq!("GET".parse::<HttpMethod>().unwrap(), HttpMethod::Get);
        assert!("get".parse::<HttpMethod>().is_err());
        assert!("Put".parse::<HttpMethod>().is_err());
    }

    #[test]
    fn http_method_display_matches_as_str() {
        assert_eq!(HttpMethod::Put.to_string(), "PUT");
        assert_eq!(HttpMethod::Head.to_string(), "HEAD");
    }

    #[test]
    fn http_headers_case_insensitive_key() {
        let mut headers = HttpHeaders::new();
        headers.insert("content-type", "text/plain");
        assert_eq!(
            headers.get("Content-Type").unwrap().to_str().unwrap(),
            "text/plain"
        );
        assert_eq!(
            headers.get("CONTENT-TYPE").unwrap().to_str().unwrap(),
            "text/plain"
        );
        assert_eq!(
            headers.get("content-type").unwrap().to_str().unwrap(),
            "text/plain"
        );
    }

    #[test]
    fn http_headers_insert_and_get() {
        let mut headers = HttpHeaders::new();
        headers.insert("x-oss-request-id", "abc123");
        assert_eq!(
            headers.get("x-oss-request-id").unwrap().to_str().unwrap(),
            "abc123"
        );
        assert!(headers.contains_key("x-oss-request-id"));
        assert!(!headers.contains_key("x-oss-nonexistent"));
    }

    #[test]
    fn content_type_from_file_extension() {
        assert_eq!(
            ContentType::from_extension("json").as_str(),
            "application/json"
        );
        assert_eq!(
            ContentType::from_extension("xml").as_str(),
            "application/xml"
        );
        assert_eq!(ContentType::from_extension("jpg").as_str(), "image/jpeg");
        assert_eq!(ContentType::from_extension("jpeg").as_str(), "image/jpeg");
        assert_eq!(ContentType::from_extension("png").as_str(), "image/png");
        assert_eq!(ContentType::from_extension("html").as_str(), "text/html");
        assert_eq!(ContentType::from_extension("txt").as_str(), "text/plain");
    }

    #[test]
    fn content_type_from_unknown_extension_defaults_to_octet_stream() {
        assert_eq!(
            ContentType::from_extension("xyz").as_str(),
            "application/octet-stream"
        );
    }

    #[test]
    fn content_type_default_is_application_octet_stream() {
        assert_eq!(ContentType::default().as_str(), "application/octet-stream");
    }

    #[test]
    fn content_type_from_str() {
        let ct: ContentType = "image/webp".into();
        assert_eq!(ct.as_str(), "image/webp");
    }

    #[test]
    fn content_type_case_insensitive_extension() {
        assert_eq!(
            ContentType::from_extension("JSON").as_str(),
            "application/json"
        );
        assert_eq!(ContentType::from_extension("Jpg").as_str(), "image/jpeg");
    }
}
