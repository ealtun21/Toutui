//! The rows of a download go away together, or they stay together. See T-214.
//!
//! **`delete_download` held two statements and no transaction.** The first one
//! removes the row of the media of `downloads`, and the second one removes the
//! rows of the files of `download_files`: rusqlite writes each statement of its
//! own, therefore a disk that refused the second one **kept** the first one.
//!
//! **The measurement of 2026-08-14** of the real program of the sandbox, with one
//! trigger of SQLite that fails one write of one row (T-213):
//!
//! ```bash
//! sqlite3 "$DB" "CREATE TRIGGER the_disk_takes_no_row_of_a_file \
//!     BEFORE DELETE ON download_files \
//!     BEGIN SELECT RAISE(ABORT, 'the disk takes no removal of the row of a file'); END;"
//! ```
//!
//! The key `X` of `The Test Chronicles Volume 3`, a download of 24648 bytes:
//!
//! ```text
//! [ERROR] [remove_download] the 24648 bytes of the download 040e9d69-… went away,
//!     and its rows of the database stay: the disk takes no removal of the row of a file
//! ```
//!
//! ```text
//!    The program removed the files of "The Test Chronicles Volume 3". Its
//!    database keeps the rows of that download. Press X again.
//! ```
//!
//! `SELECT COUNT(*) FROM downloads` said **0** and
//! `SELECT COUNT(*) FROM download_files` said **1**: the words named the rows of
//! that download, and one row of the two of them went away. **The key `X` again
//! then said `"The Test Chronicles Volume 3" holds no local copy and no ebook.`**,
//! because `remove_download` reads the row of `downloads` to find the work: the
//! row of the file stayed for ever, and the program contradicted itself in two
//! seconds (the shape of T-206).
//!
//! **The row that stayed took the disk of a media that holds no file at all.**
//! `select_sources` reads `download_files` alone, and every playback of that book
//! after the removal took the road of the disk:
//!
//! ```text
//! [INFO] [play] the download 040e9d69-… gives 1 of 1 track(s) from the disk
//! [ERROR] [worker] the engine cannot start the book: The application cannot open
//!     the file: No such file or directory (os error 2)
//! [INFO] [play] no decoder of the program reads book.mp3. The program asks the
//!     server for a stream of the whole media.
//! ```
//!
//! **This test needs no sandbox.** One trigger of SQLite gives a table of the
//! disk that takes no removal, and every other read and write of the program
//! answers (T-213).

use rusqlite::params;
use toutui::app::AppView;
use toutui::db::crud::{delete_download, get_download, get_download_files, insert_download};
use toutui::logic::download::{
    download_with_progress, the_words_of_a_download_whose_rows_stay, DownloadTarget,
};
use toutui::logic::message;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const THE_ACCOUNT: &str = "the-account-of-a-removal-of-two-tables";
const THE_SERVER: &str = "the-server-of-a-removal-of-two-tables";
const THE_BOOK: &str = "the-book-of-a-removal-of-two-tables";

static HOME: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();

fn temporary_home() {
    HOME.get_or_init(|| {
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", dir.path());
        std::env::set_var("XDG_DATA_HOME", dir.path());
        std::fs::create_dir_all(dir.path().join("toutui")).unwrap();

        let conn = toutui::db::migrate::open_conn().unwrap();
        toutui::db::migrate::run_migrations(&conn).unwrap();
        drop(conn);

        dir
    });
}

/// The rows of a whole download: the media, and the two files of it.
fn the_rows_of_a_whole_download() {
    insert_download(
        THE_BOOK,
        THE_ACCOUNT,
        "A Book Of Two Files",
        "An Author",
        "/the/first/file.mp3",
        1800.0,
        THE_BOOK,
        THE_SERVER,
    )
    .expect("the row of the download");

    for (index, path) in [(1, "/the/first/file.mp3"), (2, "/the/second/file.mp3")] {
        toutui::db::crud::insert_download_file(
            THE_BOOK,
            THE_ACCOUNT,
            index,
            &format!("ino-{index}"),
            path,
            100,
            900.0,
        )
        .expect("the row of the file");
    }
}

/// A table of the disk that takes no statement of one shape, and every other read
/// and write of the program answers. This is the harness of T-213.
fn the_disk_refuses(what: &str, table: &str, install: bool) {
    let conn = toutui::db::migrate::open_conn().expect("the database of the test");

    let name = format!("the_disk_takes_no_{what}_of_{table}");

    let statement = if install {
        format!(
            "CREATE TRIGGER {name} BEFORE {what} ON {table} \
             BEGIN SELECT RAISE(ABORT, 'the disk takes no {what} of a row of {table}'); END;"
        )
    } else {
        format!("DROP TRIGGER {name};")
    };

    conn.execute_batch(&statement).expect("the trigger");
}

/// The answer of `GET /api/items/:id` of a book of one file.
fn the_book_of_one_file() -> serde_json::Value {
    serde_json::json!({
        "id": THE_BOOK,
        "media": {
            "metadata": { "title": "A Book Of Two Files", "authorName": "An Author" },
            "audioFiles": [{
                "index": 1,
                "ino": "101",
                "duration": 20.0,
                "metadata": { "filename": "01 - Part 1.mp3", "size": 40u64 }
            }]
        }
    })
}

