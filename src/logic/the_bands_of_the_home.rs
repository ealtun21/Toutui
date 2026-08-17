//! The bands of the Home view. See T-331 and T-335.
//!
//! The maintainer asked for a Home view of bands of covers on 2026-08-16, and
//! `docs/mockups/mockup-6.txt` and
//! `docs/superpowers/specs/2026-08-17-the-home-view-of-the-bands-of-covers-design.md`
//! hold the design of it. **The bands are a shape of the render, and the flat
//! list of the lines of `crate::logic::home_view` stays the data**: the cursor
//! of the view stays one line of that flat list, therefore every key of a
//! media, the panel 5 of the facts, the message of a media that went away, and
//! the click of a row keep their work with no change at all.
//!
//! This module makes the bands of that flat list, and it gives the moves of the
//! keys `h`, `l`, `j`, `k`, `g`, and `G` over them. The functions are pure,
//! therefore a test needs no server and no screen.
//!
//! **A cell holds the line of the flat list**, and never the number of the
//! media: `HomeRow::Media` and `HomeRow::Series` each stand on a line, and the
//! two of them take a cell.

use crate::logic::home_view::HomeRow;

/// One band of the Home view: the title of a shelf, and the cells under it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ABand {
    /// The name of the shelf, of `HomeRow::Shelf`.
    pub the_title: String,
    /// The line of the flat list of each cell, in the sequence of the shelf.
    pub the_cells: Vec<usize>,
}

/// Makes the bands of the lines of the Home view.
///
/// One band for one shelf, in the sequence of the server, and one cell for one
/// line that the user can select. The decision 2 of the maintainer says that
/// sequence, and `group_home` gives it already.
///
/// **A band of no cell gives no band.** `group_home` drops a shelf that holds
/// no line already (the shelf `newest-authors` of a library of books is that
/// shelf), and this rule holds for the lines that
/// `home_view::without_the_media_that_left` gives too.
///
/// **A cell that stands under no title takes a band of its own**, with the name
/// of a shelf with no name. The lines of `group_home` always name the shelf
/// above their media, therefore no answer of the server reaches this road; a
/// line of a media that this function dropped would take the media of the user
/// away from the screen, and a line that stays is the safe road (T-203).
pub fn the_bands(rows: &[HomeRow]) -> Vec<ABand> {
    let mut bands: Vec<ABand> = Vec::new();

    for (line, row) in rows.iter().enumerate() {
        match row {
            HomeRow::Shelf { label } => bands.push(ABand {
                the_title: label.clone(),
                the_cells: Vec::new(),
            }),
            _ => {
                if bands.is_empty() {
                    bands.push(ABand {
                        the_title: crate::logic::home_view::the_name_of_the_shelf(None, ""),
                        the_cells: Vec::new(),
                    });
                }

                if let Some(band) = bands.last_mut() {
                    band.the_cells.push(line);
                }
            }
        }
    }

    bands.retain(|band| !band.the_cells.is_empty());
    bands
}

/// Gives the band and the cell of a line of the flat list.
///
/// The answer is `None` for a line that names a shelf, for a line that no band
/// holds, and for a Home view of no band at all.
pub fn the_place_of_the_line(bands: &[ABand], line: usize) -> Option<(usize, usize)> {
    for (number, band) in bands.iter().enumerate() {
        if let Some(cell) = band.the_cells.iter().position(|one| *one == line) {
            return Some((number, cell));
        }
    }

    None
}

/// Gives the line of the cell at the left, for the key `h`.
///
/// **The move stops at the first cell of the band**, and it does not go to the
/// band above: the move inside a band is a move of a picture, and a jump to the
/// other end of a shelf of covers says nothing to the user (the decision 3 of
/// the design).
pub fn the_cell_at_the_left(bands: &[ABand], line: usize) -> Option<usize> {
    let Some((band, cell)) = the_place_of_the_line(bands, line) else {
        return the_first_cell_of_the_view(bands);
    };

    bands[band].the_cells.get(cell.saturating_sub(1)).copied()
}

