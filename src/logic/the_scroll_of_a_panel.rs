//! The scroll of the panel of a description. See T-252, T-253, and T-254.
//!
//! **The bar of the scroll names the two keys that move the panel** (T-254).
//! The measurement of 2026-08-15, of the real program inside tmux: the view
//! "About and changelog" of the settings drew the bar of T-253 at the right of
//! its text, and no character of the screen said which key moves that panel.
//! The list of every key (the key `?`) holds "J / K: Scroll the description
//! down and up", and the footer of that view said
//! "j/k: move  l: take the line  h: back  Tab: home  R: refresh  ?: every key
//! Q: quit".
//!
//! **The footer of the two views that hold the longest lists has no room for
//! those words.** `FOOTER_OF_A_LIBRARY_OF_BOOKS` holds 116 characters and the
//! gate of this repository takes 130 (`every_footer_fits_in_eighty_columns`),
//! and a measurement of the Home view in 60 columns filled the two rows of
//! `FOOTER_HEIGHT` exactly. Therefore the letters stand at the two ends of the
//! bar itself: they cost no row and no column, and they come with the bar.
//!
//! **A panel that holds more text than its rows says so** (T-253). The
//! measurement of 2026-08-15, of the real program inside tmux: the view
//! "About and changelog" of the settings holds the longest text of the program,
//! and 2000 presses of the key `J` took the panel to the entry of the version
//! 0.7.41 of it. No character of the screen said that the panel holds more
//! text, and no character said where in that text the user stands. The bar of
//! the scroll of the panel is the word of it, and `the_panel_of_the_render`
//! gives the render everything that bar needs.
//!
//! **A panel that scrolled past its last line holds no line at all**, and the
//! user then cannot tell it from a media whose description the server did not
//! give: the words of T-249 ("No description available") go away with the text,
//! and the panel says nothing. The key that moves the panel down must therefore
//! stop at the last line of the text.
//!
//! The measurement of 2026-08-15, of the real program inside tmux. The library
//! `Books` of the sandbox, the view of the authors, and the author
//! `Lewis Carroll`, whose description the sandbox holds in one line:
//!
//! ```text
//! ─────────────The authors [9 items]─────────────
//! ➤ Lewis Carroll [1 book(s)]
//!
//! Charles Lutwidge Dodgson wrote under the name Lewis Carroll.
//! ```
//!
//! One press of the key `J` took that line away, and 23 presses after it
//! changed nothing more. The key `H` gave the line back, therefore the text
//! stood in the program the whole time and the panel of the user held nothing:
//!
//! ```text
//! ─────────────The authors [9 items]─────────────
//! ➤ Lewis Carroll [1 book(s)]
//!
//!
//! ```
//!
//! **The program knew no length of the text.** `self.scroll_offset` grew with
//! each press of the key `J` with no limit at all, and the fourteen panels of
//! `src/ui/tui.rs` each read that value. The panel of the changelog of the
//! settings holds the same fault, and that text is the longest of the program.
//!
//! **The box of this module holds the last scroll that the render measured.**
//! The render knows the text and the size of the panel, and the key knows
//! neither: the render therefore writes the value, and the key reads it. One
//! panel stands on the screen at one moment, therefore one box holds it.

use std::sync::atomic::{AtomicU16, Ordering};

/// The last scroll that the render of the panel measured.
static THE_LAST_SCROLL: AtomicU16 = AtomicU16::new(0);

/// Keeps a scroll inside the text of the panel.
///
/// **This is the one line of the correction of T-252**: a build with
/// `scroll` in the place of `scroll.min(last)` gives the panel of no line
/// again, on the road of the render and on the road of the key together.
fn inside_the_text(scroll: u16, last: u16) -> u16 {
    scroll.min(last)
}

