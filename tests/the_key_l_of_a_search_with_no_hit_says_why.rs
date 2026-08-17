//! The key `l` of a search with no hit says why, and it opens no view.
//! See T-375.
//!
//! **The cursor of the view of the search stands at the first line from its
//! birth** (`App::new` selects the line 0, and no code of the program takes
//! that selection away), therefore a search with no hit holds a cursor over a
//! line that the view does not have.
//!
//! The measurement of the real program v0.8.205 inside tmux, against the
//! sandbox (podman on :13399). The library `Podcasts`, the key `/`, the words
//! `zzqxzzqx`, and `Enter` gave `Search result [0 items]` with the reason
//! `The server found nothing for "zzqxzzqx".`. The key `l` of that view then
//! gave the view `Episodes [0 items]` with the words `The program gets the
//! episodes of this podcast…` — the words of a request that the program never
//! made: the log of the program held no line of the episodes, and the words
//! stood after six seconds and more. The same key of the same search of the
//! library `Books` gave no word of the screen and no line of the log
//! (26 -> 26 lines).
//!
//! **The control of the same run** (the trap 206): the key `h` of the view of
//! the episodes of no podcast gave the view of the search back, therefore the
//! keys of the program did their work while the fault stood.
//!
//! The corrected program says `This line holds no media.` for the two kinds of
//! a library, and it stays in the view of the search.
//!
//! **The parts of this test stay in one function**: two test functions of one
//! module fight for the slot of the message of that module, and `cargo test`
//! then finds a fault that nextest hides (T-144 and T-157).
//!
//! The address is a port that nothing listens on, therefore `App::new` gives
//! the offline mode (T-25) and this test needs no server.
//!
//! **The build of the fault fails it**: the guard of the line of the view
//! removed, the arm of a podcast opens `AppView::PodcastEpisode` with every
//! field empty, and the arm of a book says nothing at all.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::app::{App, AppView};
use toutui::db::database_struct::User;

/// Nothing listens on this port.
const NO_SERVER: &str = "http://127.0.0.1:1";

fn a_user() -> User {
    User {
        server_address: NO_SERVER.to_string(),
        username: "toutuitest".to_string(),
        token: "not-a-real-token".to_string(),
        is_default_usr: true,
        name_selected_lib: "Podcasts".to_string(),
        id_selected_lib: "a-library-of-podcasts".to_string(),
        is_loop_break: "0".to_string(),
        has_played_before: "1".to_string(),
        speed_rate: 1.0,
        is_show_key_bindings: "1".to_string(),
    }
}

fn the_key_l() -> KeyEvent {
    KeyEvent {
        code: KeyCode::Char('l'),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: ratatui::crossterm::event::KeyEventState::NONE,
    }
}

/// Gives the message of the screen of a view, and it forgets it.
fn the_message(view: AppView) -> String {
    let text = toutui::logic::message::for_the_screen(view).unwrap_or_default();
    toutui::logic::message::forget();
    text
}

#[tokio::test(flavor = "multi_thread")]
async fn the_key_l_of_a_search_with_no_hit_says_why() {
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

    let mut app = App::new(Arc::clone(&api)).await.expect("an application");

    toutui::logic::message::forget();

    // **The condition of the fault**: the view of the search with no hit, and
    // the cursor of the birth of the application. The titles of the search
    // hold no row, and the cursor stands at the first line.
    assert_eq!(
        app.list_state_search_results.selected(),
        Some(0),
        "the cursor of the view of the search stands at the first line from \
         its birth, and this test measures that condition"
    );
    assert!(
        app.titles_search_book.is_empty(),
        "a search with no hit holds no title"
    );

    // **The arm of a library of podcasts**: the key `l` opened the view of the
    // episodes of no podcast at all.
    app.is_podcast = true;
    app.view_state = AppView::SearchBook;
    app.handle_key(the_key_l());

    assert_eq!(
        app.view_state,
        AppView::SearchBook,
        "the key `l` of a search with no hit of a library of podcasts stays \
         in the view of the search, and it opened the view of the episodes of \
         no podcast at all"
    );
    assert_eq!(
        the_message(AppView::SearchBook),
        "This line holds no media.",
        "the key `l` of a search with no hit of a library of podcasts says \
         why it opens nothing"
    );

    // **The arm of a library of books**: the same key said nothing at all.
    app.is_podcast = false;
    app.handle_key(the_key_l());

    assert_eq!(
        app.view_state,
        AppView::SearchBook,
        "the key `l` of a search with no hit of a library of books stays in \
         the view of the search"
    );
    assert_eq!(
        the_message(AppView::SearchBook),
        "This line holds no media.",
        "the key `l` of a search with no hit of a library of books says why \
         it opens nothing, and it said nothing at all"
    );

    // **The control of the same test** (the trap 206): a search of one hit
    // keeps the work of the key. The lists of the episodes of that line hold
    // no row of this offline application, and `get` gives nothing for each of
    // them (T-126): the view of the episodes opens all the same.
    app.is_podcast = true;
    app.titles_search_book = vec!["Letters of Two Brides".to_string()];
    app.handle_key(the_key_l());

    assert_eq!(
        app.view_state,
        AppView::PodcastEpisode,
        "the key `l` of a search of one hit keeps its work: it opens the view \
         of the episodes of the line"
    );
}
