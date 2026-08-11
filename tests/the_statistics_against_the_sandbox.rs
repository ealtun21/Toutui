//! The statistics of the user come from the server. See T-24.
//!
//! Continuous integration does not run this test, because it needs a server.
//! Start the sandbox of `docs/TEST-SERVER.md`, and then run:
//!
//! ```text
//! ALSA_CONFIG_PATH=/dev/null cargo test --test the_statistics_against_the_sandbox \
//!     -- --ignored --nocapture --test-threads=1
//! ```
//!
//! The test reads only. It changes nothing on the server.

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::api::me::listening_stats::{get_listening_stats, human_time, top_items, week};
use toutui::api::stats::{get_library_stats, get_year_stats, human_size, this_year};
use toutui::logic::stats::State;
use toutui::ui::stats_tui::lines;

const SERVER: &str = "http://127.0.0.1:13399";

mod common;
use common::token;

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the sandbox server of docs/TEST-SERVER.md on port 13399"]
async fn the_server_gives_the_statistics_and_the_screen_reads_them() {
    let token = token().await;
    let pool = EndpointPool::new(vec![Endpoint::new(SERVER, 0)]);
    let api = Arc::new(ApiClient::new(Arc::new(pool), token).unwrap());

    let stats = get_listening_stats(&api)
        .await
        .expect("the server must give the statistics");

    println!(
        "in total {}, today {}, {} days, {} media, {} sessions",
        human_time(stats.total_time),
        human_time(stats.today),
        stats.days.len(),
        stats.items.len(),
        stats.recent_sessions.len()
    );

    // The sandbox played media. Therefore the answer holds a time, and it
    // holds a media.
    assert!(
        stats.total_time > 0.0,
        "the sandbox must hold a time. Play a media of docs/TEST-SERVER.md first."
    );
    assert!(!stats.items.is_empty(), "the answer must name a media");
    assert_eq!(week(&stats).len(), 7);

    let top = top_items(&stats, 5);
    assert!(!top.is_empty(), "the list of the media must hold a line");
    println!("the media of the largest time: {}", top[0].title);

    // The screen reads the answer of a real server.
    let text = lines(
        &State::Ready(Box::new(toutui::logic::stats::Statistics {
            listening: stats,
            year_number: 2026,
            ..Default::default()
        })),
        80,
    )
    .iter()
    .map(|line| {
        line.spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>()
    })
    .collect::<Vec<String>>()
    .join("\n");

    println!("{}", text);

    assert!(text.contains("In total:"));
    assert!(text.contains("The days of the week"));
    assert!(text.contains(&top[0].title));
}

/// The two groups that v0.7.9 added. See T-24, section 5.
///
/// The measurement of 2026-08-11 needed a book with a genre and a narrator, and
/// a session of that book after the metadata came. The server keeps a copy of
/// the metadata inside each session, therefore an older session gives an empty
/// list of the genres.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the sandbox server of docs/TEST-SERVER.md on port 13399"]
async fn the_server_gives_the_numbers_of_the_library_and_of_the_year() {
    let token = token().await;
    let pool = EndpointPool::new(vec![Endpoint::new(SERVER, 0)]);
    let api = Arc::new(ApiClient::new(Arc::new(pool), token).unwrap());

    let libraries: serde_json::Value = api
        .get_json("/api/libraries")
        .await
        .expect("the server must give the libraries");
    let library = libraries["libraries"]
        .as_array()
        .and_then(|all| all.iter().find(|one| one["mediaType"] == "book"))
        .expect("the sandbox must hold a library of books");
    let id = library["id"].as_str().expect("a library must have an id");
    let name = library["name"].as_str().unwrap_or_default();

    let stats = get_library_stats(&api, id)
        .await
        .expect("the server must give the numbers of the library");
    println!(
        "the library {name}: {} items, {} tracks, {}, {}",
        stats.total_items,
        stats.num_audio_tracks,
        human_size(stats.total_size),
        human_time(stats.total_duration)
    );
    assert!(stats.total_items > 0, "the sandbox must hold an item");
    assert!(stats.total_size > 0.0);
    assert!(
        !stats.longest_items.is_empty(),
        "the answer must name the longest item"
    );
    assert!(!stats.largest_items.is_empty());
    println!("the longest item: {}", stats.longest_items[0].name());

    let year = this_year();
    let of_the_year = get_year_stats(&api, year)
        .await
        .expect("the server must give the numbers of the year");
    println!(
        "the year {year}: {} in {} sessions, {} books came",
        human_time(of_the_year.total_listening_time),
        of_the_year.num_listening_sessions,
        of_the_year.num_books_added
    );
    assert!(of_the_year.num_books_added > 0);
    assert!(
        !of_the_year.top_authors.is_empty(),
        "the sandbox played a book, therefore the year must name an author"
    );
    println!(
        "the author of the year: {}",
        of_the_year.top_authors[0].label()
    );

    // The list of the genres names its value `genre` on the server, and the
    // two other lists name it `name`. A reader that takes `name` only would
    // give "No name" here.
    for genre in &of_the_year.top_genres {
        println!(
            "a genre of the year: {} ({})",
            genre.label(),
            human_time(genre.time)
        );
        assert_ne!(
            "No name",
            genre.label(),
            "the reader must take the key `genre` of the server"
        );
    }
    for narrator in &of_the_year.top_narrators {
        assert_ne!("No name", narrator.label());
    }
}
