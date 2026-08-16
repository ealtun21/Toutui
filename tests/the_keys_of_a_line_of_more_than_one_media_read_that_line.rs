//! The keys `e` and `V` of a line that holds more than one media read that
//! line, and the Home view holds such a line too. See T-222.
//!
//! T-221 gave the keys `D`, `X`, `n`, `m`, `@`, `M`, and `N` of a line of a
//! podcast and of a line of a series the words of that line, and it left the
//! other keys of those lines open. **Two of them read `selected_item_id`**, and
//! that function gives the **first book** of a line of a series (T-91 and
//! T-221): the key `e` of the reader and the key `V` of the bookmarks.
//!
//! The measurement of v0.8.50 against the sandbox, inside tmux:
//!
//! | The view | The line | The key | v0.8.50 |
//! |---|---|---|---|
//! | Library, `Books` | `The Test Chronicles [3 books]` | `e` | the reader of `5a66f3c0-…`, the **first book** of the series |
//! | Library, `Books` | `The Test Chronicles [3 books]` | `V` | the bookmarks of `The Test Chronicles Volume 1` |
//! | Library, `Podcasts` | `Letters of Two Brides` | `e` | the reader asked the server for the ebook of the **podcast** |
//! | Library, `Podcasts` | `Letters of Two Brides` | `V` | `"Letters of Two Brides" has no bookmark. Press b while it plays.` |
//! | Home, `Books` | `Depthless Hunger, Book [1 book]` | `e` | **no word of the screen, and no line of the log** |
//! | Home, `Books` | `Depthless Hunger, Book [1 book]` | `V` | `No media plays, and no media is selected.` |
//! | Home, `Books` | `Depthless Hunger, Book [1 book]` | `D`, `X`, `n` | `This line holds no media.` |
//! | Home, `Books` | `Depthless Hunger, Book [1 book]` | `M`, `N` | `No media is selected.` |
//!
//! **The Home view holds a shelf of series** (T-24): the server gives the shelf
//! `recent-series`, and `the_line_of_no_media` of T-221 read the Library view
//! alone. The same line therefore said two different things in the two views.
//!
//! **This test needs no sandbox and no server.** `App::new` takes a port that
//! nothing listens on, therefore it gives the offline mode (T-25), and the keys
//! of a line of more than one media stop before every request.
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
use toutui::logic::home_view::HomeRow;

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

/// The series of the sandbox, with two of its books.
fn the_series() -> Vec<SeriesView> {
    vec![SeriesView {
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
    }]
}

