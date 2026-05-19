//! Access Control List (ACL) types for buckets and objects.

use std::fmt;
use std::str::FromStr;

use crate::error::{ErrorContext, OssError, OssErrorKind};

/// Access Control List polity for a bucket.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BucketAcl {
    #[default]
    Private,
    PublicRead,
    PublicReadWrite,
}

impl BucketAcl {
    /// Returns the ACL string representation (e.g. "private").
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Private => "private",
            Self::PublicRead => "public-read",
            Self::PublicReadWrite => "public-read-write",
        }
    }
}

impl fmt::Display for BucketAcl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for BucketAcl {
    type Err = OssError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "private" => Ok(Self::Private),
            "public-read" => Ok(Self::PublicRead),
            "public-read-write" => Ok(Self::PublicReadWrite),
            other => Err(OssError {
                kind: OssErrorKind::ValidationError,
                context: Box::new(ErrorContext {
                    operation: Some(format!("parse BucketAcl from '{}'", other)),
                    ..Default::default()
                }),
                source: None,
            }),
        }
    }
}

/// Access Control List polity for an object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ObjectAcl {
    #[default]
    Default,
    Private,
    PublicRead,
    PublicReadWrite,
}

impl ObjectAcl {
    /// Returns the ACL string representation (e.g. "private").
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Private => "private",
            Self::PublicRead => "public-read",
            Self::PublicReadWrite => "public-read-write",
        }
    }
}

impl fmt::Display for ObjectAcl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ObjectAcl {
    type Err = OssError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "default" => Ok(Self::Default),
            "private" => Ok(Self::Private),
            "public-read" => Ok(Self::PublicRead),
            "public-read-write" => Ok(Self::PublicReadWrite),
            other => Err(OssError {
                kind: OssErrorKind::ValidationError,
                context: Box::new(ErrorContext {
                    operation: Some(format!("parse ObjectAcl from '{}'", other)),
                    ..Default::default()
                }),
                source: None,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bucket_acl_default_is_private() {
        assert_eq!(BucketAcl::default(), BucketAcl::Private);
    }

    #[test]
    fn bucket_acl_string_representation() {
        assert_eq!(BucketAcl::Private.as_str(), "private");
        assert_eq!(BucketAcl::PublicRead.as_str(), "public-read");
        assert_eq!(BucketAcl::PublicReadWrite.as_str(), "public-read-write");
    }

    #[test]
    fn bucket_acl_from_str() {
        assert_eq!("private".parse::<BucketAcl>().unwrap(), BucketAcl::Private);
        assert_eq!(
            "public-read".parse::<BucketAcl>().unwrap(),
            BucketAcl::PublicRead
        );
        assert!("invalid".parse::<BucketAcl>().is_err());
    }

    #[test]
    fn object_acl_from_str() {
        assert_eq!("private".parse::<ObjectAcl>().unwrap(), ObjectAcl::Private);
        assert_eq!(
            "public-read".parse::<ObjectAcl>().unwrap(),
            ObjectAcl::PublicRead
        );
        assert_eq!(
            "public-read-write".parse::<ObjectAcl>().unwrap(),
            ObjectAcl::PublicReadWrite
        );
        assert_eq!("default".parse::<ObjectAcl>().unwrap(), ObjectAcl::Default);
    }

    #[test]
    fn object_acl_default() {
        assert_eq!(ObjectAcl::default(), ObjectAcl::Default);
    }

    #[test]
    fn acl_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<BucketAcl>();
        assert_send_sync::<ObjectAcl>();
    }
}
