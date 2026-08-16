//! The gate of the mode of the whole library. See T-324.
//!
//! **A book of a series stood in no row of the Library view.** The request of
//! the items holds `collapseseries=1`, therefore the server answers with one
//! item for the whole series, and `group_library` then makes one line of it.
//!
//! The measurement of the real program v0.8.153 inside tmux, of the Library
//! view of the library `Books` of the sandbox at 160 columns and 45 rows:
//!
//! ```text
//! ╔4 Library [18 items] ════════════════════════════════════════════════════╗
//! ║    Title                                 Author               Time  Done║
//! ║    A Second Book Of Many Hours           Many Hours Author    8h00     -║
//! ║    The Test Chronicles [3 books]                                        ║
//! ║    Second Series [3 books]                                              ║
//! ```
//!
//! `GET /api/libraries/<Books>/items?limit=500&collapseseries=0` gives 22 items
//! and `collapseseries=1` gives 18: the four books `The Test Chronicles Volume
//! 2`, `The Test Chronicles Volume 3`, `Second Series Volume 2`, and `Second
//! Series Volume 3` stand in **no row of the list**, and the columns `Author`,
//! `Time`, and `Done` of a row of a series hold no character at all. **The user
//! had no key that gives every book of every series in one list.**
//!
//! The corrected program of the same harness, after the key `l` on the row
//! `Every book of a series` of the panel 2:
//!
//! ```text
//! ┌4 Library [22 items] ────────────────────────────────────────────────────┐
//! │  ✓ The Test Chronicles Volume 3          Series Author         <1m  done│
//! │    The Test Chronicles Volume 2          Series Author         <1m   41%│
//! │  ✓ The Test Chronicles Volume 1          Series Author         <1m  done│
//! ```

use toutui::logic::library_view::{
    group_library, the_books_stand_apart, the_group_of_the_request, the_whole_library, LibraryRow,
};
use toutui::logic::sort_filter::{self, Row};
use toutui::ui::the_panels_of_the_stack;

/// The columns of a line of the panel 2 of the stack, of a screen of 160
/// columns. The panel takes 34 columns, the border takes one of them at each
/// end, and two more go to the sign of the cursor of ratatui
/// (`the_lines_of_a_panel` of `crate::ui::the_panels_of_the_stack`).
const THE_COLUMNS_OF_THE_PANEL_2: usize = 30;

/// The row of the whole library stands in the view of the key `f` and in the
/// panel 2 of the stack, and it stands for a library of books alone.
///
/// **A library of podcasts holds no series** (T-324), therefore its request
/// takes no `collapseseries` and a row of that mode would promise a function
/// that the program does not have (T-118).
///
/// This test writes no mode, therefore it stands beside the test below.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_row_of_the_whole_library_stands_for_a_library_of_books_alone() {
    let of_the_books = sort_filter::rows(false, &[], None);
    let of_the_podcasts = sort_filter::rows(true, &[], None);

    assert!(of_the_books.contains(&Row::TheWholeLibrary));
    assert!(!of_the_podcasts.contains(&Row::TheWholeLibrary));

    // The row stands after the row of the direction, and before the group of
    // the filter.
    let at = of_the_books
        .iter()
        .position(|row| *row == Row::TheWholeLibrary)
        .expect("the row stands");

    assert_eq!(of_the_books.get(at - 1), Some(&Row::Direction));
    assert_eq!(
        of_the_books.get(at + 1),
        Some(&Row::Title("The filter".to_string()))
    );

    // The panel 2 of the stack holds the rows of the group "The sequence" of
    // that same view (T-318), therefore it holds this row too.
    let of_the_panel = the_panels_of_the_stack::the_rows_of_the_sequence(false);
    assert_eq!(of_the_panel.last(), Some(&Row::TheWholeLibrary));
    assert!(
        !the_panels_of_the_stack::the_rows_of_the_sequence(true).contains(&Row::TheWholeLibrary)
    );

    // **The user can take this row**: a row that no key reaches says a state
    // that nothing changes.
    assert!(Row::TheWholeLibrary.is_a_line_of_the_user());
}

/// The mode writes the request of the items, the rows of the list, and the mark
/// of its own row.
///
/// **The parameter of the server alone changes no screen** (the trap of T-318):
/// `group_library` collapses the answer again on the side of the program.
///
/// **The parts of this test stay in one function**, because the mode lives in
/// the process: two test functions of one binary would fight for it.
#[test]
fn the_mode_writes_the_request_and_the_rows_and_the_mark() {
    let series = [toutui::api::utils::collect_series::SeriesView {
        id: "series-1".to_string(),
        name: "The Test Chronicles".to_string(),
        description: String::new(),
        books: ["b1", "b2", "b3"]
            .iter()
            .enumerate()
            .map(
                |(index, id)| toutui::api::utils::collect_series::SeriesBookView {
                    id: id.to_string(),
                    title: format!("The Test Chronicles Volume {}", index + 1),
                    author: "Series Author".to_string(),
                    sequence: (index + 1).to_string(),
                    duration: 0.0,
                    description: String::new(),
                },
            )
            .collect(),
    }];

    let ids: Vec<String> = ["a", "b1", "b2", "b3", "z"]
        .iter()
        .map(|id| id.to_string())
        .collect();

    // The mode of the start: the request groups the books, and the list holds
    // one line for the series.
    the_whole_library::keep(false);

    assert_eq!(the_group_of_the_request(false), "&collapseseries=1");
    assert_eq!(the_group_of_the_request(true), "");
    assert!(!the_books_stand_apart(""));

    let rows = group_library(&ids, &series, the_books_stand_apart(""));
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[1].series(), Some(0));

    assert_eq!(
        sort_filter::line_of(&Row::TheWholeLibrary, "", false, ""),
        "  Every book of a series"
    );

    // The mode of the user: the request takes no parameter, and every book of
    // the series takes a line of its own.
    the_whole_library::keep(true);

    assert_eq!(the_group_of_the_request(false), "");

    // **A library of podcasts holds no series**, therefore the mode of a user
    // of a library of books writes no parameter of that request either.
    assert_eq!(the_group_of_the_request(true), "");
    assert!(the_books_stand_apart(""));

    let rows = group_library(&ids, &series, the_books_stand_apart(""));
    assert_eq!(rows.len(), ids.len());
    assert!(rows
        .iter()
        .all(|row| matches!(row, LibraryRow::Book { .. })));

    // **The mark says the state, and the words do not** (T-324): the two
    // sentences of the first form of this row each reached the panel 2 as `The
    // books of a series: one…`, and a row that says one text for the two states
    // of the program says nothing at all.
    let of_the_mode = sort_filter::line_of(&Row::TheWholeLibrary, "", false, "");
    assert_eq!(of_the_mode, "✓ Every book of a series");
    assert!(
        toutui::logic::message::the_columns_of(&of_the_mode) <= THE_COLUMNS_OF_THE_PANEL_2,
        "the row of the panel 2 holds {} columns: {}",
        THE_COLUMNS_OF_THE_PANEL_2,
        of_the_mode
    );

    // **The filter of one series is the other road to the books of a series**
    // (T-318), and the mode must not take it away.
    the_whole_library::keep(false);
    assert!(the_books_stand_apart(&sort_filter::filter_value(
        "series",
        "The Test Chronicles"
    )));

    // The mode of the start goes back, for every test of this binary after
    // this one.
    the_whole_library::keep(false);
}
