use std::fmt;

use async_trait::async_trait;

use crate::error::{OssError, OssErrorKind, Result};

#[derive(Clone)]
pub struct AccessKeyId(String);

impl AccessKeyId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for AccessKeyId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for AccessKeyId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl fmt::Display for AccessKeyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Debug for AccessKeyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("AccessKeyId")
            .field(&mask_string(&self.0))
            .finish()
    }
}

#[derive(Clone)]
pub struct AccessKeySecret(String);

impl AccessKeySecret {
    pub fn new(secret: impl Into<String>) -> Self {
        Self(secret.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for AccessKeySecret {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for AccessKeySecret {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl fmt::Debug for AccessKeySecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("AccessKeySecret").field(&"***").finish()
    }
}

fn mask_string(s: &str) -> String {
    if s.len() <= 8 {
        return "***".to_string();
    }
    let mut masked = String::with_capacity(s.len());
    masked.push_str(&s[..4]);
    masked.push_str("***");
    masked.push_str(&s[s.len() - 4..]);
    masked
}

pub struct Credentials {
    access_key_id: AccessKeyId,
    access_key_secret: AccessKeySecret,
    security_token: Option<String>,
}

impl Credentials {
    pub fn builder() -> CredentialsBuilder {
        CredentialsBuilder::default()
    }

    pub fn access_key_id(&self) -> &str {
        self.access_key_id.as_str()
    }

    pub fn access_key_secret(&self) -> &str {
        self.access_key_secret.as_str()
    }

    pub fn security_token(&self) -> Option<&str> {
        self.security_token.as_deref()
    }
}

impl fmt::Debug for Credentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Credentials")
            .field("access_key_id", &self.access_key_id)
            .field("access_key_secret", &self.access_key_secret)
            .field(
                "security_token",
                &self.security_token.as_ref().map(|_| "***"),
            )
            .finish()
    }
}

#[derive(Default)]
pub struct CredentialsBuilder {
    access_key_id: Option<AccessKeyId>,
    access_key_secret: Option<AccessKeySecret>,
    security_token: Option<String>,
}

impl CredentialsBuilder {
    pub fn access_key_id(mut self, id: impl Into<AccessKeyId>) -> Self {
        self.access_key_id = Some(id.into());
        self
    }

    pub fn access_key_secret(mut self, secret: impl Into<AccessKeySecret>) -> Self {
        self.access_key_secret = Some(secret.into());
        self
    }

    pub fn security_token(mut self, token: impl Into<String>) -> Self {
        self.security_token = Some(token.into());
        self
    }

