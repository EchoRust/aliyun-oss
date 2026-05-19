//! XML response deserialization types for OSS API responses.

use serde::Deserialize;

use super::bucket::BucketName;
use super::object::{ETag, ObjectKey};

/// Response for ListObjects (V1) API.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename = "ListBucketResult")]
pub struct ListObjectsOutput {
    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "Prefix", default)]
    pub prefix: String,

    #[serde(rename = "Marker", default)]
    pub marker: String,

    #[serde(rename = "MaxKeys")]
    pub max_keys: i32,

    #[serde(rename = "IsTruncated")]
    pub is_truncated: bool,

    #[serde(rename = "NextMarker", default)]
    pub next_marker: String,

    #[serde(rename = "Delimiter", default)]
    pub delimiter: String,

    #[serde(rename = "EncodingType", default)]
    pub encoding_type: String,

    #[serde(rename = "CommonPrefixes", default)]
    pub common_prefixes: Vec<CommonPrefix>,

    #[serde(rename = "Contents", default)]
    pub objects: Vec<ObjectSummary>,
}

impl ListObjectsOutput {
    /// Returns the bucket name parsed from the response.
    pub fn bucket_name(&self) -> Option<BucketName> {
        BucketName::new(&self.name).ok()
    }

    /// Returns the parsed object keys from the response.
    pub fn object_keys(&self) -> Vec<ObjectKey> {
        self.objects
            .iter()
            .filter_map(|o| ObjectKey::new(&o.key).ok())
            .collect()
    }
}

/// A common prefix (directory-like entry) returned in list results.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct CommonPrefix {
    #[serde(rename = "Prefix")]
    pub prefix: String,
}

/// Summary information for a single object in a list results response.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ObjectSummary {
    #[serde(rename = "Key")]
    pub key: String,

    #[serde(rename = "LastModified")]
    pub last_modified: String,

    #[serde(rename = "ETag")]
    pub etag: String,

    #[serde(rename = "Size")]
    pub size: u64,

    #[serde(rename = "StorageClass")]
    pub storage_class: String,

    #[serde(rename = "Owner", default)]
    pub owner: Option<OwnerInfo>,
}

impl ObjectSummary {
    /// Returns the parsed ETag from the response.
    pub fn etag_parsed(&self) -> Option<ETag> {
        ETag::from_header(&self.etag)
    }

    /// Returns the parsed object key.
    pub fn object_key(&self) -> Option<ObjectKey> {
        ObjectKey::new(&self.key).ok()
    }
}

/// Owner information from OSS API responses.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct OwnerInfo {
    #[serde(rename = "ID")]
    pub id: String,

    #[serde(rename = "DisplayName", default)]
    pub display_name: String,
}

/// Response for ListObjectsV2 API.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename = "ListBucketResult")]
pub struct ListObjectsV2Output {
    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "Prefix", default)]
    pub prefix: String,

    #[serde(rename = "StartAfter", default)]
    pub start_after: String,

    #[serde(rename = "MaxKeys")]
    pub max_keys: i32,

    #[serde(rename = "IsTruncated")]
    pub is_truncated: bool,

    #[serde(rename = "NextContinuationToken", default)]
    pub next_continuation_token: String,

    #[serde(rename = "ContinuationToken", default)]
    pub continuation_token: String,

    #[serde(rename = "KeyCount", default)]
    pub key_count: i32,

    #[serde(rename = "Delimiter", default)]
    pub delimiter: String,

    #[serde(rename = "EncodingType", default)]
    pub encoding_type: String,

    #[serde(rename = "CommonPrefixes", default)]
    pub common_prefixes: Vec<CommonPrefix>,

    #[serde(rename = "Contents", default)]
    pub objects: Vec<ObjectSummary>,
}

/// Response for the ListBuckets (GetService) API.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename = "ListAllMyBucketsResult")]
pub struct ListBucketsOutput {
    #[serde(rename = "Owner")]
    pub owner: OwnerInfo,

    #[serde(rename = "Buckets", default)]
    pub buckets: BucketsContainer,
}

/// Container for bucket summaries in a ListBuckets response.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
pub struct BucketsContainer {
    #[serde(rename = "Bucket", default)]
    pub bucket: Vec<BucketSummary>,
}

