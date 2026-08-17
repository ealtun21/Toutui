//! The view of the chapters holds the two bars over its list. See T-330.5.
//!
//! **The maintainer read the program v0.8.158 and they gave six points**, and
//! the part 5 of the first of them is the Chapters view of
//! `docs/mockups/mockup-7.txt`: two bars over a table of the times. This round
//! takes the two bars.
//!
//! The measurement of the real program v0.8.163 inside tmux, of the library
//! `Books` of the sandbox at 160 columns and 45 rows, with `A Book Of Many
//! Hours` at 3:20:00 of its eight hours. The view held the list of the chapters
//! and **no bar at all**: the user read the number, the title, and the start of
//! each chapter, and no place of their own inside the book or inside a chapter.
//!
//! ```text
//! ─────────────The chapters of "A Book Of Many Hours" [3 items]─────────────
//!     1. The hours of the start  (00:00)
//! ➤ ▶ 2. The hours of the middle  (2:46:40)
//!     3. The hours of the end  (5:33:20)
//! ```
//!
//! The corrected program of the same harness. Two bars stand over the list: the
//! bar of the whole book, with a mark `│` at each boundary of a chapter, and the
//! bar of the chapter of the cursor under it.
//!
//! ```text
//!  Book ██████████████████████│██████░░░░░░░░░░░░░░░░░░░░░│░░░░░░░░░░░░  42%
//!  Ch 2 ███████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  21%
//!
//! ─────────────The chapters of "A Book Of Many Hours" [3 items]─────────────
//! ```
//!
//! The controls of that same run: the key `k` moved the cursor to the chapter 1,
//! which the playback passed, and the bar of the chapter said `100%`; the key
//! `G` moved it to the chapter 3, which the playback did not reach, and that bar
//! said `0%`; and the book of 70 chapters of the sandbox gave a bar of no mark,
//! because the marks of it stood beside each other.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use toutui::logic::chapters::{
    the_bar_of_the_book, the_bars_of_the_view, the_columns_of_the_name,
    the_place_in_the_chapter_of_the_cursor, THE_MARK_OF_A_BOUNDARY, THE_WIDTH_OF_THE_MARKS,
};
use toutui::player::engine::track::Chapter;

/// The width of the bar of the mockup: the panel 4 of 66 columns, less the name
/// and the percent.
const THE_WIDTH: u16 = 52;

fn a_chapter(start: f64, end: f64, title: &str) -> Chapter {
    Chapter {
        start,
        end,
        title: title.to_string(),
    }
}

/// A book of a length, with the chapters of the same length.
fn a_book_of(count: usize, length: f64) -> Vec<Chapter> {
    let of_one = length / count as f64;

    (0..count)
        .map(|number| {
            a_chapter(
                of_one * number as f64,
                of_one * (number + 1) as f64,
                &format!("Chapter {}", number + 1),
            )
        })
        .collect()
}

fn the_marks(bar: &str) -> usize {
    bar.chars()
        .filter(|one| *one == THE_MARK_OF_A_BOUNDARY)
        .count()
}

/// A book of three chapters gives a bar of two marks, at the boundaries.
///
/// **The start of the first chapter is no boundary**: it stands at the column 0,
/// which is the start of the bar itself.
#[test]
fn the_bar_of_three_chapters_holds_two_marks() {
    let bar = the_bar_of_the_book(THE_WIDTH, 0.0, 300.0, &a_book_of(3, 300.0));

    assert_eq!(bar.chars().count(), usize::from(THE_WIDTH), "{bar}");
    assert_eq!(the_marks(&bar), 2, "{bar}");

    // 52 columns of 300 seconds: the boundaries of 100 and of 200 seconds stand
    // at the columns 17 and 34.
    let columns: Vec<usize> = bar
        .chars()
        .enumerate()
        .filter(|(_, one)| *one == THE_MARK_OF_A_BOUNDARY)
        .map(|(at, _)| at)
        .collect();
    assert_eq!(columns, vec![17, 34], "{bar}");
}