/// Gives the line of the cell at the right, for the key `l`.
///
/// The move stops at the last cell of the band. See `the_cell_at_the_left`.
pub fn the_cell_at_the_right(bands: &[ABand], line: usize) -> Option<usize> {
    let Some((band, cell)) = the_place_of_the_line(bands, line) else {
        return the_first_cell_of_the_view(bands);
    };

    let cells = &bands[band].the_cells;
    cells.get(cell + 1).or_else(|| cells.last()).copied()
}

/// Gives the line of the band under this one, for the key `j`.
///
/// **The cell keeps its number in the new band**, and the last cell of that
/// band takes it when the band is shorter. **The move goes round at the last
/// band**, which is the rule of `home_view::next_line` of today.
pub fn the_band_under(bands: &[ABand], line: usize) -> Option<usize> {
    the_band_beside(bands, line, 1)
}

/// Gives the line of the band above this one, for the key `k`.
///
/// The move goes round at the first band. See `the_band_under`.
pub fn the_band_above(bands: &[ABand], line: usize) -> Option<usize> {
    the_band_beside(bands, line, -1)
}

/// The work of `the_band_under` and of `the_band_above`.
fn the_band_beside(bands: &[ABand], line: usize, step: isize) -> Option<usize> {
    let Some((band, cell)) = the_place_of_the_line(bands, line) else {
        return the_first_cell_of_the_view(bands);
    };

    let how_many = bands.len() as isize;
    let next = (band as isize + step).rem_euclid(how_many) as usize;
    let cells = &bands[next].the_cells;

    cells.get(cell).or_else(|| cells.last()).copied()
}

/// Gives the first cell of the band of the cursor, for the key `g`.
pub fn the_first_cell_of_the_band(bands: &[ABand], line: usize) -> Option<usize> {
    let Some((band, _)) = the_place_of_the_line(bands, line) else {
        return the_first_cell_of_the_view(bands);
    };

    bands[band].the_cells.first().copied()
}

/// Gives the last cell of the band of the cursor, for the key `G`.
pub fn the_last_cell_of_the_band(bands: &[ABand], line: usize) -> Option<usize> {
    let Some((band, _)) = the_place_of_the_line(bands, line) else {
        return the_first_cell_of_the_view(bands);
    };

    bands[band].the_cells.last().copied()
}

/// Gives the first cell of the first band.
///
/// **A cursor that stands on no cell takes this line.** The cursor of the flat
/// list can stand on the line of a shelf after a refresh, and every move of the
/// bands then starts at the first media of the view, which is the rule of
/// `home_view::first_line`.
fn the_first_cell_of_the_view(bands: &[ABand]) -> Option<usize> {
    bands
        .first()
        .and_then(|band| band.the_cells.first())
        .copied()
}

