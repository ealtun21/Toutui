//! The place of a book of a program that dies waits on the disk. See T-294.
//!
//! **T-292 gave the reader a box of the process, and a box of the process goes
//! away with the process.** The two roads of the end of T-292 hold a program
//! that stops of its own will: the key `Q` and the terminal that went away. A
//! program that takes `SIGKILL`, a machine that stops, and a program that the
//! machine killed for the memory hold none of the two, and the place of the
//! user then reached no machine at all.
//!
//! **The audio playback has a table of the disk for this, and the reader had
//! none**: `pending_progress` (T-152 and T-212) holds a position in seconds, a
//! length, and the mark of the end, and the request of a book of the reader
//! holds the text of `ebookLocation` and the number of `ebookProgress` alone.
//! The version 10 of the schema therefore gives the reader
//! `pending_ebook_progress`.
//!
//! The measurement of the real program v0.8.122 inside tmux against the
//! sandbox, with `docs/harness/one_method_fails.py 13500 13399 requests.log
//! PATCH:/api/me/progress` and the one address `http://127.0.0.1:13500` of the
//! account (the trap 129). The book `Alice in Wonderland`,
//! 8fda6e43-0728-46ad-98bc-4c8634e299ad, stood at
//! `ebookLocation toutui:the-place-of-the-start`. The key `/` and the word
//! `Alice`, the key `e`, two presses of the key `n` (`chapter 4 of 14 — 9%`),
//! and the key `h` gave `The server did not take the place: The server reported
//! a fault. Status 500.` A `kill -9` of that program then took the box away:
//!
//! | The program | The disk after the `kill -9` | The server after the program of the next start |
//! |---|---|---|
//! | v0.8.122 | **no table of the reader at all** | **`toutui:the-place-of-the-start`, `ebookProgress 0`** |
//! | The correction | one row of `pending_ebook_progress`, at `epubcfi(/6/8!/4/2/2/1:0)` | `epubcfi(/6/8!/4/2/2/1:0)`, `ebookProgress 0.0916…` |
//!
//! **The user read two chapters, the program died, and the server kept the
//! place of the start on every machine of that account.**
//!
//! **This test needs no sandbox**: a host of `wiremock` gives the answer of the
//! server, and `received_requests` says which path the program asked for.

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::db::crud::get_pending_ebook_progress;
use toutui::logic::reader::the_place_that_waits::{
    say_the_place_that_waits, the_place_of_the_reader_goes_to_the_server,
    the_places_of_the_disk_go_to_the_server, the_places_that_wait, ThePlaceOfTheReader,
};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// The media of the measurement.
const THE_BOOK: &str = "8fda6e43-0728-46ad-98bc-4c8634e299ad";

/// The place of the user at `chapter 4 of 14 — 9%`.
const THE_PLACE: &str = "epubcfi(/6/8!/4/2/2/1:0)";

/// The part of the book that the user read.
const THE_PART: f64 = 0.091_630_833_716_182_39;

/// The account and the server of the row of the disk.
const THE_ACCOUNT: &str = "toutuitest";
const THE_SERVER: &str = "http://127.0.0.1:13500";

fn a_client(url: &str) -> ApiClient {
    let pool = EndpointPool::new(vec![Endpoint::new(url, 0)]);
    ApiClient::new(Arc::new(pool), "the-token-of-the-test".to_string()).unwrap()
}

fn the_place_of_the_user() -> ThePlaceOfTheReader {
    ThePlaceOfTheReader {
        item_id: THE_BOOK.to_string(),
        location: THE_PLACE.to_string(),
        fraction: THE_PART,
    }
}

