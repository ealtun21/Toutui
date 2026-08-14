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

/// The time to wait for one message. The server sends it inside a second, and
/// this value gives room for a machine that is busy.
const WAIT: Duration = Duration::from_secs(20);

mod common;
use common::token;

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
        .get_json(&format!("/api/libraries/{}/items?limit=500", library))
        .await
        .expect("the server must give the items");

    // **A media of a duration that holds the position of this test.** The test
    // sends the second 756, and the server marks a media of one second as
    // finished for that value: the box then says "Finished" and 100 percent, and
    // this test waited 20 seconds for "42" that never comes. The first form took
    // the **first** item of the library, and the sandbox holds four books of one
    // second before that one now (the books of an EPUB of T-127). See T-132 and
    // T-111.
    let item = items["results"]
        .as_array()
        .expect("the answer must hold a list")
        .iter()
        .find(|item| item["media"]["duration"].as_f64().unwrap_or(0.0) > 1000.0)
        .expect(
            "the sandbox must hold a book of more than 1000 seconds. \
             `docs/TEST-SERVER.md` holds the commands of \"A Long Test Book\".",
        )["id"]
        .as_str()
        .expect("an item must hold an identity")
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
    // **The position of the server must differ from the position that this test
    // measures**, and **the server decides that value**. The message `init` of
    // the connection carries the position of every media of the account: a row
    // that holds the value of the measurement already lets this test pass with
    // no live message at all. And `PATCH /api/me/progress/:id` does not keep
    // every value of the body: a measurement of 2026-08-12 sent
    // `progress: 0.10` with `currentTime: 180` and the server wrote
    // `progress: 0.1722`, the fraction of the position that stood there before.
    //
    // The test therefore writes a first position, it asks the server what that
    // row holds now, and it then writes the second position and waits for the
    // value **of the server**. See T-132.
    let percent_of_the_server = |row: &serde_json::Value| -> String {
        format!(
            "{}",
            (row["progress"].as_f64().unwrap_or(0.0) * 100.0).round() as i64
        )
    };

    // **The mark of the media goes away in its own request.** The server takes
    // the fields of one body in the sequence of the text, and a body that holds
    // `isFinished` beside `progress` gave the fraction of the position that stood
    // there before: a measurement of 2026-08-12 sent `progress: 0.10` with
    // `isFinished: false` and read `0.1722` back. A body of `progress` alone
    // keeps the value that it gives. See T-132.
    api.patch_json(
        &format!("/api/me/progress/{}", item),
        &serde_json::json!({ "isFinished": false }),
    )
    .await
    .expect("the server must take the mark");

    api.patch_json(
        &format!("/api/me/progress/{}", item),
        &serde_json::json!({ "progress": 0.10, "currentTime": 180.0 }),
    )
    .await
    .expect("the server must take the position of the start");

    let of_the_start: serde_json::Value = api
        .get_json(&format!("/api/me/progress/{}", item))
        .await
        .expect("the server must give the position of the start");

    // The box must hold the first position before the second one goes, therefore
    // the value of the measurement comes from a live message of that second
    // position and from no other moment.
    let of_the_start = percent_of_the_server(&of_the_start);

    wait_for("the position of the start", || {
        toutui::logic::live::progress_of(&item).is_some_and(|row| row.percent == of_the_start)
    })
    .await;

    api.patch_json(
        &format!("/api/me/progress/{}", item),
        &serde_json::json!({ "progress": 0.42, "currentTime": 756.0 }),
    )
    .await
    .expect("the server must take the position");

    let of_the_measurement: serde_json::Value = api
        .get_json(&format!("/api/me/progress/{}", item))
        .await
        .expect("the server must give the position");

    let wanted = percent_of_the_server(&of_the_measurement);

    assert_ne!(
        wanted, of_the_start,
        "the two positions of this test must differ, therefore the value of the \
         box comes from a live message and not from the message `init`"
    );

    wait_for("the position of the different client", || {
        toutui::logic::live::progress_of(&item)
            .is_some_and(|row| row.percent == wanted && row.finished == "Not finished")
    })
    .await;

    // The mark of the line of the Home view shows that value at the next frame,
    // and the program asks the server for nothing. See T-44.
    let live = toutui::logic::live::progress_of(&item).expect("the box must hold the position");
    assert_eq!(
        toutui::ui::marks::of_progress(&live.percent, &live.finished, false).trim_end(),
        format!("{}%", wanted)
    );

    // **A media whose position the server no longer holds must lose that
    // position.** See T-184.
    //
    // A different client of the same account can take the position of a media
    // away: `DELETE /api/me/progress/:id` of the web reader does that work, and
    // the row of the account then holds no position of that media. The message
    // `user_updated` of that change carries the whole account with no row of that
    // media, therefore this box must forget it.
    //
    // **The old shape of `note_the_progress` inserted alone**: the box kept the
    // old percent, the value of the box wins over the value of the request, and
    // the key `R` could therefore not correct it. The measurement of the sandbox:
    // the row of `A Book Of Many Hours` went away, the server answered the request
    // of the Home view with no position of that media, and the line of the screen
    // said `48%` after the message and after the key `R`.
    //
    // **The identity of that request is the identity of the row, and not the
    // identity of the item**: a `DELETE /api/me/progress/<the item>` answers 404.
    let row_of_the_media: serde_json::Value = api
        .get_json(&format!("/api/me/progress/{}", item))
        .await
        .expect("the server must give the row of the position");

    let id_of_the_row = row_of_the_media["id"]
        .as_str()
        .expect("a row of a position must hold an identity")
        .to_string();

    api.delete_no_content(&format!("/api/me/progress/{}", id_of_the_row))
        .await
        .expect("the server must take the position away");

    wait_for("the media that holds no position", || {
        toutui::logic::live::progress_of(&item).is_none()
    })
    .await;

    // The position of the measurement comes back, therefore the sandbox holds the
    // shape that this test finds at its next run.
    api.patch_json(
        &format!("/api/me/progress/{}", item),
        &serde_json::json!({ "progress": 0.42, "currentTime": 756.0 }),
    )
    .await
    .expect("the server must take the position again");

    wait_for("the position of the media again", || {
        toutui::logic::live::progress_of(&item).is_some()
    })
    .await;

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
