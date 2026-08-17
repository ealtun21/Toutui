//! The gate of T-359: the footer of a view with no line names no key of a line.
//!
//! The measurement of the real program v0.8.189 inside tmux, at 160 columns and
//! 45 rows, of the library `Empty` of the sandbox (`docs/TEST-SERVER.md`). That
//! library holds no media, no collection, and no playlist.
//!
//! **A footer must not promise a key that the view does not hold** (T-143, and
//! T-79 for a key that does nothing). The Collections view of that library held
//! no line, and its footer said:
//!
//! ```text
//! j/k: move  l: the media  r/D: a name/description  X: remove  h: back  ?: every key  Q: quit
//! ```
//!
//! No line of that view holds a media, a name, or a description, therefore four
//! of the seven parts named a key that does nothing. The Series view of the same
//! run said `j/k: move  l: take the line  …`, the Authors view said
//! `j/k: move  l: the books of this author  …`, the Narrators view said
//! `j/k: move  l: the books of this narrator  …`, the Chapters view of a program
//! that plays nothing said `j/k: move  l: go to the chapter  …`, the Queue view
//! of an empty queue said `j/k: move  l: play it now  X: take it out  …`, and the
//! Home view of that library said `j/k: move  l: play or open  …`.
//!
//! The corrected program of the same harness and of the same run gave
//! `h: back  ?: every key  Q: quit` to the Collections view, the Authors view,
//! the Narrators view, the Chapters view, and the Queue view;
//! `h: back  Tab: home  R: refresh  ?: every key  Q: quit` to the Series view;
//! and
//! `Tab: home/library  S-Tab: the next library  /: search  R: refresh  ?: every
//! key  Q: quit  f: sequence  1/Ctrl+h: the panels  z: hide them` to the Home
//! view. **The controls of the same run**, of the library `Books`, kept every
//! part of every footer.
//!
//! **The filter takes its parts out of the footer of the view and not of the
//! footer of the panel that holds the focus** (T-320): the keys `j`, `k`, and
//! `l` of the panel 1, of the panel 2, and of the panel 3 move the lines of that
//! panel, and those lines stand there while the list of the view holds none.
//!
//! **The parts of this test stay in one function**: the test writes
//! `XDG_CONFIG_HOME` of the process, and a second function of this binary would
//! fight it for that box (T-144 and T-157).
//!
//! **This test needs no sandbox and no server.** `App::new` takes a port that
//! nothing listens on, therefore it gives the offline mode (T-25).

use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::api::utils::collect_lists::{ListEntry, ListKind, ListView};
use toutui::app::{App, AppView};
use toutui::db::database_struct::User;

/// Nothing listens on this port. See T-25.
const NO_SERVER: &str = "http://127.0.0.1:1";

fn a_user() -> User {
    User {
        server_address: NO_SERVER.to_string(),
        username: "toutuitest".to_string(),
        token: "not-a-real-token".to_string(),
        is_default_usr: true,
        name_selected_lib: "Empty".to_string(),
        id_selected_lib: "a-library".to_string(),
        is_loop_break: "0".to_string(),
        has_played_before: "1".to_string(),
        speed_rate: 1.0,
        is_show_key_bindings: "1".to_string(),
    }
}

/// The rows of the screen, from the top to the bottom.
fn the_rows_of(terminal: &Terminal<TestBackend>) -> Vec<String> {
    let buffer = terminal.backend().buffer().clone();

    (0..buffer.area.height)
        .map(|row| {
            (0..buffer.area.width)
                .map(|column| buffer[(column, row)].symbol())
                .collect::<String>()
        })
        .collect()
}

/// The whole screen in one text. **A footer of many rows is one text** (T-302):
/// the wrap of a narrow terminal cuts the footer in two rows, therefore a test
/// of a part of it reads the rows together and not one row of them.
fn the_screen_of(terminal: &Terminal<TestBackend>) -> String {
    the_rows_of(terminal).join(" ")
}