/// Summary of a single bucket in the ListBuckets response.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct BucketSummary {
    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "CreationDate")]
    pub creation_date: String,

    #[serde(rename = "Location")]
    pub location: String,

    #[serde(rename = "ExtranetEndpoint")]
    pub extranet_endpoint: String,

    #[serde(rename = "IntranetEndpoint")]
    pub intranet_endpoint: String,

    #[serde(rename = "StorageClass")]
    pub storage_class: String,

    #[serde(rename = "Region", default)]
    pub region: String,
}

/// Response for the GetBucketInfo API.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename = "BucketInfo")]
pub struct GetBucketInfoOutput {
    #[serde(rename = "Bucket")]
    pub bucket: BucketInfoDetail,
}

/// Detailed bucket information from GetBucketInfo.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct BucketInfoDetail {
    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "CreationDate")]
    pub creation_date: String,

    #[serde(rename = "Location")]
    pub location: String,

    #[serde(rename = "ExtranetEndpoint")]
    pub extranet_endpoint: String,

    #[serde(rename = "IntranetEndpoint")]
    pub intranet_endpoint: String,

    #[serde(rename = "StorageClass")]
    pub storage_class: String,

    #[serde(rename = "Owner")]
    pub owner: OwnerInfo,

    #[serde(rename = "AccessControlList", default)]
    pub acl: Option<AccessControlList>,

    #[serde(rename = "DataRedundancyType", default)]
    pub data_redundancy_type: String,

    #[serde(rename = "Versioning", default)]
    pub versioning: String,
}

/// ACL policy detail from bucket/object info responses.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct AccessControlList {
    #[serde(rename = "Grant")]
    pub grant: String,
}

/// Response for the GetBucketStat API.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename = "BucketStat")]
pub struct GetBucketStatOutput {
    #[serde(rename = "Storage")]
    pub storage: u64,

    #[serde(rename = "ObjectCount")]
    pub object_count: u64,

    #[serde(rename = "MultipartUploadCount")]
    pub multipart_upload_count: u64,

    #[serde(rename = "LiveChannelCount")]
    pub live_channel_count: u64,

    #[serde(rename = "LastModifiedTime")]
    pub last_modified_time: u64,

    #[serde(rename = "StandardStorage")]
    pub standard_storage: u64,

    #[serde(rename = "StandardObjectCount")]
    pub standard_object_count: u64,

    #[serde(rename = "InfrequentAccessStorage")]
    pub ia_storage: u64,

    #[serde(rename = "InfrequentAccessObjectCount")]
    pub ia_object_count: u64,

    #[serde(rename = "ArchiveStorage")]
    pub archive_storage: u64,

    #[serde(rename = "ArchiveObjectCount")]
    pub archive_object_count: u64,

    #[serde(rename = "ColdArchiveStorage")]
    pub cold_archive_storage: u64,

    #[serde(rename = "ColdArchiveObjectCount")]
    pub cold_archive_object_count: u64,

    #[serde(rename = "DeepColdArchiveStorage")]
    pub deep_cold_archive_storage: u64,

    #[serde(rename = "DeepColdArchiveObjectCount")]
    pub deep_cold_archive_object_count: u64,
}

/// Response for the GetObjectAcl API.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename = "AccessControlPolicy")]
pub struct GetObjectAclOutput {
    #[serde(rename = "Owner")]
    pub owner: OwnerInfo,
    #[serde(rename = "AccessControlList")]
    pub acl: AccessControlPolicyDetail,
}

/// ACL grant detail from access policy responses.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct AccessControlPolicyDetail {
    #[serde(rename = "Grant")]
    pub grant: String,
}

/// Response for the DeleteMultipleObjects API.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename = "DeleteResult")]
pub struct DeleteResult {
    #[serde(rename = "Deleted", default)]
    pub deleted: Vec<DeletedObject>,
}

/// An entry for a successfully deleted object in a DeleteMultipleObjects response.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct DeletedObject {
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "DeleteMarker", default)]
    pub delete_marker: bool,
    #[serde(rename = "DeleteMarkerVersionId", default)]
    pub delete_marker_version_id: String,
    #[serde(rename = "VersionId", default)]
    pub version_id: String,
}

