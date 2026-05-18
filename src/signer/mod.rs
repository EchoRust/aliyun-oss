pub mod v1;
pub mod v4;

use crate::config::credentials::Credentials;
use crate::error::Result;

pub enum SignVersion {
    V1,
    V4,
}

pub struct SigningRequest {
    pub method: String,
    pub uri: String,
    pub region: String,
    pub query_params: Vec<(String, String)>,
    pub headers: Vec<(String, String)>,
    pub timestamp: String,
}

impl SigningRequest {
    pub fn builder() -> SigningRequestBuilder {
        SigningRequestBuilder::default()
    }
}

#[derive(Default)]
pub struct SigningRequestBuilder {
    method: Option<String>,
    uri: Option<String>,
    region: Option<String>,
    query_params: Vec<(String, String)>,
    headers: Vec<(String, String)>,
    timestamp: Option<String>,
}

impl SigningRequestBuilder {
    pub fn method(mut self, method: impl Into<String>) -> Self {
        self.method = Some(method.into());
        self
    }

    pub fn uri(mut self, uri: impl Into<String>) -> Self {
        self.uri = Some(uri.into());
        self
    }

    pub fn region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(region.into());
        self
    }

    pub fn query_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query_params.push((key.into(), value.into()));
        self
    }

    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }

    pub fn timestamp(mut self, ts: impl Into<String>) -> Self {
        self.timestamp = Some(ts.into());
        self
    }

    pub fn build(self) -> Result<SigningRequest> {
        Ok(SigningRequest {
            method: self.method.unwrap_or_default(),
            uri: self.uri.unwrap_or_default(),
            region: self.region.unwrap_or_default(),
            query_params: self.query_params,
            headers: self.headers,
            timestamp: self.timestamp.unwrap_or_default(),
        })
    }
}

pub trait Signer: Send + Sync {
    fn sign(&self, request: &mut SigningRequest, credentials: &Credentials) -> Result<()>;
}

pub fn create_signer(version: SignVersion) -> Box<dyn Signer> {
    match version {
        SignVersion::V4 => Box::new(v4::V4Signer),
        SignVersion::V1 => Box::new(v1::V1Signer),
    }
}

#[cfg(test)]
mod tests {
    use crate::config::credentials::Credentials;

    use super::*;

    #[test]
    fn signer_trait_object_safe() {
        fn use_signer(_signer: &dyn Signer) {}
        let v4 = v4::V4Signer;
        use_signer(&v4);
        let v1 = v1::V1Signer;
        use_signer(&v1);
    }

    #[test]
    fn default_signer_is_v4() {
        let signer = create_signer(SignVersion::V4);
        signer
            .sign(
                &mut SigningRequest {
                    method: "GET".into(),
                    uri: "/bucket/key".into(),
                    region: "cn-hangzhou".into(),
                    query_params: vec![],
                    headers: vec![],
                    timestamp: "20250411T064124Z".into(),
                },
                &Credentials::builder()
                    .access_key_id("test")
                    .access_key_secret("test")
                    .build()
                    .unwrap(),
            )
            .unwrap();
    }

    #[test]
    fn signer_request_builder() {
        let request = SigningRequest::builder()
            .method("PUT")
            .uri("/bucket/obj")
            .region("cn-hangzhou")
            .header("content-type", "text/plain")
            .timestamp("20250411T064124Z")
            .build()
            .unwrap();

        assert_eq!(request.method, "PUT");
        assert_eq!(request.uri, "/bucket/obj");
        assert_eq!(request.region, "cn-hangzhou");
        assert_eq!(request.headers.len(), 1);
    }
}
