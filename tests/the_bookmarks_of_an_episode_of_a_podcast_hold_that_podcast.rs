//! The bookmarks of an episode of a podcast are the bookmarks of that podcast.
//! See T-223.
//!
//! **A bookmark of Audiobookshelf names an item, and a podcast is one item.**
//! `POST /api/me/item/<an episode>/bookmark` of the sandbox (2.36.0) answers
//! `404`, therefore the places of every episode of one podcast stand in one list
//! and no field of a bookmark names an episode. T-219 and T-222 each left that
//! question open.
//!
//! The measurement of v0.8.51 against the sandbox, inside tmux, of the podcast
//! `Arthur Gordon Pym` of 11 episodes and of its two bookmarks:
//!
//! | The view | The line, or the playback | The key | v0.8.51 |
//! |---|---|---|---|
//! | The episodes of the podcast | `Chapter 00`, no playback | `V` | `No media plays, and no media is selected.`, and no line of the log |
//! | Home, `Podcasts` | `Chapter 02` of Continue Listening | `V` | `The bookmarks of "Chapter 02" [2 items]`, and one line of it is a place of `Chapter 05` |
//! | the playback of `Chapter 02` at 12:58 | — | `b` | the bookmark stands on the **podcast** `b793354b-…` |
//! | the playback of `Chapter 01` at 21:59 | the line `A place of Chapter 02` | `l` | `The playback goes to "A place of Chapter 02".`, and the playback of `Chapter 01` went to 12:58 |
//!
//! **The user asked for the bookmarks of one episode, and the program named
//! that episode above the places of the whole podcast** (T-91). The key of the
//! place then moved the playback of one episode to a place of another one, and
//! the program said nothing of that.
//!
//! **This test needs no sandbox and no server.** `App::new` takes a port that
//! nothing listens on, therefore it gives the offline mode (T-25), and the key
//! `V` of an offline application writes the state of the view before every
//! request.
//!
//! **The parts of this test stay in one function**: two test functions of one
//! binary take a thread each, and `cargo test` finds a fault of that shape at
//! one run of six (T-144 and T-157).

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::api::me::bookmarks::Bookmark;
use toutui::app::{App, AppView};
use toutui::db::database_struct::User;
use toutui::logic::home_view::HomeRow;

/// Nothing listens on this port.
const NO_SERVER: &str = "http://127.0.0.1:1";