/// Response for the ListObjectVersions API.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename = "ListVersionsResult")]
pub struct ListVersionsOutput {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Prefix", default)]
    pub prefix: String,
    #[serde(rename = "KeyMarker", default)]
    pub key_marker: String,
    #[serde(rename = "VersionIdMarker", default)]
    pub version_id_marker: String,
    #[serde(rename = "MaxKeys")]
    pub max_keys: i32,
    #[serde(rename = "IsTruncated")]
    pub is_truncated: bool,
    #[serde(rename = "NextKeyMarker", default)]
    pub next_key_marker: String,
    #[serde(rename = "NextVersionIdMarker", default)]
    pub next_version_id_marker: String,
    #[serde(rename = "Version", default)]
    pub versions: Vec<ObjectVersion>,
    #[serde(rename = "DeleteMarker", default)]
    pub delete_markers: Vec<DeleteMarkerSummary>,
    #[serde(rename = "CommonPrefixes", default)]
    pub common_prefixes: Vec<CommonPrefix>,
}

/// An object version entry in a ListObjectVersions response.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ObjectVersion {
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "VersionId")]
    pub version_id: String,
    #[serde(rename = "IsLatest")]
    pub is_latest: bool,
    #[serde(rename = "LastModified")]
    pub last_modified: String,
    #[serde(rename = "ETag")]
    pub etag: String,
    #[serde(rename = "Size")]
    pub size: u64,
    #[serde(rename = "StorageClass")]
    pub storage_class: String,
    #[serde(rename = "Owner", default)]
    pub owner: Option<OwnerInfo>,
}