#[tokio::test(flavor = "multi_thread")]
async fn the_keys_of_a_line_of_more_than_one_media_read_that_line() {
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
    app.series = the_series();
    app.library_rows =
        toutui::logic::library_view::group_library(&app.ids_library, &app.series, false);
    app.list_state_library.select(Some(0));
    app.view_state = AppView::Library;

    // The key `e` opened the reader of the first book of the series, with no
    // word at all.
    app.handle_key(a_key('e'));
    assert_eq!(
        the_message(AppView::Library),
        OF_A_SERIES,
        "the key e of a series says what that line holds"
    );
    assert_eq!(
        app.view_state,
        AppView::Library,
        "the key e of a series opens no reader"
    );

    // The key `V` opened the bookmarks of that same first book.
    app.handle_key(a_key('V'));
    assert_eq!(
        the_message(AppView::Library),
        OF_A_SERIES,
        "the key V of a series says what that line holds"
    );
    assert_eq!(
        app.view_state,
        AppView::Library,
        "the key V of a series opens no view of the bookmarks"
    );

    // **The control of the same run: the book of no series.** The Library view
    // groups the two books of the series in one line (T-22), therefore the line
    // after the series holds `a-book-of-no-series`. The two keys do their work
    // for a line that holds one media.
    app.list_state_library.select(Some(1));

    app.handle_key(a_key('V'));
    assert_eq!(
        app.view_state,
        AppView::Bookmarks,
        "the key V of a book opens the bookmarks"
    );
    assert_eq!(
        app.bookmarks_of, "a-book-of-no-series",
        "the bookmarks of a book belong to that book"
    );

    app.view_state = AppView::Library;
    the_message(AppView::Bookmarks);

    app.handle_key(a_key('e'));
    assert_eq!(
        app.view_state,
        AppView::Reader,
        "the key e of a book opens the reader"
    );

    app.view_state = AppView::Library;
    the_message(AppView::Reader);

    // **A line of a podcast of the Library view.** The library `Podcasts` of
    // the sandbox holds `Letters of Two Brides`, and the ebooks and the
    // bookmarks of a podcast stand in its episodes.
    app.is_podcast = true;
    app.ids_library = vec!["9fa45bd1-66bc-4c17-ba49-a5a6a5ec8806".to_string()];
    app.titles_library = vec!["Letters of Two Brides".to_string()];
    app.auth_names_library = vec!["LibriVox".to_string()];
    app.series = Vec::new();
    app.library_rows =
        toutui::logic::library_view::group_library(&app.ids_library, &app.series, false);
    app.list_state_library.select(Some(0));

    app.handle_key(a_key('e'));
    assert_eq!(
        the_message(AppView::Library),
        OF_A_PODCAST,
        "the key e of a podcast says what that line holds"
    );
    assert_eq!(
        app.view_state,
        AppView::Library,
        "the key e of a podcast opens no reader"
    );

    app.handle_key(a_key('V'));
    assert_eq!(
        the_message(AppView::Library),
        OF_A_PODCAST,
        "the key V of a podcast says what that line holds"
    );

    // **The Home view of a library of books holds a shelf of series.** The
    // shelf `recent-series` of the sandbox holds `Depthless Hunger, Book
    // [1 book]`, and `the_line_of_no_media` of T-221 read the Library view
    // alone.
    app.is_podcast = false;
    app.series = the_series();
    app._ids_cnt_list = vec!["a-book-of-a-shelf".to_string()];
    app._titles_cnt_list = vec!["A Long Test Book".to_string()];
    app.auth_names_cnt_list = vec!["Long Author".to_string()];
    app.home_rows = vec![
        HomeRow::Shelf {
            label: "Continue Listening".to_string(),
        },
        HomeRow::Media { item: 0 },
        HomeRow::Shelf {
            label: "Recent Series".to_string(),
        },
        HomeRow::Series { series: 0 },
    ];
    app.home_rows_of_the_server = app.home_rows.clone();
    app.view_state = AppView::Home;
    app.list_state_cnt_list.select(Some(3));

    assert_eq!(
        app.the_line_of_no_media(),
        TheLineOfNoMedia::ASeries,
        "a line of the shelf of the series of the Home view holds a series"
    );
    assert_eq!(
        app.selected_download(),
        None,
        "a line of a series of the Home view holds no one media"
    );

    for key in ['D', 'X', 'n', 'm', '@', 'e', 'V'] {
        app.handle_key(a_key(key));
        assert_eq!(
            the_message(AppView::Home),
            OF_A_SERIES,
            "the key {} of a series of the Home view says what that line holds",
            key
        );
        assert_eq!(
            app.view_state,
            AppView::Home,
            "the key {} of a series of the Home view opens no other view",
            key
        );
    }

    assert_eq!(
        app.words_of_a_line_with_no_place(),
        "A series holds no place. Press l for its books.",
        "the keys M and N of a series of the Home view name the key of its books"
    );

    // **The control of the same run: the media of the shelf of Continue
    // Listening.** A line of a media of the Home view keeps its keys.
    app.list_state_cnt_list.select(Some(1));

    assert_eq!(
        app.the_line_of_no_media(),
        TheLineOfNoMedia::Nothing,
        "a line of a media of the Home view holds one media"
    );
    assert!(
        app.selected_download().is_some(),
        "the keys of a media of the Home view hold that media"
    );

    // **A line of the Home view of a library of podcasts holds one episode.**
    app.is_podcast = true;

    assert_eq!(
        app.the_line_of_no_media(),
        TheLineOfNoMedia::Nothing,
        "a line of the Home view of a library of podcasts holds one episode"
    );
}
