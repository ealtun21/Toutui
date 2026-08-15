//! The keys of the view of the episodes of a podcast of a search read the list
//! that the view draws. See T-246.
//!
//! **The key of the playback of that view started no playback, and it said
//! nothing at all.** The two ways into the view of the episodes hold their
//! episodes in two different lists: `ids_pod_ep` for the Library view, and
//! `ids_pod_ep_search` for the view of the search. Every key of that view reads
//! the second one (`selected_download`), and the key of the playback read
//! `all_ids_pod_ep_search` — the box that the render of the view of the
//! **search** writes (`src/ui/tui.rs`). The program reads the episodes of a
//! podcast when the user opens it (T-126), therefore the answer of a podcast
//! that the user opens the first time comes after the view of the search went
//! away, and that box then holds no episode of that podcast at all. The same
//! key took the episodes of the view away before it read them: the block of the
//! key that opens a podcast wrote `ids_pod_ep_search` out of that same empty
//! box.
//!
//! The measurement of the real program v0.8.74 inside tmux, against the sandbox
//! (podman on :13399), of the library `Podcasts`. The key `/`, the word
//! `letters`, the key `Enter`, the key `l`, and two keys `j` gave the line
//! `Letter 3` of `Letters of Two Brides`:
//!
//! ```text
//! ──────────────────── Episodes [57 items] ────────────────────
//!       Letter 1
//!       Letter 2
//! ➤     Letter 3
//! ```
//!
//! The key `l` of that line gave no row of the player, no message, and **no
//! line of the log at all**: the log held 11 lines before that key and 11 after
//! it. **The control of the same run** (the trap 206): the key `D` of that same
//! line of that same frame said `"Letter 3" is now available offline.`, and the
//! key `h` and the key `l` of the search view gave that same view a second
//! time, where the same key `l` of the same line started the playback and wrote
//! the five lines of it in the log.
//!
//! The same keys of the corrected program gave the row of the player
//! `▶ 9:04 / 10:54 | Elapsed: 9:04 | Left: 1:50 (83%)`, the mark `▶` of the line
//! `Letter 3`, and the five lines of the log, at the **first** open of that
//! podcast.
//!
//! **The parts of this test stay in one function**: two test functions of one
//! module fight for the slot of that module, and `cargo test` then finds a
//! fault that nextest hides (T-144 and T-157).
//!
//! The box of the episodes is a slot of the whole process, therefore **this
//! test stands alone in its binary**. The address is a port that nothing
//! listens on, therefore `App::new` gives the offline mode (T-25) and this test
//! needs no server.
//!
//! **Two builds of the fault fail it**: the key of the playback that reads
//! `all_ids_pod_ep_search` instead of `ids_pod_ep_search`, and the block of the
//! key that opens a podcast with no guard of the view of the episodes.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::app::App;
use toutui::db::database_struct::User;
use toutui::logic::the_episodes::{keep, Episodes};

/// Nothing listens on this port.
const NO_SERVER: &str = "http://127.0.0.1:1";

const THE_PODCAST_OF_THE_LETTERS: &str = "the-podcast-of-letters-of-two-brides";
const THE_PODCAST_OF_PYM: &str = "the-podcast-of-arthur-gordon-pym";

fn a_user() -> User {
    User {
        server_address: NO_SERVER.to_string(),
        username: "toutuitest".to_string(),
        token: "not-a-real-token".to_string(),
        is_default_usr: true,
        name_selected_lib: "Podcasts".to_string(),
        id_selected_lib: "a-library-of-podcasts".to_string(),
        is_loop_break: "0".to_string(),
        has_played_before: "1".to_string(),
        speed_rate: 1.0,
        is_show_key_bindings: "1".to_string(),
    }
}

fn texts(values: &[&str]) -> Vec<String> {
    values.iter().map(|one| one.to_string()).collect()
}

fn a_key(code: char) -> KeyEvent {
    KeyEvent {
        code: KeyCode::Char(code),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: ratatui::crossterm::event::KeyEventState::NONE,
    }
}

