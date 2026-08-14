//! A playback that keeps no place does not start. See T-201.
//!
//! **The parts of this test stay in one function.** The test writes
//! `XDG_CONFIG_HOME`, and that variable belongs to the process: two test
//! functions of one binary fight for it. See T-144 and T-157.
//!
//! The row of `listening_session` is the one copy of the place of the user for a
//! program that dies (T-140, T-145, and T-152), and **the row of the player of
//! the screen reads it too**. The old code wrote that row with `let _ =`, and
//! `insert_listening_session` gave `Ok(())` for a connection that it did not get
//! (T-200).
//!
//! The measurement of 2026-08-14 with `docs/harness/hold_the_lock.py` and the key
//! `l` of an episode of the sandbox:
//!
//! ```text
//! [ERROR] - [insert_listening_session] the program did not open its database: database is locked
//! [INFO]  - [play] the item 9fa45bd1-… starts at 0 seconds with 1 tracks
//! ```
//!
//! `select count(*) from listening_session` said **0**, the audio of the null
//! device played, the row of the player of the screen said **`N/A`** with no
//! title and no time, and the row of the message held "Loading the media..."
//! for its six seconds. **Every write of the place of that playback after it
//! changed 0 rows**, therefore a program that dies lost the whole playback.
//!
//! **This test needs no sandbox and no sound device.** A host of a raw socket
//! answers the request of the session with the body of the sandbox, and the
//! database of the test is a file that holds no database: the write of the row
//! then fails with no wait at all, and the busy timeout of five seconds of
//! rusqlite stays out of the gate.
//!
//! **The wait of a playback blocks the thread that calls it** (T-158), therefore
//! the playback of this test takes a thread of its own and the test reads the end
//! of that thread with a limit of time.

use std::sync::Arc;
use std::time::Duration;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::logic::message;
use toutui::logic::playback::{play, PlaybackTarget};
use toutui::player::engine::PlayerHandle;

/// The longest time that the playback of this test may take.
const THE_LIMIT: Duration = Duration::from_secs(20);

/// The media of the measurement of T-182, of the sandbox.
const THE_MEDIA: &str = "6ba57b9a-acb5-44f9-b2b6-39ad9107b420";

/// The identity of the session, of the answer of the sandbox.
const THE_SESSION: &str = "ec843f64-f487-4fb9-9358-2d2d9737e8d0";

/// Starts a host that answers every request with the body of a session.
///
/// A mock server is a crate of its own, and a raw socket needs none. The body
/// holds the fields that `collect_info_item` reads, and it is the answer of
/// `POST /api/items/:id/play` of the sandbox (T-182).
async fn a_host_that_opens_a_session() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());

    tokio::spawn(async move {
        while let Ok((mut socket, _)) = listener.accept().await {
            tokio::spawn(async move {
                use tokio::io::{AsyncReadExt, AsyncWriteExt};

                let mut request = Vec::new();
                let mut byte = [0u8; 1];
                while socket.read(&mut byte).await.unwrap_or(0) == 1 {
                    request.push(byte[0]);
                    if request.ends_with(b"\r\n\r\n") {
                        break;
                    }
                }

                let of_the_request = String::from_utf8_lossy(&request).to_string();

                // The path of the session holds `/play`, and the path of the
                // item does not. `play_media` asks for the two of them: the
                // session first, and the audio files after it.
                let body = if of_the_request.contains("/play") {
                    the_session()
                } else {
                    the_item()
                };

                let head = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\
                     Content-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );

                let _ = socket.write_all(head.as_bytes()).await;
                let _ = socket.write_all(body.as_bytes()).await;
                let _ = socket.flush().await;
            });
        }
    });

    address
}

/// The answer of `POST /api/items/:id/play` of the sandbox, with the fields that
/// `collect_info_item` reads. See T-182.
fn the_session() -> String {
    serde_json::json!({
                    "id": THE_SESSION,
                    "libraryItemId": THE_MEDIA,
                    "mediaType": "book",
                    "mediaMetadata": {"title": "A Book Of Many Hours"},
                    "displayTitle": "A Book Of Many Hours",
                    "displayAuthor": "Many Hours Author",
                    "duration": 28800.0,
                    "playMethod": 0,
                    "currentTime": 0.0,
                    "audioTracks": [{
                        "index": 1,
                        "duration": 28800.0,
                        "contentUrl": "/api/items/6ba57b9a/file/9103848",
                        "mimeType": "audio/mpeg"
                    }]
    })
    .to_string()
}

/// The answer of `GET /api/items/:id`, with the fields of one audio file that
/// `track_from` reads.
fn the_item() -> String {
    serde_json::json!({
        "id": THE_MEDIA,
        "media": {
            "duration": 28800.0,
            "audioFiles": [{
                "index": 1,
                "ino": "9103848",
                "duration": 28800.0,
                "mimeType": "audio/mpeg",
                "metadata": {"filename": "a-book.mp3", "size": 100_000_000u64}
            }],
            "chapters": []
        }
    })
    .to_string()
}

#[tokio::test(flavor = "multi_thread")]
async fn a_playback_whose_session_reached_no_disk_does_not_start() {
    // No line of this test may touch the files of the user.
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", dir.path());
    std::fs::create_dir_all(dir.path().join("toutui")).unwrap();
    std::fs::copy(
        concat!(env!("CARGO_MANIFEST_DIR"), "/config.example.toml"),
        dir.path().join("toutui").join("config.toml"),
    )
    .unwrap();

    let address = a_host_that_opens_a_session().await;

    // **The database of this test holds no database.** Every call of the module
    // of the database then gives a fault with no wait at all (T-200).
    std::fs::write(
        dir.path().join("toutui").join("db.sqlite3"),
        b"this file holds no database at all",
    )
    .unwrap();

    let pool = EndpointPool::new(vec![Endpoint::new(&address, 0)]);
    let api = Arc::new(ApiClient::new(Arc::new(pool), "token".to_string()).unwrap());
    let (player, _of_the_engine) = PlayerHandle::without_engine();

    // The message of the key that came before this one must not answer for this
    // key.
    message::forget();

    let target = PlaybackTarget::Book {
        item_id: THE_MEDIA.to_string(),
        whole_book_duration: Some(28800.0),
    };

    // **The playback takes a thread of its own.** `wait_prev_session_finished`
    // blocks the thread that calls it (T-158).
    let (of_the_end, the_end) = std::sync::mpsc::channel();

    let of_the_playback = std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(async move {
            play(
                &api,
                &player,
                target,
                "toutuitest".to_string(),
                address,
                "a-key".to_string(),
            )
            .await;
        });

        let _ = of_the_end.send(());
    });

    assert!(
        the_end.recv_timeout(THE_LIMIT).is_ok(),
        "the playback did not come back in {:?}",
        THE_LIMIT
    );

    of_the_playback.join().unwrap();

    // **The answer of a key belongs to no view** (T-164), therefore every view of
    // the user reads it. The Home view stands here for all of them.
    let text = message::for_the_screen(toutui::app::AppView::Home).expect(
        "the playback kept no place, therefore the program must say why. It said \
         nothing at all: this is T-201.",
    );

    assert!(
        text.starts_with("The program did not keep the session on its disk:"),
        "the sentence must say that the disk did not take the session, and it \
         says {:?}",
        text
    );

    // The sentence names the key of the work of that fault (T-170 and T-183).
    assert!(
        text.contains("press the key again"),
        "the sentence must name the key of the work, and it says {:?}",
        text
    );

    assert!(
        !text.contains("Loading the media"),
        "the sentence of the answer must take the place of the wait, and it says {:?}",
        text
    );
}
