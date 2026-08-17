//! The panel 5 of the cover, and the words of the media beside it. See T-319.
//!
//! **The maintainer chose the mockup 1, the panels, on 2026-08-16**, therefore
//! `docs/mockups/mockup-1.txt` is the design of the program now. This module
//! holds the stage 6 of that road: the column at the right of the list gets a
//! frame, and the words of the media of the cursor go inside it.
//!
//! ## The screen before this stage
//!
//! **The measurement of the real program v0.8.150 inside tmux**, of the Library
//! view of the library `Books` of the sandbox at 160 columns and 45 rows. The
//! panel of the covers of T-23 stands on the columns 110 to 160, and it holds a
//! picture and nothing else:
//!
//! ```text
//! │  Every key                    ?│║    Second Series [3 books]             █║       ▀▀▀▀▀▀▀▄▄▄▄▀▄▄▄▀▀▄▀▀▀▄▄▄▄▄▄
//! │                                │║    Multi File Test Book      1m     -  █║       ▀▄▄▄▄▄▀▀▀▄▄▀▀▀▄▄▄▄▄▄▀▄▀▀▄▄▄▄
//! │                                │║➤   Alice in Wonderland      <1m     -  █║       ▀▀▄▀▀▀▄▀▀▄▀▀▀▀▀▀▀▄▄▄▄▄▄▄▄▄▄▄
//! │                                │╚════════════════════════════════════════╝        ▀▀▀▀▄▀▄▀▀▄▀▀▄▄▄▄▀▀▀▄▀▀▄▄▀ ▄▄
//! │                                │Author: Lewis Carroll - Year: N/A - Duration: 0m  ▀▄▄▄▄▄▀▀▄▀▀▀▄▄▄▀▀▀▀▄▄▄▀▀▀▀▀▀▀
//! │                                │Progress: 0%,  Not finished                       ▀▄▄▄▀▀▀▀▀▄▄▄▄▄▄▀▄▄▄▄▄▀▀▀▀▀▄▄
//! ```
//!
//! **The picture stands in the air**: it has no border, no title, and no
//! number of a panel, therefore no key and no click of the user can name it.
//! The nine rows under it hold no character at all, and the words of the media
//! stand at the left of it, under the list, where they take four rows of the
//! list away.
//!
//! **A media with no cover gives a column of nothing.** The same measurement,
//! with the cursor on `A Book Of An Epub With No Container`, which the server
//! holds with no cover: the 50 columns and the 41 rows of that column held no
//! character at all. That is 2050 cells of the screen of the user with no work.
//!
//! ## The rule of this module
//!
//! **The panel of the covers holds the words of the media too**, therefore the
//! column fills with the picture, with the facts of the media, and with the
//! description of it, and the list at the left of it takes every row that it
//! has. [`the_parts_of_the_panel`] is that arithmetic, and it is pure: a test
//! of it needs no terminal, no server, and no `App` at all.
//!
//! **A media that the server holds with no cover gives its rows to the words**
//! (`a_picture_comes` of that function), because a picture that never comes
//! must take no row of the screen.
//!
//! ## The picture takes every free row (T-330.3)
//!
//! **The first form of this module gave the picture a share of the height**,
//! and a tall panel therefore held a small picture over rows of nothing. The
//! measurement of the real program v0.8.161 inside tmux, of 160 columns and 60
//! rows, of `Alice in Wonderland` of the library `Books` of the sandbox: the
//! panel held 27 rows, the picture took 14 of them, the facts took 8, the
//! description of one line took 5, and **four rows of the screen held no
//! character at all**.
//!
//! **The rule turns**: the facts and the description keep the rows that they
//! need, and every row that stays goes to the picture. A description of many
//! lines keeps [`THE_SMALLEST_PICTURE`] rows for the picture and it scrolls
//! with the keys `J` and `K`.

use ratatui::layout::Rect;

