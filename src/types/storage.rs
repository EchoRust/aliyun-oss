//! Storage class and server-side encryption types.

use std::fmt;
use std::str::FromStr;

use crate::error::{ErrorContext, OssError, OssErrorKind};

/// OSS storage class for object storage tiering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StorageClass {
    #[default]
    Standard,
    IA,
    Archive,
    ColdArchive,
    DeepColdArchive,
}

impl StorageClass {
    /// Returns the storage class string representation (e.g. "Standard").
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Standard => "Standard",
            Self::IA => "IA",
            Self::Archive => "Archive",
            Self::ColdArchive => "ColdArchive",
            Self::DeepColdArchive => "DeepColdArchive",
        }
    }
}

impl fmt::Display for StorageClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for StorageClass {
    type Err = OssError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Standard" => Ok(Self::Standard),
            "IA" => Ok(Self::IA),
            "Archive" => Ok(Self::Archive),
            "ColdArchive" => Ok(Self::ColdArchive),
            "DeepColdArchive" => Ok(Self::DeepColdArchive),
            other => Err(OssError {
                kind: OssErrorKind::ValidationError,
                context: Box::new(ErrorContext {
                    operation: Some(format!("parse StorageClass from '{}'", other)),
                    ..Default::default()
                }),
                source: None,
            }),
        }
    }
}

/// Data redundancy type: locally redundant (LRS) or zone-redundant (ZRS).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DataRedundancyType {
    #[default]
    LRS,
    ZRS,
}

impl DataRedundancyType {
    /// Returns the redundancy type string (e.g. "LRS").
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LRS => "LRS",
            Self::ZRS => "ZRS",
        }
    }
}

impl fmt::Display for DataRedundancyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Server-side encryption configuration for objects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerSideEncryption {
    AES256,
    KMS,
    KMSWithKey(String),
}

impl ServerSideEncryption {
    /// Returns the encryption algorithm identifier (e.g. "AES256").
    pub fn as_str(&self) -> &str {
        match self {
            Self::AES256 => "AES256",
            Self::KMS => "KMS",
            Self::KMSWithKey(_) => "KMS",
        }
    }

    /// Returns the KMS key ID, if configured.
    pub fn key_id(&self) -> Option<&str> {
        match self {
            Self::KMSWithKey(key) => Some(key.as_str()),
            _ => None,
        }
    }
}

impl fmt::Display for ServerSideEncryption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KMSWithKey(key) => write!(f, "KMS:{}", key),
            _ => f.write_str(self.as_str()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_class_string_representation() {
        assert_eq!(StorageClass::Standard.as_str(), "Standard");
        assert_eq!(StorageClass::IA.as_str(), "IA");
        assert_eq!(StorageClass::Archive.as_str(), "Archive");
        assert_eq!(StorageClass::ColdArchive.as_str(), "ColdArchive");
        assert_eq!(StorageClass::DeepColdArchive.as_str(), "DeepColdArchive");
    }

    #[test]
    fn storage_class_default_is_standard() {
        assert_eq!(StorageClass::default(), StorageClass::Standard);
    }

    #[test]
    fn storage_class_from_str() {
        assert_eq!(
            "Standard".parse::<StorageClass>().unwrap(),
            StorageClass::Standard
        );
        assert_eq!("IA".parse::<StorageClass>().unwrap(), StorageClass::IA);
        assert!("Invalid".parse::<StorageClass>().is_err());
    }

    #[test]
    fn storage_class_display() {
        assert_eq!(StorageClass::Archive.to_string(), "Archive");
    }

    #[test]
    fn data_redundancy_type_as_str() {
        assert_eq!(DataRedundancyType::LRS.as_str(), "LRS");
        assert_eq!(DataRedundancyType::ZRS.as_str(), "ZRS");
    }

    #[test]
    fn server_side_encryption_string() {
        assert_eq!(ServerSideEncryption::AES256.as_str(), "AES256");
        assert_eq!(ServerSideEncryption::KMS.as_str(), "KMS");
        assert_eq!(
            ServerSideEncryption::KMSWithKey("cmk-id".into()).as_str(),
            "KMS"
        );
    }

    #[test]
    fn server_side_encryption_key_id() {
        assert!(ServerSideEncryption::AES256.key_id().is_none());
        assert!(ServerSideEncryption::KMS.key_id().is_none());
        assert_eq!(
            ServerSideEncryption::KMSWithKey("my-key".into()).key_id(),
            Some("my-key")
        );
    }

    #[test]
    fn server_side_encryption_display() {
        assert_eq!(ServerSideEncryption::AES256.to_string(), "AES256");
        assert_eq!(
            ServerSideEncryption::KMSWithKey("my-key".into()).to_string(),
            "KMS:my-key"
        );
    }

    #[test]
    fn storage_class_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<StorageClass>();
        assert_send_sync::<DataRedundancyType>();
        assert_send_sync::<ServerSideEncryption>();
    }
}
