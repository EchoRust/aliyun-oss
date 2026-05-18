use std::sync::{Arc, Mutex};

use aliyun_oss::client::OSSClient;
use aliyun_oss::http::client::{HttpClient, HttpRequest, HttpResponse};
use aliyun_oss::types::acl::BucketAcl;
use aliyun_oss::types::region::Region;
use aliyun_oss::types::storage::StorageClass;

mod common;

struct RecordingHttpClient {
    requests: Arc<Mutex<Vec<HttpRequest>>>,
}

#[async_trait::async_trait]
impl HttpClient for RecordingHttpClient {
    async fn send(&self, request: HttpRequest) -> aliyun_oss::error::Result<HttpResponse> {
        self.requests.lock().unwrap().push(request);

        let mut headers = http::HeaderMap::new();
        headers.insert(
            "x-oss-request-id",
            http::HeaderValue::from_static("test-request-id"),
        );

        Ok(HttpResponse {
            status: http::StatusCode::OK,
            headers,
            body: bytes::Bytes::new(),
        })
    }
}

fn create_mock_client() -> (RecordingHttpClient, Arc<Mutex<Vec<HttpRequest>>>) {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let client = RecordingHttpClient {
        requests: requests.clone(),
    };
    (client, requests)
}

#[tokio::test]
async fn client_sends_put_bucket_request_to_mock() {
    let (http, requests) = create_mock_client();

    let client = OSSClient::builder()
        .region(Region::CnHangzhou)
        .credentials("test-ak", "test-sk")
        .http_client(http)
        .build()
        .unwrap();

    let result = client
        .bucket("test-bucket")
        .unwrap()
        .create()
        .acl(BucketAcl::Private)
        .storage_class(StorageClass::Standard)
        .send()
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output.request_id, "test-request-id");

    let captured = requests.lock().unwrap();
    assert_eq!(captured.len(), 1);
    assert_eq!(captured[0].method, http::Method::PUT);
    assert!(captured[0].uri.contains("test-bucket"));
}

#[tokio::test]
async fn client_sends_put_bucket_with_acl_header() {
    let (http, requests) = create_mock_client();

    let client = OSSClient::builder()
        .region(Region::CnHangzhou)
        .credentials("test-ak", "test-sk")
        .http_client(http)
        .build()
        .unwrap();

    let _ = client
        .bucket("my-bucket")
        .unwrap()
        .create()
        .acl(BucketAcl::PublicRead)
        .send()
        .await
        .unwrap();

    let captured = requests.lock().unwrap();
    let acl_value = captured[0]
        .headers
        .get("x-oss-acl")
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(acl_value, "public-read");
}

#[tokio::test]
async fn put_bucket_with_storage_class_includes_xml_body() {
    let (http, requests) = create_mock_client();

    let client = OSSClient::builder()
        .region(Region::CnHangzhou)
        .credentials("test-ak", "test-sk")
        .http_client(http)
        .build()
        .unwrap();

    let _ = client
        .bucket("my-bucket")
        .unwrap()
        .create()
        .storage_class(StorageClass::IA)
        .send()
        .await
        .unwrap();

    let captured = requests.lock().unwrap();
    let body = captured[0].body.as_ref().unwrap();
    let body_str = std::str::from_utf8(body).unwrap();
    assert!(body_str.contains("<StorageClass>IA</StorageClass>"));
}

#[tokio::test]
async fn put_bucket_without_options_sends_empty_body() {
    let (http, requests) = create_mock_client();

    let client = OSSClient::builder()
        .region(Region::CnHangzhou)
        .credentials("test-ak", "test-sk")
        .http_client(http)
        .build()
        .unwrap();

    let _ = client
        .bucket("test-bucket")
        .unwrap()
        .create()
        .send()
        .await
        .unwrap();

    let captured = requests.lock().unwrap();
    assert!(captured[0].body.is_none());
}
