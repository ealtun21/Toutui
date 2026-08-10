//! Every view must draw when a list is shorter than the selection. See T-41.
//!
//! The user reported an "index out of bounds" that stopped the program. The
//! render read a vector with the number of the selected line, and a list of
//! the screen can be shorter than that number:
//!
//! - The user removes an account, and the list of the accounts keeps its old
//!   length until the next refresh.
//! - The server gives 40 items and 39 authors, because one item has no author.
//! - A list becomes empty while the user looks at it.
//!
//! A panic inside `Widget::render` stops the whole program, and the user then
//! has no screen at all.
//!
//! This test makes a real `App`, gives it lists of different lengths, puts the
//! selection past the end of every one of them, and draws every view. The test
//! needs no server: the address of the server is a port that nothing listens
//! on, therefore `App::new` gives the offline mode. See T-25.

use ratatui::backend::TestBackend;
use ratatui::widgets::ListState;
use ratatui::Terminal;
use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::app::{App, AppView};
use toutui::db::database_struct::User;

/// Nothing listens on this port, therefore every request fails at once and
/// `App::new` gives the offline mode.
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

/// Puts three lines in a list, and the selection on the line 99.
fn three(name: &str) -> Vec<String> {
    (1..=3)
        .map(|number| format!("{} {}", name, number))
        .collect()
}

fn past_the_end() -> ListState {
    let mut state = ListState::default();
    state.select(Some(99));
    state
}

#[tokio::test(flavor = "multi_thread")]
async fn every_view_draws_when_the_lists_are_shorter_than_the_selection() {
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

    // Every list holds three lines, and the lists of the values hold fewer.
    // A server that gives 40 items and 39 authors makes this condition.
    app.titles_library = three("A Book");
    app.ids_library = three("id");
    app.auth_names_library = vec!["One Author".to_string()];
    app.published_year_library = Vec::new();
    app.desc_library = Vec::new();
    app.duration_library = Vec::new();
    app.auth_names_library_pod = Vec::new();
    app.library_rows = toutui::logic::library_view::group_library(&app.ids_library, &app.series);

    app._titles_cnt_list = three("Continue");
    app._ids_cnt_list = three("id");
    app.auth_names_cnt_list = Vec::new();
    app.pub_year_cnt_list = Vec::new();
    app.desc_cnt_list = Vec::new();
    app.duration_cnt_list = Vec::new();
    app.book_progress_cnt_list = vec![Vec::new()];
    app.book_progress_cnt_list_cur_time = vec![Vec::new()];
    app.titles_pod_cnt_list = Vec::new();
    app.authors_pod_cnt_list = Vec::new();
    app.nums_ep_pod_cnt_list = Vec::new();
    app.durations_pod_cnt_list = Vec::new();
    app.subtitles_pod_cnt_list = Vec::new();

    app.titles_pod_ep = three("An Episode");
    app.subtitles_pod_ep = Vec::new();
    app.episodes_pod_ep = Vec::new();
    app.durations_pod_ep = Vec::new();
    app.authors_pod_ep = Vec::new();
    app.descs_pod_ep = Vec::new();
    app.seasons_pod_ep = Vec::new();
    app.titles_pod = Vec::new();

    app.titles_pod_ep_search = three("An Episode");
    app.subtitles_pod_ep_search = Vec::new();
    app.episodes_pod_ep_search = Vec::new();
    app.durations_pod_ep_search = Vec::new();
    app.authors_pod_ep_search = Vec::new();
    app.descs_pod_ep_search = Vec::new();
    app.seasons_pod_ep_search = Vec::new();
    app.titles_pod_search = Vec::new();

    app.ids_search_book = three("id");
    app.auth_names_search_book = Vec::new();
    app.auth_names_pod_search_book = Vec::new();
    app.published_year_library_search_book = Vec::new();
    app.desc_library_search_book = Vec::new();
    app.duration_library_search_book = Vec::new();

    app.all_usernames = Vec::new();
    app.libraries_names = three("A Library");
    app.libraries_ids = Vec::new();
    app.media_types = Vec::new();
    app.settings = three("A Setting");

    let views = [
        (AppView::Home, "Home"),
        (AppView::Library, "Library"),
        (AppView::SearchBook, "SearchBook"),
        (AppView::PodcastEpisode, "PodcastEpisode"),
        (AppView::Series, "Series"),
        (AppView::SeriesBook, "SeriesBook"),
        (AppView::Lists, "Lists"),
        (AppView::ListEntries, "ListEntries"),
        (AppView::Settings, "Settings"),
        (AppView::SettingsAccount, "SettingsAccount"),
        (AppView::SettingsLibrary, "SettingsLibrary"),
    ];

    for (view, name) in views {
        for is_podcast in [false, true] {
            for from_search in [false, true] {
                app.view_state = view;
                app.is_podcast = is_podcast;
                app.is_from_search_pod = from_search;

                // Every selection stands past the end of every list.
                app.list_state_cnt_list = past_the_end();
                app.list_state_library = past_the_end();
                app.list_state_search_results = past_the_end();
                app.list_state_pod_ep = past_the_end();
                app.list_state_series = past_the_end();
                app.list_state_series_book = past_the_end();
                app.list_state_lists = past_the_end();
                app.list_state_list_entries = past_the_end();
                app.list_state_settings = past_the_end();
                app.list_state_settings_account = past_the_end();
                app.list_state_settings_library = past_the_end();

                let backend = TestBackend::new(120, 40);
                let mut terminal = Terminal::new(backend).expect("a terminal");

                terminal
                    .draw(|frame| frame.render_widget(&mut app, frame.area()))
                    .unwrap_or_else(|error| {
                        panic!("the view {} did not draw: {}", name, error);
                    });

                // A screen that draws nothing at all is also a fault.
                let text: String = terminal
                    .backend()
                    .buffer()
                    .content()
                    .iter()
                    .map(|cell| cell.symbol())
                    .collect();

                assert!(
                    text.contains("Toutui"),
                    "the view {} drew no header (podcast={}, from a search={})",
                    name,
                    is_podcast,
                    from_search
                );
            }
        }
    }

    // A list with no line at all must also draw.
    app.titles_library = Vec::new();
    app.ids_library = Vec::new();
    app.library_rows = Vec::new();
    app._titles_cnt_list = Vec::new();
    app._ids_cnt_list = Vec::new();
    app.titles_pod_ep = Vec::new();
    app.titles_pod_ep_search = Vec::new();
    app.ids_search_book = Vec::new();
    app.libraries_names = Vec::new();
    app.settings = Vec::new();

    for view in [
        AppView::Home,
        AppView::Library,
        AppView::SearchBook,
        AppView::PodcastEpisode,
        AppView::Series,
        AppView::SeriesBook,
        AppView::Lists,
        AppView::ListEntries,
        AppView::Settings,
        AppView::SettingsAccount,
        AppView::SettingsLibrary,
    ] {
        app.view_state = view;
        app.list_state_library = past_the_end();
        app.list_state_cnt_list = past_the_end();

        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).expect("a terminal");
        terminal
            .draw(|frame| frame.render_widget(&mut app, frame.area()))
            .expect("the view must draw with no line at all");
    }
}
