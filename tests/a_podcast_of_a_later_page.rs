//! The key `l` of a podcast of a page that the program did not read. See T-126.
//!
//! **The key stopped the program.** The start read the episodes of every
//! podcast of the first page, therefore the lists of the episodes held 500
//! rows: a library of 520 podcasts gave the line 519 no row at all, and
//! `self.all_ids_pod_ep[index]` stopped the program with an index of a vector
//! that does not exist. The sweep of a library of 520 podcasts of 2026-08-12
//! measured it: the program went away, and tmux said "can't find pane".
//!
//! The program reads the episodes of one podcast when the user opens it now,
//! and every list of this view takes `get`.
//!
//! The test needs no server: the address is a port that nothing listens on,
//! therefore `App::new` gives the offline mode and no request answers.
//!
//! The test writes `XDG_CONFIG_HOME`, therefore it stands alone in its binary
//! (the trap 8 of the harness).

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::app::{App, AppView};
use toutui::db::database_struct::User;

/// Nothing listens on this port.
const NO_SERVER: &str = "http://127.0.0.1:1";

/// The first letters of the screen, for the words of a test that fails.
fn the_first_letters(screen: &str) -> String {
    screen.chars().take(400).collect()
}

fn a_user() -> User {
    User {
        server_address: NO_SERVER.to_string(),
        username: "toutuitest".to_string(),
        token: "not-a-real-token".to_string(),
        is_default_usr: true,
        name_selected_lib: "ManyPods".to_string(),
        id_selected_lib: "a-library".to_string(),
        is_loop_break: "0".to_string(),
        has_played_before: "1".to_string(),
        speed_rate: 1.0,
        is_show_key_bindings: "1".to_string(),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn the_key_of_a_podcast_that_the_program_did_not_read_keeps_the_program() {
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

    // A library of podcasts of 520 items, and the program holds the lines of
    // every page. The lists of the episodes hold the rows of the first page
    // only: that is the shape of the program before the pages of T-70 came.
    app.is_podcast = true;
    app.view_state = AppView::Library;
    app.library_total = 520;
    app.titles_library = (1..=520)
        .map(|one| format!("Many Podcast {}", one))
        .collect();
    app.ids_library = (1..=520).map(|one| format!("podcast-{}", one)).collect();
    app.library_rows = toutui::logic::library_view::group_library(&app.ids_library, &app.series);

    app.all_titles_pod_ep = vec![Vec::new(); 500];
    app.all_ids_pod_ep = vec![Vec::new(); 500];
    app.all_subtitles_pod_ep = vec![Vec::new(); 500];
    app.all_seasons_pod_ep = vec![Vec::new(); 500];
    app.all_episodes_pod_ep = vec![Vec::new(); 500];
    app.all_authors_pod_ep = vec![Vec::new(); 500];
    app.all_descs_pod_ep = vec![Vec::new(); 500];
    app.all_titles_pod = vec![Vec::new(); 500];
    app.all_durations_pod_ep = vec![Vec::new(); 500];
    app.the_episodes_that_came = vec![false; 500];

    // The line 519 is a podcast of the second page.
    app.list_state_library.select(Some(519));

    // **The key `l` stopped the program here.**
    app.handle_key(KeyEvent::from(KeyCode::Char('l')));

    assert!(
        matches!(app.view_state, AppView::PodcastEpisode),
        "the key opens the episodes of that podcast"
    );

    // The view draws, and it says why it holds no episode: the program asks
    // the server for them. A view must not give a reason that it does not have
    // (T-91).
    let mut terminal = Terminal::new(TestBackend::new(160, 45)).unwrap();
    terminal
        .draw(|frame| frame.render_widget(&mut app, frame.area()))
        .unwrap();

    let screen: String = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect();

    assert!(
        !screen.contains("This podcast has no episode"),
        "the program did not read the episodes of that podcast: {}",
        the_first_letters(&screen)
    );
    // The address of this test is a port that nothing listens on, therefore
    // the program is in the offline mode and it says that reason (T-91). A
    // program with a server says "The program gets the episodes of this
    // podcast…".
    assert!(
        screen.contains("does not answer"),
        "the view says what the program knows: {}",
        the_first_letters(&screen)
    );
}
