//! A name of a filter of the server stands in one row of the header (T-378).
//!
//! A genre, a tag, or a narrator of the server can hold an end of a line. The
//! words of the sequence and of the filter go into a `Paragraph` of one row of
//! the header, and a `Paragraph` splits its text at a `\n`: the header of
//! v0.8.208 showed `▣ Alpha` for the genre `Alpha\nOMEGAEND`, and the word
//! after the end of the line was gone with no mark at all.
//!
//! The measurement of the real program v0.8.208 inside tmux at 100 columns
//! against the sandbox: `PATCH /api/items/:id/media` gave `A Long Test Book`
//! the genre `Alpha\nOMEGAEND`, the view of the key `f` applied it, and the
//! second row of the header read `⇅ The time when the book came, the smallest
//! first ▣ Alpha`.
//!
//! The functions are pure, therefore this test needs no screen and no server.
//! **The parts of this test stay in one function.**

use toutui::logic::sort_filter::{filter_value, FilterChoice};
use toutui::ui::the_panels_of_the_stack::{
    the_name_of_a_filter, the_words_of_the_sequence_and_the_filter,
};

#[test]
fn a_name_of_a_filter_stands_in_one_row() {
    let value = filter_value("genres", "Alpha\nOMEGAEND");

    let of_the_server = vec![FilterChoice {
        label: "Alpha\nOMEGAEND".to_string(),
        group: "The genres",
        value: value.clone(),
    }];

    // The name of the filter collapses the end of the line into one space.
    assert_eq!(
        the_name_of_a_filter(&value, &of_the_server),
        "Alpha OMEGAEND",
        "the name of a filter of two lines stands in one line"
    );

    // The whole of the words of the header holds no end of a line, therefore
    // the `Paragraph` of one row shows every word of them.
    let the_words =
        the_words_of_the_sequence_and_the_filter(false, "", true, &value, &of_the_server);

    assert!(
        !the_words.contains(['\n', '\r']),
        "the words of the sequence and of the filter hold no end of a line: {the_words:?}"
    );
    assert!(
        the_words.contains("Alpha OMEGAEND"),
        "the words hold the whole name: {the_words:?}"
    );
}
