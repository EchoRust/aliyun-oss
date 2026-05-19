//! Request body abstraction for uploads.

use std::path::PathBuf;

/// An abstraction over upload request bodies: bytes, file path, or empty.
pub enum RequestBody {
    Bytes(bytes::Bytes),
    File(PathBuf),
    Empty,
}

impl RequestBody {
    /// Returns the length of the body in bytes (0 for File and Empty variants).
    pub fn len(&self) -> usize {
        match self {
            Self::Bytes(b) => b.len(),
            Self::File(_) => 0,
            Self::Empty => 0,
        }
    }

    /// Returns `true` if the body contains no data.
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Bytes(b) => b.is_empty(),
            Self::File(_) => true,
            Self::Empty => true,
        }
    }

    /// Returns the byte content, if the body is of the `Bytes` variant.
    pub fn to_bytes(&self) -> Option<bytes::Bytes> {
        match self {
            Self::Bytes(b) => Some(b.clone()),
            Self::File(_) | Self::Empty => None,
        }
    }
}

impl From<Vec<u8>> for RequestBody {
    fn from(v: Vec<u8>) -> Self {
        Self::Bytes(v.into())
    }
}

impl From<bytes::Bytes> for RequestBody {
    fn from(b: bytes::Bytes) -> Self {
        Self::Bytes(b)
    }
}

impl From<&str> for RequestBody {
    fn from(s: &str) -> Self {
        Self::Bytes(bytes::Bytes::copy_from_slice(s.as_bytes()))
    }
}

impl From<String> for RequestBody {
    fn from(s: String) -> Self {
        Self::Bytes(bytes::Bytes::from(s.into_bytes()))
    }
}

impl From<&[u8]> for RequestBody {
    fn from(data: &[u8]) -> Self {
        Self::Bytes(bytes::Bytes::copy_from_slice(data))
    }
}

impl From<PathBuf> for RequestBody {
    fn from(path: PathBuf) -> Self {
        Self::File(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_body_from_bytes_has_correct_length() {
        let body = RequestBody::from(b"hello world".to_vec());
        assert_eq!(body.len(), 11);
    }

    #[test]
    fn request_body_from_string_preserves_content() {
        let body = RequestBody::from("hello, oss!");
        assert_eq!(body.to_bytes().unwrap().as_ref(), b"hello, oss!");
    }

    #[test]
    fn request_body_empty_has_zero_length() {
        let body = RequestBody::Empty;
        assert_eq!(body.len(), 0);
        assert!(body.is_empty());
    }

    #[test]
    fn request_body_bytes_is_not_empty_when_populated() {
        let body = RequestBody::from(b"data".to_vec());
        assert_eq!(body.len(), 4);
        assert!(!body.is_empty());
    }

    #[test]
    fn request_body_empty_bytes_is_empty() {
        let body = RequestBody::from(b"".to_vec());
        assert_eq!(body.len(), 0);
        assert!(body.is_empty());
    }

    #[test]
    fn request_body_from_file_path() {
        let path = PathBuf::from("/tmp/test-file.bin");
        let body = RequestBody::from(path);
        assert!(matches!(body, RequestBody::File(_)));
        assert_eq!(body.len(), 0);
        assert!(body.is_empty());
        assert!(body.to_bytes().is_none());
    }

    #[test]
    fn request_body_from_bytes_type() {
        let b = bytes::Bytes::from_static(b"hello bytes");
        let body = RequestBody::from(b);
        assert_eq!(body.len(), 11);
        assert_eq!(body.to_bytes().unwrap().as_ref(), b"hello bytes");
    }

    #[test]
    fn request_body_from_slice() {
        let data: &[u8] = b"slice data";
        let body = RequestBody::from(data);
        assert_eq!(body.len(), 10);
    }

    #[test]
    fn request_body_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<RequestBody>();
    }
}
