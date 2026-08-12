//! The book that the server sends to an e-reader. See T-119.
//!
//! Every test of this file uses a mock server: the behaviour comes from real
//! HTTP answers, and the sentence for the user comes from the status **and** the
//! body of the answer.
//!
//! **The measurement that made this work** stands in `docs/TAKEOVER-BACKLOG.md`
//! under T-119. Two numbers decide the code:
//!
//! - `GET /api/emails/settings` answers `404` for an account that is not an
//!   administrator, therefore `POST /api/authorize` gives the devices.
//! - The server took **36.2 seconds** for a book of 479.5 megabytes, and
//!   `REQUEST_TIMEOUT` of the client is **15 seconds**.

use std::sync::Arc;
use std::time::Duration;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::api::ereaders::{
    send_the_ebook, the_devices_of_the_account, the_end_of_the_send, TheEnd, THE_TIME_OF_A_SEND,
};
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(url: &str) -> ApiClient {
    ApiClient::new(
        Arc::new(EndpointPool::new(vec![Endpoint::new(url, 0)])),
        "test-token".to_string(),
    )
    .unwrap()
}

/// The answer of `POST /api/authorize`, as the server of the measurement gave
/// it. The server filters `ereaderDevices` for the account of the token.
fn the_payload_of_the_login() -> serde_json::Value {
    serde_json::json!({
        "user": { "id": "u1", "username": "toutuitest", "type": "root" },
        "userDefaultLibraryId": "lib1",
        "serverSettings": {},
        "ereaderDevices": [
            { "name": "Kobo of the measurement", "email": "kobo@example.invalid",
              "availabilityOption": "adminOrUp", "users": [] },
            { "name": "A device of every user", "email": "all@example.invalid",
              "availabilityOption": "guestOrUp", "users": [] }
        ],
        "Source": "docker"
    })
}

#[tokio::test]
async fn the_devices_come_from_the_endpoint_of_the_authorization() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/authorize"))
        .respond_with(ResponseTemplate::new(200).set_body_json(the_payload_of_the_login()))
        .expect(1)
        .mount(&server)
        .await;

    let devices = the_devices_of_the_account(&client(&server.uri()))
        .await
        .expect("the server answered");

    assert_eq!(devices.len(), 2);
    assert_eq!(devices[0].name, "Kobo of the measurement");
    assert_eq!(devices[0].email, "kobo@example.invalid");
}

/// **The program never asks the settings of the e-mail.** That endpoint answers
/// `404` for an account that is not an administrator, therefore a program that
/// reads it gives no device to a user who can send a book.
#[tokio::test]
async fn the_program_asks_no_endpoint_of_the_email() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/authorize"))
        .respond_with(ResponseTemplate::new(200).set_body_json(the_payload_of_the_login()))
        .mount(&server)
        .await;

    // Every request of `/api/emails/settings` answers 404 here, as the server
    // does for an account that is not an administrator.
    Mock::given(method("GET"))
        .and(path("/api/emails/settings"))
        .respond_with(ResponseTemplate::new(404))
        .expect(0)
        .mount(&server)
        .await;

    let devices = the_devices_of_the_account(&client(&server.uri()))
        .await
        .expect("the server answered");

    assert_eq!(devices.len(), 2);
}

/// A server that holds no device answers with an empty list, and that is an
/// answer and not a fault: the view says the reason. See T-91.
#[tokio::test]
async fn a_server_with_no_device_gives_an_empty_list() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/authorize"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "user": { "username": "toutuitest" },
            "ereaderDevices": []
        })))
        .mount(&server)
        .await;

    let devices = the_devices_of_the_account(&client(&server.uri()))
        .await
        .expect("the server answered");

    assert!(devices.is_empty());
}

#[tokio::test]
async fn the_send_names_the_item_and_the_device() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/emails/send-ebook-to-device"))
        .and(body_json(serde_json::json!({
            "libraryItemId": "item-1",
            "deviceName": "Kobo of the measurement",
        })))
        .respond_with(ResponseTemplate::new(200).set_body_string("OK"))
        .expect(1)
        .mount(&server)
        .await;

    let end = send_the_ebook(&client(&server.uri()), "item-1", "Kobo of the measurement")
        .await
        .expect("the server answered");

    assert_eq!(end, TheEnd::TheServerSentIt);
}

