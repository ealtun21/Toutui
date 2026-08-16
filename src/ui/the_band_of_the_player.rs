//! The panel 7 of the frame, the band of the player. See T-322.
//!
//! **The maintainer chose the mockup 1, the panels, on 2026-08-16**, therefore
//! `docs/mockups/mockup-1.txt` is the design of the program now. This module
//! holds the stage 7 of that road: the band of the player, at the foot of the
//! screen, under the panels of the work of the view.
//!
//! ## The screen of the fault, of the real program v0.8.151 inside tmux
//!
//! The library `Books` of the sandbox at 160 columns and 45 rows, the key `l`
//! of the row `A Book Of Many Hours`, which is a book of eight hours: the three
//! rows of the player stood **in the air**, under the frame of the panels, with
//! no border, no title, and no number of a panel at all, and they said
//!
//! ```text
//!                A Book Of Many Hours by Many Hours Author | The hours of the start
//!            ▶ 1:14:07 / 8:00:00 | Elapsed: 1:14:07 | Left: 6:45:53 (15%) | Speed: 1.00x
//!   Spc: pause/play | p/u: +/−10s | P/U: nxt/prev ch. | O/I: spd +/− | o/i: vol +/− | t: sleep | Y: quit
//! ```
//!
//! **The band held no bar at all**: the place of the user in the book stood in
//! a percent of two digits and in nothing else, the place of the user in the
//! chapter stood nowhere, and 160 columns of a screen said `(15%)`. **The
//! design gives that row a bar of the seek with the two times at its two ends,
//! and a bar of the book and a bar of the chapter under it.**
//!
//! ## The rows of the band
//!
//! The band holds four rows inside its border, in the sequence of the design:
//!
//! 1. **The words**: the mark of the playback, the title, the author, the
//!    chapter, the speed, the volume, and the timer for sleep.
//! 2. **The seek**: the place of the user at the left, the bar between the two
//!    times, and the length of the media at the right.
//! 3. **The two bars**: the place in the book, and the place in the chapter.
//! 4. **The buttons**: the keys of the player, which the key `B` takes away.
//!
//! **A band that lost the row of the buttons holds three rows**
//! ([`the_rows_of_the_band`]), and the rows of that row then go to the work of
//! the view: that is the rule of T-302 for the footer, and of T-299 for the row
//! of the message.
//!
//! ## The bar of the seek, and the click of it
//!
//! **A bar of the seek that no click moves is a picture and not a control**
//! (the section (e) of `docs/mockups/mockup-1.md`), therefore
//! [`the_second_of_a_column`] is the opposite of [`the_bar_of_the_seek`]: the
//! render writes the cells of the bar of the place, and a click of a cell of
//! that bar gives the second of the media that the cell holds.
//!
//! Every function of this module is pure, therefore a test of it needs no
//! engine, no server, and no screen.

use crate::player::engine::track::Chapter;
use ratatui::layout::Rect;

/// The rows of the band inside its border, with the row of the buttons.
pub const THE_ROWS_INSIDE_WITH_THE_BUTTONS: u16 = 4;

/// The rows of the band inside its border, with no row of the buttons.
pub const THE_ROWS_INSIDE_WITH_NO_BUTTON: u16 = 3;

/// The smallest bar of the seek that says a place of the user.
///
/// A bar of fewer cells gives one cell to more than a tenth of the media, and a
/// click of it then moves the playback by more than that: the row of a screen
/// that is too narrow therefore says the two times alone.
pub const THE_SMALLEST_BAR: u16 = 8;

/// The cell of the bar of the seek that the playback passed.
pub const THE_CELL_THAT_PLAYED: char = '█';

/// The cell of the bar of the seek that holds the place of the user.
pub const THE_CELL_OF_THE_PLACE: char = '▒';

/// The cell of the bar of the seek that the playback did not reach.
pub const THE_CELL_THAT_STAYS: char = '░';

