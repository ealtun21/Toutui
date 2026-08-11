//! The live messages of the server come with no new dependency. See T-47.
//!
//! Audiobookshelf sends every change of every client over socket.io. The two
//! crates of socket.io both bring `native-tls`, therefore the rule of T-20
//! refuses both. The transport `polling` of that protocol is plain HTTP, and
//! `reqwest` does it already.
//!
//! This test proves the whole way: the task connects, a **different client**
//! changes the data, and the box that the screen reads holds the change.
//!
//! Continuous integration does not run this test, because it needs a server.
//! Start the sandbox of `docs/TEST-SERVER.md`, and then run:
//!
//! ```text
//! ALSA_CONFIG_PATH=/dev/null cargo test --test live_against_the_sandbox \
//!     -- --ignored --nocapture --test-threads=1
//! ```
//!
//! **The test writes.** It sends a position with `PATCH /api/me/progress/:id`,
//! and it changes the subtitle of one item with `PATCH /api/items/:id/media`.
//! A live message comes for a change only, therefore the test must make one. It
//! touches no other server.

use std::sync::Arc;
use std::time::{Duration, Instant};
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;

const SERVER: &str = "http://127.0.0.1:13399";
const USER: &str = "toutuitest";
const PASSWORD: &str = "toutuitest";

/// The time to wait for one message. The server sends it inside a second, and
/// this value gives room for a machine that is busy.
const WAIT: Duration = Duration::from_secs(20);

async fn token() -> String {
    let answer: serde_json::Value = reqwest::Client::new()
        .post(format!("{}/login", SERVER))
        .json(&serde_json::json!({ "username": USER, "password": PASSWORD }))
        .send()
        .await
        .expect("the sandbox server must answer")
        .json()
        .await
        .expect("the answer of the login must hold JSON");

    answer["user"]["token"]
        .as_str()
        .expect("the answer must hold a token")
        .to_string()
}

/// Waits while a value of the box is not the value that the test needs.
async fn wait_for(what: &str, mut ready: impl FnMut() -> bool) {
    let start = Instant::now();

    while start.elapsed() < WAIT {
        if ready() {
            println!("{} came after {:?}", what, start.elapsed());
            return;
        }

        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    panic!("{} did not come in {:?}", what, WAIT);
}

/// The parts of this test stay in one function. The box of the live messages
/// belongs to the process, and one task holds the connection.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the sandbox server of docs/TEST-SERVER.md on port 13399, and it writes"]
async fn a_change_of_a_different_client_comes_to_the_screen() {
    let token = token().await;
    let pool = Arc::new(EndpointPool::new(vec![Endpoint::new(SERVER, 0)]));
    let api = Arc::new(ApiClient::new(Arc::clone(&pool), token.clone()).unwrap());

    // One book of the library of the books.
    let libraries: serde_json::Value = api
        .get_json("/api/libraries")
        .await
        .expect("the server must give the libraries");

    let library = libraries["libraries"]
        .as_array()
        .expect("the answer must hold a list")
        .iter()
        .find(|library| library["mediaType"] == "book")
        .expect("the sandbox must hold a library of books")["id"]
        .as_str()
        .expect("a library must hold an identity")
        .to_string();

    let items: serde_json::Value = api
        .get_json(&format!("/api/libraries/{}/items?limit=1", library))
        .await
        .expect("the server must give the items");

    let item = items["results"][0]["id"]
        .as_str()
        .expect("the library must hold one item")
        .to_string();

    println!("the item of the test: {}", item);

    toutui::logic::live::forget();
    toutui::api::live::spawn_the_live_task(Arc::clone(&pool), token.clone());

    // The connection needs the handshake, the packet `40`, and the token.
    wait_for("the connection", || {
        toutui::logic::live::state() == toutui::logic::live::State::Ready
    })
    .await;

    // A different client of the same account moves in the book. The program
    // sends no such request here: the client of this test is that different
    // client.
    //
    // `isFinished` must not stand before `progress` in this body. The server
    // reads the fields in the sequence of the text, and a `progress` after an
    // `isFinished` writes over it. See the traps of `docs/HANDOVER.md`.
    api.patch_json(
        &format!("/api/me/progress/{}", item),
        &serde_json::json!({ "progress": 0.42, "currentTime": 756.0 }),
    )
    .await
    .expect("the server must take the position");

    wait_for("the position of the different client", || {
        toutui::logic::live::progress_of(&item)
            .is_some_and(|row| row.percent == "42" && row.finished == "Not finished")
    })
    .await;

    // The mark of the line of the Home view shows that value at the next frame,
    // and the program asks the server for nothing. See T-44.
    let live = toutui::logic::live::progress_of(&item).expect("the box must hold the position");
    assert_eq!(
        toutui::ui::marks::of_progress(&live.percent, &live.finished, false).trim_end(),
        "42%"
    );

    // A different client changes the metadata of the item. The title and the
    // cover of that item stand in many lists, therefore the header asks the
    // user for the key `R`.
    assert!(
        !toutui::logic::live::the_lists_are_old(),
        "the position of a media must not make the lists old, because the \
         program itself sends a position every ten seconds"
    );

    // **The value must differ from the value of the server.** A `PATCH` of the
    // same value changes nothing, therefore the server sends no message and this
    // test waited 20 seconds for a message that never comes. The first form of
    // this test always wrote the same subtitle, and it failed at its second run.
    let item_now: serde_json::Value = api
        .get_json(&format!("/api/items/{}", item))
        .await
        .expect("the server must give the item");

    let subtitle_of_the_server = item_now["media"]["metadata"]["subtitle"]
        .as_str()
        .unwrap_or("");

    let new_subtitle = if subtitle_of_the_server == "A live message of T-47" {
        "A live message of T-47, the second form"
    } else {
        "A live message of T-47"
    };

    println!(
        "the subtitle goes from {:?} to {:?}",
        subtitle_of_the_server, new_subtitle
    );

    api.patch_json(
        &format!("/api/items/{}/media", item),
        &serde_json::json!({ "metadata": { "subtitle": new_subtitle } }),
    )
    .await
    .expect("the server must take the metadata");

    wait_for(
        "the change of the lists",
        toutui::logic::live::the_lists_are_old,
    )
    .await;

    // The key `R` asks the server for every list again.
    toutui::logic::live::the_lists_are_new_again();
    assert!(!toutui::logic::live::the_lists_are_old());
}
