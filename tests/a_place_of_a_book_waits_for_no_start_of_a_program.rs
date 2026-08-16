//! A place of a book waits for no start of a program. See T-295.
//!
//! **T-294 gave the reader the table `pending_ebook_progress`**, and the start
//! of the program after a program that died sends every row of it. The task of
//! T-25 sends the positions of a playback every 30 seconds while the program
//! stands, and it named the table of the audio alone: a place of a book that
//! the server refused therefore waited for the **start** of a program.
//!
//! **The reader of a second book takes the rule of the time of the first one
//! away.** `get_the_book` of `src/app.rs` writes `self.reader = None` at once,
//! and `send_the_place_of_the_reader_if_it_is_time` needs a reader: a user who
//! read a book while the server did not answer, who left it with the key `h`,
//! and who then opened a second book holds no road at all to the server while
//! that program stands.
//!
//! The measurement of the real program v0.8.123 inside tmux against the
//! sandbox, with `docs/harness/one_method_fails.py 13500 13399 requests.log
//! PATCH:/api/me/progress` and the one address `http://127.0.0.1:13500` of the
//! account (the trap 129). The book `Alice in Wonderland`,
//! 8fda6e43-0728-46ad-98bc-4c8634e299ad, stood at
//! `ebookLocation toutui:the-place-of-the-start`. The key `/` and the word
//! `Alice`, the key `e`, the key `n` (`chapter 3 of 14 — 2%`), and the key `h`
//! gave `The server did not take the place: The server reported a fault. Status
//! 500.`, and the key `e` of a second book took the reader of Alice away. A
//! second proxy of the same port then gave the server back, and a row of
//! `pending_progress` of the same account stood beside the row of the book:
//!
//! | The row of the disk | The server answers again | 30 s | 60 s | 90 s | 120 s |
//! |---|---|---|---|---|---|
//! | the position of a playback | waits | **the server took it** | — | — | — |
//! | the place of the book | waits | waits | waits | waits | **waits** |
//!
//! The server still held `toutui:the-place-of-the-start` at `ebookProgress 0`,
//! and the user read two chapters.
//!
//! **This test needs no sandbox**: a host of `wiremock` gives the answer of the
//! server, and `received_requests` says which path the program asked for.

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::db::crud::{
    get_pending_ebook_progress, get_pending_progress, insert_pending_ebook_progress,
    insert_pending_progress, PendingEbookProgress, PendingProgress,
};
use toutui::logic::offline::the_places_that_wait_go_to_the_server;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// The book of the measurement.
const THE_BOOK: &str = "8fda6e43-0728-46ad-98bc-4c8634e299ad";

/// The media of the position of a playback that waits beside it.
const THE_MEDIA_OF_THE_PLAYBACK: &str = "d8f33299-a8b6-4f7a-8cd0-34d6d1d98f9a";

/// The place of the user at `chapter 3 of 14 — 2%`.
const THE_PLACE: &str = "epubcfi(/6/6!/4/2/2/1:0)";

/// The part of the book that the user read.
const THE_PART: f64 = 0.023_925_914_837_164_403;

/// The account and the server of the rows of the disk.
const THE_ACCOUNT: &str = "toutuitest";
const THE_SERVER: &str = "http://127.0.0.1:13500";

fn a_client(url: &str) -> ApiClient {
    let pool = EndpointPool::new(vec![Endpoint::new(url, 0)]);
    ApiClient::new(Arc::new(pool), "the-token-of-the-test".to_string()).unwrap()
}

