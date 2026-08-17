//! The panel 6 of the gallery of the covers. See T-327.
//!
//! **The maintainer chose the mockup 1, the panels, on 2026-08-16**, therefore
//! `docs/mockups/mockup-1.txt` is the design of the program now. The stage 6 of
//! that road gave the panel 5 its frame, its picture, its facts, and its
//! description (T-319 and T-325), and it left **the panel 6 of the gallery**
//! open. This module holds the arithmetic of that panel.
//!
//! ## The screen before this module
//!
//! **The measurement of the real program v0.8.156 inside tmux**, of the Library
//! view of the library `Large` of the sandbox at 160 columns and 45 rows. The
//! column at the right holds the panel 5 alone, and the items of that library
//! carry `coverPath: null`, therefore the panel says three facts over three
//! rows and it leaves 35 rows with no character at all:
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
//! **No area of the panel 6 stood in the code at all** (the sweep of T-323 of
//! the regions of the map of the mouse), therefore no digit, no key, and no
//! click of the user could name it.
//!
//! ## The rule of this module
//!
//! **The gallery is the list of the view and not a shelf of a series.**
//! `crate::ui::cover::plan_covers` holds a shelf of four covers of a series
//! already, and that shelf is the panel 5. The panel 6 shows the media **around
//! the cursor** of the panel 4: the user reads the covers of the rows that they
//! are near, and a click of a cell takes the cursor to that row.
//!
//! **A cell of the gallery holds two things**: the picture of the cover, and a
//! border of one line around it. The design of `docs/mockups/mockup-7.txt`
//! writes them so:
//!
//! ```text
//! ┌─6 Gallery ◉ ⇕────────────────────────────────────────────┐
//! │  ┏━━━━━━━━┓ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐  │
//! │  ┃▓▒░▒▓█▒░┃ │░▒▓█▓▒░▒│ │▒▓█░▒▓█░│ │█▓▒░█▓▒░│ │▓█▒░▓█▒░│  │
//! │  ┃▒░▓█▒░▓█┃ │▓░▒█▓░▒█│ │░▓▒█░▓▒█│ │▒█▓░▒█▓░│ │█░▓▒█░▓▒│  │
//! │  ┗━━━━━━━━┛ └────────┘ └────────┘ └────────┘ └────────┘  │
//! └──────────────────────────────────────────────────────────┘
//! ```
//!
//! **The cell held a row of the percentage and a row of the title until
//! v0.8.163** (T-330.4), and the maintainer read the two of them as noise: the
//! panel 5 says the facts of the media of the cursor already, and the gallery is
//! the picture. **The border of the cell of the cursor is heavy and bright, and
//! the border of every other cell is thin and dim**, because a colour alone is
//! not the mark of the focus. The two rows that the words gave back go to the
//! pictures: a column of the same height then holds one row of the grid more.
//!
//! **The rows of the picture come of the form of a cell of the terminal**, and
//! not of the mockup: a cell of a terminal is about two times higher than it is
//! wide, therefore a square picture of ten columns needs about five rows. The
//! boxes of the mockup hold one row of a picture, which no picture of a cover
//! can use.
//!
//! Every function of this module is pure, therefore a test of it needs no
//! terminal, no server, and no `App` at all.

use ratatui::layout::Rect;
use ratatui_image::FontSize;

/// The widths of a cell of the gallery, with its border, in columns.
///
/// **The keys `+` and `-` of the design move the user through this list**, and
/// the width of a cell decides every other number of the grid: the rows of the
/// picture come of the width, and the number of the cells comes of the two of
/// them.
pub const THE_WIDTHS_OF_A_CELL: [u16; 4] = [8, 10, 14, 20];

/// The width of a cell of the start.
///
/// The design draws cells of ten columns, and the second width of the list is
/// the nearest one that holds a picture of more than two rows.
pub const THE_WIDTH_OF_THE_START: usize = 1;

/// The largest share of the height of the column that the gallery takes, in
/// percent.
///
/// The design of `docs/mockups/mockup-1.txt` gives the panel 5 nineteen rows of
/// the thirty-one of its column, and the panel 6 twelve of them. **The gallery
/// takes whole rows of the grid alone**, therefore this number is a limit and
/// not a share: a panel of a part of a row would hold rows of the screen that
/// no cell uses, and the first form of this panel held five such rows.
pub const THE_SHARE_OF_THE_GALLERY: u16 = 50;

