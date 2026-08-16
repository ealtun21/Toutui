//! A filter of one series gives the books of that series. See T-318.
//!
//! **This is the part of the stage 5 of the road of the panels that says
//! "a series opens into its books"**, and the measurement of this round says
//! that the road of the user stands already and that the list lies on it.
//!
//! **The fault, of the real program v0.8.149 inside tmux**, of the library
//! `Books` of the sandbox on a screen of 160 columns and 45 rows. The user
//! pressed the key `f`, they took the row `The Test Chronicles` of the group
//! `The series`, and the program asked the server with
//! `&filter=series.OGE1ZGNlNzgtYzgyMy00NDFlLWE5OTgtZWJhOWY5ZThkMDZi`:
//!
//! ```text
//! ╔4 Library [1 item] — a filter is on (f) ═════════════════════════════════╗
//! ║    Title                                 Author               Time  Done║
//! ║➤   The Test Chronicles [3 books]                                        ║
//! ╚═════════════════════════════════════════════════════════════════════════╝
//! ```
//!
//! `GET /api/libraries/1b090ea8-…/items?limit=500&collapseseries=1&filter=series.…`
//! answered with `total: 3` and the three books `The Test Chronicles Volume 1`,
//! `Volume 2`, and `Volume 3`. **The program held the answer of the server and
//! it showed one line of it**, and that line repeats the name of the filter
//! that the user chose.
//!
//! **The rule of T-22 is right for a library of many series**: a series of
//! twelve books filled the screen, therefore `group_library` puts every book
//! of a series on one line. **It is wrong for a list of one series and of no
//! other media**: the user asked for that series, and the group then hides the
//! answer.
//!
//! **The correction, of the same harness**:
//!
//! ```text
//! ╔4 Library [3 items] — a filter is on (f) ════════════════════════════════╗
//! ║    Title                                 Author              Time  Done ║
//! ║➤ ✓ The Test Chronicles Volume 1          Series Author        <1m  done ║
//! ║    The Test Chronicles Volume 2          Series Author        <1m   41% ║
//! ║  ✓ The Test Chronicles Volume 3          Series Author        <1m  done ║
//! ╚═════════════════════════════════════════════════════════════════════════╝
//! ```
//!
//! The header then says the number of the books that the server gave, and each
//! of them holds the author, the length, and the place of the user of the
//! table of T-321.
//!
//! **The functions of this gate are pure**, therefore this test needs no
//! server and no terminal.

use toutui::api::utils::collect_series::{SeriesBookView, SeriesView};
use toutui::logic::library_view::{group_library, LibraryRow};
use toutui::logic::sort_filter::{filter_value, is_a_filter_of_one_series};

/// The identity of the series `The Test Chronicles` of the sandbox.
const THE_SERIES: &str = "8a5dce78-c823-441e-a998-eba9f9e8d06b";

/// The identities of the three books of that series, in the sequence of the
/// answer of the server.
const THE_BOOKS: [&str; 3] = [
    "5a66f3c0-7c4e-4dda-881e-622a6f505f9a",
    "89be0784-ce09-431a-bf2e-72f81f99e39a",
    "040e9d69-1211-44fb-ad29-3ece26936d91",
];

fn the_series_of_the_sandbox() -> Vec<SeriesView> {
    vec![SeriesView {
        id: THE_SERIES.to_string(),
        name: "The Test Chronicles".to_string(),
        description: "Three books of a test.".to_string(),
        books: THE_BOOKS
            .iter()
            .enumerate()
            .map(|(index, id)| SeriesBookView {
                id: (*id).to_string(),
                title: format!("The Test Chronicles Volume {}", index + 1),
                author: "Series Author".to_string(),
                sequence: (index + 1).to_string(),
                duration: 0.0,
                description: String::new(),
            })
            .collect(),
    }]
}

fn the_ids(values: &[&str]) -> Vec<String> {
    values.iter().map(|one| one.to_string()).collect()
}

/// The value of the filter that the view of the key `f` writes for that row.
#[test]
fn the_program_knows_the_filter_of_one_series() {
    let value = filter_value("series", THE_SERIES);

    assert_eq!(
        value, "series.OGE1ZGNlNzgtYzgyMy00NDFlLWE5OTgtZWJhOWY5ZThkMDZi",
        "the value of the filter is the value of the measurement of tmux"
    );
    assert!(is_a_filter_of_one_series(&value));

    // No other group of the filters of this program takes this road.
    assert!(!is_a_filter_of_one_series(""));
    assert!(!is_a_filter_of_one_series(&filter_value(
        "progress", "finished"
    )));
    assert!(!is_a_filter_of_one_series(&filter_value(
        "authors", THE_SERIES
    )));
    assert!(!is_a_filter_of_one_series(&filter_value(
        "genres", "Fantasy"
    )));
}

/// The list of the fault held one line for three books of the answer.
#[test]
fn the_list_of_such_a_filter_holds_a_line_for_each_book() {
    let ids = the_ids(&THE_BOOKS);
    let series = the_series_of_the_sandbox();

    // The rule of T-22, which the program used for every list.
    let of_the_fault = group_library(&ids, &series, false);
    assert_eq!(
        of_the_fault,
        vec![LibraryRow::Series {
            series: 0,
            first_item: 0
        }],
        "the fault of v0.8.149: one line for the three books of the answer"
    );

    // The rule of the correction.
    let of_the_correction = group_library(
        &ids,
        &series,
        is_a_filter_of_one_series(&filter_value("series", THE_SERIES)),
    );
    assert_eq!(
        of_the_correction,
        vec![
            LibraryRow::Book { item: 0 },
            LibraryRow::Book { item: 1 },
            LibraryRow::Book { item: 2 },
        ],
        "the correction: one line for each book that the server gave"
    );

    // The header of the panel 4 counts the lines of the list, therefore it
    // says `[3 items]` and no longer `[1 item]`.
    assert_eq!(of_the_correction.len(), THE_BOOKS.len());
}

/// **A list of no filter of a series keeps the rule of T-22.** A library of
/// twelve books of one series must not fill the screen with them.
#[test]
fn a_list_of_another_filter_keeps_the_line_of_the_series() {
    let ids = the_ids(&[
        THE_BOOKS[0],
        THE_BOOKS[1],
        THE_BOOKS[2],
        "8fda6e43-0728-46ad-98bc-4c8634e299ad",
    ]);
    let series = the_series_of_the_sandbox();

    for filter in [
        String::new(),
        filter_value("progress", "finished"),
        filter_value("authors", "cc5891d3-f0a5-42b0-ac39-6c33df199efd"),
    ] {
        let rows = group_library(&ids, &series, is_a_filter_of_one_series(&filter));

        assert_eq!(
            rows,
            vec![
                LibraryRow::Series {
                    series: 0,
                    first_item: 0
                },
                LibraryRow::Book { item: 3 },
            ],
            "the filter {:?} keeps the line of the series",
            filter
        );
    }
}
