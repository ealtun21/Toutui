//! The gate of T-364: the footer of a panel of the stack names the panel 4 by
//! what that panel holds, and not by a list that the view does not have.
//!
//! The measurement of the real program v0.8.194 inside tmux, at 160 columns and
//! 45 rows, of the library `Empty` of the sandbox (`docs/TEST-SERVER.md`). That
//! library holds no media at all, therefore the panel 4 of the Library view and
//! of the Home view of it holds one sentence and no list:
//!
//! ```text
//! ╔4 Library [0 items] ══════════════════════════════════════════════════╗
//! ║                    This library holds no media.                      ║
//! ║         Press L to tell the server to examine the library.           ║
//! ```
//!
//! **The three panels of the stack each named a list over that panel.** The
//! keys `1`, `2`, and `3` gave the focus to the panel of the views, of the
//! sequence, and of the filter, and the footer of each of them read:
//!
//! ```text
//! panel 1: j/k: move  l: open the view  h: the list  4/Ctrl+l: the list  …
//! panel 2: j/k: move  l: this sequence  h: the list  4/Ctrl+l: the list  …
//! panel 3: j/k: move  l: this filter    h: the list  4/Ctrl+l: the list  …
//! ```
//!
//! **The control of that same run** is the library `Books` of the sandbox, of
//! 35 items: the same three footers of the same keys named `the list` there,
//! and the panel 4 of that screen holds the list of the media. The word is
//! therefore true of one library and false of the other one, and the program
//! said the same word for the two of them.
//!
//! **That is the rule of T-143 for a word and not for a key.** The keys `h` and
//! `4` do their work: they take the focus back to the panel 4. The word of them
//! names a thing that the user cannot see, and the user of a library with no
//! media reads `h: the list` over a panel that says `This library holds no
//! media.` The round of T-359 took the keys of a line out of the footer of the
//! **view** with no line, and it did not reach the footer of a **panel** of the
//! stack, because that footer keeps no part of the footer of the view at all.
//!
//! **The corrected program of the same harness** said `h: the view` and
//! `4/Ctrl+l: the view` for the three panels of the library `Empty`, and it
//! said `h: the list` for the three panels of the library `Books`.
//!
//! **A test of the words of a view of this shape must read the cells of the
//! screen** (the trap 249): the two parts below therefore render the real
//! Library view into a `Buffer` of ratatui and they read the row of the footer.
//! The third part reads the pure function, which is the one maker of those
//! words.
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
use toutui::logic::library_view::LibraryRow;
use toutui::ui::frame::ThePanel;

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

/// The three panels of the stack, in the sequence of their digit.
const THE_PANELS_OF_THE_STACK: [ThePanel; 3] = [
    ThePanel::TheViews,
    ThePanel::TheSequence,
    ThePanel::TheFilter,
];

#[tokio::test(flavor = "multi_thread")]
async fn the_footer_of_a_panel_names_no_list_of_a_view_with_no_line() {
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
    app.view_state = AppView::Library;

    // ## 1. The Library view of a library with no media.
    //
    // The three panels of the stack take the focus one after the other, and the
    // footer of each of them stands on the screen of that same frame.
    for the_panel in THE_PANELS_OF_THE_STACK {
        app.library_rows = Vec::new();
        app.ids_library = Vec::new();
        app.titles_library = Vec::new();
        app.the_panel_of_the_focus = the_panel;

        terminal
            .draw(|frame| frame.render_widget(&mut app, frame.area()))
            .expect("the Library view draws");

        let rows = the_rows_of(&terminal);
        let screen = rows.join("\n");

        // The panel 4 of this screen holds the reason and no list at all: that
        // is the condition of this measurement, and a screen that holds a list
        // measures nothing.
        assert!(
            rows.iter().any(|row| row.contains("4 Library [0 items]")),
            "the panel 4 of a Library view with no line stands on the screen:\n{screen}"
        );

        assert!(
            !screen.contains("h: the list"),
            "the footer of the panel {the_panel:?} of a view with no line must name no list: the \
             panel 4 of that screen holds one sentence and no list at all.\n{screen}"
        );

        // **The words say what the panel holds**, and they are not away: a
        // footer that lost the key `h` would say nothing of the road back.
        assert!(
            screen.contains("h: the view"),
            "the footer of the panel {the_panel:?} of a view with no line names the panel 4 of \
             it.\n{screen}"
        );
    }

    // ## 2. The control of the same run: the same library with one line in it.
    //
    // The panel 4 then holds the list of the media, and the word `the list`
    // says more than the word `the view`.
    for the_panel in THE_PANELS_OF_THE_STACK {
        app.the_panel_of_the_focus = the_panel;
        app.titles_library = vec!["A Book Of The Control".to_string()];
        app.ids_library = vec!["an-item".to_string()];
        app.library_rows = vec![LibraryRow::Book { item: 0 }];

        terminal
            .draw(|frame| frame.render_widget(&mut app, frame.area()))
            .expect("the Library view draws");

        let screen = the_rows_of(&terminal).join("\n");

        assert!(
            screen.contains("h: the list"),
            "the footer of the panel {the_panel:?} of a view that holds a line names the list of \
             it.\n{screen}"
        );
    }

    // ## 3. The one maker of those words.
    //
    // `the_footer_of_a_panel` gives the footer of every panel of the frame, and
    // the five texts of it named the panel 4. A test of the function alone
    // would pass with the render of the parts above uncorrected (the trap 249),
    // therefore it stands after them and not in their place.
    let of_the_view = toutui::ui::keys::FOOTER_OF_A_LIBRARY_OF_BOOKS;

    for the_panel in [
        ThePanel::TheViews,
        ThePanel::TheSequence,
        ThePanel::TheFilter,
        ThePanel::TheCover,
        ThePanel::TheGallery,
    ] {
        let with_a_line =
            toutui::ui::keys::the_footer_of_a_panel(of_the_view, true, true, the_panel, true);
        let with_no_line =
            toutui::ui::keys::the_footer_of_a_panel(of_the_view, true, true, the_panel, false);

        assert!(
            with_a_line.contains("the list"),
            "the footer of the panel {the_panel:?} of a view that holds a line names the list: \
             {with_a_line:?}"
        );
        assert!(
            !with_no_line.contains("the list"),
            "the footer of the panel {the_panel:?} of a view with no line must name no list: \
             {with_no_line:?}"
        );
        assert!(
            with_no_line.contains("the view"),
            "the footer of the panel {the_panel:?} of a view with no line names the panel 4 of \
             it: {with_no_line:?}"
        );

        // **The two words hold the same number of columns**, therefore the rows
        // that the wrap of this footer needs do not change with the condition of
        // the view. `render_home` reads those rows before it makes the lines of
        // the view (T-336), and a word of another width there would give the
        // panel 4 one row more for a library with no media than for the same
        // library with one book in it.
        assert_eq!(
            toutui::logic::message::the_columns_of(toutui::ui::keys::the_panel_4_of_a_view(true)),
            toutui::logic::message::the_columns_of(toutui::ui::keys::the_panel_4_of_a_view(false)),
            "the two words of the panel 4 hold the same number of columns"
        );
    }
}
