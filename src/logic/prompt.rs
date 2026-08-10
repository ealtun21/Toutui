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
use crate::ui::text_field::field_view;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Terminal;
use std::io;

impl App {
    /// Asks the user for a text.
    ///
    /// `title` names the box. The function gives `None` when the user presses
    /// Esc, and it gives the text when the user presses Enter. An empty text
    /// is a text, therefore the caller decides what an empty answer means.
    pub fn ask_for_a_text(&mut self, title: &str) -> io::Result<Option<String>> {
        let stdout = io::stdout();
        let stdout = stdout.lock();

        let backend = CrosstermBackend::new(stdout);
        let mut term = Terminal::new(backend)?;

        let (bg_r, bg_g, bg_b) = rgb_parts(&self.config.colors.background_color);
        let (fg_r, fg_g, fg_b) = rgb_parts(&self.config.colors.search_bar_foreground_color);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title.to_string())
            .border_style(Style::default().fg(Color::Rgb(fg_r, fg_g, fg_b)))
            .style(Style::default().bg(Color::Rgb(bg_r, bg_g, bg_b)));

        let mut input = tui_input::Input::default();

        let size = term.size()?;
        let area = Rect {
            x: 1,
            y: size.height.saturating_sub(5),
            width: size.width.saturating_sub(2),
            height: 3,
        };
        // The borders take one column at the left and one column at the right.
        let inner_width = area.width.saturating_sub(2);

        let mut answer: Option<String> = None;

        loop {
            let view = field_view(&input, inner_width, None);

            term.draw(|frame| {
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
        term.draw(|frame| {
            let empty = Block::default().style(Style::default().bg(Color::Rgb(bg_r, bg_g, bg_b)));
            frame.render_widget(empty, area);
        })?;

        Ok(answer)
    }
}
