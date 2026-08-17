//! The gate of T-367: the panel 1 of the views names no view of a podcast in a
//! library of books, which is the other half of the rule of T-365.
//!
//! The measurement of the real program v0.8.197 inside tmux, at 160 columns and
//! 45 rows, against the sandbox (`docs/TEST-SERVER.md`). The library `Books` of
//! the sandbox is a library of books, and the panel 1 of it named the view of
//! the downloads of the server:
//!
//! ```text
//! ║  Queue                        q║
//! ║➤ Downloads                    d║
//! ║  Chapters                     C║
//! ```
//!
//! **The server downloads the episodes of a podcast alone.** The key `l` of that
//! line gave no view at all: the focus went back to the panel 4, the Home view
//! stood, the row of the message said `This library holds books. The server
//! downloads the episodes of a podcast only.`, and the log of the program grew by
//! no line (25 lines before the key, and 25 after it). The key `d` of the same
//! screen said that same word.
//!
//! **The control of that same run** is the library `Podcasts` of the sandbox: the
//! key `d` there gave `The downloads of the server [0 items]` with the reason
//! `The server downloads no episode. Press E on a podcast to get its new
//! episodes.`, and the panel 1 of that library named the same view at its row 9.
//! **The key `q` of the library `Books` is the control of the panel**: it gave
//! `The queue [0 items]` at once, therefore the keys of that view do their work
//! and the line of the downloads alone refuses.
//!
//! **This is the rule of T-365 in the other direction.** T-365 took the views of
//! a book out of a library of podcasts, and T-366 gave the panel the line of the
//! series with that same mark. A mark of one kind of a library cannot say that a
//! view belongs to the other kind, therefore `AView` holds
//! `TheLibraryOfAView` now: `Every`, `OfBooks`, or `OfPodcasts`. The
//! filter of `the_views` reads the three of them, and no library names the views
//! of the other kind.
//!
//! **A test of the words of a panel must read the cells of the screen** (the trap
//! 249): the parts below render the real Library view into a `Buffer` of ratatui
//! and they read the rows of the panel 1 alone, from the row of its title to the
//! row of its foot. A test of the pure function alone would pass with the render,
//! the keys, and the map of the mouse uncorrected.
//!
//! **The parts of this test stay in one function**: the test writes
//! `XDG_CONFIG_HOME` of the process, and a second function of this binary would
//! fight it for that box (T-144 and T-157).
//!
//! **This test needs no sandbox and no server.** `App::new` takes a port that
//! nothing listens on, therefore it gives the offline mode (T-25).

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::app::{App, AppView};
use toutui::db::database_struct::User;
use toutui::ui::frame::{the_views, TheLibraryOfAView, ThePanel, THE_VIEWS};

/// Nothing listens on this port. See T-25.
const NO_SERVER: &str = "http://127.0.0.1:1";

/// The view that a library of books does not have, and the key of it.
const THE_DOWNLOADS: &str = "Downloads";
const THE_KEY_OF_THE_DOWNLOADS: char = 'd';

/// The word of the program for the key of that view in a library of books.
const THE_WORD_OF_THE_REFUSAL: &str = "This library holds books.";

/// The width of the stack of the panels, which is the width of the panel 1.
const OF_THE_STACK: usize = 34;

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

/// The rows of the panel 1 of the screen, from its title to its foot.
///
/// **The rows of that panel alone, and not the first 34 columns of every row**:
/// the panel 2 and the panel 3 of the stack stand in those same columns under it,
/// and the name of a view can stand in the footer of the screen too.
fn the_rows_of_the_panel_of_the_views(terminal: &Terminal<TestBackend>) -> Vec<String> {
    let buffer = terminal.backend().buffer().clone();
    let width = usize::from(buffer.area.width).min(OF_THE_STACK);

    let of_the_stack: Vec<String> = (0..buffer.area.height)
        .map(|row| {
            (0..width)
                .map(|column| buffer[(column as u16, row)].symbol())
                .collect::<String>()
        })
        .collect();

    let Some(of_the_title) = of_the_stack.iter().position(|row| row.contains("1 Views")) else {
        return Vec::new();
    };

    of_the_stack
        .into_iter()
        .skip(of_the_title + 1)
        .take_while(|row| !row.starts_with('└') && !row.starts_with('╚'))
        .collect()
}

