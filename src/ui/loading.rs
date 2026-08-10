//! The screen that the program draws while it starts. See T-40.
//!
//! The program asks the server many times before the first screen of the
//! library. A slow server therefore gave a black screen, and the user could
//! not tell a slow server from a program that stopped.
//!
//! The functions that make the text are pure. Therefore a test can examine
//! them with no terminal.

use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};
use ratatui::Frame;

/// The characters of the sign that turns. The user then sees that the program
/// works, and that it did not stop.
const TURNING: [char; 8] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠇'];

/// Gives the character of the sign that turns for a number of a frame.
pub fn turning(tick: usize) -> char {
    TURNING[tick % TURNING.len()]
}

/// Makes the line that tells the user what the program does.
///
/// `seconds` is the time since the start. The line names the time after five
/// seconds, because a user who waits wants to know how long they waited.
pub fn waiting_line(tick: usize, step: &str, seconds: u64) -> String {
    if seconds >= 5 {
        format!("{} {} — {} s", turning(tick), step, seconds)
    } else {
        format!("{} {}", turning(tick), step)
    }
}

/// Makes the advice that the screen shows when the wait is long.
///
/// The program gives no advice at the start, because a wait of one second is
/// normal.
pub fn advice(seconds: u64) -> Option<&'static str> {
    match seconds {
        0..=9 => None,
        10..=29 => Some("The server is slow. The program waits for the answer."),
        _ => Some(
            "The server does not answer. The program will show the media of \
             the disk. Press Q to stop.",
        ),
    }
}

/// Draws the screen of the start.
pub fn render(frame: &mut Frame, server: &str, tick: usize, seconds: u64) {
    let area = frame.area();
    let step = crate::utils::startup::step();

    let mut lines = vec![
        Line::from(Span::styled(
            "🦜 Toutui",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!("🔗 {}", server)),
        Line::from(""),
        Line::from(Span::styled(
            waiting_line(tick, &step, seconds),
            Style::default().fg(Color::Rgb(140, 200, 255)),
        )),
    ];

    if let Some(advice) = advice(seconds) {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            advice,
            Style::default().fg(Color::Rgb(160, 160, 160)),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Q: stop the program",
        Style::default().fg(Color::Rgb(120, 120, 120)),
    )));

    // The height of the text, and two lines for the borders.
    let height = lines.len() as u16 + 2;
    let width = area.width.min(72);

    let box_area = Rect {
        x: area.x + (area.width.saturating_sub(width)) / 2,
        y: area.y + (area.height.saturating_sub(height)) / 2,
        width,
        height: height.min(area.height),
    };

    Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(90, 90, 90))),
        )
        .render(box_area, frame.buffer_mut());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sign_turns_and_it_comes_back() {
        assert_eq!(turning(0), TURNING[0]);
        assert_eq!(turning(1), TURNING[1]);
        assert_eq!(turning(TURNING.len()), TURNING[0]);
        assert_eq!(turning(usize::MAX), TURNING[usize::MAX % TURNING.len()]);
    }

    #[test]
    fn a_short_wait_names_no_time() {
        let line = waiting_line(0, "the libraries", 2);
        assert!(line.contains("the libraries"));
        assert!(!line.contains(" s"));
    }

    #[test]
    fn a_long_wait_names_the_time() {
        let line = waiting_line(0, "the libraries", 12);
        assert!(line.contains("12 s"));
    }

    #[test]
    fn the_advice_comes_only_after_a_long_wait() {
        assert_eq!(advice(0), None);
        assert_eq!(advice(9), None);
        assert!(advice(10).is_some());
        assert!(advice(30).expect("an advice").contains("does not answer"));
    }

    /// The screen must draw inside a terminal of any size, and it must draw
    /// something that a user can read.
    #[test]
    fn the_screen_draws_in_a_small_terminal() {
        for (width, height) in [(20, 6), (40, 10), (80, 24), (200, 60)] {
            let backend = ratatui::backend::TestBackend::new(width, height);
            let mut terminal = ratatui::Terminal::new(backend).expect("a terminal");

            terminal
                .draw(|frame| render(frame, "127.0.0.1:13399", 3, 12))
                .expect("the screen must draw");

            let text: String = terminal
                .backend()
                .buffer()
                .content()
                .iter()
                .map(|cell| cell.symbol())
                .collect();

            assert!(
                text.contains("Toutui"),
                "the screen of {}x{} shows no name",
                width,
                height
            );
        }
    }
}