/// Gives the number of the lines that the text takes in a panel of `width`
/// columns.
///
/// The panels of this program draw with `Wrap { trim: true }`: the text goes to
/// the line after it at a space, a line of the text that holds no character
/// takes one line of the panel, and the spaces at the start of a line go away.
/// A word that is longer than the panel takes more than one line.
///
/// **A character is not a column** (T-305 and T-306): this function measures
/// every word with `crate::logic::message::the_columns_of`, which is the crate
/// that ratatui measures every text that it draws with. A count of the
/// characters gives a number of the lines that the screen does not have, and
/// the panel then says that it holds a text that it cut.
///
/// The function is pure, therefore a test needs no screen.
pub fn the_number_of_the_lines(text: &str, width: u16) -> usize {
    let width = usize::from(width.max(1));

    text.split('\n')
        .map(|line| the_lines_of_one_line(line, width))
        .sum()
}

/// Gives the number of the lines that one line of the text takes.
fn the_lines_of_one_line(line: &str, width: usize) -> usize {
    let mut count = 1usize;
    let mut length = 0usize;

    for word in line.split_whitespace() {
        let size = crate::logic::message::the_columns_of(word);
        let room = the_room_of_a_word(word, size);

        // The word stands at the end of the line that the word before it holds.
        if length > 0 && length + 1 + room <= width {
            length += 1 + size;
            continue;
        }

        // The word takes a line of its own.
        if length > 0 {
            count += 1;
        }

        if room <= width {
            length = size;
            continue;
        }

        // A word that is longer than the panel goes over more than one line.
        let (lines, last) = the_lines_of_one_word(word, width);

        count += lines - 1;
        length = last;
    }

    count
}

/// Gives the columns that a word needs at the end of a row.
///
/// **The last character of a row takes one column of the row** (T-306): a
/// measurement of 2026-08-16 of the `Paragraph` of ratatui 0.30 with
/// `Wrap { trim: true }` gave two words of eighteen columns of the Han script
/// on one row of **36** columns, and two words of eighteen columns of ASCII on
/// **two** rows of that same width. Four words of the Han script stood on one
/// row of 74 columns, and four words of ASCII needed 75. The rule of the crate
/// is therefore the same at each number of the words: it draws the last
/// character of a row while one column of that row stays, and the terminal
/// then cuts the right half of a character of two columns.
///
/// This program does not choose that rule, and it must have the number of the
/// rows that the screen has: the largest scroll of the key `J` (T-252) and the
/// bar of the scroll (T-253) each come of that number, and a count of its own
/// gives the panel of no line again.
fn the_room_of_a_word(word: &str, size: usize) -> usize {
    let of_the_last = word
        .chars()
        .next_back()
        .map(|character| {
            let mut buffer = [0u8; 4];

            crate::logic::message::the_columns_of(character.encode_utf8(&mut buffer))
        })
        .unwrap_or(1);

    size.saturating_sub(of_the_last.saturating_sub(1))
}

/// Gives the number of the lines that one word takes alone in a panel of
/// `width` columns, and the columns of the last of those lines.
///
/// **A character of two columns that meets the last column of a row stays
/// outside that row** (T-305), because a half of a character is no character.
/// A division of the columns of the word by the columns of the panel therefore
/// says a number that the screen does not have: a panel of 79 columns holds 39
/// characters of the Han script, which is 78 columns, and one column of every
/// row of such a word stays empty.
fn the_lines_of_one_word(word: &str, width: usize) -> (usize, usize) {
    let mut count = 1usize;
    let mut length = 0usize;
    let mut buffer = [0u8; 4];

    for character in word.chars() {
        let columns = crate::logic::message::the_columns_of(character.encode_utf8(&mut buffer));

        if length > 0 && length + columns > width {
            count += 1;
            length = 0;
        }

        length += columns;
    }

    (count, length)
}

/// Gives the largest scroll of a panel that holds this text.
///
/// A panel of `rows` rows shows the whole of a text of `rows` lines or fewer,
/// therefore the largest scroll of such a text is 0.
///
/// The function is pure, therefore a test needs no screen.
pub fn the_last_scroll(text: &str, width: u16, rows: u16) -> u16 {
    let lines = the_number_of_the_lines(text, width);

    u16::try_from(lines.saturating_sub(usize::from(rows))).unwrap_or(u16::MAX)
}

