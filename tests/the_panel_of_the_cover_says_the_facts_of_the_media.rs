//! The gate of the facts of the design of the panel 5 of the cover. See T-325.
//!
//! **The answer of the server holds six facts of every book, and no view of the
//! program said one of them.** The measurement of the sandbox of 2026-08-16, of
//! `GET /api/libraries/<Books>/items?limit=100` for `A Long Test Book`:
//!
//! ```json
//! "narratorName": "A Test Narrator",
//! "genres": [ "Fiction", "Adventure" ],
//! "numAudioFiles": 1,
//! "size": 7337326,
//! "ebookFormat": "epub"
//! ```
//!
//! and `"seriesName": "The Test Chronicles #2"` for the second book of the
//! series of that same library.
//!
//! The measurement of the real program v0.8.154 inside tmux, of the Library
//! view of the library `Books` of the sandbox at 160 columns and 45 rows, with
//! the cursor on `A Long Test Book`. The panel 5 held the picture over 21 rows,
//! four facts over three rows, and **15 rows of no character at all**:
//!
//! ```text
//! │      ▄                                         │
//! │Author: Long Author - Year: N/A - Duration: 30m │
//! │Progress: 50%, 15m left, Not finished           │
//! │                                                │
//! │No description available                        │
//! │                                                │
//! ```
//!
//! The corrected program of the same harness, of that same row:
//!
//! ```text
//! │Author    Long Author                           │
//! │Narrator  A Test Narrator                       │
//! │Time      30m, 15m left                         │
//! │Genre     Fiction, Adventure                    │
//! │Files     1 file, 7.0 MB                        │
//! │Ebook     epub                                  │
//! │Progress  50%, Not finished                     │
//! │████████████████████████░░░░░░░░░░░░░░░░░░░░░░░░│
//! ```
//!
//! and the row `The Test Chronicles Volume 3` of the mode of the whole library
//! of that same run said `Series    The Test Chronicles #3` and
//! `Disk      [Downloaded]` beside them.

use toutui::api::libraries::get_all_books::Root;
use toutui::api::utils::collect_get_all_books::collect_the_facts_library;
use toutui::logic::the_facts_of_a_media::{
    the_bar_of_the_progress, the_lines_of_the_facts, TheFactsOfAMedia, TheMediaOfThePanel,
    THE_NARROWEST_BAR,
};
use toutui::ui::the_panel_of_the_cover::{
    the_parts_of_the_panel, THE_ROWS_OF_A_DESCRIPTION, THE_ROWS_OF_THE_FACTS,
};

/// The answer of the sandbox for `A Long Test Book`, of the measurement of
/// 2026-08-16.
fn the_answer_of_the_sandbox() -> Root {
    serde_json::from_value(serde_json::json!({
        "results": [
            {
                "id": "9a671047-6146-4003-8510-d215db074a9c",
                "mediaType": "book",
                "size": 7337326i64,
                "numFiles": 2,
                "media": {
                    "metadata": {
                        "title": "A Long Test Book",
                        "authorName": "Long Author",
                        "narratorName": "A Test Narrator",
                        "seriesName": "",
                        "genres": [ "Fiction", "Adventure" ],
                        "publishedYear": null,
                        "description": null
                    },
                    "numAudioFiles": 1,
                    "duration": 1800.0,
                    "size": 7337326i64,
                    "ebookFormat": "epub"
                }
            }
        ],
        "total": 1
    }))
    .expect("the answer of the server must read")
}

/// The program reads the six facts of the answer of the items.
///
/// **The name of the field of the ebook was `ebook_file_format`**, and
/// `rename_all = "camelCase"` reads that name as `ebookFileFormat`. The server
/// sends `ebookFormat`, therefore that field was `None` for every book of every
/// library of every server.
///
/// **The parts of this test stay in one function.**
#[tokio::test]
async fn the_program_reads_the_six_facts_of_the_answer_of_the_items() {
    let facts = collect_the_facts_library(&the_answer_of_the_sandbox()).await;

    assert_eq!(facts.len(), 1, "one row of the list holds one media");

    assert_eq!(
        facts[0],
        TheFactsOfAMedia {
            series: String::new(),
            narrator: "A Test Narrator".to_string(),
            genre: "Fiction, Adventure".to_string(),
            files: 1,
            size: 7337326,
            the_ebook: "epub".to_string(),
        }
    );

    // A book of a series holds the name of it and the number of the book in it.
    let of_a_series: Root = serde_json::from_value(serde_json::json!({
        "results": [ { "id": "item-1", "media": { "metadata": {
            "title": "The Test Chronicles Volume 2",
            "seriesName": "The Test Chronicles #2"
        } } } ]
    }))
    .expect("an answer");

    assert_eq!(
        collect_the_facts_library(&of_a_series).await[0].series,
        "The Test Chronicles #2"
    );
}

