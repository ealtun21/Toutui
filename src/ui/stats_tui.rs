//! The screen of the statistics of the user. See T-24.
//!
//! `GET /api/me/listening-stats` gives every number, and this file makes the
//! text. The function `lines` is pure: it takes the answer of the server and
//! the width of the screen, and it gives the lines. Therefore a test examines
//! the screen with no terminal.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

use crate::api::me::listening_stats::{bar, human_time, largest, last_days, top_items, week};
use crate::api::stats::{human_size, LibraryStats, TopName, YearStats};
use crate::logic::stats::State;
use crate::ui::keys::counted;

/// The number of days of the first group.
///
/// Two weeks stand on a screen of an usual height, and they show the habit of
/// the user.
pub const DAYS: usize = 14;

/// The number of media of the group of the media that the user played most.
pub const TOP: usize = 5;

/// The number of the last sessions.
pub const SESSIONS: usize = 5;

/// The number of lines of each list of the two groups of the statistics of the
/// library and of the year.
///
/// The server gives ten items and this view shows five, because the view holds
/// six groups and the user must reach the end of it.
pub const BIG: usize = 5;

/// The largest width of a bar, in columns.
const BAR_WIDTH: usize = 30;

/// The width of the name at the start of a line of a bar.
///
/// A date of the form `2026-08-10` takes ten columns, and the longest name of
/// a day, `Wednesday`, takes nine. One column more keeps a space between the
/// name and the bar. A measurement against the sandbox on 2026-08-11 showed
/// the date and the bar together, with no space.
const NAME_WIDTH: usize = 11;

/// Gives the width of the bars for the width of the screen.
///
/// A narrow terminal gives a short bar, and the time at the end of the line
/// stays visible. A bar of no column gives no bar, and the time stays.
pub fn bar_width(width: u16) -> usize {
    let width = usize::from(width);

    // The name, the two spaces, and the time take these columns.
    let taken = NAME_WIDTH + 2 + 14;

    width.saturating_sub(taken).min(BAR_WIDTH)
}

fn title(text: &str) -> Line<'static> {
    Line::from(Span::styled(
        text.to_string(),
        Style::default()
            .fg(crate::ui::theme::THE_ACCENT)
            .add_modifier(Modifier::BOLD),
    ))
}

fn quiet(text: String) -> Line<'static> {
    Line::from(Span::styled(text, crate::ui::theme::a_quiet_text()))
}

/// Makes one line of a bar: a name, the bar, and the time.
fn line_of_a_bar(name: &str, seconds: f64, most: f64, width: usize) -> Line<'static> {
    let mark = bar(seconds, most, width);
    let time = if seconds > 0.0 {
        human_time(seconds)
    } else {
        String::new()
    };

    Line::from(vec![
        Span::raw(format!("{:<width$}", name, width = NAME_WIDTH)),
        Span::styled(
            format!("{:<width$}", mark, width = width),
            Style::default().fg(crate::ui::theme::THE_ACCENT),
        ),
        Span::raw("  ".to_string()),
        Span::raw(time),
    ])
}

/// Makes every line of the screen, and every one of them stands in the width.
///
/// The function gives a line for a state that holds no answer. The user then
/// reads what the program does, and the screen is never empty.
///
/// **A line of this view holds a number at its end** (T-363): the time of a
/// media stands after the name of it, and the facts of a library stand after
/// each other. A `Paragraph` with no wrap cuts what does not fit, and a cut of
/// a number gives the user **another number** — `(1 h 26 min)` reads `(1 ` at
/// 40 columns, and `892.6 GB` reads `892.6`. Every line of this view therefore
/// takes the rows that it needs, and [`render`] counts those rows for the end
/// of the scroll.
pub fn lines(state: &State, width: u16) -> Vec<Line<'static>> {
    let all = the_lines_of_the_state(state, width);
    crate::ui::the_wrap_of_a_line::the_rows_of_the_lines(&all, usize::from(width))
}