/// What the render of a panel of a description draws. See T-253.
#[derive(Debug, PartialEq, Eq)]
pub struct ThePanel {
    /// The width of the text. It is one character less than the width of the
    /// panel when the bar of the scroll stands beside it.
    pub width_of_the_text: u16,
    /// The scroll of the render, inside the text.
    pub scroll: u16,
    /// The largest scroll of the panel. 0 says that the panel holds the whole
    /// of its text, and the bar of the scroll then does not come.
    pub last: u16,
    /// The number of the lines of the text, at `width_of_the_text`.
    pub lines: usize,
    /// The bar of the scroll stands beside the text. A panel of one character
    /// holds the bar or the text, and the text comes first.
    the_bar_comes: bool,
    /// The letters of the two keys stand at the two ends of the bar. A bar of
    /// few rows holds the letters or the track, and the track comes first.
    the_letters_come: bool,
}

impl ThePanel {
    /// Says that the bar of the scroll stands beside the text.
    ///
    /// **This is the one line of the correction of T-253**: a build with
    /// `false` in the place of `self.the_bar_comes` gives the panel of the
    /// changelog that says nothing of its length again.
    pub fn the_bar_comes(&self) -> bool {
        self.the_bar_comes
    }

    /// Says that the letters of the two keys stand at the two ends of the bar.
    ///
    /// **This is the one line of the correction of T-254**: a build with
    /// `false` in the place of `self.the_letters_come` gives the bar that names
    /// no key again.
    pub fn the_letters_come(&self) -> bool {
        self.the_letters_come
    }
}

/// The smallest number of the rows of a bar that holds the letters of the keys.
///
/// The two letters take one row each, and the track of the bar keeps two rows:
/// a bar of fewer rows says the key and it loses the place of the user in the
/// text, and that place is the work of T-253.
const THE_ROWS_OF_THE_LETTERS: u16 = 4;

/// Gives everything that the render of a panel of a description needs, and it
/// keeps the largest scroll of that panel for the key.
///
/// **A panel that holds more text than its rows says so** (T-253): the user of
/// the panel of the changelog pressed the key that moves it down 2000 times,
/// and no character of the screen said where the text stands or that any line
/// of it is left. The bar of the scroll of that panel is the word of it.
///
/// **The bar takes one character of the width of the text**, therefore the
/// number of the lines comes of that smaller width. A text that holds every
/// line of it in the panel takes no bar and it keeps the whole width: a bar
/// that comes of the width of the bar itself would come and go at each frame.
///
/// The function is pure but for the box of the last scroll, therefore a test
/// needs no screen.
pub fn the_panel_of_the_render(now: u16, text: &str, width: u16, rows: u16) -> ThePanel {
    // A panel of one character holds the bar or the text, and the text comes
    // first.
    let the_bar_comes = width >= 2 && the_last_scroll(text, width, rows) > 0;

    // **The letters of the keys stand at the two ends of the bar** (T-254). A
    // bar of few rows keeps its track, because the place of the user in the
    // text is the work of T-253.
    let the_letters_come = the_bar_comes && rows >= THE_ROWS_OF_THE_LETTERS;

    let width_of_the_text = if the_bar_comes { width - 1 } else { width };

    let lines = the_number_of_the_lines(text, width_of_the_text);
    let last = the_last_scroll(text, width_of_the_text, rows);

    THE_LAST_SCROLL.store(last, Ordering::Relaxed);

    ThePanel {
        width_of_the_text,
        scroll: inside_the_text(now, last),
        last,
        lines,
        the_bar_comes,
        the_letters_come,
    }
}

/// The letter of the key that moves the panel up. It stands at the top of the
/// bar of the scroll. See T-254.
pub const THE_LETTER_OF_THE_KEY_UP: &str = "K";

/// The letter of the key that moves the panel down. It stands at the foot of
/// the bar of the scroll. See T-254.
pub const THE_LETTER_OF_THE_KEY_DOWN: &str = "J";

