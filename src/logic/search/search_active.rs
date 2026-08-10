use crate::app::App;
use crate::app::AppView;
use crate::config::rgb_parts;
use crate::ui::text_field::field_view;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::backend::CrosstermBackend;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Terminal;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
};
use std::io;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

impl App {
    pub fn search_active(&mut self) -> io::Result<String> {
        let stdout = io::stdout();
        let stdout = stdout.lock();

        let backend = CrosstermBackend::new(stdout);
        let mut term = Terminal::new(backend)?;

        let (bg_r, bg_g, bg_b) = rgb_parts(&self.config.colors.background_color);
        let (fg_r, fg_g, fg_b) = rgb_parts(&self.config.colors.search_bar_foreground_color);

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Search")
            .border_style(Style::default().fg(Color::Rgb(fg_r, fg_g, fg_b)))
            .style(Style::default().bg(Color::Rgb(bg_r, bg_g, bg_b)));

        let mut input = Input::default();

        let size = term.size()?;
        let search_area = Rect {
            x: 1,
            y: size.height - 5,
            width: size.width - 2,
            height: 3,
        };
        // The borders take one column at the left and one column at the right.
        let inner_width = search_area.width.saturating_sub(2);

        loop {
            let view = field_view(&input, inner_width, None);
            term.draw(|f| {
                let bar = Paragraph::new(view.text.as_str())
                    .scroll((0, view.scroll))
                    .block(block.clone());
                f.render_widget(bar, search_area);
                f.set_cursor_position((search_area.x + 1 + view.cursor, search_area.y + 1));
            })?;

            let event = crossterm::event::read()?;
            match event {
                Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    kind: KeyEventKind::Press,
                    ..
                }) => {
                    self.search_mode = false;
                    self.search_query = input.value().to_string();
                    self.view_state = AppView::SearchBook;
                    self.list_state_search_results.select(Some(0));

                    // The server searches better than this program: it finds
                    // an author, a series, a narrator, a tag, and a genre, and
                    // this program looks in the titles that it holds. The
                    // screen shows the titles at once, and it shows the answer
                    // of the server when it comes. See T-24.
                    self.ask_the_server_to_search();
                    break;
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Esc,
                    kind: KeyEventKind::Press,
                    ..
                }) => {
                    self.search_mode = false;
                    break;
                }
                other => {
                    input.handle_event(&other);
                }
            }
        }
        term.draw(|f| {
            let empty_block =
                Block::default().style(Style::default().bg(Color::Rgb(bg_r, bg_g, bg_b)));
            f.render_widget(empty_block, search_area);
        })?;

        Ok(input.value().to_string())
    }
}
