//! The scroll of the panel of a description. See T-252 and T-253.
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
/// characters.
///
/// The panels of this program draw with `Wrap { trim: true }`: the text goes to
/// the line after it at a space, a line of the text that holds no character
/// takes one line of the panel, and the spaces at the start of a line go away.
/// A word that is longer than the panel takes more than one line.
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
        let size = word.chars().count();

        if length == 0 {
            length = size;
        } else if length + 1 + size <= width {
            length += 1 + size;
        } else {
            count += 1;
            length = size;
        }

        // A word that is longer than the panel goes over more than one line.
        while length > width {
            count += 1;
            length -= width;
        }
    }

    count
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
}

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
    }
}

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
}
