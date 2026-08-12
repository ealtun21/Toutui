//! The start must not wait for one request after the other. See T-40.
//!
//! `App::new` asks the server for the series, for the collections, for the
//! playlists, and for every item. Those four answers do not need each other.
//! The old code asked for them one after the other, therefore a slow server
//! made the user wait for the sum of the four.
//!
//! This test gives every answer a delay of 700 milliseconds, and it holds the
//! **time of each request**. Four requests that go together start inside some
//! milliseconds of each other. Four requests one after the other start 700
//! milliseconds apart, because each of them waits for the answer before it.
//!
//! **The old form of this test measured the time of the whole start**, and it
//! failed at 2 seconds. That is a measurement of the machine as much as of the
//! program: a machine that builds and runs a second program answers slowly, and
//! the test then failed with no fault of the program. A measurement of
//! 2026-08-11 gave one such fault of twelve runs, and it took 4.2 seconds. The
//! time between the first request and the last one does not change with the load
//! of the machine, therefore this test holds that value now. See T-86.
//!
//! The test uses a mock server of `wiremock`. It needs no network and no
//! sandbox, therefore continuous integration runs it.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::app::App;
use toutui::db::database_struct::User;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// The delay of each answer of the mock server.
const DELAY: Duration = Duration::from_millis(700);

/// The longest time between the first request of the start and the last one.
///
/// Four requests that go together start inside some milliseconds of each other.
/// Four requests one after the other start `DELAY` apart. This value stands
/// between the two, and it does not change with the load of the machine.
const BETWEEN_THE_FIRST_AND_THE_LAST: Duration = Duration::from_millis(500);

/// The whole start must not hold for ever. This limit is generous: it says that
/// the program came back, and it says nothing about the sequence of the
/// requests.
const THE_START_MUST_END: Duration = Duration::from_secs(20);

/// Holds the time of each request of the mock server.
///
/// `wiremock` gives every request to this rule before it answers, therefore the
/// time is the time when the request **arrived**. See T-86.
#[derive(Clone, Default)]
struct NotesTheTime(Arc<Mutex<Vec<Instant>>>);

impl wiremock::Match for NotesTheTime {
    fn matches(&self, _request: &wiremock::Request) -> bool {
        if let Ok(mut times) = self.0.lock() {
            times.push(Instant::now());
        }

        true
    }
}

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

    // **The account of the token stands outside this measurement.** The
    // program asks `GET /api/me` at the first moment of the start, and that
    // answer holds the position of every media (T-127): it does not wait for
    // the four requests below, and the four do not wait for it. A measurement
    // of 2026-08-12 saw it at 0.34 ms, while the four came at 702 ms.
    let of_the_account = NotesTheTime::default();

    Mock::given(method("GET"))
        .and(path("/api/me"))
        .and(of_the_account.clone())
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(DELAY)
                .set_body_json(serde_json::json!({})),
        )
        .mount(&server)
        .await;

    // Every other answer is slow, and it holds no item. The program takes an
    // empty answer and it goes on; this test measures the sequence only.
    let times = NotesTheTime::default();

    Mock::given(method("GET"))
        .and(times.clone())
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
        took < THE_START_MUST_END,
        "the start took {:?}. It must come back.",
        took
    );

    // **The measurement of the sequence.** Four requests that go together
    // arrive inside some milliseconds of each other. Four requests one after
    // the other arrive `DELAY` apart.
    let times = times.0.lock().expect("the times of the requests").clone();

    // **The first request stands alone, and it is not one of the four.** The
    // pool examines the address before the program asks for anything, and that
    // examination takes one answer of the server. A measurement of 2026-08-11
    // gave the times `[0 ms, 702, 702, 702, 702, 702]`: the examination, and
    // then the list of the libraries with the four requests of the start, all
    // together.
    assert!(
        times.len() >= 5,
        "the start makes the examination of the address and four requests more, \
         and the server saw {}",
        times.len()
    );

    let of_the_start = &times[1..];

    let first = of_the_start.first().expect("the first request");
    let last = of_the_start.last().expect("the last request");

    println!(
        "the server saw {} requests, at {:?}",
        times.len(),
        times
            .iter()
            .map(|time| time.duration_since(times[0]))
            .collect::<Vec<_>>()
    );

    // The account of the token: one request, and it goes with the examination
    // of the address.
    let of_the_account = of_the_account.0.lock().expect("the account").clone();

    assert_eq!(
        of_the_account.len(),
        1,
        "the start asks the server for the account of the token one time. \
         The positions of the media come with that answer (T-127)."
    );

    let between = last.duration_since(*first);

    assert!(
        between < BETWEEN_THE_FIRST_AND_THE_LAST,
        "the requests of the start arrived over {:?}. Each answer of the server \
         takes {:?}, therefore requests that wait for each other arrive {:?} \
         apart. The requests of the start do not go together.",
        between,
        DELAY,
        DELAY
    );
}
