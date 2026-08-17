//! A row of the band of the player that the screen cuts says that the screen
//! cut it. See T-369.
//!
//! **The data of this fault is the size of the terminal** (T-301) **and the
//! text of the server**, therefore it needs no proxy, no book of a harness, and
//! no change of the sandbox at all.
//!
//! The measurement of 2026-08-17, of the real program v0.8.199 inside tmux
//! against the sandbox on `:13399`, of the book of eight hours
//! `A Book Of Many Hours` of the library `Books`, with the row of the buttons
//! of the key `B`. `tmux resize-window` gave the band its widths, and the two
//! rows of it said:
//!
//! ```text
//! === 80 ===
//! │ ▶ A Book Of Many Hours  Many Hours Author  Chapter 2 of 3: The  Speed 1.00x │
//! │Spc: pause/play | p/u: +/−10s | P/U: nxt/prev ch. | O/I: spd +/− | o/i: vol +/│
//! === 60 ===
//! │ ▶ A Book Of Many Hours  Many Hours A  Speed 1.00x │
//! │Spc: pause/play | p/u: +/−10s | P/U: nxt/prev ch. | O/I: s│
//! === 40 ===
//! │ ▶ A Book Of Many  Speed 1.00x │
//! │Spc: pause/play | p/u: +/−10s | P/U: n│
//! ```
//!
//! **ratatui draws no mark of that cut at all.** The row 1 of a terminal of 60
//! columns said `Many Hours A` for an author of the name `Many Hours Author`,
//! and the row 4 of a terminal of 80 columns said `o/i: vol +/`, which is a key
//! and no word of its work: the keys `t: sleep` and `Y: quit` of that row stood
//! on the screen in no form at all, and the row of the keys holds 99 columns,
//! therefore every terminal under 102 columns loses its end.
//!
//! **The control of the same run** is the same band of 160 and of 120 columns,
//! which gave the whole of the two rows.
//!
//! These tests draw the real render of the band into a `Buffer` of ratatui with
//! no terminal and no screen (T-256). They stand beside
//! `a_title_that_is_longer_than_the_screen_keeps_its_start` of T-304 and
//! `a_line_that_is_longer_than_the_panel_says_that_it_was_cut` of T-368, which
//! are the same rule for the title of a panel and for a line of a list.

use ratatui::{buffer::Buffer, layout::Rect};
use toutui::ui::player_tui::{render_the_band, TheWordsOfTheBand};
use toutui::ui::the_band_of_the_player::the_rows_of_the_band;

/// The mark of a text that the screen cut.
const THE_MARK_OF_A_CUT: &str = "…";

/// The title, the author, and the chapter of the measurement.
const THE_TITLE: &str = "A Book Of Many Hours";
const THE_AUTHOR: &str = "Many Hours Author";
const THE_CHAPTER: &str = "Chapter 2 of 3: The hours of the middle";

/// The values of the band of the measurement.
fn the_words_of_the_band() -> TheWordsOfTheBand {
    TheWordsOfTheBand {
        title: THE_TITLE.to_string(),
        author: THE_AUTHOR.to_string(),
        chapter: THE_CHAPTER.to_string(),
        it_plays: true,
        position: 10000,
        length: 28800,
        the_chapter: Some((1000, 3600)),
        speed: "1.00".to_string(),
        volume: String::new(),
        notice: None,
        sleep: None,
        the_buttons_stand: true,
    }
}

