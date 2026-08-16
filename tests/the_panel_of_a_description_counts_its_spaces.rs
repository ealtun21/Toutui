//! The panel of a description counts the spaces of its text. See T-309.
//!
//! **A wrap of `trim: true` keeps every space that stands inside a row**
//! (T-302), and it takes away the spaces of the start of a new row alone.
//! `the_rows_of_a_message` of `src/logic/message.rs` holds that rule since
//! T-302, and `the_number_of_the_lines` of
//! `src/logic/the_scroll_of_a_panel.rs` held a second rule of its own: it read
//! **one** space between two words. That function is the one road to the
//! length of the text of a panel (T-252), therefore the largest scroll of the
//! keys `J` and `K` (T-252) and the bar of the scroll of the panel (T-253)
//! each come of that number.
//!
//! The measurement of 2026-08-16, of the real program v0.8.137 inside tmux
//! against the sandbox, with the account `toutuitest` and a terminal of 80
//! columns. **The data of this fault is the text of the server**, therefore it
//! needs no proxy, no book of a harness, and no build of the fault of the
//! source: a `PATCH /api/items/:id/media` gave the book `A Book Of An Epub
//! With No Container` of the library `Books` a description of 200 words
//! `alpha` and the word `OMEGAEND` after them, with **three** spaces between
//! two words. The key `Tab` of the Home view gave the Library view, whose
//! panel holds 18 rows and 78 columns of text, and it drew **ten** words of
//! every row:
//!
//! ```text
//! alpha   alpha   alpha   alpha   alpha   alpha   alpha   alpha   alpha   alpha
//! ```
//!
//! The text takes **21** rows of that panel, and the program counted **16**:
//! at 78 columns the old rule fitted thirteen words of five columns in a row.
//! Sixteen is fewer than the eighteen rows of the panel, therefore
//! `the_last_scroll` gave 0, **no bar of the scroll came**, and **forty
//! presses of the key `J` changed no character of the screen**. The word
//! `OMEGAEND` stood on the row 21, and no key of the program reached it.
//!
//! **The control of the same run**: the key `j` gave the book after it, whose
//! description holds the same 201 words with **one** space between two words.
//! That text takes 16 rows of the same panel, and `OMEGAEND` stood on the last
//! row of it at the first frame.
//!
//! **The corrected program**, of the same keys and the same description: the
//! bar of the scroll came, the key `J` moved the panel, and `OMEGAEND` stood
//! on the last row of it.
//!
//! **The gate below asks ratatui**, as the gate of T-306 does: the panels of
//! this program draw a `Paragraph` with `Wrap { trim: true }`, therefore the
//! number of the rows that such a paragraph takes in a `Buffer` of a width is
//! the one truth of this count. The sweep of T-306 held single spaces alone,
//! and the fault stood behind them.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;
use ratatui::widgets::{Paragraph, Wrap};

use toutui::logic::the_scroll_of_a_panel::{the_last_scroll, the_number_of_the_lines};

/// The last word of the description of the measurement. The user of the fault
/// never read it.
const THE_END: &str = "OMEGAEND";

/// The description of the measurement, of 200 words `alpha` and the word after
/// them, with `spaces` spaces between two words.
fn the_description(spaces: usize) -> String {
    let between = " ".repeat(spaces);
    let mut words = vec!["alpha"; 200];

    words.push(THE_END);
    words.join(&between)
}

/// The number of the rows that ratatui takes for this text in a panel of
/// `width` columns.
///
/// The panels of this program draw with `Wrap { trim: true }`, therefore this
/// is the number that the screen of the user has. The buffer holds many more
/// rows than the text needs, and the count is the row after the last row that
/// holds a character. See T-306.
fn the_rows_of_ratatui(text: &str, width: u16) -> usize {
    let room = text.chars().count() + text.split('\n').count() + 4;
    let area = Rect::new(0, 0, width, u16::try_from(room).unwrap());
    let mut buffer = Buffer::empty(area);

    Paragraph::new(text)
        .wrap(Wrap { trim: true })
        .render(area, &mut buffer);

    let mut rows = 0usize;

    for row in 0..area.height {
        let holds_a_character =
            (0..area.width).any(|column| buffer[(column, row)].symbol().trim() != "");

        if holds_a_character {
            rows = usize::from(row) + 1;
        }
    }

    rows
}

/// The parts of this test stay in one function: the rule of this repository
/// keeps one test of one module.
#[test]
fn the_lines_of_a_panel_hold_the_spaces_of_the_text() {
    let of_three_spaces = the_description(3);
    let of_one_space = the_description(1);

    // **The numbers of the measurement of T-309.** The panel of the Library
    // view of a terminal of 80 columns holds 78 columns of text and 18 rows.
    assert_eq!(the_rows_of_ratatui(&of_three_spaces, 78), 21);
    assert_eq!(the_number_of_the_lines(&of_three_spaces, 78), 21);

    // The old rule counted 16 rows, which is fewer than the 18 rows of the
    // panel: the bar of the scroll did not come, and the key `J` moved
    // nothing. The bar takes one column of the text, and ten words of five
    // columns stand in the 77 columns that stay.
    assert_eq!(the_last_scroll(&of_three_spaces, 77, 18), 3);

    // The control of the measurement: the same words with one space between
    // two of them take 16 rows, and the panel of 18 rows holds every one of
    // them. That text took the same 16 rows before this correction.
    assert_eq!(the_rows_of_ratatui(&of_one_space, 78), 16);
    assert_eq!(the_number_of_the_lines(&of_one_space, 78), 16);
    assert_eq!(the_last_scroll(&of_one_space, 78, 18), 0);

    // **The rule of T-309**: the number of the lines of a text is the number
    // of the rows that ratatui draws it on, and the spaces of the text keep
    // their width.
    //
    // The texts of the sweep hold the spaces of a description of a server: two
    // spaces after the end of a sentence, a run of spaces that meets the end
    // of a row, a space at the start of a line, a space at the end of one, and
    // an end of a line of its own. **The words are shorter than the narrowest
    // width of the sweep** (T-306): a word that is longer than the panel takes
    // a road of ratatui that overflows the area.
    //
    // **A text of ends of a line alone stands outside this sweep**: the count
    // of the rows of ratatui above reads the rows that hold a character,
    // therefore it cannot see the rows of a text that holds none. A text whose
    // last line holds characters keeps every empty line before it in the
    // count, and the text of Japanese and of ASCII below holds one.
    let the_texts = [
        "alpha  beta  gamma  delta  epsilon  zeta  eta  theta",
        "alpha   beta   gamma   delta   epsilon   zeta   eta",
        "alpha beta.  Gamma delta.  Epsilon zeta.  Eta theta.",
        "   alpha beta gamma   ",
        "alpha\n\n  beta  gamma\ndelta   epsilon",
        "日本語  書籍説明  文段  日本語  書籍説明  文段",
        "日本語 a  書籍 bb   説明 ccc    文段",
        "alpha          beta",
        "  ",
        "a  b",
    ];

    for text in the_texts {
        for width in 20..=200u16 {
            assert_eq!(
                the_number_of_the_lines(text, width),
                the_rows_of_ratatui(text, width).max(1),
                "the panel of {width} columns lost the spaces of: {text:?}"
            );
        }
    }
}
