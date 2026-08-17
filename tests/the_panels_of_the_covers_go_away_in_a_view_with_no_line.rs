//! The gate of T-354: a view with no line takes no panel of the covers.
//!
//! The measurement of the real program v0.8.184 inside tmux, at 160 columns and
//! 45 rows, of the library `Empty` of the sandbox (`docs/TEST-SERVER.md`). That
//! library holds no media at all, therefore the Library view and the Home view
//! of it hold no line, and each of them said its reason with
//! `App::render_the_reason`.
//!
//! **The two panels of the covers stood beside that reason with no character in
//! them**: the panel 5 of the cover held 8 rows, the panel 6 of the gallery held
//! 32 rows, the two of them took **48 columns** of the screen, and the reason of
//! the view said its two lines in the 74 columns that stayed.
//!
//! ```text
//! ┌1 Views ───────────────────────┐───────────────────────────────── ┌5 Cover ────────────────────────────────────────┐
//! │➤ Home                      Tab│      This library holds no media.│                                                │
//! │  Library                   Tab│  Press L to tell the server to …│                                                │
//! │  Sequence and filter         f│                                  │                                                │
//! │  Authors                     a│                                  │                                                │
//! │  Narrators                   v│                                  │                                                │
//! │  Collections                 c│                                  │                                                │
//! │  Queue                       q│                                  └────────────────────────────────────────────────┘
//! │  Downloads                   d│                                  ┌6 Gallery ──────────────────────────────────────┐
//! │  Chapters                    C│                                  │                                                │
//! ```
//!
//! The corrected program of the same harness gave no panel 5 and no panel 6 at
//! all, and the reason of the view took the whole width of the work:
//!
//! ```text
//! ┌1 Views ────────────────────────┐──────────────────────────────────────────────────────────────────────────────────
//! │➤ Home                       Tab│                                 This library holds no media.
//! │  Library                    Tab│                      Press L to tell the server to examine the library.
//! ```
//!
//! **A media that plays keeps the two panels** (T-23): the picture of that media
//! and the facts of it say something in a view that holds no line of its own,
//! therefore the rule reads the media of the panel and not the lines alone.
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
use toutui::app::{App, AppView};
use toutui::db::database_struct::User;
use toutui::logic::home_view::HomeRow;
use toutui::logic::library_view::LibraryRow;
use toutui::ui::cover::the_panels_of_the_covers_stand;

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
async fn the_panels_of_the_covers_go_away_in_a_view_with_no_line() {
    // **The pure function first.**
    assert!(
        !the_panels_of_the_covers_stand(0, false),
        "a view of no line and of no media of the panel holds no panel of the covers"
    );
    assert!(
        the_panels_of_the_covers_stand(0, true),
        "a media that plays keeps the two panels in a view that holds no line"
    );
    assert!(
        the_panels_of_the_covers_stand(1, false),
        "a view of one line keeps the two panels"
    );

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

    // The terminal of the measurement: 160 columns and 45 rows, which is the
    // shape of the three columns of the design (T-320).
    let mut terminal = Terminal::new(TestBackend::new(160, 45)).expect("a terminal");

    app.is_podcast = false;

    // ## 1. The Library view of a library with no media
    app.view_state = AppView::Library;
    app.library_rows = Vec::new();
    app.ids_library = Vec::new();
    app.titles_library = Vec::new();

    terminal
        .draw(|frame| frame.render_widget(&mut app, frame.area()))
        .expect("the Library view draws");

    let rows = the_rows_of(&terminal);

    assert!(
        rows.iter().any(|row| row.contains("no media")),
        "the Library view of a library with no media says why it holds no line"
    );
    assert!(
        !rows.iter().any(|row| row.contains("5 Cover")),
        "the panel 5 of the cover takes no column of a Library view with no line"
    );
    assert!(
        !rows.iter().any(|row| row.contains("6 Gallery")),
        "the panel 6 of the gallery takes no column of a Library view with no line"
    );

    // ## 2. The Home view of the same library
    app.view_state = AppView::Home;
    app.home_rows = Vec::new();
    app._ids_cnt_list = Vec::new();
    app._titles_cnt_list = Vec::new();

    terminal
        .draw(|frame| frame.render_widget(&mut app, frame.area()))
        .expect("the Home view draws");

    let rows = the_rows_of(&terminal);

    assert!(
        !rows.iter().any(|row| row.contains("5 Cover")),
        "the panel 5 of the cover takes no column of a Home view with no line"
    );
    assert!(
        !rows.iter().any(|row| row.contains("6 Gallery")),
        "the panel 6 of the gallery takes no column of a Home view with no line"
    );

    // ## 3. The control of the same run: a view of one line keeps the panels
    app.view_state = AppView::Library;
    app.library_rows = vec![LibraryRow::Book { item: 0 }];
    app.ids_library = vec!["a-media".to_string()];
    app.titles_library = vec!["A Book Of A Test".to_string()];
    app.list_state_library.select(Some(0));

    terminal
        .draw(|frame| frame.render_widget(&mut app, frame.area()))
        .expect("the Library view of one line draws");

    let rows = the_rows_of(&terminal);

    assert!(
        rows.iter().any(|row| row.contains("5 Cover")),
        "a Library view of one line keeps the panel 5 of the cover"
    );
    assert!(
        rows.iter().any(|row| row.contains("6 Gallery")),
        "a Library view of one line keeps the panel 6 of the gallery"
    );

    // **The Home view of one shelf keeps them too**, which says that the rule
    // reads the lines of each of the two views and not the lines of one of them.
    app.view_state = AppView::Home;
    app.home_rows = vec![
        HomeRow::Shelf {
            label: "Continue Listening".to_string(),
        },
        HomeRow::Media { item: 0 },
    ];
    app._ids_cnt_list = vec!["a-media".to_string()];
    app._titles_cnt_list = vec!["A Book Of A Test".to_string()];
    app.list_state_cnt_list.select(Some(1));

    terminal
        .draw(|frame| frame.render_widget(&mut app, frame.area()))
        .expect("the Home view of one shelf draws");

    let rows = the_rows_of(&terminal);

    assert!(
        rows.iter().any(|row| row.contains("5 Cover")),
        "a Home view of one shelf keeps the panel 5 of the cover"
    );
    assert!(
        rows.iter().any(|row| row.contains("6 Gallery")),
        "a Home view of one shelf keeps the panel 6 of the gallery"
    );
}
