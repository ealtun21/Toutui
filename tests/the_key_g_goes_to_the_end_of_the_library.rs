//! The key `G` goes to the end of the library, and not to the end of the page.
//! See T-112.
//!
//! The program reads one page of 500 items at the start (T-70). **The end of the
//! lines that the program holds is not the end of the library**: a sweep of a
//! library of 2056 items on 2026-08-12 pressed `G` at the first frame, and the
//! screen took the item 500 of 2056. The user had to press that key **six**
//! times to reach the last item.
//!
//! This test gives the mock server a library of 2056 items. It presses `G` one
//! time, and the program must then hold every item and stand at the last line.
//!
//! The test uses a mock server of `wiremock`. It needs no network and no
//! sandbox, therefore continuous integration runs it.
//!
//! The test writes `XDG_CONFIG_HOME`, therefore it stands alone in its binary.
//! See the trap 8 of the harness.

use std::sync::Arc;
use std::time::{Duration, Instant};
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::app::{App, AppView};
use toutui::db::database_struct::User;
use wiremock::matchers::{method, path_regex, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// The number of items of the library of this test.
const TOTAL: usize = 2056;

/// The number of items of one page. `get_all_books::PAGE_SIZE` holds it.
const PAGE: usize = 500;

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
async fn one_press_of_the_key_of_the_end_reaches_the_last_item_of_the_library() {
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

    for number in 0..5 {
        Mock::given(method("GET"))
            .and(path_regex(r"^/api/libraries/.*/items$"))
            .and(query_param("page", number.to_string()))
            .respond_with(ResponseTemplate::new(200).set_body_json(a_page(number)))
            .mount(&server)
            .await;
    }

    // Every other answer of the start is empty. This test measures the lines of
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
    assert_eq!(app.ids_library.len(), PAGE, "the start reads one page");
    assert_eq!(app.library_total, TOTAL);

    app.view_state = AppView::Library;
    app.list_state_library.select(Some(0));

    // **The measurement.** The key `G` of the user. The old code took the line
    // 500 of 2056 and it waited for a second press of the same key.
    app.select_last();

    // The render takes the page that came, and the program then asks for the
    // page after it. A poll gives the answer as soon as it comes.
    let started = Instant::now();

    while app.ids_library.len() < TOTAL {
        assert!(
            started.elapsed() < Duration::from_secs(20),
            "the program holds {} items of {}, and the key G asked for the end",
            app.ids_library.len(),
            TOTAL
        );

        tokio::time::sleep(Duration::from_millis(20)).await;
        app.take_the_next_page_of_the_library();
    }

    assert_eq!(
        app.library_rows.len(),
        TOTAL,
        "one item gives one line here"
    );
    assert_eq!(
        app.list_state_library.selected(),
        Some(TOTAL - 1),
        "the key G stands at the last item of the library"
    );
    assert_eq!(
        app.titles_library.last().map(String::as_str),
        Some("Book 2055")
    );

    // The wait is over, therefore a page of a library that comes later moves no
    // line of the user.
    assert!(!app.reads_every_page_of_the_library);

    // The key `g` gives the first line back, and it needs no request.
    app.select_first();
    assert_eq!(app.list_state_library.selected(), Some(0));
}
