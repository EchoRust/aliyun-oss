use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OssServiceError {
    pub status_code: u16,
    pub code: String,
    pub message: String,
    pub request_id: String,
    pub host_id: String,
    pub resource: Option<String>,
    pub string_to_sign: Option<String>,
}

impl OssServiceError {
    pub fn from_xml(xml: &str) -> Option<Self> {
        Some(OssServiceError {
            status_code: 0,
            code: extract_xml_tag(xml, "Code")?,
            message: extract_xml_tag(xml, "Message")?,
            request_id: extract_xml_tag(xml, "RequestId")?,
            host_id: extract_xml_tag(xml, "HostId")?,
            resource: extract_xml_tag(xml, "BucketName"),
            string_to_sign: extract_xml_tag(xml, "StringToSign"),
        })
    }
}

impl fmt::Display for OssServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

fn extract_xml_tag(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)?;
    Some(xml[start..start + end].trim().to_string())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OssErrorKind {
    ServiceError(OssServiceError),
    ConfigError,
    SigningError,
    TransportError,
    IoError,
    XmlError,
    CredentialsError,
    ValidationError,
    TimeoutError,
    RetryExhausted,
    DeserializationError,
    Unknown,
}

impl fmt::Display for OssErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ServiceError(e) => write!(f, "[ServiceError] {}", e),
            Self::ConfigError => write!(f, "[ConfigError] configuration error"),
            Self::SigningError => write!(f, "[SigningError] signing failed"),
            Self::TransportError => write!(f, "[TransportError] transport error"),
            Self::IoError => write!(f, "[IoError] I/O error"),
            Self::XmlError => write!(f, "[XmlError] XML parsing error"),
            Self::CredentialsError => write!(f, "[CredentialsError] credentials error"),
            Self::ValidationError => write!(f, "[ValidationError] validation error"),
            Self::TimeoutError => write!(f, "[TimeoutError] timeout"),
            Self::RetryExhausted => write!(f, "[RetryExhausted] retry exhausted"),
            Self::DeserializationError => write!(f, "[DeserializationError] deserialization error"),
            Self::Unknown => write!(f, "[Unknown] unknown error"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ErrorContext {
    pub operation: Option<String>,
    pub bucket: Option<String>,
    pub object_key: Option<String>,
    pub request_id: Option<String>,
    pub endpoint: Option<String>,
}

impl ErrorContext {
    fn is_empty(&self) -> bool {
        self.operation.is_none()
            && self.bucket.is_none()
            && self.object_key.is_none()
            && self.request_id.is_none()
            && self.endpoint.is_none()
    }
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if let Some(op) = &self.operation {
            parts.push(format!("operation={}", op));
        }
        if let Some(b) = &self.bucket {
            parts.push(format!("bucket={}", b));
        }
        if let Some(k) = &self.object_key {
            parts.push(format!("key={}", k));
        }
        if let Some(id) = &self.request_id {
            parts.push(format!("request_id={}", id));
        }
        write!(f, "{}", parts.join(", "))
    }
}

#[derive(Debug)]
pub struct OssError {
    pub kind: OssErrorKind,
    pub context: ErrorContext,
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl OssError {
    pub fn service(err: OssServiceError) -> Self {
        Self {
            kind: OssErrorKind::ServiceError(err),
            context: ErrorContext::default(),
            source: None,
        }
    }

    pub fn validation(
        op: impl Into<String>,
        bucket: impl Into<String>,
        key: impl Into<String>,
    ) -> Self {
        Self {
            kind: OssErrorKind::ValidationError,
            context: ErrorContext {
                operation: Some(op.into()),
                bucket: Some(bucket.into()),
                object_key: Some(key.into()),
                request_id: None,
                endpoint: None,
            },
            source: None,
        }
    }

    pub fn transport(source: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self {
            kind: OssErrorKind::TransportError,
            context: ErrorContext::default(),
            source: Some(Box::new(source)),
        }
    }
}

impl fmt::Display for OssError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)?;
        if !self.context.is_empty() {
            write!(f, " ({})", self.context)?;
        }
        if let Some(ref src) = self.source {
            write!(f, ": {}", src)?;
        }
        Ok(())
    }
}

impl std::error::Error for OssError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &dyn std::error::Error)
    }
}

pub type Result<T> = std::result::Result<T, OssError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oss_service_error_from_xml_parses_all_fields() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <Error>
          <Code>NoSuchBucket</Code>
          <Message>The specified bucket does not exist.</Message>
          <RequestId>5D8A8578E44B3E3FD474D789</RequestId>
          <HostId>oss-cn-hangzhou.aliyuncs.com</HostId>
          <BucketName>non-existent-bucket</BucketName>
        </Error>"#;

        let error = OssServiceError::from_xml(xml).unwrap();
        assert_eq!(error.code, "NoSuchBucket");
        assert_eq!(error.message, "The specified bucket does not exist.");
        assert_eq!(error.request_id, "5D8A8578E44B3E3FD474D789");
        assert_eq!(error.host_id, "oss-cn-hangzhou.aliyuncs.com");
    }

    #[test]
    fn oss_service_error_from_xml_parses_optional_fields() {
        let xml = r#"<?xml version="1.0"?>
        <Error>
          <Code>AccessDenied</Code>
          <Message>Access Denied</Message>
          <RequestId>req-id-123</RequestId>
          <HostId>oss-cn-hangzhou.aliyuncs.com</HostId>
          <BucketName>my-bucket</BucketName>
          <StringToSign>PUT\n\n\n...</StringToSign>
        </Error>"#;

        let error = OssServiceError::from_xml(xml).unwrap();
        assert_eq!(error.resource.as_deref(), Some("my-bucket"));
        assert!(error.string_to_sign.is_some());
    }

    #[test]
    fn oss_error_display_includes_error_code_and_message() {
        let err = OssError::service(OssServiceError {
            status_code: 404,
            code: "NoSuchBucket".into(),
            message: "Bucket not found".into(),
            request_id: "xxx".into(),
            host_id: "oss-cn-hangzhou.aliyuncs.com".into(),
            resource: None,
            string_to_sign: None,
        });
        let display = err.to_string();
        assert!(display.contains("NoSuchBucket"));
        assert!(display.contains("Bucket not found"));
    }

    #[test]
    fn oss_error_kind_validation_error_with_context() {
        let err = OssError::validation("PutObject", "my-bucket", "my-key");
        let display = err.to_string();
        assert!(display.contains("ValidationError"));
        assert!(display.contains("PutObject"));
        assert!(display.contains("my-bucket"));
        assert!(display.contains("my-key"));
    }

    #[test]
    fn oss_error_from_http_status_without_body_produces_transport_error() {
        let err = OssError::transport(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "connection refused",
        ));
        let display = err.to_string();
        assert!(display.contains("TransportError"));
        assert!(display.contains("connection refused"));
    }

    #[test]
    fn oss_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<OssError>();
        assert_send_sync::<OssServiceError>();
        assert_send_sync::<OssErrorKind>();
        assert_send_sync::<ErrorContext>();
    }

    #[test]
    fn oss_error_kind_display_shows_label() {
        assert!(
            OssErrorKind::ConfigError
                .to_string()
                .contains("ConfigError")
        );
        assert!(
            OssErrorKind::SigningError
                .to_string()
                .contains("SigningError")
        );
        assert!(
            OssErrorKind::TimeoutError
                .to_string()
                .contains("TimeoutError")
        );
        assert!(OssErrorKind::Unknown.to_string().contains("Unknown"));
    }
}
