//! The scroll of the list of a view. See T-255 and T-256.
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
//!
//! # The place of the cursor, and the place of the panel. See T-256.
//!
//! T-255 gave the bar the **offset** of the list: the first line that the panel
//! draws. That offset does not change while the cursor moves inside the rows of
//! the panel, therefore the bar said nothing of the first 18 lines of a list.
//! The measurement of 2026-08-15, of the real program v0.8.84 inside tmux, of
//! the view of the episodes of "Letters of Two Brides" (57 episodes in a panel
//! of 18 rows), on a screen of 160 columns and 45 rows:
//!
//! ```text
//! ───────────────Episodes [57 items]───────────────
//! ➤ 3%  Letter 1                                  █     the first frame:
//!       Letter 2                                  █     the thumb holds the
//!   ✓   Letter 3                                  █     rows 4 to 9
//! ```
//!
//! 17 presses of the key `j` took the cursor from the line 1 to the line 18 —
//! **the whole of the panel** — and the thumb held the rows 4 to 9 again.
//!
//! **The title of this item is "the list of a view says where the cursor of the
//! user stands", and the offset is not the place of the cursor.** The bar reads
//! the selected line of the `ListState` now, and the track of it counts the
//! lines of the list and not the offsets of the panel: the thumb then stands at
//! the top of the track at the first line and at the foot of it at the last
//! line, and every press of the key `j` of a list of few lines moves it.
//!
//! A list with no selected line keeps the offset: the bar then says the place
//! of the panel, which is the one place that such a list has.

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
    /// The track of the bar: the largest place of the thumb. It is one less
    /// than the number of the lines, because the thumb stands at the top of
    /// the track at the first line and at the foot of it at the last one.
    /// See T-256.
    pub the_track: usize,
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
        the_track: lines.saturating_sub(1),
        the_bar_comes,
    }
}

/// Gives the place of the thumb of the bar of a list.
///
/// **The place of the cursor of the user comes first** (T-256): a key `j` that
/// moves the cursor inside the rows of the panel changes no offset, therefore a
/// bar of the offset alone says nothing of the first panel of a list.
///
/// `selected` is the line of the cursor of the `ListState`, and `offset` is the
/// first line that the panel draws. ratatui writes the offset while it draws
/// the list. A list with no cursor keeps the offset: the place of the panel is
/// the one place that such a list has.
///
/// The value never goes past the foot of the track.
///
/// The function is pure, therefore a test needs no screen.
pub fn the_place_of_the_bar(selected: Option<usize>, offset: usize, the_track: usize) -> usize {
    selected.unwrap_or(offset).min(the_track)
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

    /// The place of the thumb of the bar. See T-255 and T-256.
    #[test]
    fn the_thumb_of_the_bar_holds_the_line_of_the_cursor() {
        // The list of the measurement of T-256: 57 episodes in a panel of 18
        // rows. The track counts the lines of the list and not the offsets of
        // the panel.
        let the_list = the_list_of_the_render(57, 160, 18);
        assert_eq!(the_list.the_track, 56);
        assert_eq!(the_list.last, 39);

        // **The cursor of the user comes first** (T-256). The 17 presses of the
        // key `j` of that measurement took the cursor over the whole of the
        // panel, and the offset of the list stayed at 0 the whole time.
        assert_eq!(the_place_of_the_bar(Some(0), 0, 56), 0);
        assert_eq!(the_place_of_the_bar(Some(17), 0, 56), 17);

        // The first line and the last line of the list stand at the two ends of
        // the track.
        assert_eq!(the_place_of_the_bar(Some(56), 39, 56), 56);

        // A list with no cursor keeps the offset of the panel.
        assert_eq!(the_place_of_the_bar(None, 13, 56), 13);
        assert_eq!(the_place_of_the_bar(None, 0, 56), 0);

        // A place above the foot of the track takes the thumb no further than
        // that foot.
        assert_eq!(the_place_of_the_bar(Some(600), 0, 56), 56);
        assert_eq!(the_place_of_the_bar(None, 600, 56), 56);
        assert_eq!(the_place_of_the_bar(Some(usize::MAX), 0, 0), 0);

        // A list of no line holds no track at all.
        assert_eq!(the_list_of_the_render(0, 160, 18).the_track, 0);
    }
}