/// The rows of the whole band, with its border.
///
/// **The row of the buttons goes away with the key `B`**, and the band then
/// stands on the rows that it needs: the work of the view takes the row that
/// the buttons left, in the same way as the row of the footer of T-302.
pub const fn the_rows_of_the_band(the_buttons_stand: bool) -> u16 {
    let inside = if the_buttons_stand {
        THE_ROWS_INSIDE_WITH_THE_BUTTONS
    } else {
        THE_ROWS_INSIDE_WITH_NO_BUTTON
    };

    // One row of the border above, and one under.
    inside + 2
}

/// The four rows of the band, inside its border.
///
/// **A row that the band has no room for is `Rect::default()`**, which holds no
/// cell of the screen at all: a terminal of few rows keeps the words and the
/// seek, which are the two rows that say the media and the place of the user,
/// and it loses the bars and the buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ThePartsOfTheBand {
    /// The row of the title, the author, the chapter, and the settings.
    pub the_words: Rect,
    /// The row of the bar of the seek, with the two times.
    pub the_seek: Rect,
    /// The row of the bar of the book and of the bar of the chapter.
    pub the_bars: Rect,
    /// The row of the keys of the player.
    pub the_buttons: Rect,
}

/// Divides the inside of the band into its rows.
pub fn the_parts_of_the_band(inside: Rect) -> ThePartsOfTheBand {
    let end = inside.y.saturating_add(inside.height);
    let mut y = inside.y;

    let a_row = |y: &mut u16| -> Rect {
        if *y >= end || inside.width == 0 {
            return Rect::default();
        }

        let row = Rect::new(inside.x, *y, inside.width, 1);
        *y = y.saturating_add(1);
        row
    };

    ThePartsOfTheBand {
        the_words: a_row(&mut y),
        the_seek: a_row(&mut y),
        the_bars: a_row(&mut y),
        the_buttons: a_row(&mut y),
    }
}

/// The three parts of the row of the seek: the time of the place, the cells of
/// the bar, and the length of the media.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThePartsOfTheSeek {
    /// The columns of the time of the place of the user.
    pub the_time_of_the_place: Rect,
    /// The cells of the bar, between `├` and `┤`.
    pub the_bar: Rect,
    /// The columns of the length of the media.
    pub the_length: Rect,
}

/// Divides the row of the seek, and gives `None` for a row that is too narrow
/// for a bar of [`THE_SMALLEST_BAR`] cells.
///
/// `of_the_place` and `of_the_length` are the columns of the two texts, and not
/// the characters of them: a text of a character of two columns therefore keeps
/// its room (the trap 245).
///
/// The row holds one space at each end, one space at each side of the bar, and
/// the two ends `├` and `┤` of it.
pub fn the_parts_of_the_seek(
    row: Rect,
    of_the_place: usize,
    of_the_length: usize,
) -> Option<ThePartsOfTheSeek> {
    // A space, the time, a space, `├`, `┤`, a space, the length, and a space.
    let outside = of_the_place.checked_add(of_the_length)?.checked_add(6)?;
    let width = usize::from(row.width);
    let cells = width.checked_sub(outside)?;

    if cells < usize::from(THE_SMALLEST_BAR) || cells > usize::from(u16::MAX) {
        return None;
    }

    let of_the_place = of_the_place as u16;
    let of_the_length = of_the_length as u16;
    let cells = cells as u16;

    let the_time_of_the_place = Rect::new(row.x + 1, row.y, of_the_place, 1);
    let the_bar = Rect::new(the_time_of_the_place.right() + 2, row.y, cells, 1);
    let the_length = Rect::new(the_bar.right() + 2, row.y, of_the_length, 1);

    Some(ThePartsOfTheSeek {
        the_time_of_the_place,
        the_bar,
        the_length,
    })
}

