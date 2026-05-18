use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;

use super::client::{HttpClient, HttpRequest, HttpResponse};
use crate::error::Result;

#[async_trait]
pub trait Middleware: Send + Sync {
    async fn handle(&self, request: HttpRequest, chain: &MiddlewareChain) -> Result<HttpResponse>;
}

pub struct MiddlewareChain {
    middlewares: Vec<Box<dyn Middleware>>,
    http: Arc<dyn HttpClient>,
}

impl MiddlewareChain {
    pub fn new(http: Arc<dyn HttpClient>) -> Self {
        Self {
            middlewares: Vec::new(),
            http,
        }
    }

    pub fn with_middleware(mut self, middleware: impl Middleware + 'static) -> Self {
        self.middlewares.push(Box::new(middleware));
        self
    }

    pub async fn send(&self, request: HttpRequest) -> Result<HttpResponse> {
        if self.middlewares.is_empty() {
            return self.http.send(request).await;
        }
        self.middlewares[0].handle(request, self).await
    }

    pub(crate) async fn send_through(
        &self,
        request: HttpRequest,
        start_index: usize,
    ) -> Result<HttpResponse> {
        let next_index = start_index + 1;
        if next_index < self.middlewares.len() {
            self.middlewares[next_index].handle(request, self).await
        } else {
            self.http.send(request).await
        }
    }
}

pub struct SigningMiddleware {
    signer: Arc<dyn crate::signer::Signer>,
    credentials: Arc<dyn crate::config::credentials::CredentialsProvider>,
    region: String,
}

impl SigningMiddleware {
    pub fn new(
        signer: Arc<dyn crate::signer::Signer>,
        credentials: Arc<dyn crate::config::credentials::CredentialsProvider>,
        region: impl Into<String>,
    ) -> Self {
        Self {
            signer,
            credentials,
            region: region.into(),
        }
    }
}

#[async_trait]
impl Middleware for SigningMiddleware {
    async fn handle(&self, request: HttpRequest, chain: &MiddlewareChain) -> Result<HttpResponse> {
        let creds = self.credentials.credentials().await?;

        let headers: Vec<(String, String)> = request
            .headers
            .iter()
            .map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        let mut signing_request = crate::signer::SigningRequest {
            method: request.method.as_str().to_string(),
            uri: extract_path(&request.uri),
            region: self.region.clone(),
            query_params: Vec::new(),
            headers,
            timestamp: chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string(),
        };

        self.signer.sign(&mut signing_request, &creds)?;

        let mut signed_request = HttpRequest::builder()
            .method(request.method.clone())
            .uri(&request.uri);

        for (key, value) in &signing_request.headers {
            if let (Ok(name), Ok(val)) = (
                http::HeaderName::from_bytes(key.as_bytes()),
                http::HeaderValue::from_str(value),
            ) {
                signed_request = signed_request.header(name, val);
            }
        }

        signed_request = signed_request.body(request.body.clone().unwrap_or_default());

        chain.send_through(signed_request.build(), 0).await
    }
}

fn extract_path(uri: &str) -> String {
    if let Some(pos) = uri.find("://") {
        let after_scheme = &uri[pos + 3..];
        if let Some(path_start) = after_scheme.find('/') {
            let path = &after_scheme[path_start..];
            if let Some(q) = path.find('?') {
                return path[..q].to_string();
            }
            return path.to_string();
        }
        return "/".to_string();
    }
    if uri.starts_with('/') {
        let path = if let Some(q) = uri.find('?') {
            &uri[..q]
        } else {
            uri
        };
        return path.to_string();
    }
    "/".to_string()
}

pub struct UserAgentMiddleware {
    user_agent: String,
}

impl UserAgentMiddleware {
    pub fn new(user_agent: impl Into<String>) -> Self {
        Self {
            user_agent: user_agent.into(),
        }
    }
}

#[async_trait]
impl Middleware for UserAgentMiddleware {
    async fn handle(
        &self,
        mut request: HttpRequest,
        chain: &MiddlewareChain,
    ) -> Result<HttpResponse> {
        request.headers.insert(
            http::HeaderName::from_static("user-agent"),
            http::HeaderValue::from_str(&self.user_agent)
                .unwrap_or(http::HeaderValue::from_static("aliyun-oss")),
        );
        chain.send_through(request, 0).await
    }
}

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub base_delay: Duration,
    pub max_backoff: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(100),
            max_backoff: Duration::from_secs(10),
        }
    }
}

impl RetryConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_retries(mut self, max: u32) -> Self {
        self.max_retries = max;
        self
    }

    pub fn with_base_delay(mut self, delay: Duration) -> Self {
        self.base_delay = delay;
        self
    }

    pub fn with_max_backoff(mut self, backoff: Duration) -> Self {
        self.max_backoff = backoff;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::client::ReqwestHttpClient;
    use http::StatusCode;

    #[test]
    fn retry_config_default_values() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.base_delay, Duration::from_millis(100));
        assert_eq!(config.max_backoff, Duration::from_secs(10));
    }

    #[test]
    fn retry_config_builder() {
        let config = RetryConfig::new()
            .with_max_retries(5)
            .with_base_delay(Duration::from_millis(200));
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.base_delay, Duration::from_millis(200));
    }

    #[test]
    fn middleware_chain_no_middleware_passes_to_http() {
        let http = Arc::new(ReqwestHttpClient::default());
        let chain = MiddlewareChain::new(http);
        assert!(chain.middlewares.is_empty());
    }

    #[test]
    fn middleware_chain_with_middleware() {
        let http = Arc::new(ReqwestHttpClient::default());
        let chain =
            MiddlewareChain::new(http).with_middleware(UserAgentMiddleware::new("aliyun-oss/0.1"));
        assert_eq!(chain.middlewares.len(), 1);
    }

    #[test]
    fn extract_path_from_full_url() {
        assert_eq!(
            extract_path("https://oss-cn-hangzhou.aliyuncs.com/bucket/key"),
            "/bucket/key"
        );
        assert_eq!(extract_path("https://oss-cn-hangzhou.aliyuncs.com/"), "/");
    }

    #[test]
    fn extract_path_from_relative() {
        assert_eq!(extract_path("/bucket/key"), "/bucket/key");
        assert_eq!(extract_path("/"), "/");
    }

    #[test]
    fn extract_path_with_query_string() {
        assert_eq!(extract_path("https://example.com/path?a=1&b=2"), "/path");
    }
}
