//! Tests of the requests that write the listening position.
//!
//! A measurement with Audiobookshelf 2.36.0 on 2026-08-10 shows that the
//! server does not always mark a media as finished when the body holds
//! `isFinished` together with `progress` and `currentTime`. The sequence of
//! the keys changes the result, and `serde_json` writes the keys in the
//! sequence of the alphabet.
//!
//! Therefore the application sends the mark in its own request. These tests
//! guard that behaviour.

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::api::me::update_media_progress::*;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(url: &str) -> ApiClient {
    let pool = EndpointPool::new(vec![Endpoint::new(url, 0)]);
    ApiClient::new(Arc::new(pool), "test-token".to_string()).unwrap()
}

/// Gives the body of each request that the mock server received.
async fn bodies(server: &MockServer) -> Vec<serde_json::Value> {
    server
        .received_requests()
        .await
        .unwrap_or_default()
        .iter()
        .map(|request| serde_json::from_slice(&request.body).unwrap_or(serde_json::Value::Null))
        .collect()
}

/// A position that is not the end gives one request, and that request holds no
/// mark.
#[tokio::test]
async fn a_position_gives_one_request_with_no_mark() {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/api/me/progress/item-1"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    update_media_progress_book(&client(&server.uri()), "item-1", Some(30), "60")
        .await
        .unwrap();

    let bodies = bodies(&server).await;

    assert_eq!(bodies.len(), 1);
    assert_eq!(bodies[0]["currentTime"], 30);
    assert_eq!(bodies[0]["progress"], 0.5);
    assert!(bodies[0].get("isFinished").is_none());
}

/// The end of a book gives two requests: the position, and then the mark. The
/// body of the mark holds one key only.
#[tokio::test]
async fn the_end_of_a_book_gives_the_mark_in_its_own_request() {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/api/me/progress/item-1"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    update_media_progress2_book(&client(&server.uri()), "item-1", Some(60), "60", true)
        .await
        .unwrap();

    let bodies = bodies(&server).await;

    assert_eq!(
        bodies.len(),
        2,
        "the position and the mark are two requests"
    );

    // The first request holds the position and no mark.
    assert_eq!(bodies[0]["currentTime"], 60);
    assert!(bodies[0].get("isFinished").is_none());

    // The second request holds the mark and nothing more.
    assert_eq!(bodies[1], serde_json::json!({ "isFinished": true }));
    assert_eq!(bodies[1].as_object().unwrap().len(), 1);
}

/// A book that is not at its end gives one request only.
#[tokio::test]
async fn a_book_that_is_not_finished_gives_one_request() {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/api/me/progress/item-1"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    update_media_progress2_book(&client(&server.uri()), "item-1", Some(10), "60", false)
        .await
        .unwrap();

    assert_eq!(bodies(&server).await.len(), 1);
}

/// The end of an episode gives the same two requests, on the address of the
/// episode.
#[tokio::test]
async fn the_end_of_an_episode_gives_the_mark_in_its_own_request() {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/api/me/progress/pod-1/ep-1"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    update_media_progress2_pod(
        &client(&server.uri()),
        "pod-1",
        Some(60),
        "60",
        true,
        "ep-1",
    )
    .await
    .unwrap();

    let bodies = bodies(&server).await;

    assert_eq!(bodies.len(), 2);
    assert_eq!(bodies[1], serde_json::json!({ "isFinished": true }));
}

/// The server does not answer the first request. The application must not send
/// the mark, and it must give the error.
#[tokio::test]
async fn a_position_that_fails_stops_the_mark() {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/api/me/progress/item-1"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    let result =
        update_media_progress2_book(&client(&server.uri()), "item-1", Some(60), "60", true).await;

    assert!(result.is_err());
}
