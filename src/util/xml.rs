//! XML serialization and deserialization utilities.

use serde::{Deserialize, Serialize};

use crate::error::{OssError, OssErrorKind, Result};

/// Serializes a value to an XML string using quick-xml.
pub fn to_xml<T: Serialize>(value: &T) -> Result<String> {
    quick_xml::se::to_string(value).map_err(|e| OssError {
        kind: OssErrorKind::XmlError,
        context: Box::new(crate::error::ErrorContext {
            operation: Some(format!("serialize {}", std::any::type_name::<T>())),
            ..Default::default()
        }),
        source: Some(Box::new(e)),
    })
}

/// Deserializes an XML string into a value using quick-xml.
pub fn from_xml<T: for<'de> Deserialize<'de>>(xml: &str) -> Result<T> {
    quick_xml::de::from_str(xml).map_err(|e| OssError {
        kind: OssErrorKind::XmlError,
        context: Box::new(crate::error::ErrorContext {
            operation: Some(format!("deserialize {}", std::any::type_name::<T>())),
            ..Default::default()
        }),
        source: Some(Box::new(e)),
    })
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct CreateBucketConfiguration {
        #[serde(rename = "StorageClass")]
        storage_class: String,
        #[serde(rename = "DataRedundancyType")]
        data_redundancy: String,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(rename = "Error")]
    struct OssErrorXml {
        #[serde(rename = "Code")]
        code: String,
        #[serde(rename = "Message")]
        message: String,
        #[serde(rename = "RequestId")]
        request_id: String,
        #[serde(rename = "HostId")]
        host_id: String,
        #[serde(rename = "BucketName")]
        #[serde(default)]
        bucket_name: Option<String>,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct ListBucketResult {
        #[serde(rename = "Name")]
        name: String,
        #[serde(rename = "Prefix")]
        #[serde(default)]
        prefix: String,
        #[serde(rename = "Marker")]
        #[serde(default)]
        marker: String,
        #[serde(rename = "MaxKeys")]
        max_keys: u32,
        #[serde(rename = "IsTruncated")]
        is_truncated: bool,
        #[serde(rename = "Contents", default)]
        contents: Vec<ObjectContent>,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct ObjectContent {
        #[serde(rename = "Key")]
        key: String,
        #[serde(rename = "Size")]
        size: u64,
        #[serde(rename = "ETag")]
        etag: String,
    }

    #[derive(Debug, Serialize, PartialEq)]
    #[serde(rename = "Delete")]
    struct DeleteObjectsRequest {
        #[serde(rename = "Quiet", default)]
        quiet: bool,
        #[serde(rename = "Object")]
        objects: Vec<DeleteObject>,
    }

    #[derive(Debug, Serialize, PartialEq)]
    struct DeleteObject {
        #[serde(rename = "Key")]
        key: String,
    }

    #[test]
    fn serialize_bucket_configuration_to_xml() {
        let config = CreateBucketConfiguration {
            storage_class: "Standard".into(),
            data_redundancy: "LRS".into(),
        };
        let xml = to_xml(&config).unwrap();
        assert!(xml.contains("<StorageClass>Standard</StorageClass>"));
        assert!(xml.contains("<DataRedundancyType>LRS</DataRedundancyType>"));
    }

    #[test]
    fn deserialize_error_xml_to_oss_service_error() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <Error>
          <Code>NoSuchBucket</Code>
          <Message>The specified bucket does not exist.</Message>
          <RequestId>5D8A8578E44B3E3FD474D789</RequestId>
          <HostId>oss-cn-hangzhou.aliyuncs.com</HostId>
          <BucketName>non-existent-bucket</BucketName>
        </Error>"#;

        let error: OssErrorXml = from_xml(xml).unwrap();
        assert_eq!(error.code, "NoSuchBucket");
        assert_eq!(error.message, "The specified bucket does not exist.");
        assert_eq!(error.request_id, "5D8A8578E44B3E3FD474D789");
        assert_eq!(error.host_id, "oss-cn-hangzhou.aliyuncs.com");
        assert_eq!(error.bucket_name.as_deref(), Some("non-existent-bucket"));
    }

    #[test]
    fn deserialize_list_objects_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <ListBucketResult>
          <Name>examplebucket</Name>
          <Prefix>test</Prefix>
          <Marker></Marker>
          <MaxKeys>100</MaxKeys>
          <IsTruncated>false</IsTruncated>
          <Contents>
            <Key>test/file1.txt</Key>
            <Size>1024</Size>
            <ETag>"abc123"</ETag>
          </Contents>
          <Contents>
            <Key>test/file2.txt</Key>
            <Size>2048</Size>
            <ETag>"def456"</ETag>
          </Contents>
        </ListBucketResult>"#;

        let result: ListBucketResult = from_xml(xml).unwrap();
        assert_eq!(result.name, "examplebucket");
        assert_eq!(result.prefix, "test");
        assert_eq!(result.max_keys, 100);
        assert!(!result.is_truncated);
        assert_eq!(result.contents.len(), 2);
        assert_eq!(result.contents[0].key, "test/file1.txt");
        assert_eq!(result.contents[0].size, 1024);
        assert_eq!(result.contents[0].etag, "\"abc123\"");
    }

    #[test]
    fn serialize_delete_multiple_objects_xml() {
        let request = DeleteObjectsRequest {
            quiet: true,
            objects: vec![
                DeleteObject {
                    key: "obj1.txt".into(),
                },
                DeleteObject {
                    key: "obj2.txt".into(),
                },
            ],
        };
        let xml = to_xml(&request).unwrap();
        assert!(xml.contains("<Delete>"));
        assert!(xml.contains("<Quiet>true</Quiet>"));
        assert!(xml.contains("<Object><Key>obj1.txt</Key></Object>"));
        assert!(xml.contains("<Object><Key>obj2.txt</Key></Object>"));
    }

    #[test]
    fn roundtrip_serialize_deserialize() {
        let config = CreateBucketConfiguration {
            storage_class: "IA".into(),
            data_redundancy: "ZRS".into(),
        };
        let xml = to_xml(&config).unwrap();
        let deserialized: CreateBucketConfiguration = from_xml(&xml).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn deserialize_missing_optional_fields() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <Error>
          <Code>AccessDenied</Code>
          <Message>Access Denied</Message>
          <RequestId>req-123</RequestId>
          <HostId>oss-cn-hangzhou.aliyuncs.com</HostId>
        </Error>"#;

        let error: OssErrorXml = from_xml(xml).unwrap();
        assert_eq!(error.code, "AccessDenied");
        assert!(error.bucket_name.is_none());
    }
}