/// A place of a book that the server refused waits on the disk, and the program
/// after the one that died sends it.
///
/// **The parts of this test stay in one function**: the box of the places and
/// `XDG_CONFIG_HOME` each belong to the process, and two test functions of one
/// binary take a thread each (T-144 and T-157).
#[tokio::test]
async fn a_place_of_a_book_of_a_program_that_dies_waits_on_the_disk() {
    let directory = tempfile::tempdir().expect("a directory");
    unsafe { std::env::set_var("XDG_CONFIG_HOME", directory.path()) };
    std::fs::create_dir_all(directory.path().join("toutui")).expect("the directory of the program");

    // **The road of the fault: the server refuses the place of the book.** The
    // key `h`, the key `s`, and the rule of the time each meet this answer, and
    // the road of the end of T-292 meets it too.
    let the_server_refuses = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path(format!("/api/me/progress/{}", THE_BOOK)))
        .respond_with(ResponseTemplate::new(500))
        .mount(&the_server_refuses)
        .await;

    say_the_place_that_waits(the_place_of_the_user());

    the_place_of_the_reader_goes_to_the_server(
        &a_client(&the_server_refuses.uri()),
        THE_ACCOUNT,
        THE_SERVER,
        "Q",
    )
    .await;

    // **The box of the process holds the place, and the disk holds it too.**
    // The box is the road of a second attempt of this program, and the row of
    // the disk is the one road of the program after a program that died.
    let the_rows = get_pending_ebook_progress(THE_ACCOUNT, THE_SERVER)
        .expect("the places of the books that wait");

    assert_eq!(
        the_rows.len(),
        1,
        "a place that the server refused must wait on the disk, and the disk held {:?}",
        the_rows
    );

    assert_eq!(
        the_rows[0].id_item, THE_BOOK,
        "the row of the disk must name the media of the book"
    );

    assert_eq!(
        the_rows[0].location, THE_PLACE,
        "the row of the disk must hold the place of the user"
    );

    assert!(
        (the_rows[0].fraction - THE_PART).abs() < 1e-9,
        "the row of the disk must hold the part of the book that the user read"
    );

    // **The program died here.** The box of the process goes away with it, and
    // the row of the disk stays.
    for place in the_places_that_wait() {
        toutui::logic::reader::the_place_that_waits::the_place_of_this_book_waits_no_more(
            &place.item_id,
        );
    }

    assert!(
        the_places_that_wait().is_empty(),
        "the box of the process must hold no place of a program that died"
    );

    // **The program after it sends the row of the disk at its start**, before
    // the first frame and before every other request of a list.
    let the_server_takes_it = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path(format!("/api/me/progress/{}", THE_BOOK)))
        .respond_with(ResponseTemplate::new(200))
        .mount(&the_server_takes_it)
        .await;

    let sent = the_places_of_the_disk_go_to_the_server(
        &a_client(&the_server_takes_it.uri()),
        THE_ACCOUNT,
        THE_SERVER,
    )
    .await;

    assert_eq!(
        sent, 1,
        "the start of the program must send the place of the book that waited on the disk"
    );

    let requests = the_server_takes_it
        .received_requests()
        .await
        .expect("the requests of the host");

    assert_eq!(
        requests.len(),
        1,
        "the start must send one request of the one row of the disk"
    );

    assert_eq!(
        requests[0].url.path(),
        format!("/api/me/progress/{}", THE_BOOK),
        "the request must name the media of the book"
    );

    let body: serde_json::Value =
        serde_json::from_slice(&requests[0].body).expect("the body of the request");

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

    // **A place that the server took must leave the disk** (T-211): a row that
    // stays gives the server that same place at every start after this one, and
    // it can then stand behind the place of a second client.
    assert!(
        get_pending_ebook_progress(THE_ACCOUNT, THE_SERVER)
            .expect("the places of the books that wait")
            .is_empty(),
        "a place that the server took must leave the disk"
    );

    // **A place of one server does not go to another server** (the rule of the
    // version 8 of the schema). The row of the account of one server belongs to
    // no other server of that user.
    say_the_place_that_waits(the_place_of_the_user());

    the_place_of_the_reader_goes_to_the_server(
        &a_client(&the_server_refuses.uri()),
        THE_ACCOUNT,
        THE_SERVER,
        "Q",
    )
    .await;

    assert!(
        get_pending_ebook_progress(THE_ACCOUNT, "http://a-different-server-of-this-user")
            .expect("the places of the books that wait")
            .is_empty(),
        "a place of one server must not go to another server of the same user"
    );

    assert!(
        get_pending_ebook_progress("a-different-account", THE_SERVER)
            .expect("the places of the books that wait")
            .is_empty(),
        "a place of one account must not go to another account of the same server"
    );

    // **The start of the program must call that function**, and the two roads
    // of the end and the road of the key must write the row: the functions
    // above hold no fault of their own when no road reads them. Each block ends
    // at a line of its own road, and not at a window of a number of characters
    // (the trap 209).
    let the_keys = std::fs::read_to_string("src/app.rs").expect("the file of the keys");

    let the_start = the_keys
        .split_once("flush_pending_progress(&api, &username, &server_key).await;")
        .expect("the flush of the positions of the start")
        .1
        .split_once("let libraries_names")
        .expect("the lists of the start")
        .0;

    assert!(
        the_start.contains("the_places_of_the_disk_go_to_the_server"),
        "the start of the program must send the places of the books that waited on the disk"
    );

    let the_road_of_the_key = the_keys
        .split_once("pub fn send_the_place_of_the_reader(&mut self)")
        .expect("the road of the send of the place of the reader")
        .1
        .split_once("\n    /// Says which place of the reader")
        .expect("the function after the send")
        .0;

    assert!(
        the_road_of_the_key.contains("the_place_of_this_book_waits_on_the_disk"),
        "a place that the server refused must reach the disk from the road of the key"
    );

    assert!(
        the_road_of_the_key.contains("the_place_of_this_book_waits_no_more_on_the_disk"),
        "a place that the server took must leave the disk from the road of the key"
    );
}