#[tokio::test(flavor = "multi_thread")]
async fn the_footer_of_a_view_with_no_line() {
    // ## 1. The rule itself, with no screen at all
    //
    // `keys::the_footer_of_a_view_with_no_line` is a pure function, therefore it
    // takes a test of its own before the test of the render.
    use toutui::ui::keys;

    assert_eq!(
        keys::the_footer_of_a_view_with_no_line(keys::FOOTER_OF_THE_LISTS),
        "h: back  ?: every key  Q: quit",
        "the footer of the Collections view with no line names no key of a line"
    );

    assert_eq!(
        keys::the_footer_of_a_view_with_no_line(keys::FOOTER_OF_A_LIST),
        "h: back  Tab: home  R: refresh  ?: every key  Q: quit",
        "the footer of a list of names with no line keeps the keys of the view"
    );

    // **The key `h` of `h: back` stays, and the key `h/l` of a cover of a band
    // goes away**: the key of a part of a footer stands before the first `": "`
    // of that part.
    let of_the_bands = keys::the_footer_of_a_view_with_no_line(keys::FOOTER_OF_THE_BANDS_OF_BOOKS);
    assert!(
        !of_the_bands.contains("h/l: a cover") && !of_the_bands.contains("Enter: play or open"),
        "the footer of the bands with no shelf names no key of a cover: {of_the_bands}"
    );
    assert!(
        of_the_bands.contains("/: search") && of_the_bands.contains("R: refresh"),
        "the footer of the bands with no shelf keeps the keys of the view: {of_the_bands}"
    );

    // **A footer that loses every part is the footer of a fault** (T-52): a
    // screen that names no key looks like a program that stopped.
    assert_eq!(
        keys::the_footer_of_a_view_with_no_line("j/k: move  l: take the line"),
        keys::FOOTER_OF_A_FAULT,
        "a footer of the keys of a line alone becomes the footer of a fault"
    );

    // **A view of one line keeps every part**, which is the control of the rule.
    assert_eq!(
        keys::the_footer_of_a_list(keys::FOOTER_OF_THE_LISTS, 1),
        keys::FOOTER_OF_THE_LISTS,
        "a view of one line keeps every part of its footer"
    );

    // ## 2. The views of the render
    //
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
    let mut app = App::new(Arc::clone(&api)).await.expect("an application");

    // The terminal of the measurement: 160 columns and 45 rows.
    let mut terminal = Terminal::new(TestBackend::new(160, 45)).expect("a terminal");

    app.is_podcast = false;
    app.search_query = "zzzznohitatall".to_string();

    // **The application of `App::new` holds no line of any of these views**: no
    // shelf, no media of the library, no series, no list, no result of a search,
    // no episode, no chapter, and no media of the queue.
    let the_views_with_no_line = [
        (AppView::Home, "the Home view"),
        (AppView::Library, "the Library view"),
        (AppView::Series, "the Series view"),
        (AppView::SeriesBook, "the books of a series"),
        (AppView::Lists, "the Collections view"),
        (AppView::ListEntries, "the media of a collection"),
        (AppView::SearchBook, "the search"),
        (AppView::PodcastEpisode, "the episodes of a podcast"),
        (AppView::Authors, "the Authors view"),
        (AppView::Chapters, "the Chapters view"),
        (AppView::Queue, "the Queue view"),
    ];

    // The parts of a footer that need a line of the list under them. A view with
    // no line must name none of them.
    let the_keys_of_a_line = [
        "j/k: move",
        "j/k: a shelf",
        "l: the media",
        "l: play",
        "l: play or open",
        "l: play it now",
        "l: take the line",
        "l: go to the chapter",
        "l: the books of this author",
        "l: the books of this narrator",
        "r/D: a name/description",
        "X: remove",
        "X: take it out",
        "h/l: a cover",
        "Enter: play or open",
    ];

    for (view, name) in the_views_with_no_line {
        app.view_state = view;

        terminal
            .draw(|frame| frame.render_widget(&mut app, frame.area()))
            .expect("the view draws");

        let screen = the_screen_of(&terminal);

        for part in the_keys_of_a_line {
            assert!(
                !screen.contains(part),
                "{name} with no line must not name the key of the part \"{part}\""
            );
        }

        // **A screen that names no key looks like a program that stopped**
        // (T-52), therefore every one of these views keeps a key of its own.
        assert!(
            screen.contains("Q: quit"),
            "{name} with no line keeps the key that stops the program"
        );
    }

    // ## 3. The controls of the same run: a view of one line keeps every key
    //
    // The Collections view of one collection of one media, which is the shape of
    // the library `Books` of the sandbox. **A correction that took the keys of a
    // line away from every view would fail this part.**
    app.view_state = AppView::Lists;
    app.lists = vec![ListView {
        id: "a-collection".to_string(),
        kind: ListKind::Collection,
        name: "A Test Collection".to_string(),
        description: String::new(),
        entries: vec![ListEntry {
            id: "a-media".to_string(),
            episode_id: None,
            title: "A Book Of A Test".to_string(),
            author: "A Writer".to_string(),
            duration: 600.0,
            description: String::new(),
        }],
    }];
    app.list_state_lists.select(Some(0));

    terminal
        .draw(|frame| frame.render_widget(&mut app, frame.area()))
        .expect("the Collections view of one line draws");

    let screen = the_screen_of(&terminal);

    for part in [
        "j/k: move",
        "l: the media",
        "r/D: a name/description",
        "X: remove",
    ] {
        assert!(
            screen.contains(part),
            "the Collections view of one line keeps the part \"{part}\" of its footer"
        );
    }

    // **The media of that collection keeps every key too**, which says that the
    // rule reads the lines of each view and not the lines of one of them.
    app.view_state = AppView::ListEntries;
    app.list_state_list_entries.select(Some(0));

    terminal
        .draw(|frame| frame.render_widget(&mut app, frame.area()))
        .expect("the media of a collection draws");

    let screen = the_screen_of(&terminal);

    assert!(
        screen.contains("j/k: move") && screen.contains("l: play"),
        "the media of a collection of one line keeps the keys of a line"
    );
}
