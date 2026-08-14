//! A media of the queue that did not play stays in the queue. See T-146.
//!
//! **The queue took the media out before the playback started.** A playback that
//! then did not start took that media of the user away for ever: the queue held
//! the media after it, and it played nothing at all.
//!
//! A measurement of 2026-08-13 with the sandbox and tmux: a queue of two books
//! ("One Chapter Book" and "Alice in Wonderland"), a book of 30 minutes that
//! played, and `podman stop -t 0 abs-test` in the middle of it. The book came to
//! its end, the log said "the queue starts \"One Chapter Book\"" and then "the
//! disk has no copy", and **the view of the queue held one media**: the first
//! media of the queue was gone, and no key gave it back.
//!
//! This test needs no sound device and no server. Nothing listens on the port of
//! `NO_SERVER`, therefore the playback goes to the offline mode (T-25), the disk
//! holds no copy of the media, and the outcome is `Fault`.

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::db::database_struct::User;
use toutui::logic::playback::play_the_media_of_the_queue;
use toutui::logic::playback::PlaybackTarget;
use toutui::logic::queue::{self, Entry};
use toutui::player::engine::PlayerHandle;

/// Nothing listens on this port, therefore every request of the playback gives
/// the offline mode. See T-25.
const NO_SERVER: &str = "http://127.0.0.1:1";

fn a_user() -> User {
    User {
        server_address: NO_SERVER.to_string(),
        username: "toutuitest".to_string(),
        token: "not-a-real-token".to_string(),
        is_default_usr: true,
        name_selected_lib: "Books".to_string(),
        id_selected_lib: "a-library".to_string(),
        is_loop_break: "1".to_string(),
        // The playback of this test must not wait for a session of a playback
        // before it.
        has_played_before: "1".to_string(),
        speed_rate: 1.0,
        is_show_key_bindings: "1".to_string(),
    }
}

fn a_book(id: &str, title: &str) -> Entry {
    Entry {
        target: PlaybackTarget::Book {
            item_id: id.to_string(),
            whole_book_duration: Some(60.0),
        },
        title: title.to_string(),
        author: "An Author".to_string(),
        duration: Some(60.0),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_playback_that_did_not_start_gives_its_media_back_to_the_queue() {
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

    toutui::db::crud::db_insert_usr(&vec![a_user()]).unwrap();

    // The queue of this account. The write of the queue then goes to the
    // database of this test.
    queue::read_the_queue_of_the_account("toutuitest", NO_SERVER);
    queue::clear();

    // The user asked for two media. The key `l` of the view of the queue takes
    // the first one out, and it plays it.
    let first = a_book("the-first-item", "The First Book");
    queue::add(a_book("the-second-item", "The Second Book")).unwrap();

    let pool = EndpointPool::new(vec![Endpoint::new(NO_SERVER, 0)]);
    let api = Arc::new(ApiClient::new(Arc::new(pool), "token".to_string()).unwrap());
    let (player, _of_the_engine) = PlayerHandle::without_engine();

    play_the_media_of_the_queue(
        &api,
        &player,
        first.clone(),
        "toutuitest".to_string(),
        NO_SERVER.to_string(),
        "a-key".to_string(),
    )
    .await;

    let queue_now = queue::snapshot();

    assert_eq!(
        queue_now.len(),
        2,
        "the playback did not start, therefore the queue must hold both media. \
         It holds {:?}",
        queue_now.lines()
    );

    assert_eq!(
        queue_now.entries()[0].title,
        "The First Book",
        "the media that did not play must stand at the front of the queue, and \
         the queue holds {:?}",
        queue_now.lines()
    );

    assert_eq!(
        queue_now.entries()[1].title,
        "The Second Book",
        "the media after it must not move, and the queue holds {:?}",
        queue_now.lines()
    );

    // The disk holds the queue too. A user who starts the program again must
    // find the media that did not play.
    let rows = toutui::db::crud::read_the_queue("toutuitest", NO_SERVER).unwrap();

    assert_eq!(
        rows.len(),
        2,
        "the queue of the disk must hold both media, and it holds {} row(s)",
        rows.len()
    );
    assert_eq!(rows[0].title, "The First Book");

    queue::clear();
    queue::forget_the_account();
}
