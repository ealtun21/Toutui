//! The gate of T-357: the six views outside the frame of the panels take no
//! column of the covers while they hold no line.
//!
//! The measurement of the real program v0.8.187 inside tmux, at 160 columns and
//! 45 rows, of the library `Empty` of the sandbox (`docs/TEST-SERVER.md`). That
//! library holds no media, no collection, and no playlist.
//!
//! **The correction of T-354 reached the Home view and the Library view alone.**
//! The six views outside the frame of the panels — the Series view, the books of
//! a series, the Collections view, the media of a collection, the search, and
//! the episodes of a podcast — each cut the area with `cover::split_for_covers`
//! with no guard at all. The Collections view of that library gave the panel 5
//! of the cover **64 columns** of a screen of 160 and **40 rows**, and no cell of
//! it held one character:
//!
//! ```text
//! ─────────────────────────────────────────────────────────── ┌5 Cover ──────────────────────────────────────────────────────┐
//!            This library has no collection and no playlist.  │                                                              │
//!                          Press h to go back.                │                                                              │
//!                                                             │                     … 36 rows of nothing …                   │
//!                                                             └──────────────────────────────────────────────────────────────┘
//! ```
//!
//! The search view of the same run, for the words `zzzznohitatall`, gave the
//! same 64 columns of nothing beside `The server found nothing for
//! "zzzznohitatall". Press / to write other words.`
//!
//! The corrected program of the same harness and of the same run gave no panel 5
//! at all in the two views, and the sentence of each took the whole 160 columns.
//! **The control of the same run** is the Collections view of the library
//! `Books`, of two lines: it kept the panel 5 of 64 columns, and the picture of
//! the collection of the cursor stood in it.
//!
//! **A media that plays keeps the column** (T-23), therefore the guard reads the
//! media of the panel beside the lines of the view
//! (`cover::the_panels_of_the_covers_stand`).
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

#[tokio::test(flavor = "multi_thread")]
async fn the_column_of_the_covers_of_a_view_with_no_line() {
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

    // ## 1. The six views with no line
    //
    // **A view of no line and of no media of the panel takes no column.** The
    // application of `App::new` holds no series, no list, no result of a search,
    // and no episode, and it plays nothing.
    let the_six = [
        (AppView::Series, "the Series view"),
        (AppView::SeriesBook, "the books of a series"),
        (AppView::Lists, "the Collections view"),
        (AppView::ListEntries, "the media of a collection"),
        (AppView::SearchBook, "the search"),
        (AppView::PodcastEpisode, "the episodes of a podcast"),
    ];

    for (view, name) in the_six {
        app.view_state = view;

        terminal
            .draw(|frame| frame.render_widget(&mut app, frame.area()))
            .expect("the view draws");

        let rows = the_rows_of(&terminal);

        assert!(
            !rows.iter().any(|row| row.contains("5 Cover")),
            "{name} with no line takes no column of the panel 5 of the cover"
        );
    }

    // ## 2. The control of the same run: a view of one line keeps the column
    //
    // The Collections view of one collection of one media, which is the shape of
    // the library `Books` of the sandbox.
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

    let rows = the_rows_of(&terminal);

    assert!(
        rows.iter().any(|row| row.contains("5 Cover")),
        "the Collections view of one line keeps the column of the panel 5 of the cover"
    );

    // **The media of that collection keeps it too**, which says that the rule
    // reads the lines of each of the six views and not the lines of one of them.
    app.view_state = AppView::ListEntries;
    app.list_state_list_entries.select(Some(0));

    terminal
        .draw(|frame| frame.render_widget(&mut app, frame.area()))
        .expect("the media of a collection draws");

    let rows = the_rows_of(&terminal);

    assert!(
        rows.iter().any(|row| row.contains("5 Cover")),
        "the media of a collection of one line keeps the column of the panel 5"
    );

    // ## 3. The two roads of the list of the episodes
    //
    // `App::the_lines_of_the_episodes_of_this_view` reads the list of the road
    // of a search and the list of the road of the library, and a view that reads
    // one of the two alone would take the column away on the other road.
    app.is_from_search_pod = false;
    app.titles_pod_ep = vec!["Chapter 01".to_string()];
    app.titles_pod_ep_search = Vec::new();
    assert_eq!(
        app.the_lines_of_the_episodes_of_this_view(),
        1,
        "the road of the library reads the episodes of the library"
    );

    app.is_from_search_pod = true;
    assert_eq!(
        app.the_lines_of_the_episodes_of_this_view(),
        0,
        "the road of a search reads the episodes of that search"
    );
}
