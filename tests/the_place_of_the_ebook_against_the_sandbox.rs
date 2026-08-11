//! The place of the ebook goes to the server as an EPUBCFI. See T-10 and T-24.
//!
//! Continuous integration does not run this test, because it needs a server.
//! Start the sandbox of `docs/TEST-SERVER.md`, and then run:
//!
//! ```text
//! ALSA_CONFIG_PATH=/dev/null cargo test --test the_place_of_the_ebook_against_the_sandbox \
//!     -- --ignored --nocapture --test-threads=1
//! ```
//!
//! The test writes `ebookLocation` and `ebookProgress` of the sandbox user. It
//! changes no position of the audio.

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::logic::reader::cfi;
use toutui::logic::reader::session::{get_the_ebook, place_of_the_server};
use toutui::logic::reader::Reader;

const SERVER: &str = "http://127.0.0.1:13399";
const USER: &str = "toutuitest";
const PASSWORD: &str = "toutuitest";
const WIDTH: u16 = 80;

async fn token() -> String {
    let answer: serde_json::Value = reqwest::Client::new()
        .post(format!("{}/login", SERVER))
        .json(&serde_json::json!({ "username": USER, "password": PASSWORD }))
        .send()
        .await
        .expect("the sandbox server must answer")
        .json()
        .await
        .expect("the answer of the login must hold JSON");

    answer["user"]["token"]
        .as_str()
        .expect("the answer must hold a token")
        .to_string()
}

/// The first item of the sandbox that holds an ebook.
async fn item_with_an_ebook(api: &Arc<ApiClient>) -> String {
    let libraries: serde_json::Value = api
        .get_json("/api/libraries")
        .await
        .expect("the server must give the libraries");

    for library in libraries["libraries"].as_array().unwrap_or(&Vec::new()) {
        if library["mediaType"].as_str() != Some("book") {
            continue;
        }
        let id = library["id"].as_str().unwrap_or_default();
        let items: serde_json::Value = api
            .get_json(&format!("/api/libraries/{}/items?limit=50", id))
            .await
            .expect("the server must give the items");
        for item in items["results"].as_array().unwrap_or(&Vec::new()) {
            if item["media"]["ebookFormat"].as_str().is_some() {
                return item["id"].as_str().unwrap_or_default().to_string();
            }
        }
    }
    panic!(
        "the sandbox must hold one item with an ebook. See docs/TEST-SERVER.md, section of T-10."
    );
}

/// Renders the chapter that the reader shows, and waits for the lines.
async fn wait_for_the_lines(reader: &mut Reader) {
    for _ in 0..100 {
        reader.render_for(WIDTH);
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        reader.take_the_answer();
        if !reader.lines.is_empty() {
            return;
        }
    }
    panic!("the chapter gave no line in 5 seconds");
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the sandbox server of docs/TEST-SERVER.md on port 13399"]
async fn the_place_goes_to_the_server_as_an_epubcfi_and_it_comes_back_as_the_same_line() {
    let token = token().await;
    let pool = EndpointPool::new(vec![Endpoint::new(SERVER, 0)]);
    let api = Arc::new(ApiClient::new(Arc::new(pool), token).unwrap());

    let item_id = item_with_an_ebook(&api).await;
    println!("the item with an ebook: {item_id}");

    let path = get_the_ebook(&api, "the-test-of-the-place", &item_id)
        .await
        .expect("the server must give the ebook");
    println!("the book stands at {}", path.display());

    // 1. The reader opens the book and it goes to a line of a real chapter.
    let mut reader = Reader::open(&path, &item_id).expect("the book must open");
    reader.go_to_chapter(2);
    wait_for_the_lines(&mut reader).await;
    let lines = reader.lines.len();
    assert!(lines > 20, "the chapter gave {lines} lines only");
    reader.scroll(40, 20);
    let line_of_the_user = reader.top_line;
    println!("the user stands at the chapter 2, line {line_of_the_user} of {lines}");

    // 2. The place has the form of the web reader.
    let location = reader.location_text();
    println!("the place goes to the server as {location}");
    assert!(
        location.starts_with("epubcfi("),
        "the place must be an EPUBCFI, and it is {location}"
    );
    let read = cfi::parse_epubcfi(&location).expect("the place must be an EPUBCFI");
    assert_eq!(2, read.spine, "the EPUBCFI must name the chapter 2");

    // 3. The server keeps the text and it changes nothing in it.
    let before: serde_json::Value = api
        .get_json(&format!("/api/me/progress/{}", item_id))
        .await
        .unwrap_or(serde_json::json!({}));
    let time_of_the_audio = before.get("currentTime").cloned();

    api.patch_json(
        &format!("/api/me/progress/{}", item_id),
        &serde_json::json!({
            "ebookLocation": location,
            "ebookProgress": reader.fraction(),
        }),
    )
    .await
    .expect("the server must take the place");

    let (from_the_server, part) = place_of_the_server(&api, &item_id)
        .await
        .expect("the server must give the place back");
    println!("the server gives {from_the_server} back, and the part {part}");
    assert_eq!(
        location, from_the_server,
        "the server must change nothing in the text"
    );

    let after: serde_json::Value = api
        .get_json(&format!("/api/me/progress/{}", item_id))
        .await
        .expect("the server must give the position");
    assert_eq!(
        time_of_the_audio,
        after.get("currentTime").cloned(),
        "the place of the ebook must not move the position of the audio"
    );

    // 4. A second reader takes that text and it opens the same line.
    let mut again = Reader::open(&path, &item_id).expect("the book must open a second time");
    again.go_to_the_place_of_the_server(&from_the_server, part);
    assert_eq!(2, again.chapter, "the reader must open the chapter 2");
    wait_for_the_lines(&mut again).await;
    println!("the second reader stands at the line {}", again.top_line);
    assert_eq!(
        line_of_the_user, again.top_line,
        "the second reader must open the same line"
    );
}