/// The podcast of the sandbox, and one episode of it.
const THE_PODCAST: &str = "b793354b-9841-480a-bd09-41923596517e";
const THE_EPISODE: &str = "ff28a3b0-4ade-4a41-a3c3-864d264354a7";
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
async fn the_bookmarks_of_an_episode_of_a_podcast_hold_that_podcast() {
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

    // **The words of the two titles and of the key of the place.** The list of
    // a podcast holds the places of every episode of it, therefore the title
    // names the podcast and the key promises no place of one episode.
    let of_a_podcast =
        toutui::logic::bookmarks::the_title(THE_NAME_OF_THE_PODCAST, "2 items", true);

    assert!(
        of_a_podcast.contains("podcast") && of_a_podcast.contains(THE_NAME_OF_THE_PODCAST),
        "the title of the list of a podcast names the podcast: {}",
        of_a_podcast
    );
    assert_eq!(
        toutui::logic::bookmarks::the_title("A Long Test Book", "1 item", false),
        "The bookmarks of \"A Long Test Book\" [1 item]",
        "the title of the list of a book does not change"
    );

    let no_bookmark =
        toutui::logic::bookmarks::the_title_of_no_bookmark(THE_NAME_OF_THE_PODCAST, true);

    assert!(
        no_bookmark.contains("an episode plays"),
        "a podcast plays no media of its own, therefore the text names an episode: {}",
        no_bookmark
    );

    let of_the_place = toutui::logic::bookmarks::the_text_of_a_place_of_a_podcast("A place");

    assert!(
        of_the_place.contains("names no episode"),
        "the key of the place says what the program cannot know: {}",
        of_the_place
    );

    // **The Home view of a library of podcasts.** The shelf Continue Listening
    // of the sandbox held the line `Chapter 02`, and the identity of that line
    // is the identity of the **podcast**: the title of the view said
    // `The bookmarks of "Chapter 02"` above a place of `Chapter 05`.
    app.is_podcast = true;
    app._ids_cnt_list = vec![THE_PODCAST.to_string()];
    app.ids_ep_cnt_list = vec![THE_EPISODE.to_string()];
    app._titles_cnt_list = vec!["Chapter 02".to_string()];
    app.titles_pod_cnt_list = vec![THE_NAME_OF_THE_PODCAST.to_string()];
    app.auth_names_cnt_list = vec!["LibriVox".to_string()];
    app.home_rows = vec![
        HomeRow::Shelf {
            label: "Continue Listening".to_string(),
        },
        HomeRow::Media { item: 0 },
    ];
    app.home_rows_of_the_server = app.home_rows.clone();
    app.view_state = AppView::Home;
    app.list_state_cnt_list.select(Some(1));

    app.handle_key(a_key('V'));

    assert_eq!(
        app.view_state,
        AppView::Bookmarks,
        "the key V of a line of an episode opens the bookmarks"
    );
    assert_eq!(
        app.bookmarks_of, THE_PODCAST,
        "the bookmarks of an episode are the bookmarks of its podcast"
    );
    assert_eq!(
        app.bookmarks_of_name, THE_NAME_OF_THE_PODCAST,
        "the view names the podcast, and not the episode of the line"
    );
    assert!(
        app.bookmarks_of_a_podcast,
        "the media of the view is a podcast"
    );

    the_message(AppView::Bookmarks);

    // **The key of the place, with no playback.** A podcast plays no media of
    // its own, therefore the sentence names an episode of it (T-221).
    toutui::logic::bookmarks::keep(toutui::logic::bookmarks::State::Ready(vec![Bookmark {
        library_item_id: THE_PODCAST.to_string(),
        time: 778.0,
        title: "A place of Chapter 02".to_string(),
    }]));
    app.list_state_bookmarks.select(Some(0));

    app.handle_key(a_key('l'));

    assert_eq!(
        the_message(AppView::Bookmarks),
        "Play an episode of this podcast first, and the bookmark then gives its place.",
        "the key of the place of a podcast names an episode of it"
    );

    // **The view of the episodes of a podcast.** `selected_item_id` holds no
    // arm of this view, therefore the key said "No media plays, and no media is
    // selected." while the podcast held two places of the user (T-219).
    app.bookmarks_of = String::new();
    app.bookmarks_of_name = String::new();
    app.bookmarks_of_a_podcast = false;
    app.is_from_search_pod = false;
    app.ids_library = vec![THE_PODCAST.to_string()];
    app.titles_library = vec![THE_NAME_OF_THE_PODCAST.to_string()];
    app.titles_pod = vec![THE_NAME_OF_THE_PODCAST.to_string()];
    app.ids_pod_ep = vec![THE_EPISODE.to_string()];
    app.titles_pod_ep = vec!["Chapter 02".to_string()];
    app.list_state_library.select(Some(0));
    app.list_state_pod_ep.select(Some(0));
    app.view_state = AppView::PodcastEpisode;

    app.handle_key(a_key('V'));

    assert_eq!(
        app.view_state,
        AppView::Bookmarks,
        "the key V of the view of the episodes opens the bookmarks"
    );
    assert_eq!(
        app.bookmarks_of, THE_PODCAST,
        "the bookmarks of the view of the episodes belong to the podcast"
    );
    assert!(
        app.bookmarks_of_a_podcast,
        "the media of the view is a podcast"
    );
    assert_eq!(
        the_message(AppView::PodcastEpisode),
        "",
        "the key V of an episode says no word of a line with no media"
    );

    // **The control of the same run: a book.** The Library view of a library of
    // books keeps the words and the road of T-163 and of T-222.
    app.bookmarks_of_a_podcast = false;
    app.is_podcast = false;
    app.series = Vec::new();
    app.ids_library = vec!["a-book-of-no-series".to_string()];
    app.titles_library = vec!["A Long Test Book".to_string()];
    app.auth_names_library = vec!["Long Author".to_string()];
    app.library_rows =
        toutui::logic::library_view::group_library(&app.ids_library, &app.series, false);
    app.list_state_library.select(Some(0));
    app.view_state = AppView::Library;

    app.handle_key(a_key('V'));

    assert_eq!(
        app.bookmarks_of, "a-book-of-no-series",
        "the bookmarks of a book belong to that book"
    );
    assert_eq!(
        app.bookmarks_of_name, "A Long Test Book",
        "the view of a book names that book"
    );
    assert!(
        !app.bookmarks_of_a_podcast,
        "the media of the view of a book is no podcast"
    );

    the_message(AppView::Bookmarks);
    app.list_state_bookmarks.select(Some(0));
    toutui::logic::bookmarks::keep(toutui::logic::bookmarks::State::Ready(vec![Bookmark {
        library_item_id: "a-book-of-no-series".to_string(),
        time: 10.0,
        title: "A place of the long book".to_string(),
    }]));

    app.handle_key(a_key('l'));

    assert_eq!(
        the_message(AppView::Bookmarks),
        "Play this media first, and the bookmark then gives its place.",
        "the key of the place of a book keeps its words"
    );
}
