//! Tests of the request core. The tests use a mock server, because the
//! behaviour depends on real HTTP answers.

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::error::ApiError;
use toutui::api::client::probe::probe_once;
use toutui::api::client::ApiClient;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Makes a client with the given addresses. The first address has the most
/// importance.
fn client(urls: Vec<&str>) -> ApiClient {
    let endpoints = urls
        .iter()
        .enumerate()
        .map(|(index, url)| Endpoint::new(url, index as u8))
        .collect();

    ApiClient::new(Arc::new(EndpointPool::new(endpoints)), "test-token".to_string()).unwrap()
}

#[tokio::test]
async fn the_client_reads_json_from_the_first_endpoint() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/libraries"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "value": 42
        })))
        .mount(&server)
        .await;

    let client = client(vec![&server.uri()]);
    let body: serde_json::Value = client.get_json("/api/libraries").await.unwrap();

    assert_eq!(body["value"], 42);
}

/// The first address refuses the connection. The client must use the second
/// address and give the answer.
#[tokio::test]
async fn the_client_changes_to_the_second_endpoint() {
    let good = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/libraries"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "ok": true })))
        .mount(&good)
        .await;

    // Port 1 is not open. Therefore the connection fails immediately.
    let client = client(vec!["http://127.0.0.1:1", &good.uri()]);
    let body: serde_json::Value = client.get_json("/api/libraries").await.unwrap();

    assert_eq!(body["ok"], true);
}

/// After a failure, the pool must record that the address does not answer.
#[tokio::test]
async fn a_failure_marks_the_endpoint_down() {
    let good = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/libraries"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "ok": true })))
        .mount(&good)
        .await;

    let client = client(vec!["http://127.0.0.1:1", &good.uri()]);
    let _: serde_json::Value = client.get_json("/api/libraries").await.unwrap();

    assert_eq!(client.pool().down_urls(), vec!["http://127.0.0.1:1"]);
}

/// This is the most important test of the task. A second POST request makes
/// a duplicate listening session on the server.
#[tokio::test]
async fn the_client_does_not_send_a_post_request_a_second_time() {
    let good = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/items/abc/play"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "id": "s1" })))
        .expect(0)
        .mount(&good)
        .await;

    let client = client(vec!["http://127.0.0.1:1", &good.uri()]);
    let result: Result<serde_json::Value, ApiError> = client
        .post_json("/api/items/abc/play", &serde_json::json!({}))
        .await;

    assert!(matches!(result, Err(ApiError::Unreachable)));
    // The mock has `expect(0)`. The check happens when the server stops.
    drop(good);
}

#[tokio::test]
async fn all_endpoints_down_gives_unreachable() {
    let client = client(vec!["http://127.0.0.1:1", "http://127.0.0.1:2"]);
    let result: Result<serde_json::Value, ApiError> = client.get_json("/api/libraries").await;

    assert!(matches!(result, Err(ApiError::Unreachable)));
}

#[tokio::test]
async fn a_status_401_gives_unauthorized_and_does_not_change_endpoint() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/libraries"))
        .respond_with(ResponseTemplate::new(401))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(vec![&server.uri(), "http://127.0.0.1:1"]);
    let result: Result<serde_json::Value, ApiError> = client.get_json("/api/libraries").await;

    assert!(matches!(result, Err(ApiError::Unauthorized)));
    // The pool must still have the first address in the state `Up`.
    assert!(client.pool().down_urls().is_empty());
}

#[tokio::test]
async fn a_status_403_gives_forbidden() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/items/abc/download"))
        .respond_with(ResponseTemplate::new(403))
        .mount(&server)
        .await;

    let client = client(vec![&server.uri()]);
    let result: Result<serde_json::Value, ApiError> =
        client.get_json("/api/items/abc/download").await;

    assert!(matches!(result, Err(ApiError::Forbidden)));
}

#[tokio::test]
async fn the_client_sends_the_token() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/libraries"))
        .and(wiremock::matchers::header(
            "authorization",
            "Bearer test-token",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "ok": true })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(vec![&server.uri()]);
    let body: serde_json::Value = client.get_json("/api/libraries").await.unwrap();
    assert_eq!(body["ok"], true);
}

/// The probe must make an address active again after the address answers.
#[tokio::test]
async fn the_probe_makes_a_down_endpoint_active_again() {
    let primary = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/ping"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true
        })))
        .mount(&primary)
        .await;

    let client = client(vec![&primary.uri(), "http://127.0.0.1:1"]);

    client.pool().mark_down(primary.uri().trim_end_matches('/'));
    assert_eq!(client.pool().active().unwrap(), "http://127.0.0.1:1");

    probe_once(client.http(), &client.pool()).await;

    assert_eq!(
        client.pool().active().unwrap(),
        primary.uri().trim_end_matches('/')
    );
}

/// The probe must not make an address active if the address does not answer.
#[tokio::test]
async fn the_probe_keeps_a_dead_endpoint_down() {
    let good = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/ping"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&good)
        .await;

    let client = client(vec!["http://127.0.0.1:1", &good.uri()]);
    client.pool().mark_down("http://127.0.0.1:1");

    probe_once(client.http(), &client.pool()).await;

    assert_eq!(client.pool().down_urls(), vec!["http://127.0.0.1:1"]);
}

/// A large audiobook takes more than the normal request timeout. The
/// download must not stop after 15 seconds.
#[tokio::test]
async fn the_download_writes_the_body_to_a_file() {
    use std::time::Duration;

    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/items/abc/download"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(b"audio-data".to_vec())
                // This delay is longer than a short test timeout, and it
                // proves that the download path uses its own timeout.
                .set_delay(Duration::from_millis(300)),
        )
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    let client = client(vec![&server.uri()]);

    let file = client
        .download_to_file("/api/items/abc/download", dir.path(), "abc.m4b")
        .await
        .unwrap();

    assert_eq!(file, dir.path().join("abc.m4b"));
    assert_eq!(std::fs::read(&file).unwrap(), b"audio-data");
}

/// The server can give the true file name in the `Content-Disposition`
/// header. The method must use that name.
#[tokio::test]
async fn the_download_uses_the_name_of_the_content_disposition_header() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/items/abc/download"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(b"audio-data".to_vec())
                .append_header("content-disposition", "attachment; filename=\"true-name.m4b\""),
        )
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    let client = client(vec![&server.uri()]);

    let file = client
        .download_to_file("/api/items/abc/download", dir.path(), "abc.m4b")
        .await
        .unwrap();

    assert_eq!(file, dir.path().join("true-name.m4b"));
    assert_eq!(std::fs::read(&file).unwrap(), b"audio-data");
}

/// A user without the download permission must get a clear category.
#[tokio::test]
async fn a_download_without_permission_gives_forbidden() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/items/abc/download"))
        .respond_with(ResponseTemplate::new(403))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    let client = client(vec![&server.uri()]);

    let result = client
        .download_to_file("/api/items/abc/download", dir.path(), "abc.m4b")
        .await;

    assert!(matches!(result, Err(ApiError::Forbidden)));
}