/// Draws the band of the player of a width, and gives the rows **inside its
/// border**: the border takes the first column and the last one, therefore a
/// row that holds them would end with the `│` of the band and never with the
/// three points.
fn the_rows_of_the_band_of(width: u16) -> Vec<String> {
    let band = Rect::new(0, 0, width, the_rows_of_the_band(true));
    let mut buf = Buffer::empty(band);

    render_the_band(band, &mut buf, &the_words_of_the_band(), &[0, 0, 0]);

    (1..the_rows_of_the_band(true) - 1)
        .map(|row| {
            (1..width - 1)
                .map(|column| buf[(column, row)].symbol())
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect()
}

/// The columns of a text, of the crate that ratatui measures with (T-305).
fn the_columns_of(text: &str) -> usize {
    unicode_width::UnicodeWidthStr::width(text)
}

/// The row of the words of the media and the row of the keys of the player each
/// stay inside the band, and each of them says the three points when the band
/// cut it. See T-369.
///
/// **The parts of this test stay in one function.**
#[test]
fn a_row_of_the_band_that_the_screen_cuts_says_that_it_was_cut() {
    // **The control of the measurement**: a band of 160 columns holds the whole
    // of the two rows, and neither of them says the three points.
    let of_the_control = the_rows_of_the_band_of(160);
    assert!(
        of_the_control[0].contains(THE_TITLE)
            && of_the_control[0].contains(THE_AUTHOR)
            && of_the_control[0].contains(THE_CHAPTER),
        "the row of the words of the control lost a text: {:?}",
        of_the_control[0]
    );
    assert!(
        of_the_control[3].contains("t: sleep") && of_the_control[3].contains("Y: quit"),
        "the row of the keys of the control lost a key: {:?}",
        of_the_control[3]
    );
    assert!(
        !of_the_control[0].contains(THE_MARK_OF_A_CUT)
            && !of_the_control[3].contains(THE_MARK_OF_A_CUT),
        "a row of the control says the three points: {of_the_control:?}"
    );

    // **The row of the keys holds 99 columns**, therefore a band of 100 columns
    // cuts it while the row of the words of this media still stands whole: the
    // two rows hold the rule apart, and a width that cuts one of them alone
    // says so.
    let of_the_hundred = the_rows_of_the_band_of(100);
    assert!(
        !of_the_hundred[0].ends_with(THE_MARK_OF_A_CUT),
        "the row of the words of 100 columns says a cut that it does not have: {:?}",
        of_the_hundred[0]
    );
    assert!(
        of_the_hundred[3].ends_with(THE_MARK_OF_A_CUT),
        "the row of the keys of 100 columns says no cut: {:?}",
        of_the_hundred[3]
    );

    // The widths of the measurement. The row of the words of this media holds
    // 82 columns, therefore each of these bands cuts the two rows together.
    for width in [80, 60, 45, 40] {
        let rows = the_rows_of_the_band_of(width);

        // **A row that the band cut says so.** That is the fault of T-369:
        // ratatui draws the cut and no mark of it. The words of the media take
        // the columns that the settings of the playback leave, therefore the
        // mark of that row stands **before** the settings and not at the end of
        // the row; the row of the keys takes the whole of its own row.
        assert!(
            rows[0].contains(THE_MARK_OF_A_CUT),
            "the row of the words of {width} columns says no cut: {:?}",
            rows[0]
        );
        assert!(
            rows[3].ends_with(THE_MARK_OF_A_CUT),
            "the row of the keys of {width} columns says no cut: {:?}",
            rows[3]
        );

        // **A row of the band stands inside the band.** The three points take a
        // column of the row, and they take no column of the border.
        for (of_the_row, row) in rows.iter().enumerate() {
            assert!(
                the_columns_of(row) <= usize::from(width - 2),
                "the row {of_the_row} of the band of {width} columns holds {} columns: {row:?}",
                the_columns_of(row)
            );
        }

        // **The row of the words keeps its start** (the rule of T-304): the
        // title of the media is the value of that row, and the chapter is the
        // part that the user can spare.
        assert!(
            rows[0].contains(THE_TITLE),
            "the row of the words of {width} columns lost the title: {:?}",
            rows[0]
        );

        // **The settings of the playback keep their own columns at the right**,
        // therefore the words of the media take no column of them.
        assert!(
            rows[0].contains("Speed 1.00x"),
            "the row of the words of {width} columns lost the settings: {:?}",
            rows[0]
        );

        // The row of the keys keeps its start too, and the first key of it is
        // the key that the user needs the most. That row takes the centre of
        // its own row, therefore a text of one column fewer than the row keeps
        // one space of the start.
        assert!(
            rows[3].trim_start().starts_with("Spc: pause/play"),
            "the row of the keys of {width} columns lost its start: {:?}",
            rows[3]
        );
    }
}

/// The row of the seek of a band that is too narrow for a bar says the two
/// times alone, and it says the three points when the band cut them. See T-372.
///
/// The measurement of 2026-08-17, of the real program v0.8.202 inside tmux
/// against the sandbox, of the same book of eight hours at the place 1:27:51,
/// with `tmux resize-window` of each width. The row 2 of the band said:
///
/// ```text
/// === 26 ===
/// │ 1:27:51 / 8:00:00      │
/// === 20 ===
/// │ 1:27:51 / 8:00:00│
/// === 16 ===
/// │ 1:27:51 / 8:0│
/// === 12 ===
/// │ 1:27:51 /│
/// ```
///
/// **ratatui draws no mark of that cut at all.** The band of 16 columns said a
/// length of `8:0` for a book of `8:00:00`, and the band of 12 columns said no
/// length at all: the row of the words above it and the row of the keys under
/// it each said the three points at those same two widths, which is the
/// control of the same run.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_row_of_the_seek_that_the_screen_cuts_says_that_it_was_cut() {
    // The two times of this media are `2:46:40` and `8:00:00`, therefore the
    // row of the two times holds 18 columns.
    const THE_COLUMNS_OF_THE_TWO_TIMES: usize = 18;

    // **The control of the measurement**: a band of 30 columns and one of 26
    // columns hold the whole of the two times, and neither of them says the
    // three points. The first of the two holds a bar and the second holds no
    // bar at all, therefore the control reads the two arms of that row.
    for width in [30, 26, 20] {
        let of_the_seek = &the_rows_of_the_band_of(width)[1];

        assert!(
            of_the_seek.contains("8:00:00"),
            "the row of the seek of {width} columns lost the length of the media: {of_the_seek:?}"
        );
        assert!(
            !of_the_seek.contains(THE_MARK_OF_A_CUT),
            "the row of the seek of {width} columns says a cut that it does not have: \
             {of_the_seek:?}"
        );
    }

    // The two widths of the measurement that cut the two times. **A band of
    // fewer than 20 columns holds fewer than 18 columns inside its border.**
    for width in [16, 12] {
        let of_the_seek = &the_rows_of_the_band_of(width)[1];

        assert!(
            usize::from(width - 2) < THE_COLUMNS_OF_THE_TWO_TIMES,
            "the band of {width} columns holds the two times whole"
        );

        // **A row that the band cut says so.** That is the fault of T-372.
        assert!(
            of_the_seek.ends_with(THE_MARK_OF_A_CUT),
            "the row of the seek of {width} columns says no cut: {of_the_seek:?}"
        );

        // **A row of the band stands inside the band**, and it keeps its start:
        // the place of the user is the value of that row, and the length of the
        // media is the part that the user can spare.
        assert!(
            the_columns_of(of_the_seek) <= usize::from(width - 2),
            "the row of the seek of {width} columns holds {} columns: {of_the_seek:?}",
            the_columns_of(of_the_seek)
        );
        assert!(
            of_the_seek.trim_start().starts_with("2:46:40"),
            "the row of the seek of {width} columns lost the place of the user: {of_the_seek:?}"
        );
    }
}

/// A band that is too narrow for a word of a row gives no panic, and the rows
/// of it stay inside it. See T-369.
///
/// The measurement of the real program inside tmux went down to a terminal of
/// **four** columns, and the program stood at every one of them.
///
/// **The parts of this test stay in one function.**
#[test]
fn a_band_that_is_narrower_than_a_word_gives_no_panic() {
    for width in [30, 20, 12, 6, 4, 3] {
        let rows = the_rows_of_the_band_of(width);

        for (of_the_row, row) in rows.iter().enumerate() {
            assert!(
                the_columns_of(row) <= usize::from(width - 2),
                "the row {of_the_row} of the band of {width} columns holds {} columns: {row:?}",
                the_columns_of(row)
            );
        }
    }
}
