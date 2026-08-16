//! The panel 7 of the frame, the band of the player, and the bar of the seek of
//! it. See T-322.
//!
//! **The maintainer chose the mockup 1, the panels, on 2026-08-16**, and the
//! stage 7 of that road is the band of the player.
//!
//! **The fault, of the real program v0.8.151 inside tmux**, of the Home view of
//! the library `Books` of the sandbox on a screen of 160 columns and 45 rows,
//! with the key `l` of the row `A Book Of Many Hours`, which is a book of eight
//! hours:
//!
//! ```text
//!                A Book Of Many Hours by Many Hours Author | The hours of the start
//!            ▶ 1:14:07 / 8:00:00 | Elapsed: 1:14:07 | Left: 6:45:53 (15%) | Speed: 1.00x
//!   Spc: pause/play | p/u: +/−10s | P/U: nxt/prev ch. | O/I: spd +/− | o/i: vol +/− | t: sleep | Y: quit
//! ```
//!
//! **The three rows stood in the air**, under the frame of the panels, with no
//! border, no title, and no number of a panel: no click of the user could name
//! them. **The band held no bar at all**: the place of the user in the book
//! stood in a percent of two digits, `(15%)`, on a screen of 160 columns, and
//! the place of the user in the chapter stood nowhere.
//!
//! **The correction, of the same harness**:
//!
//! ```text
//! ┌ Player ─────────────────────────────────────────────────────────────────────┐
//! │ ▶ A Book Of Many Hours  Many Hours Author  Chapter 1 of 3: The hours…  Speed 1.00x │
//! │ 1:10:40 ├████████████████████▒░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░┤ 8:00:00 │
//! │ Book    █████████░░░░░░░░░  14%  Chapter ███████████████░░░░░░░░░░░░  42%  │
//! │            Spc: pause/play | p/u: +/−10s | P/U: nxt/prev ch. | … | Y: quit  │
//! └─────────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! **A click of the bar of the seek moves the playback to that second**:
//! `docs/harness/click.sh` at the column 80 of the row 39 of that screen gave
//! the message `The playback goes to 3:56:31.`, the row of the seek said
//! `3:56:53`, and the row of the words said `Chapter 2 of 3: The hours of the
//! middle`. **A click of the band beside its bar does nothing**, and the row
//! of the message stayed empty for it.
//!
//! **The key `B` takes the row of the buttons away**, and the band then holds
//! five rows: the list of the panel 4 grew by one line in the same measurement.

use ratatui::layout::Rect;
use toutui::ui::the_band_of_the_player::{
    a_bar_of_a_part, the_bar_of_the_seek, the_cells_that_played, the_parts_of_the_band,
    the_parts_of_the_seek, the_percent_of_a_part, the_place_in_the_chapter, the_rows_of_the_band,
    the_second_of_a_column, the_words_of_the_chapter, THE_SMALLEST_BAR,
};
use toutui::ui::the_mouse::{the_target_of_a_point, TheAreasOfTheMouse, TheTarget};

/// The band holds the bar of the seek between the two times, and the two bars
/// of the book and of the chapter under it. See T-322.
///
/// **The band of the fault held no bar at all**: a screen of 160 columns said
/// the place of the user in `(15%)`.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_band_holds_the_bar_of_the_seek_between_the_two_times() {
    // The measurement of 2026-08-16: the band of a screen of 160 columns holds
    // 158 columns inside its border, and four rows.
    let inside = Rect::new(1, 37, 158, 4);
    let parts = the_parts_of_the_band(inside);

    // **No row of the band goes away, and no row holds two parts.**
    for (number, row) in [
        parts.the_words,
        parts.the_seek,
        parts.the_bars,
        parts.the_buttons,
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(row.height, 1);
        assert_eq!(row.x, inside.x);
        assert_eq!(row.width, inside.width);
        assert_eq!(row.y, inside.y + number as u16);
    }

    // `1:10:40` and `8:00:00` take seven columns each.
    let seek = the_parts_of_the_seek(parts.the_seek, 7, 7).expect("the row holds a bar");

    assert!(seek.the_bar.width >= THE_SMALLEST_BAR);
    assert!(seek.the_time_of_the_place.right() < seek.the_bar.x);
    assert!(seek.the_bar.right() < seek.the_length.x);
    assert!(seek.the_length.right() <= parts.the_seek.right());

    // The bar says the place of the user, and it holds the three cells of the
    // design.
    let bar = the_bar_of_the_seek(seek.the_bar.width, 4240, 28800);

    assert_eq!(bar.chars().count(), usize::from(seek.the_bar.width));
    assert!(bar.contains('█'), "the part that played");
    assert!(bar.contains('▒'), "the place of the user");
    assert!(bar.contains('░'), "the part that stays");

    // The two bars of the row 3 hold no cell of a place.
    let of_the_book = a_bar_of_a_part(20, 4240, 28800);

    assert_eq!(of_the_book.chars().count(), 20);
    assert!(!of_the_book.contains('▒'));
    assert_eq!(the_percent_of_a_part(4240, 28800), 14);
}

