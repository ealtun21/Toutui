//! The two views of a text of a scroll keep every word and every number of
//! their lines in a narrow terminal. See T-363.
//!
//! The view of the statistics and the view of the sessions each draw a text in
//! a `Paragraph` with **no wrap**, and the scroll of each of them counts the
//! lines that it draws. A line that is longer than the panel therefore lost
//! what did not fit, with no mark of the cut at all.
//!
//! **A cut of a number gives the user another number.** The measurement of the
//! real program v0.8.193 at 40 columns, against the sandbox:
//!
//! ```text
//! │3. A Long Test Book — Long Author  (1 │      the time is (1 h 26 min)
//! │2078 books came, and 9 authors.  892.6│      the size is 892.6 MB
//! │Today: 1 h 46 min      In total: 13 h │      the time is 13 h 33 min
//! ```
//!
//! And six sessions of one book read the same, because every title cut at the
//! same column:
//!
//! ```text
//! │    2 min 34 s  A Second Book Of Many │
//! │   12 min 59 s  A Second Book Of Many │
//! ```
//!
//! The rule of this item: **every line of these two views stands in the width
//! of its panel**, and a line that is longer takes the rows that it needs.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;

use toutui::api::me::sessions::{PlaySession, SessionPage};
use toutui::logic::message::the_columns_of;

/// The narrowest terminal that this fork measures (T-301).
const THE_NARROW_SCREEN: u16 = 40;

/// That screen, less the two columns of the border of the panel.
const THE_NARROW_PANEL: u16 = THE_NARROW_SCREEN - 2;

fn the_text_of(line: &Line<'static>) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect()
}

/// Gives the rows of the screen that a render of a view writes.
///
/// **The `lines` of a view holds the whole text, and the screen holds the
/// columns that fit**: a `Paragraph` with no wrap cuts the rest away. A test of
/// the words of a view must therefore read the cells of the screen, and not the
/// lines of the view. The first form of this test read the lines, and it passed
/// with the correction of T-363 removed.
fn the_rows_of_a_render(
    width: u16,
    height: u16,
    render: impl Fn(Rect, &mut Buffer),
) -> Vec<String> {
    let area = Rect::new(0, 0, width, height);
    let mut buffer = Buffer::empty(area);

    render(area, &mut buffer);

    // The render draws a `Block` of a border, and the border is no word of the
    // user: the row of the top and the row of the foot go away, and the column
    // of each side goes away with them.
    (0..height)
        .map(|row| {
            (0..width)
                .map(|column| buffer[(column, row)].symbol())
                .collect::<String>()
        })
        .filter(|row| !row.starts_with('┌') && !row.starts_with('└'))
        .map(|row| row.trim_matches('│').trim_end().to_string())
        .collect()
}

/// The whole text that the screen of a view shows, in one text.
fn the_text_of_a_render(width: u16, height: u16, render: impl Fn(Rect, &mut Buffer)) -> String {
    the_rows_of_a_render(width, height, render)
        .iter()
        .map(|row| row.trim().to_string())
        .collect::<Vec<String>>()
        .join(" ")
}

/// The statistics of the sandbox, of the measurement of 2026-08-17.
fn the_statistics() -> toutui::logic::stats::State {
    use toutui::api::me::listening_stats::{Author, ItemStat, ListeningStats, MediaMetadata};

    let of_a_media = |title: &str, author: &str, seconds: f64| ItemStat {
        time_listening: seconds,
        media_metadata: Some(MediaMetadata {
            title: Some(title.to_string()),
            authors: vec![Author {
                name: Some(author.to_string()),
            }],
        }),
    };

    let mut listening = ListeningStats {
        today: 6396.0,
        total_time: 48789.0,
        ..Default::default()
    };

    listening.days.insert("2026-08-16".to_string(), 4181.0);
    listening.days.insert("2026-08-17".to_string(), 6396.0);

    for (name, seconds) in [
        ("Monday", 6688.0),
        ("Tuesday", 593.0),
        ("Wednesday", 911.0),
        ("Sunday", 4181.0),
    ] {
        listening.day_of_week.insert(name.to_string(), seconds);
    }

    listening.items.insert(
        "one".to_string(),
        of_a_media("A Book Of Many Hours", "Many Hours Author", 31067.0),
    );
    listening.items.insert(
        "two".to_string(),
        of_a_media("A Long Test Book", "Long Author", 5186.0),
    );

    toutui::logic::stats::State::Ready(Box::new(toutui::logic::stats::Statistics {
        listening,
        library: None,
        library_name: "Books".to_string(),
        year: None,
        year_number: 2026,
    }))
}

/// The sessions of the sandbox, of the measurement of 2026-08-17.
fn the_sessions() -> toutui::logic::sessions_view::State {
    let a_session = |title: &str, author: &str, time: f64| PlaySession {
        id: Some(title.to_string()),
        display_title: Some(title.to_string()),
        display_author: Some(author.to_string()),
        date: Some("2026-08-17".to_string()),
        day_of_week: Some("Monday".to_string()),
        time_listening: time,
        current_time: 90.0,
        duration: 1800.0,
        media_player: None,
    };

    toutui::logic::sessions_view::State::Ready(Box::new(
        toutui::logic::sessions_view::Loaded::first(SessionPage {
            total: 190,
            num_pages: 8,
            page: 0,
            items_per_page: 25,
            sessions: vec![
                a_session("A Second Book Of Many Hours", "Many Hours Author", 154.0),
                a_session("A Second Book Of Many Hours", "Many Hours Author", 779.0),
                a_session("A Book Of Many Hours", "Many Hours Author", 340.0),
                a_session("A Long Test Book", "Long Author", 63.0),
            ],
        }),
    ))
}