/// **A book of 70 chapters in 52 columns gives a bar of no mark at all**: the
/// marks then stand beside each other with no space, and the bar of the cells
/// and the marks of the boundaries are one noise.
///
/// The measurement of the real program: a bar of 150 cells of the book of 70
/// chapters of the sandbox gave `█││█││█│█│█│██│█│██│██│█│░││░│░│░`.
#[test]
fn a_book_of_seventy_chapters_gives_a_bar_of_no_mark() {
    let all = a_book_of(70, 28800.0);

    let narrow = the_bar_of_the_book(THE_WIDTH, 4900.0, 28800.0, &all);
    assert_eq!(the_marks(&narrow), 0, "{narrow}");
    assert_eq!(narrow.chars().count(), usize::from(THE_WIDTH), "{narrow}");

    // The bar of the measurement of tmux: 150 cells of the screen of 160
    // columns. It holds more cells than the book holds chapters, and the marks
    // still stand beside each other.
    let wide = the_bar_of_the_book(150, 4900.0, 28800.0, &all);
    assert_eq!(the_marks(&wide), 0, "{wide}");

    // The control of the same test: a book of few chapters of that same width
    // keeps its marks, therefore this test reads the rule of the marks and not
    // a bar that lost them for every book.
    let few = the_bar_of_the_book(150, 4900.0, 28800.0, &a_book_of(5, 28800.0));
    assert_eq!(the_marks(&few), 4, "{few}");
}

/// A book of no chapter at all keeps the bar of the book, with no mark in it.
#[test]
fn a_book_of_no_chapter_gives_a_bar_of_no_mark() {
    let bar = the_bar_of_the_book(THE_WIDTH, 150.0, 300.0, &[]);

    assert_eq!(bar.chars().count(), usize::from(THE_WIDTH), "{bar}");
    assert_eq!(the_marks(&bar), 0, "{bar}");
    assert_eq!(bar.chars().filter(|one| *one == '█').count(), 26, "{bar}");
}

/// **The bar holds no mark under 40 columns**, which is the rule of the note of
/// `docs/mockups/mockup-7.md`.
#[test]
fn a_narrow_bar_holds_no_mark() {
    let all = a_book_of(3, 300.0);

    let narrow = the_bar_of_the_book(THE_WIDTH_OF_THE_MARKS - 1, 0.0, 300.0, &all);
    assert_eq!(the_marks(&narrow), 0, "{narrow}");

    let wide = the_bar_of_the_book(THE_WIDTH_OF_THE_MARKS, 0.0, 300.0, &all);
    assert_eq!(the_marks(&wide), 2, "{wide}");
}

/// The bar of the chapter is the chapter of the cursor, and the place of the
/// user goes inside it.
///
/// A chapter that the playback passed is whole, and a chapter that it did not
/// reach holds nothing: the measurement of tmux gave `100%` for the chapter 1
/// and `0%` for the chapter 3, with the place at the chapter 2.
#[test]
fn the_place_of_the_cursor_stays_inside_its_chapter() {
    let all = a_book_of(3, 300.0);

    assert_eq!(
        the_place_in_the_chapter_of_the_cursor(&all, Some(0), 150.0),
        Some((100, 100))
    );
    assert_eq!(
        the_place_in_the_chapter_of_the_cursor(&all, Some(1), 150.0),
        Some((50, 100))
    );
    assert_eq!(
        the_place_in_the_chapter_of_the_cursor(&all, Some(2), 150.0),
        Some((0, 100))
    );

    // A cursor that names no chapter of the media gives nothing.
    assert_eq!(
        the_place_in_the_chapter_of_the_cursor(&all, Some(9), 150.0),
        None
    );
    assert_eq!(
        the_place_in_the_chapter_of_the_cursor(&all, None, 150.0),
        None
    );
}

/// The two bars of the view: the name, the percent, and the cells of each.
#[test]
fn the_two_bars_say_the_book_and_the_chapter_of_the_cursor() {
    let all = a_book_of(3, 300.0);
    let [book, chapter] =
        the_bars_of_the_view(64, &all, 150.0, 300.0, Some(1), false).expect("the two bars");

    assert_eq!(book.the_name, "Book");
    assert_eq!(book.the_percent, Some(50));

    assert_eq!(chapter.the_name, "Ch 2");
    assert_eq!(chapter.the_percent, Some(50));

    // The two bars hold the same number of cells, therefore they stand under
    // each other.
    assert_eq!(
        book.the_cells.chars().count(),
        chapter.the_cells.chars().count()
    );

    // The cells that played are the cells of the accent of the render.
    assert!(book.the_cells_that_played > 0);
    assert!(book.the_cells_that_played < book.the_cells.chars().count());
}

