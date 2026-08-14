//! The key `b` of the view of the bookmarks of a podcast writes a place of the
//! episode that the user opened. See T-224.
//!
//! **One identity names every episode of one podcast** (T-223). The guard of
//! T-163 asked "does the media of this view play now?" with the identity of the
//! **item** alone, therefore a second episode of the podcast of the view passed
//! it: the user opened the places of one episode, the queue started another
//! episode of that same podcast with no key of the user, and the key `b` wrote a
//! place of that other episode with no word at all.
//!
//! The measurement of v0.8.52 against the sandbox, inside tmux, of the podcast
//! `Arthur Gordon Pym` of the library `Podcasts`:
//!
//! | The moment | The screen | The server |
//! |---|---|---|
//! | `Chapter 02` in the queue, `Chapter 01` plays, the key `V` | `The bookmarks of the podcast "Arthur Gordon Pym" [1 item]`, and the row of the player says `▶ 5:06 / 21:59` | one bookmark, the time 60 |
//! | the queue starts `Chapter 02` with no key of the user | the same view, and the row of the player says `▶ 1:26 / 38:56` | the same |
//! | the key `b`, and the name `A place of Chapter 01` | `The bookmarks of the podcast "Arthur Gordon Pym" [2 items]` | a second bookmark, **the time 96** |
//!
//! The second 96 is a place of `Chapter 02`, and the user listened to
//! `Chapter 01`. **The row of the player names the podcast alone**
//! (`Arthur Gordon Pym by LibriVox`), therefore no part of the screen said that
//! the episode changed, and no line of the log named it.
//!
//! **The program cannot name the episode of a place** (T-223): no field of a
//! bookmark of Audiobookshelf names an episode. The words of this item name the
//! podcast and the key `V`, which shows the bookmarks with the episode that
//! plays now.
//!
//! **This test needs no sandbox and no server.** `App::new` takes a port that
//! nothing listens on, therefore it gives the offline mode (T-25).
//!
//! **The road that this test does not walk**: the guard of the key `b` reads
//! `state.episode_id` of the engine, and an engine that plays stands in tmux
//! alone. The table above is that measurement. See T-223, which left the two
//! roads of `state.episode_id` in the same place.
//!
//! **The parts of this test stay in one function**: two test functions of one
//! binary take a thread each, and `cargo test` finds a fault of that shape at
//! one run of six (T-144 and T-157).

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::app::{App, AppView};
use toutui::db::database_struct::User;
use toutui::logic::bookmarks::TheMediaOfTheBookmarks;

/// Nothing listens on this port.
const NO_SERVER: &str = "http://127.0.0.1:1";

/// The podcast of the sandbox, and two episodes of it.
const THE_PODCAST: &str = "b793354b-9841-480a-bd09-41923596517e";
const THE_EPISODE_OF_THE_USER: &str = "482f0136-06eb-44a2-a202-c2ea3ad68a53";
const THE_EPISODE_OF_THE_QUEUE: &str = "ff28a3b0-4ade-4a41-a3c3-864d264354a7";
const THE_NAME_OF_THE_PODCAST: &str = "Arthur Gordon Pym";

fn a_user() -> User {
    User {
        server_address: NO_SERVER.to_string(),
        username: "toutuitest".to_string(),
        token: "not-a-real-token".to_string(),
        is_default_usr: true,
        name_selected_lib: "Podcasts".to_string(),
        id_selected_lib: "a-library".to_string(),
        is_loop_break: "0".to_string(),
        has_played_before: "1".to_string(),
        speed_rate: 1.0,
        is_show_key_bindings: "1".to_string(),
    }
}