/// The rows that the panel 5 needs before the gallery can stand under it.
///
/// **A gallery that took the rows of the words of the media would give the user
/// a picture and no word of it**: the panel 5 holds the facts and the
/// description of the media of the cursor, and those words are the answer of
/// the question that a cover asks.
pub const THE_SMALLEST_PANEL_OF_THE_COVER: u16 =
    crate::ui::the_panel_of_the_cover::THE_SMALLEST_PANEL_OF_THE_WORDS + 2;

/// One media of the gallery, in the words that the screen holds already.
///
/// `App::the_media_of_the_gallery` gives one of these for each row of the list
/// of the view that names a media, and the row of a shelf and the row of a
/// series give none.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AMediaOfTheGallery {
    /// The line of the list of the view that names this media. A click of the
    /// cell moves the cursor to that line.
    pub the_line: usize,
    /// The identity of the media, for the store of the covers. A media of no
    /// identity holds no text, and the cell of it shows no picture.
    pub id: String,
    /// The title of the media, for a cell that draws no picture. See T-339.
    ///
    /// **The cell holds the picture and its border alone** (T-330.4) while a
    /// picture comes. A terminal of no protocol of pictures, a `TOUTUI_NO_COVERS`
    /// of the user, and a media that the server holds with no cover each give a
    /// cell of a border and nothing at all: the title then stands in the rows of
    /// the picture, and the user reads which media the cell holds.
    pub the_title: String,
}

/// The text of the title of a media in the rows of the picture of a cell. See
/// T-339.
///
/// **The cell keeps its border and its place**, and the band does not become a
/// table: the keys of a view must not change with the terminal of the user
/// (`docs/mockups/mockup-6.md`).
///
/// The words take the rows of the picture with the wrap of this program, and a
/// title that needs more rows than the cell holds loses its end to the three
/// points of [`crate::logic::message::in_the_rows`]: a text that the screen cuts
/// with no mark says nothing to the user (T-91).
///
/// ```text
/// ┏━━━━━━━━┓ ┌────────┐
/// ┃The     ┃ │Depthle…│
/// ┃Kingkil…┃ │Hunger  │
/// ┗━━━━━━━━┛ └────────┘
/// ```
///
/// The function is pure, therefore a test of it needs no terminal and no server.
pub fn the_title_of_a_cell(title: &str, of_the_picture: Rect) -> String {
    crate::logic::message::in_the_rows(title, of_the_picture.width, of_the_picture.height)
}

/// One cell of the gallery: a media of the list, and the areas of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ACellOfTheGallery {
    /// The place of the media in the list of the media of the gallery.
    pub the_media: usize,
    /// The box of the cell, with its border.
    pub the_box: Rect,
    /// The area of the picture of the cover, inside the border. It holds every
    /// row of the box that the border leaves (T-330.4).
    pub the_picture: Rect,
}

/// The grid of the gallery.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TheGallery {
    /// The cells of the grid, from the first one at the left of the first row.
    pub cells: Vec<ACellOfTheGallery>,
    /// The place of the first media of the grid, in the list of the media.
    pub the_first: usize,
    /// The number of the cells of one row of the grid.
    pub the_columns: usize,
    /// The number of the rows of the grid.
    pub the_rows: usize,
}

impl TheGallery {
    /// The cell of a point of the screen, and `None` for a point that stands
    /// beside every cell.
    ///
    /// **The box of a cell is the whole of that cell** (T-330.4): the row of the
    /// title under the box went away with the words, therefore a click of the
    /// border of a cover is a click of that cover and no point of the panel
    /// belongs to two cells.
    pub fn the_cell_of_a_point(&self, column: u16, row: u16) -> Option<&ACellOfTheGallery> {
        let point = ratatui::layout::Position::new(column, row);

        self.cells.iter().find(|cell| cell.the_box.contains(point))
    }
}

/// The rows of the picture of a cell of this width.
///
/// A cell of a terminal is higher than it is wide, therefore a square picture
/// of `n` columns needs about `n / 2` rows. A font of no size at all gives one
/// row, which is the value that keeps every other number above zero.
fn the_rows_of_a_picture(of_a_cell: u16, font: FontSize) -> u16 {
    let inside = of_a_cell.saturating_sub(2);

    if inside == 0 || font.width == 0 || font.height == 0 {
        return 1;
    }

    (u32::from(inside) * u32::from(font.width) / u32::from(font.height))
        .max(1)
        .min(u32::from(u16::MAX)) as u16
}

