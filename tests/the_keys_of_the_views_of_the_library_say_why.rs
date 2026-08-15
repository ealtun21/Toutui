//! The four keys of the views of the library say why a view holds no list.
//! See T-247.
//!
//! **The table of the keys of the program promises four keys in every view.**
//! The group "The views" of that table (the key `?`) names them beside the keys
//! `Tab`, `Shift+Tab`, and `/`, which every view holds:
//!
//! ```text
//!   ▌ The views
//!      Tab             Home, and the Library
//!      Shift+Tab       The next library of the server
//!      /               Search on the server
//!      s               The series of the library
//!      a               The authors of the library
//!      v               The narrators of the library
//!      c               The collections and the playlists
//! ```
//!
//! The four keys do their work in the Home view, in the Library view, and in
//! the view of the search alone: the two arms of the keys `s` and `c` ended in
//! `_ => {}`, and `show_the_names` of the keys `a` and `v` returned with no
//! word. **A key that does nothing must say why** (T-79 and T-83), **and a text
//! must not promise a function that the program does not have** (T-118 and
//! T-143).
//!
//! The measurement of the real program v0.8.75 inside tmux, against the sandbox
//! (podman on :13399), of the library `Podcasts`. The key `Tab` and the key `l`
//! gave the view of the episodes of `Letters of Two Brides`, and the four keys
//! of that view each gave no word of the screen and no line of the log:
//!
//! ```text
//!   s | view=Episodes [57 items] | log 11->11 | (no message)
//!   a | view=Episodes [57 items] | log 11->11 | (no message)
//!   v | view=Episodes [57 items] | log 11->11 | (no message)
//!   c | view=Episodes [57 items] | log 11->11 | (no message)
//! ```
//!
//! **The control of the same run** (the trap 206): the same four keys of the
//! Library view of that same library said `A library of podcasts has no
//! series.`, `A library of podcasts has no author.`, `A library of podcasts has
//! no narrator.`, and the key `c` opened the view `Collections and playlists`.
//! The key `D` of the same line of the same view of the episodes said `"Letter
//! 1" is now available offline.`, therefore the row of the message and the keys
//! of that view do their work.
//!
//! **A message lives six seconds** (T-59), therefore each key of the
//! measurement waited 6.5 seconds after the key before it: a shorter step reads
//! the message of the key before, and every key then looks as if it speaks.
//!
//! The four keys of the corrected program say:
//!
//! ```text
//!   s | The Home view and the Library view show the series of the library. Press h to go back.
//!   a | The Home view and the Library view show the authors of the library. Press h to go back.
//!   v | The Home view and the Library view show the narrators of the library. Press h to go back.
//!   c | The Home view and the Library view show the collections and the playlists. Press h to go back.
//! ```
//!
//! **The parts of this test stay in one function**: two test functions of one
//! module fight for the slot of the message of that module, and `cargo test`
//! then finds a fault that nextest hides (T-144 and T-157).
//!
//! The address is a port that nothing listens on, therefore `App::new` gives
//! the offline mode (T-25) and this test needs no server.
//!
//! **Four builds of the fault fail it**: the arm `_ => {}` of the key `s`, the
//! arm `_ => {}` of the key `c`, and the two roads of `show_the_names` that
//! returned with no word.

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

fn a_key(code: char) -> KeyEvent {
    KeyEvent {
        code: KeyCode::Char(code),
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
async fn the_keys_of_the_views_of_the_library_say_why() {
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

    // **The view of the episodes of a podcast of the measurement.** No line of
    // it holds a series, an author, a narrator, or a list of the library: the
    // four keys need the Home view or the Library view.
    app.is_podcast = true;
    app.view_state = AppView::PodcastEpisode;

    // **The four keys of the measurement, one at a time.** The message of a key
    // stands above every view (T-164), therefore `the_message` of any view
    // gives it, and it forgets it for the key after this one.
    for (key, words) in [
        ('s', toutui::app::THE_SERIES_STAND_IN_TWO_VIEWS),
        ('c', toutui::app::THE_LISTS_STAND_IN_TWO_VIEWS),
    ] {
        app.handle_key(a_key(key));

        assert_eq!(
            the_message(AppView::PodcastEpisode),
            words,
            "the key `{}` of the view of the episodes says why that view holds \
             no list of the library, and it said nothing at all",
            key
        );
    }

    for (key, kind) in [
        ('a', toutui::logic::authors::Kind::Authors),
        ('v', toutui::logic::authors::Kind::Narrators),
    ] {
        app.handle_key(a_key(key));

        assert_eq!(
            the_message(AppView::PodcastEpisode),
            kind.message_of_a_view_that_holds_no_list(),
            "the key `{}` of the view of the episodes says why that view holds \
             no list of the library, and it said nothing at all",
            key
        );
    }

    // **The four keys reach every view that is not one of the three** (T-247):
    // the view of the queue of the measurement gave the same four sentences.
    app.view_state = AppView::Queue;
    app.handle_key(a_key('c'));

    assert_eq!(
        the_message(AppView::Queue),
        toutui::app::THE_LISTS_STAND_IN_TWO_VIEWS,
        "the key `c` of the view of the queue says why that view holds no list \
         of the library"
    );

    // **The control of the same test** (the trap 206): the Library view still
    // does the work of the four keys. A library of podcasts holds no series and
    // no author, therefore the two keys of it say the sentence of T-83, and the
    // key `c` opens the view of the collections and of the playlists.
    app.view_state = AppView::Library;
    app.handle_key(a_key('s'));

    assert_eq!(
        the_message(AppView::Library),
        "A library of podcasts has no series.",
        "the Library view keeps the work of the key `s`"
    );

    app.handle_key(a_key('a'));

    assert_eq!(
        the_message(AppView::Library),
        toutui::logic::authors::Kind::Authors.message_of_a_library_of_podcasts(),
        "the Library view keeps the work of the key `a`"
    );

    app.view_state = AppView::Library;
    app.handle_key(a_key('c'));

    assert_eq!(
        app.view_state,
        AppView::Lists,
        "the key `c` of the Library view opens the collections and the playlists"
    );
    assert_eq!(
        the_message(AppView::Lists),
        "",
        "the key `c` of the Library view opens a view, and it says nothing"
    );
}
