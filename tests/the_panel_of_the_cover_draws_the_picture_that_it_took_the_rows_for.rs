//! The panel 5 of the cover stands only at a screen that holds the picture
//! that it took the rows for. See T-351.
//!
//! **The limit of the height stood on two different numbers of rows.**
//! `split_for_covers` compares the height of the **whole** panel with
//! `the_smallest_panel_of_the_cover`, and that function gave
//! `MIN_HEIGHT_FOR_COVER` for a media that holds a picture. `plan_covers`
//! compares the height **inside** the border with that same
//! `MIN_HEIGHT_FOR_COVER`. The border takes two rows, therefore a panel of 8
//! or of 9 rows stood with 6 or 7 rows inside it, and `plan_covers` then gave
//! no rectangle at all.
//!
//! **The arm of a media with no cover held the border already**
//! (`THE_ROWS_OF_THE_BORDER + THE_ROWS_OF_THE_FACTS`, T-349), and the arm of a
//! picture did not.
//!
//! **The measurement of the real program v0.8.181 inside tmux**, of 160
//! columns, of `A Long Test Book` of the library `Books` of the sandbox, whose
//! cover the server holds. The screen of 13 rows and the screen of 14 rows:
//!
//! ```text
//! ┌5 Cover ────────────┐
//! │                    │
//! │                    │
//! │                    │
//! │                    │      ← the panel holds no character at all
//! │                    │
//! │                    │
//! └────────────────────┘
//! ```
//!
//! The same run at 15 rows drew a picture of 16 columns inside a panel of 22.
//! The panel of 13 and of 14 rows therefore took 22 columns and 8 rows of the
//! list, and it said nothing: no picture, no fact, and no word of the media.
//!
//! **The corrected program of the same harness**: the panel goes away at 13
//! and at 14 rows, the list takes those 22 columns, and the two rows under the
//! list say `Author: Long Author - Year: N/A - Duration: 30m` and
//! `Progress: 50%, 15m left, Not finished`. At 15 rows the panel stands and it
//! draws its picture, as it did before.
//!
//! **The words of the panel hold no such rule**: a panel of 15 rows draws the
//! picture alone, because `THE_SMALLEST_PANEL_OF_THE_WORDS` needs 13 rows
//! inside the border, and the words then stay under the list where they stood
//! (T-349). This test holds the **picture** alone.

use ratatui::layout::Rect;
use ratatui_image::FontSize;
use toutui::ui::cover::{plan_covers, split_for_covers, MIN_HEIGHT_FOR_COVER};
use toutui::ui::the_panel_of_the_cover::the_parts_of_the_panel;

/// The font of the measurement: a cell of 10 by 20 pixels.
const FONT: FontSize = FontSize {
    width: 10,
    height: 20,
};

/// The two rows of the border of the panel.
const THE_ROWS_OF_THE_BORDER: u16 = 2;

/// A panel of the cover that stands draws its picture.
///
/// The test walks the whole chain of the render of `crate::ui::tui`: the split
/// of the column, the border of the block, the parts of the panel, and the
/// plan of the covers. **A panel that holds a picture and that draws none is
/// columns of the list for nothing.**
///
/// **The parts of this test stay in one function.**
#[test]
fn a_panel_that_stands_draws_the_picture_that_it_took_the_rows_for() {
    // The facts of a book of the design take nine lines, and the description
    // of the measurement takes one. See T-325.
    let of_the_facts = 9;
    let of_the_description = 1;

    let mut the_panels_that_stood = 0;

    for height in 0..40u16 {
        let (_text, panel) = split_for_covers(Rect::new(0, 0, 126, height), 160, FONT, true);

        let Some(panel) = panel else {
            continue;
        };

        the_panels_that_stood += 1;

        // `block.inner(panel)` of the render takes the border away.
        let inside = Rect {
            x: panel.x + 1,
            y: panel.y + 1,
            width: panel.width.saturating_sub(THE_ROWS_OF_THE_BORDER),
            height: panel.height.saturating_sub(THE_ROWS_OF_THE_BORDER),
        };

        let parts = the_parts_of_the_panel(inside, true, of_the_facts, of_the_description);

        let cover = parts
            .cover
            .unwrap_or_else(|| panic!("a panel of {height} rows that holds a picture"));

        let plan = plan_covers(cover, FONT, false, 1, None);

        assert!(
            !plan.shelf.is_empty(),
            "the panel of a screen of {height} rows stands with {} rows inside \
             its border, and it draws no picture at all",
            inside.height
        );
    }

    assert!(
        the_panels_that_stood > 0,
        "the sweep must reach a screen that holds the panel"
    );

    // The gate of the measurement: a screen of 13 rows gave the panel 8 rows,
    // and the panel of 8 rows drew nothing. The smallest panel of a picture is
    // the border and the picture.
    assert_eq!(
        split_for_covers(
            Rect::new(0, 0, 126, MIN_HEIGHT_FOR_COVER + THE_ROWS_OF_THE_BORDER - 1),
            160,
            FONT,
            true,
        )
        .1,
        None,
        "a panel that cannot hold the whole picture goes away"
    );

    assert!(
        split_for_covers(
            Rect::new(0, 0, 126, MIN_HEIGHT_FOR_COVER + THE_ROWS_OF_THE_BORDER),
            160,
            FONT,
            true,
        )
        .1
        .is_some(),
        "a panel of the border and of the whole picture stands"
    );
}
