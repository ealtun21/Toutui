//! The place of a book of a program that stops goes to the server. See T-292
//! and T-293.
//!
//! **T-293 gave the box one place for each book.** The box of T-292 held one
//! place for the whole program, and the loop of the application wrote it at
//! each turn: a user who left a book with the key `h` while the server refused
//! that place, and who then opened a second book, lost the place of the first
//! one before the program stopped. The measurement of the real program v0.8.121
//! inside tmux, with `docs/harness/one_method_fails.py 13500 13399 requests.log
//! PATCH:/api/me/progress`, gave one `PATCH` of the second book at the key `Q`
//! and none of the first one.
//!
//! **The reader holds no table of the disk.** The audio playback keeps the place
//! of the user in the row of `listening_session` (T-201) and in the table
//! `pending_progress` (T-212), therefore a program that stops gives that place
//! to the server, and a program that dies leaves it for the program after it.
//! The reader keeps its place in the `App` alone.
//!
//! **The program stops on two roads that hold no `App`**: the key `Q`, which the
//! footer of the view of the reader names, and the terminal that went away
//! (T-271). The old code of both of them called `sync_session_from_database`
//! alone, and that function reads the rows of the audio: **it asks the reader
//! nothing at all.**
//!
//! The measurement of the real program v0.8.120 inside tmux against the sandbox.
//! The server held `Alice in Wonderland` at
//! `ebookLocation epubcfi(/6/6!/4/2/14/1:698)`, the reader of the key `e` opened
//! at `chapter 3 of 14 — 4%`, and two presses of the key `n` gave
//! `chapter 5 of 14 — 16%`:
//!
//! | The road of the end | v0.8.120 | The correction |
//! |---|---|---|
//! | The key `Q` | **`epubcfi(/6/6!/4/2/14/1:698)`, `ebookProgress 0`** | `epubcfi(/6/10!/4/2/2/1:0)`, `ebookProgress 0.156…` |
//! | The terminal that went away | **`epubcfi(/6/6!/4/2/14/1:698)`, `ebookProgress 0`** | `epubcfi(/6/10!/4/2/2/1:0)`, `ebookProgress 0.156…` |
//! | The log of the old program | no line of the reader at all | `[Q] the place of the book of the media 8fda… went to the server before the program stopped` |
//!
//! The key `h` of the same run sent that place at once
//! (`The server has the place of the book.`), therefore the fault stands in the
//! roads of the end alone. **The user read two chapters and stopped the program
//! with the key that the view names, and the server kept the place of chapter 3
//! on every machine of that account.**
//!
//! **This test needs no sandbox**: a host of `wiremock` gives the answer of the
//! server, and `received_requests` says which path the program asked for.

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::logic::reader::the_place_that_waits::{
    say_the_place_that_waits, the_loop_says_the_place_of_the_reader,
    the_place_of_the_reader_goes_to_the_server, the_place_of_this_book_that_waits,
    the_place_of_this_book_waits_no_more, the_places_that_wait, ThePlaceOfTheReader,
};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// The media of the measurement.
const THE_BOOK: &str = "8fda6e43-0728-46ad-98bc-4c8634e299ad";

/// The place of the user at `chapter 5 of 14`.
const THE_PLACE: &str = "epubcfi(/6/10!/4/2/2/1:0)";

/// The second book of the measurement of T-293, `A Long Test Book`.
const THE_SECOND_BOOK: &str = "9a671047-6146-4003-8510-d215db074a9c";

/// The place of the user in that second book.
const THE_PLACE_OF_THE_SECOND_BOOK: &str = "epubcfi(/6/4!/4/2/2/1:0)";

/// The account and the server of the row of the disk of T-294. **The road of
/// the end writes a row of `pending_ebook_progress` for a place that the server
/// refuses**, therefore this test needs a database of its own.
const THE_ACCOUNT: &str = "the-account-of-the-test";
const THE_SERVER: &str = "http://the-server-of-the-test";

fn a_client(url: &str) -> ApiClient {
    let pool = EndpointPool::new(vec![Endpoint::new(url, 0)]);
    ApiClient::new(Arc::new(pool), "the-token-of-the-test".to_string()).unwrap()
}

