//! The view of the chapters holds the table of the times. See T-330.5.
//!
//! **The maintainer read the program v0.8.158 and they gave six points**, and
//! the part 5 of the first of them is the Chapters view of
//! `docs/mockups/mockup-7.txt`. That part holds three rounds: the two bars
//! (v0.8.164), the table of the times, and the key `Enter` of a chapter. This
//! round takes the table of the times.
//!
//! **The fault.** The measurement of the real program v0.8.164 inside tmux, of
//! the library `Books` of the sandbox at 160 columns and 45 rows, with `A Second
//! Book Of Many Hours` and its 70 chapters. Each row of the list held the
//! number, the title, and the start of the chapter in parentheses, **and no
//! length at all**, and no row of a header named the values:
//!
//! ```text
//! ────────The chapters of "A Second Book Of Many Hours" [70 items]────────
//!     1. Chapter 1 of the second book  (00:00)
//!     2. Chapter 2 of the second book  (04:02)
//!     3. Chapter 3 of the second book  (08:34)
//! ```
//!
//! **The start stood in no column of the panel**: the parentheses came after the
//! title, therefore a long title took the time of that row far to the right and
//! the user could not read the times of two rows together. The user could not
//! see the length of one chapter at all.
//!
//! **The corrected program of the same harness**, at 160 columns:
//!
//! ```text
//!      #  Title                                    Start  Length
//!      1  Chapter 1 of the second book             00:00   4m02s
//!      2  Chapter 2 of the second book             04:02   4m32s
//!     11  Chapter 11 of the second book          1:02:50   9m02s
//! ➤ ▶ 25  Chapter 25 of the second book          2:40:59   4m41s
//! ```
//!
//! **The controls of the same run.** At 80 columns, `A Book Of Many Hours` of
//! three chapters of two hours gave `2:46:40` in the column `Start` and `2h46m`
//! in the column `Length`, and the first chapter gave `00:00` right-aligned
//! under the seven columns of the widest start. The key `G` moved the cursor to
//! the last chapter and **the row of the header stayed** over the list. **A
//! click of the row 8 of the screen, which is the first row under the header,
//! gave the chapter 1 and not the chapter 2**: the row of the header takes one
//! row of the panel, and the map of the mouse reads the rows that stay.
//! At 40 columns the table did not stand and **the line of today came back**,
//! with no row of a header at all.
//!
//! **The build of the fault** (the trap 147), of four edits of one line each
//! that keep every other line, made seven of the twelve tests of this file
//! fail.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::ListState;
use toutui::config::Colors;
use toutui::logic::chapters::{
    lines, the_columns_of_the_table, the_header_of_the_table, the_length_of_a_chapter, THE_LENGTH,
    THE_NUMBER, THE_START, THE_TITLE,
};
use toutui::logic::message::the_columns_of;
use toutui::player::engine::track::Chapter;

/// The columns of a line of the list of the measurement of 80 columns: the
/// panel of 80 columns, less the bar of the scroll and the sign of the cursor.
const OF_A_LINE: u16 = 77;

/// The columns of a line of the measurement of 40 columns.
const OF_A_NARROW_LINE: u16 = 37;

fn a_chapter(start: f64, end: f64, title: &str) -> Chapter {
    Chapter {
        start,
        end,
        title: title.to_string(),
    }
}

/// The book of the measurement of 80 columns: three chapters of a book of eight
/// hours.
fn a_book_of_many_hours() -> Vec<Chapter> {
    vec![
        a_chapter(0.0, 10000.0, "The hours of the start"),
        a_chapter(10000.0, 20000.0, "The hours of the middle"),
        a_chapter(20000.0, 28800.0, "The hours of the end"),
    ]
}

/// A book of `count` chapters of the same length, of a media of 30 minutes.
fn a_book_of(count: usize) -> Vec<Chapter> {
    let of_one = 1800.0 / count as f64;

    (0..count)
        .map(|index| {
            a_chapter(
                index as f64 * of_one,
                (index + 1) as f64 * of_one,
                &format!("Chapter {}", index + 1),
            )
        })
        .collect()
}

