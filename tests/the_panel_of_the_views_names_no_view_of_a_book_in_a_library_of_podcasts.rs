//! The gate of T-365: the panel 1 of the views names the views of the library
//! that stands, and no view that the library does not have.
//!
//! The measurement of the real program v0.8.195 inside tmux, at 160 columns and
//! 45 rows, against the sandbox (`docs/TEST-SERVER.md`). The library `Podcasts`
//! of the sandbox is a library of podcasts, and the panel 1 of it named the
//! view of the authors and the view of the narrators:
//!
//! ```text
//! ╔1 Views ════════════════════════╗
//! ║  Home                       Tab║
//! ║  Library                    Tab║
//! ║  Sequence and filter          f║
//! ║➤ Authors                      a║
//! ║  Narrators                    v║
//! ```
//!
//! **A library of podcasts holds no author and no narrator.** The footer of
//! that panel said `l: open the view`, and the key `l` of the line `Authors`
//! gave no view at all: the focus went back to the panel 4, the Home view
//! stood, and the row of the message said `A library of podcasts has no
//! author.` The key `a` of the same library said the same word, and the key `v`
//! said `A library of podcasts has no narrator.`
//!
//! **The control of that same run** is the library `Books` of the sandbox: the
//! same line of the same panel, and the same key `l`, gave
//! `The authors [9 items]`. The panel therefore named a view of one library
//! that the other library does not have, and it said the same word for the two
//! of them.
//!
//! **This is the rule of T-318 for the panel 1.** The panel 2 of the sequence
//! holds it already: `the_rows_of_the_sequence(is_podcast)` names the fields of
//! the library that the user reads and no field more, and T-324 took the row of
//! the whole library out of a library of podcasts for the same reason. The
//! panel 1 held a constant of fourteen lines for every library, and the comment
//! of that constant names T-118 and T-79 itself: a panel that promises a view
//! that the program refuses is the fault of T-118, and a line of it that does
//! nothing is the fault of T-79.
//!
//! **The corrected program of the same harness** gave the library `Podcasts` a
//! panel 1 of twelve lines whose fourth line is `Collections c`, and the key
//! `l` of that line gave `Collections and playlists [1 item]` with no message
//! of a refusal at all. The key `G` of it took the line `Every key`, which is
//! the last line of twelve and not the fourteenth line of a list that the panel
//! does not draw, and a click of the row under that line moved no line. The
//! library `Books` of the same run kept its fourteen lines.
//!
//! **A test of the words of a panel must read the cells of the screen** (the
//! trap 249): the two parts below therefore render the real Library view into a
//! `Buffer` of ratatui and they read the columns of the panel 1. A test of the
//! pure function alone would pass with the render, the keys, and the map of the
//! mouse uncorrected.
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
use toutui::ui::frame::{the_views, ThePanel, THE_VIEWS};

/// Nothing listens on this port. See T-25.
const NO_SERVER: &str = "http://127.0.0.1:1";

/// The two views that a library of podcasts does not have. The program says the
/// word of each of them already: `A library of podcasts has no author.` and
/// `A library of podcasts has no narrator.` of `src/logic/authors.rs`.
const THE_VIEWS_OF_A_BOOK: [&str; 2] = ["Authors", "Narrators"];

/// The width of the stack of the panels, which is the width of the panel 1.
const OF_THE_STACK: usize = 34;

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