/// The rows of a box of a cell of this width: the two rows of the border, and
/// the picture between them.
///
/// **The box is one row of the grid too** (T-330.4): the row of the percentage
/// inside the border and the row of the title under the box each went away, and
/// the rows that they gave back go to the pictures.
pub fn the_rows_of_a_box(of_a_cell: u16, font: FontSize) -> u16 {
    the_rows_of_a_picture(of_a_cell, font) + 2
}

/// The smallest panel 6 that holds one row of the grid, with its border.
pub fn the_smallest_gallery(of_a_cell: u16, font: FontSize) -> u16 {
    the_rows_of_a_box(of_a_cell, font) + 2
}

/// Divides the column at the right of the list into the panel 5 of the cover
/// and the panel 6 of the gallery. See T-327.
///
/// `column` is the whole column, with the border of each panel inside it.
///
/// **The gallery goes away before the panel 5 does**: a column that holds no
/// room for the two panels gives every row to the panel 5, because the words of
/// the media of the cursor say more than the covers of the media beside it.
pub fn the_two_panels(column: Rect, of_a_cell: u16, font: FontSize) -> (Rect, Option<Rect>) {
    let the_smallest = the_smallest_gallery(of_a_cell, font);

    if column.height < THE_SMALLEST_PANEL_OF_THE_COVER + the_smallest {
        return (column, None);
    }

    // **The gallery holds whole rows of the grid alone**: a panel of a part of
    // a row would hold rows of the screen that no cell uses. The share of the
    // design is therefore a limit, and the panel takes the rows of the grid
    // that stand under it.
    let of_a_row = the_rows_of_a_box(of_a_cell, font);
    let of_the_share = column.height * THE_SHARE_OF_THE_GALLERY / 100;
    let the_rows = (of_the_share.saturating_sub(2) / of_a_row).max(1);
    let of_the_gallery = the_rows * of_a_row + 2;

    // The panel 5 keeps the rows that its words need, therefore a gallery of
    // more than one row goes back to one row before it takes them.
    let of_the_gallery = if column.height - of_the_gallery < THE_SMALLEST_PANEL_OF_THE_COVER {
        the_smallest
    } else {
        of_the_gallery
    };

    let cover = Rect {
        height: column.height - of_the_gallery,
        ..column
    };

    let gallery = Rect {
        y: column.y + cover.height,
        height: of_the_gallery,
        ..column
    };

    (cover, Some(gallery))
}

