use std::collections::BTreeMap;

use hmac::{Hmac, KeyInit, Mac};
use sha1::Sha1;

use crate::config::credentials::Credentials;
use crate::error::Result;

type HmacSha1 = Hmac<Sha1>;

pub struct V1SigningRequest<'a> {
    pub verb: &'a str,
    pub content_md5: &'a str,
    pub content_type: &'a str,
    pub date: &'a str,
    pub bucket: &'a str,
    pub object_key: &'a str,
    pub oss_headers: &'a [(&'a str, &'a str)],
    pub query_params: &'a [(&'a str, &'a str)],
}

pub struct V1Signer;

impl V1Signer {
    pub fn sign(
        &self,
        request: &V1SigningRequest<'_>,
        credentials: &Credentials,
    ) -> Result<String> {
        let canonicalized_oss_headers = self.build_canonicalized_oss_headers(request.oss_headers);
        let canonicalized_resource = self.build_canonicalized_resource(
            request.bucket,
            request.object_key,
            request.query_params,
        );
        let string_to_sign = self.build_string_to_sign(
            request.verb,
            request.content_md5,
            request.content_type,
            request.date,
            &canonicalized_oss_headers,
            &canonicalized_resource,
        );
        let signature = self.compute_signature(credentials.access_key_secret(), &string_to_sign);

        Ok(format!("OSS {}:{}", credentials.access_key_id(), signature))
    }

    fn build_string_to_sign(
        &self,
        verb: &str,
        content_md5: &str,
        content_type: &str,
        date: &str,
        canonicalized_oss_headers: &str,
        canonicalized_resource: &str,
    ) -> String {
        format!(
            "{}\n{}\n{}\n{}\n{}{}",
            verb,
            content_md5,
            content_type,
            date,
            canonicalized_oss_headers,
            canonicalized_resource
        )
    }

    fn build_canonicalized_oss_headers(&self, headers: &[(&str, &str)]) -> String {
        let mut map: BTreeMap<String, String> = BTreeMap::new();
        for (name, value) in headers {
            let lower = name.to_lowercase();
            if lower.starts_with("x-oss-") {
                map.insert(lower, value.trim().to_string());
            }
        }

        if map.is_empty() {
            return String::new();
        }

        let mut result = String::new();
        for (name, value) in &map {
            result.push_str(&format!("{}:{}\n", name, value));
        }
        result
    }

    fn build_canonicalized_resource(
        &self,
        bucket: &str,
        object_key: &str,
        query_params: &[(&str, &str)],
    ) -> String {
        let resource = if object_key.is_empty() {
            format!("/{}/", bucket)
        } else if object_key.starts_with('/') {
            format!("/{}{}", bucket, object_key)
        } else {
            format!("/{}/{}", bucket, object_key)
        };

        if query_params.is_empty() {
            return resource;
        }

        let mut sorted: BTreeMap<&str, &str> = BTreeMap::new();
        for (k, v) in query_params {
            sorted.insert(k, v);
        }

        let query_str: Vec<String> = sorted
            .iter()
            .map(|(k, v)| {
                if v.is_empty() {
                    k.to_string()
                } else {
                    format!("{}={}", k, v)
                }
            })
            .collect();

        format!("{}?{}", resource, query_str.join("&"))
    }

    fn compute_signature(&self, secret: &str, string_to_sign: &str) -> String {
        let mut mac =
            HmacSha1::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
        mac.update(string_to_sign.as_bytes());
        let result = mac.finalize().into_bytes();
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, result)
    }
}

impl crate::signer::Signer for V1Signer {
    fn sign(
        &self,
        request: &mut crate::signer::SigningRequest,
        credentials: &Credentials,
    ) -> Result<()> {
        let content_md5 = request
            .headers
            .iter()
            .find(|(k, _)| k.to_lowercase() == "content-md5")
            .map(|(_, v)| v.as_str())
            .unwrap_or("");

        let content_type = request
            .headers
            .iter()
            .find(|(k, _)| k.to_lowercase() == "content-type")
            .map(|(_, v)| v.as_str())
            .unwrap_or("");

        let oss_headers: Vec<(&str, &str)> = request
            .headers
            .iter()
            .filter(|(k, _)| k.to_lowercase().starts_with("x-oss-"))
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        let query_refs: Vec<(&str, &str)> = request
            .query_params
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        let (bucket, object_key) = parse_bucket_key(&request.uri);

        let v1_request = V1SigningRequest {
            verb: &request.method,
            content_md5,
            content_type,
            date: &request.timestamp,
            bucket: &bucket,
            object_key: &object_key,
            oss_headers: &oss_headers,
            query_params: &query_refs,
        };

        let auth = self.sign(&v1_request, credentials)?;

        request.headers.push(("Authorization".into(), auth));

        Ok(())
    }
}