/// The key `D` of the user, against a host of `wiremock`.
async fn the_key_that_downloads(url: &str) {
    download_with_progress(
        Some("secret".to_string()),
        DownloadTarget::Book {
            item_id: THE_BOOK.to_string(),
        },
        url.to_string(),
        THE_ACCOUNT.to_string(),
        "A Book Of Two Files".to_string(),
        "An Author".to_string(),
        THE_SERVER.to_string(),
        toutui::logic::download::new_progress_map(),
    )
    .await;
}

/// The number of the rows of the media, and the number of the rows of its files.
fn the_rows_of_the_download() -> (usize, usize) {
    let of_the_media = get_download(THE_BOOK, THE_ACCOUNT)
        .expect("the read of the row of the media")
        .map(|_| 1)
        .unwrap_or(0);

    let of_the_files = get_download_files(THE_BOOK, THE_ACCOUNT)
        .expect("the read of the rows of the files")
        .len();

    (of_the_media, of_the_files)
}

/// The rows of a download come away together.
///
/// **The parts of this test stay in one function**: two test functions of one
/// binary take a thread each, and `cargo test` finds a fault of that shape at
/// one run of six (T-144 and T-157).
#[tokio::test(flavor = "multi_thread")]
async fn the_rows_of_a_download_that_the_disk_kept_stay_together() {
    temporary_home();

    // The row of the media and the two rows of its files stand on the disk.
    the_rows_of_a_whole_download();
    assert_eq!(
        the_rows_of_the_download(),
        (1, 2),
        "the rows of a whole download stand on the disk"
    );

    // **The disk refuses the removal of the rows of the files alone.** The old
    // code removed the row of the media before them, and that row went away.
    the_disk_refuses("DELETE", "download_files", true);

    let answer = delete_download(THE_BOOK, THE_ACCOUNT);

    assert!(
        answer.is_err(),
        "a removal that the disk refused gives a fault"
    );
    assert_eq!(
        the_rows_of_the_download(),
        (1, 2),
        "the row of the media stays with the rows of its files"
    );

    the_disk_refuses("DELETE", "download_files", false);

    // **The other half of the same rule.** A disk that refuses the removal of the
    // row of the media keeps the rows of the files of it.
    the_disk_refuses("DELETE", "downloads", true);

    let answer = delete_download(THE_BOOK, THE_ACCOUNT);

    assert!(
        answer.is_err(),
        "a removal that the disk refused gives a fault"
    );
    assert_eq!(
        the_rows_of_the_download(),
        (1, 2),
        "the rows of the files stay with the row of their media"
    );

    the_disk_refuses("DELETE", "downloads", false);

    // The disk takes the removal, and the whole download goes away.
    delete_download(THE_BOOK, THE_ACCOUNT).expect("the removal of the download");

    assert_eq!(
        the_rows_of_the_download(),
        (0, 0),
        "a removal that the disk took removes the media and its files"
    );

    // A second removal of a download that stands in no row gives no fault.
    delete_download(THE_BOOK, THE_ACCOUNT).expect("the removal of a download of no row");

    // **The second road of the same rule is the key `D`.** The rows of a download
    // that the database refused go away with a removal (T-200), and a disk that
    // refuses that removal too leaves a media of the disk that holds no file.
    let host = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!("/api/items/{THE_BOOK}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(the_book_of_one_file()))
        .mount(&host)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/api/items/{THE_BOOK}/file/101/download")))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![b'a'; 40]))
        .mount(&host)
        .await;

    the_disk_refuses("INSERT", "download_files", true);
    the_disk_refuses("DELETE", "downloads", true);

    message::forget();
    the_key_that_downloads(&host.uri()).await;

    assert_eq!(
        the_rows_of_the_download(),
        (1, 0),
        "the rollback that the disk refused leaves the row of the media"
    );

    // **A write of the disk that no caller reads is a write that said nothing**
    // (T-207). The old code held `let _ = delete_download(…)`, and the user then
    // read the words of a download whose rows went away.
    assert_eq!(
        message::for_the_screen(AppView::Home).as_deref(),
        Some(the_words_of_a_download_whose_rows_stay("A Book Of Two Files").as_str()),
        "the words name the rows of the download that stay on the disk"
    );

    // The disk takes the rows of the files again, and the key `D` writes every row
    // of that download: the files of the disk stay, therefore that key needs no
    // byte of the server a second time (T-200).
    the_disk_refuses("INSERT", "download_files", false);
    the_disk_refuses("DELETE", "downloads", false);

    message::forget();
    the_key_that_downloads(&host.uri()).await;

    assert_eq!(
        the_rows_of_the_download(),
        (1, 1),
        "the key D writes the row of the media and the row of its file"
    );

    // The trigger of this test stands on no table now, therefore the database of
    // the test holds the shape of the program.
    let conn = toutui::db::migrate::open_conn().expect("the database of the test");
    let triggers: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'trigger'",
            params![],
            |row| row.get(0),
        )
        .expect("the count of the triggers");

    assert_eq!(triggers, 0, "the triggers of the test go away");
}
