//! A library with no filter names no filter in the header. See T-385.
//!
//! The measurement of v0.8.215 inside tmux at 100 columns, against the
//! sandbox: the second row of the header read
//! `⇅ The sequence of the server ▣ No filter` at the start of the program,
//! and it read the same words after the user took a filter away with the row
//! `No filter` of the view of the key `f`. The words `▣ No filter` say a
//! filter when no filter stands, and a text must not say what the program
//! does not have (T-91, the leftover of T-379).
//!
//! The correction: the words of the header hold the sequence alone when the
//! filter is empty. The row `No filter` of the view of the key `f` and the
//! panel 3 of the stack keep their words, because each of them stands in a
//! place that the user opened for the filter.
//!
//! **The parts of this test stay in one function.**

use toutui::logic::sort_filter::filter_value;
use toutui::ui::the_panels_of_the_stack::the_words_of_the_sequence_and_the_filter;

#[test]
fn a_header_of_no_filter_names_no_filter() {
    // The start of the program: no sequence and no filter.
    let of_the_start = the_words_of_the_sequence_and_the_filter(false, "", false, "", &[]);
    assert_eq!(
        of_the_start, "⇅ The sequence of the server",
        "a header with no filter holds the sequence alone"
    );
    assert!(
        !of_the_start.contains("No filter") && !of_the_start.contains('▣'),
        "a header with no filter says no word of a filter: {of_the_start:?}"
    );

    // A sequence of the user, and still no filter: the removal of a filter
    // gives this shape, because the sequence of the row stays.
    let of_a_sequence =
        the_words_of_the_sequence_and_the_filter(false, "media.metadata.title", true, "", &[]);
    assert_eq!(
        of_a_sequence, "⇅ The title, the largest first",
        "a header of a sequence and of no filter names the sequence alone"
    );

    // A filter that stands keeps its mark and its name.
    let value = filter_value("progress", "in-progress");
    let of_a_filter = the_words_of_the_sequence_and_the_filter(false, "", false, &value, &[]);
    assert_eq!(
        of_a_filter, "⇅ The sequence of the server ▣ Started, not finished",
        "a header of a filter names the filter"
    );

    // A library of podcasts holds the same rule.
    let of_a_podcast = the_words_of_the_sequence_and_the_filter(true, "", false, "", &[]);
    assert!(
        !of_a_podcast.contains("No filter") && !of_a_podcast.contains('▣'),
        "a header of a library of podcasts with no filter says no word of a filter: {of_a_podcast:?}"
    );
}
