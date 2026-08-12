//! A refresh of the screen must keep the engine that plays. See T-131.
//!
//! **The key `R` makes a new application**, and so does every key that
//! refreshes: the key that takes the next library of the server (T-66) and the
//! keys of the sequence of the library. `App::new` started a new engine of the
//! sound for each of them, and the old engine kept the playback:
//!
//! - The row of the player went away, because the render reads the state of the
//!   engine of the application (`app.player.state()`).
//! - **Every key of the playback went to the new engine**, therefore `Space`
//!   stopped nothing and `Y` closed nothing.
//!
//! A measurement of 2026-08-12 with the sandbox and tmux pressed `R` at the
//! minute 2 of a book of 30 minutes: the row of the player went away, the log
//! held five lines "the application uses the sound device", and the book played
//! to its end. With the correction the log holds one such line, the row stays,
//! and the key `Space` stopped the playback at the second 310.
//!
//! This test needs no sound device and no server. `PlayerHandle::without_engine`
//! gives a handle whose commands go to a channel of the test, therefore the test
//! reads the commands that the application sends.

use std::sync::Arc;
use std::time::Duration;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::app::App;
use toutui::db::database_struct::User;
use toutui::player::engine::{PlayerCommand, PlayerHandle};

/// Nothing listens on this port, therefore `App::new` gives the offline mode
/// and the test needs no server. See T-25.
const NO_SERVER: &str = "http://127.0.0.1:1";

fn a_user() -> User {
    User {
        server_address: NO_SERVER.to_string(),
        username: "toutuitest".to_string(),
        token: "not-a-real-token".to_string(),
        is_default_usr: true,
        name_selected_lib: "Books".to_string(),
        id_selected_lib: "a-library".to_string(),
        is_loop_break: "0".to_string(),
        has_played_before: "1".to_string(),
        speed_rate: 1.0,
        is_show_key_bindings: "1".to_string(),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_refresh_keeps_the_engine_of_the_playback() {
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

    let pool = EndpointPool::new(vec![Endpoint::new(NO_SERVER, 0)]);
    let api = Arc::new(ApiClient::new(Arc::new(pool), "token".to_string()).unwrap());

    // The engine of a program that plays already. The test holds the other end
    // of its channel, therefore it reads every command of the application.
    let (player, of_the_engine) = PlayerHandle::without_engine();

    let app = App::new_with_the_engine(Arc::clone(&api), Some((player, None)))
        .await
        .expect("an application");

    // The application must send the command of a key of the user to **this**
    // engine. A new engine takes the command, and the playback of the user then
    // plays on with no key that reaches it.
    app.player.send(PlayerCommand::Stop);

    let command = of_the_engine
        .recv_timeout(Duration::from_secs(5))
        .expect("the application must send the command to the engine that plays");

    assert!(
        matches!(command, PlayerCommand::Stop),
        "the engine of the playback took {:?}, and the key of the user says Stop",
        command
    );
}

/// **The refresh of the program must take that function.** The rule lives in
/// `src/main.rs`, and no unit test reaches that loop: a test may read the source
/// of the program, as `every_key_of_the_handler_stands_in_the_list` does. See
/// T-131.
#[test]
fn the_refresh_of_the_program_keeps_the_engine() {
    let source = include_str!("../src/main.rs");

    assert!(
        source.contains("App::new_with_the_engine("),
        "the loop of the program must make the new application with the engine \
         that plays. `App::new` starts a new engine, and the playback of the \
         user then belongs to no key."
    );
}
