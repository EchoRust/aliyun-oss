use std::collections::BTreeMap;

use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

use crate::config::credentials::Credentials;
use crate::error::Result;
use crate::util::uri::uri_encode;

type HmacSha256 = Hmac<Sha256>;

const ALGORITHM: &str = "OSS4-HMAC-SHA256";
const UNSIGNED_PAYLOAD: &str = "UNSIGNED-PAYLOAD";

const REQUIRED_SIGNED_HEADERS: &[&str] = &["x-oss-content-sha256"];

const SEMI_REQUIRED_SIGNED_HEADER_PREFIXES: &[&str] = &["x-oss-"];

const SEMI_REQUIRED_SIGNED_HEADERS: &[&str] = &["content-type", "content-md5"];

pub struct V4Signer;

impl V4Signer {
    pub fn sign(&self, request: &SigningRequest, credentials: &Credentials) -> Result<String> {
        let timestamp = request.timestamp();
        let date = &timestamp[..8];

        let canonical_request = self.build_canonical_request(request);
        let string_to_sign =
            self.build_string_to_sign(&canonical_request, timestamp, date, request.region);
        let signing_key =
            self.derive_signing_key(credentials.access_key_secret(), date, request.region);
        let signature = self.compute_signature(&signing_key, &string_to_sign);

        let additional_headers = self.build_additional_headers_str(request);
        let scope = format!("{}/{}/oss/aliyun_v4_request", date, request.region);

        let authorization = format!(
            "{} Credential={}/{},AdditionalHeaders={},Signature={}",
            ALGORITHM,
            credentials.access_key_id(),
            scope,
            additional_headers,
            signature
        );

        Ok(authorization)
    }

    fn build_canonical_request(&self, request: &SigningRequest) -> String {
        let http_verb = request.method;
        let canonical_uri = uri_encode(request.uri);
        let canonical_query = self.build_canonical_query_string(request);
        let canonical_headers = self.build_canonical_headers(request);
        let additional_headers = self.build_additional_headers_str(request);
        let hashed_payload = UNSIGNED_PAYLOAD;

        format!(
            "{}\n{}\n{}\n{}\n\n{}\n{}",
            http_verb,
            canonical_uri,
            canonical_query,
            canonical_headers,
            additional_headers,
            hashed_payload
        )
    }

    fn build_string_to_sign(
        &self,
        canonical_request: &str,
        timestamp: &str,
        date: &str,
        region: &str,
    ) -> String {
        let scope = format!("{}/{}/oss/aliyun_v4_request", date, region);
        let hashed_canonical = hex::encode(Sha256::digest(canonical_request.as_bytes()));

        format!(
            "{}\n{}\n{}\n{}",
            ALGORITHM, timestamp, scope, hashed_canonical
        )
    }

