//! The gate of the day of the start of the media of the panel 5. See T-328.
//!
//! **The design of the panel 5 names the day when the user started the media,
//! and no view of this program said it.** `docs/mockups/mockup-1.txt` holds the
//! line `Started  14 Aug 2026` between the line of the time and the line of the
//! genre, and the round of T-325 left that fact open: the answer of the items of
//! a library holds no day of a start, therefore the plumbing of the other facts
//! could not reach it.
//!
//! **The answer of the account holds that day for every media.** The
//! measurement of the sandbox of 2026-08-16, of `GET /api/me`:
//!
//! ```json
//! "libraryItemId": "9a671047-6146-4003-8510-d215db074a9c",
//! "progress": 0.5,
//! "isFinished": false,
//! "startedAt": 1786905843790
//! ```
//!
//! `crate::api::me::get_media_progress::Root` reads `startedAt` since T-127 and
//! **no call site of the program read that field**: the value stood in the
//! memory of the program at every frame.
//!
//! The measurement of the real program v0.8.157 inside tmux, of the Library view
//! of the library `Books` of the sandbox at 160 columns and 45 rows, with the
//! cursor on `A Long Test Book`:
//!
//! ```text
//! │Author    Long Author                           │
//! │Narrator  A Test Narrator                       │
//! │Time      30m, 15m left                         │
//! │Genre     Fiction, Adventure                    │
//! │Files     1 file, 7.0 MB                        │
//! │Ebook     epub                                  │
//! │Progress  50%, Not finished                     │
//! ```
//!
//! The corrected program of the same harness says `Started   16 Aug 2026` under
//! the line of the time.
//!
//! **A media that the user never started takes no line of a day**, which is the
//! rule of T-325: a row that says `Started   N/A` costs a row of the screen and
//! it tells the user nothing.

use toutui::api::me::get_media_progress::Root;
use toutui::api::utils::collect_get_media_progress::the_day_of_the_start;
use toutui::logic::the_facts_of_a_media::{
    the_lines_of_the_facts, TheFactsOfAMedia, TheMediaOfThePanel,
};
use toutui::logic::the_positions::the_places_of_the_account;

/// The rows of `mediaProgress` of the answer of `GET /api/me` of the sandbox, of
/// the measurement of 2026-08-16.
///
/// The first row is `A Long Test Book`, which the user started; the second row
/// is `Multi File Test Book`, which the answer of the server holds with the day
/// of its start too; and the third row is a media of a server that gave no day
/// at all.
fn the_rows_of_the_account() -> Vec<Root> {
    serde_json::from_value(serde_json::json!([
        {
            "id": "one",
            "userId": "a-user",
            "libraryItemId": "9a671047-6146-4003-8510-d215db074a9c",
            "episodeId": null,
            "duration": 1800.0,
            "progress": 0.5,
            "currentTime": 900.0,
            "isFinished": false,
            "lastUpdate": 1786905849998i64,
            "startedAt": 1786905843790i64
        },
        {
            "id": "two",
            "userId": "a-user",
            "libraryItemId": "ac365248-ba42-47ec-a92b-0e5818abc00d",
            "episodeId": null,
            "duration": 60.0,
            "progress": 0.0,
            "currentTime": 0.0,
            "isFinished": false,
            "lastUpdate": 1786905849998i64,
            "startedAt": 0
        }
    ]))
    .expect("the rows of the answer of the account read")
}

/// The facts of `A Long Test Book` of the sandbox.
fn a_long_test_book() -> TheFactsOfAMedia {
    TheFactsOfAMedia {
        series: String::new(),
        narrator: "A Test Narrator".to_string(),
        genre: "Fiction, Adventure".to_string(),
        files: 1,
        size: 7337326,
        the_ebook: "epub".to_string(),
    }
}

/// **The day of the start comes of the row of the account**, and a media that
/// the user never started gives no day at all.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_day_of_the_start_comes_of_the_answer_of_the_account() {
    let rows = the_rows_of_the_account();

    let day = the_day_of_the_start(&rows[0]);
    assert!(
        day.ends_with(" Aug 2026"),
        "the day of the start of the sandbox stands in August of 2026: {day}"
    );

    // A media of a row of no day of a start.
    assert_eq!(the_day_of_the_start(&rows[1]), "");
}

/// **The box of the places of the account carries that day beside the place of
/// the user.** The panel of a line of the Library view and the panel of a line
/// of the Home view read that box, and neither of the two views asks the server
/// for the place of a book (T-241).
///
/// **The parts of this test stay in one function.**
#[tokio::test]
async fn the_box_of_the_places_holds_the_day_of_the_start() {
    let places = the_places_of_the_account(&the_rows_of_the_account()).await;

    let row = places
        .get("9a671047-6146-4003-8510-d215db074a9c")
        .expect("the box holds the media of the answer");

    // **The three values of the box stay where they stood**: every reader of
    // the box before T-328 takes the value of its own place.
    assert_eq!(row[0], "50", "the percent of the user");
    assert_eq!(row[1], "Not finished", "the mark of the end");
    assert_eq!(row[2], "900", "the place of the user in seconds");
    assert!(
        row[3].ends_with(" Aug 2026"),
        "the day of the start stands after the three of them: {:?}",
        row
    );

    let row = places
        .get("ac365248-ba42-47ec-a92b-0e5818abc00d")
        .expect("the box holds the media of no day of a start");
    assert_eq!(
        row[3], "",
        "a media that the user never started holds no day"
    );
}

/// **The panel of the media says the day of the start under the line of the
/// time**, as the design gives it.
#[test]
fn the_panel_says_the_day_of_the_start_under_the_time() {
    let facts = a_long_test_book();

    let lines = the_lines_of_the_facts(
        &TheMediaOfThePanel {
            facts: &facts,
            author: "Long Author",
            year: "N/A",
            length: "30m",
            of_the_disk: "",
            percent: "50",
            the_time_that_is_left: "15m left,",
            the_end: "Not finished",
            the_day_of_the_start: "16 Aug 2026",
        },
        48,
    );

    assert_eq!(
        lines,
        vec![
            "Author    Long Author",
            "Narrator  A Test Narrator",
            "Time      30m, 15m left",
            "Started   16 Aug 2026",
            "Genre     Fiction, Adventure",
            "Files     1 file, 7.0 MB",
            "Ebook     epub",
            "Progress  50%, Not finished",
            "████████████████████████░░░░░░░░░░░░░░░░░░░░░░░░",
        ],
        "the panel must say the day of the start between the time and the genre"
    );
}

/// **A media that the user never started takes no line of a day at all**, which
/// is the rule of T-325.
#[test]
fn a_media_of_no_day_of_a_start_takes_no_line() {
    let facts = a_long_test_book();

    let lines = the_lines_of_the_facts(
        &TheMediaOfThePanel {
            facts: &facts,
            author: "Long Author",
            year: "N/A",
            length: "30m",
            of_the_disk: "",
            percent: "0",
            the_time_that_is_left: "30m left,",
            the_end: "Not finished",
            the_day_of_the_start: "",
        },
        48,
    );

    assert!(
        !lines.iter().any(|line| line.starts_with("Started")),
        "no line of the panel may name a day that the server did not give: {:#?}",
        lines
    );
}