/// Gives the count of the title of a band: `6 of 24`.
///
/// **The count says the media that the program holds**, and never the field
/// `total` of the shelf of the server: a shelf holds ten entities at the most
/// and `recently-added` of the library `Large` of the sandbox says
/// `total: 2056`, therefore a band that said `6 of 2056` would promise 2050
/// media that no key of the user can reach (T-118, and the decision 4 of the
/// design).
///
/// `draws` is the number of the cells that the panel has the room for, and
/// `holds` is the number of the cells of the band. A panel that has the room
/// for more cells than the band holds says the cells of the band.
pub fn the_count_of_a_band(draws: usize, holds: usize) -> String {
    format!("{} of {}", draws.min(holds), holds)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A shelf of two media, a shelf of one series, and a shelf of one media.
    fn the_rows() -> Vec<HomeRow> {
        vec![
            HomeRow::Shelf {
                label: "Continue Listening".to_string(),
            },
            HomeRow::Media { item: 0 },
            HomeRow::Media { item: 1 },
            HomeRow::Shelf {
                label: "Recent Series".to_string(),
            },
            HomeRow::Series { series: 0 },
            HomeRow::Shelf {
                label: "Discover".to_string(),
            },
            HomeRow::Media { item: 2 },
        ]
    }

    #[test]
    fn a_band_holds_the_lines_of_its_shelf() {
        let bands = the_bands(&the_rows());

        assert_eq!(bands.len(), 3);
        assert_eq!(bands[0].the_title, "Continue Listening");
        assert_eq!(bands[0].the_cells, vec![1, 2]);
        assert_eq!(bands[1].the_cells, vec![4]);
        assert_eq!(bands[2].the_cells, vec![6]);
    }

    #[test]
    fn a_shelf_of_no_media_gives_no_band() {
        let rows = vec![
            HomeRow::Shelf {
                label: "Newest Authors".to_string(),
            },
            HomeRow::Shelf {
                label: "Discover".to_string(),
            },
            HomeRow::Media { item: 0 },
        ];

        let bands = the_bands(&rows);

        assert_eq!(bands.len(), 1);
        assert_eq!(bands[0].the_title, "Discover");
        assert_eq!(bands[0].the_cells, vec![2]);
    }

    #[test]
    fn the_moves_of_a_band_stop_at_its_two_ends() {
        let bands = the_bands(&the_rows());

        assert_eq!(the_cell_at_the_right(&bands, 1), Some(2));
        assert_eq!(the_cell_at_the_right(&bands, 2), Some(2));
        assert_eq!(the_cell_at_the_left(&bands, 2), Some(1));
        assert_eq!(the_cell_at_the_left(&bands, 1), Some(1));
    }

    #[test]
    fn the_moves_of_the_bands_go_round() {
        let bands = the_bands(&the_rows());

        assert_eq!(the_band_under(&bands, 1), Some(4));
        assert_eq!(the_band_under(&bands, 6), Some(1));
        assert_eq!(the_band_above(&bands, 1), Some(6));
    }

    #[test]
    fn the_cell_of_the_cursor_keeps_its_number_in_the_new_band() {
        let bands = the_bands(&the_rows());

        // The cell 1 of the first band, and the band under it holds one cell.
        assert_eq!(the_band_under(&bands, 2), Some(4));
    }

    #[test]
    fn the_two_ends_of_a_band_come_of_the_key_g_and_of_the_key_of_the_capital_g() {
        let bands = the_bands(&the_rows());

        assert_eq!(the_first_cell_of_the_band(&bands, 2), Some(1));
        assert_eq!(the_last_cell_of_the_band(&bands, 1), Some(2));
        assert_eq!(the_first_cell_of_the_band(&bands, 4), Some(4));
        assert_eq!(the_last_cell_of_the_band(&bands, 4), Some(4));
    }

    #[test]
    fn a_line_of_a_shelf_gives_the_first_media_of_the_view() {
        let bands = the_bands(&the_rows());

        for line in [0, 3, 5, 99] {
            assert_eq!(the_place_of_the_line(&bands, line), None);
            assert_eq!(the_cell_at_the_right(&bands, line), Some(1));
            assert_eq!(the_cell_at_the_left(&bands, line), Some(1));
            assert_eq!(the_band_under(&bands, line), Some(1));
            assert_eq!(the_first_cell_of_the_band(&bands, line), Some(1));
        }
    }

    #[test]
    fn a_home_view_of_no_band_gives_no_move_and_no_fault() {
        let bands = the_bands(&[]);

        assert!(bands.is_empty());
        assert_eq!(the_place_of_the_line(&bands, 0), None);
        assert_eq!(the_cell_at_the_right(&bands, 0), None);
        assert_eq!(the_band_under(&bands, 0), None);
        assert_eq!(the_last_cell_of_the_band(&bands, 0), None);
    }

    #[test]
    fn a_media_under_no_title_keeps_its_cell() {
        let bands = the_bands(&[HomeRow::Media { item: 0 }, HomeRow::Media { item: 1 }]);

        assert_eq!(bands.len(), 1);
        assert_eq!(bands[0].the_title, "A shelf with no name");
        assert_eq!(bands[0].the_cells, vec![0, 1]);
    }

    #[test]
    fn the_count_of_a_title_says_the_media_of_the_program() {
        assert_eq!(the_count_of_a_band(6, 24), "6 of 24");
        assert_eq!(the_count_of_a_band(8, 5), "5 of 5");
        assert_eq!(the_count_of_a_band(0, 0), "0 of 0");
    }
}
