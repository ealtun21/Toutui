//! The gate of T-358: a view with no line says the name of the list that holds
//! no line.
//!
//! The measurement of the real program v0.8.188 inside tmux, at 160 columns and
//! 45 rows, of the library `Empty` of the sandbox (`docs/TEST-SERVER.md`). That
//! library holds no media, no collection, and no playlist. The correction of
//! T-357 took the panel of the cover of those views away, therefore the sentence
//! of each of them stood over the whole width of the screen, **under a rule of
//! 160 columns with no word in it**:
//!
//! ```text
//! ────────────────────────────────────────────────────────────────────────────
//!                  This library has no collection and no playlist.
//!                                Press h to go back.
//! ```
//!
//! The Series view of the same run said `This library has no series.` under the
//! same bare rule, and the Episodes view of the podcast `Letters of Two Brides`,
//! with `docs/harness/one_path_fails.py` on the path of its item (T-278), said
//! `The server did not give the episodes of this podcast: The server reported a
//! fault. Status 500.` under it.
//!
//! **The control of the same run is the same view with its lines.** The
//! Collections view of the library `Books` drew that same rule with
//! `Collections and playlists [2 items]` in the middle of it, and the search
//! view of that library drew `Search result [14 items]`. A view of lines
//! therefore names its list already, and a view with no line lost that name.
//!
//! The corrected program of the same harness, of the same screen:
//!
//! ```text
//! ──────────────────────────Collections and playlists [0 items]───────────────
//!                  This library has no collection and no playlist.
//!                                Press h to go back.
//! ```
//!
//! **Why.** `render_list` of `crate::ui::the_list_of_a_view` gives the rule of a
//! view outside the frame of the panels the title of the caller
//! (`Block::new().title(Line::raw(title).centered()).borders(Borders::TOP)`).
//! The three widgets of a view with no line did not: `App::render_the_reason`
//! took a title and it dropped it in the arm of the screen with no frame,
//! `render_lists` built a `Paragraph` of its own with no title at all, and
//! `crate::ui::the_message_of_a_view::render_the_message` took no title at all.
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
use toutui::api::utils::collect_lists::{ListEntry, ListKind, ListView};
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
async fn a_view_with_no_line_says_the_name_of_its_list() {
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

    // The terminal of the measurement: 160 columns and 45 rows.
    let mut terminal = Terminal::new(TestBackend::new(160, 45)).expect("a terminal");

    app.is_podcast = false;

    // ## 1. The Collections view with no line
    //
    // This road held a `Paragraph` of its own, with a block of one border at the
    // top and no title at all.
    app.view_state = AppView::Lists;
    app.lists = Vec::new();

    terminal
        .draw(|frame| frame.render_widget(&mut app, frame.area()))
        .expect("the Collections view with no line draws");

    let rows = the_rows_of(&terminal);

    assert!(
        rows.iter().any(|row| row.contains("no collection")),
        "the Collections view with no line says why it holds no line.\n{}",
        rows.join("\n")
    );
    assert!(
        rows.iter()
            .any(|row| row.contains("Collections and playlists [0 items]")),
        "the Collections view with no line says the name of the list that holds no line.\n{}",
        rows.join("\n")
    );

    // ## 2. The Series view with no line
    //
    // This road calls `App::render_the_reason`, which took the title of the
    // caller and drew nothing of it while the frame of the panels does not
    // stand — and the frame stands in the Home view and in the Library view
    // alone (T-320).
    app.view_state = AppView::Series;
    app.series = Vec::new();

    terminal
        .draw(|frame| frame.render_widget(&mut app, frame.area()))
        .expect("the Series view with no line draws");

    let rows = the_rows_of(&terminal);

    assert!(
        rows.iter().any(|row| row.contains("no series")),
        "the Series view with no line says why it holds no line.\n{}",
        rows.join("\n")
    );
    assert!(
        rows.iter().any(|row| row.contains("Series [0 items]")),
        "the Series view with no line says the name of the list that holds no line.\n{}",
        rows.join("\n")
    );

    // ## 3. The Episodes view of a podcast with no line
    //
    // This road calls `crate::ui::the_message_of_a_view::render_the_message`,
    // which took no title at all.
    app.view_state = AppView::PodcastEpisode;
    app.is_from_search_pod = false;
    app.titles_pod_ep = Vec::new();
    app.titles_pod_ep_search = Vec::new();

    terminal
        .draw(|frame| frame.render_widget(&mut app, frame.area()))
        .expect("the Episodes view with no line draws");

    let rows = the_rows_of(&terminal);

    assert!(
        rows.iter().any(|row| row.contains("Episodes [0 items]")),
        "the Episodes view with no line says the name of the list that holds no line.\n{}",
        rows.join("\n")
    );

    // **The road of a search of a podcast says the same name**, and it reads a
    // list of its own: a correction that gives the title of one road alone
    // fails here.
    app.is_from_search_pod = true;

    terminal
        .draw(|frame| frame.render_widget(&mut app, frame.area()))
        .expect("the Episodes view of a search with no line draws");

    let rows = the_rows_of(&terminal);

    assert!(
        rows.iter().any(|row| row.contains("Episodes [0 items]")),
        "the Episodes view of a search with no line says the name of its list.\n{}",
        rows.join("\n")
    );

    // ## 4. The control of the same run: the same view with its lines
    //
    // The view of lines named its list before this item, and it must say the
    // same words after it: the two roads of one view say one name.
    app.view_state = AppView::Lists;
    app.lists = vec![ListView {
        id: "a-collection".to_string(),
        kind: ListKind::Collection,
        name: "A Test Collection".to_string(),
        description: String::new(),
        entries: vec![ListEntry {
            id: "a-media".to_string(),
            episode_id: None,
            title: "A Book Of A Test".to_string(),
            author: "A Writer".to_string(),
            duration: 600.0,
            description: String::new(),
        }],
    }];
    app.list_state_lists.select(Some(0));

    terminal
        .draw(|frame| frame.render_widget(&mut app, frame.area()))
        .expect("the Collections view of one line draws");

    let rows = the_rows_of(&terminal);

    assert!(
        rows.iter()
            .any(|row| row.contains("Collections and playlists [1 item]")),
        "the Collections view of one line says the name of its list.\n{}",
        rows.join("\n")
    );
}
