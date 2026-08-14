//! A book that lost a file on the server loses that file on the disk. See
//! T-187.
//!
//! **The measurement of 2026-08-14, against the sandbox.** The account
//! `toutuitest` held the whole download of `Multi File Test Book` of three
//! files of 20 seconds. The two files `02 - Part 2.mp3` and `03 - Part 3.mp3`
//! then left the library of the server, and `POST /api/items/:id/scan` gave the
//! book of **one** audio file of 20 seconds. The key `D` of the program after
//! it said:
//!
//! ```text
//! "Multi File Test Book" is now available offline.
//! ```
//!
//! and the disk held **three** files, and `download_files` held **three** rows.
//! The row of `downloads` held the new length of 20 seconds beside them. The
//! server then went away, and the offline playback of the same media said:
//!
//! ```text
//! [play] the offline mode plays Multi File Test Book at 0 seconds with 3 track(s)
//! [follow_playback_offline] the playback stopped at 60 seconds, finished=true
//! [offline] the position 60s of ac365248-… waits for the server
//! ```
//!
//! **The user heard two parts that the book does not hold**, and the program
//! wrote the place 60 seconds of a book of 20 seconds for the server.
//!
//! **The parts of this test stay in one function.** It writes
//! `XDG_CONFIG_HOME` and `XDG_DATA_HOME`, and those are boxes of the process:
//! two test functions of one binary would fight for them (the shape of T-144
//! and of T-157).

use toutui::logic::download::{download_with_progress, downloads_base_dir, DownloadTarget};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// The identity of the item of this test.
const ITEM: &str = "item-1";

/// The name of the account of this test.
const WHO: &str = "a-user";

/// Makes the answer of `GET /api/items/:id` of a book of `how_many` files.
fn the_book_of(how_many: u32) -> serde_json::Value {
    let files: Vec<serde_json::Value> = (1..=how_many)
        .map(|number| {
            serde_json::json!({
                "index": number,
                "ino": (100 + number).to_string(),
                "duration": 20.0,
                "metadata": { "filename": format!("0{number} - Part {number}.mp3"), "size": 40u64 }
            })
        })
        .collect();

    serde_json::json!({
        "id": ITEM,
        "media": {
            "metadata": { "title": "Multi File Test Book", "authorName": "Test Author" },
            "audioFiles": files
        }
    })
}

/// Gives the names of the files of the directory of the download.
fn the_names_of_the_disk(directory: &std::path::Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(directory)
        .expect("the directory of the download")
        .flatten()
        .map(|row| row.file_name().to_string_lossy().to_string())
        // The lock of T-148 goes away with the download that took it.
        .filter(|name| !name.starts_with('.'))
        .collect();

    names.sort();
    names
}

#[tokio::test(flavor = "multi_thread")]
async fn a_book_that_lost_a_file_loses_it_on_the_disk() {
    let home = tempfile::tempdir().expect("the directory of the test");
    std::env::set_var("XDG_CONFIG_HOME", home.path());
    std::env::set_var("XDG_DATA_HOME", home.path());

    std::fs::create_dir_all(home.path().join("toutui")).expect("the directory of the database");

    let conn = toutui::db::migrate::open_conn().expect("the database of the test");
    toutui::db::migrate::run_migrations(&conn).expect("the tables of the test");
    drop(conn);

    let server = MockServer::start().await;

    // Every file of the book holds the same 40 bytes. This test measures the
    // files of the disk, and not their bytes.
    Mock::given(method("GET"))
        .and(path(format!("/api/items/{ITEM}/file/101/download")))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![b'a'; 40]))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/api/items/{ITEM}/file/102/download")))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![b'b'; 40]))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/api/items/{ITEM}/file/103/download")))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![b'c'; 40]))
        .mount(&server)
        .await;

    let the_three_files = Mock::given(method("GET"))
        .and(path(format!("/api/items/{ITEM}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(the_book_of(3)))
        .mount_as_scoped(&server)
        .await;

    let target = || DownloadTarget::Book {
        item_id: ITEM.to_string(),
    };

    download_with_progress(
        Some("secret".to_string()),
        target(),
        server.uri(),
        WHO.to_string(),
        "Multi File Test Book".to_string(),
        "Test Author".to_string(),
        "a-server".to_string(),
        toutui::logic::download::new_progress_map(),
    )
    .await;

    let directory = downloads_base_dir(WHO).join(ITEM);

    assert_eq!(
        the_names_of_the_disk(&directory),
        vec![
            "001 - 01 - Part 1.mp3".to_string(),
            "002 - 02 - Part 2.mp3".to_string(),
            "003 - 03 - Part 3.mp3".to_string(),
        ],
        "the disk holds the whole book of the server"
    );

    let tracks = toutui::logic::offline::tracks_from_downloads(ITEM, WHO)
        .expect("the offline mode holds the book");
    assert_eq!(tracks.len(), 3, "the book of the server holds three files");
    assert_eq!(tracks.total_duration(), 60.0);

    // **The book of the server loses two of its files.**
    drop(the_three_files);
    Mock::given(method("GET"))
        .and(path(format!("/api/items/{ITEM}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(the_book_of(1)))
        .mount(&server)
        .await;

    download_with_progress(
        Some("secret".to_string()),
        target(),
        server.uri(),
        WHO.to_string(),
        "Multi File Test Book".to_string(),
        "Test Author".to_string(),
        "a-server".to_string(),
        toutui::logic::download::new_progress_map(),
    )
    .await;

    assert_eq!(
        the_names_of_the_disk(&directory),
        vec!["001 - 01 - Part 1.mp3".to_string()],
        "the file of a book that the server no longer holds leaves the disk"
    );

    let tracks = toutui::logic::offline::tracks_from_downloads(ITEM, WHO)
        .expect("the offline mode holds the book");
    assert_eq!(
        tracks.len(),
        1,
        "the offline playback plays no part that the book no longer holds"
    );
    assert_eq!(
        tracks.total_duration(),
        20.0,
        "the length of the disk is the length of the book of the server"
    );
}
