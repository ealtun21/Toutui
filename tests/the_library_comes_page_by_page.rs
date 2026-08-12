//! The program reads the library page by page. See T-70.
//!
//! `get_all_books` read **every** page of the library before the first frame.
//! A page holds 500 items, therefore a library of 2056 items made five requests
//! and a library of 250000 items made 500 of them: the cost of the start grew
//! with the library of the user.
//!
//! This test gives the mock server a library of 2056 items, and it counts the
//! requests of `/api/libraries/:id/items`. **The start makes one request now**,
//! and the program asks for the page after it when the user comes near the end
//! of the lines that it holds.
//!
//! The test uses a mock server of `wiremock`. It needs no network and no
//! sandbox, therefore continuous integration runs it.
//!
//! The test writes `XDG_CONFIG_HOME`, therefore it stands alone in its binary.
//! See the trap 8 of the harness.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::app::App;
use toutui::db::database_struct::User;
use wiremock::matchers::{method, path_regex, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// The number of items of the library of this test.
const TOTAL: usize = 2056;

/// The number of items of one page. `get_all_books::PAGE_SIZE` holds it.
const PAGE: usize = 500;

/// Counts the requests of one path.
#[derive(Clone, Default)]
struct Counts(Arc<Mutex<usize>>);

impl Counts {
    fn value(&self) -> usize {
        *self.0.lock().expect("the count")
    }
}

impl wiremock::Match for Counts {
    fn matches(&self, _request: &wiremock::Request) -> bool {
        *self.0.lock().expect("the count") += 1;
        true
    }
}

/// Makes the answer of one page of the library.
fn a_page(number: usize) -> serde_json::Value {
    let first = number * PAGE;
    let last = ((number + 1) * PAGE).min(TOTAL);

    let results: Vec<serde_json::Value> = (first..last)
        .map(|item| {
            serde_json::json!({
                "id": format!("item-{}", item),
                "mediaType": "book",
                "media": {
                    "metadata": {
                        "title": format!("Book {}", item),
                        "authorName": "Test Author",
                        "description": "A book of the test.",
                        "publishedYear": "2026"
                    },
                    "duration": 60.0
                }
            })
        })
        .collect();

    serde_json::json!({
        "results": results,
        "total": TOTAL,
        "limit": PAGE,
        "page": number,
    })
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
async fn the_start_reads_one_page_and_the_user_gets_the_next_one() {
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

    Mock::given(method("GET"))
        .and(wiremock::matchers::path("/api/libraries"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(include_str!("data/one_library.json")),
        )
        .mount(&server)
        .await;

    let of_the_items = Counts::default();

    for number in 0..5 {
        Mock::given(method("GET"))
            .and(path_regex(r"^/api/libraries/.*/items$"))
            .and(query_param("page", number.to_string()))
            .and(of_the_items.clone())
            .respond_with(ResponseTemplate::new(200).set_body_json(a_page(number)))
            .mount(&server)
            .await;
    }

    // Every other answer of the start is empty. This test measures the pages of
    // the library only.
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&server)
        .await;

    toutui::db::crud::db_insert_usr(&vec![a_user(&server.uri())]).unwrap();

    let pool = EndpointPool::new(vec![Endpoint::new(&server.uri(), 0)]);
    let api = Arc::new(ApiClient::new(Arc::new(pool), "token".to_string()).unwrap());

    let mut app = App::new(Arc::clone(&api)).await.expect("an application");

    assert!(!app.is_offline, "the mock server answered");

    // **The measurement.** The old code asked for the five pages of 2056 items
    // before the first frame.
    assert_eq!(
        of_the_items.value(),
        1,
        "the start must read one page of the library, and it read {}",
        of_the_items.value()
    );

    assert_eq!(app.ids_library.len(), PAGE);
    assert_eq!(app.library_total, TOTAL);
    assert_eq!(
        app.titles_library.first().map(String::as_str),
        Some("Book 0")
    );

    // The user stands at the first line. The program holds 500 items, therefore
    // it needs no page.
    app.list_state_library.select(Some(0));
    app.ask_for_the_next_page_of_the_library();
    assert_eq!(of_the_items.value(), 1, "a user at the first line waits");

    // The user comes near the end of the lines that the program holds.
    app.list_state_library.select(Some(PAGE - 1));
    app.ask_for_the_next_page_of_the_library();

    // The task asks the server, and the render takes the page at the next
    // frame. A poll gives the answer as soon as it comes.
    let started = Instant::now();

    while app.ids_library.len() == PAGE {
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "the page of the library did not come"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
        app.take_the_next_page_of_the_library();
    }

    assert_eq!(of_the_items.value(), 2, "the program asks for one page");
    assert_eq!(app.ids_library.len(), PAGE * 2);
    assert_eq!(app.library_page, 1);

    // Every list of the library grew together. A list that stays short gives a
    // wrong author or a wrong length on the line of a book.
    assert_eq!(app.titles_library.len(), PAGE * 2);
    assert_eq!(app.auth_names_library.len(), PAGE * 2);
    assert_eq!(app.duration_library.len(), PAGE * 2);
    assert_eq!(app.desc_library.len(), PAGE * 2);
    assert_eq!(app.published_year_library.len(), PAGE * 2);

    // The lines of the pages before this one did not move: the library holds no
    // series in this test, therefore one item gives one line.
    assert_eq!(app.library_rows.len(), PAGE * 2);
    assert_eq!(
        app.titles_library.get(PAGE).map(String::as_str),
        Some("Book 500")
    );
}