/// **No line of these two views is wider than the panel that draws it.**
///
/// A `Paragraph` with no wrap cuts the columns after the width, and the user
/// then reads a text that the program did not write.
#[test]
fn no_line_of_the_two_views_is_wider_than_the_panel() {
    let statistics = the_statistics();
    let sessions = the_sessions();

    for width in 20..=160u16 {
        for (name, lines) in [
            (
                "the statistics",
                toutui::ui::stats_tui::lines(&statistics, width),
            ),
            (
                "the sessions",
                toutui::ui::sessions_tui::lines(&sessions, width),
            ),
        ] {
            for line in &lines {
                let text = the_text_of(line);
                let columns = the_columns_of(text.trim_end());

                assert!(
                    columns <= usize::from(width),
                    "a line of {name} takes {columns} columns at a width of {width}: {text:?}"
                );
            }
        }
    }
}

/// **The time of a media of the statistics stands at the end of its line**, and
/// a cut of it gives the user another number.
#[test]
fn the_narrow_statistics_keep_the_time_of_a_media() {
    let state = the_statistics();

    // The screen is high enough for every line, therefore the scroll of it
    // stands at the first line and the whole view is visible.
    let together = the_text_of_a_render(THE_NARROW_SCREEN, 120, |area, buffer| {
        toutui::ui::stats_tui::render(&state, 0, area, buffer);
    });

    // `8 h 37 min` of the first media, and `1 h 26 min` of the second one.
    for time in ["8 h 37 min", "1 h 26 min"] {
        assert!(
            together.contains(time),
            "the view lost the time {time:?} at {THE_NARROW_PANEL} columns:\n{together}"
        );
    }

    // The name of every media stays whole beside its time.
    for name in ["A Book Of Many Hours", "A Long Test Book", "Long Author"] {
        assert!(
            together.contains(name),
            "the view lost the name {name:?} at {THE_NARROW_PANEL} columns:\n{together}"
        );
    }
}

/// **Two sessions of two media must not read the same.** Six rows of the
/// measurement all said `A Second Book Of Many`, because the title cut at the
/// column of the panel.
#[test]
fn the_narrow_sessions_tell_two_media_apart() {
    let state = the_sessions();

    let together = the_text_of_a_render(THE_NARROW_SCREEN, 60, |area, buffer| {
        toutui::ui::sessions_tui::render(&state, 0, area, buffer);
    });

    for name in [
        "A Second Book Of Many Hours",
        "A Book Of Many Hours",
        "A Long Test Book",
        "Many Hours Author",
        "Long Author",
    ] {
        assert!(
            together.contains(name),
            "the view lost the name {name:?} at {THE_NARROW_PANEL} columns:\n{together}"
        );
    }
}

/// **The times of the sessions stand in a column of their own.** The field of
/// twelve columns of the time is the whitespace at the start of the line, and a
/// wrap that drops it takes that column away.
#[test]
fn the_narrow_sessions_keep_the_column_of_the_times() {
    let state = the_sessions();

    let rows = the_rows_of_a_render(THE_NARROW_SCREEN, 60, |area, buffer| {
        toutui::ui::sessions_tui::render(&state, 0, area, buffer);
    });

    let of_the_end = |time: &str| {
        let row = rows
            .iter()
            .find(|text| text.contains(time))
            .unwrap_or_else(|| panic!("no row of the screen holds the time {time:?}"));

        let at = row.find(time).expect("the row holds the time");
        the_columns_of(&row[..at]) + the_columns_of(time)
    };

    assert_eq!(
        of_the_end("2 min 34 s"),
        of_the_end("12 min 59 s"),
        "the times of two sessions end at two columns"
    );
}

/// **A line that stands in the width already takes one row.** The first form of
/// this item gave every row the width less the indent, and a bar of a day of
/// the statistics then took two rows while it stood in the screen.
#[test]
fn a_bar_of_the_statistics_takes_one_row_in_a_narrow_terminal() {
    let lines = toutui::ui::stats_tui::lines(&the_statistics(), THE_NARROW_PANEL);
    let rows: Vec<String> = lines.iter().map(the_text_of).collect();

    let of_the_days = rows
        .iter()
        .position(|text| text.starts_with("The days of the week"))
        .expect("the view holds the group of the days of the week");

    for name in ["Monday", "Tuesday", "Wednesday", "Sunday"] {
        let at = rows
            .iter()
            .skip(of_the_days)
            .position(|text| text.starts_with(name))
            .map(|at| at + of_the_days)
            .unwrap_or_else(|| panic!("the view holds no bar of {name}"));

        // The row after the bar is the bar of the day after it, and never a row
        // of a wrap of this one.
        let after = rows.get(at + 1).map(String::as_str).unwrap_or("");

        assert!(
            !after.starts_with("    "),
            "the bar of {name} took two rows at {THE_NARROW_PANEL} columns: {:?} and {after:?}",
            rows[at]
        );
    }
}