/// Makes the lines of the view, before the wrap of them.
fn the_lines_of_the_state(state: &State, width: u16) -> Vec<Line<'static>> {
    let all = match state {
        State::Nothing => return vec![quiet("The program did not ask the server.".to_string())],
        State::Waiting => {
            return vec![quiet("The program asks the server…".to_string())];
        }
        State::Fault(text) => {
            return vec![
                Line::from(Span::styled(
                    "The server gave no statistics.".to_string(),
                    crate::ui::theme::a_text_of_a_fault(),
                )),
                quiet(text.clone()),
            ];
        }
        State::Ready(all) => all,
    };
    let (library, year) = (all.library.as_ref(), all.year.as_ref());
    let stats = &all.listening;

    let bars = bar_width(width);
    let mut out: Vec<Line<'static>> = Vec::new();

    out.push(Line::from(vec![
        Span::raw("Today: ".to_string()),
        Span::styled(
            human_time(stats.today),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw("      In total: ".to_string()),
        Span::styled(
            human_time(stats.total_time),
            Style::default().add_modifier(Modifier::BOLD),
        ),
    ]));
    out.push(Line::raw(""));

    // The days.
    let days = last_days(stats, DAYS);
    out.push(title(&format!(
        "The last {} that you played",
        counted(DAYS, "day")
    )));

    if days.is_empty() {
        out.push(quiet("You played no media.".to_string()));
    } else {
        let most = largest(&days.iter().map(|(_, value)| *value).collect::<Vec<f64>>());

        for (day, seconds) in &days {
            out.push(line_of_a_bar(day, *seconds, most, bars));
        }
    }

    out.push(Line::raw(""));

    // The days of the week.
    let week = week(stats);
    let most = largest(&week.iter().map(|(_, value)| *value).collect::<Vec<f64>>());
    out.push(title("The days of the week"));

    for (day, seconds) in &week {
        out.push(line_of_a_bar(day, *seconds, most, bars));
    }

    out.push(Line::raw(""));

    // The media.
    let top = top_items(stats, TOP);
    out.push(title("The media that you played most"));

    if top.is_empty() {
        out.push(quiet("You played no media.".to_string()));
    } else {
        for (number, item) in top.iter().enumerate() {
            let name = if item.author.is_empty() {
                item.title.clone()
            } else {
                format!("{} — {}", item.title, item.author)
            };

            out.push(Line::from(format!(
                "{}. {}  ({})",
                number + 1,
                name,
                human_time(item.seconds)
            )));
        }
    }

    out.push(Line::raw(""));

    // The sessions.
    out.push(title("The last sessions"));

    if stats.recent_sessions.is_empty() {
        out.push(quiet("The server holds no session.".to_string()));
    } else {
        for session in stats.recent_sessions.iter().take(SESSIONS) {
            let name = session.display_title.clone().unwrap_or_default();
            let day = session.date.clone().unwrap_or_default();

            out.push(Line::from(format!(
                "{}  {}  ({})",
                day,
                name,
                human_time(session.time_listening)
            )));
        }
    }

    out.push(Line::raw(""));
    lines_of_the_library(&mut out, library, &all.library_name);

    out.push(Line::raw(""));
    lines_of_the_year(&mut out, year, all.year_number);

    out
}

/// Makes a line of a list of names, with a number and a time.
fn line_of_a_name(number: usize, name: &TopName) -> Line<'static> {
    Line::from(format!(
        "{}. {}  ({})",
        number + 1,
        name.label(),
        human_time(name.time)
    ))
}

/// The group of the statistics of the library.
///
/// A library that gives no answer keeps one quiet line. The user then knows
/// that the group exists, and that the server said nothing.
fn lines_of_the_library(out: &mut Vec<Line<'static>>, stats: Option<&LibraryStats>, name: &str) {
    let heading = if name.trim().is_empty() {
        "The library".to_string()
    } else {
        format!("The library {}", name)
    };
    out.push(title(&heading));

    let Some(stats) = stats else {
        out.push(quiet(
            "The server gave no number for this library.".to_string(),
        ));
        return;
    };

    // **A library of one media said "1 items, 57 tracks, 0 authors, 0 genres".**
    // A sweep of 2026-08-12 read that line. See T-106.
    out.push(Line::from(format!(
        "{},  {},  {},  {}",
        counted(stats.total_items as usize, "item"),
        counted(stats.num_audio_tracks as usize, "track"),
        counted(stats.total_authors as usize, "author"),
        counted(stats.total_genres as usize, "genre")
    )));
    out.push(Line::from(format!(
        "{} on the disk,  {} of media",
        human_size(stats.total_size),
        human_time(stats.total_duration)
    )));

    if !stats.longest_items.is_empty() {
        out.push(Line::raw(""));
        out.push(quiet("The longest items".to_string()));
        for (number, item) in stats.longest_items.iter().take(BIG).enumerate() {
            out.push(Line::from(format!(
                "{}. {}  ({})",
                number + 1,
                item.name(),
                human_time(item.duration)
            )));
        }
    }

    if !stats.largest_items.is_empty() {
        out.push(Line::raw(""));
        out.push(quiet("The largest items".to_string()));
        for (number, item) in stats.largest_items.iter().take(BIG).enumerate() {
            out.push(Line::from(format!(
                "{}. {}  ({})",
                number + 1,
                item.name(),
                human_size(item.size)
            )));
        }
    }
}