/// The rows of the facts of the media, inside the panel 5.
///
/// **The row of the facts of a list takes three rows already**
/// (`the_areas_of_a_list` of `crate::ui::tui`), and the words of it wrap in a
/// panel that is not wide. This module keeps that number, therefore the words
/// of the media say the same thing in the panel and under the list.
pub const THE_ROWS_OF_THE_FACTS: u16 = 3;

/// The rows that the description needs before it says anything.
///
/// One row of a description says almost nothing, and the bar of the scroll of
/// `crate::logic::the_scroll_of_a_panel` needs a row of its own.
pub const THE_ROWS_OF_A_DESCRIPTION: u16 = 2;

/// The rows of the panel that the words never take.
///
/// **The picture takes every row that the facts and the description leave**
/// (T-330.3), therefore a description of many lines would take the picture
/// down to nothing. The picture keeps this number of rows, and the description
/// then scrolls with the keys `J` and `K`, which is the work of
/// `crate::logic::the_scroll_of_a_panel`.
pub const THE_SMALLEST_PICTURE: u16 = crate::ui::cover::MIN_HEIGHT_FOR_COVER;

/// The smallest panel that holds a picture and the words of the media
/// together.
///
/// A panel under this height holds the picture alone, and the words of the
/// media then stay under the list, where they stood before this stage.
pub const THE_SMALLEST_PANEL_OF_THE_WORDS: u16 =
    crate::ui::cover::MIN_HEIGHT_FOR_COVER + THE_ROWS_OF_THE_FACTS + THE_ROWS_OF_A_DESCRIPTION;

/// The three parts of the panel 5: the picture, the facts, and the
/// description.
///
/// An area of no cell at all says that the part does not stand on the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThePartsOfThePanel {
    /// The area of the picture of the cover, and `None` for a panel that shows
    /// no picture.
    pub cover: Option<Rect>,
    /// The area of the facts of the media: the author, the year, the length,
    /// and the place of the user.
    pub facts: Rect,
    /// The area of the description of the media.
    pub description: Rect,
}

impl ThePartsOfThePanel {
    /// Says if the panel holds the words of the media.
    ///
    /// **The words stand in one place of one frame**: a panel that says them
    /// takes them away from the area under the list, and a panel that is too
    /// small for them leaves them where they stood.
    pub fn the_words_stand_here(&self) -> bool {
        self.facts.height > 0 || self.description.height > 0
    }
}

/// Gives an area of no cell at all, at the corner of this area.
fn nothing(of: Rect) -> Rect {
    Rect {
        x: of.x,
        y: of.y,
        width: 0,
        height: 0,
    }
}

/// Divides the inside of the panel 5 into the picture, the facts, and the
/// description. See T-319.
///
/// `inside` is the area inside the border of the panel.
///
/// `a_picture_comes` is `false` for a media that the server holds with no
/// cover at all (`crate::ui::cover::no_picture_comes`). The picture then takes
/// no row, and the words take the whole panel: **a picture that never comes
/// must take no row of the screen of the user.**
///
/// `of_the_description` is the number of the rows that the text of the
/// description needs at the width of the panel
/// (`crate::logic::the_scroll_of_a_panel::the_number_of_the_lines`).
///
/// **The facts and the description keep the rows that they need, and every row
/// that stays goes to the picture** (T-330.3). A share of the height gave the
/// picture few rows of a tall panel and it left the rest of them empty: the
/// screen of 60 rows of the measurement held a picture of 14 rows, 8 rows of
/// the facts, one row of the description, and **four rows of nothing at all**.
///
/// The function is pure, therefore a test needs no terminal and no server.
pub fn the_parts_of_the_panel(
    inside: Rect,
    a_picture_comes: bool,
    of_the_facts: u16,
    of_the_description: u16,
) -> ThePartsOfThePanel {
    if inside.width == 0 || inside.height == 0 {
        return ThePartsOfThePanel {
            cover: None,
            facts: nothing(inside),
            description: nothing(inside),
        };
    }

    // A media with no cover gives every row of the panel to the words.
    if !a_picture_comes {
        return the_words_of(inside, None, of_the_facts);
    }

    // A panel that is not tall holds the picture alone. The words then stay
    // under the list, where they stood before this stage.
    if inside.height < THE_SMALLEST_PANEL_OF_THE_WORDS {
        return ThePartsOfThePanel {
            cover: Some(inside),
            facts: nothing(inside),
            description: nothing(inside),
        };
    }

    // **The words keep the rows that they need, and the picture takes every
    // row that stays** (T-330.3). The picture keeps `THE_SMALLEST_PICTURE`
    // rows, therefore a description of many lines scrolls and it does not take
    // the picture away.
    let the_most_of_the_words = inside.height.saturating_sub(THE_SMALLEST_PICTURE);
    let of_the_words = of_the_facts
        .saturating_add(of_the_description)
        .min(the_most_of_the_words);
    let of_the_cover = inside.height - of_the_words;

    let cover = Rect {
        height: of_the_cover,
        ..inside
    };

    the_words_of(
        Rect {
            y: inside.y + of_the_cover,
            height: inside.height - of_the_cover,
            ..inside
        },
        Some(cover),
        of_the_facts,
    )
}

