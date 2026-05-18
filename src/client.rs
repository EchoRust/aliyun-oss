use std::sync::Arc;

use crate::config::credentials::{CredentialsProvider, StaticCredentialsProvider};
use crate::error::{ErrorContext, OssError, OssErrorKind, Result};
use crate::http::client::{HttpClient, ReqwestHttpClient};
use crate::signer::{SignVersion, Signer, create_signer};
use crate::types::bucket::BucketName;
use crate::types::region::Region;

#[allow(dead_code)]
pub(crate) struct OSSClientInner {
    pub(crate) http: Arc<dyn HttpClient>,
    pub(crate) credentials: Arc<dyn CredentialsProvider>,
    pub(crate) signer: Arc<dyn Signer>,
    pub(crate) region: Region,
    pub(crate) endpoint: String,
}

#[derive(Clone)]
pub struct OSSClient {
    inner: Arc<OSSClientInner>,
}

impl OSSClient {
    pub fn builder() -> OSSClientBuilder {
        OSSClientBuilder::default()
    }

    pub fn region(&self) -> &Region {
        &self.inner.region
    }

    pub fn endpoint(&self) -> &str {
        &self.inner.endpoint
    }

    pub fn bucket(&self, name: impl Into<String>) -> Result<BucketOperations> {
        let bucket = BucketName::new(name.into())?;
        Ok(BucketOperations {
            client: self.inner.clone(),
            bucket,
        })
    }
}

#[derive(Default)]
pub struct OSSClientBuilder {
    region: Option<Region>,
    credentials: Option<Arc<dyn CredentialsProvider>>,
    http: Option<Arc<dyn HttpClient>>,
    signer: Option<Arc<dyn Signer>>,
    endpoint: Option<String>,
}

impl OSSClientBuilder {
    pub fn region(mut self, region: Region) -> Self {
        self.region = Some(region);
        self
    }

    pub fn credentials(
        mut self,
        access_key_id: impl Into<String>,
        access_key_secret: impl Into<String>,
    ) -> Self {
        let creds = crate::config::credentials::Credentials::builder()
            .access_key_id(access_key_id.into())
            .access_key_secret(access_key_secret.into())
            .build()
            .expect("credentials build should succeed");
        self.credentials = Some(Arc::new(StaticCredentialsProvider::new(creds)));
        self
    }

    pub fn credentials_provider(mut self, provider: impl CredentialsProvider + 'static) -> Self {
        self.credentials = Some(Arc::new(provider));
        self
    }

    pub fn http_client(mut self, client: impl HttpClient + 'static) -> Self {
        self.http = Some(Arc::new(client));
        self
    }

    pub fn signer(mut self, signer: impl Signer + 'static) -> Self {
        self.signer = Some(Arc::new(signer));
        self
    }

    pub fn endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }

    pub fn build(self) -> Result<OSSClient> {
        let region = self.region.ok_or_else(|| OssError {
            kind: OssErrorKind::ConfigError,
            context: Box::new(ErrorContext {
                operation: Some("OSSClientBuilder: region is required".into()),
                ..Default::default()
            }),
            source: None,
        })?;

        let credentials = self.credentials.ok_or_else(|| OssError {
            kind: OssErrorKind::ConfigError,
            context: Box::new(ErrorContext {
                operation: Some("OSSClientBuilder: credentials are required".into()),
                ..Default::default()
            }),
            source: None,
        })?;

        let http = self
            .http
            .unwrap_or_else(|| Arc::new(ReqwestHttpClient::default()));

        let signer = self
            .signer
            .unwrap_or_else(|| Arc::from(create_signer(SignVersion::V4)));

        let endpoint = self
            .endpoint
            .unwrap_or_else(|| region.external_endpoint().to_string());

        Ok(OSSClient {
            inner: Arc::new(OSSClientInner {
                http,
                credentials,
                signer,
                region,
                endpoint,
            }),
        })
    }
}

#[allow(dead_code)]
pub struct BucketOperations {
    client: Arc<OSSClientInner>,
    bucket: BucketName,
}

