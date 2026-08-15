//! The scroll of the list of a view. See T-255.
//!
//! **A list that holds more lines than its rows says so.** The measurement of
//! 2026-08-15, of the real program v0.8.83 inside tmux, against the sandbox:
//! the Library view of the library `ManyPods` of 520 podcasts, on a screen of
//! 160 columns and 45 rows. The list drew 18 of the 500 lines of the filter,
//! and 30 presses of the key `j` took the cursor to the line 31:
//!
//! ```text
//! ──────────Library [500 items of 520] — a filter is on (f)──────────
//! ➤     Many Podcast 520              |       Many Podcast 507
//!       Many Podcast 519              |       Many Podcast 506
//!       ...                           |       ...
//!       Many Podcast 503              | ➤     Many Podcast 490
//! ```
//!
//! **The two frames hold the same number of characters and no other mark at
//! all.** The title says how many items the list holds, and no character of
//! the screen says where in that list the cursor of the user stands, or that
//! any line is left below it.
//!
//! T-253 gave the panel of a description the bar of its scroll. The list of a
//! view is the part of the screen that the user moves through the most, and it
//! held none: `render_list` of `src/ui/tui.rs` drew a `List` of ratatui with no
//! bar at all, and the 24 views of that function each took the fault.
//!
//! **The bar of a list names no key** (T-254 gave the bar of a panel the
//! letters `K` and `J`, because no footer of the program has room for those
//! words). The footer of every view of a list says `j/k: move` already,
//! therefore the letters of that bar would say a second time what the footer
//! says. The bar of a list keeps the whole of its track for the place of the
//! user.
//!
//! **The render is the one road to the place of the bar.** The offset of the
//! list — the first line that the panel draws — belongs to the `ListState` of
//! ratatui, and ratatui writes it while it draws the list. The render
//! therefore draws the list first and it reads that offset after, and no part
//! of this program has to say the offset a second time.

/// What the render of the list of a view draws. See T-255.
#[derive(Debug, PartialEq, Eq)]
pub struct TheList {
    /// The width of the lines. It is one character less than the width of the
    /// panel when the bar of the scroll stands beside them.
    pub width_of_the_lines: u16,
    /// The largest offset of the list: the number of the lines that stand
    /// above the panel when the last line of the list stands in it. 0 says
    /// that the panel holds every line, and the bar then does not come.
    pub last: usize,
    /// The bar of the scroll stands beside the lines. A panel of one character
    /// holds the bar or the lines, and the lines come first.
    the_bar_comes: bool,
}

impl TheList {
    /// Says that the bar of the scroll stands beside the lines.
    ///
    /// **This is the one line of the correction of T-255**: a build with
    /// `false` in the place of `self.the_bar_comes` gives the list of 500
    /// podcasts that says nothing of its length again.
    pub fn the_bar_comes(&self) -> bool {
        self.the_bar_comes
    }
}

/// Gives everything that the render of the list of a view needs.
///
/// `lines` is the number of the lines of the list, `width` and `rows` are the
/// size of the panel that holds them — the area inside the header of the view.
///
/// **One line of the list takes one row of the panel.** A `List` of ratatui
/// cuts a line that is longer than the panel, and it wraps no line, therefore
/// the number of the rows of the list is the number of its lines.
///
/// **The bar takes one character of the width of the lines.** A line that
/// reaches the bar loses its last character, in the same way as the text of a
/// panel of a description does (T-253). The number of the rows does not change
/// with the width, therefore the bar of a list comes and goes with no decision
/// that reads its own output (the trap 226).
///
/// The function is pure, therefore a test needs no screen.
pub fn the_list_of_the_render(lines: usize, width: u16, rows: u16) -> TheList {
    let last = lines.saturating_sub(usize::from(rows));

    // A panel of one character holds the bar or the lines, and the lines come
    // first.
    let the_bar_comes = width >= 2 && last > 0;

    let width_of_the_lines = if the_bar_comes { width - 1 } else { width };

    TheList {
        width_of_the_lines,
        last,
        the_bar_comes,
    }
}

/// Keeps the offset of the list inside the track of the bar.
///
/// ratatui writes the offset of the `ListState` while it draws the list, and
/// that offset stands between 0 and the largest offset. A value above the
/// largest offset would take the thumb of the bar past the foot of its track.
///
/// The function is pure, therefore a test needs no screen.
pub fn the_place_of_the_bar(offset: usize, last: usize) -> usize {
    offset.min(last)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The bar of the scroll of a list. See T-255.
    #[test]
    fn the_bar_of_the_scroll_comes_of_a_list_that_is_longer_than_the_panel() {
        // The list of the measurement of T-255: 500 podcasts in a panel of 18
        // rows. The bar comes, and the lines then hold one character fewer.
        let many = the_list_of_the_render(500, 160, 18);
        assert!(many.the_bar_comes());
        assert_eq!(many.width_of_the_lines, 159);
        assert_eq!(many.last, 482);

        // The control of that same measurement: a list that holds every line
        // of it in the panel takes no bar, and it keeps the whole width.
        let few = the_list_of_the_render(2, 160, 18);
        assert!(!few.the_bar_comes());
        assert_eq!(few.width_of_the_lines, 160);
        assert_eq!(few.last, 0);

        // A list of no line, and a list of the number of the rows of the
        // panel: the last line of each of them stands in the panel already.
        assert!(!the_list_of_the_render(0, 160, 18).the_bar_comes());
        assert!(!the_list_of_the_render(18, 160, 18).the_bar_comes());

        // One line more than the rows of the panel gives the bar.
        let one_more = the_list_of_the_render(19, 160, 18);
        assert!(one_more.the_bar_comes());
        assert_eq!(one_more.last, 1);

        // A panel of one character holds the bar or the lines, and the lines
        // come first: the width of the lines is never 0.
        let narrow = the_list_of_the_render(500, 1, 18);
        assert!(!narrow.the_bar_comes());
        assert_eq!(narrow.width_of_the_lines, 1);
        assert_eq!(narrow.last, 482);

        // A panel of no row: every line of the list stands above it.
        assert_eq!(the_list_of_the_render(500, 160, 0).last, 500);
    }

    /// The place of the thumb of the bar. See T-255.
    #[test]
    fn the_thumb_of_the_bar_stays_inside_its_track() {
        // The offset of the first frame of the measurement of T-255, and the
        // offset of the last line of that list.
        assert_eq!(the_place_of_the_bar(0, 482), 0);
        assert_eq!(the_place_of_the_bar(13, 482), 13);
        assert_eq!(the_place_of_the_bar(482, 482), 482);

        // An offset above the largest offset takes the thumb no further than
        // the foot of the track.
        assert_eq!(the_place_of_the_bar(600, 482), 482);
        assert_eq!(the_place_of_the_bar(usize::MAX, 0), 0);
    }
}