/// The length of a chapter names the second under one hour, and the minute over
/// it. **`convert_seconds` rounds to the minute**, therefore it gives `0m` for
/// every chapter of less than 30 seconds.
#[test]
fn the_length_of_a_chapter_names_the_second_and_the_minute() {
    assert_eq!(the_length_of_a_chapter(45.0), "45s");
    assert_eq!(the_length_of_a_chapter(470.0), "7m50s");
    assert_eq!(the_length_of_a_chapter(626.0), "10m26s");
    assert_eq!(the_length_of_a_chapter(3720.0), "1h02m");
    assert_eq!(the_length_of_a_chapter(10000.0), "2h46m");

    // A length that the media does not have takes the word of the table of the
    // panel 4 for a value that is absent (T-321).
    assert_eq!(the_length_of_a_chapter(0.0), "-");
    assert_eq!(the_length_of_a_chapter(-12.0), "-");
    assert_eq!(the_length_of_a_chapter(f64::NAN), "-");
}

/// The columns of the times take the width of the widest value that they hold:
/// a book of eight hours gives `7:59:12` in seven columns, and a book of 30
/// minutes gives `29:12` in five.
#[test]
fn the_columns_of_the_times_take_the_widest_value() {
    let of_many_hours = the_columns_of_the_table(OF_A_LINE, &a_book_of_many_hours());
    assert!(of_many_hours.the_table_stands());
    assert_eq!(of_many_hours.the_start, 7);

    let of_thirty_minutes = the_columns_of_the_table(OF_A_LINE, &a_book_of(3));
    assert!(of_thirty_minutes.the_table_stands());
    assert_eq!(of_thirty_minutes.the_start, 5);

    // The number of a chapter takes the digits of the count of them.
    assert_eq!(
        the_columns_of_the_table(OF_A_LINE, &a_book_of(9)).the_number,
        1
    );
    assert_eq!(
        the_columns_of_the_table(OF_A_LINE, &a_book_of(70)).the_number,
        2
    );
}

/// **A row that has no room for the columns of the table keeps the line of
/// today** (T-91): the line of today says the start of the chapter already, and
/// a text that the row cuts says nothing to the user.
#[test]
fn a_narrow_row_gives_the_line_of_today() {
    let book = a_book_of_many_hours();

    let the_columns = the_columns_of_the_table(OF_A_NARROW_LINE, &book);
    assert!(!the_columns.the_table_stands());
    assert_eq!(the_header_of_the_table(the_columns), None);

    let the_lines = lines(&book, 12000.0, OF_A_NARROW_LINE);
    assert!(
        the_lines[0].contains("1. The hours of the start  (00:00)"),
        "{:?}",
        the_lines[0]
    );

    // The control of that same width: the row of 80 columns holds the table.
    let the_lines = lines(&book, 12000.0, OF_A_LINE);
    assert!(
        !the_lines[0].contains("(00:00)"),
        "the row of 80 columns must hold the table: {:?}",
        the_lines[0]
    );
}

/// The row of the header names the four columns of the design.
#[test]
fn the_header_names_the_four_columns() {
    let the_columns = the_columns_of_the_table(OF_A_LINE, &a_book_of_many_hours());
    let the_header = the_header_of_the_table(the_columns).expect("the table stands");

    for word in [THE_NUMBER, THE_TITLE, THE_START, THE_LENGTH] {
        assert!(the_header.contains(word), "{:?} of {:?}", word, the_header);
    }
}

/// **A book of no chapter gives no row of a header**: the view then says that
/// the book holds no chapter, and a header of no row under it names columns
/// that no row holds (T-91).
#[test]
fn a_book_of_no_chapter_gives_no_header() {
    let the_columns = the_columns_of_the_table(OF_A_LINE, &[]);

    assert!(!the_columns.the_table_stands());
    assert_eq!(the_header_of_the_table(the_columns), None);
    assert!(lines(&[], 0.0, OF_A_LINE).is_empty());
}