/// The cells of the bar of the seek.
///
/// **A media of no length gives a bar of no place at all** (T-180): the audio
/// files of a book of a server of another version hold no length, and a bar
/// that showed the place of the user in a length of zero would be a
/// measurement that no program made. Every cell of such a bar therefore says
/// that the playback did not reach it.
///
/// The cell of the place stands between the part that played and the part that
/// stays, and a media that came to its end holds no such cell.
pub fn the_bar_of_the_seek(width: u16, position: u32, length: u32) -> String {
    if width == 0 {
        return String::new();
    }

    if length == 0 {
        return THE_CELL_THAT_STAYS.to_string().repeat(usize::from(width));
    }

    let played = the_cells_that_played(width, position, length);
    let stays = usize::from(width) - played;

    let mut bar = THE_CELL_THAT_PLAYED.to_string().repeat(played);

    if stays > 0 {
        bar.push(THE_CELL_OF_THE_PLACE);
        bar.push_str(&THE_CELL_THAT_STAYS.to_string().repeat(stays - 1));
    }

    bar
}

/// The number of cells of a bar of this width that the playback passed.
///
/// **The render reads this number for the colour of the cells** (T-322): the
/// cells that played take the one accent of the program, and a colour that came
/// of the percent of the row would stand one cell away from the cells that
/// [`the_bar_of_the_seek`] wrote.
pub fn the_cells_that_played(width: u16, position: u32, length: u32) -> usize {
    if length == 0 {
        return 0;
    }

    let cells = u64::from(width) * u64::from(position) / u64::from(length);

    (cells as usize).min(usize::from(width))
}

/// The second of the media that a column of the bar of the seek holds.
///
/// **This is the opposite of [`the_bar_of_the_seek`]**: it gives the smallest
/// second of the media whose bar holds that number of cells that played,
/// therefore a click of a cell takes the playback to a place where the bar
/// paints that same cell. A division that took the floor of the answer gave a
/// second one cell before the cell of the click: `1 × 28800 / 138` is 208, and
/// the bar of the second 208 of a book of 28800 seconds holds **no** cell that
/// played.
///
/// It gives `None` for a column outside the bar, for a bar of no cell, and for
/// a media of no length: a click that names no second of the media must move no
/// playback at all (T-79).
pub fn the_second_of_a_column(the_bar: Rect, length: u32, column: u16) -> Option<u32> {
    if the_bar.width == 0 || length == 0 {
        return None;
    }

    if column < the_bar.x || column >= the_bar.right() {
        return None;
    }

    let cell = u64::from(column - the_bar.x);
    let width = u64::from(the_bar.width);
    // The answer goes up: a floor of it names the cell before this one.
    let second = (cell * u64::from(length)).div_ceil(width);

    Some(second.min(u64::from(length)) as u32)
}

/// The cells of a bar of a part of a whole, with no cell of a place.
///
/// The bar of the book and the bar of the chapter of the row 3 of the band take
/// this shape: they say a part of a whole, and the user seeks with the bar of
/// the seek alone.
pub fn a_bar_of_a_part(width: u16, done: u32, whole: u32) -> String {
    if width == 0 {
        return String::new();
    }

    let played = the_cells_that_played(width, done, whole);

    format!(
        "{}{}",
        THE_CELL_THAT_PLAYED.to_string().repeat(played),
        THE_CELL_THAT_STAYS
            .to_string()
            .repeat(usize::from(width) - played)
    )
}

/// The percent of a part of a whole, between 0 and 100.
///
/// A whole of zero gives 0: the caller says whether it holds a whole at all.
pub fn the_percent_of_a_part(done: u32, whole: u32) -> u32 {
    if whole == 0 {
        return 0;
    }

    ((u64::from(done) * 100 / u64::from(whole)) as u32).min(100)
}

/// The number of the chapter that holds a place of the media, from 0.
///
/// **A chapter holds its start and it does not hold its end**, therefore a
/// place that stands on the second of the end of a chapter belongs to the
/// chapter after it. A place outside every chapter gives `None`, and a media of
/// no chapter gives `None` too.
pub fn the_chapter_of_the_place(chapters: &[Chapter], position: f64) -> Option<usize> {
    if !position.is_finite() {
        return None;
    }

    chapters
        .iter()
        .position(|chapter| position >= chapter.start && position < chapter.end)
}