/// A click of a cell of the bar of the seek names the second of the media that
/// the cell holds. See T-322.
///
/// **The bar and the click of it are opposites**: the render writes the cells of
/// the place, and the click reads the place of a cell.
///
/// **The parts of this test stay in one function.**
#[test]
fn a_click_of_the_bar_of_the_seek_names_the_second_of_that_cell() {
    // The measurement of 2026-08-16: the bar of a screen of 160 columns stands
    // at the column 11, it holds 138 cells, and the book holds 28800 seconds.
    let bar = Rect::new(11, 39, 138, 1);

    let areas = TheAreasOfTheMouse {
        the_band_of_the_player: Rect::new(0, 37, 160, 6),
        the_bar_of_the_seek: bar,
        the_length_of_the_media: 28800,
        ..TheAreasOfTheMouse::default()
    };

    // The click of the column 79 gave `The playback goes to 3:56:31.`
    assert_eq!(
        the_target_of_a_point(&areas, true, 79, 39),
        TheTarget::TheBarOfTheSeek { the_second: 14192 }
    );

    // **A click of the band beside its bar names the band and nothing more**:
    // the band takes no focus, therefore that click does nothing.
    assert_eq!(
        the_target_of_a_point(&areas, true, 79, 41),
        TheTarget::TheBandOfThePlayer
    );
    assert_eq!(
        the_target_of_a_point(&areas, true, 5, 39),
        TheTarget::TheBandOfThePlayer
    );

    // **The band takes no `the_stack_stands`**, because it holds no focus: a
    // screen of two columns and a screen of one column each hold its bar.
    assert_eq!(
        the_target_of_a_point(&areas, false, 79, 39),
        TheTarget::TheBarOfTheSeek { the_second: 14192 }
    );

    // A frame that drew no band takes no click of it (T-316).
    let of_no_band = TheAreasOfTheMouse::default();
    assert_eq!(
        the_target_of_a_point(&of_no_band, true, 79, 39),
        TheTarget::Nothing
    );

    // **A media of no length gives no second at all** (T-180), therefore a
    // click of its bar moves no playback.
    let mut of_no_length = areas;
    of_no_length.the_length_of_the_media = 0;
    assert_eq!(
        the_target_of_a_point(&of_no_length, true, 79, 39),
        TheTarget::TheBandOfThePlayer
    );

    // Every cell of the bar names the second of its own place.
    for column in bar.x..bar.right() {
        let second = the_second_of_a_column(bar, 28800, column).expect("the cell holds a second");

        assert_eq!(
            the_cells_that_played(bar.width, second, 28800),
            usize::from(column - bar.x),
            "the click of the column {column} must name the cell of that column"
        );
    }
}

/// The band stands on the rows that it needs, and the row of the buttons goes
/// away with the key `B`. See T-322.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_band_stands_on_the_rows_that_it_needs() {
    assert_eq!(the_rows_of_the_band(true), 6);
    assert_eq!(
        the_rows_of_the_band(false),
        the_rows_of_the_band(true) - 1,
        "the row of the buttons goes to the work of the view"
    );

    // A band of three rows keeps the words, the seek, and the two bars.
    let parts = the_parts_of_the_band(Rect::new(1, 37, 158, 3));
    assert_ne!(parts.the_bars, Rect::default());
    assert_eq!(parts.the_buttons, Rect::default());
}

/// The band says the chapter of the place of the user, and a media of no
/// chapter gives the whole row of the bars to the book. See T-322.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_band_says_the_chapter_of_the_place_of_the_user() {
    // The three chapters of "A Book Of Many Hours" of the sandbox, of the
    // section 6i of `docs/TEST-SERVER.md`.
    let chapters = vec![
        toutui::player::engine::track::Chapter {
            start: 0.0,
            end: 3600.0,
            title: "The hours of the start".to_string(),
        },
        toutui::player::engine::track::Chapter {
            start: 3600.0,
            end: 18000.0,
            title: "The hours of the middle".to_string(),
        },
        toutui::player::engine::track::Chapter {
            start: 18000.0,
            end: 28800.0,
            title: "The hours of the end".to_string(),
        },
    ];

    // The click of the measurement gave 14192 seconds, and the row of the words
    // then said the second chapter.
    assert_eq!(
        the_words_of_the_chapter(&chapters, 14192.0, "The hours of the middle"),
        "Chapter 2 of 3: The hours of the middle"
    );
    assert_eq!(
        the_place_in_the_chapter(&chapters, 14192.0),
        Some((10592, 14400))
    );

    // **A media of no chapter says no number that the program does not have**
    // (T-91), and it gives no bar of a chapter at all.
    assert_eq!(
        the_words_of_the_chapter(&[], 100.0, "No chapter"),
        "No chapter"
    );
    assert_eq!(the_place_in_the_chapter(&[], 100.0), None);
}
