//! The name of the library at the header of the screen keeps one line. See
//! T-314.
//!
//! **The name of a library is a text of the server, and an administrator of
//! that server gives it.** `PATCH /api/libraries/:id` of Audiobookshelf takes a
//! name that holds an end of a line, and `GET /api/libraries` then gives that
//! name to every client. `App::new` of `src/app.rs` makes
//! `📖 <the name> (<the type of the media>)` of it, and `render_header` of
//! `src/ui/tui.rs` draws that text in a `Paragraph` with **no wrap**, in the
//! area of the header. **The header holds two rows**: the row of the account
//! and of the address stands at the left of them, and the name of the program
//! and the notice stand at the right. An end of a line in the name of the
//! library therefore puts every character after it on the second row of the
//! header, at the middle, beside the address of the server.
//!
//! The measurement of 2026-08-16, of the real program v0.8.142 inside tmux
//! against the sandbox on `:13399` with the account `toutuitest`, of a terminal
//! of **80** columns and 45 rows. It needs no proxy and no build of the fault:
//! one request gave the data of the fault.
//!
//! ```bash
//! curl -X PATCH http://localhost:13399/api/libraries/b4473d74-... \
//!     -H "Authorization: Bearer $TOK" -H 'Content-Type: application/json' \
//!     -d '{"name":"Alpha\nOMEGAEND"}'
//! ```
//!
//! A `sqlite3` of `name_selected_lib` and of `id_selected_lib` gave that
//! library to the row of the account before the start (the trap 203 and the
//! trap 204), and the two rows at the top of the screen then held
//!
//! ```text
//! 👋 Connected as toutuitest          📖 Alpha                  🦜 Toutui v0.8.142
//! 🔗 localhost:13399               OMEGAEND (book)
//! ```
//!
//! The row of the address of the server holds the word `OMEGAEND` and the type
//! of the media, and no row of the screen says the name of the library.
//!
//! **The control of the same run** (the trap 206): the same library of the same
//! keys, with the name `AlphaOMEGAEND` of one line, gave
//!
//! ```text
//! 👋 Connected as toutuitest   📖 AlphaOMEGAEND (book)          🦜 Toutui v0.8.142
//! 🔗 localhost:13399
//! ```
//!
//! These tests need no network and no sandbox: the first reads the text that
//! `App::new` makes, and the second draws it with the widget of `render_header`
//! into a `Buffer` of ratatui with no terminal (T-256).

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    widgets::{Paragraph, Widget},
};

/// The width and the height of the header of the measurement.
const WIDTH: u16 = 80;
const ROWS_OF_THE_HEADER: u16 = 2;

/// The rows of the header, with the name of the library alone in it.
///
/// The widget is the widget of `render_header` of `src/ui/tui.rs`: a
/// `Paragraph` of no wrap, bold, and at the middle.
fn the_rows_of_the_name(name: &str, media_type: &str) -> Vec<String> {
    let area = Rect::new(0, 0, WIDTH, ROWS_OF_THE_HEADER);
    let mut buf = Buffer::empty(area);

    Paragraph::new(toutui::ui::keys::the_name_of_the_library(name, media_type))
        .bold()
        .centered()
        .render(area, &mut buf);

    (0..ROWS_OF_THE_HEADER)
        .map(|row| {
            (0..WIDTH)
                .map(|column| buf[(column, row)].symbol())
                .collect::<String>()
                .trim()
                .to_string()
        })
        .collect()
}

/// The name of the library that holds an end of a line stands on one line, and
/// the words of the two lines of it hold one space between them. See T-314.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_name_of_a_library_of_two_lines_gives_one_line() {
    let the_name = toutui::ui::keys::the_name_of_the_library("Alpha\nOMEGAEND", "book");

    assert_eq!(
        the_name, "📖 Alpha OMEGAEND (book)",
        "the name of the library holds more than one line: {the_name:?}"
    );

    // The end of a line of the answer of the server takes every shape of RFC
    // 5322: the program reads a `\r` and a `\r\n` in the same way.
    assert_eq!(
        toutui::ui::keys::the_name_of_the_library("Alpha\r\nOMEGAEND", "book"),
        "📖 Alpha OMEGAEND (book)"
    );

    // **The type of the media comes of the server too**, and it takes the same
    // rule.
    assert_eq!(
        toutui::ui::keys::the_name_of_the_library("Books", "bo\nok"),
        "📖 Books (bo ok)"
    );

    // The control: a name of one line keeps every character of the measurement.
    assert_eq!(
        toutui::ui::keys::the_name_of_the_library("AlphaOMEGAEND", "book"),
        "📖 AlphaOMEGAEND (book)"
    );
}

/// The name of the library takes one row of the header, and the second row of
/// the header holds no character of it. See T-314.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_name_of_a_library_takes_one_row_of_the_header() {
    let rows = the_rows_of_the_name("Alpha\nOMEGAEND", "book");

    // The whole name of the library stands on the first row of the header. The
    // sign `📖` takes two cells of the buffer, and the second of them holds no
    // character: the row therefore holds two spaces after it.
    assert!(
        rows[0].contains("Alpha OMEGAEND (book)") && rows[0].starts_with('📖'),
        "the first row of the header holds no whole name: {:?}",
        rows[0]
    );

    // **The second row of the header belongs to the address of the server and
    // to the notice.** The row of the fault held `OMEGAEND (book)` at the
    // middle of it.
    assert_eq!(
        rows[1], "",
        "the second row of the header holds a part of the name: {:?}",
        rows[1]
    );

    // The control: a name of one line gives the same two rows.
    let rows = the_rows_of_the_name("AlphaOMEGAEND", "book");
    assert!(rows[0].contains("AlphaOMEGAEND (book)"));
    assert_eq!(rows[1], "");
}
