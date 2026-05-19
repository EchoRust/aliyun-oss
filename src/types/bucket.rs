//! Bucket name newtype with validation.

use std::fmt;

use crate::error::{ErrorContext, OssError, OssErrorKind, Result};

const MIN_BUCKET_NAME_LENGTH: usize = 3;
const MAX_BUCKET_NAME_LENGTH: usize = 63;

/// A validated OSS bucket name (3-63 lowercase alphanumeric and hyphens).
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct BucketName(String);

impl BucketName {
    /// Creates a new `BucketName` after validating the input.
    pub fn new(name: impl Into<String>) -> Result<Self> {
        let name = name.into();
        validate_bucket_name(&name)?;
        Ok(Self(name))
    }

    /// Returns the bucket name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for BucketName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("BucketName").field(&self.0).finish()
    }
}

impl fmt::Display for BucketName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn validate_bucket_name(name: &str) -> Result<()> {
    let len = name.len();
    if !(MIN_BUCKET_NAME_LENGTH..=MAX_BUCKET_NAME_LENGTH).contains(&len) {
        return Err(OssError {
            kind: OssErrorKind::ValidationError,
            context: Box::new(ErrorContext {
                operation: Some(format!(
                    "validate BucketName length: {} (min {MIN_BUCKET_NAME_LENGTH}, max {MAX_BUCKET_NAME_LENGTH})",
                    len
                )),
                bucket: Some(name.to_string()),
                ..Default::default()
            }),
            source: None,
        });
    }

    if name.starts_with('-') || name.ends_with('-') {
        return Err(OssError {
            kind: OssErrorKind::ValidationError,
            context: Box::new(ErrorContext {
                operation: Some("validate BucketName hyphen position".into()),
                bucket: Some(name.to_string()),
                ..Default::default()
            }),
            source: None,
        });
    }

    if is_ip_format(name) {
        return Err(OssError {
            kind: OssErrorKind::ValidationError,
            context: Box::new(ErrorContext {
                operation: Some("validate BucketName: IP format not allowed".into()),
                bucket: Some(name.to_string()),
                ..Default::default()
            }),
            source: None,
        });
    }

    for (i, ch) in name.char_indices() {
        let valid = ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-';
        if !valid {
            return Err(OssError {
                kind: OssErrorKind::ValidationError,
                context: Box::new(ErrorContext {
                    operation: Some(format!(
                        "validate BucketName: invalid char '{}' at position {}",
                        ch, i
                    )),
                    bucket: Some(name.to_string()),
                    ..Default::default()
                }),
                source: None,
            });
        }
    }

    Ok(())
}

fn is_ip_format(name: &str) -> bool {
    let parts: Vec<&str> = name.split('.').collect();
    if parts.len() != 4 {
        return false;
    }
    parts.iter().all(|p| p.parse::<u8>().is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bucket_name_new_valid_name() {
        assert!(BucketName::new("my-bucket").is_ok());
        assert!(BucketName::new("my-bucket-123").is_ok());
        assert!(BucketName::new("abc").is_ok());
        assert!(BucketName::new("a".repeat(63)).is_ok());
    }

    #[test]
    fn bucket_name_rejects_invalid_length() {
        assert!(BucketName::new("ab").is_err());
        assert!(BucketName::new("a".repeat(64)).is_err());
    }

    #[test]
    fn bucket_name_rejects_invalid_characters() {
        assert!(BucketName::new("MyBucket").is_err());
        assert!(BucketName::new("my_bucket").is_err());
        assert!(BucketName::new("-mybucket").is_err());
        assert!(BucketName::new("mybucket-").is_err());
        assert!(BucketName::new("my bucket").is_err());
    }

    #[test]
    fn bucket_name_rejects_ip_format() {
        assert!(BucketName::new("192.168.1.1").is_err());
        assert!(BucketName::new("10.0.0.0").is_err());
    }

    #[test]
    fn bucket_name_accepts_letters_digits_and_dashes() {
        assert!(BucketName::new("a-1").is_ok());
        assert!(BucketName::new("1-2-3-4").is_ok());
        assert!(BucketName::new("my-bucket-99").is_ok());
    }

    #[test]
    fn bucket_name_display() {
        let name = BucketName::new("my-bucket").unwrap();
        assert_eq!(name.to_string(), "my-bucket");
    }

    #[test]
    fn bucket_name_as_str() {
        let name = BucketName::new("my-bucket").unwrap();
        assert_eq!(name.as_str(), "my-bucket");
    }
}
