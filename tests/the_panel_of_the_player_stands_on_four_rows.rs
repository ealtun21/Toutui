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
//!
//! **The band of T-322 holds the same rule**, and the rows of it are the words
//! of the media, the bar of the seek, the two bars of the book and of the
//! chapter, and the buttons: `render_the_band` gives every end of a line one
//! space, therefore the row of the buttons stays inside the band.

use ratatui::{buffer::Buffer, layout::Rect};
use toutui::ui::player_tui::{render_the_band, TheWordsOfTheBand};
use toutui::ui::the_band_of_the_player::the_rows_of_the_band;

/// The screen of the measurement: 80 columns.
const WIDTH: u16 = 80;

/// The rows of the band, inside its border. See T-322.
const THE_ROWS_OF_THE_PANEL: u16 = the_rows_of_the_band(true) - 2;

/// The values of the band, with the title, the author, and the chapter of the
/// measurement.
fn the_words_of_the_band(title: &str, author: &str, chapter: &str) -> TheWordsOfTheBand {
    TheWordsOfTheBand {
        title: title.to_string(),
        author: author.to_string(),
        chapter: chapter.to_string(),
        it_plays: true,
        position: 25180,
        length: 28800,
        the_chapter: Some((1000, 3600)),
        speed: "1.00".to_string(),
        volume: String::new(),
        notice: None,
        sleep: None,
        the_buttons_stand: true,
    }
}

/// Draws the band of the player of those values, and gives the rows inside its
/// border.
fn the_rows_of_the_panel(words: TheWordsOfTheBand) -> Vec<String> {
    let band = Rect::new(0, 0, WIDTH, the_rows_of_the_band(true));
    let mut buf = Buffer::empty(band);

    render_the_band(band, &mut buf, &words, &[0, 0, 0]);

    (1..1 + THE_ROWS_OF_THE_PANEL)
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
    let of_the_control = the_rows_of_the_panel(the_words_of_the_band(
        "A Book Of Many Hours",
        "Many Hours Author",
        "The hours of the end",
    ));
    assert!(
        of_the_control[0].contains("A Book Of Many Hours  Many Hours Author"),
        "the row of the words of the control says: {:?}",
        of_the_control[0]
    );
    assert!(
        of_the_control[1].contains("6:59:40") && of_the_control[1].contains("8:00:00"),
        "the row of the seek of the control says: {:?}",
        of_the_control[1]
    );
    assert!(
        of_the_control[3].contains(THE_WORDS_OF_THE_KEYS),
        "the row of the buttons of the control says: {:?}",
        of_the_control[3]
    );

    // The title of the measurement, with the end of a line in it.
    let rows = the_rows_of_the_panel(the_words_of_the_band(
        "Alpha\nOMEGAEND",
        "Many Hours Author",
        "The hours of the end",
    ));

    // The whole title stands on the row of the name, and the two lines of it
    // hold one space between them.
    assert!(
        rows[0].contains("Alpha OMEGAEND  Many Hours Author"),
        "the row of the words holds no whole title: {:?}",
        rows[0]
    );

    // **No row of the panel holds the second line of the title alone.**
    assert!(
        !rows.iter().any(|row| row.trim() == "OMEGAEND"),
        "a row of the panel holds a part of the title alone: {rows:?}"
    );

    // The row of the seek keeps its own row, and it did not move down.
    assert!(
        rows[1].contains("6:59:40") && rows[1].contains("8:00:00"),
        "the row of the seek says: {:?}",
        rows[1]
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
        let rows = the_rows_of_the_panel(the_words_of_the_band(title, author, chapter));

        assert!(
            rows[1].contains("6:59:40") && rows[1].contains("8:00:00"),
            "the row of the seek of {chapter:?} says: {:?}",
            rows[1]
        );
        assert!(
            rows[3].contains(THE_WORDS_OF_THE_KEYS),
            "the panel of {author:?} and {chapter:?} lost the row of the keys: {rows:?}"
        );
    }
}
