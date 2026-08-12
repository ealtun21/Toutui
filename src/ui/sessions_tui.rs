//! The screen of the sessions of the user. See T-24.
//!
//! `GET /api/me/listening-sessions` gives the sessions in pages, and this file
//! makes the text. The function [`lines`] is pure: it takes the state and the
//! width of the screen, and it gives the lines. Therefore a test examines the
//! screen with no terminal.
//!
//! The view puts the sessions of one day under the date of that day. A user
//! reads "what did I do on Monday", and not a list of 200 lines that all carry
//! the same date.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

use crate::api::me::listening_stats::human_time;
use crate::logic::sessions_view::State;
use crate::ui::keys::counted;

/// The smallest width where the view shows the part of the media.
///
/// A narrow terminal keeps the title and the time, and it drops the part.
const WIDTH_FOR_THE_PART: u16 = 60;

fn title(text: String) -> Line<'static> {
    Line::from(Span::styled(
        text,
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

/// The date of a day, with the name of the day of the week beside it.
fn heading_of_a_day(date: &str, day_of_week: Option<&str>) -> String {
    let date = if date.trim().is_empty() {
        "A session with no date"
    } else {
        date
    };
    match day_of_week {
        Some(name) if !name.trim().is_empty() => format!("{} — {}", date, name),
        _ => date.to_string(),
    }
}

/// Makes every line of the screen.
pub fn lines(state: &State, width: u16) -> Vec<Line<'static>> {
    let loaded = match state {
        State::Nothing => return vec![quiet("The program did not ask the server.".to_string())],
        State::Waiting => return vec![quiet("The program asks the server…".to_string())],
        State::Fault(text) => {
            return vec![
                Line::from(Span::styled(
                    "The server gave no session.".to_string(),
                    Style::default().fg(Color::Rgb(220, 120, 120)),
                )),
                quiet(text.clone()),
            ];
        }
        State::Ready(loaded) => loaded,
    };

    if loaded.sessions.is_empty() {
        return vec![quiet("You played no media.".to_string())];
    }

    let mut out: Vec<Line<'static>> = Vec::new();

    // **A user of one session read "1 sessions of 1".** See T-108.
    out.push(quiet(format!(
        "{} of {}",
        counted(loaded.sessions.len(), "session"),
        loaded.total
    )));

    let mut day_before: Option<String> = None;

    for session in &loaded.sessions {
        let day = session.day();

        if day_before.as_deref() != Some(day.as_str()) {
            if day_before.is_some() {
                out.push(Line::raw(""));
            }
            out.push(title(heading_of_a_day(
                &day,
                session.day_of_week.as_deref(),
            )));
            day_before = Some(day);
        }

        let author = session.author();
        let name = if author.is_empty() {
            session.title()
        } else {
            format!("{} — {}", session.title(), author)
        };

        let part = if width >= WIDTH_FOR_THE_PART && session.duration > 0.0 {
            format!("  [{:.0}% of the media]", session.fraction() * 100.0)
        } else {
            String::new()
        };

        out.push(Line::from(vec![
            Span::raw("  ".to_string()),
            Span::styled(
                format!("{:>12}", human_time(session.time_listening)),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("  {}{}", name, part)),
        ]));
    }

    if loaded.more {
        out.push(Line::raw(""));
        out.push(quiet(
            "The program reads the next sessions when you go down…".to_string(),
        ));
    }

    out
}

/// Draws the sessions, and gives the largest first line.
///
/// The caller keeps that number. The keys `j` and `k` then stop when the last
/// line is visible, and the user never moves into an empty screen.
pub fn render(state: &State, scroll: u16, area: Rect, buf: &mut Buffer) -> u16 {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("The sessions that you played");

    let inside = block.inner(area);
    block.render(area, buf);

    let all = lines(state, inside.width);
    let count = u16::try_from(all.len()).unwrap_or(u16::MAX);
    let last = count.saturating_sub(inside.height.max(1));

    Paragraph::new(all)
        .scroll((scroll.min(last), 0))
        .render(inside, buf);

    last
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::me::sessions::{PlaySession, SessionPage};
    use crate::logic::sessions_view::Loaded;

    fn a_session(title: &str, author: &str, day: &str, week: &str, time: f64) -> PlaySession {
        PlaySession {
            id: Some(title.to_string()),
            display_title: Some(title.to_string()),
            display_author: Some(author.to_string()),
            date: Some(day.to_string()),
            day_of_week: Some(week.to_string()),
            time_listening: time,
            current_time: 90.0,
            duration: 1800.0,
            media_player: None,
        }
    }

    fn the_answer_of_the_server() -> Loaded {
        Loaded::first(SessionPage {
            total: 4,
            num_pages: 1,
            page: 0,
            items_per_page: 25,
            sessions: vec![
                a_session(
                    "A Long Test Book",
                    "Long Author",
                    "2026-08-11",
                    "Tuesday",
                    120.0,
                ),
                a_session(
                    "A Long Test Book",
                    "Long Author",
                    "2026-08-10",
                    "Monday",
                    71.0,
                ),
                a_session(
                    "Multi File Test Book",
                    "Test Author",
                    "2026-08-10",
                    "Monday",
                    5.0,
                ),
                a_session(
                    "A Long Test Book",
                    "Long Author",
                    "2026-08-10",
                    "Monday",
                    69.0,
                ),
            ],
        })
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
    fn the_screen_shows_the_sessions_under_their_day() {
        let state = State::Ready(Box::new(the_answer_of_the_server()));
        let text = text_of(&lines(&state, 80));

        assert!(text.contains("4 sessions of 4"), "{text}");
        assert!(text.contains("2026-08-11 — Tuesday"));
        assert!(text.contains("2026-08-10 — Monday"));
        assert!(text.contains("A Long Test Book — Long Author"));
        assert!(text.contains("Multi File Test Book — Test Author"));
        assert!(text.contains("2 min 00 s"));
        assert!(text.contains("5% of the media"));
    }

    /// A date must have one heading, and the sessions of that date must stand
    /// under it. A heading for each session would fill the screen.
    #[test]
    fn a_day_gives_one_heading_only() {
        let state = State::Ready(Box::new(the_answer_of_the_server()));
        let text = text_of(&lines(&state, 80));

        assert_eq!(1, text.matches("2026-08-11 — Tuesday").count());
        assert_eq!(1, text.matches("2026-08-10 — Monday").count());
    }

    #[test]
    fn a_view_that_holds_a_part_of_the_sessions_says_so() {
        let mut loaded = the_answer_of_the_server();
        loaded.total = 60;
        loaded.more = true;
        let text = text_of(&lines(&State::Ready(Box::new(loaded)), 80));

        assert!(text.contains("4 sessions of 60"));
        assert!(text.contains("The program reads the next sessions when you go down…"));
    }

    #[test]
    fn a_view_that_holds_every_session_says_nothing_about_a_next_page() {
        let state = State::Ready(Box::new(the_answer_of_the_server()));
        let text = text_of(&lines(&state, 80));
        assert!(!text.contains("when you go down"));
    }

    #[test]
    fn an_account_with_no_session_gives_a_sentence_and_no_fault() {
        let empty = Loaded::first(SessionPage::default());
        let text = text_of(&lines(&State::Ready(Box::new(empty)), 80));
        assert_eq!("You played no media.", text);
    }

    #[test]
    fn a_state_with_no_answer_gives_a_sentence() {
        assert!(text_of(&lines(&State::Nothing, 80)).contains("did not ask"));
        assert!(text_of(&lines(&State::Waiting, 80)).contains("asks the server"));
        let fault = State::Fault("the server does not answer".to_string());
        let text = text_of(&lines(&fault, 80));
        assert!(text.contains("The server gave no session."));
        assert!(text.contains("the server does not answer"));
    }

    #[test]
    fn a_session_with_no_date_keeps_a_heading() {
        let mut loaded = the_answer_of_the_server();
        loaded.sessions[0].date = None;
        loaded.sessions[0].day_of_week = None;
        let text = text_of(&lines(&State::Ready(Box::new(loaded)), 80));
        assert!(text.contains("A session with no date"), "{text}");
    }

    #[test]
    fn a_narrow_screen_drops_the_part_of_the_media_and_keeps_the_title() {
        let state = State::Ready(Box::new(the_answer_of_the_server()));
        let text = text_of(&lines(&state, 40));

        assert!(text.contains("A Long Test Book"));
        assert!(!text.contains("% of the media"));
    }

    #[test]
    fn a_session_with_no_time_of_the_media_shows_no_part() {
        let mut loaded = the_answer_of_the_server();
        for session in &mut loaded.sessions {
            session.duration = 0.0;
        }
        let text = text_of(&lines(&State::Ready(Box::new(loaded)), 80));
        assert!(!text.contains("% of the media"));
    }

    #[test]
    fn a_narrow_screen_and_a_screen_of_no_width_give_no_panic() {
        let state = State::Ready(Box::new(the_answer_of_the_server()));
        for width in [0u16, 1, 2, 10, 59, 60, 200] {
            let all = lines(&state, width);
            assert!(!all.is_empty(), "the width {width} gave no line");
        }
    }
}