#[allow(dead_code)]
impl BucketOperations {
    pub fn bucket_name(&self) -> &BucketName {
        &self.bucket
    }

    pub(crate) fn client_inner(&self) -> &Arc<OSSClientInner> {
        &self.client
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::credentials::Credentials;
    use crate::http::client::{HttpRequest, HttpResponse};
    use crate::types::region::Region;

    struct MockHttpClient;

    #[async_trait::async_trait]
    impl HttpClient for MockHttpClient {
        async fn send(&self, _request: HttpRequest) -> Result<HttpResponse> {
            Ok(HttpResponse::new(http::StatusCode::OK))
        }
    }

    #[test]
    fn oss_client_builder_requires_region_and_credentials() {
        let client = OSSClient::builder()
            .region(Region::CnHangzhou)
            .credentials("test-ak", "test-sk")
            .http_client(MockHttpClient)
            .build();

        assert!(client.is_ok());
    }

    #[test]
    fn oss_client_builder_missing_region_returns_error() {
        let client = OSSClient::builder()
            .credentials("test-ak", "test-sk")
            .http_client(MockHttpClient)
            .build();

        assert!(client.is_err());
    }

    #[test]
    fn oss_client_builder_missing_credentials_returns_error() {
        let client = OSSClient::builder()
            .region(Region::CnHangzhou)
            .http_client(MockHttpClient)
            .build();

        assert!(client.is_err());
    }

    #[test]
    fn oss_client_builder_default_endpoint_from_region() {
        let client = OSSClient::builder()
            .region(Region::CnHangzhou)
            .credentials("ak", "sk")
            .http_client(MockHttpClient)
            .build()
            .unwrap();

        assert_eq!(client.endpoint(), "oss-cn-hangzhou.aliyuncs.com");
    }

    #[test]
    fn oss_client_builder_custom_endpoint() {
        let client = OSSClient::builder()
            .region(Region::CnHangzhou)
            .credentials("ak", "sk")
            .http_client(MockHttpClient)
            .endpoint("custom.oss.example.com")
            .build()
            .unwrap();

        assert_eq!(client.endpoint(), "custom.oss.example.com");
    }

    #[test]
    fn oss_client_bucket_accessor_returns_bucket_operations() {
        let client = OSSClient::builder()
            .region(Region::CnHangzhou)
            .credentials("ak", "sk")
            .http_client(MockHttpClient)
            .build()
            .unwrap();

        let bucket_ops = client.bucket("my-bucket").unwrap();
        assert_eq!(bucket_ops.bucket_name().as_str(), "my-bucket");
    }

    #[test]
    fn oss_client_bucket_accessor_rejects_invalid_bucket() {
        let client = OSSClient::builder()
            .region(Region::CnHangzhou)
            .credentials("ak", "sk")
            .http_client(MockHttpClient)
            .build()
            .unwrap();

        let result = client.bucket("AB");
        assert!(result.is_err());
    }

    #[test]
    fn oss_client_clone() {
        let client = OSSClient::builder()
            .region(Region::CnHangzhou)
            .credentials("ak", "sk")
            .http_client(MockHttpClient)
            .build()
            .unwrap();

        let cloned = client.clone();
        assert_eq!(cloned.region(), &Region::CnHangzhou);
        assert_eq!(cloned.endpoint(), client.endpoint());
    }

    #[test]
    fn oss_client_with_credentials_provider() {
        let credentials = Credentials::builder()
            .access_key_id("ak")
            .access_key_secret("sk")
            .build()
            .unwrap();
        let provider = StaticCredentialsProvider::new(credentials);

        let client = OSSClient::builder()
            .region(Region::CnShanghai)
            .credentials_provider(provider)
            .http_client(MockHttpClient)
            .build()
            .unwrap();

        assert_eq!(client.region(), &Region::CnShanghai);
    }

    #[test]
    fn oss_client_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<OSSClient>();
        assert_send_sync::<BucketOperations>();
    }
}
