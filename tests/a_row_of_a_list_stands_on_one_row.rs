//! A line of a list of a view stands on one row of the panel. See T-311.
//!
//! **The data of this fault is the text of the server.** A title of a media of
//! Audiobookshelf can hold an end of a line, and every list of this program
//! draws that title in a `ListItem`. A `ListItem` of a text of a `\n` holds the
//! rows of the ends of the lines of that text, therefore one media of the
//! library took two rows of the panel, and the rules of the list then failed
//! together:
//!
//! - The mark of the line (`✓`, a percent) and the sign `➤` of the cursor stand
//!   on the first row alone, therefore the row after it reads as a media of its
//!   own that the library does not hold.
//! - The count of the bar of the scroll of T-255 reads the lines of the list
//!   and not the rows of the panel, therefore a list that stands whole in its
//!   panel by that count loses its last line, and no character of a bar says
//!   that the line is there.
//!
//! The measurement of 2026-08-16, of the real program v0.8.139 inside tmux
//! against the sandbox on `:13399`, of a terminal of 80 columns: the Library
//! view of 18 books of the library `Books` in a panel of 18 rows. A `PATCH` of
//! `/api/items/a4d8b9b2-c4a4-4e80-8ed0-07662933fa71/media` with
//! `{"metadata":{"title":"Alpha\nOMEGAEND"}}` gave the first book a title with
//! an end of a line. The screen then held `✓   Alpha` and, under it, a row
//! `OMEGAEND` of no mark, and the book `One File With No Decoder` of the last
//! line had **no row and no bar of the scroll at all**. The control of the same
//! run: the book after it, `A Book Of A Broken Epub`, whose title holds no end
//! of a line, stood on one row.
//!
//! These tests draw the real render of the list into a `Buffer` of ratatui with
//! no terminal and no screen (T-256).

use ratatui::{buffer::Buffer, layout::Rect, widgets::ListState};
use toutui::config::Colors;
use toutui::ui::the_list_of_a_view::render_the_list;

/// The panel of the measurement: 80 columns, and 18 rows of lines under the
/// header of the block.
const WIDTH: u16 = 80;
const ROWS: u16 = 18;

/// Draws a list of lines with the cursor at `selected`, and gives the rows of
/// the buffer of it, from the first line of the list to the last row.
///
/// The row 0 of the buffer is the header of the block, therefore the first line
/// of the list stands at the row 1.
fn the_rows_of_the_list(lines: &[String], selected: Option<usize>) -> Vec<String> {
    let area = Rect::new(0, 0, WIDTH, ROWS + 1);
    let mut buf = Buffer::empty(area);

    let mut state = ListState::default();
    state.select(selected);

    render_the_list(
        area,
        &mut buf,
        &Colors::default(),
        "Library",
        lines,
        &mut state,
    );

    (1..area.height)
        .map(|row| {
            (0..WIDTH)
                .map(|column| buf[(column, row)].symbol())
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect()
}

/// The 18 lines of the measurement of T-311. The line 0 holds the title of the
/// server with the end of a line in it, and the line 17 is the book that went
/// away.
fn the_lines_of_the_measurement() -> Vec<String> {
    let mut lines = vec!["✓   Alpha\nOMEGAEND".to_string()];
    lines.extend((1..17).map(|i| format!("    A Book Of The Line {i}")));
    lines.push("✓   One File With No Decoder".to_string());
    lines
}

/// A line of a list that holds an end of a line stands on one row of the panel,
/// and every other line of the list keeps its own row. See T-311.
///
/// **The parts of this test stay in one function.**
#[test]
fn a_line_of_an_end_of_a_line_takes_one_row_of_the_panel() {
    let lines = the_lines_of_the_measurement();
    assert_eq!(lines.len(), usize::from(ROWS));

    let rows = the_rows_of_the_list(&lines, Some(0));

    // The whole title of the server stands on the row of its line, and the
    // words of the two lines of it hold one space between them.
    assert!(
        rows[0].contains("Alpha OMEGAEND"),
        "the first row of the panel holds no whole title: {:?}",
        rows[0]
    );

    // **No row of the panel holds the second line of the title alone.** That
    // row of the fault named a media that the library does not hold.
    assert!(
        !rows.iter().any(|row| row.trim() == "OMEGAEND"),
        "a row of the panel names a media of no library: {rows:?}"
    );

    // **The last line of the list keeps its row.** The list holds 18 lines and
    // the panel holds 18 rows, therefore every line of it stands on the screen.
    assert!(
        rows[usize::from(ROWS) - 1].contains("One File With No Decoder"),
        "the last line of the list has no row: {:?}",
        rows[usize::from(ROWS) - 1]
    );

    // The bar of the scroll does not come: every line of the list stands in the
    // rows of the panel (T-255).
    assert!(
        !rows.iter().any(|row| row.contains('█')),
        "a list that stands whole in its panel drew a bar: {rows:?}"
    );
}

/// The control of the measurement, and the ends of the lines of every shape.
/// See T-311.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_lines_of_a_list_hold_the_rows_of_the_panel() {
    // **The control of the measurement** (the trap 206): a title of no end of a
    // line keeps every character of it, and the render changes nothing.
    let of_no_end: Vec<String> = (0..3)
        .map(|i| format!("    A Book Of A Broken Epub {i}"))
        .collect();
    let rows = the_rows_of_the_list(&of_no_end, Some(0));
    for (i, line) in of_no_end.iter().enumerate() {
        assert!(
            rows[i].contains(line.trim()),
            "the row {i} lost the text of its line: {:?}",
            rows[i]
        );
    }

    // **A list of many ends of a line keeps one line for one row.** Every one of
    // the three shapes of an end of a line takes one space.
    let of_every_end = vec![
        "one\ntwo".to_string(),
        "three\r\nfour".to_string(),
        "five\rsix".to_string(),
        "seven\n\n\neight".to_string(),
    ];
    let rows = the_rows_of_the_list(&of_every_end, Some(0));
    assert!(rows[0].contains("one two"), "{:?}", rows[0]);
    assert!(rows[1].contains("three four"), "{:?}", rows[1]);
    assert!(rows[2].contains("five six"), "{:?}", rows[2]);
    assert!(rows[3].contains("seven eight"), "{:?}", rows[3]);

    // The row after the last line of the list holds no character of a line.
    assert_eq!(rows[4], "", "a line of the list drew a row of its own");

    // **A list of the lines of the fault holds the number of the lines that it
    // has**: the rows of the panel that hold a character are the lines of the
    // list, and no line takes a row of another one.
    let of_ends = vec!["a\nb".to_string(), "c\nd".to_string(), "e\nf".to_string()];
    let rows = the_rows_of_the_list(&of_ends, Some(0));
    let with_a_character = rows.iter().filter(|row| !row.is_empty()).count();
    assert_eq!(
        with_a_character,
        of_ends.len(),
        "the rows of the panel are not the lines of the list: {rows:?}"
    );
}