fn parse_bucket_key(uri: &str) -> (String, String) {
    let path = uri.trim_start_matches('/');
    if path.is_empty() {
        return (String::new(), String::new());
    }
    if let Some(slash_pos) = path.find('/') {
        (
            path[..slash_pos].to_string(),
            path[slash_pos + 1..].to_string(),
        )
    } else {
        (path.to_string(), String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_credentials() -> Credentials {
        Credentials::builder()
            .access_key_id("test-ak")
            .access_key_secret("test-sk")
            .build()
            .unwrap()
    }

    #[test]
    fn v1_signature_uses_hmac_sha1() {
        let signer = V1Signer;
        let sig = signer.compute_signature("secret", "hello");
        assert!(!sig.is_empty());
        assert_eq!(sig.len(), 28);
    }

    #[test]
    fn v1_string_to_sign_format() {
        let signer = V1Signer;
        let sts = signer.build_string_to_sign(
            "PUT",
            "abc123",
            "text/plain",
            "Mon, 18 May 2026 12:00:00 GMT",
            "x-oss-meta-author:echo\n",
            "/bucket/key",
        );

        let expected = concat!(
            "PUT\n",
            "abc123\n",
            "text/plain\n",
            "Mon, 18 May 2026 12:00:00 GMT\n",
            "x-oss-meta-author:echo\n",
            "/bucket/key"
        );
        assert_eq!(sts, expected);
    }

    #[test]
    fn v1_canonicalized_oss_headers_sorted() {
        let signer = V1Signer;
        let headers = vec![
            ("x-oss-meta-version", "2"),
            ("x-oss-meta-author", "echo"),
            ("Content-Type", "text/plain"),
        ];
        let result = signer.build_canonicalized_oss_headers(&headers);

        let expected = concat!("x-oss-meta-author:echo\n", "x-oss-meta-version:2\n");
        assert_eq!(result, expected);
    }

    #[test]
    fn v1_canonicalized_oss_headers_ignores_non_x_oss() {
        let signer = V1Signer;
        let headers = vec![
            ("Content-Type", "text/plain"),
            ("Content-MD5", "abc"),
            ("Date", "today"),
        ];
        let result = signer.build_canonicalized_oss_headers(&headers);
        assert!(result.is_empty());
    }

    #[test]
    fn v1_canonicalized_resource_with_object() {
        let signer = V1Signer;
        let result = signer.build_canonicalized_resource("mybucket", "path/to/obj.jpg", &[]);
        assert_eq!(result, "/mybucket/path/to/obj.jpg");
    }

    #[test]
    fn v1_canonicalized_resource_without_object() {
        let signer = V1Signer;
        let result = signer.build_canonicalized_resource("mybucket", "", &[]);
        assert_eq!(result, "/mybucket/");
    }

    #[test]
    fn v1_canonicalized_resource_with_query_params() {
        let signer = V1Signer;
        let result = signer.build_canonicalized_resource(
            "mybucket",
            "",
            &[("acl", ""), ("versioning", ""), ("max-keys", "10")],
        );
        assert_eq!(result, "/mybucket/?acl&max-keys=10&versioning");
    }

    #[test]
    fn v1_authorization_header_format() {
        let signer = V1Signer;
        let credentials = test_credentials();
        let request = V1SigningRequest {
            verb: "GET",
            content_md5: "",
            content_type: "",
            date: "Mon, 18 May 2026 12:00:00 GMT",
            bucket: "mybucket",
            object_key: "mykey",
            oss_headers: &[],
            query_params: &[],
        };
        let auth = signer.sign(&request, &credentials).unwrap();

        assert!(auth.starts_with("OSS test-ak:"));
        assert!(!auth.contains("test-sk"));
    }

    #[test]
    fn v1_signer_known_answer() {
        let signer = V1Signer;
        let credentials = Credentials::builder()
            .access_key_id("44CF9590006BF252F707")
            .access_key_secret("OtxrzxIsfpFjA7SwPzILwy8Bw21TLhquhboDYROV")
            .build()
            .unwrap();

        let request = V1SigningRequest {
            verb: "PUT",
            content_md5: "",
            content_type: "text/html",
            date: "Thu, 17 Nov 2005 18:49:58 GMT",
            bucket: "oss-example",
            object_key: "nelson",
            oss_headers: &[
                ("x-oss-meta-author", "foo@bar.com"),
                ("x-oss-magic", "abracadabra"),
            ],
            query_params: &[],
        };
        let auth = signer.sign(&request, &credentials).unwrap();

        assert_eq!(
            auth,
            "OSS 44CF9590006BF252F707:vqsY6+ZQQmL5H9NGFJfVCs66np4="
        );
    }

    #[test]
    fn v1_signer_query_params_encoded_in_resource() {
        let signer = V1Signer;
        let result = signer.build_canonicalized_resource(
            "bucket",
            "key",
            &[
                ("response-content-type", "text/plain"),
                ("response-cache-control", "no-cache"),
            ],
        );
        assert!(result.contains("response-cache-control=no-cache"));
        assert!(result.contains("response-content-type=text/plain"));
    }
}
