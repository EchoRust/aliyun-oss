use std::time::Duration;

use hmac::{Hmac, KeyInit, Mac};
use sha1::Sha1;
use sha2::{Digest, Sha256};

use crate::config::credentials::Credentials;
use crate::error::{ErrorContext, OssError, OssErrorKind, Result};
use crate::util::uri::uri_encode;

type HmacSha256 = Hmac<Sha256>;
type HmacSha1 = Hmac<Sha1>;

const ALGORITHM: &str = "OSS4-HMAC-SHA256";
const MAX_EXPIRES: Duration = Duration::from_secs(7 * 24 * 3600);

fn derive_signing_key(secret: &str, date: &str, region: &str) -> Vec<u8> {
    let prefix = format!("aliyun_v4{}", secret);
    let mut mac =
        HmacSha256::new_from_slice(prefix.as_bytes()).expect("HMAC can take key of any size");
    mac.update(date.as_bytes());
    let date_key = mac.finalize().into_bytes();

    let mut mac = HmacSha256::new_from_slice(&date_key).expect("HMAC can take key of any size");
    mac.update(region.as_bytes());
    let date_region_key = mac.finalize().into_bytes();

    let mut mac =
        HmacSha256::new_from_slice(&date_region_key).expect("HMAC can take key of any size");
    mac.update(b"oss");
    let date_region_service_key = mac.finalize().into_bytes();

    let mut mac = HmacSha256::new_from_slice(&date_region_service_key)
        .expect("HMAC can take key of any size");
    mac.update(b"aliyun_v4_request");
    mac.finalize().into_bytes().to_vec()
}

fn compute_signature(signing_key: &[u8], string_to_sign: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(signing_key).expect("HMAC can take key of any size");
    mac.update(string_to_sign.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

fn compute_v1_signature(secret: &str, string_to_sign: &str) -> String {
    let mut mac =
        HmacSha1::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(string_to_sign.as_bytes());
    base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        mac.finalize().into_bytes().as_slice(),
    )
}

fn query_encode(s: &str) -> String {
    let mut result = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(b as char)
            }
            _ => result.push_str(&format!("%{:02X}", b)),
        }
    }
    result
}

fn url_escape(s: &str) -> String {
    s.replace(':', "%3A")
}

fn url_escape_path(s: &str) -> String {
    let mut result = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                result.push(b as char)
            }
            _ => result.push_str(&format!("%{:02X}", b)),
        }
    }
    result
}

pub struct PreSignedUrlBuilder {
    credentials: Credentials,
    region: String,
    endpoint: String,
    bucket: String,
    key: String,
    method: String,
    expires: Duration,
    content_type: Option<String>,
    content_md5: Option<String>,
    additional_query: Vec<(String, String)>,
}

impl PreSignedUrlBuilder {
    pub(crate) fn new(
        credentials: Credentials,
        region: String,
        endpoint: String,
        bucket: String,
        key: String,
    ) -> Self {
        Self {
            credentials,
            region,
            endpoint,
            bucket,
            key,
            method: "GET".into(),
            expires: Duration::from_secs(3600),
            content_type: None,
            content_md5: None,
            additional_query: Vec::new(),
        }
    }

    pub fn method(mut self, m: impl Into<String>) -> Self {
        self.method = m.into();
        self
    }

    pub fn expires(mut self, d: Duration) -> Self {
        self.expires = d;
        self
    }

    pub fn content_type(mut self, ct: impl Into<String>) -> Self {
        self.content_type = Some(ct.into());
        self
    }

    pub fn content_md5(mut self, md5: impl Into<String>) -> Self {
        self.content_md5 = Some(md5.into());
        self
    }