/// Divides the area of the words into the facts and the description.
fn the_words_of(area: Rect, cover: Option<Rect>, of_the_facts: u16) -> ThePartsOfThePanel {
    let of_the_facts = of_the_facts.min(area.height);

    ThePartsOfThePanel {
        cover,
        facts: Rect {
            height: of_the_facts,
            ..area
        },
        description: Rect {
            y: area.y + of_the_facts,
            height: area.height - of_the_facts,
            ..area
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The panel of a screen of 160 columns holds the picture, the facts, and
    /// the description, and the three of them touch with no row between them.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn the_panel_holds_the_picture_and_the_words() {
        // The measurement of 2026-08-16: the column of the covers of a screen
        // of 160 by 45 stands at the column 110 and it holds 41 rows. The
        // border of the panel takes one row at each end.
        let inside = Rect::new(111, 3, 48, 39);
        let parts = the_parts_of_the_panel(
            inside,
            true,
            THE_ROWS_OF_THE_FACTS,
            THE_ROWS_OF_A_DESCRIPTION,
        );

        let cover = parts
            .cover
            .expect("a picture comes, therefore it has an area");

        // **The picture takes every row that the words leave** (T-330.3).
        assert_eq!(cover, Rect::new(111, 3, 48, 34));

        // The facts stand under the picture, and the description under them.
        assert_eq!(parts.facts, Rect::new(111, 37, 48, THE_ROWS_OF_THE_FACTS));
        assert_eq!(
            parts.description,
            Rect::new(111, 40, 48, THE_ROWS_OF_A_DESCRIPTION)
        );
        assert!(parts.the_words_stand_here());

        // No row of the panel goes away, and no row holds two parts.
        assert_eq!(
            cover.height + parts.facts.height + parts.description.height,
            inside.height
        );
    }

    /// A media that the server holds with no cover gives every row of the
    /// panel to the words.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn a_media_with_no_cover_gives_its_rows_to_the_words() {
        let inside = Rect::new(111, 3, 48, 39);
        let parts = the_parts_of_the_panel(
            inside,
            false,
            THE_ROWS_OF_THE_FACTS,
            THE_ROWS_OF_A_DESCRIPTION,
        );

        assert_eq!(parts.cover, None);
        assert_eq!(parts.facts, Rect::new(111, 3, 48, THE_ROWS_OF_THE_FACTS));
        assert_eq!(parts.description, Rect::new(111, 6, 48, 36));
        assert!(parts.the_words_stand_here());
    }

    /// A panel that is not tall holds the picture alone, and the words then
    /// stay under the list.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn a_panel_that_is_not_tall_holds_the_picture_alone() {
        // One row under the smallest panel of the words.
        let inside = Rect::new(111, 3, 48, THE_SMALLEST_PANEL_OF_THE_WORDS - 1);
        let parts = the_parts_of_the_panel(
            inside,
            true,
            THE_ROWS_OF_THE_FACTS,
            THE_ROWS_OF_A_DESCRIPTION,
        );

        assert_eq!(parts.cover, Some(inside));
        assert_eq!(parts.facts.height, 0);
        assert_eq!(parts.description.height, 0);
        assert!(!parts.the_words_stand_here());

        // The smallest panel of the words holds the three parts, and the
        // picture keeps the rows that a cover needs.
        let inside = Rect::new(111, 3, 48, THE_SMALLEST_PANEL_OF_THE_WORDS);
        let parts = the_parts_of_the_panel(
            inside,
            true,
            THE_ROWS_OF_THE_FACTS,
            THE_ROWS_OF_A_DESCRIPTION,
        );

        assert_eq!(
            parts.cover.map(|of| of.height),
            Some(crate::ui::cover::MIN_HEIGHT_FOR_COVER)
        );
        assert_eq!(parts.facts.height, THE_ROWS_OF_THE_FACTS);
        assert_eq!(parts.description.height, THE_ROWS_OF_A_DESCRIPTION);

        // **A media with no cover of a panel that is not tall still says the
        // words**: the picture takes no row at all, therefore the rows of the
        // words come of no picture.
        let inside = Rect::new(111, 3, 48, 4);
        let parts = the_parts_of_the_panel(
            inside,
            false,
            THE_ROWS_OF_THE_FACTS,
            THE_ROWS_OF_A_DESCRIPTION,
        );

        assert_eq!(parts.cover, None);
        assert_eq!(parts.facts.height, THE_ROWS_OF_THE_FACTS);
        assert_eq!(parts.description.height, 1);
    }

    /// **The smallest panel of a media with no cover holds the whole of the
    /// facts** (T-349).
    ///
    /// `crate::ui::cover::the_smallest_panel_of_the_cover` gives the rows of
    /// that panel, and the border of it takes one row at each end. The rows that
    /// stay inside must hold the three facts of the media, because a panel that
    /// says fewer facts than the row under the list says takes the columns of
    /// the list for nothing.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn the_smallest_panel_of_no_picture_holds_the_whole_of_the_facts() {
        let of_the_border = 2;

        // The number is the number of the measurement, and not the value of the
        // function: a test that takes its own bounds of the function that it
        // measures cannot fail while that function holds a fault.
        assert_eq!(
            crate::ui::cover::the_smallest_panel_of_the_cover(false),
            THE_ROWS_OF_THE_FACTS + of_the_border
        );

        let inside = Rect::new(111, 3, 48, THE_ROWS_OF_THE_FACTS);

        let parts = the_parts_of_the_panel(
            inside,
            false,
            THE_ROWS_OF_THE_FACTS,
            THE_ROWS_OF_A_DESCRIPTION,
        );

        assert_eq!(parts.cover, None);
        assert_eq!(
            parts.facts.height, THE_ROWS_OF_THE_FACTS,
            "the smallest panel of a media with no cover must say every fact"
        );
        assert!(parts.the_words_stand_here());
    }

    /// A panel of no cell at all holds no part.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn a_panel_of_no_cell_holds_no_part() {
        for inside in [
            Rect::default(),
            Rect::new(10, 4, 0, 20),
            Rect::new(10, 4, 30, 0),
        ] {
            for a_picture_comes in [true, false] {
                let parts = the_parts_of_the_panel(
                    inside,
                    a_picture_comes,
                    THE_ROWS_OF_THE_FACTS,
                    THE_ROWS_OF_A_DESCRIPTION,
                );

                assert_eq!(parts.cover, None);
                assert_eq!(parts.facts.height, 0);
                assert_eq!(parts.description.height, 0);
                assert!(!parts.the_words_stand_here());
            }
        }
    }
}
