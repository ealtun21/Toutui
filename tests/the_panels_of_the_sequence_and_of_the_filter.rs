//! The panels 2 and 3 of the stack, and the words of the header. See T-318.
//!
//! **The maintainer chose the mockup 1, the panels, on 2026-08-16**, and the
//! stage 5 of that road is the panel 2 of the sequence and the panel 3 of the
//! filter. The stack of 34 columns held the panel 1 of the views alone.
//!
//! **The fault, of the real program v0.8.148 inside tmux**, of the Library view
//! of the library `Large` of the sandbox on a screen of 160 columns and 45
//! rows:
//!
//! ```text
//! ┌1 Views ────────────────────────┐╔4 Library [500 items of 2056] ══════════╗
//! │➤ Home                       Tab│║    Title              Author  Time Done║
//! │  Library                    Tab│║➤   Large Book 2056             <1m    -█
//! │  Sequence and filter          f│║    Large Book 2055             <1m    -│
//! …
//! │                                │
//! └────────────────────────────────┘
//!      j/k: move  l: play or open  Tab: home/library  S-Tab: the next library
//!      /: search  R: refresh  ?: every key  Q: quit  1/Ctrl+h: the panel …
//! ```
//!
//! **The screen said no word of the sequence that stands and no word of the
//! filter that stands**, and the footer named no key `f`: the user could read
//! those two values in the view of the key `f` alone, and that view takes the
//! whole screen and it hides the list that it describes. **The 20 rows under
//! the last line of the panel 1 held nothing at all.**
//!
//! **The correction, of the same harness**: the stack holds three panels, and
//! the key `2` and the keys `j j l` gave the sequence of the author.
//!
//! ```text
//! ┌1 Views ────────────────────────┐┌4 Library [500 items of 2056] — The author
//! │➤ Home                       Tab││    Title              Author  Time Done
//! └────────────────────────────────┘
//! ╔2 Sequence ═════════════════════╗
//! ║    The title                   ║
//! ║    The title, with no "A" and… ║
//! ║➤ ✓ The author                  ║
//! ║    The time when the book came ║
//! ║    The year                    ║
//! ║    The length                  ║
//! ║    The size on the disk        ║
//! ║    The direction: the largest… ║
//! ╚════════════════════════════════╝
//! ┌3 Filter ───────────────────────┐
//! │➤ ✓ No filter                   │
//! │    The media that you finished │
//! │    The media that you started  │
//! │    The media that you did not… │
//! └────────────────────────────────┘
//!      j/k: move  l: this filter  h: the list  4/Ctrl+l: the list  ?: every key
//! ```
//!
//! **A terminal of 100 columns draws no stack**, and the header of that screen
//! then holds the words (the decision 3 of the road of the panels):
//!
//! ```text
//! 👋 Connected as toutuitest      📖 Large (book)        🦜 Toutui v0.8.148
//! 🔗 localhost:13399        ⇅ The sequence of the server ▣ No filter
//! ```
//!
//! **A terminal of 22 rows loses the panel 3**, and the digit `3` of that
//! screen did nothing at all: a key of a panel that the frame did not draw is
//! the fault of T-79.

use toutui::ui::frame::ThePanel;
use toutui::ui::the_panels_of_the_stack as the_panels;

/// The footer of each panel of the stack names the work of its own key `l`,
/// and the footer of the panel 4 names the key of the sequence. See T-318.
///
/// **A footer must not promise a key that the view does not hold** (T-143), and
/// the reverse of that rule holds too: the key `f` opened the sequence and the
/// filter, it stood in the panel 1 and in no footer, and a user who cannot find
/// a key has no key at all.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_footer_of_a_panel_of_the_stack_names_the_work_of_its_key() {
    let of_the_view = toutui::ui::keys::FOOTER_OF_A_LIBRARY_OF_BOOKS;

    // A screen that holds no frame keeps the footer of the view, with no word
    // of a panel at all.
    assert_eq!(
        toutui::ui::keys::the_footer_of_a_panel(of_the_view, false, false, ThePanel::TheSequence),
        of_the_view
    );

    // **The key `l` of the panel 2 takes a sequence, and the key `l` of the
    // panel 3 takes a filter**: a footer of `l: play or open` at that moment
    // would name a work that the key does not do.
    let of_the_sequence =
        toutui::ui::keys::the_footer_of_a_panel(of_the_view, true, true, ThePanel::TheSequence);
    assert!(
        of_the_sequence.contains("l: this sequence"),
        "{of_the_sequence:?}"
    );

    let of_the_filter =
        toutui::ui::keys::the_footer_of_a_panel(of_the_view, true, true, ThePanel::TheFilter);
    assert!(
        of_the_filter.contains("l: this filter"),
        "{of_the_filter:?}"
    );

    for footer in [&of_the_sequence, &of_the_filter] {
        assert!(
            !footer.contains("play or open"),
            "the footer of a panel of the stack must not promise a playback: {footer:?}"
        );
        // **The key `h` of a panel gives the focus back and it takes no view
        // away** (the trap 210), and the footer says which one it is.
        assert!(footer.contains("h: the list"), "{footer:?}");
    }

    // **The footer of the panel 4 names the key `f`**: the measurement of the
    // real program v0.8.148 gave a footer of eight keys and no `f` in it.
    let of_the_list =
        toutui::ui::keys::the_footer_of_a_panel(of_the_view, true, true, ThePanel::TheList);
    assert!(of_the_list.starts_with(of_the_view));
    assert!(
        of_the_list.contains("f: sequence"),
        "the footer of the panel of the list must name the key of the sequence: {of_the_list:?}"
    );
}