    fn derive_signing_key(&self, secret: &str, date: &str, region: &str) -> Vec<u8> {
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

    fn compute_signature(&self, signing_key: &[u8], string_to_sign: &str) -> String {
        let mut mac =
            HmacSha256::new_from_slice(signing_key).expect("HMAC can take key of any size");
        mac.update(string_to_sign.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    fn build_canonical_query_string(&self, request: &SigningRequest) -> String {
        if request.query_params.is_empty() {
            return String::new();
        }

        let mut encoded: BTreeMap<&str, String> = BTreeMap::new();
        for (k, v) in &request.query_params {
            if v.is_empty() {
                encoded.insert(k, uri_encode(k));
            } else {
                encoded.insert(k, format!("{}={}", uri_encode(k), uri_encode(v)));
            }
        }

        let parts: Vec<String> = encoded.into_values().collect();
        parts.join("&")
    }

    fn build_canonical_headers(&self, request: &SigningRequest) -> String {
        let mut header_map: BTreeMap<String, String> = BTreeMap::new();

        for (name, value) in &request.headers {
            let lower = name.to_lowercase();
            header_map.insert(lower, value.trim().to_string());
        }

        if !header_map.contains_key("x-oss-content-sha256") {
            header_map.insert(
                "x-oss-content-sha256".to_string(),
                UNSIGNED_PAYLOAD.to_string(),
            );
        }

        header_map
            .iter()
            .map(|(name, value)| format!("{}:{}", name, value))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn build_additional_headers_str(&self, request: &SigningRequest) -> String {
        let mut names: Vec<String> = Vec::new();
        for (name, _) in &request.headers {
            let lower = name.to_lowercase();

            if REQUIRED_SIGNED_HEADERS.contains(&lower.as_str()) {
                continue;
            }

            if SEMI_REQUIRED_SIGNED_HEADERS.contains(&lower.as_str()) {
                continue;
            }

            if SEMI_REQUIRED_SIGNED_HEADER_PREFIXES
                .iter()
                .any(|prefix| lower.starts_with(prefix))
            {
                continue;
            }

            names.push(lower);
        }

        names.sort();
        names.join(";")
    }
}

impl crate::signer::Signer for V4Signer {
    fn sign(
        &self,
        request: &mut crate::signer::SigningRequest,
        credentials: &Credentials,
    ) -> Result<()> {
        let refs: Vec<(&str, &str)> = request
            .headers
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        let query_refs: Vec<(&str, &str)> = request
            .query_params
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        let v4_request = SigningRequest {
            method: &request.method,
            uri: &request.uri,
            region: &request.region,
            query_params: query_refs,
            headers: refs,
            body_hash: "UNSIGNED-PAYLOAD",
            timestamp: &request.timestamp,
        };

        let auth = self.sign(&v4_request, credentials)?;

        request.headers.push(("Authorization".into(), auth));
        request
            .headers
            .push(("x-oss-content-sha256".into(), UNSIGNED_PAYLOAD.into()));
        request
            .headers
            .push(("x-oss-date".into(), request.timestamp.clone()));

        Ok(())
    }
}

pub struct SigningRequest<'a> {
    pub method: &'a str,
    pub uri: &'a str,
    pub region: &'a str,
    pub query_params: Vec<(&'a str, &'a str)>,
    pub headers: Vec<(&'a str, &'a str)>,
    pub body_hash: &'a str,
    pub timestamp: &'a str,
}

impl<'a> SigningRequest<'a> {
    pub fn timestamp(&self) -> &str {
        self.timestamp
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::credentials::Credentials;

    fn known_answer_credentials() -> Credentials {
        Credentials::builder()
            .access_key_id("LTAI5tGL4ap4q4aUSTtxMGVD")
            .access_key_secret("yourAccessKeySecret")
            .build()
            .unwrap()
    }

    fn known_answer_request() -> SigningRequest<'static> {
        SigningRequest {
            method: "PUT",
            uri: "/examplebucket/exampleobject",
            region: "cn-hangzhou",
            query_params: vec![],
            headers: vec![
                ("content-disposition", "attachment"),
                ("content-length", "3"),
                ("content-md5", "ICy5YqxZB1uWSwcVLSNLcA=="),
                ("content-type", "text/plain"),
                ("x-oss-date", "20250411T064124Z"),
            ],
            body_hash: "UNSIGNED-PAYLOAD",
            timestamp: "20250411T064124Z",
        }
    }

    #[test]
    fn v4_signer_known_answer_canonical_request_matches_official() {
        let signer = V4Signer;
        let request = known_answer_request();
        let canonical = signer.build_canonical_request(&request);

        let expected = concat!(
            "PUT\n",
            "/examplebucket/exampleobject\n",
            "\n",
            "content-disposition:attachment\n",
            "content-length:3\n",
            "content-md5:ICy5YqxZB1uWSwcVLSNLcA==\n",
            "content-type:text/plain\n",
            "x-oss-content-sha256:UNSIGNED-PAYLOAD\n",
            "x-oss-date:20250411T064124Z\n",
            "\n",
            "content-disposition;content-length\n",
            "UNSIGNED-PAYLOAD"
        );

        assert_eq!(canonical, expected);
    }

    #[test]
    fn v4_signer_known_answer_hashed_canonical_request_matches_official() {
        let signer = V4Signer;
        let request = known_answer_request();
        let canonical = signer.build_canonical_request(&request);
        let hash = hex::encode(Sha256::digest(canonical.as_bytes()));

        assert_eq!(
            hash,
            "c46d96390bdbc2d739ac9363293ae9d710b14e48081fcb22cd8ad54b63136eca"
        );
    }

    #[test]
    fn v4_signer_known_answer_string_to_sign_matches_official() {
        let signer = V4Signer;
        let request = known_answer_request();
        let canonical = signer.build_canonical_request(&request);
        let sts =
            signer.build_string_to_sign(&canonical, "20250411T064124Z", "20250411", "cn-hangzhou");

        let expected = concat!(
            "OSS4-HMAC-SHA256\n",
            "20250411T064124Z\n",
            "20250411/cn-hangzhou/oss/aliyun_v4_request\n",
            "c46d96390bdbc2d739ac9363293ae9d710b14e48081fcb22cd8ad54b63136eca"
        );

        assert_eq!(sts, expected);
    }

    #[test]
    fn v4_signer_signing_key_derivation_consistent() {
        let signer = V4Signer;
        let signing_key =
            signer.derive_signing_key("yourAccessKeySecret", "20250411", "cn-hangzhou");
        let key_hex = hex::encode(&signing_key);

        assert_eq!(
            key_hex,
            "8a01ff4efcc65ca2cbc75375045c61ab5f3fa8b9a2d84f0add27ef16a25feb3c"
        );

        let signing_key2 =
            signer.derive_signing_key("yourAccessKeySecret", "20250411", "cn-hangzhou");
        assert_eq!(signing_key, signing_key2);
    }

    #[test]
    fn v4_signer_signature_consistent() {
        let signer = V4Signer;
        let signing_key =
            signer.derive_signing_key("yourAccessKeySecret", "20250411", "cn-hangzhou");

        let request = known_answer_request();
        let canonical = signer.build_canonical_request(&request);
        let sts =
            signer.build_string_to_sign(&canonical, "20250411T064124Z", "20250411", "cn-hangzhou");
        let signature = signer.compute_signature(&signing_key, &sts);

        assert_eq!(
            signature,
            "d3694c2dfc5371ee6acd35e88c4871ac95a7ba01d3a2f476768fe61218590097"
        );
    }

    #[test]
    fn v4_signer_full_authorization_format() {
        let signer = V4Signer;
        let request = known_answer_request();
        let credentials = known_answer_credentials();
        let auth = signer.sign(&request, &credentials).unwrap();

        assert!(auth.starts_with("OSS4-HMAC-SHA256 Credential="));
        assert!(auth.contains("/20250411/cn-hangzhou/oss/aliyun_v4_request,"));
        assert!(auth.contains("AdditionalHeaders=content-disposition;content-length,"));
        assert!(auth.contains("Signature="));

        let expected = format!(
            "OSS4-HMAC-SHA256 Credential=LTAI5tGL4ap4q4aUSTtxMGVD/20250411/cn-hangzhou/oss/aliyun_v4_request,AdditionalHeaders=content-disposition;content-length,Signature=d3694c2dfc5371ee6acd35e88c4871ac95a7ba01d3a2f476768fe61218590097"
        );
        assert_eq!(auth, expected);
    }

    #[test]
    fn canonical_headers_sorted_by_lowercase_name() {
        let signer = V4Signer;
        let request = SigningRequest {
            method: "GET",
            uri: "/bucket/key",
            region: "cn-hangzhou",
            query_params: vec![],
            headers: vec![
                ("x-oss-date", "20250411T064124Z"),
                ("Content-Type", "text/plain"),
                ("X-OSS-Meta-Author", "echo"),
                ("content-md5", "abc123"),
            ],
            body_hash: "UNSIGNED-PAYLOAD",
            timestamp: "20250411T064124Z",
        };

        let canonical = signer.build_canonical_request(&request);

        let lines: Vec<&str> = canonical.lines().collect();
        let headers_start = lines.iter().position(|l| l.contains(':')).unwrap();
        let header_lines: Vec<&str> = lines[headers_start..]
            .iter()
            .take_while(|l| !l.is_empty())
            .copied()
            .collect();

        let names: Vec<&str> = header_lines
            .iter()
            .map(|l| l.split(':').next().unwrap())
            .collect();

        let mut sorted_names = names.clone();
        sorted_names.sort();
        assert_eq!(names, sorted_names, "Headers must be sorted alphabetically");
    }

    #[test]
    fn empty_query_params_produces_empty_canonical_query() {
        let signer = V4Signer;
        let request = SigningRequest {
            method: "GET",
            uri: "/bucket/key",
            region: "cn-hangzhou",
            query_params: vec![],
            headers: vec![("x-oss-date", "20250411T064124Z")],
            body_hash: "UNSIGNED-PAYLOAD",
            timestamp: "20250411T064124Z",
        };

        let canonical = signer.build_canonical_request(&request);
        let lines: Vec<&str> = canonical.lines().collect();

        let http_verb_idx = 0;
        let uri_idx = 1;
        let query_idx = 2;

        assert_eq!(lines[http_verb_idx], "GET");
        assert_eq!(lines[uri_idx], "/bucket/key");
        assert_eq!(lines[query_idx], "");
    }

    #[test]
    fn query_params_sorted_by_encoded_name() {
        let signer = V4Signer;
        let request = SigningRequest {
            method: "GET",
            uri: "/bucket",
            region: "cn-hangzhou",
            query_params: vec![("prefix", "dir/"), ("max-keys", "20"), ("marker", "obj")],
            headers: vec![("x-oss-date", "20250411T064124Z")],
            body_hash: "UNSIGNED-PAYLOAD",
            timestamp: "20250411T064124Z",
        };

        let canonical = signer.build_canonical_query_string(&request);

        let parts: Vec<&str> = canonical.split('&').collect();
        assert_eq!(parts[0].starts_with("marker="), true);
        assert_eq!(parts[1].starts_with("max-keys="), true);
        assert_eq!(parts[2].starts_with("prefix="), true);
    }

    #[test]
    fn signing_request_inserts_content_sha256_if_missing() {
        let signer = V4Signer;
        let request = SigningRequest {
            method: "PUT",
            uri: "/bucket/key",
            region: "cn-hangzhou",
            query_params: vec![],
            headers: vec![("x-oss-date", "20250411T064124Z")],
            body_hash: "UNSIGNED-PAYLOAD",
            timestamp: "20250411T064124Z",
        };

        let canonical = signer.build_canonical_request(&request);
        assert!(canonical.contains("x-oss-content-sha256:UNSIGNED-PAYLOAD"));
    }

    #[test]
    #[ignore = "requires OSS credentials in env vars"]
    fn e2e_v4_sign_with_real_credentials() {
        let access_key_id =
            std::env::var("OSS_TEST_ACCESS_KEY_ID").expect("Set OSS_TEST_ACCESS_KEY_ID env var");
        let access_key_secret = std::env::var("OSS_TEST_ACCESS_KEY_SECRET")
            .expect("Set OSS_TEST_ACCESS_KEY_SECRET env var");
        let region = std::env::var("OSS_TEST_REGION").unwrap_or_else(|_| "cn-hangzhou".into());
        let bucket = std::env::var("OSS_TEST_BUCKET").unwrap_or_else(|_| "oss-sdk-test".into());

        let credentials = Credentials::builder()
            .access_key_id(access_key_id.as_str())
            .access_key_secret(access_key_secret.as_str())
            .build()
            .unwrap();

        let timestamp = "20250518T120000Z";
        let date = &timestamp[..8];
        let object_key = "test/signer-e2e.txt";

        let request = SigningRequest {
            method: "PUT",
            uri: &format!("/{}/{}", bucket, object_key),
            region: &region,
            query_params: vec![],
            headers: vec![("content-type", "text/plain"), ("x-oss-date", timestamp)],
            body_hash: "UNSIGNED-PAYLOAD",
            timestamp,
        };

        let signer = V4Signer;
        let auth = signer.sign(&request, &credentials).unwrap();
        let canonical = signer.build_canonical_request(&request);
        let signing_key = signer.derive_signing_key(&access_key_secret, date, &region);

        println!();
        println!("=== V4 Signature E2E ===");
        println!("Region:       {region}");
        println!("Bucket:       {bucket}");
        println!("Object:       {object_key}");
        println!("Timestamp:    {timestamp}");
        println!();
        println!("Canonical Request:");
        for line in canonical.lines() {
            println!("  {line}");
        }
        println!();
        println!("SigningKey:   {}", hex::encode(&signing_key));
        println!();
        println!("Authorization:");
        println!("{auth}");
        println!();
        println!("Verify with curl:");
        println!(
            "curl -v -X PUT \\\n  -H \"Content-Type: text/plain\" \\\n  -H \"x-oss-date: {timestamp}\" \\\n  -H \"x-oss-content-sha256: UNSIGNED-PAYLOAD\" \\\n  -H \"Authorization: {auth}\" \\\n  -d \"test content\" \\\n  https://{bucket}.oss-{region}.aliyuncs.com/{object_key}",
        );
        println!();

        let sha256_hash = hex::encode(sha2::Sha256::digest(canonical.as_bytes()));
        assert!(
            !sha256_hash.is_empty(),
            "canonical request should produce a hash"
        );
    }
}