fn the_place_of_the_user() -> ThePlaceOfTheReader {
    ThePlaceOfTheReader {
        item_id: THE_BOOK.to_string(),
        location: THE_PLACE.to_string(),
        fraction: 0.156_439_865_168_307_32,
    }
}

fn the_place_of_the_second_book() -> ThePlaceOfTheReader {
    ThePlaceOfTheReader {
        item_id: THE_SECOND_BOOK.to_string(),
        location: THE_PLACE_OF_THE_SECOND_BOOK.to_string(),
        fraction: 0.5,
    }
}

/// The road of the end sends the place of each book, and it keeps a place while
/// no machine holds it.
///
/// **The parts of this test stay in one function**: the box of the places stands
/// in the process, and two test functions of one binary take a thread each
/// (T-144 and T-157).
#[tokio::test]
async fn a_place_of_a_book_of_a_program_that_stops_goes_to_the_server() {
    // A program with no reader, and a reader whose place the server holds
    // already, each leave no place behind them. The road of the end then asks
    // the server nothing at all.
    // **The road of the end writes the disk from T-294**, therefore this test
    // needs a `XDG_CONFIG_HOME` of its own: a test that writes the database of
    // the user is a test that changes the machine of that user.
    let directory = tempfile::tempdir().expect("a directory");
    unsafe { std::env::set_var("XDG_CONFIG_HOME", directory.path()) };
    std::fs::create_dir_all(directory.path().join("toutui")).expect("the directory of the program");

    the_place_of_this_book_waits_no_more(THE_BOOK);
    the_place_of_this_book_waits_no_more(THE_SECOND_BOOK);

    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path(format!("/api/me/progress/{}", THE_BOOK)))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    the_place_of_the_reader_goes_to_the_server(
        &a_client(&server.uri()),
        THE_ACCOUNT,
        THE_SERVER,
        "Q",
    )
    .await;

    assert!(
        server
            .received_requests()
            .await
            .expect("the requests of the host")
            .is_empty(),
        "a program with no place of a reader must ask the server nothing"
    );

    // The loop of the application says which place no machine holds.
    say_the_place_that_waits(the_place_of_the_user());

    assert_eq!(
        the_place_of_this_book_that_waits(THE_BOOK),
        Some(the_place_of_the_user()),
        "the box must hold the place of the user for the road of the end"
    );

    // **The road of the key `Q` and the road of the terminal that went away.**
    // The old program sent nothing here, and the place of two chapters of
    // reading went away with the process.
    the_place_of_the_reader_goes_to_the_server(
        &a_client(&server.uri()),
        THE_ACCOUNT,
        THE_SERVER,
        "Q",
    )
    .await;

    let requests = server
        .received_requests()
        .await
        .expect("the requests of the host");

    assert_eq!(
        requests.len(),
        1,
        "the road of the end must send the place of the reader one time"
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
            - 0.156_439_865_168_307_32)
            .abs()
            < 1e-9,
        "the request must hold the part of the book that the user read"
    );

    // A place that the server took goes to the server no second time: the two
    // roads of the end can meet in one program.
    assert_eq!(
        the_place_of_this_book_that_waits(THE_BOOK),
        None,
        "a place that the server took must leave the box"
    );

    // **A place that the server did not take keeps the box** (T-291 and T-212).
    // The program says the fault in the log, and it holds no other machine for
    // that place.
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

    assert_eq!(
        the_place_of_this_book_that_waits(THE_BOOK),
        Some(the_place_of_the_user()),
        "a place that the server did not take must stay in the box"
    );

    // **The place of a book that the reader left must not go away with a
    // second book** (T-293). The user reads a book, the send of the key `h`
    // fails, and the user then opens a second book: `self.reader` holds the
    // second book alone, and the old box of one place lost the first one.
    say_the_place_that_waits(the_place_of_the_second_book());

    let the_two_books = the_places_that_wait();

    assert_eq!(
        the_two_books.len(),
        2,
        "the box must hold the place of each book that no machine holds"
    );

    assert_eq!(
        the_place_of_this_book_that_waits(THE_BOOK),
        Some(the_place_of_the_user()),
        "the place of a second book must not take the place of the first one away"
    );

    // **The road of the end sends the place of each of them.** A book whose
    // place the server refuses stops no other book.
    let the_server_takes_them = MockServer::start().await;

    for book in [THE_BOOK, THE_SECOND_BOOK] {
        Mock::given(method("PATCH"))
            .and(path(format!("/api/me/progress/{}", book)))
            .respond_with(ResponseTemplate::new(200))
            .mount(&the_server_takes_them)
            .await;
    }

    the_place_of_the_reader_goes_to_the_server(
        &a_client(&the_server_takes_them.uri()),
        THE_ACCOUNT,
        THE_SERVER,
        "Q",
    )
    .await;

    let the_paths: Vec<String> = the_server_takes_them
        .received_requests()
        .await
        .expect("the requests of the host")
        .iter()
        .map(|request| request.url.path().to_string())
        .collect();

    assert!(
        the_paths.contains(&format!("/api/me/progress/{}", THE_BOOK))
            && the_paths.contains(&format!("/api/me/progress/{}", THE_SECOND_BOOK)),
        "the road of the end must send the place of each book, and it sent {:?}",
        the_paths
    );

    assert!(
        the_places_that_wait().is_empty(),
        "every place that the server took must leave the box"
    );

    // **The loop of the application must not empty the box for a reader that
    // went away** (T-293). The key `h` gives the view before the reader back,
    // and the key `e` of a second book writes `self.reader = None` at once:
    // the turn of the loop of that moment holds no place and no book of the
    // server, and the box must then keep every place that it holds.
    say_the_place_that_waits(the_place_of_the_user());
    say_the_place_that_waits(the_place_of_the_second_book());

    the_loop_says_the_place_of_the_reader(None, None);

    assert_eq!(
        the_places_that_wait().len(),
        2,
        "a reader that went away must take no place of the box with it"
    );

    // A reader whose place the server holds already takes its own place out of
    // the box, and it leaves the place of every other book.
    the_loop_says_the_place_of_the_reader(None, Some(THE_SECOND_BOOK));

    assert_eq!(
        the_place_of_this_book_that_waits(THE_SECOND_BOOK),
        None,
        "the place of a book that the server holds must leave the box"
    );

    assert_eq!(
        the_place_of_this_book_that_waits(THE_BOOK),
        Some(the_place_of_the_user()),
        "the place of one book must not take the place of another book away"
    );

    // The place of the reader of this turn goes in the box, and it leaves the
    // place of every other book where it stands.
    the_loop_says_the_place_of_the_reader(Some(the_place_of_the_second_book()), None);

    assert_eq!(
        the_places_that_wait().len(),
        2,
        "the place of the reader of this turn must stand beside the places of the books before it"
    );

    the_place_of_this_book_waits_no_more(THE_BOOK);
    the_place_of_this_book_waits_no_more(THE_SECOND_BOOK);

    // **The two roads of the end must call that function, and the loop of the
    // application must fill the box.** The function above holds no fault of its
    // own when no road reads it: the fault of v0.8.120 was the absence of these
    // three lines. Each block ends at a line of its own road, and not at a
    // window of a number of characters (the trap 209).
    let the_keys = std::fs::read_to_string("src/app.rs").expect("the file of the keys");

    let the_road_of_the_key_q = the_keys
        .split_once("KeyCode::Char('Q') | KeyCode::Esc => {")
        .expect("the arm of the key Q")
        .1
        .split_once("sync_session_from_database(")
        .expect("the sync of the arm of the key Q")
        .0;

    assert!(
        the_road_of_the_key_q.contains("the_place_of_the_reader_goes_to_the_server"),
        "the key `Q` must send the place of the reader before the sync stops the program"
    );

    let the_watch = std::fs::read_to_string("src/utils/the_terminal_that_went_away.rs")
        .expect("the file of the watch of the terminal");

    let the_road_of_the_terminal = the_watch
        .split_once("TheAnswerOfTheWatch::TheTerminalWentAway => {")
        .expect("the arm of the terminal that went away")
        .1
        .split_once("sync_session_from_database(")
        .expect("the sync of the arm of the terminal that went away")
        .0;

    assert!(
        the_road_of_the_terminal.contains("the_place_of_the_reader_goes_to_the_server"),
        "a terminal that went away must send the place of the reader before the sync"
    );

    let the_loop = std::fs::read_to_string("src/main.rs").expect("the file of the loop");

    assert!(
        the_loop.contains("say_the_place_of_the_reader_that_waits()"),
        "the loop of the application must say which place of the reader no machine holds"
    );
}
