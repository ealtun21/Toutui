//! The gate of the panel 6 of the gallery of the covers. See T-327.
//!
//! **The design of `docs/mockups/mockup-1.txt` holds seven panels, and the
//! panel 6 of the gallery had no line of code at all.** The sweep of T-323 of
//! the regions of the map of the mouse said it: "the whole of the panel 6 of
//! the gallery (no area of it stands in the code at all)".
//!
//! The measurement of the real program v0.8.156 inside tmux, of the Library
//! view of the library `Large` of the sandbox at 160 columns and 45 rows. The
//! column at the right held the panel 5 alone, and the items of that library
//! carry `coverPath: null`, therefore the panel said three facts over three
//! rows and it left **35 rows with no character at all**:
//!
//! ```text
//! ┌5 Cover ────────────────────────────────────────┐
//! │Time      0m                                    │
//! │Files     1 file, 0.0 MB                        │
//! │No description available                        │
//! │                                                │
//! │                                     … 35 rows …│
//! └────────────────────────────────────────────────┘
//! ```
//!
//! The corrected program of the same harness, of the library `Books`, with the
//! cursor on the first row of the list:
//!
//! ```text
//! ┌6 Gallery ──────────────────────────────────────┐
//! │  ╔════════╗ ┌────────┐ ┌────────┐ ┌────────┐   │
//! │  ║        ║ │        │ │        │ │        │   │
//! │  ║        ║ │        │ │        │ │        │   │
//! │  ║        ║ │        │ │        │ │        │   │
//! │  ║        ║ │        │ │        │ │        │   │
//! │  ║  done  ║ │    -   │ │  done  │ │    -   │   │
//! │  ╚════════╝ └────────┘ └────────┘ └────────┘   │
//! │  A Book Of… A Book Of… A Book Of… A Very La…   │
//! │  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐   │
//! │  │        │ │        │ │        │ │        │   │
//! │  │        │ │        │ │        │ │        │   │
//! │  │        │ │        │ │        │ │        │   │
//! │  │        │ │        │ │        │ │        │   │
//! │  │   90%  │ │    -   │ │  done  │ │   50%  │   │
//! │  └────────┘ └────────┘ └────────┘ └────────┘   │
//! │  A Big Boo… A Book Th… A Huge Bo… A Long Te…   │
//! └────────────────────────────────────────────────┘
//! ```
//!
//! The picture of a cover stands inside the box, and `tmux capture-pane` with
//! no `-e` gives no character of it: the cover of `A Long Test Book` of the
//! sandbox is 400 pixels of one red, therefore the halfblocks of that picture
//! are cells of a background of red and of no letter. The same capture with
//! `-e` gave `48;2;254;0;0` on four rows of that panel.

use ratatui::layout::Rect;
use ratatui_image::FontSize;
use toutui::ui::frame::ThePanel;
use toutui::ui::keys::the_footer_of_a_panel;
use toutui::ui::the_panel_of_the_cover::THE_SMALLEST_PANEL_OF_THE_WORDS;
use toutui::ui::the_panel_of_the_gallery::{
    plan_the_gallery, the_rows_of_a_row_of_the_grid, the_smallest_gallery, the_two_panels,
    THE_WIDTHS_OF_A_CELL, THE_WIDTH_OF_THE_START,
};

/// The font of the terminal of the measurement of this round.
///
/// The log of the program said it: "the terminal uses Halfblocks with a font of
/// 10 by 20 pixels".
const FONT: FontSize = FontSize {
    width: 10,
    height: 20,
};

