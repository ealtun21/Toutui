//! The limit of the cache of the ebooks follows `config.toml`. See T-142.
//!
//! **One account can hold two programs**, and both of them write that file: the
//! key `S` of one window gives the cache 4096 MB, and the other window keeps the
//! value that it read at its start. T-140 held the same shape for the row of a
//! listening session.
//!
//! A measurement of 2026-08-13 with two programs of one account, tmux, and the
//! sandbox: the window A gave the cache 4096 MB, the window B then said "The
//! cache of the ebooks — 4096 MB now" on its own screen, and B **removed two
//! books of 105 MB** of the disk to hold a limit of 512 MB. The log of B said
//! "the cache of the ebooks holds 536870912 byte(s) at the most".
//!
//! The limit stands in two places, because the task that removes a book holds no
//! `App` (T-72): `self.config` for the screen, and a slot of the module for the
//! task. **Both of them must come of the file.**
//!
//! This test needs no sound device and no server.

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::app::App;
use toutui::db::database_struct::User;
use toutui::logic::reader::cache;
use toutui::player::engine::PlayerHandle;

/// Nothing listens on this port, therefore `App::new` gives the offline mode
/// and the test needs no server. See T-25.
const NO_SERVER: &str = "http://127.0.0.1:1";

const MEGABYTE: u64 = 1024 * 1024;

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
async fn a_second_program_of_the_account_holds_the_limit_of_the_file() {
    // No line of this test may touch the files of the user.
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", dir.path());
    std::env::remove_var(cache::LIMIT_VARIABLE);
    std::fs::create_dir_all(dir.path().join("toutui")).unwrap();
    std::fs::copy(
        concat!(env!("CARGO_MANIFEST_DIR"), "/config.example.toml"),
        dir.path().join("toutui").join("config.toml"),
    )
    .unwrap();

    toutui::config::write_the_value("reader", "ebook_cache_mb", "512").unwrap();

    let conn = toutui::db::migrate::open_conn().unwrap();
    toutui::db::migrate::run_migrations(&conn).unwrap();
    drop(conn);

    toutui::db::crud::db_insert_usr(&vec![a_user()]).unwrap();

    // The start of the program writes the slot of the limit, as `src/main.rs`
    // does.
    cache::keep_the_limit_of_the_configuration(512);
    assert_eq!(cache::the_limit(), 512 * MEGABYTE);

    let pool = EndpointPool::new(vec![Endpoint::new(NO_SERVER, 0)]);
    let api = Arc::new(ApiClient::new(Arc::new(pool), "token".to_string()).unwrap());
    let (player, _of_the_engine) = PlayerHandle::without_engine();

    // **The second program of this account writes the file**: the user pressed
    // the key `S` in the other window, and that window took 4096 MB.
    toutui::config::write_the_value("reader", "ebook_cache_mb", "4096").unwrap();

    // The user presses the key `R` in this window: a refresh makes a new
    // application, and that application reads the file again.
    let mut of_the_user = App::new_with_the_engine(Arc::clone(&api), Some((player, None)))
        .await
        .expect("an application");

    assert_eq!(
        of_the_user.megabytes_of_the_cache(),
        4096,
        "the new application must hold the value of the file"
    );

    // This is the fault of T-142: the screen said 4096 MB, and the task that
    // removes a book held 512 MB.
    assert_eq!(
        cache::the_limit(),
        4096 * MEGABYTE,
        "the task that removes a book must hold the value of the file"
    );

    // **The view of the settings says the value of the file too.** The other
    // window writes it again, and this window presses no key of a refresh.
    toutui::config::write_the_value("reader", "ebook_cache_mb", "256").unwrap();

    of_the_user.show_the_settings_of_the_reader();

    let line = of_the_user
        .list_state_settings_reader
        .selected()
        .expect("the view holds a line");

    assert_eq!(
        cache::THE_VALUES_OF_THE_SETTINGS.get(line).copied(),
        Some(256),
        "the line of the view must stand on the value of the file"
    );
    assert_eq!(of_the_user.megabytes_of_the_cache(), 256);
    assert_eq!(cache::the_limit(), 256 * MEGABYTE);
}

/// The removal itself reads the file again.
///
/// **The removal takes a book of the disk away**, and it comes with no key of
/// this window: the other window plays no part in it. Therefore the value of the
/// file decides at that moment too. See T-142.
#[test]
fn the_removal_reads_the_file_again() {
    let source = include_str!("../src/logic/reader/session.rs");

    assert!(
        source.contains("cache::read_the_limit_of_the_configuration_again()"),
        "`hold_the_limit_of_the_cache` must read `config.toml` again before it \
         removes a book of the user: a second program of this account writes \
         that file."
    );
}
