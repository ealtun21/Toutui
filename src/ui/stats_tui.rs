//! The screen of the statistics of the user. See T-24.
//!
//! `GET /api/me/listening-stats` gives every number, and this file makes the
//! text. The function `lines` is pure: it takes the answer of the server and
//! the width of the screen, and it gives the lines. Therefore a test examines
//! the screen with no terminal.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

use crate::api::me::listening_stats::{bar, human_time, largest, last_days, top_items, week};
use crate::logic::stats::State;

/// The number of days of the first group.
///
/// Two weeks stand on a screen of an usual height, and they show the habit of
/// the user.
pub const DAYS: usize = 14;

/// The number of media of the group of the media that the user played most.
pub const TOP: usize = 5;

/// The number of the last sessions.
pub const SESSIONS: usize = 5;

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
            .fg(Color::Rgb(120, 190, 255))
            .add_modifier(Modifier::BOLD),
    ))
}

fn quiet(text: String) -> Line<'static> {
    Line::from(Span::styled(
        text,
        Style::default().fg(Color::Rgb(150, 150, 150)),
    ))
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
            Style::default().fg(Color::Rgb(120, 190, 255)),
        ),
        Span::raw("  ".to_string()),
        Span::raw(time),
    ])
}

/// Makes every line of the screen.
///
/// The function gives a line for a state that holds no answer. The user then
/// reads what the program does, and the screen is never empty.
pub fn lines(state: &State, width: u16) -> Vec<Line<'static>> {
    let stats = match state {
        State::Nothing => return vec![quiet("The program did not ask the server.".to_string())],
        State::Waiting => {
            return vec![quiet("The program asks the server…".to_string())];
        }
        State::Fault(text) => {
            return vec![
                Line::from(Span::styled(
                    "The server gave no statistics.".to_string(),
                    Style::default().fg(Color::Rgb(220, 120, 120)),
                )),
                quiet(text.clone()),
            ];
        }
        State::Ready(stats) => stats,
    };

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
    out.push(title(&format!("The last {} days that you played", DAYS)));

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

    out
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
        let state = State::Ready(Box::new(the_answer_of_the_server()));
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
        let text = text_of(&lines(&State::Ready(Box::new(empty)), 80));

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
        let state = State::Ready(Box::new(the_answer_of_the_server()));
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

    #[test]
    fn a_narrow_screen_gives_no_bar_and_no_fault() {
        assert_eq!(bar_width(10), 0);
        assert_eq!(bar_width(0), 0);

        let state = State::Ready(Box::new(the_answer_of_the_server()));
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
        let state = State::Ready(Box::new(the_answer_of_the_server()));

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
