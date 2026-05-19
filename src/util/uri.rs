pub fn uri_encode(input: &str) -> String {
    let mut result = String::with_capacity(input.len() * 3);
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                result.push(byte as char);
            }
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

pub fn oss_endpoint_url(endpoint: &str, bucket: Option<&str>, key: Option<&str>) -> String {
    match (bucket, key) {
        (Some(bucket), Some(key)) => {
            let encoded_key = uri_encode(key);
            format!("https://{}.{}/{}", bucket, endpoint, encoded_key)
        }
        (Some(bucket), None) => {
            format!("https://{}.{}", bucket, endpoint)
        }
        (None, None) => {
            format!("https://{}", endpoint)
        }
        (None, Some(_)) => {
            format!("https://{}", endpoint)
        }
    }
}

pub fn oss_endpoint_url_http(endpoint: &str, bucket: Option<&str>, key: Option<&str>) -> String {
    let url = oss_endpoint_url(endpoint, bucket, key);
    url.replacen("https://", "http://", 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uri_encode_slash_not_encoded() {
        assert_eq!(uri_encode("/"), "/");
    }

    #[test]
    fn uri_encode_path_with_special_chars() {
        assert_eq!(
            uri_encode("/bucket/对象名.jpg"),
            "/bucket/%E5%AF%B9%E8%B1%A1%E5%90%8D.jpg"
        );
    }

    #[test]
    fn uri_encode_chinese_characters() {
        let encoded = uri_encode("对象名");
        assert_eq!(encoded, "%E5%AF%B9%E8%B1%A1%E5%90%8D");
        assert!(!encoded.contains("对象"));
    }

    #[test]
    fn uri_encode_tilde_not_encoded() {
        assert_eq!(uri_encode("~"), "~");
        assert_eq!(uri_encode("file~name.txt"), "file~name.txt");
    }

    #[test]
    fn uri_encode_space_encoded() {
        assert_eq!(uri_encode("a b"), "a%20b");
    }

    #[test]
    fn uri_encode_reserved_characters() {
        assert_eq!(uri_encode("a+b"), "a%2Bb");
        assert_eq!(uri_encode("a=b"), "a%3Db");
        assert_eq!(uri_encode("a@b"), "a%40b");
        assert_eq!(uri_encode("a:b"), "a%3Ab");
    }

    #[test]
    fn uri_encode_alphanumeric_and_safe_chars_not_encoded() {
        let encoded = uri_encode("abc-123_./~");
        assert_eq!(encoded, "abc-123_./~");
    }

    #[test]
    fn uri_encode_empty() {
        assert_eq!(uri_encode(""), "");
    }

    #[test]
    fn build_endpoint_url_with_bucket_and_object() {
        let url = oss_endpoint_url(
            "oss-cn-hangzhou.aliyuncs.com",
            Some("my-bucket"),
            Some("path/to/object.jpg"),
        );
        assert_eq!(
            url,
            "https://my-bucket.oss-cn-hangzhou.aliyuncs.com/path/to/object.jpg"
        );
    }

    #[test]
    fn build_endpoint_url_with_bucket_and_encoded_object() {
        let url = oss_endpoint_url(
            "oss-cn-hangzhou.aliyuncs.com",
            Some("my-bucket"),
            Some("文件 名.jpg"),
        );
        assert_eq!(
            url,
            "https://my-bucket.oss-cn-hangzhou.aliyuncs.com/%E6%96%87%E4%BB%B6%20%E5%90%8D.jpg"
        );
    }

    #[test]
    fn build_endpoint_url_without_bucket() {
        let url = oss_endpoint_url("oss-cn-hangzhou.aliyuncs.com", None, None);
        assert_eq!(url, "https://oss-cn-hangzhou.aliyuncs.com");
    }

    #[test]
    fn build_endpoint_url_with_bucket_only() {
        let url = oss_endpoint_url("oss-cn-hangzhou.aliyuncs.com", Some("my-bucket"), None);
        assert_eq!(url, "https://my-bucket.oss-cn-hangzhou.aliyuncs.com");
    }

    #[test]
    fn build_endpoint_url_http_variant() {
        let url =
            oss_endpoint_url_http("oss-cn-hangzhou.aliyuncs.com", Some("bucket"), Some("key"));
        assert!(url.starts_with("http://"));
        assert!(!url.starts_with("https://"));
        assert!(url.contains("bucket.oss-cn-hangzhou.aliyuncs.com"));
    }

    #[test]
    fn uri_encode_percent_already_encoded_preserved() {
        let encoded = uri_encode("%E5%AF%B9");
        assert_eq!(encoded, "%25E5%25AF%25B9");
    }
}
