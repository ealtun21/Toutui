//! The gate of T-356: the panel 4 of a view with no line takes the clicks of
//! the panel that the frame drew.
//!
//! The measurement of the real program v0.8.186 inside tmux, at 160 columns and
//! 45 rows, of the library `Empty` of the sandbox (`docs/TEST-SERVER.md`). That
//! library holds no media at all, therefore the Library view and the Home view
//! of it hold no line, and each of them says its reason with
//! `App::render_the_reason`. The correction of T-355 gives that sentence the
//! border, the number, and the name of the panel 4 already.
//!
//! **No click of that panel did anything at all.** The key `1` gave the focus
//! to the panel of the views, and a click of the column 45 and a click of the
//! column 130 of the row 10 — the two of them inside the panel 4 that the frame
//! drew — each left the focus where it stood:
//!
//! ```text
//! click column 45 row 10 -> the panel 1 border is '╔1'   (the focus stayed)
//! click column 130 row 10 -> the panel 1 border is '╔1'  (the focus stayed)
//! ```
//!
//! **The control of that same run**: a click of the column 10 of the row 6 gave
//! the focus to the panel 1 and it moved the cursor to `Sequence and filter`,
//! therefore the mouse reached the program in that view, and the panel 4 alone
//! was dead to it.
//!
//! **Why.** `the_areas_of_the_list_of_the_mouse` stood inside
//! `render_the_list_of_the_panel_4` and inside the bands of the Home view
//! alone. The empty road of `render_library`, of `render_home`, and of
//! `render_series` calls `App::render_the_reason` and it comes back, therefore
//! it wrote no area of the panel 4 at all, and the areas of that panel stayed
//! the areas of the frame before it. **The key of the next library makes a new
//! application** (`must_refresh` of `src/main.rs`), and the areas of a new
//! application are nothing at all: `the_panel_of_the_list` then held
//! `Rect::default()`, which holds no cell of the screen, and
//! `the_target_of_a_point` gave `TheTarget::Nothing` for every point of the
//! panel that the user could see.
//!
//! **The corrected program of the same harness**, of the same screen: the two
//! clicks each gave the panel 4 the focus, and the Home view of the same
//! library did the same.
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
use toutui::ui::the_mouse::{the_target_of_a_point, TheTarget};

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
async fn the_areas_of_the_mouse_of_a_view_with_no_line() {
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

    // ## 1. The Library view of a library with no media.
    //
    // **The areas of this application are the areas of a new application**,
    // which is the condition of the key of the next library and of the key `R`
    // (`must_refresh` of `src/main.rs`): no frame before this one wrote the
    // panel of the list.
    app.view_state = AppView::Library;
    app.library_rows = Vec::new();
    app.ids_library = Vec::new();
    app.titles_library = Vec::new();
    app.the_panel_of_the_focus = ThePanel::TheViews;

    terminal
        .draw(|frame| frame.render_widget(&mut app, frame.area()))
        .expect("the Library view draws");

    let rows = the_rows_of(&terminal);

    // The frame drew that panel: T-355 gives it its border, its number, and its
    // name, and this test measures the clicks of the panel that the user sees.
    let the_row_of_the_panel = rows
        .iter()
        .position(|row| row.contains("4 Library [0 items]"))
        .expect("the panel 4 of a Library view with no line stands on the screen");

    // The two columns of the measurement of the real program. The panel 4 of
    // this view takes the width of the work, because a view with no line holds
    // no panel of the cover and no panel of the gallery (T-354).
    for column in [45u16, 130] {
        let target = the_target_of_a_point(&app.the_areas_of_the_mouse, true, column, 10);

        assert_eq!(
            target,
            TheTarget::TheListOfTheView { the_line: None },
            "a click of the column {} of the panel 4 of a Library view with no line names that \
             panel, and it reads no row of a list that the view does not hold.\n{}",
            column,
            rows.join("\n")
        );
    }

    // A click of the row of the title of that panel names it too: the border
    // and the title are parts of the panel that the user can see.
    assert_eq!(
        the_target_of_a_point(
            &app.the_areas_of_the_mouse,
            true,
            80,
            the_row_of_the_panel as u16
        ),
        TheTarget::TheListOfTheView { the_line: None },
        "a click of the border of the panel 4 names that panel."
    );

    // **The number of the lines of those areas is 0**, therefore no wheel and
    // no click of that panel reads a row of the frame before it.
    assert_eq!(
        app.the_areas_of_the_mouse.the_lines, 0,
        "the areas of a view with no line hold no line at all."
    );

    // ## 2. The control of the same run: the panel 1 keeps its clicks.
    //
    // A correction that gave the whole screen to the panel 4 would fail this.
    assert!(
        matches!(
            the_target_of_a_point(&app.the_areas_of_the_mouse, true, 10, 6),
            TheTarget::ThePanelOfTheViews { the_line: Some(_) }
        ),
        "the panel 1 of the same view keeps the clicks of its lines.\n{}",
        rows.join("\n")
    );

    // ## 3. The Home view of the same library.
    app.view_state = AppView::Home;
    app.home_rows = Vec::new();
    app._ids_cnt_list = Vec::new();
    app._titles_cnt_list = Vec::new();

    terminal
        .draw(|frame| frame.render_widget(&mut app, frame.area()))
        .expect("the Home view draws");

    let rows = the_rows_of(&terminal);

    assert!(
        rows.iter().any(|row| row.contains("4 Home [0 items]")),
        "the panel 4 of a Home view with no line stands on the screen.\n{}",
        rows.join("\n")
    );
    assert_eq!(
        the_target_of_a_point(&app.the_areas_of_the_mouse, true, 100, 12),
        TheTarget::TheListOfTheView { the_line: None },
        "a click of the panel 4 of a Home view with no line names that panel.\n{}",
        rows.join("\n")
    );
}