/// Every fact of the design takes a line of its own, and a fact that the server
/// did not give takes no line at all.
///
/// **The parts of this test stay in one function.**
#[tokio::test]
async fn every_fact_of_the_server_takes_a_line_of_the_panel() {
    let facts = collect_the_facts_library(&the_answer_of_the_sandbox()).await;

    let media = TheMediaOfThePanel {
        facts: &facts[0],
        author: "Long Author",
        year: "N/A",
        length: "30m",
        of_the_disk: "",
        percent: "50",
        // The words of the panel of today end with a comma, because the line of
        // that panel is `Progress: {}%, {} {}`.
        the_time_that_is_left: "15m left,",
        the_end: "Not finished",
    };

    let lines = the_lines_of_the_facts(&media, 48);

    assert_eq!(
        lines,
        vec![
            "Author    Long Author".to_string(),
            "Narrator  A Test Narrator".to_string(),
            "Time      30m, 15m left".to_string(),
            "Genre     Fiction, Adventure".to_string(),
            "Files     1 file, 7.0 MB".to_string(),
            "Ebook     epub".to_string(),
            "Progress  50%, Not finished".to_string(),
            format!("{}{}", "█".repeat(24), "░".repeat(24)),
        ],
        "the screen of the measurement of 2026-08-16"
    );

    // The year of this book is `N/A` and it stands in no series, therefore
    // neither of them takes a row of the screen of the user.
    assert!(!lines.iter().any(|line| line.starts_with("Year")));
    assert!(!lines.iter().any(|line| line.starts_with("Series")));
}

/// The bar of the progress says the place of the user with no letter at all.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_bar_of_the_progress_holds_one_cell_of_each_column() {
    assert_eq!(the_bar_of_the_progress("0", 10), Some("░".repeat(10)));
    assert_eq!(the_bar_of_the_progress("100", 10), Some("█".repeat(10)));

    // A bar of a panel that is narrow says no part of a whole.
    assert_eq!(the_bar_of_the_progress("50", THE_NARROWEST_BAR - 1), None);

    // A percent that the program does not have gives no bar.
    assert_eq!(the_bar_of_the_progress("", 40), None);
}

/// The facts of the design take the rows that they need, and the picture gives
/// them its rows.
///
/// **The panel of T-319 gave the facts three rows** (`THE_ROWS_OF_THE_FACTS`),
/// because the words of the panel said the two lines of the area under the
/// list. A list of eight facts of eight lines does not stand in three rows.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_facts_of_the_design_take_the_rows_that_they_need() {
    // The measurement of 2026-08-16: the panel 5 of a screen of 160 by 45
    // stands at the column 110 and it holds 39 rows inside its border.
    let inside = ratatui::layout::Rect::new(111, 3, 48, 39);

    let of_the_design = 8;
    let parts = the_parts_of_the_panel(inside, true, of_the_design);

    assert_eq!(parts.facts.height, of_the_design);
    assert!(
        parts.facts.height > THE_ROWS_OF_THE_FACTS,
        "the facts of the design take more rows than the words of T-319"
    );

    // No row of the panel goes away, and no row holds two parts.
    let cover = parts.cover.expect("a picture comes");
    assert_eq!(
        cover.height + parts.facts.height + parts.description.height,
        inside.height
    );

    // The description keeps the rows that it needs.
    assert!(parts.description.height >= THE_ROWS_OF_A_DESCRIPTION);

    // **A panel that is not tall gives the facts the rows that it has**, and
    // the picture never goes under the height that a cover needs.
    let small = ratatui::layout::Rect::new(111, 3, 48, 16);
    let parts = the_parts_of_the_panel(small, true, of_the_design);

    assert_eq!(
        parts.cover.map(|of| of.height),
        Some(toutui::ui::cover::MIN_HEIGHT_FOR_COVER)
    );
    assert_eq!(
        parts.facts.height + parts.description.height,
        small.height - toutui::ui::cover::MIN_HEIGHT_FOR_COVER
    );
}
