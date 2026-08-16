//! The words of the sequence and of the filter never write on the address of
//! the server. See T-329.
//!
//! **The second row of the header holds three parts, and each of them is a
//! paragraph of its own over the whole area**: the address of the server at the
//! left, the words of the sequence and of the filter in the middle, and the
//! notice of the key `R` at the right. A paragraph that is too long therefore
//! writes on the letters of its neighbour, which is the fault of T-115 one row
//! below.
//!
//! **The measurement of the real program v0.8.158 inside tmux**, of the Home
//! view of the library `Books` of the sandbox, of the sequence "the title, the
//! largest first" and of the filter "the media that you finished":
//!
//! ```text
//! | The width | The second row of the header |
//! |---|---|
//! | 84 | `🔗 localhost:13399title, the largest first ▣ The media that you finished` |
//! | 90 | `🔗 localhost:13399he title, the largest first ▣ The media that you finished` |
//! ```
//!
//! The user read no gap, no mark `⇅`, and no first word, and 84 to 119 columns
//! is the shape of two columns that the design gives those words (the decision
//! 3 of the road of the panels). The same row of the sequence of the server
//! held `🔗 localhost:13399quence of the server ▣ No filter` at 60 columns.

use toutui::logic::message::the_columns_of;
use toutui::ui::the_panels_of_the_stack::{the_column_of_the_words, THE_GAP_OF_THE_WORDS};

/// The columns of the row of the address of the sandbox, `🔗 localhost:13399`:
/// the mark takes two columns, the space one, and the address fifteen.
const THE_ADDRESS: u16 = 18;

/// The columns of the words of the measurement above.
fn the_words_of_the_measurement() -> u16 {
    let words = "⇅ The title, the largest first ▣ The media that you finished";

    // **The mark takes three bytes and one column** (the trap 245), therefore
    // the length of the text is not the width of it.
    u16::try_from(the_columns_of(words)).expect("the words of a header hold few columns")
}

/// The four widths of the measurement, and the rule of each of them.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_words_never_stand_on_the_columns_of_the_address() {
    let words = the_words_of_the_measurement();
    assert_eq!(words, 60, "the words of the measurement hold 60 columns");

    // 84 columns: the shape of two columns of the design. The middle of that
    // row stands at the column 12, which is inside the address, therefore the
    // words take the first free column after it.
    let at_84 = the_column_of_the_words(84, THE_ADDRESS, 0, words)
        .expect("84 columns hold the words beside the address");
    assert_eq!(at_84, THE_ADDRESS + THE_GAP_OF_THE_WORDS);
    assert!(
        u32::from(at_84) + u32::from(words) <= 84,
        "the words end inside the row: {at_84} + {words}"
    );

    // 90 columns: the same rule, with more room after the words.
    let at_90 = the_column_of_the_words(90, THE_ADDRESS, 0, words)
        .expect("90 columns hold the words beside the address");
    assert_eq!(at_90, THE_ADDRESS + THE_GAP_OF_THE_WORDS);

    // 60 columns: the row has no room for the whole of the words, therefore it
    // holds none of them. A text that the row cuts says nothing to the user
    // (T-91), and the view of the key `f` holds the two values at every width.
    assert_eq!(the_column_of_the_words(60, THE_ADDRESS, 0, words), None);

    // 160 columns: the middle of the row stands after the address, therefore
    // the words keep the middle. Every screen that stood before T-329 stands in
    // the same shape.
    assert_eq!(
        the_column_of_the_words(160, THE_ADDRESS, 0, words),
        Some((160 - words) / 2)
    );
}

/// The words of the sequence of the server, which is the value of the start of
/// every account.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_words_of_the_sequence_of_the_server_keep_every_screen_that_stood() {
    let words = u16::try_from(the_columns_of("⇅ The sequence of the server ▣ No filter"))
        .expect("the words of a header hold few columns");
    assert_eq!(words, 40);

    // The measurement of the program before T-329 read these three rows whole,
    // therefore the correction must not move them.
    for (width, column) in [(80_u16, 20_u16), (84, 22), (100, 30), (119, 39)] {
        assert_eq!(
            the_column_of_the_words(width, THE_ADDRESS, 0, words),
            Some(column),
            "the words of {width} columns stand at the column {column}"
        );
    }

    // The same measurement read `🔗 localhost:13399he sequence of the server`
    // at 70 columns. The words stand beside the address there now.
    assert_eq!(
        the_column_of_the_words(70, THE_ADDRESS, 0, words),
        Some(THE_ADDRESS + THE_GAP_OF_THE_WORDS)
    );

    // 40 columns is the narrowest terminal that this fork measures (T-301), and
    // no arrangement of 40 columns of words and 18 of an address holds it.
    assert_eq!(the_column_of_the_words(40, THE_ADDRESS, 0, words), None);
}

/// The notice of the key `R` stands at the right of the same row, therefore the
/// words must keep away from it too.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_words_keep_away_from_the_notice_at_the_right() {
    // `R: the server has newer data` and the space of its paragraph.
    let notice = u16::try_from(the_columns_of("R: the server has newer data") + 1)
        .expect("a notice holds few columns");
    let words = 40;

    // A row of 100 columns holds the address, the words, and the notice: the
    // middle of it stands at the column 30, and the notice starts at 71.
    let at_100 = the_column_of_the_words(100, THE_ADDRESS, notice, words)
        .expect("100 columns hold the three parts");
    assert!(at_100 >= THE_ADDRESS + THE_GAP_OF_THE_WORDS);
    assert!(
        u32::from(at_100) + u32::from(words) + u32::from(THE_GAP_OF_THE_WORDS)
            <= u32::from(100_u16 - notice),
        "the words end before the notice: {at_100} + {words}"
    );

    // A row of 84 columns holds no room for the three of them, therefore it
    // holds the address and the notice alone.
    assert_eq!(
        the_column_of_the_words(84, THE_ADDRESS, notice, words),
        None,
        "the words go away before they write on the notice"
    );
}

/// A row that holds no word, and a row of no width at all.
///
/// **The parts of this test stay in one function.**
#[test]
fn a_row_with_no_room_holds_no_word() {
    // A screen of no column at all, which `saturating_sub` must hold.
    assert_eq!(the_column_of_the_words(0, 0, 0, 40), None);

    // The two neighbours meet in the middle of the row.
    assert_eq!(the_column_of_the_words(60, 30, 30, 1), None);

    // No words at all take no column of the row.
    assert_eq!(the_column_of_the_words(160, THE_ADDRESS, 0, 0), None);

    // An address that is longer than the whole row takes every column of it.
    assert_eq!(the_column_of_the_words(60, 200, 0, 40), None);
}