/// The view of the key `?` names the digits of the panel 2 and of the panel 3,
/// and the two keys of the focus of the stack. See T-318.
///
/// **A key that the user cannot find is a key that the program does not have**,
/// and the view of the key `?` is the list of every key of this program.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_view_of_the_keys_names_the_keys_of_the_panels_of_the_stack() {
    let lines = toutui::ui::keys::lines();
    let text = lines.join("\n");

    for what in [
        "The focus goes to the panel 2 of the sequence",
        "The focus goes to the panel 3 of the filter",
        "The focus goes to the panel below",
        "The focus goes to the panel above",
    ] {
        assert!(text.contains(what), "the view of the keys says no {what:?}");
    }

    // **The keys of the focus of the stack are `Ctrl+j` and `Ctrl+k`** (the
    // decision 2 of the road of the panels): `Tab` is the Home view and the
    // Library view, and `Shift+Tab` is the next library (the trap 196),
    // therefore neither of them can move the focus.
    for key in ["Ctrl+j", "Ctrl+k"] {
        assert!(
            lines.iter().any(|line| line.contains(key)),
            "the view of the keys names no key {key}"
        );
    }
}

/// Every row of the panel 2 and of the panel 3 is a row that the program acts
/// on, and the two panels and the view of the key `f` hold one rule. See T-318.
///
/// **A key that does nothing is a fault of its own** (T-79): a row of a title
/// or of a note takes the key `l` and it changes no request at all, therefore
/// the two panels hold no such row.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_two_panels_and_the_view_of_the_key_f_hold_one_rule() {
    use toutui::logic::sort_filter::Row;

    let of_the_sequence = the_panels::the_rows_of_the_sequence(false);
    let of_the_filter = the_panels::the_rows_of_the_filter();

    for row in of_the_sequence.iter().chain(of_the_filter.iter()) {
        assert!(
            row.is_a_line_of_the_user(),
            "the row {row:?} of a panel of the stack takes no key of the user"
        );
    }

    // **Every row of the two panels stands in the rows of the view of the key
    // `f`**: the two roads write one field of the request of the items, and a
    // panel that named a sequence which that view does not hold would be a text
    // that promises a function that the program does not have (T-118).
    let of_the_view = toutui::logic::sort_filter::rows(false, &[], None);

    for row in of_the_sequence.iter().chain(of_the_filter.iter()) {
        assert!(
            of_the_view.contains(row),
            "the row {row:?} of a panel of the stack stands in no row of the view of the key f"
        );
    }

    // **The source holds one function for the work of a row**: the view of the
    // key `f` and the panels of the stack call
    // `apply_the_row_of_the_sequence_or_the_filter`, therefore no second road
    // of this program writes `library_sort`, `library_desc`, or
    // `library_filter` for a row of the user.
    let source = std::fs::read_to_string("src/app.rs").expect("src/app.rs");
    assert_eq!(
        source
            .matches("fn apply_the_row_of_the_sequence_or_the_filter")
            .count(),
        1,
        "the work of a row of the sequence must stand in one function"
    );
    assert!(
        source
            .matches("apply_the_row_of_the_sequence_or_the_filter")
            .count()
            >= 3,
        "the view of the key f and the two panels of the stack must call that one function"
    );

    // The last row of the panel 2 is the direction, and the first row of the
    // panel 3 is the line that removes the filter.
    assert_eq!(of_the_sequence.last(), Some(&Row::Direction));
    assert_eq!(of_the_filter.first(), Some(&Row::NoFilter));
}

/// The words of the header of a screen that draws no stack name the sequence
/// and the filter of the library. See T-318.
///
/// **The stack stands at 120 columns and up**, therefore the panel 2 and the
/// panel 3 say those two values there; a terminal of 84 to 119 columns holds
/// neither of them, and the decision 3 of the road of the panels gives the
/// words of it to the header.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_header_of_a_screen_of_no_stack_names_the_sequence_and_the_filter() {
    // The measurement of tmux at 100 columns, of the library `Large` of the
    // sandbox at the start of the program:
    // `🔗 localhost:13399        ⇅ The sequence of the server ▣ No filter`
    let of_the_start =
        the_panels::the_words_of_the_sequence_and_the_filter(false, "", false, "", &[]);
    assert_eq!(of_the_start, "⇅ The sequence of the server ▣ No filter");

    // The sequence of the author, the largest first, and the media that the
    // user started.
    let of_the_place = toutui::logic::sort_filter::filter_value("progress", "in-progress");
    let words = the_panels::the_words_of_the_sequence_and_the_filter(
        false,
        "media.metadata.authorNameLF",
        true,
        &of_the_place,
        &[],
    );
    assert_eq!(
        words,
        "⇅ The author, the largest first ▣ The media that you started"
    );

    // **The words keep the width of a narrow terminal** (T-301): the header of
    // 84 columns holds the account at the left and the name of the program at
    // the right, therefore these words must not take the whole row.
    assert!(
        toutui::logic::message::the_columns_of(&words) <= 60,
        "the words of the header hold {} columns: {words:?}",
        toutui::logic::message::the_columns_of(&words)
    );

    // **The words stand for the shape of two columns alone**: a screen that
    // draws the stack holds the panel 2 and the panel 3, and the same two
    // values on the header and on two panels are one value two times.
    let source = std::fs::read_to_string("src/ui/tui.rs").expect("src/ui/tui.rs");
    let at = source
        .find("the_words_of_the_sequence_and_the_filter")
        .expect("the header of src/ui/tui.rs writes the words");
    let before = &source[at.saturating_sub(700)..at];
    assert!(
        before.contains("if !self.the_stack_of_the_panels_stands()"),
        "the header must write the words for a screen that draws no stack alone"
    );
}
