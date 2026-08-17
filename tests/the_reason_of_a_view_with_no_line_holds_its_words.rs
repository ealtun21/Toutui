//! The gate of T-361: the reason of a view with no line keeps every word of it
//! in a narrow terminal.
//!
//! **A reason of a view stands in the body of the panel and never in the title
//! of it.** The title of a block takes no wrap: `ratatui` draws it on one row
//! and it cuts what stands over the width of the panel. Nine views gave their
//! reason to the title of `render_list`, therefore the user of a narrow
//! terminal read a part of a sentence and lost the key of the work.
//!
//! The measurement of the real program v0.8.191 inside tmux, at **40 columns**
//! and 30 rows (`COLUMNS_OF_THE_SCREEN=40` of `docs/harness/drive.sh`, of the
//! shape of T-301), against the sandbox (`docs/TEST-SERVER.md`):
//!
//! ```text
//! the key V   →  "A Book Of An Epub With No Container" h…
//! the key q   →  The queue is empty. Press n on a media…─
//! the key C   →  No media plays now. A media that plays…─
//! the key v   →  This library has no narrator. A narrato…
//! the search  →  The server found nothing for "zzqqxnoth…
//! ```
//!
//! **Every one of those rows is the whole panel.** The rows under it held
//! nothing at all: the sentence stood in the title, and the body of the
//! `Paragraph` was empty. The user therefore read no key of the work — not
//! `Press b while it plays.`, not `Press n on a media`, not
//! `Press / to write other words.`
//!
//! **The control of the same run**, of the views that T-358 corrected already:
//! the Library view and the Collections view of the library `Empty` at those
//! same 40 columns each said the name of the list in the title and the whole
//! reason under it, over three rows of a wrap:
//!
//! ```text
//! ───────────Library [0 items]────────────
//!       This library holds no media.
//!   Press L to tell the server to examine
//!               the library.
//! ```
//!
//! **That is the fault that the `wrap` of T-278 corrected**, and the road of a
//! title brought it back for the nine views that this item names.
//!
//! The corrected program of the same harness and of the same road:
//!
//! ```text
//! ──────────The queue [0 items]───────────
//!  The queue is empty. Press n on a media
//!          to put it in the queue.
//!
//! ─────────The chapters [0 items]─────────
//!  No media plays now. A media that plays
//!  gives its chapters. Press h to go back.
//!
//! ────────Search result [0 items]─────────
//!       The server found nothing for
//!   "zzqqxnothingatall". Press / to write
//!               other words.
//! ```
//!
//! **The controls of the same run stayed as they were**: the authors of the
//! library `Books` kept `The authors [9 items]`, the lists that take a media
//! kept `Put "A Book Of An Epub With No Container" in a list [2 items]`, and a
//! search of one hit kept `Search result [1 item]` with the panel of the cover
//! beside it.
//!
//! **The parts of this test stay in one function**: the test writes
//! `XDG_CONFIG_HOME` of the process and the box of the bookmarks, and a second
//! function of this binary would fight it for the two of them (T-144 and
//! T-157).
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

/// Nothing listens on this port. See T-25.
const NO_SERVER: &str = "http://127.0.0.1:1";

/// The terminal of the measurement. **The narrowest terminal that this fork
/// measures** (T-301): every sentence of this test is longer than it.
const THE_NARROW_SCREEN: u16 = 40;

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

