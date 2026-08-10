//! The start must not wait for one request after the other. See T-40.
//!
//! `App::new` asks the server for the series, for the collections, for the
//! playlists, and for every item. Those four answers do not need each other.
//! The old code asked for them one after the other, therefore a slow server
//! made the user wait for the sum of the four.
//!
//! This test gives every answer a delay of 700 milliseconds. Four requests one
//! after the other need 2.8 seconds. Four requests together need about 0.7
//! seconds. The test fails at 2 seconds, therefore it tells the two apart.
//!
//! The test uses a mock server of `wiremock`. It needs no network and no
//! sandbox, therefore continuous integration runs it.

use std::sync::Arc;
use std::time::{Duration, Instant};
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::app::App;
use toutui::db::database_struct::User;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// The delay of each answer of the mock server.
const DELAY: Duration = Duration::from_millis(700);

/// Four requests one after the other need 2.8 seconds. The test fails there.
const LIMIT: Duration = Duration::from_millis(2000);

fn a_user(address: &str) -> User {
    User {
        server_address: address.to_string(),
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

#[tokio::test(flavor = "multi_thread")]
async fn the_four_requests_of_the_start_go_together() {
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

    let server = MockServer::start().await;

    // The list of the libraries comes at once. The program needs it before it
    // can ask anything else, therefore no delay belongs here.
    //
    // The answer is the answer of a real Audiobookshelf 2.36.0, and not a text
    // that a person wrote. A hand-written answer misses a field, and the test
    // then measures the offline mode.
    Mock::given(method("GET"))
        .and(path("/api/libraries"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(include_str!("data/one_library.json")),
        )
        .mount(&server)
        .await;

    // Every other answer is slow, and it holds no item. The program takes an
    // empty answer and it goes on; this test measures the time only.
    Mock::given(method("GET"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(DELAY)
                .set_body_json(serde_json::json!({})),
        )
        .mount(&server)
        .await;

    toutui::db::crud::db_insert_usr(&vec![a_user(&server.uri())]).unwrap();

    let pool = EndpointPool::new(vec![Endpoint::new(&server.uri(), 0)]);
    let api = Arc::new(ApiClient::new(Arc::new(pool), "token".to_string()).unwrap());

    let started = Instant::now();
    let app = App::new(Arc::clone(&api)).await.expect("an application");
    let took = started.elapsed();

    // The program must have reached the server, and not the offline mode.
    assert!(
        !app.is_offline,
        "the mock server answered, therefore the program must not be offline"
    );

    assert!(
        took < LIMIT,
        "the start took {:?}. Four requests of {:?} one after the other need \
         {:?}. The requests of the start do not go together.",
        took,
        DELAY,
        DELAY * 4
    );
}
