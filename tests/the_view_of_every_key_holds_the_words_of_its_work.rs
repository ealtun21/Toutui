//! The gate of T-362: the view of every key of the program keeps the words of
//! the work of each key in a narrow terminal.
//!
//! **A line of a list stands on one row of the panel** (T-311), and a
//! `ListItem` of ratatui holds no wrap: a line that is longer than the panel
//! therefore loses its end with no mark of a cut at all. The view of the key `?`
//! gave the work of each of its 83 keys to a column of the line, at the column
//! 19, and the name of each of its 13 groups to a line of its own.
//!
//! The measurement of the real program v0.8.192 inside tmux, at **40 columns**
//! and 30 rows (`COLUMNS_OF_THE_SCREEN=40` of `docs/harness/drive.sh`, of the
//! shape of T-301), against the sandbox (`docs/TEST-SERVER.md`). The key `?` of
//! the Home view:
//!
//! ```text
//! ────────Every key of the program────────
//! ➤ ▌ The panels (a screen of 120 columns█
//!      1               The focus goes to █
//!      2               The focus goes to █
//!      3               The focus goes to █
//!      Ctrl+h          The focus goes to │
//!      l / → / Enter   The panel 1 opens │
//!      z               Hide the panels 1,│
//!      Click           The line of the po│
//! ```
//!
//! **No key of the program said what it does.** The 37 columns of a line held
//! the two columns of the sign of the cursor, the three columns of the indent,
//! the fifteen columns of the key, and one space: **16 columns stayed for the
//! work**, and the shortest work of the 83 keys holds 17. The view of the keys
//! is the one text of this program that says what a key does, and a user of a
//! narrow terminal therefore read the keys and no word of their work.
//!
//! **The name of a group went the same road**: `The panels (a screen of 120
//! columns and more, Home and Library)` gave `The panels (a screen of 120
//! columns`, and the words that name the two views of those panels stood
//! outside the screen.
//!
//! **The correction gives the work of a key the rows that it needs.** The two
//! columns of the design stand while the work has
//! `THE_SMALLEST_COLUMN_OF_THE_WORK` columns beside the key, and the rows of the
//! wrap of the work then stand under the first row of it, at the column of the
//! work. A panel that is narrower draws the key on a row of its own and the work
//! under it, at an indent of five columns. Every row of those wraps is a **line**
//! of the list, therefore the rule of T-311 holds: the bar of the scroll counts
//! them, and the keys `j`, `k`, and `G` move over them.
//!
//! The corrected program of the same harness and of the same road:
//!
//! ```text
//! ────────Every key of the program────────
//! ➤ ▌ The panels (a screen of 120 columns█
//!     and more, Home and Library)        █
//!      1                                 │
//!        The focus goes to the panel 1 of│
//!        the views                       │
//!      2                                 │
//!        The focus goes to the panel 2 of│
//!        the sequence                    │
//! ```
//!
//! **The controls of the same run.** At 60 columns the two columns of the design
//! stand and the work wraps under itself
//! (`1               The focus goes to the panel 1 of the` / `views`), and at
//! **160 columns every row of the view stands as it stood before this item**.
//! The key `G` of each of the three widths gave the last line of the view,
//! therefore the count of the lines reads the width of the panel that the user
//! has.
//!
//! **The parts of this test stay in one function**: the test writes
//! `XDG_CONFIG_HOME` of the process, and a second function of this binary would
//! fight it for that variable (T-144 and T-157).
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
use toutui::logic::message::the_columns_of;
use toutui::ui::keys;
use toutui::ui::the_list_of_a_view::the_columns_of_a_line;

/// Nothing listens on this port. See T-25.
const NO_SERVER: &str = "http://127.0.0.1:1";

/// The terminal of the measurement. **The narrowest terminal that this fork
/// measures** (T-301).
const THE_NARROW_SCREEN: u16 = 40;

/// The columns that a line of the list of that terminal has: the width of the
/// panel, less the bar of the scroll and the sign of the cursor.
const THE_COLUMNS_OF_A_LINE: u16 = 37;

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

