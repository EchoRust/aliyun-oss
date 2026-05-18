use aliyun_oss::config::credentials::Credentials;
use aliyun_oss::types::region::Region;

pub fn test_credentials() -> Credentials {
    Credentials::builder()
        .access_key_id("test-access-key-id")
        .access_key_secret("test-access-key-secret")
        .build()
        .unwrap()
}

pub fn test_region() -> Region {
    Region::CnHangzhou
}

pub fn test_endpoint(server_addr: &str) -> String {
    server_addr.to_string()
}