/// Gives the grid of the gallery of a panel. See T-327.
///
/// `inside` is the area inside the border of the panel 6. `the_media` is the
/// number of the media of the list of the view, and `the_cursor` is the place
/// of the media of the cursor in that list.
///
/// **The grid stands around the cursor**: the row of the cursor goes in the
/// middle of the rows of the grid, therefore the user sees the covers before
/// the cursor and the covers after it together.
///
/// The function is pure, therefore a test needs no terminal and no server.
pub fn plan_the_gallery(
    inside: Rect,
    of_a_cell: u16,
    font: FontSize,
    the_media: usize,
    the_cursor: usize,
) -> TheGallery {
    let of_a_row = the_rows_of_a_box(of_a_cell, font);

    if inside.width < of_a_cell || inside.height < of_a_row || the_media == 0 || of_a_cell < 3 {
        return TheGallery::default();
    }

    // One column stays between two cells of a row.
    let the_columns = usize::from((inside.width + 1) / (of_a_cell + 1)).max(1);
    let the_rows = usize::from(inside.height / of_a_row);

    if the_rows == 0 {
        return TheGallery::default();
    }

    // The row of the cursor stands in the middle of the rows of the grid, and
    // the last row of the grid holds the last media of the list.
    let of_the_cursor = the_cursor.min(the_media - 1) / the_columns;
    let the_last_row = the_media.div_ceil(the_columns);
    let the_highest = the_last_row.saturating_sub(the_rows);
    let the_first_row = of_the_cursor.saturating_sub(the_rows / 2).min(the_highest);
    let the_first = the_first_row * the_columns;

    // The grid stands in the middle of the width of the panel, so that the
    // columns that no cell uses stand at the two ends of it.
    let of_the_grid = the_columns as u16 * (of_a_cell + 1) - 1;
    let x = inside.x + (inside.width - of_the_grid) / 2;

    let the_rows_of_a_picture = the_rows_of_a_picture(of_a_cell, font);
    let mut cells = Vec::new();

    for row in 0..the_rows {
        for column in 0..the_columns {
            let the_media_of_the_cell = the_first + row * the_columns + column;

            if the_media_of_the_cell >= the_media {
                break;
            }

            let the_box = Rect {
                x: x + column as u16 * (of_a_cell + 1),
                y: inside.y + row as u16 * of_a_row,
                width: of_a_cell,
                height: the_rows_of_a_box(of_a_cell, font),
            };

            cells.push(ACellOfTheGallery {
                the_media: the_media_of_the_cell,
                the_box,
                the_picture: Rect {
                    x: the_box.x + 1,
                    y: the_box.y + 1,
                    width: of_a_cell - 2,
                    height: the_rows_of_a_picture,
                },
            });
        }
    }

    TheGallery {
        cells,
        the_first,
        the_columns,
        the_rows,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The font of a terminal of this measurement: a cell of 8 pixels by 16.
    const FONT: FontSize = FontSize {
        width: 8,
        height: 16,
    };

    /// The column at the right holds the panel 5 over the panel 6, and the two
    /// of them use every row of it. See T-327.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn the_column_holds_the_panel_of_the_cover_over_the_gallery() {
        // The measurement of 2026-08-16: the column of the covers of a screen
        // of 160 by 45 stands at the column 110 and it holds 41 rows.
        let column = Rect::new(110, 2, 50, 41);
        let of_a_cell = THE_WIDTHS_OF_A_CELL[THE_WIDTH_OF_THE_START];

        let (cover, gallery) = the_two_panels(column, of_a_cell, FONT);
        let gallery = gallery.expect("a column of 41 rows holds the two panels");

        assert_eq!(cover.y, column.y);
        assert_eq!(gallery.y, cover.y + cover.height);
        assert_eq!(cover.height + gallery.height, column.height);
        assert_eq!(gallery.x, column.x);
        assert_eq!(gallery.width, column.width);

        // The panel 5 keeps the rows that the words of the media need.
        assert!(
            cover.height >= THE_SMALLEST_PANEL_OF_THE_COVER,
            "the panel 5 holds {} rows and it needs {THE_SMALLEST_PANEL_OF_THE_COVER}",
            cover.height
        );
        assert!(gallery.height >= the_smallest_gallery(of_a_cell, FONT));

        // **A column that holds no room for the two panels gives every row to
        // the panel 5**: the words of the media of the cursor say more than the
        // covers of the media beside it.
        let short = Rect::new(110, 2, 50, THE_SMALLEST_PANEL_OF_THE_COVER + 1);
        assert_eq!(the_two_panels(short, of_a_cell, FONT), (short, None));

        // A cell that is larger needs a taller column, therefore the same
        // column gives no gallery for the largest cell of the list.
        let of_the_largest = THE_WIDTHS_OF_A_CELL[THE_WIDTHS_OF_A_CELL.len() - 1];
        assert!(
            the_smallest_gallery(of_the_largest, FONT) > the_smallest_gallery(of_a_cell, FONT),
            "a larger cell must need more rows"
        );
    }

    /// The grid holds the cells of the media around the cursor, and no cell
    /// stands over another one. See T-327.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn the_grid_holds_the_cells_of_the_media_around_the_cursor() {
        // The panel 6 of the measurement of this round: 50 columns and 16 rows,
        // and 48 columns and 14 rows inside its border.
        let inside = Rect::new(111, 27, 48, 14);
        let of_a_cell = THE_WIDTHS_OF_A_CELL[THE_WIDTH_OF_THE_START];

        let plan = plan_the_gallery(inside, of_a_cell, FONT, 500, 0);

        // A cell of 10 columns and one column between two of them gives four
        // cells of a row of 48 columns.
        assert_eq!(plan.the_columns, 4);
        assert!(plan.the_rows >= 1);
        assert_eq!(plan.the_first, 0);
        assert_eq!(plan.cells.len(), plan.the_columns * plan.the_rows);

        // Every cell stands inside the panel, and no two cells hold one cell of
        // the screen.
        for (at, cell) in plan.cells.iter().enumerate() {
            assert_eq!(cell.the_media, at);
            assert!(
                inside.union(cell.the_box) == inside,
                "{cell:?} left the panel"
            );
            assert_eq!(
                cell.the_picture.height,
                the_rows_of_a_picture(of_a_cell, FONT)
            );

            // **The picture holds every row of the box that the border
            // leaves** (T-330.4).
            assert_eq!(cell.the_picture.height + 2, cell.the_box.height);
            assert_eq!(cell.the_picture.y, cell.the_box.y + 1);

            for other in plan.cells.iter().skip(at + 1) {
                assert!(
                    cell.the_box.intersection(other.the_box).area() == 0,
                    "the cells {cell:?} and {other:?} hold one cell of the screen"
                );
            }
        }

        // **The row of the cursor stands in the middle of the grid**: a cursor
        // of the middle of a long list gives a first media above it.
        let of_the_middle = plan_the_gallery(inside, of_a_cell, FONT, 500, 250);
        assert!(
            of_the_middle.the_first < 250,
            "the grid must hold the media before the cursor"
        );
        assert!(
            of_the_middle.cells.iter().any(|cell| cell.the_media == 250),
            "the grid must hold the media of the cursor"
        );

        // The last media of the list stands in the last grid, and the grid then
        // holds no cell of a media that the list does not have.
        let of_the_end = plan_the_gallery(inside, of_a_cell, FONT, 500, 499);
        assert!(of_the_end.cells.iter().any(|cell| cell.the_media == 499));
        assert!(of_the_end.cells.iter().all(|cell| cell.the_media < 500));

        // A list of fewer media than the grid holds gives one cell for each of
        // them and no more.
        let of_two = plan_the_gallery(inside, of_a_cell, FONT, 2, 0);
        assert_eq!(of_two.cells.len(), 2);
        assert_eq!(of_two.the_first, 0);
    }

    /// A panel that holds no room for one cell holds no grid at all, and a list
    /// of no media holds none either. See T-327.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn a_panel_that_holds_no_cell_holds_no_grid() {
        let of_a_cell = THE_WIDTHS_OF_A_CELL[THE_WIDTH_OF_THE_START];
        let of_a_row = the_rows_of_a_box(of_a_cell, FONT);

        for inside in [
            Rect::default(),
            // One column under the width of a cell.
            Rect::new(111, 27, of_a_cell - 1, 20),
            // One row under the rows of a row of the grid.
            Rect::new(111, 27, 48, of_a_row - 1),
        ] {
            let plan = plan_the_gallery(inside, of_a_cell, FONT, 500, 0);
            assert_eq!(
                plan,
                TheGallery::default(),
                "the panel {inside:?} holds no grid"
            );
        }

        // **A list of no media holds no cell**: an empty library and the frame
        // of the start of the program each give that condition.
        let plan = plan_the_gallery(Rect::new(111, 27, 48, 14), of_a_cell, FONT, 0, 0);
        assert!(plan.cells.is_empty());

        // A font of no size at all gives no panic, and the picture then holds
        // one row.
        let no_font = FontSize {
            width: 0,
            height: 0,
        };
        assert_eq!(the_rows_of_a_picture(of_a_cell, no_font), 1);
        assert!(
            !plan_the_gallery(Rect::new(111, 27, 48, 14), of_a_cell, no_font, 20, 0)
                .cells
                .is_empty()
        );
    }

    /// A point of the screen names the cell of the box under it and the cell of
    /// the title under it. See T-327.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn a_point_of_the_screen_names_the_cell_of_the_gallery() {
        let inside = Rect::new(111, 27, 48, 14);
        let of_a_cell = THE_WIDTHS_OF_A_CELL[THE_WIDTH_OF_THE_START];
        let plan = plan_the_gallery(inside, of_a_cell, FONT, 500, 0);

        let first = plan.cells[0];
        let second = plan.cells[1];

        assert_eq!(
            plan.the_cell_of_a_point(first.the_picture.x, first.the_picture.y)
                .map(|cell| cell.the_media),
            Some(first.the_media)
        );

        // **The border of a cell belongs to that cell** (T-330.4): the box is
        // the whole of the cell, and the row of the title went away.
        assert_eq!(
            plan.the_cell_of_a_point(first.the_box.x, first.the_box.y + first.the_box.height - 1)
                .map(|cell| cell.the_media),
            Some(first.the_media)
        );

        assert_eq!(
            plan.the_cell_of_a_point(second.the_box.x, second.the_box.y)
                .map(|cell| cell.the_media),
            Some(second.the_media)
        );

        // The column between two cells belongs to no cell at all.
        assert_eq!(
            plan.the_cell_of_a_point(first.the_box.x + of_a_cell, first.the_box.y),
            None
        );

        // A point outside the panel names no cell.
        assert_eq!(plan.the_cell_of_a_point(0, 0), None);
    }
}
