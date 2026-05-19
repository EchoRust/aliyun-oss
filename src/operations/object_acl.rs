use std::sync::Arc;

use crate::client::{BucketOperations, OSSClientInner};
use crate::error::{ErrorContext, OssError, OssErrorKind, Result};
use crate::http::client::HttpRequest;
use crate::types::acl::ObjectAcl;
use crate::types::bucket::BucketName;
use crate::types::object::ObjectKey;
use crate::util::uri::oss_endpoint_url;

pub struct GetObjectAclBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    key: ObjectKey,
    version_id: Option<String>,
}

impl GetObjectAclBuilder {
    pub(crate) fn new(client: Arc<OSSClientInner>, bucket: BucketName, key: ObjectKey) -> Self {
        Self {
            client,
            bucket,
            key,
            version_id: None,
        }
    }

    pub fn version_id(mut self, id: impl Into<String>) -> Self {
        self.version_id = Some(id.into());
        self
    }

    pub async fn send(self) -> Result<crate::types::response::GetObjectAclOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = oss_endpoint_url(
            &endpoint,
            Some(self.bucket.as_str()),
            Some(self.key.as_str()),
        );

        let mut query_pairs: Vec<(String, String)> = vec![("acl".into(), String::new())];
        if let Some(ref vid) = self.version_id {
            query_pairs.push(("versionId".into(), vid.clone()));
        }

        let parts: Vec<String> = query_pairs
            .iter()
            .filter(|(_, v)| !v.is_empty())
            .map(|(k, v)| format!("{}={}", k, v))
            .chain(
                query_pairs
                    .iter()
                    .filter(|(_, v)| v.is_empty())
                    .map(|(k, _)| k.clone()),
            )
            .collect();
        let query_string = if parts.is_empty() {
            String::new()
        } else {
            format!("?{}", parts.join("&"))
        };
        let full_uri = format!("{}{}", uri, query_string);

        let request = HttpRequest::builder()
            .method(http::Method::GET)
            .uri(&full_uri)
            .build();

