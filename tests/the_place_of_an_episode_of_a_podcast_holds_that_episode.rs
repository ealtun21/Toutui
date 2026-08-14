//! The keys `M` and `N` of an episode of a podcast name that episode. See T-219.
//!
//! **The place of an episode of a podcast stands at
//! `/api/me/progress/:item/:episode`** (T-182 and T-188). The two keys read
//! `selected_item_id`, and that function gives the identity of the **item**
//! alone: an episode of a podcast therefore took the path of its podcast.
//!
//! The measurement of v0.8.48 against the sandbox, of the podcast
//! `Arthur Gordon Pym` of the library `Podcasts`. `Chapter 00` stood finished
//! and `Chapter 01` stood at 30 seconds:
//!
//! ```text
//! GET /api/me/progress/<the podcast>   ->  200, episodeId "Chapter 00", isFinished true
//! PATCH /api/me/progress/<the podcast> ->  400 Library item is not a book
//! ```
//!
//! | The view | The line | The key | v0.8.48 |
//! |---|---|---|---|
//! | Home of a library of podcasts | `Chapter 01` | `M` | `The server did not take the mark: … Status 400.` |
//! | Home of a library of podcasts | `Chapter 01` | `N` | `The server did not take the change: … Status 400.` |
//! | The episodes of a podcast | `Chapter 00` | `M` | `No media is selected.` |
//! | The episodes of a podcast | `Chapter 00` | `N` | `No media is selected.` |
//!
//! **The keys `D`, `X`, `n`, `m`, and `l` of those same lines do their work**,
//! therefore the line holds a media and the two keys said the opposite (T-79 and
//! T-91). The read of the item gave the place of another episode, and no fault
//! named it (T-188): a wrong path that answers is worse than a wrong path that
//! fails.
//!
//! `selected_place` reads `selected_download`, which holds the item **and** the
//! episode of every view that shows an episode. A podcast of the Library view
//! holds no place of its own, therefore that line names the key `l` of its
//! episodes (T-83).
//!
//! **This test needs no sandbox.** `App::new` takes a port that nothing listens
//! on, therefore it gives the offline mode (T-25), and a host of a raw socket
//! writes down the path of every request of the two keys.
//!
//! **The parts of this test stay in one function**: two test functions of one
//! binary take a thread each, and `cargo test` finds a fault of that shape at one
//! run of six (T-144 and T-157).

use std::sync::{Arc, Mutex};
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::app::{hide_the_media, mark_the_media, App, AppView};
use toutui::db::database_struct::User;
use toutui::logic::download::DownloadTarget;
use toutui::logic::home_view::HomeRow;

/// Nothing listens on this port.
const NO_SERVER: &str = "http://127.0.0.1:1";

/// The podcast of this measurement, and two of its episodes.
const THE_PODCAST: &str = "b793354b-9841-480a-bd09-41923596517e";
const CHAPTER_00: &str = "845f9d16-2121-40b1-a3ed-682cab9ed178";
const CHAPTER_01: &str = "482f0136-06eb-44a2-a202-c2ea3ad68a53";

fn a_user() -> User {
    User {
        server_address: NO_SERVER.to_string(),
        username: "toutuitest".to_string(),
        token: "not-a-real-token".to_string(),
        is_default_usr: true,
        name_selected_lib: "Podcasts".to_string(),
        id_selected_lib: "a-library".to_string(),
        is_loop_break: "0".to_string(),
        has_played_before: "1".to_string(),
        speed_rate: 1.0,
        is_show_key_bindings: "1".to_string(),
    }
}

/// Starts a host that answers every request with `200` and an empty body.
///
/// The list holds the method and the path of every request that came, therefore
/// the test says which path the two keys asked for.
async fn a_host_that_writes_down_every_path(the_requests: Arc<Mutex<Vec<String>>>) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());

    tokio::spawn(async move {
        while let Ok((mut socket, _)) = listener.accept().await {
            let the_requests = Arc::clone(&the_requests);

            tokio::spawn(async move {
                use tokio::io::{AsyncReadExt, AsyncWriteExt};

                let mut head = Vec::new();
                let mut byte = [0u8; 1];
                while socket.read(&mut byte).await.unwrap_or(0) == 1 {
                    head.push(byte[0]);
                    if head.ends_with(b"\r\n\r\n") {
                        break;
                    }
                }

                let head = String::from_utf8_lossy(&head).to_string();
                let first = head.lines().next().unwrap_or("").to_string();
                let mut words = first.split(' ');
                let method = words.next().unwrap_or("").to_string();
                let path = words.next().unwrap_or("").to_string();

                // The body of a `PATCH` must leave the socket. A host that
                // closes the connection before it gives the client a fault of
                // the network.
                let mut length = 0usize;
                for line in head.lines() {
                    if let Some(value) = line.to_lowercase().strip_prefix("content-length:") {
                        length = value.trim().parse().unwrap_or(0);
                    }
                }
                if length > 0 {
                    let mut body = vec![0u8; length];
                    let _ = socket.read_exact(&mut body).await;
                }

                if let Ok(mut list) = the_requests.lock() {
                    list.push(format!("{} {}", method, path));
                }

                let body: &[u8] = b"{}";
                let answer = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\
                     Content-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );

                let _ = socket.write_all(answer.as_bytes()).await;
                let _ = socket.write_all(body).await;
                let _ = socket.flush().await;
            });
        }
    });

    address
}