/// Gives the scroll after one press of the key that moves the panel down.
///
/// The key reads the box that the render wrote at the frame before it. A panel
/// whose text ends inside the panel gives 0, therefore the key then changes
/// nothing at all.
pub fn the_scroll_after_one_step_down(now: u16) -> u16 {
    inside_the_text(
        now.saturating_add(1),
        THE_LAST_SCROLL.load(Ordering::Relaxed),
    )
}

/// Writes the box of the last scroll. A test needs this.
#[cfg(test)]
pub fn keep_the_last_scroll(value: u16) {
    THE_LAST_SCROLL.store(value, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A text of no line, of one line, and of many lines.
    ///
    /// The width of 20 characters holds "Charles Lutwidge" and no more.
    #[test]
    fn the_number_of_the_lines_reads_the_width_of_the_panel() {
        assert_eq!(the_number_of_the_lines("", 20), 1);
        assert_eq!(the_number_of_the_lines("A book.", 20), 1);
        assert_eq!(the_number_of_the_lines("one\ntwo\nthree", 20), 3);

        // 60 characters in a panel of 20 characters take three lines.
        assert_eq!(
            the_number_of_the_lines(
                "Charles Lutwidge Dodgson wrote under the name Lewis Carroll.",
                20
            ),
            4
        );

        // A word that is longer than the panel goes over more than one line.
        assert_eq!(the_number_of_the_lines("aaaaaaaaaa", 4), 3);

        // A panel of no width gives no division by zero.
        assert_eq!(the_number_of_the_lines("a b", 0), 2);
    }

    /// A text that ends inside the panel takes no scroll.
    #[test]
    fn a_text_that_ends_inside_the_panel_takes_no_scroll() {
        assert_eq!(the_last_scroll("A book.", 80, 4), 0);
        assert_eq!(the_last_scroll("", 80, 4), 0);
        assert_eq!(the_last_scroll("one\ntwo\nthree\nfour", 80, 4), 0);

        // A fifth line gives one scroll, and a sixth gives two.
        assert_eq!(the_last_scroll("one\ntwo\nthree\nfour\nfive", 80, 4), 1);
        assert_eq!(the_last_scroll("1\n2\n3\n4\n5\n6", 80, 4), 2);
    }

    /// **The parts of this test stay in one function**, because the box of the
    /// process holds one value for the whole binary of the test.
    #[test]
    fn the_key_that_moves_the_panel_down_stops_at_the_last_line() {
        // The render of the panel of the author of the measurement of T-252:
        // one line of text in a panel of four rows.
        let text = "Charles Lutwidge Dodgson wrote under the name Lewis Carroll.";

        assert_eq!(the_panel_of_the_render(0, text, 160, 4).scroll, 0);

        // **The key of that measurement took the whole line away.** It changes
        // nothing now.
        assert_eq!(the_scroll_after_one_step_down(0), 0);
        assert_eq!(the_scroll_after_one_step_down(23), 0);

        // A text of ten lines in a panel of four rows: the last scroll is six.
        let long = "1\n2\n3\n4\n5\n6\n7\n8\n9\n10";
        assert_eq!(the_panel_of_the_render(0, long, 160, 4).scroll, 0);
        assert_eq!(the_scroll_after_one_step_down(0), 1);
        assert_eq!(the_scroll_after_one_step_down(5), 6);
        assert_eq!(the_scroll_after_one_step_down(6), 6);
        assert_eq!(the_scroll_after_one_step_down(u16::MAX), 6);

        // The render of a scroll that stands above the text gives the last
        // line of it, therefore a panel that changed its text holds every
        // line of the new one.
        assert_eq!(the_panel_of_the_render(40, long, 160, 4).scroll, 6);
        assert_eq!(the_panel_of_the_render(40, text, 160, 4).scroll, 0);

        // The box takes a value of a test, and the key reads it.
        keep_the_last_scroll(3);
        assert_eq!(the_scroll_after_one_step_down(0), 1);
        assert_eq!(the_scroll_after_one_step_down(9), 3);
    }

    /// The bar of the scroll of the panel. See T-253.
    ///
    /// **The parts of this test stay in one function**, because the box of the
    /// process holds one value for the whole binary of the test.
    #[test]
    fn the_bar_of_the_scroll_comes_of_a_text_that_is_longer_than_the_panel() {
        // A text that ends inside the panel takes no bar, and it keeps the
        // whole width of the panel for its characters.
        let short = the_panel_of_the_render(0, "A book.", 80, 4);
        assert!(!short.the_bar_comes());
        assert_eq!(short.width_of_the_text, 80);
        assert_eq!(short.lines, 1);
        assert_eq!(short.last, 0);

        // A text of ten lines in a panel of four rows: the bar comes, and the
        // text then holds one character fewer of each line.
        let long = the_panel_of_the_render(0, "1\n2\n3\n4\n5\n6\n7\n8\n9\n10", 80, 4);
        assert!(long.the_bar_comes());
        assert_eq!(long.width_of_the_text, 79);
        assert_eq!(long.lines, 10);
        assert_eq!(long.last, 6);

        // **The number of the lines comes of the width of the text and not of
        // the width of the panel**: 40 characters take one line of a panel of
        // 40 characters, and two lines of a text of 39.
        let text = "1234567890123456789012345678901234567890\nb\nc\nd\ne";
        let panel = the_panel_of_the_render(0, text, 40, 4);
        assert!(panel.the_bar_comes());
        assert_eq!(panel.width_of_the_text, 39);
        assert_eq!(panel.lines, 6);
        assert_eq!(panel.last, 2);

        // A panel of one character holds the bar or the text, and the text
        // comes first: the width of the text is never 0.
        let narrow = the_panel_of_the_render(0, "a\nb\nc", 1, 1);
        assert_eq!(narrow.width_of_the_text, 1);
        assert!(!narrow.the_bar_comes());
        assert_eq!(narrow.last, 2);
    }

    /// The letters of the keys at the two ends of the bar. See T-254.
    ///
    /// **The parts of this test stay in one function**, because the box of the
    /// process holds one value for the whole binary of the test.
    #[test]
    fn the_bar_of_the_scroll_names_the_keys_that_move_the_panel() {
        // The panel of the measurement of T-254: the changelog of the settings
        // in a panel of many rows. The bar comes, and it names the two keys.
        let long = "1\n2\n3\n4\n5\n6\n7\n8\n9\n10";
        let panel = the_panel_of_the_render(0, long, 80, 4);
        assert!(panel.the_bar_comes());
        assert!(panel.the_letters_come());

        // A text that ends inside the panel takes no bar, therefore it names no
        // key at all.
        let short = the_panel_of_the_render(0, "A book.", 80, 4);
        assert!(!short.the_bar_comes());
        assert!(!short.the_letters_come());

        // **A bar of few rows keeps its track**: the two letters take one row
        // each, and a bar of three rows would then hold one row of the place of
        // the user. The bar of such a panel comes, and it names no key.
        for rows in 1..THE_ROWS_OF_THE_LETTERS {
            let small = the_panel_of_the_render(0, long, 80, rows);
            assert!(small.the_bar_comes(), "{} rows", rows);
            assert!(!small.the_letters_come(), "{} rows", rows);
        }

        // The letter of the key that moves the panel up stands at the top of
        // the bar, and the letter of the key that moves it down stands at the
        // foot of it. The keys of `App` are `K` and `J`.
        assert_eq!(THE_LETTER_OF_THE_KEY_UP, "K");
        assert_eq!(THE_LETTER_OF_THE_KEY_DOWN, "J");

        // A panel of one character holds the text alone, therefore it names no
        // key.
        let narrow = the_panel_of_the_render(0, long, 1, 40);
        assert!(!narrow.the_bar_comes());
        assert!(!narrow.the_letters_come());
    }
}
