//! The panel 5 of the cover, and the words of the media inside it. See T-319.
//!
//! **The maintainer chose the mockup 1, the panels, on 2026-08-16**, and the
//! stage 6 of that road is the panel 5 of the cover: the column at the right of
//! the list holds a picture and nothing else.
//!
//! **The fault, of the real program v0.8.150 inside tmux**, of the Library view
//! of the library `Books` of the sandbox on a screen of 160 columns and 45
//! rows, with the cursor on `Alice in Wonderland`:
//!
//! ```text
//! │                                │║➤   Alice in Wonderland   <1m   -█║   ▀▀▄▀▀▀▄▀▀▄▀▀▀▀▀▀▀▄▄▄▄
//! │                                │╚═════════════════════════════════╝   ▀▀▀▀▄▀▄▀▀▄▀▀▄▄▄▄▀▀▀▄▀▀
//! │                                │Author: Lewis Carroll - Year: N/A     ▀▄▄▄▄▄▀▀▄▀▀▀▄▄▄▀▀▀▀▄▄▄
//! │                                │Progress: 0%,  Not finished           ▀▄▄▄▀▀▀▀▀▄▄▄▄▄▄▀▄▄▄▄▄
//! ```
//!
//! **The picture stood in the air**: no border, no title, and no number of a
//! panel, therefore no key and no click of the user could name it. The nine
//! rows under it held no character at all, and the words of the media stood at
//! the left of it, under the list, where they took four rows of the list away.
//!
//! **A media that the server holds with no cover gave a column of nothing.**
//! The same measurement with the cursor on `A Book Of An Epub With No
//! Container`: the 50 columns and the 41 rows of that column held no character
//! at all, which is 2050 cells of the screen of the user with no work.
//!
//! **The correction, of the same harness**: the column holds the panel 5, and
//! the words of the media stand inside it.
//!
//! ```text
//! ╔4 Library [18 items] ════════════════════════╗ ┌5 Cover ─────────────────┐
//! ║    Title              Author    Time   Done ║ │Author: Lewis Carroll    │
//! ║➤   Alice in Wonderland Lewis…   <1m      -  ║ │Progress: 0%, Not fini…  │
//! ║    Depthless Hunger, Book [1 book]          ║ │                         │
//! ║  ✓ One File With No Decoder     30m   done  ║ │No description available │
//! ╚═════════════════════════════════════════════╝ └─────────────────────────┘
//! ```
//!
//! **The list of the panel 4 grew from 16 lines to 20**, because the facts and
//! the description left the column of the list.
//!
//! **The key `5` gives the panel the focus**, and the border of it says so:
//! `╔5 Cover ═══╗` against `┌4 Library [18 items] ───┐`, with the footer
//! `j/k: the description  l: play or open  h: the list  4/Ctrl+h: the list`.
//! Three keys `j` of that focus moved the description of `Letters of Two
//! Brides` from its first line to its third one, and the key `h` gave the focus
//! back to the panel 4.

use ratatui::layout::Rect;
use toutui::ui::frame::ThePanel;
use toutui::ui::the_panel_of_the_cover::{
    the_parts_of_the_panel, THE_ROWS_OF_THE_FACTS, THE_SMALLEST_PANEL_OF_THE_WORDS,
};

/// The panel of a screen of 160 columns holds the picture and the words of the
/// media, and no row of it stays empty. See T-319.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_panel_of_the_cover_holds_the_picture_and_the_words() {
    // The measurement of 2026-08-16: the column of the covers of a screen of
    // 160 by 45 stands at the column 110 and it holds 41 rows. The border of
    // the panel takes one row and one column at each end.
    let inside = Rect::new(111, 3, 48, 39);
    let parts = the_parts_of_the_panel(inside, true);

    let cover = parts
        .cover
        .expect("a picture comes, therefore the panel gives it an area");

    // **No row of the panel goes away, and no row holds two parts**: the nine
    // rows under the picture of the fault held nothing at all.
    assert_eq!(
        cover.height + parts.facts.height + parts.description.height,
        inside.height,
        "the three parts must fill the panel"
    );
    assert_eq!(cover.y, inside.y);
    assert_eq!(parts.facts.y, cover.y + cover.height);
    assert_eq!(parts.description.y, parts.facts.y + parts.facts.height);

    // The facts keep the three rows that the area under the list gave them.
    assert_eq!(parts.facts.height, THE_ROWS_OF_THE_FACTS);
    assert!(parts.the_words_stand_here());

    // Every part keeps the width of the panel.
    for part in [cover, parts.facts, parts.description] {
        assert_eq!(part.x, inside.x);
        assert_eq!(part.width, inside.width);
    }
}