/// The task of the flush sends the place of a book that waits, and the program
/// of the user does not stop for it.
///
/// **The parts of this test stay in one function**: `XDG_CONFIG_HOME` belongs
/// to the process, and two test functions of one binary take a thread each
/// (T-144 and T-157).
#[tokio::test]
async fn a_place_of_a_book_waits_for_no_start_of_a_program() {
    let directory = tempfile::tempdir().expect("a directory");
    unsafe { std::env::set_var("XDG_CONFIG_HOME", directory.path()) };
    std::fs::create_dir_all(directory.path().join("toutui")).expect("the directory of the program");

    // **The disk holds the two places of the user**: the position of a playback
    // that ended while the server did not answer, and the place of the book of
    // the reader that the server refused.
    insert_pending_progress(
        THE_ACCOUNT,
        THE_SERVER,
        &PendingProgress {
            id_item: THE_MEDIA_OF_THE_PLAYBACK.to_string(),
            id_pod: String::new(),
            current_time: 123.0,
            duration: 600.0,
            is_finished: false,
            updated_at: 1_755_000_000_000,
        },
    )
    .expect("the position of the playback");

    insert_pending_ebook_progress(
        THE_ACCOUNT,
        THE_SERVER,
        &PendingEbookProgress {
            id_item: THE_BOOK.to_string(),
            location: THE_PLACE.to_string(),
            fraction: THE_PART,
            updated_at: 1_755_000_000_000,
        },
    )
    .expect("the place of the book");

    // **The server answers again.** A media that never played gives 404 to the
    // read of its position, and the program then sends the position of the
    // disk. See T-188.
    let the_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!(
            "/api/me/progress/{}",
            THE_MEDIA_OF_THE_PLAYBACK
        )))
        .respond_with(ResponseTemplate::new(404))
        .mount(&the_server)
        .await;

    Mock::given(method("PATCH"))
        .and(path(format!(
            "/api/me/progress/{}",
            THE_MEDIA_OF_THE_PLAYBACK
        )))
        .respond_with(ResponseTemplate::new(200))
        .mount(&the_server)
        .await;

    Mock::given(method("PATCH"))
        .and(path(format!("/api/me/progress/{}", THE_BOOK)))
        .respond_with(ResponseTemplate::new(200))
        .mount(&the_server)
        .await;

    // One turn of the task of the flush, and no start of a program at all.
    let (positions, books) = the_places_that_wait_go_to_the_server(
        &a_client(&the_server.uri()),
        THE_ACCOUNT,
        THE_SERVER,
    )
    .await;

    assert_eq!(
        positions, 1,
        "the task must send the position of the playback that waits"
    );

    assert_eq!(
        books, 1,
        "the task must send the place of the book that waits, and not the start of the program \
         after this one"
    );

    // **A place that the server took must leave the disk** (T-211).
    assert!(
        get_pending_ebook_progress(THE_ACCOUNT, THE_SERVER)
            .expect("the places of the books that wait")
            .is_empty(),
        "the disk must keep no place of a book that the server took"
    );

    assert!(
        get_pending_progress(THE_ACCOUNT, THE_SERVER)
            .expect("the positions that wait")
            .is_empty(),
        "the disk must keep no position that the server took"
    );

    let requests = the_server
        .received_requests()
        .await
        .expect("the requests of the host");

    let the_place_of_the_book: Vec<_> = requests
        .iter()
        .filter(|request| {
            request.method == wiremock::http::Method::PATCH
                && request.url.path() == format!("/api/me/progress/{}", THE_BOOK)
        })
        .collect();

    assert_eq!(
        the_place_of_the_book.len(),
        1,
        "the task must ask the server one time for the place of the book"
    );

    let body: serde_json::Value = serde_json::from_slice(&the_place_of_the_book[0].body)
        .expect("the body of the request of the place of the book");

    assert_eq!(
        body["ebookLocation"], THE_PLACE,
        "the request must hold the place of the user"
    );

    assert!(
        (body["ebookProgress"]
            .as_f64()
            .expect("the part of the book")
            - THE_PART)
            .abs()
            < 1e-9,
        "the request must hold the part of the book that the user read"
    );

    // **A disk with no place that waits costs no request at all**: the task
    // runs every 30 seconds while the program stands.
    let the_second_server = MockServer::start().await;

    let (positions, books) = the_places_that_wait_go_to_the_server(
        &a_client(&the_second_server.uri()),
        THE_ACCOUNT,
        THE_SERVER,
    )
    .await;

    assert_eq!((positions, books), (0, 0), "no place of the disk waits now");

    assert!(
        the_second_server
            .received_requests()
            .await
            .expect("the requests of the second host")
            .is_empty(),
        "a turn of the task with no place of the disk must send no request"
    );
}
