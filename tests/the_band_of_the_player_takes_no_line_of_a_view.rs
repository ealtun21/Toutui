//! The gate of T-343: the band of the player stands under the work of a view,
//! and it draws over no line of it.
//!
//! **The band of the player belongs to the layout of the view** (T-322): the
//! band and the view read one number of every row, therefore the two of them
//! cannot disagree. Fifteen views of this program built their layout
//! themselves, with the header, the work of the view, and the footer, and they
//! gave the band no row at all: `render_the_band_of_the_player` then drew its
//! six rows over the last six lines of the work of those views.
//!
//! The measurement of the real program v0.8.173 inside tmux, of 160 columns and
//! 45 rows, with "A Second Book Of Many Hours" of the sandbox in a playback.
//! **The reader of an ebook lost six lines of the book**, and the line after the
//! band went on with the text of the page:
//!
//! ```text
//! suddenly, thump! thump! down she came upon a heap of sticks and dry leaves, and the fall was over.
//! ┌ Player ────────────────────────────────────────────────────────────────┐
//! │ ▶ A Second Book Of Many Hours  Many Hours Author  Chapter 23 of 70 …    │
//! │ 2:31:47 ├████████████████▒░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░┤ 8:00:00 │
//! │ Book    ███████░░░░░░░░░  31%  Chapter ███████████░░░░░░░░░░░░░  46%   │
//! │            Spc: pause/play | p/u: +/−10s | … | Y: quit                  │
//! └────────────────────────────────────────────────────────────────────────┘
//! herself in a long, low hall, which was lit up by a row of lamps hanging from the roof.
//! ```
//!
//! The user of that screen reads a paragraph, then six lines that the band
//! holds, and then the text again. The same road took six lines of the Chapters
//! view, of the view of every key, of the queue, of the bookmarks, of the
//! statistics, of the sessions, and of the eight other views of a layout of
//! their own.
//!
//! The corrected program of the same harness: the text of the book ends above
//! the band, and the footer of the reader stands under it.
//!
//! **This test needs no sandbox and no server**: `App::new` takes a port that
//! no program holds (T-25), and the state of the player comes of
//! `PlayerHandle::without_engine`, therefore no line of it opens the sound
//! device.
//!
//! **The parts of this test stay in one function**: the test writes
//! `XDG_CONFIG_HOME` of the process, and a second test function of this file
//! would fight for that place (the shape of T-144 and of T-157).

use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::app::{App, AppView};
use toutui::db::database_struct::User;
use toutui::player::engine::track::Chapter;
use toutui::player::engine::{PlaybackStatus, PlayerHandle};

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

/// The rows of the screen, as the user reads them.
fn the_rows_of(app: &mut App, rows: u16) -> Vec<String> {
    let mut terminal = Terminal::new(TestBackend::new(160, rows)).expect("a terminal");

    terminal
        .draw(|frame| frame.render_widget(&mut *app, frame.area()))
        .expect("the view draws");

    let buffer = terminal.backend().buffer().clone();

    (0..buffer.area.height)
        .map(|row| {
            (0..buffer.area.width)
                .map(|column| buffer[(column, row)].symbol())
                .collect()
        })
        .collect()
}

/// The rows of the screen under the band of the player.
///
/// The band holds one row of a border above its words and one under them,
/// therefore the row of the second border is the end of it.
fn under_the_band(screen: &[String]) -> Vec<String> {
    let top = screen
        .iter()
        .position(|row| row.starts_with("┌ Player"))
        .expect("the band of the player stands on the screen");

    let bottom = screen
        .iter()
        .skip(top + 1)
        .position(|row| row.starts_with('└'))
        .map(|row| row + top + 1)
        .expect("the band of the player holds its border");

    screen[bottom + 1..].to_vec()
}

#[tokio::test(flavor = "multi_thread")]
async fn the_band_of_the_player_takes_no_line_of_a_view() {
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

    // The media of the measurement: a book of eight hours and of 70 chapters,
    // which is "A Second Book Of Many Hours" of the sandbox (the section 6k of
    // `docs/TEST-SERVER.md`).
    //
    // **The engine of the player stays away**: `without_engine` gives the
    // handle and the state of it, and the receiver holds the other end of the
    // channel. A receiver that goes away makes every command of the program a
    // fault of the log.
    let (player, _the_other_end_of_the_channel) = PlayerHandle::without_engine();

    {
        let state = player.shared_state();
        let mut state = state.write().expect("the state of the player");

        state.playback_id = 1;
        state.item_id = "e2b76945-10de-45f9-a09c-86c4666b9808".to_string();
        state.title = "A Second Book Of Many Hours".to_string();
        state.author = "Many Hours Author".to_string();
        state.position = 4_318.0;
        state.duration = 28_800.0;
        state.status = PlaybackStatus::Playing;
        state.chapter_title = Some("Chapter 11 of the second book".to_string());
        state.chapters = (0..70)
            .map(|number| Chapter {
                start: f64::from(number) * 411.0,
                end: f64::from(number + 1) * 411.0,
                title: format!("Chapter {} of the second book", number + 1),
            })
            .collect();
    }

    app.player = player;

    // The view of every key holds more lines than a terminal of 45 rows, and
    // it needs no server at all: `crate::ui::keys::lines` writes them.
    app.view_state = AppView::Keys;

    let screen = the_rows_of(&mut app, 45);
    let under = under_the_band(&screen);

    // **The row under the band is the row of the message** (the trap 39), and
    // it holds nothing while the program says nothing.
    assert!(
        under.first().is_some_and(|row| row.trim().is_empty()),
        "the row under the band of the view of the keys must hold nothing:\n{}",
        under.join("\n")
    );

    // The footer of the view stands under that row, and no line of the list
    // stands there: a line of a list of this program holds the border of its
    // panel or the sign of the cursor.
    for row in &under {
        assert!(
            !row.contains('│') && !row.contains('➤'),
            "a line of the view of the keys stands under the band:\n{row}"
        );
    }

    // The Chapters view holds the two bars over its table, and the chapters
    // come of the state of the player alone.
    app.view_state = AppView::Chapters;
    app.list_state_chapters.select(Some(10));

    let screen = the_rows_of(&mut app, 45);
    let under = under_the_band(&screen);

    assert!(
        under.first().is_some_and(|row| row.trim().is_empty()),
        "the row under the band of the Chapters view must hold nothing:\n{}",
        under.join("\n")
    );

    for row in &under {
        assert!(
            !row.contains('│') && !row.contains('➤'),
            "a line of the Chapters view stands under the band:\n{row}"
        );
    }

    // **The list of a view goes away last** (T-342 and T-343): a terminal of 8
    // rows gives the Chapters view one line of its table, and the band and the
    // two bars take the rows that stay.
    let screen = the_rows_of(&mut app, 8);
    let of_the_screen = screen.join("\n");

    assert!(
        !of_the_screen.contains("┌ Player"),
        "a screen of 8 rows must give the band no row at all:\n{of_the_screen}"
    );
    assert!(
        of_the_screen.contains("Chapter 11 of the second book"),
        "a screen of 8 rows must hold one line of the table of the chapters:\n{of_the_screen}"
    );
}
