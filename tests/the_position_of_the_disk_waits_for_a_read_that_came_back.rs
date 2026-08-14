//! The flush of the positions of the disk reads before it writes. See T-188.
//!
//! **An offline playback keeps its place in `pending_progress` at each second**
//! (T-152), and that row is the one copy of such a playback: no row of
//! `listening_session` stands beside it. `flush_pending_progress` sends the row
//! when the server answers again, and it asks the server for the position of
//! that media first: a different client can hold a newer position, and the
//! newer position wins.
//!
//! **That read and that write stand on one path**, therefore this is the shape
//! of T-175. The old code read **every** fault of the read as "the server holds
//! no position of this media", and it then wrote its own position over the
//! position of the server.
//!
//! The three measurements of 2026-08-14 against the sandbox, with one book of
//! eight hours and one podcast of 11 episodes:
//!
//! | The road | The server before | The disk | The server after |
//! |---|---|---|---|
//! | `docs/harness/one_method_fails.py GET:/api/me/progress` | 5000 s, the moment of now | 100 s, one hour old | **100 s** |
//! | `docs/harness/a_field_of_the_answer_goes_away.py /api/me/progress/:id lastUpdate` | 5000 s | 100 s, one hour old | **100 s** |
//! | no proxy: the episode 01 of the server holds 10 s, and the episode 00 holds the newest moment | 10 s of the episode 01 | 500 s of the episode 01 | **10 s** |
//!
//! The first two roads threw the place of the user of the **server** away, and
//! the log said "the server took the position 100s". The third road threw the
//! place of the user of the **disk** away: `GET /api/me/progress/:id` of a
//! podcast answers with the position of **one** episode of that podcast, and the
//! moment of that other episode then decided for this one. The log said "the
//! server has a newer position", and the offline listening of 500 seconds went
//! away with no word for the user.
//!
//! **This test needs no sandbox.** A host of `wiremock` gives the fault to the
//! read alone, and it writes down every request that came.

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::db::crud::{get_pending_progress, insert_pending_progress, PendingProgress};
use toutui::logic::offline::flush_pending_progress;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// The media of this measurement, and the two episodes of the podcast.
const THE_BOOK: &str = "6ba57b9a-acb5-44f9-b2b6-39ad9107b420";
const THE_PODCAST: &str = "b793354b-9841-480a-bd09-41923596517e";
const THE_EPISODE: &str = "482f0136-06eb-44a2-a202-c2ea3ad68a53";

const THE_ACCOUNT: &str = "the-account-of-the-flush";
const THE_SERVER: &str = "the-server-of-the-flush";

static HOME: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();

fn temporary_home() {
    HOME.get_or_init(|| {
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", dir.path());
        std::fs::create_dir_all(dir.path().join("toutui")).unwrap();

        let conn = toutui::db::migrate::open_conn().unwrap();
        toutui::db::migrate::run_migrations(&conn).unwrap();
        drop(conn);

        dir
    });
}

fn a_client(url: &str) -> ApiClient {
    let pool = EndpointPool::new(vec![Endpoint::new(url, 0)]);
    ApiClient::new(Arc::new(pool), "test-token".to_string()).unwrap()
}

/// Writes the one row of the disk, and it takes every other row away.
fn the_row_of_the_disk(id_item: &str, id_pod: &str, at: f64, moments_ago: i64) {
    let conn = toutui::db::migrate::open_conn().expect("the database of the test");
    conn.execute("DELETE FROM pending_progress", [])
        .expect("the table of the test");
    drop(conn);

    insert_pending_progress(
        THE_ACCOUNT,
        THE_SERVER,
        &PendingProgress {
            id_item: id_item.to_string(),
            id_pod: id_pod.to_string(),
            current_time: at,
            duration: 28800.0,
            is_finished: false,
            updated_at: toutui::logic::offline::now_ms() - moments_ago,
        },
    )
    .expect("the row of the test");
}

/// The body of a position of the server, with the moment of it.
fn the_position_of_the_server(at: f64, last_update: i64) -> serde_json::Value {
    serde_json::json!({
        "libraryItemId": THE_BOOK,
        "currentTime": at,
        "duration": 28800,
        "isFinished": false,
        "lastUpdate": last_update,
    })
}