/// The group of the statistics of the year.
fn lines_of_the_year(out: &mut Vec<Line<'static>>, stats: Option<&YearStats>, year: i32) {
    out.push(title(&format!("The year {}", year)));

    let Some(stats) = stats else {
        out.push(quiet(
            "The server gave no number for this year.".to_string(),
        ));
        return;
    };

    out.push(Line::from(format!(
        "{} of listening in {}",
        human_time(stats.total_listening_time),
        counted(stats.num_listening_sessions as usize, "session")
    )));
    out.push(Line::from(format!(
        "{} came, and {}.  {} of them on the disk",
        counted(stats.num_books_added as usize, "book"),
        counted(stats.num_authors_added as usize, "author"),
        human_size(stats.total_books_added_size)
    )));

    for (heading, list) in [
        ("The authors of the year", &stats.top_authors),
        ("The narrators of the year", &stats.top_narrators),
        ("The genres of the year", &stats.top_genres),
    ] {
        if list.is_empty() {
            continue;
        }
        out.push(Line::raw(""));
        out.push(quiet(heading.to_string()));
        for (number, name) in list.iter().take(BIG).enumerate() {
            out.push(line_of_a_name(number, name));
        }
    }
}

/// Draws the statistics, and gives the largest first line.
///
/// The caller keeps that number. The keys `j` and `k` then stop when the last
/// line is visible, and the user never moves into an empty screen.
pub fn render(state: &State, scroll: u16, area: Rect, buf: &mut Buffer) -> u16 {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Your listening time");

    let inside = block.inner(area);
    block.render(area, buf);

    let all = lines(state, inside.width);
    let count = u16::try_from(all.len()).unwrap_or(u16::MAX);

    Paragraph::new(all).scroll((scroll, 0)).render(inside, buf);

    count.saturating_sub(inside.height)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::me::listening_stats::ListeningStats;

    fn the_answer_of_the_server() -> ListeningStats {
        serde_json::from_value(serde_json::json!({
            "totalTime": 281,
            "items": {
                "9a671047": {
                    "timeListening": 276,
                    "mediaMetadata": {
                        "title": "A Long Test Book",
                        "authors": [ { "name": "Long Author" } ]
                    }
                }
            },
            "days": { "2026-08-10": 281 },
            "dayOfWeek": { "Monday": 281 },
            "today": 281,
            "recentSessions": [ {
                "displayTitle": "Multi File Test Book",
                "displayAuthor": "Test Author",
                "date": "2026-08-10",
                "timeListening": 5
            } ]
        }))
        .expect("the answer of the server must read")
    }

    /// The answer of the server, in the shape that the screen takes. The two
    /// groups of the library and of the year are absent, therefore each of them
    /// gives its quiet line.
    fn the_state_of_the_screen() -> State {
        State::Ready(Box::new(crate::logic::stats::Statistics {
            listening: the_answer_of_the_server(),
            year_number: 2026,
            ..Default::default()
        }))
    }

    fn text_of(lines: &[Line<'static>]) -> String {
        lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    #[test]
    fn the_screen_shows_every_group() {
        let state = the_state_of_the_screen();
        let text = text_of(&lines(&state, 80));

        assert!(text.contains("Today: 4 min 41 s"));
        assert!(text.contains("In total: 4 min 41 s"));
        assert!(text.contains("The last 14 days that you played"));
        assert!(text.contains("2026-08-10"));
        assert!(text.contains("The days of the week"));
        assert!(text.contains("Monday"));
        assert!(text.contains("Sunday"));
        assert!(text.contains("The media that you played most"));
        assert!(text.contains("A Long Test Book — Long Author"));
        assert!(text.contains("The last sessions"));
        assert!(text.contains("Multi File Test Book"));
    }

    /// **A library of one media said "1 items".** A sweep of 2026-08-12 chose
    /// the library of one podcast and pressed `T`, and the screen said
    /// "1 items,  57 tracks,  0 authors,  0 genres". The rule of `ui::keys`
    /// stood in the titles of the views only. See T-106 and T-85.
    #[test]
    fn the_statistics_name_one_thing_in_the_singular() {
        let one = State::Ready(Box::new(crate::logic::stats::Statistics {
            listening: the_answer_of_the_server(),
            library: Some(crate::api::stats::LibraryStats {
                total_items: 1,
                num_audio_tracks: 1,
                total_authors: 1,
                total_genres: 1,
                ..Default::default()
            }),
            library_name: "Podcasts".to_string(),
            year: Some(crate::api::stats::YearStats {
                num_listening_sessions: 1,
                num_books_added: 1,
                num_authors_added: 1,
                ..Default::default()
            }),
            year_number: 2026,
        }));

        let text = text_of(&lines(&one, 80));

        assert!(
            text.contains("1 item,  1 track,  1 author,  1 genre"),
            "the statistics of a library of one media say: {}",
            text
        );
        assert!(text.contains("in 1 session"), "{}", text);
        assert!(text.contains("1 book came, and 1 author"), "{}", text);

        // A number that is not one keeps the plural.
        let many = State::Ready(Box::new(crate::logic::stats::Statistics {
            listening: the_answer_of_the_server(),
            library: Some(crate::api::stats::LibraryStats {
                total_items: 2,
                num_audio_tracks: 0,
                total_authors: 3,
                total_genres: 4,
                ..Default::default()
            }),
            library_name: "Books".to_string(),
            year: Some(crate::api::stats::YearStats {
                num_listening_sessions: 6,
                num_books_added: 9,
                num_authors_added: 2,
                ..Default::default()
            }),
            year_number: 2026,
        }));

        let text = text_of(&lines(&many, 80));
        assert!(
            text.contains("2 items,  0 tracks,  3 authors,  4 genres"),
            "{}",
            text
        );
        assert!(text.contains("in 6 sessions"), "{}", text);
        assert!(text.contains("9 books came, and 2 authors"), "{}", text);
    }

    #[test]
    fn a_state_with_no_answer_gives_a_sentence() {
        assert!(text_of(&lines(&State::Nothing, 80)).contains("did not ask the server"));
        assert!(text_of(&lines(&State::Waiting, 80)).contains("asks the server"));

        let fault = State::Fault("the server does not answer".to_string());
        let text = text_of(&lines(&fault, 80));
        assert!(text.contains("The server gave no statistics."));
        assert!(text.contains("the server does not answer"));
    }

    #[test]
    fn an_account_with_no_session_gives_every_group_and_no_fault() {
        let empty: ListeningStats =
            serde_json::from_value(serde_json::json!({})).expect("an answer must read");
        let text = text_of(&lines(
            &State::Ready(Box::new(crate::logic::stats::Statistics {
                listening: empty,
                year_number: 2026,
                ..Default::default()
            })),
            80,
        ));

        assert!(text.contains("Today: 0 s"));
        assert!(text.contains("You played no media."));
        assert!(text.contains("The server holds no session."));
        // The seven days keep their lines.
        assert!(text.contains("Wednesday"));
    }

    /// A day of the week with no time must give no bar, and it keeps its
    /// line. A line of a day that has time must hold a bar.
    #[test]
    fn a_day_with_no_time_gives_no_bar() {
        let state = the_state_of_the_screen();
        let text = text_of(&lines(&state, 80));

        let monday = text
            .lines()
            .find(|line| line.starts_with("Monday"))
            .expect("Monday must have a line");
        let sunday = text
            .lines()
            .find(|line| line.starts_with("Sunday"))
            .expect("Sunday must have a line");

        assert!(monday.contains('█'));
        assert!(!sunday.contains('█'));
        assert!(!sunday.contains(" s"));
    }

    /// The answers of the sandbox on 2026-08-11, for the two new groups.
    fn the_state_with_the_two_groups() -> State {
        let library: crate::api::stats::LibraryStats = serde_json::from_value(serde_json::json!({
            "totalItems": 9, "totalSize": 7987553, "totalDuration": 1883,
            "numAudioTracks": 11, "totalAuthors": 4, "totalGenres": 2,
            "largestItems": [{"id":"a","title":"A Long Test Book","size":7200565}],
            "longestItems": [{"id":"a","title":"A Long Test Book","duration":1800}]
        }))
        .expect("the answer of the library must read");

        let year: crate::api::stats::YearStats = serde_json::from_value(serde_json::json!({
            "numListeningSessions": 6, "totalListeningTime": 401,
            "numBooksAdded": 9, "numAuthorsAdded": 4,
            "totalBooksAddedSize": 7987553,
            "topAuthors":   [{"name":"Long Author","time":396}],
            "topNarrators": [{"name":"A Test Narrator","time":120}],
            "topGenres":    [{"genre":"Fiction","time":120}]
        }))
        .expect("the answer of the year must read");

        State::Ready(Box::new(crate::logic::stats::Statistics {
            listening: the_answer_of_the_server(),
            library: Some(library),
            library_name: "Books".to_string(),
            year: Some(year),
            year_number: 2026,
        }))
    }

    #[test]
    fn the_screen_shows_the_group_of_the_library() {
        let text = text_of(&lines(&the_state_with_the_two_groups(), 80));

        assert!(text.contains("The library Books"), "{text}");
        assert!(text.contains("9 items,  11 tracks,  4 authors,  2 genres"));
        assert!(text.contains("7.6 MB on the disk,  31 min 23 s of media"));
        assert!(text.contains("The longest items"));
        assert!(text.contains("1. A Long Test Book  (30 min 00 s)"));
        assert!(text.contains("The largest items"));
        assert!(text.contains("1. A Long Test Book  (6.9 MB)"));
    }

    #[test]
    fn the_screen_shows_the_group_of_the_year() {
        let text = text_of(&lines(&the_state_with_the_two_groups(), 80));

        assert!(text.contains("The year 2026"), "{text}");
        assert!(text.contains("6 min 41 s of listening in 6 sessions"));
        assert!(text.contains("9 books came, and 4 authors.  7.6 MB of them on the disk"));
        assert!(text.contains("The authors of the year"));
        assert!(text.contains("1. Long Author  (6 min 36 s)"));
        assert!(text.contains("The narrators of the year"));
        assert!(text.contains("1. A Test Narrator  (2 min 00 s)"));
        // The list of the genres names its value `genre` on the server. The
        // screen must show that name, and never "No name".
        assert!(text.contains("The genres of the year"));
        assert!(text.contains("1. Fiction  (2 min 00 s)"));
        assert!(!text.contains("No name"));
    }

    /// A server that gives no number for one of the two groups must take that
    /// group away only. The user keeps the rest of the view.
    #[test]
    fn a_group_with_no_answer_keeps_one_quiet_line() {
        let text = text_of(&lines(&the_state_of_the_screen(), 80));

        assert!(text.contains("The library"));
        assert!(text.contains("The server gave no number for this library."));
        assert!(text.contains("The year 2026"));
        assert!(text.contains("The server gave no number for this year."));
        // The groups of the time of the user stay.
        assert!(text.contains("The days of the week"));
        assert!(text.contains("The last sessions"));
    }

    /// A list with no name gives no heading. An empty heading with nothing
    /// below it would say that the server lost something.
    #[test]
    fn a_list_with_no_name_gives_no_heading() {
        let year: crate::api::stats::YearStats =
            serde_json::from_str("{}").expect("an empty answer must read");
        let state = State::Ready(Box::new(crate::logic::stats::Statistics {
            listening: the_answer_of_the_server(),
            year: Some(year),
            year_number: 2026,
            ..Default::default()
        }));
        let text = text_of(&lines(&state, 80));

        assert!(text.contains("The year 2026"));
        assert!(!text.contains("The authors of the year"));
        assert!(!text.contains("The genres of the year"));
        // The library has no name here, therefore the heading is short.
        assert!(text.contains("The library\n"), "{text}");
    }

    #[test]
    fn a_narrow_screen_gives_no_bar_and_no_fault() {
        assert_eq!(bar_width(10), 0);
        assert_eq!(bar_width(0), 0);

        let state = the_state_of_the_screen();
        let text = text_of(&lines(&state, 10));

        assert!(text.contains("Monday"));
        assert!(!text.contains('█'));
    }

    #[test]
    fn a_wide_screen_stops_at_the_largest_bar() {
        assert_eq!(bar_width(500), BAR_WIDTH);
    }

    /// No line may be wider than the screen, because a wide line hides the
    /// time at its end.
    #[test]
    fn no_line_of_a_bar_is_wider_than_the_screen() {
        let state = the_state_of_the_screen();

        // A screen of fewer columns than the name, the space, and the time
        // cannot hold a line. No terminal of that width can show a book.
        for width in 27..120u16 {
            let all = lines(&state, width);

            for line in all.iter() {
                let text = line
                    .spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect::<String>();

                // A line of a bar starts with the name and it holds the bar.
                if text.starts_with("Monday") {
                    assert!(
                        text.chars().count() <= usize::from(width),
                        "the line of {} columns is too wide: {:?}",
                        width,
                        text
                    );
                }
            }
        }
    }
}
