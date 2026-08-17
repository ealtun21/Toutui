//! A click of the bar of the book of the Chapters view moves the media to that
//! place. See T-333.
//!
//! **The map of the mouse of `docs/mockups/mockup-7.md` holds three lines**, and
//! the round of T-330.5 took two of them: "A click on a row — Plays that
//! chapter" and "The wheel over the table — Moves the list". The third one is
//! "A click on the bar of the book — Moves the media to that place", and **the
//! two bars of that view stood in no area of `TheAreasOfTheMouse` at all**.
//!
//! **The fault.** The measurement of the real program v0.8.166 inside tmux, of
//! the library `Books` of the sandbox at 160 columns and 45 rows, with `A Second
//! Book Of Many Hours` and its 70 chapters at 3:19:53 of its eight hours. The
//! bar of the book stands on the row 3 of the screen, and its cells stand from
//! the column 8 to the column 154:
//!
//! ```text
//!  Book  █████████████████████████████████████████░░░░░░░░░░░░░░░░░░░░░  41%
//! ```
//!
//! A click of the column 40 of that row said nothing at all, and the playback
//! went on from 3:19:53 to 3:21:22 with no move of its own.
//!
//! **The corrected program of the same harness**, with the playback in pause at
//! 5:13:34, gave the place of the cell of every click:
//!
//! | The click | The place of the media |
//! |---|---|
//! | The column 40 | 1:44:30 |
//! | The column 154, the last cell | 7:56:45 |
//! | The column 8, the first cell | 0:00 |
//! | The column 100 | 5:00:25, and the message `The playback goes to 5:00:25.` |
//!
//! **The controls of the same run**: a click of the column 3 of the row 3, which
//! is the name `Book` of the bar, moved the media nowhere, and a click of the
//! column 40 of the row 4, which is the bar of the chapter, moved it nowhere
//! either.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use toutui::logic::chapters::{the_bars_of_the_view, the_columns_of_the_name, ABarOfTheView};
use toutui::player::engine::track::Chapter;
use toutui::ui::the_bars_of_the_chapters::{render, the_area_of_the_bar_of_the_book};
use toutui::ui::the_mouse::{the_target_of_a_point, TheAreasOfTheMouse, TheTarget};

/// The length of the book of the measurement, in seconds.
const THE_EIGHT_HOURS: u32 = 28800;

/// The areas of the frame of the measurement: the bar of the book of the row 2
/// of the screen, of the column 7 to the column 153, and a book of eight hours.
///
/// The numbers count from 0, as `Rect` does, therefore the column 7 here is the
/// column 8 of the screen of `tmux`.
fn the_areas_of_the_measurement() -> TheAreasOfTheMouse {
    TheAreasOfTheMouse {
        the_bar_of_the_book: Rect::new(7, 2, 147, 1),
        the_length_of_the_media: THE_EIGHT_HOURS,
        ..TheAreasOfTheMouse::default()
    }
}

/// A click of a cell of the bar of the book names the second of that cell.
///
/// **The parts of this test stay in one function.**
#[test]
fn a_click_of_the_bar_of_the_book_names_the_second_of_that_cell() {
    let areas = the_areas_of_the_measurement();

    // The column 40 of the screen of the measurement, which gave 1:44:30.
    assert_eq!(
        the_target_of_a_point(&areas, false, 39, 2),
        TheTarget::TheBarOfTheBook { the_second: 6270 },
        "a click of the column 40 of the bar of the book of a book of eight \
         hours must name 1:44:30, which is the place that the real program gave \
         (T-333)"
    );

    // The first cell of the bar, which gave 0:00.
    assert_eq!(
        the_target_of_a_point(&areas, false, 7, 2),
        TheTarget::TheBarOfTheBook { the_second: 0 },
        "the first cell of the bar must name the start of the book (T-333)"
    );

    // The last cell of the bar, which gave 7:56:45.
    assert_eq!(
        the_target_of_a_point(&areas, false, 153, 2),
        TheTarget::TheBarOfTheBook { the_second: 28605 },
        "the last cell of the bar must name the place that the real program \
         gave, and not the end of the book: the answer of a cell goes up, and \
         the cell after the last one is the end (T-322 and T-333)"
    );

    // The column 100 of the screen, which gave the message of the measurement.
    assert_eq!(
        the_target_of_a_point(&areas, false, 99, 2),
        TheTarget::TheBarOfTheBook { the_second: 18025 },
        "a click of the column 100 must name 5:00:25, which is the place of the \
         message `The playback goes to 5:00:25.` of the measurement (T-333)"
    );
}