/// A delete marker entry in a ListObjectVersions response.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct DeleteMarkerSummary {
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "VersionId")]
    pub version_id: String,
    #[serde(rename = "IsLatest")]
    pub is_latest: bool,
    #[serde(rename = "LastModified")]
    pub last_modified: String,
    #[serde(rename = "Owner", default)]
    pub owner: Option<OwnerInfo>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::xml::from_xml;

    #[test]
    fn list_objects_output_parsed_from_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <ListBucketResult>
          <Name>examplebucket</Name>
          <Prefix>test</Prefix>
          <Marker></Marker>
          <MaxKeys>100</MaxKeys>
          <IsTruncated>false</IsTruncated>
          <Contents>
            <Key>test/file1.txt</Key>
            <LastModified>2024-01-01T00:00:00.000Z</LastModified>
            <ETag>"abc123"</ETag>
            <Size>1024</Size>
            <StorageClass>Standard</StorageClass>
            <Owner>
              <ID>owner-id</ID>
              <DisplayName>owner</DisplayName>
            </Owner>
          </Contents>
        </ListBucketResult>"#;

        let result: ListObjectsOutput = from_xml(xml).unwrap();
        assert_eq!(result.name, "examplebucket");
        assert_eq!(result.objects.len(), 1);
        assert_eq!(result.objects[0].key, "test/file1.txt");
        assert_eq!(result.objects[0].size, 1024);
        assert_eq!(result.objects[0].etag_parsed().unwrap().as_str(), "abc123");
    }

    #[test]
    fn list_objects_truncated_with_next_marker() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <ListBucketResult>
          <Name>bucket</Name>
          <MaxKeys>2</MaxKeys>
          <IsTruncated>true</IsTruncated>
          <NextMarker>next-token</NextMarker>
          <Contents>
            <Key>obj1</Key>
            <LastModified>2024-01-01T00:00:00.000Z</LastModified>
            <ETag>"etag1"</ETag>
            <Size>100</Size>
            <StorageClass>Standard</StorageClass>
          </Contents>
        </ListBucketResult>"#;

        let result: ListObjectsOutput = from_xml(xml).unwrap();
        assert!(result.is_truncated);
        assert_eq!(result.next_marker, "next-token");
    }

    #[test]
    fn common_prefixes_parsed_correctly() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <ListBucketResult>
          <Name>bucket</Name>
          <MaxKeys>100</MaxKeys>
          <IsTruncated>false</IsTruncated>
          <Prefix>dir/</Prefix>
          <Delimiter>/</Delimiter>
          <CommonPrefixes>
            <Prefix>dir/subdir1/</Prefix>
          </CommonPrefixes>
          <CommonPrefixes>
            <Prefix>dir/subdir2/</Prefix>
          </CommonPrefixes>
        </ListBucketResult>"#;

        let result: ListObjectsOutput = from_xml(xml).unwrap();
        assert_eq!(result.common_prefixes.len(), 2);
        assert_eq!(result.common_prefixes[0].prefix, "dir/subdir1/");
        assert_eq!(result.common_prefixes[1].prefix, "dir/subdir2/");
    }

    #[test]
    fn list_buckets_output_parsed_from_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <ListAllMyBucketsResult>
          <Owner>
            <ID>owner-id</ID>
            <DisplayName>owner-name</DisplayName>
          </Owner>
          <Buckets>
            <Bucket>
              <Name>bucket1</Name>
              <CreationDate>2024-01-01T00:00:00.000Z</CreationDate>
              <Location>oss-cn-hangzhou</Location>
              <ExtranetEndpoint>oss-cn-hangzhou.aliyuncs.com</ExtranetEndpoint>
              <IntranetEndpoint>oss-cn-hangzhou-internal.aliyuncs.com</IntranetEndpoint>
              <StorageClass>Standard</StorageClass>
            </Bucket>
          </Buckets>
        </ListAllMyBucketsResult>"#;

        let result: ListBucketsOutput = from_xml(xml).unwrap();
        assert_eq!(result.owner.id, "owner-id");
        assert_eq!(result.buckets.bucket.len(), 1);
        assert_eq!(result.buckets.bucket[0].name, "bucket1");
    }

    #[test]
    fn list_objects_v2_output_parsed_from_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <ListBucketResult>
          <Name>bucket</Name>
          <MaxKeys>100</MaxKeys>
          <IsTruncated>false</IsTruncated>
          <KeyCount>2</KeyCount>
          <ContinuationToken>token1</ContinuationToken>
          <Contents>
            <Key>obj1</Key>
            <LastModified>2024-01-01T00:00:00.000Z</LastModified>
            <ETag>"etag1"</ETag>
            <Size>100</Size>
            <StorageClass>Standard</StorageClass>
          </Contents>
        </ListBucketResult>"#;

        let result: ListObjectsV2Output = from_xml(xml).unwrap();
        assert_eq!(result.key_count, 2);
        assert_eq!(result.continuation_token, "token1");
    }

    #[test]
    fn get_bucket_info_output_parsed_from_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <BucketInfo>
          <Bucket>
            <Name>my-bucket</Name>
            <CreationDate>2024-01-01T00:00:00.000Z</CreationDate>
            <Location>oss-cn-hangzhou</Location>
            <StorageClass>Standard</StorageClass>
            <ExtranetEndpoint>oss-cn-hangzhou.aliyuncs.com</ExtranetEndpoint>
            <IntranetEndpoint>oss-cn-hangzhou-internal.aliyuncs.com</IntranetEndpoint>
            <Owner>
              <ID>owner-id</ID>
            </Owner>
            <AccessControlList>
              <Grant>private</Grant>
            </AccessControlList>
          </Bucket>
        </BucketInfo>"#;

        let result: GetBucketInfoOutput = from_xml(xml).unwrap();
        assert_eq!(result.bucket.name, "my-bucket");
        assert!(result.bucket.acl.is_some());
    }

    #[test]
    fn get_bucket_stat_output_parsed_from_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <BucketStat>
          <Storage>1024</Storage>
          <ObjectCount>10</ObjectCount>
          <MultipartUploadCount>2</MultipartUploadCount>
          <LiveChannelCount>0</LiveChannelCount>
          <LastModifiedTime>1700000000</LastModifiedTime>
          <StandardStorage>512</StandardStorage>
          <StandardObjectCount>5</StandardObjectCount>
          <InfrequentAccessStorage>256</InfrequentAccessStorage>
          <InfrequentAccessObjectCount>3</InfrequentAccessObjectCount>
          <ArchiveStorage>256</ArchiveStorage>
          <ArchiveObjectCount>2</ArchiveObjectCount>
          <ColdArchiveStorage>0</ColdArchiveStorage>
          <ColdArchiveObjectCount>0</ColdArchiveObjectCount>
          <DeepColdArchiveStorage>0</DeepColdArchiveStorage>
          <DeepColdArchiveObjectCount>0</DeepColdArchiveObjectCount>
        </BucketStat>"#;

        let result: GetBucketStatOutput = from_xml(xml).unwrap();
        assert_eq!(result.storage, 1024);
        assert_eq!(result.object_count, 10);
        assert_eq!(result.standard_storage, 512);
    }
}
