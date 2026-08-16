//! A line that holds more than one media names the key that opens it. See T-221.
//!
//! **Two lines of the program hold more than one media**: a podcast of the
//! Library view and of the view of the search (its episodes come with the key
//! `l`, T-126), and a line of a series of the Library view (its books come with
//! the same key, T-22). `selected_download` and `selected_place` give nothing
//! for the two of them.
//!
//! The measurement of v0.8.49 against the sandbox, of the line
//! `Letters of Two Brides` of the library `Podcasts` and of the line
//! `The Test Chronicles [3 books]` of the library `Books`:
//!
//! | The line | `D` | `X` | `n` | `m` | `M` and `N` |
//! |---|---|---|---|---|---|
//! | a podcast | nothing | nothing | `This line holds no media.` | `This line holds no book and no episode.` | `A podcast holds no place. Press l for its episodes.` |
//! | a series | nothing | nothing | `This line holds no media.` | `This line holds no book and no episode.` | `A podcast holds no place. Press l for its episodes.` |
//!
//! The keys `D` and `X` wrote **no word of the screen and no line of the log**:
//! that is T-79, and no sweep of the words for the user finds it (T-174 and
//! T-218). The keys `M` and `N` of a **series of a library of books** said that
//! a podcast holds no place: `words_of_a_line_with_no_place` of T-219 read
//! `selected_item_id`, and that function gives the **first book** of a line of a
//! series. That is T-91: the program said a reason that it does not have, and
//! it named a key that gives no episode.
//!
//! **This test needs no sandbox and no server.** `App::new` takes a port that
//! nothing listens on, therefore it gives the offline mode (T-25), and the two
//! keys stop before every request.
//!
//! **The parts of this test stay in one function**: two test functions of one
//! binary take a thread each, and `cargo test` finds a fault of that shape at
//! one run of six (T-144 and T-157).

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::api::utils::collect_series::{SeriesBookView, SeriesView};
use toutui::app::{App, AppView, TheLineOfNoMedia};
use toutui::db::database_struct::User;

/// Nothing listens on this port.
const NO_SERVER: &str = "http://127.0.0.1:1";

