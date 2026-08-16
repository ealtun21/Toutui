use crate::app::App;
use crate::app::AppView;
use crate::config::rgb_parts;
use crate::ui::text_field::field_view;
use crate::ui::text_field::the_backend_of_a_field;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Terminal;
use std::io;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

impl App {
    pub fn search_active(&mut self) -> io::Result<String> {
        // **The screen of a field takes no lock of the standard output.** A
        // panic of another thread would then wait for this screen for ever.
        // See T-174.
        let mut term = Terminal::new(the_backend_of_a_field())?;

        let (bg_r, bg_g, bg_b) = rgb_parts(&self.config.colors.background_color);
        let (fg_r, fg_g, fg_b) = rgb_parts(&self.config.colors.search_bar_foreground_color);

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Search")
            .border_style(Style::default().fg(Color::Rgb(fg_r, fg_g, fg_b)))
            .style(Style::default().bg(Color::Rgb(bg_r, bg_g, bg_b)));

        let mut input = Input::default();

        loop {
            // **The terminal can change its size while the box stands**,
            // therefore the area comes at each turn of this loop. See T-115 and
            // `logic::prompt`.
            let (search_area, _) = crate::logic::prompt::the_areas_of_the_box(&term)?;

            // The borders take one column at the left and one column at the
            // right.
            let inner_width = search_area.width.saturating_sub(2);
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

                    // The key `h` of the view of the search gives this view
                    // back. A second search inside that view keeps the view
                    // that the first search came from. See T-79.
                    if !matches!(self.view_state, AppView::SearchBook) {
                        self.the_view_before_the_search = self.view_state;
                    }

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
        let (search_area, _) = crate::logic::prompt::the_areas_of_the_box(&term)?;

        term.draw(|f| {
            let empty_block =
                Block::default().style(Style::default().bg(Color::Rgb(bg_r, bg_g, bg_b)));
            f.render_widget(empty_block, search_area);
        })?;

        // The box wrote on the cells of the view below it, and the terminal of
        // the program knows nothing of that work: it writes the cells that
        // changed only. Therefore the loop of the program draws every cell
        // again. See T-303, and `logic::prompt` for the same answer of the box
        // of a text.
        self.the_box_of_a_field_went_away();

        Ok(input.value().to_string())
    }
}