#[tokio::test(flavor = "multi_thread")]
async fn the_panel_of_the_views_names_no_view_of_a_podcast_in_a_library_of_books() {
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

    // ## 1. The panel 1 of a library of books names no download of the server.
    app.view_state = AppView::Library;
    app.the_panel_of_the_focus = ThePanel::TheViews;
    app.is_podcast = false;

    terminal
        .draw(|frame| frame.render_widget(&mut app, frame.area()))
        .expect("the Library view of a library of books draws");

    let of_the_books = the_rows_of_the_panel_of_the_views(&terminal);

    // The panel of this screen stands: a screen with no panel 1 at all would
    // hold no row, and it would pass every rule below with nothing measured.
    assert!(
        of_the_books.iter().any(|row| row.contains("Queue")),
        "the panel 1 of the stack stands on the screen of a library of books:\n{}",
        of_the_books.join("\n")
    );

    assert!(
        of_the_books.iter().all(|row| !row.contains(THE_DOWNLOADS)),
        "the panel 1 of a library of books must name no view {THE_DOWNLOADS:?}: the server \
         downloads the episodes of a podcast alone, and the key of that view answers with a word \
         and no view at all.\n{}",
        of_the_books.join("\n")
    );

    // **The key of that view says why it does nothing** (T-79 and T-83): the
    // panel names the view nowhere now, and the key of the user stays.
    toutui::logic::message::forget();
    app.handle_key(KeyEvent::new(
        KeyCode::Char(THE_KEY_OF_THE_DOWNLOADS),
        KeyModifiers::NONE,
    ));

    let of_the_key = toutui::logic::message::for_the_screen(app.view_state).unwrap_or_default();

    assert_ne!(
        app.view_state,
        AppView::Downloads,
        "the key of the downloads of the server opens no view in a library of books"
    );
    assert!(
        of_the_key.contains(THE_WORD_OF_THE_REFUSAL),
        "the key of the downloads of the server says why it does nothing: {of_the_key:?}"
    );

    // ## 2. The control of the same run: a library of podcasts.
    //
    // The view of the downloads stands there, and the key of it opens that view:
    // a correction that took the line away from every library would fail this
    // part.
    app.view_state = AppView::Library;
    app.the_panel_of_the_focus = ThePanel::TheViews;
    app.is_podcast = true;

    terminal
        .draw(|frame| frame.render_widget(&mut app, frame.area()))
        .expect("the Library view of a library of podcasts draws");

    let of_the_podcasts = the_rows_of_the_panel_of_the_views(&terminal);

    assert!(
        of_the_podcasts
            .iter()
            .any(|row| row.contains(THE_DOWNLOADS)),
        "the panel 1 of a library of podcasts names the view {THE_DOWNLOADS:?}.\n{}",
        of_the_podcasts.join("\n")
    );

    // **The map of the mouse counts the lines that the panel draws** (T-316,
    // T-365, and T-367): a count of the whole list would give the rows under the
    // last line to a view.
    assert_eq!(
        app.the_areas_of_the_mouse.the_views,
        the_views(true).len(),
        "the map of the mouse of a library of podcasts counts the lines of that panel"
    );

    // **The key `l` of the line of the downloads of that library opens the
    // view**: the line of the user goes to that row, and the key of the panel
    // then does the work of it.
    let at = the_views(true)
        .iter()
        .position(|view| view.name == THE_DOWNLOADS)
        .expect("the views of a library of podcasts hold the downloads of the server");

    app.the_line_of_the_views.select(Some(at));
    app.the_panel_of_the_focus = ThePanel::TheViews;
    app.handle_key(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE));

    assert_eq!(
        app.view_state,
        AppView::Downloads,
        "the key l of the row of the downloads of the panel 1 of a library of podcasts opens it"
    );

    // ## 3. The one maker of those lines.
    //
    // `the_views` gives the views of the library that stands, and the render, the
    // keys, and the map of the mouse each read it. A test of this function alone
    // would pass with the three of them uncorrected (the trap 249), therefore it
    // stands after the parts above and not in the place of them.
    assert!(
        THE_VIEWS
            .iter()
            .any(|view| view.name == THE_DOWNLOADS
                && view.the_library == TheLibraryOfAView::OfPodcasts),
        "the view of the downloads of the server carries the mark of a library of podcasts"
    );

    // **No view of the constant belongs to no library at all**: every line stands
    // in the panel of a library of books, in the panel of a library of podcasts,
    // or in the two of them.
    for view in THE_VIEWS {
        let of_the_books = the_views(false).iter().any(|one| one.name == view.name);
        let of_the_podcasts = the_views(true).iter().any(|one| one.name == view.name);

        assert!(
            of_the_books || of_the_podcasts,
            "the view {:?} of the panel 1 reaches the screen of a library",
            view.name
        );
    }
}