        let response = self
            .client
            .send_signed(request, Some(&self.bucket), query_pairs)
            .await
            .map_err(|e| OssError {
                kind: OssErrorKind::TransportError,
                context: Box::new(ErrorContext {
                    operation: Some("GetObjectAcl".into()),
                    bucket: Some(self.bucket.to_string()),
                    object_key: Some(self.key.to_string()),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.is_success() {
            let body_str = response.body_as_str().unwrap_or("");
            Ok(crate::util::xml::from_xml(body_str)?)
        } else {
            Err(OssError {
                kind: OssErrorKind::ServiceError(Box::new(crate::error::OssServiceError {
                    status_code: response.status().as_u16(),
                    code: String::new(),
                    message: String::new(),
                    request_id: String::new(),
                    host_id: String::new(),
                    resource: Some(self.key.to_string()),
                    string_to_sign: None,
                })),
                context: Box::new(ErrorContext {
                    operation: Some("GetObjectAcl".into()),
                    bucket: Some(self.bucket.to_string()),
                    object_key: Some(self.key.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

pub struct PutObjectAclBuilder {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
    key: ObjectKey,
    acl: ObjectAcl,
    version_id: Option<String>,
}

impl PutObjectAclBuilder {
    pub(crate) fn new(
        client: Arc<OSSClientInner>,
        bucket: BucketName,
        key: ObjectKey,
        acl: ObjectAcl,
    ) -> Self {
        Self {
            client,
            bucket,
            key,
            acl,
            version_id: None,
        }
    }

    pub fn version_id(mut self, id: impl Into<String>) -> Self {
        self.version_id = Some(id.into());
        self
    }

    pub async fn send(self) -> Result<PutObjectAclOutput> {
        let endpoint = self.client.endpoint.clone();
        let uri = oss_endpoint_url(
            &endpoint,
            Some(self.bucket.as_str()),
            Some(self.key.as_str()),
        );

        let mut query_pairs: Vec<(String, String)> = vec![("acl".into(), String::new())];
        if let Some(ref vid) = self.version_id {
            query_pairs.push(("versionId".into(), vid.clone()));
        }

        let parts: Vec<String> = query_pairs
            .iter()
            .filter(|(_, v)| !v.is_empty())
            .map(|(k, v)| format!("{}={}", k, v))
            .chain(
                query_pairs
                    .iter()
                    .filter(|(_, v)| v.is_empty())
                    .map(|(k, _)| k.clone()),
            )
            .collect();
        let query_string = if parts.is_empty() {
            String::new()
        } else {
            format!("?{}", parts.join("&"))
        };
        let full_uri = format!("{}{}", uri, query_string);

        let body_xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?><AccessControlPolicy><Owner><ID>default</ID></Owner><AccessControlList><Grant>{}</Grant></AccessControlList></AccessControlPolicy>"#,
            self.acl.as_str()
        );

        let request = HttpRequest::builder()
            .method(http::Method::PUT)
            .uri(&full_uri)
            .body(bytes::Bytes::from(body_xml))
            .build();

        let response = self
            .client
            .send_signed(request, Some(&self.bucket), query_pairs)
            .await
            .map_err(|e| OssError {
                kind: OssErrorKind::TransportError,
                context: Box::new(ErrorContext {
                    operation: Some("PutObjectAcl".into()),
                    bucket: Some(self.bucket.to_string()),
                    object_key: Some(self.key.to_string()),
                    ..Default::default()
                }),
                source: Some(Box::new(e)),
            })?;

        if response.is_success() {
            Ok(PutObjectAclOutput {
                request_id: response
                    .headers
                    .get("x-oss-request-id")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("")
                    .to_string(),
            })
        } else {
            Err(OssError {
                kind: OssErrorKind::ServiceError(Box::new(crate::error::OssServiceError {
                    status_code: response.status().as_u16(),
                    code: String::new(),
                    message: String::new(),
                    request_id: String::new(),
                    host_id: String::new(),
                    resource: Some(self.key.to_string()),
                    string_to_sign: None,
                })),
                context: Box::new(ErrorContext {
                    operation: Some("PutObjectAcl".into()),
                    bucket: Some(self.bucket.to_string()),
                    object_key: Some(self.key.to_string()),
                    ..Default::default()
                }),
                source: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct PutObjectAclOutput {
    pub request_id: String,
}

impl BucketOperations {
    pub fn get_object_acl(&self, key: impl Into<String>) -> Result<GetObjectAclBuilder> {
        let object_key = ObjectKey::new(key.into())?;
        Ok(GetObjectAclBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            object_key,
        ))
    }

    pub fn put_object_acl(
        &self,
        key: impl Into<String>,
        acl: ObjectAcl,
    ) -> Result<PutObjectAclBuilder> {
        let object_key = ObjectKey::new(key.into())?;
        Ok(PutObjectAclBuilder::new(
            self.client_inner().clone(),
            self.bucket_name().clone(),
            object_key,
            acl,
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use std::sync::Mutex;

    use crate::client::OSSClientInner;
    use crate::config::credentials::Credentials;
    use crate::http::client::{HttpClient, HttpRequest, HttpResponse};
    use crate::types::region::Region;

    use super::*;

    struct RecordingHttpClient {
        requests: Arc<Mutex<Vec<HttpRequest>>>,
        response_body: bytes::Bytes,
    }

    #[async_trait::async_trait]
    impl HttpClient for RecordingHttpClient {
        async fn send(&self, request: HttpRequest) -> crate::error::Result<HttpResponse> {
            self.requests.lock().unwrap().push(request);
            let mut headers = http::HeaderMap::new();
            headers.insert(
                "x-oss-request-id",
                http::HeaderValue::from_static("rid-acl"),
            );
            Ok(HttpResponse {
                status: http::StatusCode::OK,
                headers,
                body: self.response_body.clone(),
            })
        }
    }

    fn create_test_inner(
        body: bytes::Bytes,
    ) -> (Arc<OSSClientInner>, Arc<Mutex<Vec<HttpRequest>>>) {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let http = Arc::new(RecordingHttpClient {
            requests: requests.clone(),
            response_body: body,
        });
        let credentials = Arc::new(crate::config::credentials::StaticCredentialsProvider::new(
            Credentials::builder()
                .access_key_id("test-ak")
                .access_key_secret("test-sk")
                .build()
                .unwrap(),
        ));
        let inner = Arc::new(OSSClientInner {
            http,
            credentials,
            signer: Arc::from(crate::signer::create_signer(crate::signer::SignVersion::V4)),
            region: Region::CnHangzhou,
            endpoint: "oss-cn-hangzhou.aliyuncs.com".into(),
        });
        (inner, requests)
    }

    #[tokio::test]
    async fn get_object_acl_sends_request() {
        let acl_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<AccessControlPolicy>
  <Owner><ID>owner-id</ID></Owner>
  <AccessControlList><Grant>private</Grant></AccessControlList>
</AccessControlPolicy>"#;

        let (inner, requests) = create_test_inner(bytes::Bytes::from(acl_xml));
        let builder = GetObjectAclBuilder::new(
            inner,
            BucketName::new("test-bucket").unwrap(),
            ObjectKey::new("obj.txt").unwrap(),
        );

        let output = builder.send().await.unwrap();
        assert_eq!(output.acl.grant, "private");

        let captured = requests.lock().unwrap();
        assert!(captured[0].uri.contains("acl"));
    }

    #[tokio::test]
    async fn put_object_acl_sends_with_xml_body() {
        let (inner, requests) = create_test_inner(bytes::Bytes::new());
        let builder = PutObjectAclBuilder::new(
            inner,
            BucketName::new("test-bucket").unwrap(),
            ObjectKey::new("obj.txt").unwrap(),
            ObjectAcl::PublicRead,
        );

        builder.send().await.unwrap();

        let captured = requests.lock().unwrap();
        assert_eq!(captured[0].method, http::Method::PUT);
        let body_str = captured[0]
            .body
            .as_ref()
            .map(|b| String::from_utf8_lossy(b).to_string());
        assert!(body_str.unwrap().contains("public-read"));
    }

    #[tokio::test]
    #[ignore = "requires valid OSS credentials"]
    async fn e2e_object_acl_round_trip() {
        let ak = std::env::var("OSS_ACCESS_KEY_ID").expect("OSS_ACCESS_KEY_ID not set");
        let sk = std::env::var("OSS_ACCESS_KEY_SECRET").expect("OSS_ACCESS_KEY_SECRET not set");
        let region_str = std::env::var("OSS_REGION").unwrap_or_else(|_| "cn-wulanchabu".into());
        let bucket_str = std::env::var("OSS_BUCKET").expect("OSS_BUCKET not set");

        let region = Region::from_str(&region_str).unwrap_or_else(|_| Region::Custom {
            endpoint: format!("oss-{}.aliyuncs.com", region_str),
            region_id: region_str.clone(),
        });

        let client = crate::client::OSSClient::builder()
            .region(region)
            .credentials(ak, sk)
            .build()
            .unwrap();

        let key = format!("test-acl-{}.txt", chrono::Utc::now().timestamp());

        client
            .bucket(&bucket_str)
            .unwrap()
            .put_object(&key)
            .unwrap()
            .body(bytes::Bytes::from("acl test"))
            .send()
            .await
            .unwrap();

        client
            .bucket(&bucket_str)
            .unwrap()
            .put_object_acl(&key, ObjectAcl::PublicRead)
            .unwrap()
            .send()
            .await
            .unwrap();

        let acl_output = client
            .bucket(&bucket_str)
            .unwrap()
            .get_object_acl(&key)
            .unwrap()
            .send()
            .await
            .unwrap();

        assert_eq!(acl_output.acl.grant, "public-read");
        eprintln!(
            "ACL round-trip '{}' succeeded: grant={}",
            key, acl_output.acl.grant
        );

        client
            .bucket(&bucket_str)
            .unwrap()
            .delete_object(&key)
            .unwrap()
            .send()
            .await
            .unwrap();
    }
}
