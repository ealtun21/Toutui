//! Tests of the pagination of a library.
//!
//! The old code asked for every item in one request. A library with 10000
//! books then made a very large answer. See upstream issue 35.

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::api::libraries::get_all_books::{get_all_books, PAGE_SIZE};
use toutui::api::libraries::get_all_series::get_all_series;
use toutui::api::utils::collect_series::collect_series;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(url: &str) -> ApiClient {
    let pool = EndpointPool::new(vec![Endpoint::new(url, 0)]);
    ApiClient::new(Arc::new(pool), "test-token".to_string()).unwrap()
}

/// Makes a page of items. Each item has a name that shows its number.
fn page(first: usize, count: usize, total: i64) -> serde_json::Value {
    let results: Vec<serde_json::Value> = (first..first + count)
        .map(|number| {
            serde_json::json!({
                "id": format!("item-{}", number),
                "media": { "metadata": { "title": format!("Book {}", number) } }
            })
        })
        .collect();

    serde_json::json!({ "results": results, "total": total })
}

/// A library that has fewer items than one page needs one request.
#[tokio::test]
async fn a_small_library_needs_one_request() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/libraries/lib1/items"))
        .and(query_param("page", "0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(page(0, 12, 12)))
        .expect(1)
        .mount(&server)
        .await;

    let root = get_all_books(&client(&server.uri()), "lib1").await.unwrap();

    assert_eq!(root.results.unwrap().len(), 12);
    drop(server);
}

/// The request must hold the limit and the page. It must not hold `limit=0`.
#[tokio::test]
async fn the_request_asks_for_one_page() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/libraries/lib1/items"))
        .and(query_param("limit", PAGE_SIZE.to_string().as_str()))
        .and(query_param("page", "0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(page(0, 3, 3)))
        .expect(1)
        .mount(&server)
        .await;

    let root = get_all_books(&client(&server.uri()), "lib1").await.unwrap();

    assert_eq!(root.results.unwrap().len(), 3);
    drop(server);
}

/// A library that is larger than one page needs more than one request, and
/// the function gives every item together.
#[tokio::test]
async fn a_large_library_gives_every_item() {
    let server = MockServer::start().await;
    let total = PAGE_SIZE * 2 + 56;

    for number in 0..3 {
        let first = (number * PAGE_SIZE) as usize;
        let count = if number == 2 { 56 } else { PAGE_SIZE as usize };

        Mock::given(method("GET"))
            .and(path("/api/libraries/lib1/items"))
            .and(query_param("page", number.to_string().as_str()))
            .respond_with(ResponseTemplate::new(200).set_body_json(page(first, count, total)))
            .expect(1)
            .mount(&server)
            .await;
    }

    let root = get_all_books(&client(&server.uri()), "lib1").await.unwrap();
    let items = root.results.unwrap();

    assert_eq!(items.len(), total as usize);

    // The sequence must stay correct over the boundary of a page.
    assert_eq!(items[0].id.as_deref(), Some("item-0"));
    assert_eq!(
        items[PAGE_SIZE as usize].id.as_deref(),
        Some(format!("item-{}", PAGE_SIZE).as_str())
    );
    assert_eq!(
        items[total as usize - 1].id.as_deref(),
        Some(format!("item-{}", total - 1).as_str())
    );

    drop(server);
}

/// A page that is full and a total that agrees stop the loop. The function
/// must not ask for a page that does not exist.
#[tokio::test]
async fn a_library_of_exactly_one_page_needs_one_request() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/libraries/lib1/items"))
        .and(query_param("page", "0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(page(
            0,
            PAGE_SIZE as usize,
            PAGE_SIZE,
        )))
        .expect(1)
        .mount(&server)
        .await;

    let root = get_all_books(&client(&server.uri()), "lib1").await.unwrap();

    assert_eq!(root.results.unwrap().len(), PAGE_SIZE as usize);
    drop(server);
}

/// A library with no item gives an empty list, and not an error.
#[tokio::test]
async fn an_empty_library_gives_an_empty_list() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/libraries/lib1/items"))
        .respond_with(ResponseTemplate::new(200).set_body_json(page(0, 0, 0)))
        .mount(&server)
        .await;

    let root = get_all_books(&client(&server.uri()), "lib1").await.unwrap();

    assert_eq!(root.results.unwrap().len(), 0);
}

/// Makes a page of series. Each series holds two books.
fn series_page(first: usize, count: usize, total: i64) -> serde_json::Value {
    let results: Vec<serde_json::Value> = (first..first + count)
        .map(|number| {
            serde_json::json!({
                "id": format!("series-{}", number),
                "name": format!("Series {}", number),
                "books": [
                    { "id": format!("book-{}-2", number), "media": { "metadata":
                        { "title": "Second", "seriesName": format!("Series {} #2", number) } } },
                    { "id": format!("book-{}-1", number), "media": { "metadata":
                        { "title": "First", "seriesName": format!("Series {} #1", number) } } }
                ]
            })
        })
        .collect();

    serde_json::json!({ "results": results, "total": total })
}

/// The endpoint of the series gives an empty list for `limit=0`. Therefore the
/// request must always hold a limit that is not zero, and a page.
#[tokio::test]
async fn the_request_for_the_series_asks_for_one_page() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/libraries/lib1/series"))
        .and(query_param("limit", PAGE_SIZE.to_string().as_str()))
        .and(query_param("page", "0"))
        .and(query_param("sort", "name"))
        .respond_with(ResponseTemplate::new(200).set_body_json(series_page(0, 2, 2)))
        .expect(1)
        .mount(&server)
        .await;

    let root = get_all_series(&client(&server.uri()), "lib1")
        .await
        .unwrap();

    assert_eq!(root.results.unwrap().len(), 2);
    drop(server);
}

/// A library with many series needs more than one request.
#[tokio::test]
async fn a_library_with_many_series_gives_every_series() {
    let server = MockServer::start().await;
    let total = PAGE_SIZE + 7;

    for number in 0..2 {
        let first = (number * PAGE_SIZE) as usize;
        let count = if number == 1 { 7 } else { PAGE_SIZE as usize };

        Mock::given(method("GET"))
            .and(path("/api/libraries/lib1/series"))
            .and(query_param("page", number.to_string().as_str()))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(series_page(first, count, total)),
            )
            .expect(1)
            .mount(&server)
            .await;
    }

    let root = get_all_series(&client(&server.uri()), "lib1")
        .await
        .unwrap();

    assert_eq!(root.results.unwrap().len(), total as usize);
    drop(server);
}

/// A library with no series gives an empty list, and not an error.
#[tokio::test]
async fn a_library_with_no_series_gives_an_empty_list() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/libraries/lib1/series"))
        .respond_with(ResponseTemplate::new(200).set_body_json(series_page(0, 0, 0)))
        .mount(&server)
        .await;

    let root = get_all_series(&client(&server.uri()), "lib1")
        .await
        .unwrap();

    assert_eq!(root.results.unwrap().len(), 0);
}

/// The display data must come in the sequence of the series.
#[tokio::test]
async fn the_books_of_a_series_come_in_sequence() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/libraries/lib1/series"))
        .respond_with(ResponseTemplate::new(200).set_body_json(series_page(0, 1, 1)))
        .mount(&server)
        .await;

    let root = get_all_series(&client(&server.uri()), "lib1")
        .await
        .unwrap();
    let series = collect_series(&root);

    assert_eq!(series.len(), 1);
    assert_eq!(series[0].line(), "Series 0 [2 books]");
    assert_eq!(series[0].books[0].line(), "#1 - First");
    assert_eq!(series[0].books[1].line(), "#2 - Second");
}