/// The answer of the server for one podcast, in the form of the box of T-126.
fn the_episodes_of(place: usize, id: &str, titles: &[&str]) -> Episodes {
    Episodes {
        place,
        id: id.to_string(),
        titles: texts(titles),
        ids: titles.iter().map(|one| format!("{}/{}", id, one)).collect(),
        subtitles: vec!["N/A".to_string(); titles.len()],
        seasons: vec!["N/A".to_string(); titles.len()],
        numbers: vec!["N/A".to_string(); titles.len()],
        authors: vec!["LibriVox".to_string(); titles.len()],
        descriptions: vec!["N/A".to_string(); titles.len()],
        titles_of_the_podcast: vec!["Letters of Two Brides".to_string(); titles.len()],
        durations: vec!["29m".to_string(); titles.len()],
        lengths: vec![Some(1740.0); titles.len()],
        places: vec![texts(&[" N/A", " N/A", " N/A"]); titles.len()],
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn the_keys_of_the_view_of_the_episodes_of_a_search_read_that_view() {
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

    // The library of the measurement: two podcasts, and the lists of the
    // episodes hold one empty row for each of them (T-126).
    app.is_podcast = true;
    app.ids_library = texts(&[THE_PODCAST_OF_THE_LETTERS, THE_PODCAST_OF_PYM]);
    app.titles_library = texts(&["Letters of Two Brides", "Arthur Gordon Pym"]);
    app.all_titles_pod_ep = vec![Vec::new(); 2];
    app.all_ids_pod_ep = vec![Vec::new(); 2];
    app.all_subtitles_pod_ep = vec![Vec::new(); 2];
    app.all_seasons_pod_ep = vec![Vec::new(); 2];
    app.all_episodes_pod_ep = vec![Vec::new(); 2];
    app.all_authors_pod_ep = vec![Vec::new(); 2];
    app.all_descs_pod_ep = vec![Vec::new(); 2];
    app.all_titles_pod = vec![Vec::new(); 2];
    app.all_durations_pod_ep = vec![Vec::new(); 2];
    app.all_the_lengths_of_the_episodes = vec![Vec::new(); 2];
    app.all_pod_ep_places = vec![Vec::new(); 2];
    app.the_episodes_that_came = vec![false; 2];

    // **The user searched, and they opened the one podcast that the search
    // found.** The render of the view of the search wrote the lists of that
    // view out of the lists of the library, and the lists of the library held
    // no episode of that podcast at that moment: the program asks the server
    // for them at this key (T-126).
    app.ids_library_pod_search = texts(&[THE_PODCAST_OF_THE_LETTERS]);
    app.ids_search_book = texts(&[THE_PODCAST_OF_THE_LETTERS]);
    app.all_ids_pod_ep_search = vec![Vec::new(); 1];
    app.all_pod_ep_places_search = vec![Vec::new(); 1];
    app.list_state_search_results.select(Some(0));
    app.is_from_search_pod = true;
    app.view_state = toutui::app::AppView::PodcastEpisode;

    // **The answer of the podcast comes after the view of the search went
    // away.** It reaches the lists of that view, and the lists of the library.
    keep(the_episodes_of(
        0,
        THE_PODCAST_OF_THE_LETTERS,
        &["Letter 1", "Letter 2", "Letter 3"],
    ));
    app.take_the_episodes_that_came();

    assert_eq!(
        app.ids_pod_ep_search,
        texts(&[
            "the-podcast-of-letters-of-two-brides/Letter 1",
            "the-podcast-of-letters-of-two-brides/Letter 2",
            "the-podcast-of-letters-of-two-brides/Letter 3",
        ]),
        "the answer of the podcast gives the view of the search its episodes"
    );

    // The user stands on the third line, as two keys `j` of the measurement
    // gave it.
    app.list_state_pod_ep.select(Some(2));

    assert_eq!(
        app.the_episode_of_the_line_of_the_episodes(),
        Some("the-podcast-of-letters-of-two-brides/Letter 3".to_string()),
        "the key of the playback of that view names the episode of the line, and \
         it read a box that the render of the view of the search writes"
    );

    // **The measurement.** The key of the playback of that line. The address is
    // a port that nothing listens on, therefore the playback of the task that
    // this key starts reaches no server, and the lists of the view are the
    // measurement.
    app.handle_key(a_key('l'));

    assert_eq!(
        app.ids_pod_ep_search,
        texts(&[
            "the-podcast-of-letters-of-two-brides/Letter 1",
            "the-podcast-of-letters-of-two-brides/Letter 2",
            "the-podcast-of-letters-of-two-brides/Letter 3",
        ]),
        "the key of the view of the episodes keeps the episodes of that view, \
         and it took them out of the box of the render of the view of the search"
    );
    assert_eq!(
        app.the_episode_of_the_line_of_the_episodes(),
        Some("the-podcast-of-letters-of-two-brides/Letter 3".to_string()),
        "the key names the episode of the line after it, therefore the key `D` \
         and the key `X` of that same line name that same episode"
    );

    // **The road of the Library view keeps its own list.** The key `l` of a
    // podcast of that view gives the view the episodes of the line out of the
    // lists of the library, and the view holds them in `ids_pod_ep`.
    app.is_from_search_pod = false;
    app.view_state = toutui::app::AppView::Library;
    app.list_state_library.select(Some(0));
    app.handle_key(a_key('l'));
    app.list_state_pod_ep.select(Some(1));

    assert_eq!(
        app.view_state,
        toutui::app::AppView::PodcastEpisode,
        "the key `l` of a podcast of the Library view opens its episodes"
    );
    assert_eq!(
        app.the_episode_of_the_line_of_the_episodes(),
        Some("the-podcast-of-letters-of-two-brides/Letter 2".to_string()),
        "the view of the episodes of the Library view names the episode of its \
         own line"
    );
}