/// A media that the server holds with no cover gives every row of the panel to
/// the words. See T-319.
///
/// **A picture that never comes must take no row of the screen of the user**:
/// the column of such a media held 2050 cells with no character at all.
///
/// **The parts of this test stay in one function.**
#[test]
fn a_media_with_no_cover_gives_every_row_to_the_words() {
    let inside = Rect::new(111, 3, 48, 39);
    let parts = the_parts_of_the_panel(inside, false);

    assert_eq!(parts.cover, None, "no picture comes, therefore no area");
    assert_eq!(parts.facts, Rect::new(111, 3, 48, THE_ROWS_OF_THE_FACTS));
    assert_eq!(
        parts.facts.height + parts.description.height,
        inside.height,
        "the words must fill the panel of a media with no cover"
    );
    assert!(parts.the_words_stand_here());

    // A panel that is not tall still says the words of such a media, because
    // the picture takes no row at all.
    let parts = the_parts_of_the_panel(Rect::new(111, 3, 48, 4), false);
    assert_eq!(parts.cover, None);
    assert!(parts.the_words_stand_here());
}

/// A panel that is not tall holds the picture alone, and the words then stay
/// under the list. See T-319.
///
/// **The words of a media stand in one place of one frame**: a panel that says
/// them takes them away from the area under the list, therefore a panel with no
/// room for them must say that it has none.
///
/// **The parts of this test stay in one function.**
#[test]
fn a_panel_that_is_not_tall_leaves_the_words_under_the_list() {
    let parts = the_parts_of_the_panel(
        Rect::new(111, 3, 48, THE_SMALLEST_PANEL_OF_THE_WORDS - 1),
        true,
    );

    assert!(
        !parts.the_words_stand_here(),
        "a panel of {} rows holds no word of the media",
        THE_SMALLEST_PANEL_OF_THE_WORDS - 1
    );
    assert_eq!(
        parts.cover.map(|of| of.height),
        Some(THE_SMALLEST_PANEL_OF_THE_WORDS - 1),
        "the picture then takes the whole panel"
    );

    // One row more, and the three parts stand together.
    let parts =
        the_parts_of_the_panel(Rect::new(111, 3, 48, THE_SMALLEST_PANEL_OF_THE_WORDS), true);
    assert!(parts.the_words_stand_here());
    assert!(parts.cover.is_some());

    // A screen that draws no panel at all gives no part: that is the terminal
    // of 40 columns of the measurement, where the covers go away with the
    // second column (T-320).
    let parts = the_parts_of_the_panel(Rect::default(), true);
    assert_eq!(parts.cover, None);
    assert!(!parts.the_words_stand_here());
}

/// The digit `5` names the panel of the cover, and the key of the panel at the
/// right of the list gives that panel too. See T-319.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_digit_of_the_panel_of_the_cover_names_it() {
    assert_eq!(ThePanel::of_the_digit('5'), Some(ThePanel::TheCover));
    assert_eq!(ThePanel::TheCover.the_number(), 5);

    // **The panel 5 stands at the right of the panel 4**, and the panel at the
    // right of it is that panel itself: the panel 6 of the gallery comes with
    // the rest of T-319.
    assert_eq!(ThePanel::TheList.at_the_right(), ThePanel::TheCover);
    assert_eq!(ThePanel::TheCover.at_the_right(), ThePanel::TheCover);
    assert_eq!(ThePanel::TheCover.at_the_left(), ThePanel::TheList);

    // The panel 5 stands in no stack, therefore the keys `Ctrl+j` and `Ctrl+k`
    // move no focus there.
    assert!(!ThePanel::TheCover.is_of_the_stack());
    assert_eq!(ThePanel::TheCover.below(), ThePanel::TheCover);
    assert_eq!(ThePanel::TheCover.above(), ThePanel::TheCover);

    // **The digit 6 of the gallery is no key of this program yet** (T-79).
    assert_eq!(ThePanel::of_the_digit('6'), None);
}

