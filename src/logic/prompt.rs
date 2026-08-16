//! A question that takes a text from the user. See T-24.
//!
//! The search of the key `/` had its own loop of events. A bookmark needs a
//! name, therefore the program needs the same work a second time. This module
//! holds it one time, and the search of a media keeps its own file because it
//! does more: it changes the view and it asks the server.
//!
//! The function draws a box of three lines above the keys, and it reads the
//! keys itself. It gives the text of the user, and it gives nothing when the
//! user presses Esc.

use crate::app::App;
use crate::config::rgb_parts;
use crate::ui::text_field::{field_view, the_backend_of_a_field};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Terminal;
use std::io;

/// Gives the area of the box and the whole rows that it stands on.
///
/// **The box starts at the column 1, and it ends one column before the end of
/// the screen.** The letters of the view that stood in those two columns stay on
/// the screen: a measurement of 2026-08-11 read a "T" of the text of a view
/// beside the left border of the box, and that letter stayed after the box went
/// away. Therefore the second rectangle holds the whole rows of the box, and the
/// program writes it before the box and after it. See T-89.
///
/// **The size comes at each turn of the loop of the box**, because the terminal
/// of the user can change its size while the box stands. See T-115.
pub fn the_areas_of_the_box<W: io::Write>(
    term: &Terminal<CrosstermBackend<W>>,
) -> io::Result<(Rect, Rect)> {
    Ok(the_areas_of_a_box_of_this_size(term.size()?))
}

/// Gives the two areas of a box on a screen of one size.
///
/// The function is pure, therefore a test needs no terminal. See T-115.
pub fn the_areas_of_a_box_of_this_size(size: ratatui::layout::Size) -> (Rect, Rect) {
    let area = Rect {
        x: 1,
        y: size.height.saturating_sub(5),
        width: size.width.saturating_sub(2),
        height: 3,
    };

    let the_whole_rows = Rect {
        x: 0,
        y: area.y,
        width: size.width,
        height: area.height,
    };

    (area, the_whole_rows)
}

impl App {
    /// Asks the user for a text.
    ///
    /// `title` names the box. The function gives `None` when the user presses
    /// Esc, and it gives the text when the user presses Enter. An empty text
    /// is a text, therefore the caller decides what an empty answer means.
    pub fn ask_for_a_text(&mut self, title: &str) -> io::Result<Option<String>> {
        // **The screen of a field takes no lock of the standard output.** A
        // panic of another thread would then wait for this screen for ever.
        // See T-174.
        let mut term = Terminal::new(the_backend_of_a_field())?;

        let (bg_r, bg_g, bg_b) = rgb_parts(&self.config.colors.background_color);
        let (fg_r, fg_g, fg_b) = rgb_parts(&self.config.colors.search_bar_foreground_color);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title.to_string())
            .border_style(Style::default().fg(Color::Rgb(fg_r, fg_g, fg_b)))
            .style(Style::default().bg(Color::Rgb(bg_r, bg_g, bg_b)));

        let mut input = tui_input::Input::default();

        let mut answer: Option<String> = None;

        loop {
            // **The terminal can change its size while the box stands.** The
            // area therefore comes at each turn of this loop: a measurement of
            // 2026-08-12 made the terminal 80 by 24 while the box of the search
            // stood in a terminal of 160 by 45, and the box then drew at the row
            // 40 of a screen of 24 rows. The user saw an empty screen, and every
            // letter that they wrote went to a box that they could not see. See
            // T-115.
            let (area, the_whole_rows) = the_areas_of_the_box(&term)?;

            // The borders take one column at the left and one column at the
            // right.
            let inner_width = area.width.saturating_sub(2);
            let view = field_view(&input, inner_width, None);

            term.draw(|frame| {
                // The letters that stood in the two columns beside the box go
                // away. See T-89.
                frame.render_widget(Clear, the_whole_rows);
                frame.render_widget(
                    Block::default().style(Style::default().bg(Color::Rgb(bg_r, bg_g, bg_b))),
                    the_whole_rows,
                );

                let bar = Paragraph::new(view.text.as_str())
                    .scroll((0, view.scroll))
                    .block(block.clone());
                frame.render_widget(bar, area);
                frame.set_cursor_position((area.x + 1 + view.cursor, area.y + 1));
            })?;

            match crossterm::event::read()? {
                Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    kind: KeyEventKind::Press,
                    ..
                }) => {
                    answer = Some(input.value().to_string());
                    break;
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Esc,
                    kind: KeyEventKind::Press,
                    ..
                }) => break,
                other => {
                    use tui_input::backend::crossterm::EventHandler;
                    input.handle_event(&other);
                }
            }
        }

        // The box goes away. The next frame of the application draws the view
        // that stands below it.
        let (_, the_whole_rows) = the_areas_of_the_box(&term)?;

        term.draw(|frame| {
            frame.render_widget(Clear, the_whole_rows);
            let empty = Block::default().style(Style::default().bg(Color::Rgb(bg_r, bg_g, bg_b)));
            frame.render_widget(empty, the_whole_rows);
        })?;

        // The box wrote on the cells of the view below it, and the terminal of
        // the program knows nothing of that work: it writes the cells that
        // changed only. Therefore the loop of the program draws every cell
        // again. See T-89, T-303, and T-42 for the same answer after a refresh.
        self.the_box_of_a_field_went_away();

        Ok(answer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Size;

    /// **The box stands on the screen of now.** A measurement of 2026-08-12 made
    /// the terminal 80 by 24 while the box of the search stood in a terminal of
    /// 160 by 45: the box kept the row 40, the screen of 24 rows held nothing at
    /// all, and every letter of the user went to a box that they could not see.
    /// See T-115.
    #[test]
    fn the_box_stands_on_the_screen_of_now() {
        let (of_the_large, whole_of_the_large) =
            the_areas_of_a_box_of_this_size(Size::new(160, 45));

        assert_eq!(of_the_large.y, 40);
        assert_eq!(of_the_large.width, 158);
        assert_eq!(whole_of_the_large.width, 160);

        // The same box in a terminal that became small. The old code kept the
        // row 40 of the screen before it.
        let (of_the_small, whole_of_the_small) = the_areas_of_a_box_of_this_size(Size::new(80, 24));

        assert_eq!(of_the_small.y, 19);
        assert_eq!(of_the_small.width, 78);
        assert_eq!(whole_of_the_small.width, 80);
        assert!(of_the_small.y + of_the_small.height <= 24);

        // A screen of no room gives a box of the row 0, and it draws nothing
        // outside the screen.
        let (of_a_small_screen, _) = the_areas_of_a_box_of_this_size(Size::new(1, 1));

        assert_eq!(of_a_small_screen.y, 0);
        assert_eq!(of_a_small_screen.width, 0);
    }
}
