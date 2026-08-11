//! Every session of the user comes in pages. See T-24.
//!
//! Continuous integration does not run this test, because it needs a server.
//! Start the sandbox of `docs/TEST-SERVER.md`, and then run:
//!
//! ```text
//! ALSA_CONFIG_PATH=/dev/null cargo test --test the_sessions_against_the_sandbox \
//!     -- --ignored --nocapture --test-threads=1
//! ```
//!
//! The test reads only. It changes nothing on the server.
//!
//! The test of the pages needs more sessions than one page. The measurement of
//! 2026-08-11 made 36 sessions of the sandbox with `POST /api/items/:id/play`
//! and `POST /api/session/:id/close`. A sandbox with fewer sessions gives one
//! page, and the test then measures the first page only.

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::api::me::sessions::{get_sessions, there_is_more, PER_PAGE};
use toutui::logic::sessions_view::{Loaded, State};
use toutui::ui::sessions_tui::lines;

const SERVER: &str = "http://127.0.0.1:13399";

mod common;
use common::token;

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the sandbox server of docs/TEST-SERVER.md on port 13399"]
async fn the_pages_come_one_after_the_other_and_the_screen_reads_them() {
    let token = token().await;
    let pool = EndpointPool::new(vec![Endpoint::new(SERVER, 0)]);
    let api = Arc::new(ApiClient::new(Arc::new(pool), token).unwrap());

    let first = get_sessions(&api, 0, PER_PAGE)
        .await
        .expect("the server must give the first page");
    println!(
        "the page 0: {} sessions of {}, {} pages",
        first.sessions.len(),
        first.total,
        first.num_pages
    );
    assert!(first.total > 0, "the sandbox must hold a session");
    assert_eq!(0, first.page, "the first page is the page 0");
    assert_eq!(PER_PAGE, first.items_per_page);

    let mut loaded = Loaded::first(first.clone());
    assert_eq!(loaded.more, there_is_more(loaded.sessions.len(), &first));

    // The program reads every page, and it never asks for ever. The limit of
    // 100 stops a server that gives a page that is always full.
    let mut reads = 0;
    while loaded.more && reads < 100 {
        reads += 1;
        let next = get_sessions(&api, loaded.page + 1, PER_PAGE)
            .await
            .unwrap_or_else(|error| panic!("the page {} must come: {error}", loaded.page + 1));
        println!("the page {}: {} sessions", next.page, next.sessions.len());
        loaded.add(next);
    }

    println!("the program holds {} sessions", loaded.sessions.len());
    assert_eq!(
        loaded.total,
        loaded.sessions.len(),
        "the program must hold every session"
    );
    assert!(!loaded.more, "the reads must stop at the last page");

    // A page after the last page gives 200 and an empty list, and not an
    // error. The program must stop there.
    let after = get_sessions(&api, loaded.page + 50, PER_PAGE)
        .await
        .expect("a page after the last page must give an answer");
    assert!(
        after.sessions.is_empty(),
        "a page after the last page must give no session"
    );
    assert!(!there_is_more(0, &after));

    // The screen reads the answer of a real server.
    let text = lines(&State::Ready(Box::new(loaded.clone())), 80)
        .iter()
        .map(|line| {
            line.spans
                .iter()
                .map(|span| span.content.as_ref())
                .collect::<String>()
        })
        .collect::<Vec<String>>()
        .join("\n");

    println!("{}", text.lines().take(8).collect::<Vec<&str>>().join("\n"));

    assert!(text.contains(&format!(
        "{} sessions of {}",
        loaded.sessions.len(),
        loaded.total
    )));
    // Every session of the sandbox carries a date, therefore the screen must
    // hold a heading of a day and no message of a date that is absent.
    assert!(!text.contains("A session with no date"));
    let title = loaded.sessions[0].title();
    assert!(text.contains(&title), "the screen must name {title}");
}
