use aliyun_oss::config::credentials::Credentials;
use aliyun_oss::signer::v4::{SigningRequest, V4Signer};
use aliyun_oss::signer::{SignVersion, Signer, SigningRequest as UnifiedRequest};
use aliyun_oss::types::region::Region;

mod common;

#[test]
fn v4_signs_put_object_request() {
    let credentials = common::test_credentials();
    let region = common::test_region();

    let request = SigningRequest {
        method: "PUT",
        uri: "/test-bucket/test-key.txt",
        region: region.region_id(),
        query_params: vec![],
        headers: vec![
            ("content-type", "text/plain"),
            ("x-oss-date", "20250411T064124Z"),
        ],
        body_hash: "UNSIGNED-PAYLOAD",
        timestamp: "20250411T064124Z",
    };

    let signer = V4Signer;
    let auth = signer.sign(&request, &credentials).unwrap();

    assert!(auth.starts_with("OSS4-HMAC-SHA256 Credential="));
    assert!(auth.contains("/cn-hangzhou/oss/aliyun_v4_request,"));
    assert!(auth.contains("Signature="));
}

#[test]
fn unified_signer_v4_produces_authorization_header() {
    let credentials = common::test_credentials();
    let region = common::test_region();

    let mut request = UnifiedRequest {
        method: "PUT".into(),
        uri: "/test-bucket/test-key.txt".into(),
        region: region.region_id().into(),
        query_params: vec![],
        headers: vec![
            ("content-type".into(), "text/plain".into()),
            ("x-oss-date".into(), "20250411T064124Z".into()),
        ],
        timestamp: "20250411T064124Z".into(),
    };

    let signer = aliyun_oss::signer::create_signer(SignVersion::V4);
    signer.sign(&mut request, &credentials).unwrap();

    let auth = request
        .headers
        .iter()
        .find(|(k, _)| k == "Authorization")
        .expect("Authorization header should be added");

    assert!(auth.1.starts_with("OSS4-HMAC-SHA256 Credential="));
}

#[test]
fn unified_signer_v1_produces_authorization_header() {
    let credentials = common::test_credentials();
    let region = common::test_region();

    let mut request = UnifiedRequest {
        method: "GET".into(),
        uri: "/test-bucket/test-key.txt".into(),
        region: region.region_id().into(),
        query_params: vec![],
        headers: vec![("x-oss-date".into(), "Wed, 18 May 2026 12:00:00 GMT".into())],
        timestamp: "Wed, 18 May 2026 12:00:00 GMT".into(),
    };

    let signer = aliyun_oss::signer::create_signer(SignVersion::V1);
    signer.sign(&mut request, &credentials).unwrap();

    let auth = request
        .headers
        .iter()
        .find(|(k, _)| k == "Authorization")
        .expect("Authorization header should be added");

    assert!(auth.1.starts_with("OSS "));
}

#[test]
fn signer_pipeline_sets_required_oss_headers() {
    let credentials = common::test_credentials();
    let region = common::test_region();

    let mut request = UnifiedRequest {
        method: "PUT".into(),
        uri: "/test-bucket/test-key.txt".into(),
        region: region.region_id().into(),
        query_params: vec![],
        headers: vec![
            ("content-type".into(), "text/plain".into()),
            ("x-oss-date".into(), "20250411T064124Z".into()),
        ],
        timestamp: "20250411T064124Z".into(),
    };

    let signer = aliyun_oss::signer::create_signer(SignVersion::V4);
    signer.sign(&mut request, &credentials).unwrap();

    let has_content_sha256 = request
        .headers
        .iter()
        .any(|(k, v)| k == "x-oss-content-sha256" && v == "UNSIGNED-PAYLOAD");
    assert!(
        has_content_sha256,
        "x-oss-content-sha256 header should be set"
    );
}
