//! A refresh of the screen must keep the timer for sleep. See T-135.
//!
//! **The key `R` makes a new application**, and so does every key that
//! refreshes: the key that takes the next library of the server (T-66) and the
//! keys of the sequence of the library. T-131 gave the engine of the playback to
//! that new application, and **the timer for sleep stayed behind**: every field
//! of a new application starts at its first value.
//!
//! A measurement of 2026-08-13 with tmux and a real server: the key `t` gave the
//! row of the player `💤 4:58`, the key `R` took that text away, and the book
//! played on. The user set a media to stop after five minutes, and it would play
//! for ever.
//!
//! The engine belongs to the playback, therefore the identity of the playback
//! does not change with a refresh: the timer of the old application measures the
//! same media, and it needs no correction.
//!
//! This test needs no sound device and no server. `PlayerHandle::without_engine`
//! gives a handle whose commands go to a channel of the test.

use std::sync::Arc;
use std::time::{Duration, Instant};
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::app::App;
use toutui::db::database_struct::User;
use toutui::logic::sleep_timer::Timer;
use toutui::player::engine::PlayerHandle;

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
async fn a_refresh_keeps_the_timer_of_the_user() {
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

    let (player, _of_the_engine) = PlayerHandle::without_engine();

    let mut of_the_user = App::new_with_the_engine(Arc::clone(&api), Some((player.clone(), None)))
        .await
        .expect("an application");

    // The user pressed the key `t`, and the playback stops after five minutes.
    let timer = Timer {
        ends_at: Instant::now() + Duration::from_secs(300),
        volume: 0.8,
        playback_id: 7,
        label: "5 minutes",
    };

    of_the_user.sleep = Some(timer);
    of_the_user.sleep_choice = Some(5);

    assert!(of_the_user.text_of_the_timer_for_sleep().is_some());

    // The user presses the key `R`.
    let the_state = of_the_user.the_state_that_a_refresh_keeps();

    let mut after_the_refresh = App::new_with_the_engine(Arc::clone(&api), Some((player, None)))
        .await
        .expect("an application");

    // The new application holds no timer of its own. That is the fault of
    // T-135, and the line after it is the correction.
    assert_eq!(after_the_refresh.sleep, None);

    after_the_refresh.keep_the_state_of_the_application_before(the_state);

    assert_eq!(after_the_refresh.sleep, Some(timer));
    assert_eq!(after_the_refresh.sleep_choice, Some(5));

    // The row of the player says the timer again, and it says the same time.
    let text = after_the_refresh
        .text_of_the_timer_for_sleep()
        .expect("the row of the player must hold the timer");

    assert!(text.contains("4:5"), "the row of the player says {}", text);
}

/// **The loop of the program must take those two functions.** The rule lives in
/// `src/main.rs`, and no unit test reaches that loop: a test may read the source
/// of the program, as the test of T-131 does. See T-135.
#[test]
fn the_refresh_of_the_program_keeps_the_state_of_the_user() {
    let source = include_str!("../src/main.rs");

    assert!(
        source.contains("app.the_state_that_a_refresh_keeps()"),
        "the loop of the program must take the state of the user before it makes \
         the new application."
    );

    assert!(
        source.contains("keep_the_state_of_the_application_before("),
        "the loop of the program must give the state of the user to the new \
         application. Without it the timer for sleep of the user goes away, and \
         the media that they set to stop plays on."
    );
}
