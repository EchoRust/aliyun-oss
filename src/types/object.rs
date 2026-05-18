use std::fmt;

use crate::error::{ErrorContext, OssError, OssErrorKind, Result};

const MAX_OBJECT_KEY_LENGTH: usize = 1024;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ObjectKey(String);

impl ObjectKey {
    pub fn new(key: impl Into<String>) -> Result<Self> {
        let key = key.into();
        validate_object_key(&key)?;
        Ok(Self(key))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn parent(&self) -> Option<&str> {
        let path = &self.0;
        if let Some(pos) = path.rfind('/') {
            Some(&path[..=pos])
        } else {
            None
        }
    }

    pub fn file_name(&self) -> &str {
        let path = &self.0;
        if let Some(pos) = path.rfind('/') {
            &path[pos + 1..]
        } else {
            path
        }
    }
}

impl fmt::Debug for ObjectKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ObjectKey").field(&self.0).finish()
    }
}

impl fmt::Display for ObjectKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn validate_object_key(key: &str) -> Result<()> {
    if key.is_empty() {
        return Err(OssError {
            kind: OssErrorKind::ValidationError,
            context: Box::new(ErrorContext {
                operation: Some("validate ObjectKey: empty".into()),
                object_key: Some(key.to_string()),
                ..Default::default()
            }),
            source: None,
        });
    }

    if key.len() > MAX_OBJECT_KEY_LENGTH {
        return Err(OssError {
            kind: OssErrorKind::ValidationError,
            context: Box::new(ErrorContext {
                operation: Some(format!(
                    "validate ObjectKey length: {} (max {MAX_OBJECT_KEY_LENGTH})",
                    key.len()
                )),
                object_key: Some(key.to_string()),
                ..Default::default()
            }),
            source: None,
        });
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ETag(String);

impl ETag {
    pub fn new(tag: impl Into<String>) -> Self {
        Self(tag.into())
    }

    pub fn from_header(value: &str) -> Option<Self> {
        let trimmed = value.trim();
        if trimmed.len() < 2 {
            return None;
        }
        if trimmed.starts_with('"') && trimmed.ends_with('"') {
            Some(Self(trimmed[1..trimmed.len() - 1].to_string()))
        } else {
            Some(Self(trimmed.to_string()))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ETag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Owner {
    pub id: String,
    pub display_name: Option<String>,
}

impl Owner {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            display_name: None,
        }
    }

    pub fn with_display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn object_key_new_valid() {
        assert!(ObjectKey::new("hello.txt").is_ok());
        assert!(ObjectKey::new("a/b/c/d.jpg").is_ok());
        assert!(ObjectKey::new("中文/文件名.jpg").is_ok());
    }

    #[test]
    fn object_key_rejects_empty() {
        assert!(ObjectKey::new("").is_err());
    }

    #[test]
    fn object_key_rejects_over_max_length() {
        assert!(ObjectKey::new("a".repeat(1025)).is_err());
    }

    #[test]
    fn object_key_accepts_exact_max_length() {
        assert!(ObjectKey::new("a".repeat(1024)).is_ok());
    }

    #[test]
    fn object_key_parent_returns_directory() {
        let key = ObjectKey::new("a/b/c.txt").unwrap();
        assert_eq!(key.parent(), Some("a/b/"));
    }

    #[test]
    fn object_key_parent_no_directory() {
        let key = ObjectKey::new("file.txt").unwrap();
        assert_eq!(key.parent(), None);
    }

    #[test]
    fn object_key_file_name_extracts_name() {
        let key = ObjectKey::new("a/b/c.txt").unwrap();
        assert_eq!(key.file_name(), "c.txt");
        let key2 = ObjectKey::new("noext").unwrap();
        assert_eq!(key2.file_name(), "noext");
    }

    #[test]
    fn etag_from_response_header_with_quotes() {
        let etag = ETag::from_header("\"abc123\"").unwrap();
        assert_eq!(etag.as_str(), "abc123");
    }

    #[test]
    fn etag_from_response_header_without_quotes() {
        let etag = ETag::from_header("abc123").unwrap();
        assert_eq!(etag.as_str(), "abc123");
    }

    #[test]
    fn etag_from_response_header_empty() {
        assert!(ETag::from_header("").is_none());
    }

    #[test]
    fn etag_display_includes_quotes() {
        let etag = ETag::new("abc123");
        assert_eq!(etag.to_string(), "\"abc123\"");
    }

    #[test]
    fn owner_new() {
        let owner = Owner::new("owner-id");
        assert_eq!(owner.id, "owner-id");
        assert!(owner.display_name.is_none());
    }

    #[test]
    fn owner_with_display_name() {
        let owner = Owner::new("owner-id").with_display_name("Owner Name");
        assert_eq!(owner.id, "owner-id");
        assert_eq!(owner.display_name.as_deref(), Some("Owner Name"));
    }

    #[test]
    fn object_key_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ObjectKey>();
        assert_send_sync::<ETag>();
        assert_send_sync::<Owner>();
    }
}
