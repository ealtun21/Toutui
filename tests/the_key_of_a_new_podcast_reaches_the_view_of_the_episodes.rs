//! The key `A` of a new podcast reaches the view of the episodes, and it says
//! why every other view adds no podcast. See T-248.
//!
//! **The table of the keys of the program promises this key in every view.** The
//! group "The library and the server" of that table (the key `?`) holds five
//! keys:
//!
//! ```text
//!   ▌ The library and the server
//!      R               Ask the server for every list again
//!      L               The server examines the library
//!      A               Add a podcast to the library
//!      E               The server gets the new episodes of the feed
//!      d               The episodes that the server downloads, and the queue
//! ```
//!
//! Four of the five do their work in the view of the episodes of a podcast: the
//! key `E` of that view said `The server gets 16 episode(s).` in the sweep of
//! T-247. **The key `A` alone did nothing and said nothing there**:
//! `look_for_a_podcast` held `Home | Library | SearchBook | NewPodcast` and an
//! early `return` with no word.
//!
//! The measurement of the real program v0.8.76 inside tmux, against the sandbox
//! (podman on :13399), of the library `Podcasts`. The key `Tab` and the key `l`
//! gave `Episodes [57 items]` of `Letters of Two Brides`, and the key `A` of
//! that view gave no field, no message, and no line of the log:
//!
//! ```text
//!   A | view=Episodes [57 items] | log 11->11 | (no message, no field)
//! ```
//!
//! **The control of the same run** (the trap 206): the key `V` of that same view
//! said `The podcast "Letters of Two Brides" has no bookmark. Press b while an
//! episode plays.`, therefore the row of the message of that view does its work;
//! and the key `A` of the Library view of that same library opened the field
//! `The name of the podcast (Enter, or Esc)`, therefore the key does its work
//! where the guard holds it.
//!
//! **A message lives six seconds** (T-59), therefore the key of the measurement
//! waited 6.5 seconds (the trap 220).
//!
//! The corrected program takes the key `A` in the view of the episodes — that
//! view belongs to a library of podcasts, and the key needs no line of any list
//! — and it says the sentence of T-247 in every view that adds no podcast.
//!
//! **The parts of this test stay in one function**: two test functions of one
//! module fight for the slot of the message of that module, and `cargo test`
//! then finds a fault that nextest hides (T-144 and T-157).
//!
//! The address is a port that nothing listens on, therefore `App::new` gives the
//! offline mode (T-25) and this test needs no server. **That mode is the value
//! of the measurement**: the sentence `The server does not answer.` stands after
//! the guard of the view and before the field of the text, therefore this test
//! reads the road of the key with no terminal at all.
//!
//! **Two builds of the fault fail it**: `AppView::PodcastEpisode` out of the
//! list of the views, and the `say` of the sentence out of the early return.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::app::{App, AppView};
use toutui::db::database_struct::User;

/// Nothing listens on this port.
const NO_SERVER: &str = "http://127.0.0.1:1";

/// The words of the offline mode. They stand after the guard of the view of
/// `look_for_a_podcast`, therefore a view that reaches them is a view that the
/// key holds.
const THE_SERVER_IS_AWAY: &str = "The server does not answer.";

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
async fn the_key_of_a_new_podcast_reaches_the_view_of_the_episodes() {
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

    // The condition of the measurement: a library of podcasts, and the view of
    // the episodes of one of them.
    app.is_podcast = true;
    app.is_offline = true;
    app.view_state = AppView::PodcastEpisode;

    app.handle_key(a_key('A'));

    assert_eq!(
        the_message(AppView::PodcastEpisode),
        THE_SERVER_IS_AWAY,
        "the key `A` of the view of the episodes of a podcast adds a podcast to \
         the library of that view, and it did nothing at all"
    );

    // **The key reaches every view that adds no podcast** (T-248), and it says
    // why. The view of the queue holds no library.
    for view in [AppView::Queue, AppView::Bookmarks, AppView::Chapters] {
        app.view_state = view;
        app.handle_key(a_key('A'));

        assert_eq!(
            the_message(view),
            toutui::app::THE_NEW_PODCAST_STANDS_IN_TWO_VIEWS,
            "the key `A` of {:?} says why that view adds no podcast to the \
             library, and it said nothing at all",
            view
        );
    }

    // **The control of the same test** (the trap 206): the Library view keeps
    // the work of the key, and it reaches the same words of the offline mode.
    app.view_state = AppView::Library;
    app.handle_key(a_key('A'));

    assert_eq!(
        the_message(AppView::Library),
        THE_SERVER_IS_AWAY,
        "the Library view keeps the work of the key `A`"
    );

    // A library of books holds no podcast, and that sentence stands before the
    // words of the offline mode. It names the key of the next library.
    app.is_podcast = false;
    app.view_state = AppView::PodcastEpisode;
    app.handle_key(a_key('A'));

    assert_eq!(
        the_message(AppView::PodcastEpisode),
        "This library holds books. Choose a library of podcasts with S.",
        "the key `A` of a library of books says that a library of books holds \
         no podcast"
    );
}
