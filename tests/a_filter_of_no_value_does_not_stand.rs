//! A filter of an identity that holds no identity does not stand. See T-386.
//!
//! The measurement of the real program v0.8.216 inside tmux against the
//! sandbox: an author of the server with no identity gave the row of the
//! account the filter `authors.` and the name `Test Author`. The request
//! `filter=authors.` gave 0 items, the view said `No media of this library
//! agrees with the filter.`, and the view of the key `f` held no mark `✓`
//! on any row: the program said that a filter is on, and it could not show
//! it. The row of the disk kept that filter over every start.
//!
//! **A filter whose value holds no character asks the server for nothing**,
//! therefore the start does not apply it, in the way of T-382 and of T-383.
//! A write of the sequence then erases it from the row too, because
//! `the_filter_of_the_row_stays` keeps a filter of another condition alone
//! (T-384): a filter of no value acts in no library, and no library gives
//! it back.
//!
//! The parts of this test stay in one function: two test functions of one
//! module fight for the slot of that module in the run of `cargo test`.

use toutui::logic::sort_filter::{is_a_filter_of_no_value, the_filter_of_the_row_stays};

#[test]
fn a_filter_of_no_value_does_not_stand() {
    // A filter of an identity of no character is a filter of no value, for
    // every kind: the value after the point is the address of the request.
    assert!(is_a_filter_of_no_value("authors."));
    assert!(is_a_filter_of_no_value("narrators."));
    assert!(is_a_filter_of_no_value("series."));
    assert!(is_a_filter_of_no_value("genres."));

    // A filter with a value stands, and an empty filter is no filter at all:
    // the guard of the start has nothing to clear.
    assert!(!is_a_filter_of_no_value("authors.Y2M1"));
    assert!(!is_a_filter_of_no_value("progress.ZmluaXNoZWQ="));
    assert!(!is_a_filter_of_no_value(""));

    // A write of the sequence erases a filter of no value from the row: the
    // predicate of T-384 keeps a filter that acts in another library, and a
    // filter of no value acts in no library.
    assert!(!the_filter_of_the_row_stays(
        "authors.", "lib1", "lib1", false
    ));
}