#[tokio::test(flavor = "multi_thread")]
async fn the_two_keys_name_the_episode_of_the_line() {
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

    app.is_podcast = true;

    // **The Home view of a library of podcasts holds the episodes.** The shelf
    // Continue Listening of the sandbox held `Chapter 01` of `Arthur Gordon
    // Pym`.
    app._ids_cnt_list = vec![THE_PODCAST.to_string()];
    app.ids_ep_cnt_list = vec![CHAPTER_01.to_string()];
    app._titles_cnt_list = vec!["Chapter 01".to_string()];
    app.titles_pod_cnt_list = vec!["Arthur Gordon Pym".to_string()];
    app.home_rows = vec![
        HomeRow::Shelf {
            label: "Continue Listening".to_string(),
        },
        HomeRow::Media { item: 0 },
    ];
    app.list_state_cnt_list.select(Some(1));
    app.view_state = AppView::Home;

    assert_eq!(
        app.selected_download(),
        Some((
            DownloadTarget::Episode {
                item_id: THE_PODCAST.to_string(),
                episode_id: CHAPTER_01.to_string(),
            },
            "Chapter 01".to_string(),
            "Arthur Gordon Pym".to_string(),
        )),
        "the line of the shelf holds the episode, and the keys D and X hold it"
    );

    assert_eq!(
        app.selected_place(),
        Some((THE_PODCAST.to_string(), Some(CHAPTER_01.to_string()))),
        "the keys M and N of that line name the episode, and not the podcast"
    );

    // **The view of the episodes of a podcast.** The two keys said "No media is
    // selected." for a line that holds an episode.
    app.ids_library = vec![THE_PODCAST.to_string()];
    app.library_rows = toutui::logic::library_view::group_library(&app.ids_library, &app.series);
    app.list_state_library.select(Some(0));
    app.is_from_search_pod = false;
    app.ids_pod_ep = vec![CHAPTER_00.to_string(), CHAPTER_01.to_string()];
    app.titles_pod_ep = vec!["Chapter 00".to_string(), "Chapter 01".to_string()];
    app.titles_pod = vec!["Arthur Gordon Pym".to_string()];
    app.list_state_pod_ep.select(Some(0));
    app.view_state = AppView::PodcastEpisode;

    assert_eq!(
        app.selected_place(),
        Some((THE_PODCAST.to_string(), Some(CHAPTER_00.to_string()))),
        "the keys M and N of the view of the episodes name the episode of the line"
    );

    // **A podcast holds no place of its own.** The line of the Library view
    // names the key that opens the episodes, and it does not say that the line
    // holds no media (T-83 and T-91).
    app.view_state = AppView::Library;

    assert_eq!(
        app.selected_place(),
        None,
        "a podcast of the Library view holds no place"
    );
    assert_eq!(
        app.words_of_a_line_with_no_place(),
        "A podcast holds no place. Press l for its episodes.",
        "the words of that line name the key of its episodes"
    );

    // **A view that holds no media at all keeps the words of before.**
    app.view_state = AppView::Settings;

    assert_eq!(
        app.selected_place(),
        None,
        "the view of the settings holds no media"
    );
    assert_eq!(
        app.words_of_a_line_with_no_place(),
        "No media is selected.",
        "a view with no media of a line says that it holds none"
    );

    // **A book of a library of books keeps the path of the item.**
    app.is_podcast = false;
    app.ids_library = vec!["a-book".to_string()];
    app.titles_library = vec!["A Test Book".to_string()];
    app.auth_names_library = vec!["A Test Author".to_string()];
    app.library_rows = toutui::logic::library_view::group_library(&app.ids_library, &app.series);
    app.list_state_library.select(Some(0));
    app.view_state = AppView::Library;

    assert_eq!(
        app.selected_place(),
        Some(("a-book".to_string(), None)),
        "a book holds the path of its item alone"
    );

    // **The two requests of each key.** The host writes down the path of the
    // read and the path of the write.
    let the_requests = Arc::new(Mutex::new(Vec::new()));
    let address = a_host_that_writes_down_every_path(Arc::clone(&the_requests)).await;
    let pool = EndpointPool::new(vec![Endpoint::new(&address, 0)]);
    let api = Arc::new(ApiClient::new(Arc::new(pool), "token".to_string()).unwrap());

    let _ = mark_the_media(&api, THE_PODCAST, Some(CHAPTER_01)).await;
    let _ = hide_the_media(&api, THE_PODCAST, Some(CHAPTER_01)).await;

    let paths = the_requests.lock().unwrap().clone();
    let of_the_episode = format!("/api/me/progress/{}/{}", THE_PODCAST, CHAPTER_01);

    assert_eq!(
        paths,
        vec![
            format!("GET {}", of_the_episode),
            format!("PATCH {}", of_the_episode),
            format!("GET {}", of_the_episode),
            format!("PATCH {}", of_the_episode),
        ],
        "the read and the write of the two keys name the episode"
    );

    // **A book takes the path of the item.** The road of the correction must not
    // give an episode to a media that holds none.
    the_requests.lock().unwrap().clear();
    let _ = mark_the_media(&api, "a-book", None).await;

    assert_eq!(
        the_requests.lock().unwrap().clone(),
        vec![
            "GET /api/me/progress/a-book".to_string(),
            "PATCH /api/me/progress/a-book".to_string(),
        ],
        "a book holds the path of its item alone"
    );
}
