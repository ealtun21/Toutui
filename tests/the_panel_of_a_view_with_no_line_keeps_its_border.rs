//! The gate of T-355: the panel 4 of a view with no line keeps its border, its
//! number, and its name.
//!
//! The measurement of the real program v0.8.185 inside tmux, at 160 columns and
//! 45 rows, of the library `Empty` of the sandbox (`docs/TEST-SERVER.md`). That
//! library holds no media at all, therefore the Library view and the Home view
//! of it hold no line, and each of them said its reason with
//! `App::render_the_reason`. **That sentence stood under a bare line of
//! `Borders::TOP`**: the panels 1, 2, and 3 of the stack stood beside it with
//! their borders, their numbers, and their names, and the panel 4 had none of
//! the three. The key `4` gave that panel the focus, and the screen said
//! nothing at all:
//!
//! ```text
//! ┌1 Views ────────────────────────┐──────────────────────────────────────────────────────────
//! │➤ Home                       Tab│                    This library holds no media.
//! │  Library                    Tab│         Press L to tell the server to examine the library.
//! ```
//!
//! The corrected program of the same harness, of the same screen:
//!
//! ```text
//! ┌1 Views ────────────────────────┐╔4 Library [0 items] ═════════════════════════════════════╗
//! │➤ Home                       Tab│║                  This library holds no media.            ║
//! │  Library                    Tab│║       Press L to tell the server to examine the library. ║
//! ```
//!
//! **A user who cannot read the number of a panel cannot press the digit of
//! it** (T-320), and the title of the panel says how many lines the view holds.
//!
//! **The control of the same run is a terminal of 100 columns**: that screen
//! holds no frame of the panels (T-320), therefore the sentence keeps the block
//! of one border at the top that it had, and a correction that gives every
//! screen a panel 4 fails this test.
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

#[tokio::test(flavor = "multi_thread")]
async fn the_panel_of_a_view_with_no_line_keeps_its_border() {
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

    // ## 1. The Library view of a library with no media, with the focus on the
    // panel 4
    app.view_state = AppView::Library;
    app.library_rows = Vec::new();
    app.ids_library = Vec::new();
    app.titles_library = Vec::new();
    app.the_panel_of_the_focus = ThePanel::TheList;

    terminal
        .draw(|frame| frame.render_widget(&mut app, frame.area()))
        .expect("the Library view draws");

    let rows = the_rows_of(&terminal);

    assert!(
        rows.iter().any(|row| row.contains("no media")),
        "the Library view of a library with no media says why it holds no line"
    );
    assert!(
        rows.iter().any(|row| row.contains("4 Library [0 items]")),
        "the panel 4 of a Library view with no line says its number and its name.\n{}",
        rows.join("\n")
    );
    assert!(
        rows.iter().any(|row| row.contains("╔4 Library")),
        "the panel 4 that holds the focus takes the heavy border of the focus.\n{}",
        rows.join("\n")
    );

    // ## 2. The same view with the focus of another panel
    app.the_panel_of_the_focus = ThePanel::TheViews;

    terminal
        .draw(|frame| frame.render_widget(&mut app, frame.area()))
        .expect("the Library view of another focus draws");

    let rows = the_rows_of(&terminal);

    assert!(
        rows.iter().any(|row| row.contains("┌4 Library")),
        "the panel 4 that holds no focus takes the light border.\n{}",
        rows.join("\n")
    );

    // ## 3. The Home view of the same library
    app.view_state = AppView::Home;
    app.home_rows = Vec::new();
    app._ids_cnt_list = Vec::new();
    app._titles_cnt_list = Vec::new();
    app.the_panel_of_the_focus = ThePanel::TheList;

    terminal
        .draw(|frame| frame.render_widget(&mut app, frame.area()))
        .expect("the Home view draws");

    let rows = the_rows_of(&terminal);

    assert!(
        rows.iter().any(|row| row.contains("╔4 Home [0 items]")),
        "the panel 4 of a Home view with no line says its number and its name.\n{}",
        rows.join("\n")
    );

    // ## 4. The control of the same run: a terminal that holds no frame
    let mut narrow = Terminal::new(TestBackend::new(100, 45)).expect("a terminal of 100 columns");

    narrow
        .draw(|frame| frame.render_widget(&mut app, frame.area()))
        .expect("the Home view of 100 columns draws");

    let rows = the_rows_of(&narrow);

    assert!(
        rows.iter().any(|row| row.contains("no shelf")
            || row.contains("no media")
            || row.contains("did not")),
        "the view of 100 columns says why it holds no line.\n{}",
        rows.join("\n")
    );
    assert!(
        !rows.iter().any(|row| row.contains("4 Home")),
        "a terminal that holds no frame of the panels takes no panel 4 at all.\n{}",
        rows.join("\n")
    );
}