    pub fn build(self) -> Result<Credentials> {
        Ok(Credentials {
            access_key_id: self.access_key_id.ok_or_else(|| OssError {
                kind: OssErrorKind::CredentialsError,
                context: Box::new(crate::error::ErrorContext {
                    operation: Some("build Credentials".into()),
                    ..Default::default()
                }),
                source: None,
            })?,
            access_key_secret: self.access_key_secret.ok_or_else(|| OssError {
                kind: OssErrorKind::CredentialsError,
                context: Box::new(crate::error::ErrorContext {
                    operation: Some("build Credentials".into()),
                    ..Default::default()
                }),
                source: None,
            })?,
            security_token: self.security_token,
        })
    }
}

#[async_trait]
pub trait CredentialsProvider: Send + Sync {
    async fn credentials(&self) -> Result<Credentials>;
}

pub struct StaticCredentialsProvider {
    credentials: Credentials,
}

impl StaticCredentialsProvider {
    pub fn new(credentials: Credentials) -> Self {
        Self { credentials }
    }
}

#[async_trait]
impl CredentialsProvider for StaticCredentialsProvider {
    async fn credentials(&self) -> Result<Credentials> {
        Ok(Credentials {
            access_key_id: self.credentials.access_key_id.clone(),
            access_key_secret: self.credentials.access_key_secret.clone(),
            security_token: self.credentials.security_token.clone(),
        })
    }
}

pub struct EnvironmentCredentialsProvider;

impl EnvironmentCredentialsProvider {
    const ENV_ACCESS_KEY_ID: &'static str = "OSS_ACCESS_KEY_ID";
    const ENV_ACCESS_KEY_SECRET: &'static str = "OSS_ACCESS_KEY_SECRET";
    const ENV_SECURITY_TOKEN: &'static str = "OSS_SECURITY_TOKEN";
}

#[async_trait]
impl CredentialsProvider for EnvironmentCredentialsProvider {
    async fn credentials(&self) -> Result<Credentials> {
        let access_key_id = std::env::var(Self::ENV_ACCESS_KEY_ID).map_err(|_| OssError {
            kind: OssErrorKind::CredentialsError,
            context: Box::new(crate::error::ErrorContext {
                operation: Some(format!("read {} from environment", Self::ENV_ACCESS_KEY_ID)),
                ..Default::default()
            }),
            source: None,
        })?;

        let access_key_secret =
            std::env::var(Self::ENV_ACCESS_KEY_SECRET).map_err(|_| OssError {
                kind: OssErrorKind::CredentialsError,
                context: Box::new(crate::error::ErrorContext {
                    operation: Some(format!(
                        "read {} from environment",
                        Self::ENV_ACCESS_KEY_SECRET
                    )),
                    ..Default::default()
                }),
                source: None,
            })?;

        let security_token = std::env::var(Self::ENV_SECURITY_TOKEN).ok();

        Credentials::builder()
            .access_key_id(access_key_id)
            .access_key_secret(access_key_secret)
            .security_token(security_token.unwrap_or_default())
            .build()
    }
}

pub struct CredentialsChain {
    providers: Vec<Box<dyn CredentialsProvider>>,
}

impl CredentialsChain {
    pub fn new(providers: Vec<Box<dyn CredentialsProvider>>) -> Self {
        Self { providers }
    }
}

#[async_trait]
impl CredentialsProvider for CredentialsChain {
    async fn credentials(&self) -> Result<Credentials> {
        for provider in &self.providers {
            if let Ok(creds) = provider.credentials().await {
                return Ok(creds);
            }
        }
        Err(OssError {
            kind: OssErrorKind::CredentialsError,
            context: Box::new(crate::error::ErrorContext {
                operation: Some("CredentialsChain".into()),
                ..Default::default()
            }),
            source: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credentials_builder_creates_valid_credentials() {
        let creds = Credentials::builder()
            .access_key_id("test-ak")
            .access_key_secret("test-sk")
            .build()
            .unwrap();
        assert_eq!(creds.access_key_id(), "test-ak");
        assert_eq!(creds.access_key_secret(), "test-sk");
        assert!(creds.security_token().is_none());
    }

    #[test]
    fn credentials_with_security_token() {
        let creds = Credentials::builder()
            .access_key_id("ak")
            .access_key_secret("sk")
            .security_token("token123")
            .build()
            .unwrap();
        assert_eq!(creds.security_token().unwrap(), "token123");
    }

    #[test]
    fn credentials_builder_missing_access_key_id_returns_error() {
        let result = Credentials::builder().access_key_secret("sk").build();
        assert!(result.is_err());
    }

    #[test]
    fn credentials_builder_missing_access_key_secret_returns_error() {
        let result = Credentials::builder().access_key_id("ak").build();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn static_credentials_provider_returns_same_credentials() {
        let creds = Credentials::builder()
            .access_key_id("ak")
            .access_key_secret("sk")
            .build()
            .unwrap();

        let provider = StaticCredentialsProvider::new(creds);
        let retrieved = provider.credentials().await.unwrap();
        assert_eq!(retrieved.access_key_id(), "ak");
        assert_eq!(retrieved.access_key_secret(), "sk");
    }

    #[test]
    fn access_key_secret_debug_does_not_leak_value() {
        let secret = "super-secret-key-12345";
        let ak_secret = AccessKeySecret::from(secret);
        let debug_str = format!("{:?}", ak_secret);
        assert!(!debug_str.contains("super-secret-key-12345"));
        assert!(debug_str.contains("***"));
    }

    #[test]
    fn access_key_id_debug_masks_short_value() {
        let id = AccessKeyId::from("short");
        let debug_str = format!("{:?}", id);
        assert!(debug_str.contains("***"));
    }

    #[test]
    fn access_key_id_debug_masks_long_value() {
        let id = AccessKeyId::from("LTAI5tVeryLongAccessKeyId");
        let debug_str = format!("{:?}", id);
        assert!(!debug_str.contains("VeryLongAccess"));
        assert!(debug_str.starts_with("AccessKeyId"));
        assert!(debug_str.contains("***"));
    }

    #[test]
    fn credentials_debug_does_not_leak_secret() {
        let creds = Credentials::builder()
            .access_key_id("my-ak")
            .access_key_secret("my-secret-sk")
            .security_token("my-token")
            .build()
            .unwrap();
        let debug_str = format!("{:?}", creds);
        assert!(!debug_str.contains("my-secret-sk"));
        assert!(debug_str.contains("***"));
    }

    #[test]
    fn environment_credentials_provider_reads_from_env() {
        let rt = tokio::runtime::Runtime::new().unwrap();

        let provider = EnvironmentCredentialsProvider;
        let result = rt.block_on(provider.credentials());
        assert!(result.is_err());

        unsafe {
            std::env::set_var("OSS_ACCESS_KEY_ID", "env-ak");
            std::env::set_var("OSS_ACCESS_KEY_SECRET", "env-sk");
            std::env::set_var("OSS_SECURITY_TOKEN", "env-token");
        }

        let creds = rt.block_on(provider.credentials()).unwrap();
        assert_eq!(creds.access_key_id(), "env-ak");
        assert_eq!(creds.access_key_secret(), "env-sk");

        unsafe {
            std::env::remove_var("OSS_ACCESS_KEY_ID");
            std::env::remove_var("OSS_ACCESS_KEY_SECRET");
            std::env::remove_var("OSS_SECURITY_TOKEN");
        }
    }

    #[tokio::test]
    async fn credentials_chain_falls_back_to_next_provider() {
        let good_creds = Credentials::builder()
            .access_key_id("good")
            .access_key_secret("good")
            .build()
            .unwrap();

        let provider = StaticCredentialsProvider::new(good_creds);
        let chain = CredentialsChain::new(vec![Box::new(provider)]);

        let retrieved = chain.credentials().await.unwrap();
        assert_eq!(retrieved.access_key_id(), "good");
    }

    #[test]
    fn credentials_chain_exhausted_returns_error() {
        let chain = CredentialsChain::new(vec![]);
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(chain.credentials());
        assert!(result.is_err());
    }

    #[test]
    fn credentials_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Credentials>();
        assert_send_sync::<AccessKeyId>();
        assert_send_sync::<AccessKeySecret>();
        assert_send_sync::<StaticCredentialsProvider>();
        assert_send_sync::<EnvironmentCredentialsProvider>();
    }
}
