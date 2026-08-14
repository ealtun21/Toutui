//! A row of a place that the disk did not remove is not a place that waits.
//! See T-211.
//!
//! The flush of the positions sends a place, and it then removes the row of that
//! place. The old code was `let _ = delete_pending_progress(...)`: a disk that
//! takes no write kept the row, and the task of every 30 seconds sent the same
//! place of the same media to the server for the whole life of the program.
//!
//! **The measurement of 2026-08-14**, of the real program of the sandbox, with
//! one row of `pending_progress` of `A Long Test Book` at 900 seconds and
//! `chmod 444` of the database of the account (T-206):
//!
//! ```text
//! [offline] 1 position(s) wait for the server
//! [offline] the server took the position 900s of 9a671047-…
//! [offline] the server answers again. 1 position(s) went to it.
//! [offline] 1 position(s) wait for the server
//! [offline] the server has a newer position of 9a671047-…. The local position goes away.
//! [offline] 1 position(s) wait for the server
//! [offline] the server has a newer position of 9a671047-…. The local position goes away.
//! ```
//!
//! Four attempts in 128 seconds, `SELECT COUNT(*) FROM pending_progress` said
//! **1** at each of them, and the words "The local position goes away" named a
//! removal that never happened. `count_pending_progress` of the header of the
//! offline mode then says that a place of the user waits for a server that holds
//! it already.
//!
//! **This test needs no sandbox.** A host of `wiremock` takes the place, and one
//! `chmod` of the file gives the disk that answers a read and that refuses a
//! write.

use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::db::crud::{get_pending_progress, insert_pending_progress, PendingProgress};
use toutui::logic::offline::flush_pending_progress;
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

const THE_ACCOUNT: &str = "the-account-of-a-disk-that-takes-no-write";
const THE_SERVER: &str = "the-server-of-a-disk-that-takes-no-write";

static HOME: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();

fn temporary_home() -> std::path::PathBuf {
    HOME.get_or_init(|| {
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", dir.path());
        std::fs::create_dir_all(dir.path().join("toutui")).unwrap();

        let conn = toutui::db::migrate::open_conn().unwrap();
        toutui::db::migrate::run_migrations(&conn).unwrap();
        drop(conn);

        dir
    })
    .path()
    .join("toutui/db.sqlite3")
}

fn a_client(url: &str) -> ApiClient {
    let pool = EndpointPool::new(vec![Endpoint::new(url, 0)]);
    ApiClient::new(Arc::new(pool), "test-token".to_string()).unwrap()
}

/// The mode of the file of the database. `444` is a disk that answers every read
/// of the program and that refuses every write of it (T-206).
fn the_disk_takes_a_write(of_the_file: &std::path::Path, takes: bool) {
    let mode = if takes { 0o644 } else { 0o444 };
    std::fs::set_permissions(of_the_file, std::fs::Permissions::from_mode(mode))
        .expect("the mode of the file of the database");
}

fn a_place_that_waits(of_the_media: &str) {
    insert_pending_progress(
        THE_ACCOUNT,
        THE_SERVER,
        &PendingProgress {
            id_item: of_the_media.to_string(),
            id_pod: String::new(),
            current_time: 900.0,
            duration: 1800.0,
            is_finished: false,
            updated_at: toutui::logic::offline::now_ms(),
        },
    )
    .expect("the row of the test");
}

/// A host that holds no place of a media, and that takes every place of the
/// program.
async fn a_host_that_takes_every_place() -> MockServer {
    let host = MockServer::start().await;

    // The status 404 is the media that never played (T-188), therefore the
    // program sends the place of the disk.
    Mock::given(method("GET"))
        .and(path_regex(r"^/api/me/progress/.+$"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&host)
        .await;

    Mock::given(method("PATCH"))
        .and(path_regex(r"^/api/me/progress/.+$"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&host)
        .await;

    host
}

async fn the_writes_of_the_places(host: &MockServer) -> usize {
    host.received_requests()
        .await
        .expect("the requests of the host")
        .iter()
        .filter(|request| request.method == wiremock::http::Method::PATCH)
        .count()
}

/// **The parts of this test stay in one function**: the temporary home of the
/// binary holds one value, and two test functions of one binary take a thread
/// each (T-144 and T-157).
#[tokio::test(flavor = "multi_thread")]
async fn a_place_of_a_row_that_stayed_on_the_disk_goes_to_the_server_one_time() {
    let of_the_file = temporary_home();

    // ── The road of the fault: two places wait, the server takes each of them,
    // and the disk keeps every row.
    a_place_that_waits("the-first-book-of-a-disk-of-no-write");
    a_place_that_waits("the-second-book-of-a-disk-of-no-write");

    let host = a_host_that_takes_every_place().await;
    let api = a_client(&host.uri());

    the_disk_takes_a_write(&of_the_file, false);

    let sent = flush_pending_progress(&api, THE_ACCOUNT, THE_SERVER).await;

    // The disk kept the first row, therefore every other row of this attempt
    // stays as well: a pass that goes on costs the server one request of each
    // waiting media, and it changes nothing at all.
    assert_eq!(
        sent, 1,
        "the flush stops at a disk that took no write: the old code sent every \
         place of the disk, and it removed no row of it. See T-211."
    );

    assert_eq!(
        the_writes_of_the_places(&host).await,
        1,
        "one place of the user reached the server, and the second row waits for \
         a disk that takes a write again"
    );

    the_disk_takes_a_write(&of_the_file, true);

    assert_eq!(
        get_pending_progress(THE_ACCOUNT, THE_SERVER).unwrap().len(),
        2,
        "the disk took no write, therefore the two rows of the places stand \
         still: the count of the header of the offline mode holds that truth"
    );

    // ── The road back: the disk takes a write again, and the flush finishes the
    // work of the two places.
    let host = a_host_that_takes_every_place().await;
    let api = a_client(&host.uri());

    let sent = flush_pending_progress(&api, THE_ACCOUNT, THE_SERVER).await;

    assert_eq!(
        sent, 2,
        "the disk takes a write, therefore the pass goes on"
    );

    assert!(
        get_pending_progress(THE_ACCOUNT, THE_SERVER)
            .unwrap()
            .is_empty(),
        "the server holds the two places, therefore no row of them waits"
    );

    // ── A second attempt of the same account sends nothing at all: that is the
    // condition that the fault of this item took away.
    let sent = flush_pending_progress(&api, THE_ACCOUNT, THE_SERVER).await;

    assert_eq!(sent, 0, "no place of this account waits for the server");

    assert_eq!(
        the_writes_of_the_places(&host).await,
        2,
        "the two places of the user reached the server one time each: the old \
         code sent each of them again at every attempt of the task, for the \
         whole life of the program. See T-211."
    );
}
