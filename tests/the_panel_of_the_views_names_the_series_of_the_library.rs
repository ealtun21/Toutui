//! The gate of T-366: the panel 1 of the views names the Series view of the
//! library, which the program holds and which that panel named nowhere.
//!
//! The measurement of the real program v0.8.196 inside tmux, at 160 columns and
//! 45 rows, against the sandbox (`docs/TEST-SERVER.md`). The library `Books` of
//! the sandbox is a library of books, and the panel 1 of it held fourteen lines
//! and no line of the series at all:
//!
//! ```text
//! ┌1 Views ────────────────────────┐
//! │➤ Home                       Tab│
//! │  Library                    Tab│
//! │  Sequence and filter          f│
//! │  Authors                      a│
//! │  Narrators                    v│
//! │  Collections                  c│
//! ```
//!
//! **The program holds that view, and the key `s` of the same screen opened
//! it**: the header of the next frame said `Series [3 items]` and the list of
//! it held `Depthless Hunger, Book [1 book]`, `Second Series [3 books]`, and
//! `The Test Chronicles [3 books]`. A `grep` of the word `Series` in the first
//! 34 columns of the screen of the panel gave no line.
//!
//! **The mockup 1 names that line already**: the third line of the panel 1 of
//! `docs/mockups/mockup-1.txt` is `Series  s`, and the design of the panel is
//! the design of the program since T-320. The panel of the views is the road of
//! the user to the views of the program, therefore a view that no line of it
//! names has no road of that panel at all, and the user reads the key of it in
//! the list of every key alone (`src/ui/keys.rs`, the group "The views").
//!
//! **A library of podcasts holds no series** (the key `s` of such a library says
//! `A library of podcasts has no series.`), therefore the line carries the mark
//! of a library of books of T-365, and the panel of a library of podcasts
//! names it nowhere. The gate of that rule stands in
//! `tests/the_panel_of_the_views_names_no_view_of_a_book_in_a_library_of_podcasts.rs`,
//! which reads the three views of a book now.
//!
//! **A test of the words of a panel must read the cells of the screen** (the
//! trap 249): the parts below render the real Library view into a `Buffer` of
//! ratatui and they read the rows of the panel 1 alone. **The rows of that panel
//! alone, and not the first 34 columns of every row**: the panel 2 of the
//! sequence of a library of books holds the row `The series and the number`,
//! which stands in those same columns, therefore a read of the whole column of
//! the stack would find the word `Series` of a panel that is not this one.
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
use toutui::ui::frame::{the_views, ThePanel, TheWork};

/// Nothing listens on this port. See T-25.
const NO_SERVER: &str = "http://127.0.0.1:1";

/// The name of the view of the series in the panel 1, and the key that opens it.
const THE_SERIES: &str = "Series";
const THE_KEY_OF_THE_SERIES: &str = "s";

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
/// The row of the title holds `1 Views`, and the row of the foot is the first
/// row after it that starts with a corner of a border. **The rows between them
/// are the lines of the views and no line more**, therefore the word of another
/// panel of the same column reaches no row of this list (the panel 2 of a
/// library of books holds the row of the series of the sequence).
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

/// The row of the panel that names this view, if the panel holds one.
fn the_row_of(rows: &[String], name: &str) -> Option<String> {
    rows.iter().find(|row| row.contains(name)).cloned()
}

#[tokio::test(flavor = "multi_thread")]
async fn the_panel_of_the_views_names_the_series_of_the_library() {
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

    // ## 1. The panel 1 of a library of books names the view of the series.
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
        !of_the_books.is_empty(),
        "the panel 1 of the stack stands on the screen of a library of books"
    );
    assert!(
        the_row_of(&of_the_books, "Home").is_some(),
        "the panel 1 names the Home view:\n{}",
        of_the_books.join("\n")
    );

    let of_the_series = the_row_of(&of_the_books, THE_SERIES).unwrap_or_else(|| {
        panic!(
            "the panel 1 of a library of books names the view {THE_SERIES:?}: the key `s` of the \
             same screen opens that view, and a view of the program that no line of this panel \
             names has no road of the panel of the views at all.\n{}",
            of_the_books.join("\n")
        )
    });

    // **The row names the key of that view too** (T-118 and T-143): a row of a
    // name and of no key promises no road, and the key of the row is the key of
    // `src/app.rs` and not the key of the mockup.
    //
    // **The read takes the part of the row after the name**: the name `Series`
    // ends with the letter of its own key, therefore a rule of the end of the
    // row alone passes for a row that holds no key at all — a build of the fault
    // of this round measured that hole. `str::find` gives the index of a byte
    // (the trap 245), which is the index of a slice and not a column.
    let of_the_border: &str = of_the_series.trim_end_matches(['│', '║']);
    let at = of_the_border
        .find(THE_SERIES)
        .expect("the row of the series holds the name of it");
    let after = &of_the_border[at + THE_SERIES.len()..];

    assert_eq!(
        after.trim(),
        THE_KEY_OF_THE_SERIES,
        "the row {of_the_series:?} of the panel 1 names the key \
         {THE_KEY_OF_THE_SERIES:?} of the view of the series after the name of it"
    );

    // **The row stands at the third line of the panel** (`mockup-1.txt`): the
    // design gives the series the line after the Library view.
    assert_eq!(
        the_views(false)
            .iter()
            .position(|view| view.name == THE_SERIES),
        Some(2),
        "the mockup 1 gives the view of the series the third line of the panel 1"
    );

    // ## 2. The key `l` of that row opens the Series view.
    //
    // A row that names a view and that does nothing is the fault of T-79. The
    // line of the user goes to the row of the series, and the key of the panel
    // then does the work of it.
    app.the_line_of_the_views.select(Some(2));
    app.the_panel_of_the_focus = ThePanel::TheViews;
    app.handle_key(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE));

    assert_eq!(
        app.view_state,
        AppView::Series,
        "the key l of the row of the series of the panel 1 opens the Series view"
    );

    // **The work of that row is the key `s` of the program**, and no road of its
    // own: the key handler is the authority of the work of a key, therefore a
    // row that held a second road would say another word than the key.
    assert_eq!(
        the_views(false)
            .iter()
            .find(|view| view.name == THE_SERIES)
            .map(|view| view.work),
        Some(TheWork::TheKey('s')),
        "the row of the series sends the key `s` of the program"
    );

    // ## 3. The control of the same run: a library of podcasts.
    //
    // **A library of podcasts holds no series** (T-365 and T-366): the key `s`
    // of it says `A library of podcasts has no series.` and it gives no view at
    // all, therefore no row of the panel of such a library names that view.
    app.view_state = AppView::Library;
    app.the_panel_of_the_focus = ThePanel::TheViews;
    app.is_podcast = true;

    terminal
        .draw(|frame| frame.render_widget(&mut app, frame.area()))
        .expect("the Library view of a library of podcasts draws");

    let of_the_podcasts = the_rows_of_the_panel_of_the_views(&terminal);

    assert!(
        the_row_of(&of_the_podcasts, "Home").is_some(),
        "the panel 1 of a library of podcasts keeps the rows that it holds:\n{}",
        of_the_podcasts.join("\n")
    );
    assert!(
        the_row_of(&of_the_podcasts, THE_SERIES).is_none(),
        "the panel 1 of a library of podcasts names no view {THE_SERIES:?}.\n{}",
        of_the_podcasts.join("\n")
    );
}