const OF_A_PODCAST: &str = "A podcast holds more than one media. Press l for its episodes.";
const OF_A_SERIES: &str = "A series holds more than one book. Press l for its books.";

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
async fn a_line_of_more_than_one_media_names_the_key_that_opens_it() {
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

    // The server refuses a download for an account that may not download, and
    // the key `D` says that before every other word.
    app.account.permissions.download = true;

    // **A podcast of the Library view.** The library `Podcasts` of the sandbox
    // holds `Letters of Two Brides`.
    app.is_podcast = true;
    app.ids_library = vec!["9fa45bd1-66bc-4c17-ba49-a5a6a5ec8806".to_string()];
    app.titles_library = vec!["Letters of Two Brides".to_string()];
    app.auth_names_library = vec!["LibriVox".to_string()];
    app.library_rows =
        toutui::logic::library_view::group_library(&app.ids_library, &app.series, false);
    app.list_state_library.select(Some(0));
    app.view_state = AppView::Library;

    assert_eq!(
        app.the_line_of_no_media(),
        TheLineOfNoMedia::APodcast,
        "a line of a library of podcasts holds a podcast"
    );
    assert_eq!(
        app.selected_download(),
        None,
        "a podcast holds no one media"
    );

    app.handle_key(a_key('D'));
    assert_eq!(
        the_message(AppView::Library),
        OF_A_PODCAST,
        "the key D of a podcast says what that line holds"
    );

    app.handle_key(a_key('X'));
    assert_eq!(
        the_message(AppView::Library),
        OF_A_PODCAST,
        "the key X of a podcast says what that line holds"
    );

    app.handle_key(a_key('n'));
    assert_eq!(
        the_message(AppView::Library),
        OF_A_PODCAST,
        "the key n of a podcast says what that line holds"
    );

    app.handle_key(a_key('m'));
    assert_eq!(
        the_message(AppView::Library),
        OF_A_PODCAST,
        "the key m of a podcast says what that line holds"
    );

    assert_eq!(
        app.words_of_a_line_with_no_place(),
        "A podcast holds no place. Press l for its episodes.",
        "the keys M and N of a podcast name the key of its episodes"
    );

    // **The view of the search of a library of podcasts holds the same line.**
    app.ids_search_book = app.ids_library.clone();
    app.titles_search_book = app.titles_library.clone();
    app.auth_names_search_book = app.auth_names_library.clone();
    app.list_state_search_results.select(Some(0));
    app.view_state = AppView::SearchBook;

    assert_eq!(
        app.the_line_of_no_media(),
        TheLineOfNoMedia::APodcast,
        "a line of the search of a library of podcasts holds a podcast"
    );

    app.handle_key(a_key('D'));
    assert_eq!(
        the_message(AppView::SearchBook),
        OF_A_PODCAST,
        "the key D of the view of the search says what that line holds"
    );

    // **A line of a series of the Library view.** The library `Books` of the
    // sandbox holds `The Test Chronicles [3 books]`.
    app.is_podcast = false;
    app.ids_library = vec![
        "the-first-book".to_string(),
        "the-second-book".to_string(),
        "a-book-of-no-series".to_string(),
    ];
    app.titles_library = vec![
        "The Test Chronicles, Book 1".to_string(),
        "The Test Chronicles, Book 2".to_string(),
        "Alice in Wonderland".to_string(),
    ];
    app.auth_names_library = vec![
        "A Test Author".to_string(),
        "A Test Author".to_string(),
        "Lewis Carroll".to_string(),
    ];
    app.series = vec![SeriesView {
        id: "a-series".to_string(),
        name: "The Test Chronicles".to_string(),
        description: String::new(),
        books: vec![
            SeriesBookView {
                id: "the-first-book".to_string(),
                title: "The Test Chronicles, Book 1".to_string(),
                author: "A Test Author".to_string(),
                sequence: "1".to_string(),
                duration: 60.0,
                description: String::new(),
            },
            SeriesBookView {
                id: "the-second-book".to_string(),
                title: "The Test Chronicles, Book 2".to_string(),
                author: "A Test Author".to_string(),
                sequence: "2".to_string(),
                duration: 60.0,
                description: String::new(),
            },
        ],
    }];
    app.library_rows =
        toutui::logic::library_view::group_library(&app.ids_library, &app.series, false);
    app.list_state_library.select(Some(0));
    app.view_state = AppView::Library;

    assert_eq!(
        app.the_line_of_no_media(),
        TheLineOfNoMedia::ASeries,
        "the first line of that library holds the series"
    );
    assert_eq!(app.selected_download(), None, "a series holds no one media");

    app.handle_key(a_key('D'));
    assert_eq!(
        the_message(AppView::Library),
        OF_A_SERIES,
        "the key D of a series says what that line holds"
    );

    app.handle_key(a_key('X'));
    assert_eq!(
        the_message(AppView::Library),
        OF_A_SERIES,
        "the key X of a series says what that line holds"
    );

    app.handle_key(a_key('n'));
    assert_eq!(
        the_message(AppView::Library),
        OF_A_SERIES,
        "the key n of a series says what that line holds"
    );

    app.handle_key(a_key('m'));
    assert_eq!(
        the_message(AppView::Library),
        OF_A_SERIES,
        "the key m of a series says what that line holds"
    );

    // **The words of T-219 named a podcast for this line.**
    assert_eq!(
        app.words_of_a_line_with_no_place(),
        "A series holds no place. Press l for its books.",
        "the keys M and N of a series name the key of its books"
    );

    // **The book of the same library keeps its keys.** The line after the
    // series holds no series, therefore the keys do their work.
    app.list_state_library.select(Some(1));

    assert_eq!(
        app.the_line_of_no_media(),
        TheLineOfNoMedia::Nothing,
        "a line of a book holds one media"
    );
    assert!(
        app.selected_download().is_some(),
        "the keys D and X of a book hold that book"
    );

    // **A view that holds no line of a media keeps the words of its key.**
    app.view_state = AppView::Settings;

    assert_eq!(
        app.the_line_of_no_media(),
        TheLineOfNoMedia::Nothing,
        "the view of the settings holds no line of a media"
    );
    assert_eq!(
        app.words_of_a_line_with_no_media("This line holds no media."),
        "This line holds no media.",
        "a view with no media says that the line holds none"
    );
    assert_eq!(
        app.words_of_a_line_with_no_place(),
        "No media is selected.",
        "the keys M and N of such a view say that no media is selected"
    );
}