/// The column of the start and the column of the length stand at the same
/// column of every row, and of the header over them.
#[test]
fn every_row_holds_its_times_in_the_same_columns() {
    let book = a_book_of(70);
    let the_columns = the_columns_of_the_table(OF_A_LINE, &book);
    let the_header = the_header_of_the_table(the_columns).expect("the table stands");
    let the_lines = lines(&book, 0.0, OF_A_LINE);

    // **`String::find` gives the index of a byte and not the column of the
    // screen** (the trap 245), therefore the place of a word comes of the
    // columns of the text before it.
    let the_column_of = |text: &str, word: &str| -> usize {
        let at = text
            .find(word)
            .unwrap_or_else(|| panic!("{:?} of {:?}", word, text));
        the_columns_of(&text[..at])
    };

    let of_the_header = the_column_of(&the_header, THE_LENGTH);

    for (index, line) in the_lines.iter().enumerate() {
        let length = the_length_of_a_chapter(book[index].end - book[index].start);
        let start = toutui::utils::convert_seconds::clock(book[index].start);

        // The columns are to the right, therefore the end of each value stands
        // at the same column, and the end of the header of it with them.
        assert_eq!(
            the_column_of(line, &length) + the_columns_of(&length),
            of_the_header + the_columns_of(THE_LENGTH),
            "the length of the row {} is not in line: {:?}",
            index + 1,
            line
        );

        assert!(
            the_column_of(line, &start) < the_column_of(line, &length),
            "the start of the row {} must stand before its length: {:?}",
            index + 1,
            line
        );
    }
}

/// **A title of a letter of two columns must not move the columns after it**
/// (the trap 245): `format!` with a width counts the characters of a text and
/// not the columns of the screen.
#[test]
fn a_title_of_wide_letters_keeps_the_columns_of_the_row() {
    let of_one_column = vec![
        a_chapter(0.0, 600.0, "AAAA"),
        a_chapter(600.0, 1200.0, "BBBB"),
    ];
    let of_two_columns = vec![
        a_chapter(0.0, 600.0, "あああ"),
        a_chapter(600.0, 1200.0, "BBBB"),
    ];

    let one = lines(&of_one_column, 0.0, OF_A_LINE);
    let two = lines(&of_two_columns, 0.0, OF_A_LINE);

    assert_eq!(
        the_columns_of(&one[0]),
        the_columns_of(&two[0]),
        "{:?} and {:?}",
        one[0],
        two[0]
    );
}

/// A title that is wider than its column takes the three points, and the
/// columns after it stay where they stand.
#[test]
fn a_long_title_takes_the_three_points() {
    let long = "A title of a chapter that is longer than every column of the panel of this view";
    let book = vec![a_chapter(0.0, 600.0, long)];

    let the_lines = lines(&book, 0.0, OF_A_LINE);

    assert!(the_lines[0].contains('…'), "{:?}", the_lines[0]);
    assert!(the_lines[0].ends_with("10m00s"), "{:?}", the_lines[0]);
    assert!(
        the_columns_of(&the_lines[0]) <= usize::from(OF_A_LINE),
        "{:?}",
        the_lines[0]
    );
}

/// The mark of the chapter that plays keeps its place at the start of the row of
/// the table.
#[test]
fn the_chapter_that_plays_keeps_its_mark() {
    let the_lines = lines(&a_book_of_many_hours(), 12000.0, OF_A_LINE);

    assert!(the_lines[1].starts_with('▶'), "{:?}", the_lines[1]);
    assert!(the_lines[0].starts_with("  "), "{:?}", the_lines[0]);
}