    pub fn query_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.additional_query.push((key.into(), value.into()));
        self
    }

    pub fn generate(self) -> Result<String> {
        self.generate_v4()
    }

    pub fn generate_v1(self) -> Result<String> {
        if self.expires > MAX_EXPIRES {
            return Err(OssError {
                kind: OssErrorKind::ValidationError,
                context: Box::new(ErrorContext {
                    operation: Some("PreSignedUrl: expires must not exceed 7 days".into()),
                    ..Default::default()
                }),
                source: None,
            });
        }

        let now = chrono::Utc::now();
        let expires_unix = now.timestamp() + self.expires.as_secs() as i64;

        let bucket_str = url_escape(&self.bucket);
        let key_str = url_escape_path(&self.key);
        let resource = format!("/{}/{}", bucket_str, key_str);

        let ct = self.content_type.as_deref().unwrap_or("");
        let md5 = self.content_md5.as_deref().unwrap_or("");

        let string_to_sign = format!(
            "{}\n{}\n{}\n{}\n{}",
            self.method, md5, ct, expires_unix, resource
        );

        let signature = compute_v1_signature(self.credentials.access_key_secret(), &string_to_sign);

        let mut params: Vec<(String, String)> = vec![
            (
                "OSSAccessKeyId".into(),
                self.credentials.access_key_id().to_string(),
            ),
            ("Expires".into(), expires_unix.to_string()),
            ("Signature".into(), signature),
        ];

        if let Some(ref token) = self.credentials.security_token() {
            params.push(("security-token".into(), token.to_string()));
        }

        for (k, v) in &self.additional_query {
            params.push((k.clone(), v.clone()));
        }

        let qs: Vec<String> = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, query_encode(v)))
            .collect();

        Ok(format!(
            "https://{}.{}/{}?{}",
            bucket_str,
            self.endpoint,
            key_str,
            qs.join("&")
        ))
    }

    pub fn generate_v4(self) -> Result<String> {
        if self.expires > MAX_EXPIRES {
            return Err(OssError {
                kind: OssErrorKind::ValidationError,
                context: Box::new(ErrorContext {
                    operation: Some("PreSignedUrl: expires must not exceed 7 days".into()),
                    ..Default::default()
                }),
                source: None,
            });
        }

        let now = chrono::Utc::now();
        let timestamp = now.format("%Y%m%dT%H%M%SZ").to_string();
        let datestamp = &timestamp[..8];
        let expires_unix = now.timestamp() + self.expires.as_secs() as i64;

        let scope = format!("{}/{}/oss/aliyun_v4_request", datestamp, self.region);

        let bucket_str = url_escape(&self.bucket);
        let key_str = url_escape_path(&self.key);

        let mut query_pairs: Vec<(String, String)> = Vec::new();
        query_pairs.push((
            "x-oss-credential".into(),
            format!("{}/{}", self.credentials.access_key_id(), scope),
        ));
        query_pairs.push(("x-oss-date".into(), timestamp.clone()));
        query_pairs.push(("x-oss-expires".into(), expires_unix.to_string()));

        if let Some(ref sts_token) = self.credentials.security_token() {
            query_pairs.push(("x-oss-security-token".into(), sts_token.to_string()));
        }

        for (k, v) in &self.additional_query {
            query_pairs.push((k.clone(), v.clone()));
        }

        let canonical_uri = uri_encode(&format!("/{}/{}", bucket_str, key_str));

        let canonical_query = {
            let mut sorted: Vec<(&str, &str)> = query_pairs
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            sorted.sort_by_key(|(k, _)| *k);
            let parts: Vec<String> = sorted
                .iter()
                .map(|(k, v)| {
                    if v.is_empty() {
                        query_encode(k)
                    } else {
                        format!("{}={}", query_encode(k), query_encode(v))
                    }
                })
                .collect();
            parts.join("&")
        };

        let canonical_headers = {
            let mut headers: Vec<(&str, &str)> = vec![("host", &bucket_str)];
            if let Some(ref ct) = self.content_type {
                headers.push(("content-type", ct));
            }
            headers.sort_by_key(|(n, _)| *n);
            headers
                .iter()
                .map(|(n, v)| format!("{}:{}", n, v.trim()))
                .collect::<Vec<_>>()
                .join("\n")
        };

        let signed_headers = {
            let mut names: Vec<&str> = if self.content_type.is_some() {
                vec!["host", "content-type"]
            } else {
                vec!["host"]
            };
            names.sort();
            names.join(";")
        };

        let canonical_request = format!(
            "{}\n{}\n{}\n{}\n\n{}\nUNSIGNED-PAYLOAD",
            self.method, canonical_uri, canonical_query, canonical_headers, signed_headers
        );

        let hashed_canonical = hex::encode(Sha256::digest(canonical_request.as_bytes()));
        let string_to_sign = format!(
            "{}\n{}\n{}\n{}",
            ALGORITHM, timestamp, scope, hashed_canonical
        );

        let signing_key = derive_signing_key(
            self.credentials.access_key_secret(),
            datestamp,
            &self.region,
        );
        let signature = compute_signature(&signing_key, &string_to_sign);

        query_pairs.push(("x-oss-signature".into(), signature));

        if signed_headers != "host" {
            query_pairs.push(("x-oss-additional-headers".into(), signed_headers));
        }

        let query_string = {
            let mut sorted: Vec<(&str, &str)> = query_pairs
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            sorted.sort_by_key(|(k, _)| *k);
            sorted
                .iter()
                .map(|(k, v)| {
                    if v.is_empty() {
                        k.to_string()
                    } else {
                        format!("{}={}", k, query_encode(v))
                    }
                })
                .collect::<Vec<_>>()
                .join("&")
        };

        Ok(format!(
            "https://{}.{}/{}?{}",
            bucket_str, self.endpoint, key_str, query_string
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::config::credentials::Credentials;
    use crate::types::region::Region;

    use super::*;

    fn test_credentials() -> Credentials {
        Credentials::builder()
            .access_key_id("test-ak")
            .access_key_secret("test-sk")
            .build()
            .unwrap()
    }

    fn test_builder() -> PreSignedUrlBuilder {
        PreSignedUrlBuilder::new(
            test_credentials(),
            "cn-hangzhou".into(),
            "oss-cn-hangzhou.aliyuncs.com".into(),
            "test-bucket".into(),
            "test-key.txt".into(),
        )
    }

    #[test]
    fn presigned_v1_url_contains_required_params() {
        let url = test_builder()
            .method("GET")
            .expires(Duration::from_secs(3600))
            .generate_v1()
            .unwrap();

        assert!(url.starts_with("https://test-bucket.oss-cn-hangzhou.aliyuncs.com/test-key.txt?"));
        assert!(url.contains("OSSAccessKeyId=test-ak"));
        assert!(url.contains("Expires="));
        assert!(url.contains("Signature="));
    }

    #[test]
    fn presigned_v4_url_contains_required_params() {
        let url = test_builder()
            .method("GET")
            .expires(Duration::from_secs(3600))
            .generate_v4()
            .unwrap();

        assert!(url.starts_with("https://test-bucket.oss-cn-hangzhou.aliyuncs.com/test-key.txt?"));
        assert!(url.contains("x-oss-signature="));
        assert!(url.contains("x-oss-credential=test-ak%2F"));
        assert!(url.contains("x-oss-date="));
        assert!(url.contains("x-oss-expires="));
    }

    #[test]
    fn presigned_url_with_additional_query_params() {
        let url = test_builder()
            .method("GET")
            .query_param("response-content-type", "image/jpeg")
            .expires(Duration::from_secs(3600))
            .generate_v1()
            .unwrap();

        assert!(url.contains("response-content-type=image%2Fjpeg"));
    }

    #[test]
    fn presigned_url_with_security_token() {
        let creds = Credentials::builder()
            .access_key_id("ak")
            .access_key_secret("sk")
            .security_token("test-token")
            .build()
            .unwrap();

        let url = PreSignedUrlBuilder::new(
            creds,
            "cn-hangzhou".into(),
            "oss-cn-hangzhou.aliyuncs.com".into(),
            "bucket".into(),
            "key".into(),
        )
        .expires(Duration::from_secs(3600))
        .generate_v1()
        .unwrap();

        assert!(url.contains("security-token=test-token"));
    }

    #[test]
    fn presigned_url_expires_too_long_returns_error() {
        let result = test_builder()
            .expires(Duration::from_secs(8 * 24 * 3600))
            .generate_v1();
        assert!(result.is_err());
    }

    #[tokio::test]
    #[ignore = "requires valid OSS credentials"]
    async fn e2e_presigned_get_url_v1() {
        let ak = std::env::var("OSS_ACCESS_KEY_ID").expect("OSS_ACCESS_KEY_ID not set");
        let sk = std::env::var("OSS_ACCESS_KEY_SECRET").expect("OSS_ACCESS_KEY_SECRET not set");
        let region_str = std::env::var("OSS_REGION").unwrap_or_else(|_| "cn-wulanchabu".into());
        let bucket_str = std::env::var("OSS_BUCKET").expect("OSS_BUCKET not set");

        let region = Region::from_str(&region_str).unwrap_or_else(|_| Region::Custom {
            endpoint: format!("oss-{}.aliyuncs.com", region_str),
            region_id: region_str.clone(),
        });

        let credentials = Credentials::builder()
            .access_key_id(ak)
            .access_key_secret(sk)
            .build()
            .unwrap();

        let url = PreSignedUrlBuilder::new(
            credentials,
            region.region_id().to_string(),
            region.external_endpoint().to_string(),
            bucket_str.clone(),
            "test.txt".into(),
        )
        .method("GET")
        .expires(Duration::from_secs(3600))
        .generate_v1()
        .unwrap();

        let resp = reqwest::get(&url).await.unwrap();
        eprintln!("V1 URL: {}", url);
        eprintln!("Status: {}", resp.status());
        assert!(resp.status().is_success());
    }

    #[tokio::test]
    #[ignore = "requires valid OSS credentials"]
    async fn e2e_presigned_get_url_v4() {
        let ak = std::env::var("OSS_ACCESS_KEY_ID").expect("OSS_ACCESS_KEY_ID not set");
        let sk = std::env::var("OSS_ACCESS_KEY_SECRET").expect("OSS_ACCESS_KEY_SECRET not set");
        let region_str = std::env::var("OSS_REGION").unwrap_or_else(|_| "cn-wulanchabu".into());
        let bucket_str = std::env::var("OSS_BUCKET").expect("OSS_BUCKET not set");

        let region = Region::from_str(&region_str).unwrap_or_else(|_| Region::Custom {
            endpoint: format!("oss-{}.aliyuncs.com", region_str),
            region_id: region_str.clone(),
        });

        let credentials = Credentials::builder()
            .access_key_id(ak)
            .access_key_secret(sk)
            .build()
            .unwrap();

        let url = PreSignedUrlBuilder::new(
            credentials,
            region.region_id().to_string(),
            region.external_endpoint().to_string(),
            bucket_str.clone(),
            "test.txt".into(),
        )
        .method("GET")
        .expires(Duration::from_secs(3600))
        .generate_v4()
        .unwrap();

        eprintln!("V4 URL: {}", url);
        let resp = reqwest::get(&url).await.unwrap();
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        eprintln!("Status: {}", status);
        if !status.is_success() {
            eprintln!("Body: {}", body);
        }
        assert!(status.is_success(), "V4 presigned failed: {}", body);
    }
}
