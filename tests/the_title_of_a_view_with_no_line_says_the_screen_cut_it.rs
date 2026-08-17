//! The gate of T-373: the title of a view with no line that the screen cuts
//! keeps its start, and the three points say that the screen cut it.
//!
//! The measurement of the real program v0.8.203 inside tmux against the
//! sandbox, at 40 columns (`COLUMNS_OF_THE_SCREEN=40` of
//! `docs/harness/drive.sh`, the narrowest terminal that this fork measures).
//! The book "A Book Of An Epub With No Container" of the library `Books` holds
//! no bookmark, therefore the key `V` of its line gives the bookmarks view
//! with no line, and the title of that view holds 65 characters:
//!
//! ```text
//! With No Container" [0 items]────────────
//!   "A Book Of An Epub With No Container"
//! has no bookmark. Press b while it plays.
//! ```
//!
//! The title lost its start, no mark said that the screen cut it, and no word
//! of the screen named the view. **The control of the same run**: the reason
//! under the title wrapped whole (T-361), and the search view of the same
//! terminal — the road of a view **with** lines — cut its long line with the
//! three points (T-304).
//!
//! **Why.** ratatui gives a centered title that is wider than the block a
//! smaller area (`width - (title - width) / 2` columns) and it draws the title
//! right-aligned in it, therefore the title loses its start and its end
//! together. T-304 gave the road of the lines
//! (`render_the_table_of_a_panel`) the cut of `in_one_row`, and the two roads
//! of a view with no line — `App::render_the_reason` and
//! `render_the_message` of `crate::ui::the_message_of_a_view` — kept the raw
//! title.
//!
//! The corrected program of the same harness, of the same screen:
//!
//! ```text
//! The bookmarks of "A Book Of An Epub Wi…─
//! ```
//!
//! **The parts of this test stay in one function**: the test writes
//! `XDG_CONFIG_HOME` of the process, and a second function of this binary
//! would fight it for that box (T-144 and T-157).
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
async fn the_title_of_a_view_with_no_line_says_the_screen_cut_it() {
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

    // The terminal of the measurement: 40 columns, the narrowest width that
    // this fork measures (T-301).
    let mut terminal = Terminal::new(TestBackend::new(40, 45)).expect("a terminal");

    // The bookmarks view of the book of the measurement, with no bookmark.
    // The title of this view holds the name of the media of the server
    // (T-163), therefore no width of the screen bounds it.
    app.view_state = AppView::Bookmarks;
    app.bookmarks_of = "an-item-of-the-measurement".to_string();
    app.bookmarks_of_name = "A Book Of An Epub With No Container".to_string();
    app.bookmarks_of_a_podcast = false;
    app.bookmarks_of_episode = None;

    // The answer of the server of the measurement: the media holds no
    // bookmark. The box of the bookmarks belongs to the process, and the
    // parts of this test stay in one function for that reason too.
    toutui::logic::bookmarks::keep(toutui::logic::bookmarks::State::Ready(Vec::new()));

    terminal
        .draw(|frame| frame.render_widget(&mut app, frame.area()))
        .expect("the bookmarks view with no line draws");

    let rows = the_rows_of(&terminal);
    let the_title_row = rows
        .iter()
        .find(|row| row.contains("The bookmarks of") || row.contains("[0 items]"))
        .unwrap_or_else(|| {
            panic!(
                "a row of the screen holds the title of the view.\n{}",
                rows.join("\n")
            )
        });

    // **The title keeps its start** (the rule of T-304): the start names the
    // view, therefore the end is the part that the user can spare.
    assert!(
        the_title_row.contains("The bookmarks of"),
        "the title of a view with no line keeps its start at 40 columns: {the_title_row}\n{}",
        rows.join("\n")
    );
    assert!(
        the_title_row.contains('…'),
        "the three points say that the screen cut the title: {the_title_row}"
    );

    // The control: the reason of the view stands whole in the body, on the
    // rows under the title (T-361).
    let the_words = rows.join(" ");
    let one_space = the_words
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ");
    assert!(
        one_space.contains("has no bookmark"),
        "the reason of the view stands whole in the body.\n{}",
        rows.join("\n")
    );
}