/// The whole screen in one text, with every run of the whitespace made one
/// space.
///
/// **The work of a key stands on more than one row of a narrow screen**,
/// therefore a test of the words of that work reads the rows together.
///
/// **The bar of the scroll stands at the last column of every row of the panel**
/// (T-255), and it is no word: the glyphs of the thumb and of the track become a
/// space, or the last word of a row and the first word of the row after it join
/// into a word that no text of this program holds.
fn the_words_of(terminal: &Terminal<TestBackend>) -> String {
    let buffer = terminal.backend().buffer().clone();

    let rows: Vec<String> = (0..buffer.area.height)
        .map(|row| {
            (0..buffer.area.width)
                .map(|column| buffer[(column, row)].symbol())
                .collect::<String>()
        })
        .collect();

    rows.join(" ")
        .replace(['█', '│'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[tokio::test(flavor = "multi_thread")]
async fn the_view_of_every_key_holds_the_words_of_its_work() {
    // ## 1. The rule of the pure functions, with no screen at all
    //
    // **No line of this view is longer than a line of its panel.** That is the
    // whole rule of the item: a line that is longer loses its end, because a
    // `ListItem` holds no wrap (T-311).
    for columns in [THE_COLUMNS_OF_A_LINE, 20, 30, 55, 100, 157] {
        for line in keys::lines_of_a_width(columns) {
            assert!(
                the_columns_of(&line) <= usize::from(columns),
                "no line of the view of the keys is longer than the {columns} columns of \
                 its panel: {line:?} holds {} columns",
                the_columns_of(&line)
            );
        }
    }

    // **The work of a key has the columns of a text of the user, or the key
    // stands on a row of its own.** A wrap keeps every word at every width,
    // therefore the words alone do not say which of the two forms the view
    // draws: a panel of 37 columns that keeps the two columns of the design
    // gives the work 18 columns, and the work of one key then stands on four
    // rows of two words each.
    for columns in keys::THE_INDENT_OF_THE_WORK + keys::THE_SMALLEST_COLUMN_OF_THE_WORK..=160 {
        assert!(
            keys::the_columns_of_the_work(columns) >= keys::THE_SMALLEST_COLUMN_OF_THE_WORK,
            "a panel of {columns} columns gives the work of a key \
             {} columns, and it needs {}",
            keys::the_columns_of_the_work(columns),
            keys::THE_SMALLEST_COLUMN_OF_THE_WORK
        );
    }

    // **The two columns of the design stand at every width that holds them**,
    // therefore a correction that gave every panel the rows of the work would
    // fail here.
    assert!(
        keys::the_two_columns_stand(160) && keys::the_two_columns_stand(0),
        "a wide panel keeps the two columns of the design"
    );
    assert!(
        !keys::the_two_columns_stand(THE_COLUMNS_OF_A_LINE),
        "a panel of {THE_COLUMNS_OF_A_LINE} columns draws the key and the work under it"
    );

    // **Every word of the work of every key stands in those lines**, and the
    // words of the name of every group stand there too. A wrap makes the rows of
    // a text, therefore the words of one work stand over more than one line: the
    // lines join with one space, in the same way as the rows of the screen do.
    let the_lines = keys::lines_of_a_width(THE_COLUMNS_OF_A_LINE);
    let the_words = the_lines
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    for group in keys::GROUPS {
        assert!(
            the_words.contains(group.name),
            "the view of the keys of {THE_COLUMNS_OF_A_LINE} columns keeps every word of the \
             name of a group: {:?} is not in the view",
            group.name
        );

        for one in group.keys {
            assert!(
                the_words.contains(one.what),
                "the view of the keys of {THE_COLUMNS_OF_A_LINE} columns keeps every word of \
                 the work of the key {:?}: {:?} is not in the view",
                one.key,
                one.what
            );
        }
    }

    // **A width of 0 is a caller with no width at all**, and the lines of it hold
    // the two columns of the design with no wrap.
    let of_no_width = keys::lines_of_a_width(0);

    assert_eq!(
        of_no_width,
        keys::lines(),
        "the lines of no width are the lines of the caller with no width"
    );
    assert!(
        of_no_width
            .iter()
            .any(|line| line
                == "   z               Hide the panels 1, 2, and 3, and show them again"),
        "the lines of no width hold the work of a key beside that key: {of_no_width:?}"
    );

    // **The columns of a line come of the panel** (T-362): the block of a list
    // holds one border at the top and none at the sides, therefore the bar of
    // the scroll and the sign of the cursor are the two columns that the lines
    // do not have.
    assert_eq!(
        the_columns_of_a_line(ratatui::layout::Rect::new(0, 3, THE_NARROW_SCREEN, 26)),
        THE_COLUMNS_OF_A_LINE,
        "a panel of {THE_NARROW_SCREEN} columns gives a line {THE_COLUMNS_OF_A_LINE} of them"
    );

    // ## 2. The view of the render, at the narrowest terminal of this fork
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

    let mut terminal = Terminal::new(TestBackend::new(THE_NARROW_SCREEN, 30)).expect("a terminal");

    app.view_state = AppView::Keys;

    terminal
        .draw(|frame| frame.render_widget(&mut app, frame.area()))
        .expect("the view of the keys draws");

    let words = the_words_of(&terminal);

    // **The first rows of the panel hold the first keys of the first group**,
    // and the work of each of them stands there whole. The screen of the fault
    // held `The focus goes to` and no panel at all.
    for the_work in [
        "The panels (a screen of 120 columns and more, Home and Library)",
        "The focus goes to the panel 1 of the views",
        "The focus goes to the panel 2 of the sequence",
    ] {
        assert!(
            words.contains(the_work),
            "the view of the keys of a terminal of {THE_NARROW_SCREEN} columns says the work \
             of a key whole.\nthe work: {the_work}\nthe screen: {words}"
        );
    }

    // **The render writes the columns of its lines**, and the keys of the move
    // read them: a count of the lines of another width gives a cursor that the
    // user cannot reach or a cursor that goes past the last line.
    assert_eq!(
        app.the_columns_of_the_lines_of_the_keys, THE_COLUMNS_OF_A_LINE,
        "the render of the view of the keys writes the columns of its lines"
    );

    app.select_last();

    assert_eq!(
        app.list_state_keys.selected(),
        Some(app.the_lines_of_the_view_of_the_keys().len() - 1),
        "the key G of the view of the keys goes to the last line of the width of the panel"
    );

    terminal
        .draw(|frame| frame.render_widget(&mut app, frame.area()))
        .expect("the last line of the view of the keys draws");

    let words = the_words_of(&terminal);
    let the_last = keys::GROUPS
        .last()
        .and_then(|group| group.keys.last())
        .expect("a last key of the view");

    assert!(
        words.contains(the_last.what),
        "the last line of the view of the keys says the work of the last key whole.\n\
         the work: {}\nthe screen: {words}",
        the_last.what
    );

    // ## 3. The control of the same run, at a wide terminal
    //
    // **A wide terminal keeps the two columns of the design**, therefore a
    // correction that gave every width the rows of the work would fail here.
    let mut the_wide_screen = Terminal::new(TestBackend::new(160, 45)).expect("a terminal");

    app.list_state_keys.select(Some(0));

    the_wide_screen
        .draw(|frame| frame.render_widget(&mut app, frame.area()))
        .expect("the view of the keys of a wide screen draws");

    let buffer = the_wide_screen.backend().buffer().clone();
    let rows: Vec<String> = (0..buffer.area.height)
        .map(|row| {
            (0..buffer.area.width)
                .map(|column| buffer[(column, row)].symbol())
                .collect::<String>()
        })
        .collect();

    assert!(
        rows.iter()
            .any(|row| row
                .contains("z               Hide the panels 1, 2, and 3, and show them again")),
        "a terminal of 160 columns draws the work of a key beside that key: {rows:?}"
    );
    assert!(
        rows.iter()
            .any(|row| row
                .contains("▌ The panels (a screen of 120 columns and more, Home and Library)")),
        "a terminal of 160 columns draws the name of a group on one row: {rows:?}"
    );
}