/// The columns of the panel 1 of the screen, from the top to the bottom.
///
/// **The panel 1 stands in the first 34 columns of the screen** (T-320), and a
/// read of the whole width would find the name of a view in the footer, in the
/// panel 5 of the cover, or in a title of the panel 4: the harness of the
/// measurement read those columns with `cut -c1-34` for that same reason.
fn the_panel_of_the_views(terminal: &Terminal<TestBackend>) -> String {
    let buffer = terminal.backend().buffer().clone();
    let width = usize::from(buffer.area.width).min(OF_THE_STACK);

    (0..buffer.area.height)
        .map(|row| {
            (0..width)
                .map(|column| buffer[(column as u16, row)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<String>>()
        .join("\n")
}

#[tokio::test(flavor = "multi_thread")]
async fn the_panel_of_the_views_names_no_view_of_a_book_in_a_library_of_podcasts() {
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

    // ## 1. The panel 1 of a library of podcasts.
    app.view_state = AppView::Library;
    app.the_panel_of_the_focus = ThePanel::TheViews;
    app.is_podcast = true;

    terminal
        .draw(|frame| frame.render_widget(&mut app, frame.area()))
        .expect("the Library view of a library of podcasts draws");

    let of_the_podcasts = the_panel_of_the_views(&terminal);

    // The panel of this screen stands: a screen with no panel 1 at all would
    // hold no name of a view, and it would pass the two rules below with
    // nothing measured.
    assert!(
        of_the_podcasts.contains("1 Views"),
        "the panel 1 of the stack stands on the screen of a library of podcasts:\n\
         {of_the_podcasts}"
    );

    for name in THE_VIEWS_OF_A_BOOK {
        assert!(
            !of_the_podcasts.contains(name),
            "the panel 1 of a library of podcasts must name no view {name:?}: a podcast has no \
             author and no narrator, and the key of that view answers with a word and no view at \
             all.\n{of_the_podcasts}"
        );
    }

    // **The words of the views that stay are not away**: a panel that lost
    // every line would pass the rule above and it would say nothing.
    for view in the_views(true) {
        assert!(
            of_the_podcasts.contains(view.name),
            "the panel 1 of a library of podcasts names the view {:?}.\n{of_the_podcasts}",
            view.name
        );
    }

    // **The map of the mouse counts the lines that the panel draws** (T-316 and
    // T-365): a count of the whole list would give the rows under the last line
    // to a view of a book.
    assert_eq!(
        app.the_areas_of_the_mouse.the_views,
        THE_VIEWS.len() - THE_VIEWS_OF_A_BOOK.len(),
        "the map of the mouse of a library of podcasts counts the lines of that panel"
    );

    // **The key `G` takes the last line of the panel and no line under it**: a
    // key that took the fourteenth line of a panel of twelve would give the
    // key `l` of it a line that the library does not hold, and that key would
    // then do nothing at all.
    app.handle_key(KeyEvent::new(KeyCode::Char('G'), KeyModifiers::NONE));

    let of_the_end = the_views(true).len() - 1;

    assert_eq!(
        app.the_line_of_the_views.selected(),
        Some(of_the_end),
        "the key G of the panel 1 of a library of podcasts takes the last line of it"
    );
    assert_eq!(
        the_views(true)[of_the_end].name,
        "Every key",
        "the last line of the panel 1 is the view of every key"
    );

    app.handle_key(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE));

    assert_eq!(
        app.view_state,
        AppView::Keys,
        "the key l of the last line of the panel 1 of a library of podcasts opens the view of it"
    );

    // ## 2. The control of the same run: a library of books.
    //
    // The two views of a book stand there, and the key of each of them opens a
    // view: a correction that took the two lines away from every library would
    // fail this part.
    app.view_state = AppView::Library;
    app.the_panel_of_the_focus = ThePanel::TheViews;
    app.is_podcast = false;

    terminal
        .draw(|frame| frame.render_widget(&mut app, frame.area()))
        .expect("the Library view of a library of books draws");

    let of_the_books = the_panel_of_the_views(&terminal);

    for name in THE_VIEWS_OF_A_BOOK {
        assert!(
            of_the_books.contains(name),
            "the panel 1 of a library of books names the view {name:?}.\n{of_the_books}"
        );
    }

    assert_eq!(
        app.the_areas_of_the_mouse.the_views,
        THE_VIEWS.len(),
        "the map of the mouse of a library of books counts every line of that panel"
    );

    app.handle_key(KeyEvent::new(KeyCode::Char('G'), KeyModifiers::NONE));

    assert_eq!(
        app.the_line_of_the_views.selected(),
        Some(THE_VIEWS.len() - 1),
        "the key G of the panel 1 of a library of books takes the last line of the whole list"
    );

    // ## 3. The one maker of those lines.
    //
    // `the_views` gives the views of the library that stands, and the render,
    // the keys, and the map of the mouse each read it. A test of this function
    // alone would pass with the three of them uncorrected (the trap 249),
    // therefore it stands after the parts above and not in the place of them.
    assert_eq!(the_views(false).len(), THE_VIEWS.len());
    assert_eq!(
        the_views(true).len(),
        THE_VIEWS.len() - THE_VIEWS_OF_A_BOOK.len()
    );

    for name in THE_VIEWS_OF_A_BOOK {
        assert!(
            THE_VIEWS
                .iter()
                .any(|view| view.name == name && view.of_a_library_of_books),
            "the view {name:?} of the panel 1 carries the mark of a library of books"
        );
        assert!(
            the_views(true).iter().all(|view| view.name != name),
            "the views of a library of podcasts hold no view {name:?}"
        );
    }

    // **The sequence of the lines does not change with the library**: the panel
    // of a library of podcasts is the panel of a library of books with two
    // lines taken out, and the user of the two libraries reads one design.
    let of_the_names: Vec<&str> = the_views(false)
        .iter()
        .filter(|view| !view.of_a_library_of_books)
        .map(|view| view.name)
        .collect();

    assert_eq!(
        of_the_names,
        the_views(true)
            .iter()
            .map(|view| view.name)
            .collect::<Vec<&str>>()
    );
}