/// The column of the covers of the measurement: the panel 5 over the panel 6,
/// and no row of the screen that no part uses. See T-327.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_column_holds_the_two_panels_and_no_row_that_no_part_uses() {
    // The measurement of 2026-08-16: the column of the covers of a screen of
    // 160 by 45 holds 41 rows.
    let column = Rect::new(111, 2, 50, 41);
    let of_a_cell = THE_WIDTHS_OF_A_CELL[THE_WIDTH_OF_THE_START];

    let (cover, gallery) = the_two_panels(column, of_a_cell, FONT);
    let gallery = gallery.expect("a column of 41 rows holds the panel 5 and the panel 6");

    // The panel 5 stands above the panel 6, and the two of them use every row.
    assert_eq!(cover.y, column.y);
    assert_eq!(gallery.y, cover.y + cover.height);
    assert_eq!(cover.height + gallery.height, column.height);

    // **The panel 6 holds whole rows of the grid alone** (T-327): the first
    // form of this panel took 40 percent of the column, and it then held five
    // rows of the screen that no cell used.
    let of_a_row = the_rows_of_a_row_of_the_grid(of_a_cell, FONT);
    assert_eq!(
        (gallery.height - 2) % of_a_row,
        0,
        "the panel 6 of {} rows holds a part of a row of the grid of {of_a_row} rows",
        gallery.height
    );

    // The panel 5 keeps the rows that the facts and the description need.
    assert!(
        cover.height >= THE_SMALLEST_PANEL_OF_THE_WORDS + 2,
        "the panel 5 holds {} rows and the words of the media need {}",
        cover.height,
        THE_SMALLEST_PANEL_OF_THE_WORDS + 2
    );

    // **A column that holds no room for the two panels gives every row to the
    // panel 5**: the words of the media of the cursor say more than the covers
    // of the media beside it.
    let short = Rect::new(111, 2, 50, THE_SMALLEST_PANEL_OF_THE_WORDS + 2);
    assert_eq!(the_two_panels(short, of_a_cell, FONT), (short, None));

    // Every width of a cell of the design gives a panel of whole rows of the
    // grid, and a cell that is larger needs more rows.
    let mut before = 0;

    for of_a_cell in THE_WIDTHS_OF_A_CELL {
        let the_smallest = the_smallest_gallery(of_a_cell, FONT);
        assert!(
            the_smallest > before,
            "the cell of {of_a_cell} columns needs no more rows than the cell before it"
        );
        before = the_smallest;

        if let (_, Some(gallery)) = the_two_panels(column, of_a_cell, FONT) {
            let of_a_row = the_rows_of_a_row_of_the_grid(of_a_cell, FONT);
            assert_eq!((gallery.height - 2) % of_a_row, 0);
        }
    }
}

/// The grid of the gallery holds the media around the cursor of the list, and
/// no cell of it names a media that the list does not hold. See T-327.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_grid_holds_the_media_around_the_cursor_of_the_list() {
    let column = Rect::new(111, 2, 50, 41);
    let of_a_cell = THE_WIDTHS_OF_A_CELL[THE_WIDTH_OF_THE_START];
    let (_, gallery) = the_two_panels(column, of_a_cell, FONT);
    let inside = {
        let gallery = gallery.expect("a column of 41 rows holds the panel 6");
        Rect::new(
            gallery.x + 1,
            gallery.y + 1,
            gallery.width - 2,
            gallery.height - 2,
        )
    };

    // The library `Large` of the sandbox holds 2056 items, and the list of the
    // view holds the first 500 of them.
    let plan = plan_the_gallery(inside, of_a_cell, FONT, 500, 0);

    assert!(plan.the_columns >= 1 && plan.the_rows >= 1);
    assert_eq!(plan.the_first, 0);
    assert_eq!(plan.cells.len(), plan.the_columns * plan.the_rows);

    // **The grid stands around the cursor**: a cursor of the middle of the list
    // gives a grid that holds the media before it and the media after it.
    let of_the_middle = plan_the_gallery(inside, of_a_cell, FONT, 500, 250);
    assert!(
        of_the_middle.cells.iter().any(|cell| cell.the_media == 250),
        "the grid of the cursor 250 holds no cell of that media"
    );
    assert!(
        of_the_middle.the_first < 250,
        "the grid of the cursor 250 starts at {} and it shows no media before it",
        of_the_middle.the_first
    );

    // The cursor of the end of the list gives a grid of the last media, and no
    // cell of that grid names a media that the list does not hold.
    let of_the_end = plan_the_gallery(inside, of_a_cell, FONT, 500, 499);
    assert!(of_the_end.cells.iter().any(|cell| cell.the_media == 499));
    assert!(of_the_end.cells.iter().all(|cell| cell.the_media < 500));

    // Every cell stands inside the panel, and no two cells hold one cell of the
    // screen.
    for (at, cell) in plan.cells.iter().enumerate() {
        assert_eq!(
            inside.union(cell.the_box),
            inside,
            "{cell:?} left the panel"
        );
        assert_eq!(
            inside.union(cell.the_title),
            inside,
            "{cell:?} left the panel"
        );

        for other in plan.cells.iter().skip(at + 1) {
            assert_eq!(
                cell.the_box.intersection(other.the_box).area(),
                0,
                "the cells {cell:?} and {other:?} hold one cell of the screen"
            );
        }
    }

    // **A click of a cell names that cell**, and a click of the column between
    // two cells names none.
    let first = plan.cells[0];
    assert_eq!(
        plan.the_cell_of_a_point(first.the_picture.x, first.the_picture.y)
            .map(|cell| cell.the_media),
        Some(0)
    );
    assert_eq!(
        plan.the_cell_of_a_point(first.the_title.x, first.the_title.y)
            .map(|cell| cell.the_media),
        Some(0)
    );
    assert_eq!(
        plan.the_cell_of_a_point(first.the_box.x + of_a_cell, first.the_box.y),
        None
    );
}