/// The whole screen in one text, with the spaces of the wrap taken out.
///
/// **A sentence of a body stands on more than one row** (T-278): the
/// `Paragraph` of the reason holds `Wrap { trim: true }` and it is centered,
/// therefore the words of one sentence stand on three rows of a screen of 40
/// columns with a run of spaces at each side of each row. This function joins
/// the rows with one space and it makes every run of the whitespace one space,
/// therefore a test of the sentence reads the sentence itself.
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
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[tokio::test(flavor = "multi_thread")]
async fn the_reason_of_a_view_with_no_line_holds_its_words() {
    // ## 1. The rule of the pure functions, with no screen at all
    //
    // **The title of a view gives the name of its list at every count**, and
    // the reason of a view with no line comes of a function of its own.
    use toutui::logic::chapters::{the_header_of_the_view, the_reason_of_no_chapter};
    use toutui::logic::search::{the_reason_of_no_hit, the_title_of_the_search};

    assert_eq!(
        the_title_of_the_search(true, &[], 0, 0),
        "Search result [0 items]",
        "the title of a search of no hit names the list and no reason at all"
    );
    assert_eq!(
        the_reason_of_no_hit("zzqqxnothingatall", true, 0),
        "The server found nothing for \"zzqqxnothingatall\". Press / to write other words.",
        "the reason of a search of no hit names the words and the key"
    );

    assert_eq!(
        the_header_of_the_view("A Long Test Book", None, 0, true),
        "The chapters [0 items]",
        "the header of the chapters of no media names the list and no reason at all"
    );
    assert_eq!(
        the_header_of_the_view("A Long Test Book", None, 0, false),
        "The chapters of \"A Long Test Book\" [0 items]",
        "the header of the chapters of a media of no chapter names that media"
    );
    assert_eq!(
        the_reason_of_no_chapter("A Long Test Book", None, false),
        "\"A Long Test Book\" holds no chapter. Press h to go back.",
        "the reason of a media of no chapter names that media (T-227)"
    );

    // **No title of this program names a key of the user**: a key stands in the
    // reason, which holds a wrap, and never in the title, which holds none.
    for title in [
        the_title_of_the_search(true, &[], 0, 0),
        the_title_of_the_search(false, &[], 0, 0),
        the_header_of_the_view("A Long Test Book", None, 0, true),
        the_header_of_the_view("A Long Test Book", None, 0, false),
    ] {
        assert!(
            !title.contains("Press "),
            "the title of a view with no line names no key of the user: {title}"
        );
    }

    // ## 2. The views of the render, at the narrowest terminal of this fork
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

    // **The application of `App::new` holds no line of these three views**: the
    // queue took no media, no playback stands, and the search asked no server.
    // Each of the three sentences is longer than the 40 columns of the screen,
    // therefore a sentence of a title loses its words and a sentence of a body
    // keeps them.
    //
    // The bookmarks take a state of their own, because the state of `App::new`
    // is `Nothing`, whose sentence fits the screen.
    toutui::logic::bookmarks::keep(toutui::logic::bookmarks::State::Ready(Vec::new()));
    app.bookmarks_of_name = "A Book Of An Epub With No Container".to_string();
    app.bookmarks_of_a_podcast = false;

    // The fourth part of a row is the start of the title of the view: the mark
    // of a cut belongs to the row of the title alone (T-373), and the start of
    // a title stands through every cut (T-304).
    let the_views_with_no_line = [
        (
            AppView::Queue,
            "the queue",
            "The queue is empty. Press n on a media to put it in the queue.",
            "The queue [",
        ),
        (
            AppView::Chapters,
            "the chapters",
            "No media plays now. A media that plays gives its chapters. Press h to go back.",
            "The chapters [",
        ),
        (
            AppView::SearchBook,
            "the search",
            "The program looks in its own titles. The answer of the server comes.",
            "Search result [",
        ),
        (
            AppView::Bookmarks,
            "the bookmarks",
            "\"A Book Of An Epub With No Container\" has no bookmark. Press b while it plays.",
            "The bookmarks of \"",
        ),
    ];

    for (view, name, the_reason, the_start_of_the_title) in the_views_with_no_line {
        app.view_state = view;

        terminal
            .draw(|frame| frame.render_widget(&mut app, frame.area()))
            .expect("the view draws");

        let words = the_words_of(&terminal);

        assert!(
            words.contains(the_reason),
            "{name} with no line keeps every word of its reason in a terminal of \
             {THE_NARROW_SCREEN} columns.\nthe reason: {the_reason}\nthe screen: {words}"
        );

        // **A body that holds a wrap never ends with the mark of a cut**: the
        // screen of the fault said `The queue is empty. Press n on a media…`
        // on the row of the border. **The title of the view can hold that mark
        // now** (T-373): a title that is wider than the screen keeps its start
        // and it says that the screen cut it. A row that holds the mark must
        // therefore hold the start of the title too, and a mark on a row of
        // the body is the fault of T-361 again.
        let buffer = terminal.backend().buffer().clone();
        for row in 0..buffer.area.height {
            let the_row: String = (0..buffer.area.width)
                .map(|column| buffer[(column, row)].symbol())
                .collect();

            assert!(
                !the_row.contains('…') || the_row.contains(the_start_of_the_title),
                "{name} with no line draws no cut of a row of its body: {the_row}"
            );
        }
    }

    // ## 3. The controls of the same run
    //
    // **A view that holds its lines keeps its list and its name**, therefore a
    // correction that gave every view the sentence of a reason would fail this
    // part.
    //
    // **The name of the media of the control is short**, because a title holds
    // no wrap: the name of the measurement above is longer than the 40 columns
    // of this screen, and the count of the lines of it therefore stands outside
    // the screen. That is the trade of T-358, and this test measures the reason
    // and not the name.
    app.bookmarks_of_name = "A Book".to_string();

    toutui::logic::bookmarks::keep(toutui::logic::bookmarks::State::Ready(vec![
        toutui::api::me::bookmarks::Bookmark {
            library_item_id: "a-media".to_string(),
            title: "A place of a test".to_string(),
            time: 42.0,
        },
    ]));

    app.view_state = AppView::Bookmarks;

    terminal
        .draw(|frame| frame.render_widget(&mut app, frame.area()))
        .expect("the bookmarks of one line draw");

    let words = the_words_of(&terminal);

    assert!(
        words.contains("[1 item]"),
        "the bookmarks of one line name the count of that line: {words}"
    );
    assert!(
        !words.contains("has no bookmark"),
        "the bookmarks of one line say no reason of a view with no line: {words}"
    );
    assert!(
        words.contains("A place of a test"),
        "the bookmarks of one line draw that line: {words}"
    );

    // **A wide terminal keeps the name of the list too**: the title of a view
    // with no line is the title of the same view of lines, with the count 0.
    let mut the_wide_screen = Terminal::new(TestBackend::new(160, 45)).expect("a terminal");

    toutui::logic::bookmarks::keep(toutui::logic::bookmarks::State::Ready(Vec::new()));
    app.view_state = AppView::Queue;

    the_wide_screen
        .draw(|frame| frame.render_widget(&mut app, frame.area()))
        .expect("the queue of a wide screen draws");

    let words = the_words_of(&the_wide_screen);

    assert!(
        words.contains("The queue [0 items]"),
        "the queue with no line names its list and the count of it: {words}"
    );
    assert!(
        words.contains("The queue is empty. Press n on a media to put it in the queue."),
        "the queue with no line says its reason under that name: {words}"
    );
}
