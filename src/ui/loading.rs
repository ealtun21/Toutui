//! The screen that the program draws while it starts. See T-40.
//!
//! The program asks the server many times before the first screen of the
//! library. A slow server therefore gave a black screen, and the user could
//! not tell a slow server from a program that stopped.
//!
//! The functions that make the text are pure. Therefore a test can examine
//! them with no terminal.

use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Modifier, Style};
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

/// The rows of the box of the screen of the start, and the style of each of
/// them. `width` is the columns inside the border of the box.
///
/// **A text that is wider than the box goes over the rows under it** (T-371):
/// the `Paragraph` of [`render`] holds no `wrap`, therefore ratatui cut every
/// text of this box at the width of it and it wrote no mark of that cut. The
/// advice of a slow server holds 53 columns, and the box of a terminal of 40
/// columns holds 38 of them: the measurement of the real program v0.8.201 read
/// `The server is slow. The program waits ` and no word after it. A wrap keeps
/// every word, and the height of the box comes of the rows that this function
/// gives.
///
/// **The address of the server takes that same rule** (the head of
/// `crate::ui::the_row_of_the_header`): a text that the screen cuts into a
/// different address says something that is not true, and a word that is longer
/// than the box goes over the rows under it too.
pub fn the_rows_of_the_box(
    server: &str,
    step: &str,
    tick: usize,
    seconds: u64,
    width: u16,
) -> Vec<(String, Style)> {
    let plain = Style::default();
    let quiet = crate::ui::theme::a_quiet_text();

    let mut the_texts = vec![
        (
            String::from("🦜 Toutui"),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        (String::new(), plain),
        (format!("🔗 {server}"), plain),
        (String::new(), plain),
        (
            waiting_line(tick, step, seconds),
            crate::ui::theme::a_title(),
        ),
    ];

    if let Some(advice) = advice(seconds) {
        the_texts.push((String::new(), plain));
        the_texts.push((String::from(advice), quiet));
    }

    the_texts.push((String::new(), plain));
    the_texts.push((String::from("Q: stop the program"), quiet));

    // **A box of no column inside gives one row for each text**: a wrap of no
    // width holds no word at all.
    let of_the_wrap = usize::from(width.max(1));

    the_texts
        .into_iter()
        .flat_map(|(text, style)| {
            crate::logic::message::the_parts_of_a_wrap(&text, of_the_wrap)
                .into_iter()
                .map(move |row| (row, style))
        })
        .collect()
}

/// Draws the screen of the start.
pub fn render(frame: &mut Frame, server: &str, tick: usize, seconds: u64) {
    let area = frame.area();
    let step = crate::utils::startup::step();
    let width = area.width.min(72);

    let lines: Vec<Line> = the_rows_of_the_box(
        server,
        &step,
        tick,
        seconds,
        // The two columns of the border of the box.
        width.saturating_sub(2),
    )
    .into_iter()
    .map(|(text, style)| Line::from(Span::styled(text, style)))
    .collect();

    // The height of the text, and two lines for the borders.
    let height = lines.len() as u16 + 2;

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
                .border_style(crate::ui::theme::a_quiet_text()),
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

    /// A text of the box that is wider than the box keeps every word of it, in
    /// the rows under it. See T-371.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn a_text_that_is_wider_than_the_box_goes_over_more_than_one_row() {
        // The box of a terminal of 40 columns: two columns of the border, and
        // 38 columns inside it. The advice of a slow server holds 53 columns.
        let of_the_advice = advice(12).expect("an advice of a slow server");
        assert_eq!(crate::logic::message::the_columns_of(of_the_advice), 53);

        let rows = the_rows_of_the_box("127.0.0.1:13399", "the libraries", 3, 12, 38);

        // No row of the box is wider than the box.
        for (text, _) in &rows {
            assert!(
                crate::logic::message::the_columns_of(text) <= 38,
                "the row {text:?} is wider than the box"
            );
        }

        // The advice stands whole in the rows of the box, and the measurement
        // of v0.8.201 read `The server is slow. The program waits ` alone.
        let the_words: Vec<&str> = rows
            .iter()
            .map(|(text, _)| text.as_str())
            .filter(|text| !text.is_empty())
            .collect();
        assert!(
            the_words.join(" ").contains(of_the_advice),
            "the box holds no whole advice: {the_words:?}"
        );

        // The advice of a server that answers nothing holds 89 columns, and it
        // names the key of the user at its end.
        let of_the_server_away = advice(30).expect("an advice of a server that is away");
        let of_the_rows = the_rows_of_the_box("127.0.0.1:13399", "the libraries", 3, 30, 70);
        let the_text: String = of_the_rows
            .iter()
            .map(|(text, _)| text.as_str())
            .filter(|text| !text.is_empty())
            .collect::<Vec<&str>>()
            .join(" ");
        assert_eq!(
            crate::logic::message::the_columns_of(of_the_server_away),
            89
        );
        assert!(
            the_text.contains("Press Q to stop."),
            "the box of 72 columns holds no end of the advice: {the_text}"
        );

        // An address that is longer than the box keeps every letter of it: a
        // text that the screen cuts into a different address says something
        // that is not true.
        let of_a_long_address = the_rows_of_the_box(
            "http://a-server-of-a-name-that-is-longer-than-the-box.example.com:13399",
            "the libraries",
            3,
            2,
            38,
        );
        let the_address: String = of_a_long_address
            .iter()
            .map(|(text, _)| text.as_str())
            .collect();
        assert!(
            the_address.contains("example.com:13399"),
            "the box holds no end of the address: {the_address}"
        );
        for (text, _) in &of_a_long_address {
            assert!(crate::logic::message::the_columns_of(text) <= 38);
        }

        // A box of no column inside gives no arithmetic under zero and no loop
        // that never ends.
        assert!(!the_rows_of_the_box("127.0.0.1:13399", "the libraries", 3, 30, 0).is_empty());
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