/// The columns of the name of the bar, the columns of the percent, and the row
/// of the bar of the chapter take no click of a place. See T-333.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_bar_of_the_chapter_and_the_words_beside_the_bar_take_no_click() {
    let areas = the_areas_of_the_measurement();

    // The column 3 of the screen of the measurement, which is the name `Book`.
    assert_eq!(
        the_target_of_a_point(&areas, false, 2, 2),
        TheTarget::Nothing,
        "the name of the bar is no cell of it: a click of it must move the media \
         nowhere, and the measurement of the real program says so (T-333)"
    );

    // The column 156 of the screen, which is the percent at the right.
    assert_eq!(
        the_target_of_a_point(&areas, false, 155, 2),
        TheTarget::Nothing,
        "the percent at the right of the bar is no cell of it (T-333)"
    );

    // The row under the bar of the book, which is the bar of the chapter.
    assert_eq!(
        the_target_of_a_point(&areas, false, 39, 3),
        TheTarget::Nothing,
        "the bar of the chapter takes no click: the map of the mouse of \
         docs/mockups/mockup-7.md names the bar of the book alone, and a bar of \
         a chapter of two minutes gives one cell of the screen to a second of \
         the media (T-333)"
    );
}

/// A frame that draws no bar of the book takes no click of one. See T-333.
///
/// **The areas of the mouse are the areas of the last frame**, therefore a bar
/// of the Chapters view that stayed in the areas would take a click of the view
/// after it.
#[test]
fn a_frame_of_no_bar_of_the_book_takes_no_click_of_one() {
    let areas = TheAreasOfTheMouse::default();

    for column in [0, 7, 39, 153] {
        assert_eq!(
            the_target_of_a_point(&areas, false, column, 2),
            TheTarget::Nothing,
            "the areas of a frame that drew no bar hold no cell of the screen \
             at all (T-333)"
        );
    }
}

/// A media of no length takes no click of a place. See T-180 and T-333.
#[test]
fn a_media_of_no_length_takes_no_click_of_a_place() {
    let areas = TheAreasOfTheMouse {
        the_length_of_the_media: 0,
        ..the_areas_of_the_measurement()
    };

    assert_eq!(
        the_target_of_a_point(&areas, false, 39, 2),
        TheTarget::Nothing,
        "a click of the bar of a media of no length must move no playback \
         (T-180): the program knows no place of that media at all"
    );
}

/// The render of the bars gives the columns of the cells of the bar of the book,
/// and no column of the name and no column of the percent. See T-333.
///
/// **A gate of the arithmetic alone says nothing of the render** (the shape of
/// T-256): this test draws the two bars into a `Buffer` and it then reads the
/// columns that hold a cell of a bar.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_render_of_the_bars_names_the_columns_of_the_cells_of_the_book() {
    // The book of the measurement: 70 chapters of eight hours.
    let chapters: Vec<Chapter> = (0..70)
        .map(|number| Chapter {
            start: f64::from(number) * 411.0,
            end: f64::from(number + 1) * 411.0,
            title: format!("Chapter {} of the second book", number + 1),
        })
        .collect();

    let the_name = the_columns_of_the_name(chapters.len());
    let area = Rect::new(0, 2, 160, 3);

    let the_bars = the_bars_of_the_view(
        area.width - 2,
        &chapters,
        11993.0,
        f64::from(THE_EIGHT_HOURS),
        Some(28),
        false,
    )
    .expect("a book of eight hours in 158 columns holds the two bars");

    let mut buf = Buffer::empty(Rect::new(0, 0, 160, 6));
    let the_bar = render(area, &mut buf, &the_bars, the_name);

    // The columns of the row of the bar of the book that hold a cell of a bar.
    let the_cells: Vec<u16> = (0..160)
        .filter(|column| matches!(buf[(*column, area.y)].symbol(), "█" | "░" | "▒" | "│"))
        .collect();

    let first = *the_cells.first().expect("the render draws the bar");
    let last = *the_cells.last().expect("the render draws the bar");

    assert_eq!(
        the_bar,
        Rect::new(first, area.y, last - first + 1, 1),
        "the area of the click must hold the cells of the bar of the book that \
         the render drew, and no column beside them: the arithmetic of the \
         render and the arithmetic of the click stay in one place (T-333)"
    );

    // The measurement of the real program of 160 columns: the cells stand from
    // the column 8 to the column 154 of the screen, which count from 1.
    assert_eq!(
        (the_bar.x, the_bar.width),
        (7, 147),
        "the bar of the book of a screen of 160 columns stands from the column \
         8 to the column 154, as the measurement of the real program gave it \
         (T-333)"
    );
}