/// The digit `6` names the panel 6, and the focus moves between the panel 5 and
/// the panel 6 of the column at the right. See T-327.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_digit_of_the_gallery_names_the_panel_and_the_focus_moves_to_it() {
    assert_eq!(ThePanel::of_the_digit('6'), Some(ThePanel::TheGallery));
    assert_eq!(ThePanel::TheGallery.the_number(), 6);

    // **The panel 6 stands under the panel 5, in the column at the right**, in
    // the same way as the panels 2 and 3 under the panel 1.
    assert_eq!(ThePanel::TheCover.below(), ThePanel::TheGallery);
    assert_eq!(ThePanel::TheGallery.above(), ThePanel::TheCover);

    // The movement stops at the ends of the column and of the row.
    assert_eq!(ThePanel::TheGallery.below(), ThePanel::TheGallery);
    assert_eq!(ThePanel::TheGallery.at_the_right(), ThePanel::TheGallery);
    assert_eq!(ThePanel::TheGallery.at_the_left(), ThePanel::TheList);
    assert!(!ThePanel::TheGallery.is_of_the_stack());

    // **The band of the player holds no digit** (T-322): every key of the
    // player works in every view of this program already.
    for digit in ['7', '0', '8', '9'] {
        assert_eq!(ThePanel::of_the_digit(digit), None);
    }
}

/// The footer of the panel 6 names the keys of that panel, and it promises no
/// key that the panel does not hold. See T-327 and T-143.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_footer_of_the_gallery_names_the_keys_of_the_gallery() {
    let footer = the_footer_of_a_panel("j/k: move", true, true, ThePanel::TheGallery);

    // **A key that the footer does not name is a key that the user does not
    // have** (the rule of T-143 in reverse): the keys of the size of a cell are
    // the buttons `[+ bigger]` and `[- smaller]` of the design, and no other
    // view of this program holds them.
    for key in ["j/k", "+/-", "l:", "h:", "Q: quit"] {
        assert!(
            footer.contains(key),
            "the footer {footer:?} names no key {key}"
        );
    }

    // **A footer must not promise a key that the view does not hold** (T-143):
    // the panel 6 holds no key of a search, of a refresh, and of the next
    // library.
    for key in ["/: search", "R: refresh", "S-Tab"] {
        assert!(
            !footer.contains(key),
            "the footer {footer:?} promises the key {key}, and the panel 6 does not hold it"
        );
    }

    // The footer of the panel 6 is not the footer of the panel 5, and it is not
    // the footer of the view.
    assert_ne!(
        footer,
        the_footer_of_a_panel("j/k: move", true, true, ThePanel::TheCover)
    );

    // **A screen that draws no frame gives the footer of the view** (T-320):
    // the panels stand at 120 columns and more.
    assert_eq!(
        the_footer_of_a_panel("j/k: move", false, false, ThePanel::TheGallery),
        "j/k: move"
    );
}
