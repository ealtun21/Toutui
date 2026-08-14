//! A playback that did not start says why. See T-167.
//!
//! **The user pressed `l`, they read "Loading the media...", and then they read
//! nothing at all.** The three faults of `logic::playback::play_media` wrote to
//! the log alone: the six seconds of the first message went by, the row of the
//! message became empty, and no media played.
//!
//! A measurement of 2026-08-14 with the sandbox and tmux: the user stood in the
//! view of the episodes of "Letters of Two Brides" with the cursor on
//! "Letter 5", a second program took that episode out of the podcast
//! (`DELETE /api/podcasts/:id/episode/:episode?hard=1`), and the key `l` gave
//! "Loading the media..." and then an empty row. The log held
//! `[play] the server did not start the session: The server does not have this
//! item.` The same key of the Home view gave the same answer.
//!
//! **This test needs no sandbox and no sound device.** A host of a raw socket
//! answers `404` to every request: the fault is therefore not the offline mode
//! of T-25 — a server that does not answer keeps the copy of the disk — but the
//! fault of a server that answers and that does not hold the media.
//!
//! **The wait of a playback blocks the thread that calls it** (T-158), therefore
//! the playback of this test takes a thread of its own and the test reads the
//! end of that thread with a limit of time.

use std::sync::Arc;
use std::time::Duration;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::db::database_struct::User;
use toutui::logic::message;
use toutui::logic::playback::{play, PlaybackTarget};
use toutui::player::engine::PlayerHandle;

/// The longest time that the playback of this test may take.
const THE_LIMIT: Duration = Duration::from_secs(20);

fn a_user(address: &str) -> User {
    User {
        server_address: address.to_string(),
        username: "toutuitest".to_string(),
        token: "not-a-real-token".to_string(),
        is_default_usr: true,
        name_selected_lib: "Podcasts".to_string(),
        id_selected_lib: "a-library".to_string(),
        is_loop_break: "1".to_string(),
        // The playback of this test must not wait for a session of a playback
        // before it.
        has_played_before: "1".to_string(),
        speed_rate: 1.0,
        is_show_key_bindings: "1".to_string(),
    }
}

/// Starts a host that answers `404 Not Found` to every request.
///
/// A mock server is a crate of its own, and a raw socket needs none: the answer
/// holds a length, therefore the client of the program reads it and stops.
async fn a_host_that_holds_nothing() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());

    tokio::spawn(async move {
        while let Ok((mut socket, _)) = listener.accept().await {
            tokio::spawn(async move {
                use tokio::io::{AsyncReadExt, AsyncWriteExt};

                // Read the request, and stop at the empty line. A request with
                // a body gives more bytes, and the answer needs none of them.
                let mut request = Vec::new();
                let mut byte = [0u8; 1];
                while socket.read(&mut byte).await.unwrap_or(0) == 1 {
                    request.push(byte[0]);
                    if request.ends_with(b"\r\n\r\n") {
                        break;
                    }
                }

                let body = b"Item not found";
                let head = format!(
                    "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\n\
                     Content-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );

                let _ = socket.write_all(head.as_bytes()).await;
                let _ = socket.write_all(body).await;
                let _ = socket.flush().await;
            });
        }
    });

    address
}

/// The key `l` on an episode that the server does not hold says one sentence.
///
/// **The parts of this test stay in one function.** Two test functions of one
/// binary share the database and `XDG_CONFIG_HOME` of the box of the process,
/// and `cargo test` gives each of them a thread: that shape is T-144 and T-157.
#[tokio::test(flavor = "multi_thread")]
async fn a_playback_that_did_not_start_says_one_sentence() {
    // No line of this test may touch the files of the user.
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", dir.path());
    std::fs::create_dir_all(dir.path().join("toutui")).unwrap();
    std::fs::copy(
        concat!(env!("CARGO_MANIFEST_DIR"), "/config.example.toml"),
        dir.path().join("toutui").join("config.toml"),
    )
    .unwrap();

    let conn = toutui::db::migrate::open_conn().unwrap();
    toutui::db::migrate::run_migrations(&conn).unwrap();
    drop(conn);

    let address = a_host_that_holds_nothing().await;

    toutui::db::crud::db_insert_usr(&vec![a_user(&address)]).unwrap();

    let pool = EndpointPool::new(vec![Endpoint::new(&address, 0)]);
    let api = Arc::new(ApiClient::new(Arc::new(pool), "token".to_string()).unwrap());
    let (player, _of_the_engine) = PlayerHandle::without_engine();

    // The message of the key that came before this one must not answer for this
    // key.
    message::forget();

    // The episode of the measurement: the podcast stays, and a second program
    // took the episode out of it.
    let target = PlaybackTarget::Episode {
        item_id: "9fa45bd1-66bc-4c17-ba49-a5a6a5ec8806".to_string(),
        episode_id: "58db88de-0857-4454-a9eb-e6579a1a51ec".to_string(),
    };

    // **The playback takes a thread of its own.** `wait_prev_session_finished`
    // blocks the thread that calls it, therefore a limit of time on the future
    // alone says nothing (T-158).
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

    // **The answer of a key belongs to no view** (T-164), therefore every view
    // of the user reads it. The Home view stands here for all of them.
    let text = message::for_the_screen(toutui::app::AppView::Home);

    let text = text.expect(
        "the playback did not start, therefore the program must say why. \
         It said nothing at all: this is T-167.",
    );

    assert!(
        text.starts_with("The server did not start the playback:"),
        "the sentence must say that the server did not start the playback, and \
         it says {:?}",
        text
    );

    assert!(
        text.len() > "The server did not start the playback:".len() + 1,
        "the sentence must name what the server said, and it says {:?}",
        text
    );

    // The message of the wait must not stand here: the user reads the answer of
    // their key, and not "Loading the media...".
    assert!(
        !text.contains("Loading the media"),
        "the sentence of the answer must take the place of the wait, and it says {:?}",
        text
    );
}
