//! The panel of the player stands on its four rows. See T-312.
//!
//! **The data of this fault is the text of the server.** The title, the author,
//! and the chapter of a media of Audiobookshelf go straight into the
//! `Paragraph` of `render_player`, and that paragraph draws in a `Rect` of
//! **four** rows: an empty row, the row of the name of the media, the row of
//! the position, and the row of the keys of the player of the key `B`. A text
//! of an end of a line takes a row of its own, therefore every row after it
//! moves down by one and the row of the keys falls outside the panel.
//!
//! The measurement of 2026-08-16, of the real program v0.8.140 inside tmux
//! against the sandbox on `:13399`, of a terminal of 80 columns and 45 rows,
//! with `is_show_key_bindings` of the row of the account at 1. A `PATCH` of
//! `/api/items/6ba57b9a-acb5-44f9-b2b6-39ad9107b420/media` with
//! `{"metadata":{"title":"Alpha\nOMEGAEND"}}` gave the book of eight hours
//! `A Book Of Many Hours` a title with an end of a line, and the key `l` of the
//! Library view then played it. The screen held:
//!
//! ```text
//! 38                                      Alpha
//! 39              OMEGAEND by Many Hours Author | The hours of the end
//! 40    ▶ 6:59:40 / 8:00:00 | Elapsed: 6:59:40 | Left: 1:00:20 (87%) | Speed: 1.0
//! 41
//! ```
//!
//! **The control of the same run**: the same book of the same keys, with the
//! title of the server back at `A Book Of Many Hours`, gave the row of the
//! position at the line 39 and, at the line 40,
//! `Spc: pause/play | p/u: +/−10s | P/U: nxt/prev ch. | O/I: spd +/− | o/i: vo`.
//!
//! These tests draw the real render of the player into a `Buffer` of ratatui
//! with no terminal and no screen (T-256).

use ratatui::{buffer::Buffer, layout::Rect};
use toutui::ui::player_tui::render_player;

/// The screen of the measurement: 80 columns and 45 rows.
const WIDTH: u16 = 80;
const HEIGHT: u16 = 45;

/// The panel of the player stands nine rows above the end of the screen, and it
/// holds four rows. `render_player` of `src/ui/player_tui.rs` reads the same
/// two numbers.
const THE_FIRST_ROW_OF_THE_PANEL: u16 = HEIGHT - 9;
const THE_ROWS_OF_THE_PANEL: u16 = 4;

/// The eleven values of the player, with the title, the author, and the chapter
/// of the measurement. The values after them are the numbers of the position.
fn the_values_of_the_player(title: &str, author: &str, chapter: &str) -> Vec<String> {
    vec![
        title.to_string(),
        author.to_string(),
        chapter.to_string(),
        "true".to_string(),
        "6:59:40".to_string(),
        "8:00:00".to_string(),
        "6:59:40".to_string(),
        "1:00:20".to_string(),
        "87".to_string(),
        "1.00".to_string(),
        String::new(),
    ]
}

/// Draws the panel of the player of those values, and gives its four rows.
fn the_rows_of_the_panel(values: Vec<String>) -> Vec<String> {
    let area = Rect::new(0, 0, WIDTH, HEIGHT);
    let mut buf = Buffer::empty(area);

    render_player(area, &mut buf, values, vec![0, 0, 0], true, None, None);

    (THE_FIRST_ROW_OF_THE_PANEL..THE_FIRST_ROW_OF_THE_PANEL + THE_ROWS_OF_THE_PANEL)
        .map(|row| {
            (0..WIDTH)
                .map(|column| buf[(column, row)].symbol())
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect()
}

/// The words of the row of the keys of the player. The panel holds it while
/// `the_key_bindings_stand` is true.
const THE_WORDS_OF_THE_KEYS: &str = "Spc: pause/play";

/// A title of the server that holds an end of a line keeps the four rows of the
/// panel of the player, and the row of the keys stays inside it. See T-312.
///
/// **The parts of this test stay in one function.**
#[test]
fn a_title_of_an_end_of_a_line_keeps_the_rows_of_the_panel() {
    // The control of the measurement: a title of no end of a line.
    let of_the_control = the_rows_of_the_panel(the_values_of_the_player(
        "A Book Of Many Hours",
        "Many Hours Author",
        "The hours of the end",
    ));
    assert!(
        of_the_control[1].contains("A Book Of Many Hours by Many Hours Author"),
        "the row of the name of the control says: {:?}",
        of_the_control[1]
    );
    assert!(
        of_the_control[2].contains("6:59:40 / 8:00:00"),
        "the row of the position of the control says: {:?}",
        of_the_control[2]
    );
    assert!(
        of_the_control[3].contains(THE_WORDS_OF_THE_KEYS),
        "the row of the keys of the control says: {:?}",
        of_the_control[3]
    );

    // The title of the measurement, with the end of a line in it.
    let rows = the_rows_of_the_panel(the_values_of_the_player(
        "Alpha\nOMEGAEND",
        "Many Hours Author",
        "The hours of the end",
    ));

    // The whole title stands on the row of the name, and the two lines of it
    // hold one space between them.
    assert!(
        rows[1].contains("Alpha OMEGAEND by Many Hours Author"),
        "the row of the name holds no whole title: {:?}",
        rows[1]
    );

    // **No row of the panel holds the second line of the title alone.**
    assert!(
        !rows.iter().any(|row| row.trim() == "OMEGAEND"),
        "a row of the panel holds a part of the title alone: {rows:?}"
    );

    // The row of the position keeps its own row, and it did not move down.
    assert!(
        rows[2].contains("6:59:40 / 8:00:00"),
        "the row of the position says: {:?}",
        rows[2]
    );

    // **The row of the keys of the player stays inside the panel.** That row is
    // the one that the fault took away.
    assert!(
        rows[3].contains(THE_WORDS_OF_THE_KEYS),
        "the panel lost the row of the keys of the player: {rows:?}"
    );
}

/// The author and the chapter of the server take the same rule as the title,
/// and a `\r\n` and a run of the ends of the lines each take one space. See
/// T-312.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_author_and_the_chapter_take_the_rule_of_the_title() {
    for (title, author, chapter) in [
        ("A Book Of Many Hours", "An\nAuthor", "The hours of the end"),
        ("A Book Of Many Hours", "Many Hours Author", "A\r\nChapter"),
        (
            "A Book Of Many Hours",
            "Many Hours Author",
            "A\n\n\nChapter",
        ),
    ] {
        let rows = the_rows_of_the_panel(the_values_of_the_player(title, author, chapter));

        assert!(
            rows[2].contains("6:59:40 / 8:00:00"),
            "the row of the position of {chapter:?} says: {:?}",
            rows[2]
        );
        assert!(
            rows[3].contains(THE_WORDS_OF_THE_KEYS),
            "the panel of {author:?} and {chapter:?} lost the row of the keys: {rows:?}"
        );
    }
}
