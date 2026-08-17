//! The bands of covers of the panel 4 of the Home view. See T-331 and T-336.
//!
//! The maintainer asked for this view on 2026-08-16, and
//! `docs/superpowers/specs/2026-08-17-the-home-view-of-the-bands-of-covers-design.md`
//! holds the design of it. `crate::logic::the_bands_of_the_home` gives the bands
//! of the flat list of the Home view and the moves over them (T-335); **this
//! module holds the arithmetic of the screen of those bands**, and it is the
//! round 2 of the road of that design.
//!
//! ## The screen before this module
//!
//! **The measurement of the real program v0.8.167 inside tmux of 2026-08-17**,
//! of the Home view of the library `Large` of the sandbox at 160 columns and 45
//! rows. The panel 4 says the four columns of the table of T-321 for every shelf
//! of the view, and the covers of those media stand in the panel 6 alone:
//!
//! ```text
//! ╔4 Home [20 items] ═══════════════════════════════════════════════╗
//! ║    Title                            Author           Time  Done ║
//! ║  ▌ Recently Added                                               ║
//! ║➤   Large Book 0001                                  <1m     -   ║
//! ║    Large Book 0002                                  <1m     -   ║
//! ```
//!
//! ## The rule of this module
//!
//! **One band for one shelf, and one cell for one line of that shelf.** The cell
//! holds the picture of the cover and a border of one line around it, which is
//! the cell of the panel 6 of the gallery (T-327): the two of them are one
//! picture in one border, therefore the width of a cell comes of
//! `THE_WIDTHS_OF_A_CELL` and the rows of its picture come of the `FontSize` of
//! the picker of the terminal of the user, and never of a number of the mockup.
//!
//! ```text
//! Continue Listening ──────────────────────────── 6 of 10  ›
//! ┏━━━━━━━━┓ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐
//! ┃        ┃ │        │ │        │ │        │ │        │
//! ┗━━━━━━━━┛ └────────┘ └────────┘ └────────┘ └────────┘
//! ```
//!
//! **A panel that has no room for one whole band gives no band at all**, and the
//! Home view then draws the table of today. That is the rule of the panel 6 of
//! T-327 for a part of a row of its grid, and the rule of T-321 for a panel that
//! is too narrow for its table: **no key of the user turns between the two
//! shapes** (the decision 5 of the maintainer).
//!
//! **The offset of a band comes of the cursor, and no band holds a state of its
//! own.** The design names an offset of each band; the wheel of the mouse of the
//! round 3 is the one road that moves a band with no cursor in it, therefore
//! that state belongs to that round. A render that reads the cursor alone cannot
//! go out of agreement with the flat list of the lines.
//!
//! Every function of this module is pure, therefore a test of it needs no
//! terminal, no server, and no `App` at all.

use crate::logic::the_bands_of_the_home::ABand;
use ratatui::layout::Rect;
use ratatui_image::FontSize;

/// The rows that one band takes: the title, the box of a cell, and one space.
///
/// **The space under a band keeps the two bands apart**: a title that stands on
/// the row under a border reads as a part of that border.
pub const THE_ROWS_OF_A_TITLE: u16 = 1;
/// The rows of the space under one band. See [`THE_ROWS_OF_A_TITLE`].
pub const THE_ROWS_OF_THE_SPACE: u16 = 1;

/// One cell of a band: a line of the flat list of the Home view, and its areas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ACellOfABand {
    /// The line of the flat list of `crate::logic::home_view` that this cell
    /// holds. **The cursor of the view is that line** (the decision 1 of the
    /// design), therefore a click of the cell moves the cursor to it.
    pub the_line: usize,
    /// The box of the cell, with its border.
    pub the_box: Rect,
    /// The area of the picture of the cover, inside the border.
    pub the_picture: Rect,
}

/// One band of the screen: the title of a shelf, and the cells that it draws.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ABandOfTheScreen {
    /// The place of this band in the bands of `the_bands`.
    pub the_band: usize,
    /// The name of the shelf, of `home_view::the_name_of_the_shelf`.
    pub the_name: String,
    /// The count of the title: the cells that the band draws, of the cells that
    /// it holds. See `the_bands_of_the_home::the_count_of_a_band`.
    pub the_count: String,
    /// The row of the title, over the cells.
    pub the_title: Rect,
    /// The band holds a cell before the first cell that it draws.
    pub at_the_left: bool,
    /// The band holds a cell after the last cell that it draws.
    pub at_the_right: bool,
    /// The cells that this band draws, from the one at the left.
    pub cells: Vec<ACellOfABand>,
}

