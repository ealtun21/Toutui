//! The view of the episodes of a podcast holds the place of the episodes of
//! that podcast. See T-245.
//!
//! **That view held the places of the podcast that the user opened before it.**
//! The lists of the episodes of a line come of the lists of the library
//! (T-126), and the places of T-229 came of the request of one podcast alone:
//! no list of the library held them. A podcast whose episodes the program read
//! already makes no request, therefore the places of the podcast before it
//! stayed on the screen for ever.
//!
//! The measurement of the real program v0.8.73 inside tmux, against the sandbox
//! (podman on :13399), of the library `Podcasts` of two podcasts. The server
//! held `Chapter 00` of `Arthur Gordon Pym` at 22 percent, `Chapter 01` at 74,
//! and `Chapter 02` at 89, and it held no place of `Letter 1`, of `Letter 2`,
//! and of `Letter 3` of `Letters of Two Brides`. The user opened the episodes
//! of `Letters of Two Brides`, they went back, they opened `Arthur Gordon Pym`,
//! they went back, and they opened `Letters of Two Brides` a second time:
//!
//! ```text
//! ➤ 22% Letter 1
//!   74% Letter 2
//!   89% Letter 3
//!       Letter 4
//! ```
//!
//! The panel of that first line said
//! `Progress: 22%, 28m left, Not finished` — the place of `Chapter 00` of the
//! other podcast, and the time that is left of the length of `Letter 1`.
//!
//! **The control of the same run** (the trap 206): the first open of that same
//! podcast of that same program gave `Letter 1` and `Letter 2` with no percent
//! at all, and the open of `Arthur Gordon Pym` between them gave
//! `22% Chapter 00`, `74% Chapter 01`, and `89% Chapter 02` — the places of the
//! right podcast.
//!
//! The same measurement of the corrected program gave `Letter 1` with no
//! percent and the panel `Progress:  N/A%,   N/A`. A second control gave
//! `Letter 2` of `Letters of Two Brides` the place 40 percent with `curl`, and
//! the third open of that podcast said `40% Letter 2` and nothing of the other
//! lines: the view holds the places of its own podcast, and it holds them with
//! no request at all.
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
//! **Two builds of the fault fail it**: a line of the view that reads no list
//! of the library (`take_the_episodes_of_the_line` with no
//! `self.pod_ep_places = …`), and an answer of the server that reaches no list
//! of the library (`take_the_episodes_that_came` with no write of
//! `all_pod_ep_places`).

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::app::App;
use toutui::db::database_struct::User;
use toutui::logic::the_episodes::{keep, Episodes};

/// Nothing listens on this port.
const NO_SERVER: &str = "http://127.0.0.1:1";

const THE_PODCAST_OF_PYM: &str = "the-podcast-of-arthur-gordon-pym";
const THE_PODCAST_OF_THE_LETTERS: &str = "the-podcast-of-letters-of-two-brides";

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

/// The place of the user of each episode, in the form of
/// `App::book_progress_cnt_list`: the percent, the mark of the end, and the
/// place in seconds (T-234).
fn the_places_of_pym() -> Vec<Vec<String>> {
    vec![
        texts(&["22", "Not finished", "66"]),
        texts(&["74", "Not finished", "1000"]),
        texts(&["89", "Not finished", "2070"]),
    ]
}

fn no_place_at_all() -> Vec<Vec<String>> {
    vec![texts(&[" N/A", " N/A", " N/A"]); 3]
}

/// The answer of the server for one podcast, in the form of the box of T-126.
fn the_episodes_of(place: usize, id: &str, titles: &[&str], places: Vec<Vec<String>>) -> Episodes {
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
        titles_of_the_podcast: vec!["A Podcast".to_string(); titles.len()],
        durations: vec!["29m".to_string(); titles.len()],
        lengths: vec![Some(1740.0); titles.len()],
        places,
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn the_places_of_the_view_of_the_episodes_hold_their_podcast() {
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

    // **The user opens the podcast of the letters.** The answer of the server
    // holds no place of any episode of it.
    app.view_state = toutui::app::AppView::PodcastEpisode;
    keep(the_episodes_of(
        0,
        THE_PODCAST_OF_THE_LETTERS,
        &["Letter 1", "Letter 2", "Letter 3"],
        no_place_at_all(),
    ));
    app.take_the_episodes_that_came();

    // **The user opens the podcast of Arthur Gordon Pym.** The answer of the
    // server holds the place of three episodes of it.
    keep(the_episodes_of(
        1,
        THE_PODCAST_OF_PYM,
        &["Chapter 00", "Chapter 01", "Chapter 02"],
        the_places_of_pym(),
    ));
    app.take_the_episodes_that_came();

    assert_eq!(
        app.pod_ep_places,
        the_places_of_pym(),
        "the view of the episodes of the podcast that the user opened holds the \
         places of the answer of that podcast"
    );

    // **The answer of every podcast reaches the lists of the library.** A
    // podcast whose episodes the program read already makes no request (T-126),
    // therefore the place of an episode of it lives here alone.
    assert_eq!(
        app.all_pod_ep_places,
        vec![no_place_at_all(), the_places_of_pym()],
        "the lists of the library hold the places of each podcast of the library"
    );

    // **The measurement.** The user goes back, and they open the podcast of the
    // letters a second time. No request comes of that key.
    app.take_the_episodes_of_the_line(0);

    assert_eq!(
        app.pod_ep_places,
        no_place_at_all(),
        "the view holds the places of the podcast of the line, and it held the \
         places of the podcast that the user opened before it"
    );
    assert_eq!(
        app.titles_pod_ep,
        texts(&["Letter 1", "Letter 2", "Letter 3"]),
        "the titles of that view name the episodes of the podcast of the line"
    );

    // The podcast of Arthur Gordon Pym keeps its places for the key that opens
    // it again.
    app.take_the_episodes_of_the_line(1);

    assert_eq!(
        app.pod_ep_places,
        the_places_of_pym(),
        "a second visit of a podcast holds the places of the answer of the first \
         visit, because that key makes no request"
    );
}
