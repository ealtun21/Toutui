//! A playback that does not start must release the wait of the next playback.
//!
//! `wait_prev_session_finished` waits while `is_loop_break` is not `1`, and it
//! gives that value `0` before the playback begins. Therefore every playback
//! must give the value `1` again when it ends.
//!
//! `play` came back in five places without that value: a server that gives an
//! error, an item that the server does not give, an item with no audio file,
//! and two conditions of the offline mode. The next playback then waited for
//! ever, and the screen held the message "Syncing your last listening session.
//! Please wait...".
//!
//! The test needs no sound card and no server. `PlayerHandle::without_engine`
//! gives the handle, and `wiremock` gives the server.

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::db::crud::{db_insert_usr, get_is_loop_break};
use toutui::db::database_struct::User;
use toutui::logic::playback::{play, PlaybackTarget};
use toutui::player::engine::PlayerHandle;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const USER: &str = "user-of-the-test";

/// The directory of configuration of this test binary. No line of a test may
/// touch the database of the user.
static HOME: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();

fn temporary_home() {
    HOME.get_or_init(|| {
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", dir.path());
        std::fs::create_dir_all(dir.path().join("toutui")).unwrap();

        let conn = toutui::db::migrate::open_conn().unwrap();
        toutui::db::migrate::run_migrations(&conn).unwrap();
        drop(conn);

        dir
    });
}

/// Writes the user of the test.
///
/// `has_played_before` is `1`, therefore the playback of the test does not wait
/// for a playback before it. No playback runs in this test.
fn write_the_user() {
    let user = User {
        server_address: "http://127.0.0.1:1".to_string(),
        username: USER.to_string(),
        token: "token".to_string(),
        is_default_usr: true,
        name_selected_lib: String::new(),
        id_selected_lib: String::new(),
        is_loop_break: "0".to_string(),
        has_played_before: "1".to_string(),
        speed_rate: 1.0,
        is_show_key_bindings: "1".to_string(),
    };

    db_insert_usr(&vec![user]).unwrap();
}

fn client(url: &str) -> ApiClient {
    let pool = EndpointPool::new(vec![Endpoint::new(url, 0)]);
    ApiClient::new(Arc::new(pool), "test-token".to_string()).unwrap()
}

/// The server gives an error for the request that opens the session. `play`
/// then comes back, and the next playback must not wait.
///
/// A measurement with the old code left the value `0`. The next playback then
/// waited for ever.
#[tokio::test(flavor = "multi_thread")]
async fn a_server_that_gives_an_error_releases_the_wait() {
    temporary_home();
    write_the_user();

    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/items/item-1/play"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    let (player, _receiver) = PlayerHandle::without_engine();

    play(
        &client(&server.uri()),
        &player,
        PlaybackTarget::Book {
            item_id: "item-1".to_string(),
            whole_book_duration: Some(60.0),
        },
        USER.to_string(),
        server.uri(),
        "key-of-the-server".to_string(),
    )
    .await;

    assert_eq!(
        get_is_loop_break(USER),
        "1",
        "a playback that does not start must release the wait of the next \
         playback"
    );
}