/// The bands that the panel 4 draws.
///
/// **A plan of no band says that the panel has no room for one whole band**, and
/// the Home view then draws the table of today.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TheBandsOfThePanel {
    /// The bands of the screen, from the one at the top.
    pub bands: Vec<ABandOfTheScreen>,
    /// The place of the first band of the screen, in the bands of the view.
    pub the_first_band: usize,
    /// The cells of one band that the panel has the room for.
    pub the_cells_of_a_band: usize,
}

impl TheBandsOfThePanel {
    /// The panel draws bands, and not the table of today.
    pub fn stands(&self) -> bool {
        !self.bands.is_empty()
    }

    /// The cell of a point of the screen, and `None` for a point that stands
    /// beside every cell.
    ///
    /// **The box of a cell is the whole of that cell**, which is the rule of the
    /// panel 6 (T-330.4): a click of the border of a cover is a click of that
    /// cover, and no point of the panel belongs to two cells.
    pub fn the_cell_of_a_point(&self, column: u16, row: u16) -> Option<&ACellOfABand> {
        let point = ratatui::layout::Position::new(column, row);

        self.bands
            .iter()
            .flat_map(|band| band.cells.iter())
            .find(|cell| cell.the_box.contains(point))
    }
}

/// The rows that one band of this width takes on the screen.
///
/// The title, the box of a cell with its border and its picture, and one space
/// under it.
pub fn the_rows_of_a_band(of_a_cell: u16, font: FontSize) -> u16 {
    THE_ROWS_OF_A_TITLE
        + crate::ui::the_panel_of_the_gallery::the_rows_of_a_box(of_a_cell, font)
        + THE_ROWS_OF_THE_SPACE
}

/// Gives the bands that the panel 4 of the Home view draws. See T-336.
///
/// `inside` is the area inside the border of the panel 4. `bands` are the bands
/// of the flat list, and `the_line` is the line of the cursor of the view.
///
/// **The band of the cursor stands on the screen**: the bands go up and down as
/// the rows of the panel 6 do (T-327), and the band of the cursor stands in the
/// middle of the rows that the panel holds.
///
/// **The panel draws whole bands alone**: a part of a band holds rows of the
/// screen that no cell uses, which is the rule of T-327.
///
/// The function is pure, therefore a test needs no terminal and no server.
pub fn plan_the_bands(
    inside: Rect,
    of_a_cell: u16,
    font: FontSize,
    bands: &[ABand],
    the_line: usize,
) -> TheBandsOfThePanel {
    let of_a_band = the_rows_of_a_band(of_a_cell, font);

    if bands.is_empty() || of_a_cell < 3 || inside.width < of_a_cell || inside.height < of_a_band {
        return TheBandsOfThePanel::default();
    }

    // One column stays between two cells of a band, which is the grid of the
    // panel 6 of the gallery.
    let the_columns = usize::from((inside.width + 1) / (of_a_cell + 1));
    let the_rows = usize::from(inside.height / of_a_band);

    if the_columns == 0 || the_rows == 0 {
        return TheBandsOfThePanel::default();
    }

    // **A cursor that stands on the line of a shelf gives the first band**,
    // which is the rule of `the_bands_of_the_home::the_place_of_the_line`: the
    // cursor of the flat list can stand on such a line after a refresh.
    let (of_the_cursor, the_cell_of_the_cursor) =
        crate::logic::the_bands_of_the_home::the_place_of_the_line(bands, the_line)
            .unwrap_or((0, 0));

    let the_highest = bands.len().saturating_sub(the_rows);
    let the_first_band = of_the_cursor.saturating_sub(the_rows / 2).min(the_highest);

    let of_the_picture =
        crate::ui::the_panel_of_the_gallery::the_rows_of_a_box(of_a_cell, font) - 2;
    let mut of_the_screen = Vec::new();

    for row in 0..the_rows {
        let number = the_first_band + row;

        let Some(band) = bands.get(number) else {
            break;
        };

        // **The cells of the band of the cursor end at the cursor**, and every
        // other band draws its first cells. See the rule of the offset above.
        let the_first_cell = if number == of_the_cursor {
            the_cell_of_the_cursor.saturating_sub(the_columns.saturating_sub(1))
        } else {
            0
        };

        let y = inside.y + row as u16 * of_a_band;
        let mut cells = Vec::new();

        for column in 0..the_columns {
            let Some(the_line) = band.the_cells.get(the_first_cell + column) else {
                break;
            };

            let the_box = Rect {
                x: inside.x + column as u16 * (of_a_cell + 1),
                y: y + THE_ROWS_OF_A_TITLE,
                width: of_a_cell,
                height: of_the_picture + 2,
            };

            cells.push(ACellOfABand {
                the_line: *the_line,
                the_box,
                the_picture: Rect {
                    x: the_box.x + 1,
                    y: the_box.y + 1,
                    width: of_a_cell - 2,
                    height: of_the_picture,
                },
            });
        }

        of_the_screen.push(ABandOfTheScreen {
            the_band: number,
            the_name: band.the_title.clone(),
            the_count: crate::logic::the_bands_of_the_home::the_count_of_a_band(
                cells.len(),
                band.the_cells.len(),
            ),
            the_title: Rect {
                x: inside.x,
                y,
                width: inside.width,
                height: THE_ROWS_OF_A_TITLE,
            },
            at_the_left: the_first_cell > 0,
            at_the_right: the_first_cell + cells.len() < band.the_cells.len(),
            cells,
        });
    }

    TheBandsOfThePanel {
        bands: of_the_screen,
        the_first_band,
        the_cells_of_a_band: the_columns,
    }
}