/// **The three answers of `404` say three different things**, and the body is
/// the one place that tells them apart. A program that reads the status alone
/// says "the server does not have this item" for an audiobook that holds no
/// ebook.
#[tokio::test]
async fn the_body_of_the_fault_makes_the_sentence() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/emails/send-ebook-to-device"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Ebook file not found"))
        .mount(&server)
        .await;

    let end = send_the_ebook(&client(&server.uri()), "item-1", "Kobo")
        .await
        .expect("the server answered");

    let TheEnd::TheServerRefused(words) = end else {
        panic!("the server refused the request");
    };

    assert!(
        words.contains("holds no ebook"),
        "the sentence names the condition of this body: {}",
        words
    );
}

/// A status of `400` is a fault of the request, and not of the address. One
/// answer of 400 must not take the one address of the pool away. See T-87.
#[tokio::test]
async fn a_refusal_of_the_server_keeps_the_address() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/emails/send-ebook-to-device"))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_string("Failed to verify SMTP connection configuration"),
        )
        .mount(&server)
        .await;

    let client = client(&server.uri());

    let end = send_the_ebook(&client, "item-1", "Kobo")
        .await
        .expect("the server answered");

    assert!(matches!(end, TheEnd::TheServerRefused(_)));
    assert!(
        client.pool().active().is_some(),
        "the address of the server stays: a status is a fault of the request"
    );
}

/// **The time limit of the send is not the time limit of a request.**
///
/// The server needs 36 seconds for a book of 480 megabytes, and
/// `REQUEST_TIMEOUT` is 15 seconds. The old shape of this work
/// (`post_no_content`) therefore gave the user a fault while the server sent
/// the book, and a second such request marked the address down (T-97).
///
/// The test holds the rule with no wait: `THE_TIME_OF_A_SEND` must carry the
/// biggest book that this program takes, at a rate much slower than the
/// measurement.
#[test]
fn the_time_of_a_send_carries_the_biggest_book() {
    assert!(
        THE_TIME_OF_A_SEND > Duration::from_secs(15),
        "the time limit of a request is 15 seconds, and the server needs more"
    );

    // `MAX_BOOK_BYTES` of this program is 502 megabytes. The measurement gave
    // about 13 megabytes each second, and this rule holds for a server sixteen
    // times slower than that.
    let the_biggest_book_in_megabytes = 502.0;
    let the_slow_rate_in_megabytes = 0.8;

    let need = Duration::from_secs_f64(the_biggest_book_in_megabytes / the_slow_rate_in_megabytes);

    assert!(
        THE_TIME_OF_A_SEND >= need,
        "the time limit must carry {} MB at {} MB/s: {:?} against {:?}",
        the_biggest_book_in_megabytes,
        the_slow_rate_in_megabytes,
        THE_TIME_OF_A_SEND,
        need
    );
}

/// The measurement of the rule above, with a real answer that comes late.
///
/// **This test waits 16 seconds**, therefore it carries `#[ignore]`: the fast
/// suite stays at about two seconds. Run it with
/// `cargo nextest run --run-ignored all`.
///
/// A build with `post_no_content` in place of `post_and_read_the_answer` fails
/// this test with `ApiError::Timeout`.
#[tokio::test]
#[ignore = "it waits 16 seconds for the time limit of the client"]
async fn a_server_that_answers_after_the_limit_of_a_request_still_sends_the_book() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/emails/send-ebook-to-device"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string("OK")
                .set_delay(Duration::from_secs(16)),
        )
        .mount(&server)
        .await;

    let client = client(&server.uri());

    let end = send_the_ebook(&client, "item-1", "Kobo")
        .await
        .expect("the send holds a time limit of its own");

    assert_eq!(end, TheEnd::TheServerSentIt);
    assert!(
        client.pool().active().is_some(),
        "a slow answer must not take the address away"
    );
}

/// The old code of this program held one sentence for every status, and the
/// pure function holds each of them apart now.
#[test]
fn every_condition_of_the_server_holds_its_own_sentence() {
    let all = [
        the_end_of_the_send(200, "OK"),
        the_end_of_the_send(400, "Failed to verify SMTP connection configuration"),
        the_end_of_the_send(403, "Forbidden"),
        the_end_of_the_send(404, "Ereader device not found"),
        the_end_of_the_send(404, "Library item not found"),
        the_end_of_the_send(404, "Ebook file not found"),
    ];

    let words: Vec<String> = all
        .iter()
        .map(|end| match end {
            TheEnd::TheServerSentIt => "the book went".to_string(),
            TheEnd::TheServerRefused(reason) => reason.clone(),
        })
        .collect();

    let mut every = words.clone();
    every.sort();
    every.dedup();

    assert_eq!(
        every.len(),
        words.len(),
        "every condition of the server says its own sentence: {:?}",
        words
    );
}