/// **A gate of the pure function alone says nothing of the render** (the shape
/// of T-256): a view that makes the row of the header and that draws no row of
/// it passes every test above. This test draws the list with its header into a
/// `Buffer`, and it holds that the words of the header stand on the screen and
/// that the lines of the list start under them.
#[test]
fn the_render_draws_the_header_over_the_lines() {
    let book = a_book_of(70);
    let the_columns = the_columns_of_the_table(OF_A_LINE, &book);
    let the_header = the_header_of_the_table(the_columns).expect("the table stands");
    let the_lines = lines(&book, 0.0, OF_A_LINE);

    let area = Rect::new(0, 0, 80, 20);
    let mut buf = Buffer::empty(area);
    let mut list_state = ListState::default();
    list_state.select(Some(0));

    let the_lines_of_the_list = toutui::ui::the_list_of_a_view::render_the_list_with_a_header(
        area,
        &mut buf,
        &Colors::default(),
        "The chapters of a book",
        &the_lines,
        Some(the_header.as_str()),
        &mut list_state,
    );

    let the_row_of = |y: u16| -> String {
        (0..area.width)
            .map(|x| buf[(x, y)].symbol())
            .collect::<String>()
    };

    // The row 0 is the border of the block with the title of the view, the row
    // 1 is the header of the columns, and the row 2 is the first line.
    let header = the_row_of(1);
    for word in [THE_NUMBER, THE_TITLE, THE_START, THE_LENGTH] {
        assert!(header.contains(word), "{:?} of the row {:?}", word, header);
    }

    assert!(the_row_of(2).contains("Chapter 1 "), "{:?}", the_row_of(2));

    // **The rows of the lines start under the header** (T-316): the map of the
    // mouse reads this area, therefore a click of the first row of it must give
    // the first chapter and not the second one.
    assert_eq!(the_lines_of_the_list.y, 2);

    // The control of this test: a list with no header of its own keeps the
    // first line at the row 1.
    let mut buf = Buffer::empty(area);
    let of_no_header = toutui::ui::the_list_of_a_view::render_the_list_with_a_header(
        area,
        &mut buf,
        &Colors::default(),
        "The chapters of a book",
        &the_lines,
        None,
        &mut list_state,
    );

    assert_eq!(of_no_header.y, 1);
}

/// The row of the header stands over the words of the rows, and not over the
/// sign of the cursor: the words of the two must agree at every column.
#[test]
fn the_words_of_the_header_stand_over_the_words_of_the_rows() {
    let book = a_book_of_many_hours();
    let the_columns = the_columns_of_the_table(OF_A_LINE, &book);
    let the_header = the_header_of_the_table(the_columns).expect("the table stands");
    let the_lines = lines(&book, 0.0, OF_A_LINE);

    let area = Rect::new(0, 0, 80, 20);
    let mut buf = Buffer::empty(area);
    let mut list_state = ListState::default();
    list_state.select(Some(0));

    toutui::ui::the_list_of_a_view::render_the_list_with_a_header(
        area,
        &mut buf,
        &Colors::default(),
        "The chapters of a book",
        &the_lines,
        Some(the_header.as_str()),
        &mut list_state,
    );

    let the_row_of = |y: u16| -> String {
        (0..area.width)
            .map(|x| buf[(x, y)].symbol())
            .collect::<String>()
    };

    let the_column_of = |text: &str, word: &str| -> usize {
        let at = text
            .find(word)
            .unwrap_or_else(|| panic!("{:?} of {:?}", word, text));
        the_columns_of(&text[..at])
    };

    let header = the_row_of(1);
    let first = the_row_of(2);

    // The end of the word `Length` and the end of the length of the first
    // chapter stand at the same column of the screen.
    assert_eq!(
        the_column_of(&header, THE_LENGTH) + the_columns_of(THE_LENGTH),
        the_column_of(&first, "2h46m") + the_columns_of("2h46m"),
        "the header {:?} and the row {:?}",
        header,
        first
    );
}

/// The line of a list of this program holds no end of a line (T-311), and the
/// row of the header holds none either.
#[test]
fn no_row_of_the_table_holds_an_end_of_a_line() {
    let book = a_book_of(70);
    let the_columns = the_columns_of_the_table(OF_A_LINE, &book);
    let the_header = the_header_of_the_table(the_columns).expect("the table stands");

    assert!(!the_header.contains('\n'));

    for line in lines(&book, 0.0, OF_A_LINE) {
        assert!(!line.contains('\n'), "{:?}", line);
        assert!(
            the_columns_of(&line) <= usize::from(OF_A_LINE),
            "the row is wider than the panel: {:?}",
            line
        );
    }
}