/// The text of the row of the title of a band. See T-336.
///
/// ```text
/// Continue Listening ──────────────────────────── 6 of 10  ›
/// ```
///
/// **The arrows say that the band holds more cells than the row shows**: a band
/// of a count of `6 of 10` says the number already, and the arrow says the
/// direction of the key that reaches them.
///
/// **A row that has no room for the name and the count holds the name alone**,
/// cut with the three points of this program: a text that the row cuts says
/// nothing to the user (T-91), and the name of the shelf is the word that the
/// user reads first.
pub fn the_row_of_a_title(band: &ABandOfTheScreen, width: u16) -> String {
    if width == 0 {
        return String::new();
    }

    let at_the_left = if band.at_the_left { "‹ " } else { "" };
    let at_the_right = if band.at_the_right { " ›" } else { "" };
    let of_the_right = format!("{}{}{}", at_the_left, band.the_count, at_the_right);

    let width = usize::from(width);
    let of_the_name = crate::logic::message::the_columns_of(&band.the_name);
    let of_the_right_side = crate::logic::message::the_columns_of(&of_the_right);

    // The name, one space, one line at the least, one space, and the count.
    if of_the_name + of_the_right_side + 3 > width {
        return crate::logic::message::in_one_row(&band.the_name, width as u16);
    }

    let of_the_line = width - of_the_name - of_the_right_side - 2;

    format!(
        "{} {} {}",
        band.the_name,
        "─".repeat(of_the_line),
        of_the_right
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::home_view::HomeRow;
    use crate::logic::the_bands_of_the_home::the_bands;

    /// A font of a cell of ten columns by twenty rows, which is the shape of a
    /// cell of a terminal of today.
    fn the_font() -> FontSize {
        (10, 20).into()
    }

    /// A shelf of twelve media, a shelf of two, and a shelf of one.
    fn the_rows() -> Vec<HomeRow> {
        let mut rows = vec![HomeRow::Shelf {
            label: "Continue Listening".to_string(),
        }];

        for item in 0..12 {
            rows.push(HomeRow::Media { item });
        }

        rows.push(HomeRow::Shelf {
            label: "Discover".to_string(),
        });
        rows.push(HomeRow::Media { item: 12 });
        rows.push(HomeRow::Media { item: 13 });
        rows.push(HomeRow::Shelf {
            label: "Listen Again".to_string(),
        });
        rows.push(HomeRow::Media { item: 14 });

        rows
    }

    /// A panel of the width and the height of the measurement of the sandbox.
    fn the_panel(width: u16, height: u16) -> Rect {
        Rect {
            x: 2,
            y: 3,
            width,
            height,
        }
    }

    #[test]
    fn a_band_holds_the_cells_that_the_panel_has_the_room_for() {
        let bands = the_bands(&the_rows());
        let plan = plan_the_bands(the_panel(71, 39), 10, the_font(), &bands, 1);

        // (71 + 1) / (10 + 1) = 6 cells of a band.
        assert_eq!(plan.the_cells_of_a_band, 6);
        assert!(plan.stands());
        assert_eq!(plan.bands[0].cells.len(), 6);
        assert_eq!(plan.bands[0].the_count, "6 of 12");
        assert_eq!(plan.bands[1].cells.len(), 2);
        assert_eq!(plan.bands[2].cells.len(), 1);
    }

    #[test]
    fn a_panel_of_no_room_for_one_whole_band_gives_no_band() {
        let bands = the_bands(&the_rows());

        // The rows of a band of ten columns: 1 + (4 + 2) + 1 = 8.
        assert_eq!(the_rows_of_a_band(10, the_font()), 8);

        assert!(!plan_the_bands(the_panel(71, 7), 10, the_font(), &bands, 1).stands());
        assert!(!plan_the_bands(the_panel(9, 39), 10, the_font(), &bands, 1).stands());
        assert!(!plan_the_bands(the_panel(71, 39), 10, the_font(), &[], 1).stands());
    }

    #[test]
    fn the_cells_of_the_band_of_the_cursor_end_at_the_cursor() {
        let bands = the_bands(&the_rows());

        // The cursor at the cell 8 of the first band: the six cells of the row
        // end at it, therefore the first of them is the cell 3.
        let plan = plan_the_bands(the_panel(71, 39), 10, the_font(), &bands, 9);
        let of_the_cursor = &plan.bands[0];

        assert_eq!(of_the_cursor.cells[0].the_line, 4);
        assert_eq!(of_the_cursor.cells[5].the_line, 9);
        assert!(of_the_cursor.at_the_left);
        assert!(of_the_cursor.at_the_right);

        // Every other band draws its first cells.
        assert_eq!(plan.bands[1].cells[0].the_line, 14);
        assert!(!plan.bands[1].at_the_left);
        assert!(!plan.bands[1].at_the_right);
    }

    #[test]
    fn a_cursor_on_the_line_of_a_shelf_gives_the_first_band() {
        let bands = the_bands(&the_rows());
        let plan = plan_the_bands(the_panel(71, 39), 10, the_font(), &bands, 0);

        assert_eq!(plan.the_first_band, 0);
        assert_eq!(plan.bands[0].cells[0].the_line, 1);
        assert!(!plan.bands[0].at_the_left);
    }

    #[test]
    fn the_band_of_the_cursor_stands_on_the_screen() {
        let bands = the_bands(&the_rows());

        // A panel of one band alone, and the cursor in the third band.
        let plan = plan_the_bands(the_panel(71, 8), 10, the_font(), &bands, 17);

        assert_eq!(plan.bands.len(), 1);
        assert_eq!(plan.the_first_band, 2);
        assert_eq!(plan.bands[0].the_name, "Listen Again");
    }

    #[test]
    fn a_cell_holds_its_picture_inside_its_border() {
        let bands = the_bands(&the_rows());
        let plan = plan_the_bands(the_panel(71, 39), 10, the_font(), &bands, 1);
        let cell = plan.bands[0].cells[0];

        assert_eq!(cell.the_box.x, 2);
        assert_eq!(cell.the_box.y, 4);
        assert_eq!(cell.the_box.width, 10);
        assert_eq!(cell.the_picture.x, 3);
        assert_eq!(cell.the_picture.y, 5);
        assert_eq!(cell.the_picture.width, 8);
        assert_eq!(cell.the_picture.height, cell.the_box.height - 2);

        // One column stands between two cells.
        assert_eq!(plan.bands[0].cells[1].the_box.x, 13);
    }

    #[test]
    fn a_click_of_a_cell_gives_the_line_of_that_cell() {
        let bands = the_bands(&the_rows());
        let plan = plan_the_bands(the_panel(71, 39), 10, the_font(), &bands, 1);
        let cell = plan.bands[0].cells[1];

        assert_eq!(
            plan.the_cell_of_a_point(cell.the_box.x, cell.the_box.y)
                .map(|one| one.the_line),
            Some(2)
        );
        // The row of the title belongs to no cell.
        assert_eq!(plan.the_cell_of_a_point(2, 3), None);
    }

    #[test]
    fn the_row_of_a_title_says_the_name_the_count_and_the_arrows() {
        let bands = the_bands(&the_rows());
        let plan = plan_the_bands(the_panel(71, 39), 10, the_font(), &bands, 9);
        let row = the_row_of_a_title(&plan.bands[0], 71);

        assert!(row.starts_with("Continue Listening ─"), "{}", row);
        assert!(row.ends_with("‹ 6 of 12 ›"), "{}", row);
        assert_eq!(crate::logic::message::the_columns_of(&row), 71);

        // A band that holds no cell beside the ones that it draws says no arrow.
        let row = the_row_of_a_title(&plan.bands[1], 71);
        assert!(row.ends_with("2 of 2"), "{}", row);
    }

    #[test]
    fn a_row_that_has_no_room_for_the_count_holds_the_name_alone() {
        let bands = the_bands(&the_rows());
        let plan = plan_the_bands(the_panel(71, 39), 10, the_font(), &bands, 1);

        assert_eq!(the_row_of_a_title(&plan.bands[0], 0), "");
        assert_eq!(the_row_of_a_title(&plan.bands[0], 10), "Continue…");
        // The name and the count with no room for a line between them: the row
        // holds the whole name and no count at all.
        assert_eq!(the_row_of_a_title(&plan.bands[0], 20), "Continue Listening");
    }
}