/// A bar whose cells stand outside the area holds no cell of the screen.
#[test]
fn a_bar_that_stands_outside_its_area_takes_no_click() {
    let bar = ABarOfTheView {
        the_name: "Book".to_string(),
        the_cells: String::new(),
        the_cells_that_played: 0,
        the_percent: Some(0),
    };

    assert_eq!(
        the_area_of_the_bar_of_the_book(Rect::new(0, 2, 160, 3), &bar, 6),
        Rect::default(),
        "a bar of no cell holds no cell of the screen at all (T-333)"
    );
}

/// The render of the chapters writes the area of the bar of the book, and every
/// other frame takes it away. See T-333.
///
/// This test reads the source, as the tests of T-135, T-143, and T-164 do: the
/// areas of the mouse are the areas of the **last** frame, therefore a bar that
/// no frame takes away gives a click of the Chapters view to the view after it.
///
/// **The parts of this test stay in one function.**
#[test]
fn every_frame_takes_the_area_of_the_bar_of_the_book_away() {
    let source = include_str!("../src/ui/tui.rs");

    let of_the_road_back = source
        .find("self.the_areas_of_the_mouse.the_bar_of_the_book = Rect::default();")
        .expect(
            "the render of a frame must take the area of the bar of the book \
             away before the view of that frame draws it (T-333)",
        );
    let of_the_view = source
        .find("match self.view_state {")
        .expect("the render of a frame names the view of it");

    assert!(
        of_the_road_back < of_the_view,
        "the area of the bar of the book goes away before the view of the frame \
         draws, and the Chapters view then writes it again: a bar of the frame \
         before this one takes a click of another view (T-333)"
    );

    let at = source
        .find("fn render_chapters(")
        .expect("the render of the chapters must stand");
    // The block ends at the function after this one, and not at a number of
    // characters (the trap 209).
    let end = source[at..]
        .find("fn render_sort_filter(")
        .expect("the render of the sequence and the filter stands after it");
    let block = &source[at..at + end];

    assert!(
        block.contains("self.the_areas_of_the_mouse.the_bar_of_the_book ="),
        "the render of the chapters must write the area of the cells of the bar \
         of the book, which the render of the bars gives it back (T-333)"
    );
}

/// The click of the bar of the book takes the road of the click of the bar of
/// the seek, and the two therefore say the same words. See T-333.
#[test]
fn the_click_of_the_bar_of_the_book_takes_the_road_of_the_bar_of_the_seek() {
    let source = include_str!("../src/app.rs");

    let at = source
        .find("TheTarget::TheBarOfTheSeek { the_second }")
        .expect("the map of the mouse must hold the bar of the seek");
    let end = source[at..]
        .find("TheTarget::TheBandOfThePlayer => {}")
        .expect("the arm of the band of the player stands after this one");
    let arm = &source[at..at + end];

    assert!(
        arm.contains("TheTarget::TheBarOfTheBook { the_second }"),
        "the bar of the book is the bar of the seek of the Chapters view: the \
         two of them must do one work and say one sentence, therefore they \
         stand in one arm of the map of the mouse (T-333)"
    );

    assert!(
        arm.contains("self.the_playback_goes_to(f64::from(the_second))"),
        "a click of a cell of a bar of a place must move the playback to the \
         second of that cell, and `the_playback_goes_to` says the words of it \
         (T-322 and T-333)"
    );
}