/// The place of the user in the chapter of that place, and the length of that
/// chapter, in seconds.
///
/// A media of no chapter, and a place outside every chapter, gives `None`: the
/// row of the bars then gives the whole row to the bar of the book.
pub fn the_place_in_the_chapter(chapters: &[Chapter], position: f64) -> Option<(u32, u32)> {
    let chapter = chapters.get(the_chapter_of_the_place(chapters, position)?)?;
    let length = chapter.end - chapter.start;

    if length <= 0.0 {
        return None;
    }

    Some(((position - chapter.start).max(0.0) as u32, length as u32))
}

/// The words of the chapter of the band: the number of the chapter, the number
/// of the chapters, and the name of that chapter.
///
/// **A media of no chapter says the words that the server gave** (`No chapter`
/// for a book with none), and it does not say a number that the program does
/// not have (T-91).
pub fn the_words_of_the_chapter(
    chapters: &[Chapter],
    position: f64,
    of_the_engine: &str,
) -> String {
    let Some(number) = the_chapter_of_the_place(chapters, position) else {
        return of_the_engine.to_string();
    };

    let name = chapters
        .get(number)
        .map(|chapter| chapter.title.as_str())
        .unwrap_or(of_the_engine);

    format!("Chapter {} of {}: {}", number + 1, chapters.len(), name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a_chapter(start: f64, end: f64, title: &str) -> Chapter {
        Chapter {
            start,
            end,
            title: title.to_string(),
        }
    }

    /// The band stands on the rows that it needs, and the rows that a part has
    /// no room for hold no cell of the screen. See T-322.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn the_band_stands_on_the_rows_that_it_needs() {
        assert_eq!(the_rows_of_the_band(true), 6);
        assert_eq!(the_rows_of_the_band(false), 5);

        let parts = the_parts_of_the_band(Rect::new(0, 10, 80, 4));
        assert_eq!(parts.the_words, Rect::new(0, 10, 80, 1));
        assert_eq!(parts.the_seek, Rect::new(0, 11, 80, 1));
        assert_eq!(parts.the_bars, Rect::new(0, 12, 80, 1));
        assert_eq!(parts.the_buttons, Rect::new(0, 13, 80, 1));

        // A band of three rows keeps the words, the seek, and the bars.
        let parts = the_parts_of_the_band(Rect::new(0, 10, 80, 3));
        assert_eq!(parts.the_bars, Rect::new(0, 12, 80, 1));
        assert_eq!(parts.the_buttons, Rect::default());

        // A band of two rows keeps the words and the place of the user.
        let parts = the_parts_of_the_band(Rect::new(0, 10, 80, 2));
        assert_eq!(parts.the_seek, Rect::new(0, 11, 80, 1));
        assert_eq!(parts.the_bars, Rect::default());

        let parts = the_parts_of_the_band(Rect::new(0, 10, 0, 4));
        assert_eq!(parts.the_words, Rect::default());
    }

    /// The row of the seek holds the two times at its two ends, and a row that
    /// is too narrow for a bar holds no bar at all. See T-322.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn the_row_of_the_seek_holds_the_bar_between_the_two_times() {
        // `1:04:12` and `8:00:00` take seven columns each.
        let row = Rect::new(0, 20, 40, 1);
        let parts = the_parts_of_the_seek(row, 7, 7).expect("the row holds a bar");

        assert_eq!(parts.the_time_of_the_place, Rect::new(1, 20, 7, 1));
        assert_eq!(parts.the_bar, Rect::new(10, 20, 20, 1));
        assert_eq!(parts.the_length, Rect::new(32, 20, 7, 1));
        assert!(parts.the_length.right() <= row.right());

        // 7 + 7 + 6 + 8 = 28 columns is the narrowest row that holds a bar.
        assert!(the_parts_of_the_seek(Rect::new(0, 20, 28, 1), 7, 7).is_some());
        assert!(the_parts_of_the_seek(Rect::new(0, 20, 27, 1), 7, 7).is_none());
    }

    /// The bar of the seek says the place of the user, and the column of a
    /// click of it gives the second of the media of that place. See T-322.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn the_bar_of_the_seek_and_the_click_of_it_are_opposites() {
        assert_eq!(the_bar_of_the_seek(10, 0, 100), "▒░░░░░░░░░");
        assert_eq!(the_bar_of_the_seek(10, 50, 100), "█████▒░░░░");
        assert_eq!(the_bar_of_the_seek(10, 100, 100), "██████████");

        // **A media of no length gives a bar of no place** (T-180).
        assert_eq!(the_bar_of_the_seek(4, 30, 0), "░░░░");
        assert_eq!(the_bar_of_the_seek(0, 30, 100), "");

        let bar = Rect::new(10, 20, 10, 1);
        assert_eq!(the_second_of_a_column(bar, 100, 10), Some(0));
        assert_eq!(the_second_of_a_column(bar, 100, 15), Some(50));
        assert_eq!(the_second_of_a_column(bar, 100, 19), Some(90));

        // A column outside the bar names no second of the media.
        assert_eq!(the_second_of_a_column(bar, 100, 9), None);
        assert_eq!(the_second_of_a_column(bar, 100, 20), None);
        assert_eq!(the_second_of_a_column(bar, 0, 15), None);

        // The click of a cell gives the first second of that cell, therefore
        // the bar of that second holds the same number of cells that played.
        for column in bar.x..bar.right() {
            let second = the_second_of_a_column(bar, 100, column).expect("the cell holds a second");
            let cells = the_cells_that_played(bar.width, second, 100);
            assert_eq!(cells, usize::from(column - bar.x));
        }
    }

    /// The bar of a part holds no cell of a place, and the percent of it stays
    /// between 0 and 100. See T-322.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn a_bar_of_a_part_says_a_part_of_a_whole() {
        assert_eq!(a_bar_of_a_part(8, 4, 8), "████░░░░");
        assert_eq!(a_bar_of_a_part(8, 0, 8), "░░░░░░░░");
        assert_eq!(a_bar_of_a_part(8, 8, 8), "████████");
        assert_eq!(a_bar_of_a_part(8, 9, 8), "████████");
        assert_eq!(a_bar_of_a_part(8, 4, 0), "░░░░░░░░");

        assert_eq!(the_percent_of_a_part(11, 100), 11);
        assert_eq!(the_percent_of_a_part(0, 100), 0);
        assert_eq!(the_percent_of_a_part(200, 100), 100);
        assert_eq!(the_percent_of_a_part(5, 0), 0);
    }

    /// The chapter of a place holds its start and it does not hold its end, and
    /// a media of no chapter gives the words of the engine. See T-322.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn the_chapter_of_a_place_holds_its_start() {
        let chapters = vec![
            a_chapter(0.0, 100.0, "The hours of the start"),
            a_chapter(100.0, 250.0, "The hours of the middle"),
            a_chapter(250.0, 400.0, "The hours of the end"),
        ];

        assert_eq!(the_chapter_of_the_place(&chapters, 0.0), Some(0));
        assert_eq!(the_chapter_of_the_place(&chapters, 99.9), Some(0));
        assert_eq!(the_chapter_of_the_place(&chapters, 100.0), Some(1));
        assert_eq!(the_chapter_of_the_place(&chapters, 400.0), None);
        assert_eq!(the_chapter_of_the_place(&chapters, f64::NAN), None);
        assert_eq!(the_chapter_of_the_place(&[], 10.0), None);

        assert_eq!(the_place_in_the_chapter(&chapters, 175.0), Some((75, 150)));
        assert_eq!(the_place_in_the_chapter(&chapters, 400.0), None);
        assert_eq!(the_place_in_the_chapter(&[], 10.0), None);
        assert_eq!(
            the_place_in_the_chapter(&[a_chapter(10.0, 10.0, "No time")], 10.0),
            None
        );

        assert_eq!(
            the_words_of_the_chapter(&chapters, 175.0, "The hours of the middle"),
            "Chapter 2 of 3: The hours of the middle"
        );
        // **A media of no chapter says no number that the program does not
        // have** (T-91).
        assert_eq!(
            the_words_of_the_chapter(&[], 10.0, "No chapter"),
            "No chapter"
        );
    }
}