/// **A media of no chapter keeps the row of the second bar**, and that row says
/// no number of a chapter and no percent at all (T-91).
#[test]
fn a_media_of_no_chapter_keeps_the_two_bars_and_says_no_percent() {
    let [book, chapter] =
        the_bars_of_the_view(64, &[], 150.0, 300.0, None, false).expect("the two bars");

    assert_eq!(book.the_percent, Some(50));
    assert_eq!(chapter.the_name, "Ch -");
    assert_eq!(chapter.the_percent, None);
    assert_eq!(chapter.the_cells_that_played, 0);
}

/// **A playback that stopped gives no bar at all**: the engine keeps the length
/// and the place of the media that played last, and the two bars of that media
/// stood over the words `No media plays now.` in the measurement of this round.
#[test]
fn a_playback_that_stopped_gives_no_bar() {
    let all = a_book_of(3, 300.0);

    assert!(the_bars_of_the_view(64, &all, 150.0, 300.0, Some(1), true).is_none());

    // A media of no length gives no bar either (T-180).
    assert!(the_bars_of_the_view(64, &all, 0.0, 0.0, Some(1), false).is_none());
}

/// A row that holds no bar of eight cells holds no bar at all, and the list of
/// the chapters then takes every row of the view.
#[test]
fn a_row_that_is_too_narrow_holds_no_bar() {
    let all = a_book_of(3, 300.0);

    // The name of a book of three chapters takes five columns, and the percent
    // takes five: a row of 17 columns gives a bar of seven cells.
    assert_eq!(the_columns_of_the_name(3), 5);
    assert!(the_bars_of_the_view(17, &all, 150.0, 300.0, Some(1), false).is_none());
    assert!(the_bars_of_the_view(18, &all, 150.0, 300.0, Some(1), false).is_some());
}

/// The render writes the two bars in the buffer of the screen.
///
/// **A gate of the pure function alone says nothing of the render** (the shape
/// of T-256): a view that makes the two bars and that draws no row of them
/// would pass every test above.
#[test]
fn the_render_writes_the_two_bars_in_the_screen() {
    let all = a_book_of(3, 300.0);
    let the_bars =
        the_bars_of_the_view(64, &all, 150.0, 300.0, Some(1), false).expect("the two bars");

    let area = Rect::new(0, 0, 65, 3);
    let mut buf = Buffer::empty(area);

    toutui::ui::the_bars_of_the_chapters::render(area, &mut buf, &the_bars, 5);

    let rows: Vec<String> = (0..area.height)
        .map(|row| {
            (0..area.width)
                .map(|column| buf[(column, row)].symbol())
                .collect()
        })
        .collect();

    assert!(rows[0].contains("Book"), "{:?}", rows);
    assert!(rows[0].contains("50%"), "{:?}", rows);
    assert!(rows[0].contains(THE_MARK_OF_A_BOUNDARY), "{:?}", rows);
    assert!(rows[1].contains("Ch 2"), "{:?}", rows);
    assert!(rows[1].contains("50%"), "{:?}", rows);

    // The two names start at the same column, and the two bars with them.
    assert_eq!(
        rows[0].chars().position(|one| one == 'B'),
        rows[1].chars().position(|one| one == 'C'),
        "{:?}",
        rows
    );

    // The third row of the area holds nothing: the row between the bars and the
    // list of the chapters.
    assert_eq!(rows[2].trim(), "", "{:?}", rows);
}

/// The two names stand in one field, therefore the two bars start at the same
/// column of the screen. The field grows with the number of the chapters.
#[test]
fn the_field_of_the_name_holds_the_longest_name() {
    assert_eq!(the_columns_of_the_name(0), 5);
    assert_eq!(the_columns_of_the_name(9), 5);
    assert_eq!(the_columns_of_the_name(70), 6);
    assert_eq!(the_columns_of_the_name(137), 7);
}