/// The key of the user, with no modifier.
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
async fn a_second_episode_of_a_podcast_takes_no_place() {
    // **The decision of the key `b`.** The identity of the item passes for every
    // episode of one podcast, therefore the episode decides.
    assert_eq!(
        toutui::logic::bookmarks::what_the_media_of_the_bookmarks_is(
            THE_PODCAST,
            Some(THE_PODCAST),
            Some(THE_EPISODE_OF_THE_USER),
            Some(THE_EPISODE_OF_THE_QUEUE),
        ),
        TheMediaOfTheBookmarks::AnotherEpisodePlays,
        "the queue started a second episode of the podcast of the view"
    );
    assert_eq!(
        toutui::logic::bookmarks::what_the_media_of_the_bookmarks_is(
            THE_PODCAST,
            Some(THE_PODCAST),
            Some(THE_EPISODE_OF_THE_USER),
            Some(THE_EPISODE_OF_THE_USER),
        ),
        TheMediaOfTheBookmarks::ItPlays,
        "the episode of the view plays, and the key writes its place"
    );

    // **The control: a book.** One identity names one book, therefore the rule
    // of T-163 stands with no change of its words.
    assert_eq!(
        toutui::logic::bookmarks::what_the_media_of_the_bookmarks_is(
            "a-book",
            Some("a-book"),
            None,
            None,
        ),
        TheMediaOfTheBookmarks::ItPlays,
        "the book of the view plays"
    );
    assert_eq!(
        toutui::logic::bookmarks::what_the_media_of_the_bookmarks_is(
            "a-book",
            Some("another-book"),
            None,
            None,
        ),
        TheMediaOfTheBookmarks::ItDoesNotPlay,
        "the queue started another book"
    );

    // **The words.** They name the podcast, and they name no episode: no answer
    // of the server gives this program the name of the episode of a place
    // (T-223). They name the key `V`, and that key shows the bookmarks with the
    // episode that plays (T-118 and T-143).
    let text = toutui::logic::bookmarks::the_text_of_another_episode(THE_NAME_OF_THE_PODCAST);

    assert!(
        text.contains(THE_NAME_OF_THE_PODCAST),
        "the sentence names the podcast: {}",
        text
    );
    assert!(
        text.contains("different episode"),
        "the sentence says that the episode changed: {}",
        text
    );
    assert!(
        text.contains("key V"),
        "the sentence names the key of the road back: {}",
        text
    );
    assert_ne!(
        text,
        toutui::logic::bookmarks::the_text_of_the_media_that_does_not_play(THE_NAME_OF_THE_PODCAST),
        "the podcast of the view plays, therefore the sentence of a media that \
         does not play is not true"
    );

    // **The view keeps the episode.** The key `V` of a line of an episode gives
    // `bookmarks_of` the identity of the podcast (T-223), therefore the episode
    // of that line is the one value that tells the places of the user from the
    // places of the episode of the queue.
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

    app.is_podcast = true;
    app.is_from_search_pod = false;
    app.ids_library = vec![THE_PODCAST.to_string()];
    app.titles_library = vec![THE_NAME_OF_THE_PODCAST.to_string()];
    app.titles_pod = vec![THE_NAME_OF_THE_PODCAST.to_string()];
    app.ids_pod_ep = vec![THE_EPISODE_OF_THE_USER.to_string()];
    app.titles_pod_ep = vec!["Chapter 01".to_string()];
    app.list_state_library.select(Some(0));
    app.list_state_pod_ep.select(Some(0));
    app.view_state = AppView::PodcastEpisode;

    app.handle_key(a_key('V'));

    assert_eq!(
        app.bookmarks_of, THE_PODCAST,
        "the bookmarks of an episode are the bookmarks of its podcast"
    );
    assert_eq!(
        app.bookmarks_of_episode.as_deref(),
        Some(THE_EPISODE_OF_THE_USER),
        "the view keeps the episode that the user opened"
    );

    the_message(AppView::Bookmarks);

    // **The control of the same run: a book.** A view of a book holds no
    // episode, therefore the guard of T-163 reads the identity of the item
    // alone and its words do not change.
    app.is_podcast = false;
    app.series = Vec::new();
    app.ids_library = vec!["a-book-of-no-series".to_string()];
    app.titles_library = vec!["A Long Test Book".to_string()];
    app.auth_names_library = vec!["Long Author".to_string()];
    app.library_rows = toutui::logic::library_view::group_library(&app.ids_library, &app.series);
    app.list_state_library.select(Some(0));
    app.view_state = AppView::Library;

    app.handle_key(a_key('V'));

    assert_eq!(
        app.bookmarks_of, "a-book-of-no-series",
        "the bookmarks of a book belong to that book"
    );
    assert_eq!(app.bookmarks_of_episode, None, "a book holds no episode");

    the_message(AppView::Bookmarks);

    // **No media plays**, therefore the key `b` of the view of a book keeps the
    // words of T-163 with no change at all.
    app.handle_key(a_key('b'));

    assert_eq!(
        the_message(AppView::Bookmarks),
        toutui::logic::bookmarks::the_text_of_the_media_that_does_not_play("A Long Test Book"),
        "the media of the view of a book does not play, and its words stand"
    );
}