/// A read that came back with a fault writes nothing, and the row of the disk
/// waits.
///
/// **The parts of this test stay in one function**: two test functions of one
/// binary take a thread each, and `cargo test` finds a fault of that shape at
/// one run of six (T-144 and T-157).
#[tokio::test(flavor = "multi_thread")]
async fn the_flush_writes_no_position_that_it_did_not_read() {
    temporary_home();

    // ── The first road: the read gives the status 500, and the write of the
    // same path answers 200. The old code wrote the position of the disk over
    // the newer position of the server.
    let host = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!("/api/me/progress/{}", THE_BOOK)))
        .respond_with(ResponseTemplate::new(500))
        .mount(&host)
        .await;

    Mock::given(method("PATCH"))
        .and(path(format!("/api/me/progress/{}", THE_BOOK)))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&host)
        .await;

    the_row_of_the_disk(THE_BOOK, "", 100.0, 3_600_000);

    let api = a_client(&host.uri());
    let sent = flush_pending_progress(&api, THE_ACCOUNT, THE_SERVER).await;

    assert_eq!(
        sent, 0,
        "a read that came back with the status 500 must send no position"
    );

    let the_writes = host
        .received_requests()
        .await
        .unwrap_or_default()
        .into_iter()
        .filter(|request| request.method.as_str() == "PATCH")
        .count();

    assert_eq!(
        the_writes, 0,
        "the program must write no position that it did not read"
    );

    assert_eq!(
        get_pending_progress(THE_ACCOUNT, THE_SERVER).unwrap().len(),
        1,
        "the row of the disk is the one copy of an offline playback (T-152), \
         therefore a fault of the read must keep it"
    );

    // ── The second road: the read answers 200, and it holds no `lastUpdate`.
    // The field takes the default 0, and the old code then compared the moment
    // of the disk with the moment of 1970.
    let host = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!("/api/me/progress/{}", THE_BOOK)))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "libraryItemId": THE_BOOK,
            "currentTime": 5000,
            "duration": 28800,
            "isFinished": false,
        })))
        .mount(&host)
        .await;

    Mock::given(method("PATCH"))
        .and(path(format!("/api/me/progress/{}", THE_BOOK)))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&host)
        .await;

    the_row_of_the_disk(THE_BOOK, "", 100.0, 3_600_000);

    let api = a_client(&host.uri());
    let sent = flush_pending_progress(&api, THE_ACCOUNT, THE_SERVER).await;

    assert_eq!(
        sent, 0,
        "a position of the server with no moment gives no comparison, \
         therefore the program sends nothing"
    );

    let the_writes = host
        .received_requests()
        .await
        .unwrap_or_default()
        .into_iter()
        .filter(|request| request.method.as_str() == "PATCH")
        .count();

    assert_eq!(
        the_writes, 0,
        "a moment of 0 is a moment that the server did not give (T-180), \
         and the program must write no position over it"
    );

    assert_eq!(
        get_pending_progress(THE_ACCOUNT, THE_SERVER).unwrap().len(),
        1,
        "the row of the disk waits for a moment that the program can compare"
    );

    // ── The third road: a position of one episode of a podcast. The path of the
    // item alone answers with the position of **another** episode, and the
    // moment of that one is the newest of the account.
    let host = MockServer::start().await;
    let now = toutui::logic::offline::now_ms();

    Mock::given(method("GET"))
        .and(path(format!("/api/me/progress/{}", THE_PODCAST)))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(the_position_of_the_server(999.0, now)),
        )
        .mount(&host)
        .await;

    Mock::given(method("GET"))
        .and(path(format!(
            "/api/me/progress/{}/{}",
            THE_PODCAST, THE_EPISODE
        )))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(the_position_of_the_server(10.0, now - 7_200_000)),
        )
        .mount(&host)
        .await;

    Mock::given(method("PATCH"))
        .and(path(format!(
            "/api/me/progress/{}/{}",
            THE_PODCAST, THE_EPISODE
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&host)
        .await;

    the_row_of_the_disk(THE_PODCAST, THE_EPISODE, 500.0, 3_600_000);

    let api = a_client(&host.uri());
    let sent = flush_pending_progress(&api, THE_ACCOUNT, THE_SERVER).await;

    let the_requests: Vec<String> = host
        .received_requests()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|request| format!("{} {}", request.method.as_str(), request.url.path()))
        .collect();

    assert!(
        the_requests.contains(&format!(
            "GET /api/me/progress/{}/{}",
            THE_PODCAST, THE_EPISODE
        )),
        "the position of an episode stands at the path of that episode (T-182), \
         and the requests are {:?}",
        the_requests
    );

    assert!(
        !the_requests.contains(&format!("GET /api/me/progress/{}", THE_PODCAST)),
        "the path of the item alone holds the position of another episode, \
         and the requests are {:?}",
        the_requests
    );

    assert_eq!(
        sent, 1,
        "the disk holds the newer position of this episode, therefore it goes \
         to the server"
    );

    assert!(
        get_pending_progress(THE_ACCOUNT, THE_SERVER)
            .unwrap()
            .is_empty(),
        "a position that the server took leaves the disk"
    );

    // ── The road of a media that never played stays: the server says `404`, and
    // the program then sends its position.
    let host = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!("/api/me/progress/{}", THE_BOOK)))
        .respond_with(ResponseTemplate::new(404))
        .mount(&host)
        .await;

    Mock::given(method("PATCH"))
        .and(path(format!("/api/me/progress/{}", THE_BOOK)))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&host)
        .await;

    the_row_of_the_disk(THE_BOOK, "", 100.0, 3_600_000);

    let api = a_client(&host.uri());
    let sent = flush_pending_progress(&api, THE_ACCOUNT, THE_SERVER).await;

    assert_eq!(
        sent, 1,
        "a media that never played gives 404, and the position of the disk is \
         then the one position of that media"
    );

    assert!(
        get_pending_progress(THE_ACCOUNT, THE_SERVER)
            .unwrap()
            .is_empty(),
        "a position that the server took leaves the disk"
    );
}