/// The footer of the panel 5 names the keys of that panel, and the view of the
/// key `?` names its digit. See T-319.
///
/// **A footer must not promise a key that the view does not hold** (T-143), and
/// a key that the user cannot find is a key that the program does not have.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_words_of_the_keys_of_the_panel_of_the_cover_stand() {
    let of_the_view = toutui::ui::keys::FOOTER_OF_A_LIBRARY_OF_BOOKS;
    let footer =
        toutui::ui::keys::the_footer_of_a_panel(of_the_view, true, true, ThePanel::TheCover);

    // The keys `j` and `k` of this panel move the description of the media,
    // and not the line of a list.
    assert!(footer.contains("j/k: the description"), "{footer:?}");
    assert!(!footer.contains("j/k: move"), "{footer:?}");

    // **The key `h` gives the focus back to the panel 4** (the trap 210), and
    // the key `l` plays the media, which is the button `[l Play]` of the
    // design.
    assert!(footer.contains("h: the list"), "{footer:?}");
    assert!(footer.contains("l: play or open"), "{footer:?}");

    // A screen that holds no frame keeps the footer of the view.
    assert_eq!(
        toutui::ui::keys::the_footer_of_a_panel(of_the_view, false, false, ThePanel::TheCover),
        of_the_view
    );

    let text = toutui::ui::keys::lines().join("\n");
    assert!(
        text.contains("The focus goes to the panel 5 of the cover"),
        "the view of the key ? must name the digit of the panel 5"
    );
    assert!(
        text.contains("The panel 5 moves the description of the media"),
        "the view of the key ? must name the keys j and k of the panel 5"
    );
    // **The panel 6 of the gallery holds no key yet** (T-79).
    assert!(!text.contains("panel 6"), "{text}");
}

/// A click of the panel 5 names that panel, and a click of it on a screen of
/// two columns names nothing. See T-319.
///
/// **The focus of a panel belongs to the shape of three columns alone**
/// (T-320): a screen of two columns draws the panel 5 and it takes no key of a
/// panel, therefore a click that gave it the focus would show a border of a
/// focus that no key of the user can use (T-79).
///
/// **The parts of this test stay in one function.**
#[test]
fn a_click_of_the_panel_of_the_cover_names_that_panel() {
    use toutui::ui::the_mouse::{the_target_of_a_point, TheAreasOfTheMouse, TheTarget};

    let areas = TheAreasOfTheMouse {
        the_panel_of_the_list: Rect::new(34, 2, 75, 41),
        the_lines_of_the_list: Rect::new(35, 4, 73, 38),
        the_lines: 18,
        the_panel_of_the_cover: Rect::new(110, 2, 50, 41),
        ..TheAreasOfTheMouse::default()
    };

    // The measurement of 2026-08-16: a click at the column 130 of the row 20
    // gave the panel 5 the focus, and the footer of that panel came.
    assert_eq!(
        the_target_of_a_point(&areas, true, 130, 20),
        TheTarget::ThePanelOfTheCover
    );

    // The border of the panel names it too, as the border of every other panel
    // does.
    assert_eq!(
        the_target_of_a_point(&areas, true, 110, 2),
        TheTarget::ThePanelOfTheCover
    );

    // A screen of two columns draws no stack, and a click of the panel 5 of it
    // names nothing.
    assert_eq!(
        the_target_of_a_point(&areas, false, 130, 20),
        TheTarget::Nothing
    );

    // The column between the panel 4 and the panel 5 belongs to no panel.
    assert_eq!(
        the_target_of_a_point(&areas, true, 109, 20),
        TheTarget::Nothing
    );

    // A frame that draws no panel 5 takes no click of it: that is the value of
    // the start, before the first frame.
    assert_eq!(
        the_target_of_a_point(&TheAreasOfTheMouse::default(), true, 130, 20),
        TheTarget::Nothing
    );
}
