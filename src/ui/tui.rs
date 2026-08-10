use crate::app::App;
use crate::app::AppView;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::Line,
    widgets::{
        Block, Borders, Gauge, HighlightSpacing, List, ListItem , ListState,  Paragraph,
        StatefulWidget, Widget, Wrap
    },
};
use crate::logic::download::progress::DownloadState;
use crate::utils::convert_seconds::*;
use crate::config::*;


// const version
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// init widget for selected AppView 
impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.view_state {
            AppView::Home => self.render_home(area, buf),
            AppView::Library => self.render_library(area, buf),
            AppView::SearchBook => self.render_search_book(area, buf),
            AppView::PodcastEpisode => self.render_pod_ep(area, buf),
            AppView::Series => self.render_series(area, buf),
            AppView::SeriesBook => self.render_series_book(area, buf),
            AppView::Settings => self.render_settings(area, buf),
            AppView::SettingsAccount => self.render_settings_account(area, buf),
            AppView::SettingsLibrary => self.render_settings_library(area, buf),
            AppView::SettingsAbout => {},
            AppView::SettingsUpdateUninstall => {},
        }

        // The bar goes above the other widgets. Therefore the user sees a
        // download in every view.
        App::render_downloads(area, buf);
    }
}

/// The number of lines of one download bar.
const DOWNLOAD_BAR_HEIGHT: u16 = 1;

/// The largest number of bars on the screen at the same time.
const DOWNLOAD_BAR_MAX: usize = 3;

impl App {
    /// Draws a bar for each download that runs.
    ///
    /// The function draws nothing when no download runs. It reads the global
    /// map of the progress. The download task writes that map.
    fn render_downloads(area: Rect, buf: &mut Buffer) {
        let map = crate::logic::download::downloads();

        // A lock that fails must not stop the screen. The function draws
        // nothing in that condition.
        let Ok(map) = map.read() else {
            return;
        };

        let mut running: Vec<_> = map
            .values()
            .filter(|item| item.state == DownloadState::Running)
            .collect();

        if running.is_empty() {
            return;
        }

        // The sequence must be the same for each frame. Therefore the
        // function sorts by the identity of the download.
        running.sort_by(|a, b| a.key.cmp(&b.key));
        running.truncate(DOWNLOAD_BAR_MAX);

        let height = DOWNLOAD_BAR_HEIGHT * running.len() as u16;

        if area.height <= height {
            return;
        }

        for (row, item) in running.iter().enumerate() {
            let line = Rect {
                x: area.x,
                y: area.y + area.height - height + row as u16,
                width: area.width,
                height: DOWNLOAD_BAR_HEIGHT,
            };

            let label = if item.file_count > 1 {
                format!(
                    " ⬇ {}  file {}/{}  {} / {} ",
                    shorten(&item.title, 28),
                    item.file_index,
                    item.file_count,
                    megabytes(item.bytes_done),
                    megabytes(item.bytes_total),
                )
            } else {
                format!(
                    " ⬇ {}  {} / {} ",
                    shorten(&item.title, 34),
                    megabytes(item.bytes_done),
                    megabytes(item.bytes_total),
                )
            };

            Gauge::default()
                .gauge_style(Style::default().fg(Color::Green).bg(Color::DarkGray))
                .percent(item.percent())
                .label(label)
                .render(line, buf);
        }
    }
}

/// Changes a number of bytes to a text in megabytes.
fn megabytes(bytes: u64) -> String {
    format!("{:.1} MB", bytes as f64 / 1_048_576.0)
}

/// Makes a text shorter. The function adds a full stop character.
fn shorten(text: &str, width: usize) -> String {
    if text.chars().count() <= width {
        return text.to_string();
    }

    let kept: String = text.chars().take(width.saturating_sub(1)).collect();
    format!("{}…", kept)
}


/// Rendering logic
impl App {
    /// AppView::Home rendering
    fn render_home(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, _player_area, _refresh_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(6),
            Constraint::Length(1),
            Constraint::Length(2),
        ]).areas(area);

        let [list_area, item_area1, item_area2] = Layout::vertical([Constraint::Fill(1), Constraint::Length(3), Constraint::Fill(1)]).areas(main_area);

        let items_number = self._titles_cnt_list.len();
        let render_list_title = format!("Continue Listening [{} items]", items_number);

        let text_render_footer = "j/↓, k/↑: move, l/→: play, Tab: library, R: refresh, S: Settings, Q/Esc: quit\n B: toggle player ctrl, D: download offline, X: remove offline, s: series, '/': search, Scroll desc: J(↓) K(↑) H(⇡), g/G: top/bot";

        App::render_header(header_area, buf, self.lib_name_type.clone(), &self.username, &self.server_address_pretty, VERSION, &self.update_msg);
        App::render_footer(footer_area, buf, text_render_footer);
        self.render_list(list_area, buf, &render_list_title, &self._titles_cnt_list.clone(), &mut self.list_state_cnt_list.clone());
        if !&self._titles_cnt_list.is_empty() {
            self.render_info_home(item_area1, buf, &self.list_state_cnt_list.clone());
            self.render_desc_home(item_area2, buf, &self.list_state_cnt_list.clone());
        }
    }

    /// AppView::Library rendering
    fn render_library(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, _player_area, _refresh_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(6),
            Constraint::Length(1),
            Constraint::Length(2),
        ]).areas(area);

        let [list_area, item_area1, item_area2] = Layout::vertical([Constraint::Fill(1), Constraint::Length(3), Constraint::Fill(1)]).areas(main_area);

        let items_number = self.titles_library.len();
        let render_list_title = format!("Library [{} items]", items_number);

        let mut _text_render_footer = "";
        if self.is_podcast {
        _text_render_footer = "j/↓, k/↑: move, l/→: episodes, Tab: home, R: refresh, S: Settings, Q/Esc: quit\n B: toggle player ctrl, '/': search, Scroll desc: J(↓) K(↑) H(⇡), g/G: top/bot"       
        } else {
        _text_render_footer = "j/↓, k/↑: move, l/→: play, Tab: home, R: refresh, S: Settings, Q/Esc: quit\n B: toggle player ctrl, D: download offline, X: remove offline, s: series, '/': search, Scroll desc: J(↓) K(↑) H(⇡), g/G: top/bot";
        }

        App::render_header(header_area, buf, self.lib_name_type.clone(), &self.username, &self.server_address_pretty, VERSION, &self.update_msg);
        App::render_footer(footer_area, buf, _text_render_footer);
        self.render_list(list_area, buf, &render_list_title, &self.titles_library.clone(), &mut self.list_state_library.clone());
        if !&self.titles_library.is_empty() {
            self.render_info_library(item_area1, buf, &self.list_state_library.clone());
            self.render_desc_library(item_area2, buf, &self.list_state_library.clone());
        }
    }

    /// AppView::Series rendering: the list of the series of the library.
    fn render_series(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, _player_area, _refresh_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(6),
            Constraint::Length(1),
            Constraint::Length(2),
        ]).areas(area);

        let [list_area, item_area1, item_area2] = Layout::vertical([Constraint::Fill(1), Constraint::Length(3), Constraint::Fill(1)]).areas(main_area);

        let text_render_footer = "j/↓, k/↑: move, l/→: books of the series, h: back, Tab: home, R: refresh, S: Settings, Q/Esc: quit\n Scroll desc: J(down) K(up) H(top), g/G: top/bottom";

        App::render_header(header_area, buf, self.lib_name_type.clone(), &self.username, &self.server_address_pretty, VERSION, &self.update_msg);
        App::render_footer(footer_area, buf, text_render_footer);

        if self.series.is_empty() {
            Paragraph::new("This library has no series.\nPress 'h' to go back.")
                .centered()
                .block(Block::new().borders(Borders::TOP).border_style(Style::new().fg(Color::DarkGray)))
                .render(main_area, buf);
            return;
        }

        let lines: Vec<String> = self.series.iter().map(|series| series.line()).collect();
        let render_list_title = format!("Series [{} items]", self.series.len());

        self.render_list(list_area, buf, &render_list_title, &lines, &mut self.list_state_series.clone());

        if let Some(series) = self.selected_series() {
            let books = series.books.len();
            let seconds: f64 = series.books.iter().map(|book| book.duration).sum();

            Paragraph::new(format!(
                "{} - {} book(s) - Duration: {}",
                series.name,
                books,
                convert_seconds(vec![seconds]).first().cloned().unwrap_or_default(),
            ))
                .left_aligned()
                .render(item_area1, buf);

            Paragraph::new(series.description.clone())
                .scroll((self.scroll_offset, 0))
                .wrap(Wrap { trim: true })
                .render(item_area2, buf);
        }
    }

    /// AppView::SeriesBook rendering: the books of one series.
    fn render_series_book(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, _player_area, _refresh_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(6),
            Constraint::Length(1),
            Constraint::Length(2),
        ]).areas(area);

        let [list_area, item_area1, item_area2] = Layout::vertical([Constraint::Fill(1), Constraint::Length(3), Constraint::Fill(1)]).areas(main_area);

        let text_render_footer = "j/↓, k/↑: move, l/→: play, h: back, Tab: home, R: refresh, S: Settings, Q/Esc: quit\n D: download offline, X: remove offline, Scroll desc: J(down) K(up) H(top), g/G: top/bottom";

        App::render_header(header_area, buf, self.lib_name_type.clone(), &self.username, &self.server_address_pretty, VERSION, &self.update_msg);
        App::render_footer(footer_area, buf, text_render_footer);

        let Some(series) = self.selected_series() else {
            return;
        };

        let name = series.name.clone();
        let lines: Vec<String> = series.books.iter().map(|book| book.line()).collect();
        let render_list_title = format!("{} [{} items]", name, lines.len());

        self.render_list(list_area, buf, &render_list_title, &lines, &mut self.list_state_series_book.clone());

        if let Some(book) = self.selected_series_book() {
            let is_offline = crate::db::crud::get_download(&book.id, &self.username).is_some();

            Paragraph::new(format!(
                "Author: {} - Duration: {}{}",
                book.author,
                convert_seconds(vec![book.duration]).first().cloned().unwrap_or_default(),
                if is_offline { " - [Downloaded]" } else { "" },
            ))
                .left_aligned()
                .render(item_area1, buf);

            Paragraph::new(book.description.clone())
                .scroll((self.scroll_offset, 0))
                .wrap(Wrap { trim: true })
                .render(item_area2, buf);
        }
    }

    /// AppView::Settings rendering
    fn render_settings(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, _player_area, _refresh_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(6),
            Constraint::Length(1),
            Constraint::Length(2),
        ]).areas(area);

        let [list_area, item_area1, item_area2] = Layout::vertical([Constraint::Fill(1), Constraint::Length(3), Constraint::Fill(1)]).areas(main_area);

        let render_list_title = "Settings";

        let mut _text_render_footer = "";
        if self.list_state_settings.selected() == Some(2) {
            // for `About` section
            _text_render_footer = "j/↓, k/↑: move, Scroll what's new: J(down) K(up) H(top),\n Tab: home, R: refresh, Q/Esc: quit.";
        }
        else if self.list_state_settings.selected() == Some(3) {
            _text_render_footer = "j/↓, k/↑: move, Scroll : J(down) K(up) H(top),\n Tab: home, R: refresh, Q/Esc: quit.";

        } else {
            _text_render_footer = "j/↓, k/↑: move, l/→: see options,\n Tab: home, R: refresh, Q/Esc: quit.";
        }

        App::render_header(header_area, buf, self.lib_name_type.clone(), &self.username, &self.server_address_pretty, VERSION, &self.update_msg);
        App::render_footer(footer_area, buf, _text_render_footer);
        self.render_list(list_area, buf, render_list_title, &self.settings.clone(), &mut self.list_state_settings.clone());
        self.render_info_settings(item_area1, buf, &self.list_state_settings.clone());
        self.render_desc_settings(item_area2, buf, &self.list_state_settings.clone());
    }

    /// AppView::SettingsAccount rendering
    fn render_settings_account(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, _player_area, _refresh_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(6),
            Constraint::Length(1),
            Constraint::Length(2),
        ]).areas(area);

        let [list_area, _item_area] = Layout::vertical([Constraint::Fill(1), Constraint::Fill(1),]).areas(main_area);

        let render_list_title = "Settings account";
        let text_render_footer = "h: back, l/→: remove saved user,\n Tab: home, R: refresh, Q/Esc: quit.";

        App::render_header(header_area, buf, self.lib_name_type.clone(), &self.username, &self.server_address_pretty, VERSION, &self.update_msg);
        App::render_footer(footer_area, buf, text_render_footer);
        self.render_list(list_area, buf, render_list_title, &self.all_usernames.clone(), &mut self.list_state_settings_account.clone() );
        //self.render_selected_item(item_area, buf, &self.titles_library.clone(), self.auth_names_library.clone());
    }

    /// AppView::SettingsLibrary rendering
    fn render_settings_library(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, _player_area, _refresh_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(6),
            Constraint::Length(1),
            Constraint::Length(2),
        ]).areas(area);

        let [list_area, item_area] = Layout::vertical([Constraint::Fill(1), Constraint::Fill(1),]).areas(main_area);

        let items_number = self.libraries_names.len();
        let render_list_title = format!("Settings Library [{} items]", items_number);

        let text_render_footer = "h: back, l/→: change library,\n Tab: home, R: refresh, Q/Esc: quit.";

        App::render_header(header_area, buf, self.lib_name_type.clone(), &self.username, &self.server_address_pretty, VERSION, &self.update_msg);
        App::render_footer(footer_area, buf, text_render_footer);
        self.render_list(list_area, buf, &render_list_title, &self.libraries_names.clone(), &mut self.list_state_settings_library.clone());
        self.render_info_settings_library(item_area, buf, &self.list_state_settings_library.clone());
    }


    /// AppView::SearchBook rendering
    fn render_search_book(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, _player_area, _refresh_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(6),
            Constraint::Length(1),
            Constraint::Length(2),
        ]).areas(area);

        let [list_area, item_area1, item_area2] = Layout::vertical([Constraint::Fill(1), Constraint::Length(3), Constraint::Fill(1)]).areas(main_area);

        let render_list_title = "Search result";
        let mut _text_render_footer = "";
        if self.is_podcast {
        _text_render_footer = "j/↓, k/↑: move, l/→: episodes, Tab: home, R: refresh, S: Settings, Q/Esc: quit\n '/': search, Scroll desc: J(down) K(up) H(top), g/G: top/bottom";
        } else {
        _text_render_footer = "j/↓, k/↑: move, l/→: play, Tab: home, R: refresh, S: Settings, Q/Esc: quit\n D: download offline, X: remove offline, s: series, '/': search, Scroll desc: J(down) K(up) H(top), g/G: top/bottom";
        }


        if self.search_mode {
            if let Ok(query) = self.search_active() {
                self.search_query = query.to_string();
                self.search_mode = false; 
            }
        }

        // init variables for search result (search by a book by title)
        let idx_and_titles: Vec<(usize, String)> = self.titles_library
            .iter()
            .enumerate() 
            .filter(|(_, x)| x.to_lowercase().contains(&self.search_query.to_lowercase())) 
            .map(|(index, title)| (index, title.clone())) 
            .collect();

        let mut titles_search_book_or_pod: Vec<String> = Vec::new();
        let mut index_to_keep: Vec<usize> = Vec::new();
        for (index, title) in idx_and_titles {
            titles_search_book_or_pod.push(title.to_string());
            index_to_keep.push(index)
        }

        let titles_search_book_or_pod: &[String] = &titles_search_book_or_pod;

        // apply search filtering for book
        self.ids_search_book = self.ids_library
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.auth_names_pod_search_book = self.auth_names_library_pod
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.auth_names_search_book = self.auth_names_library
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.published_year_library_search_book = self.published_year_library
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.desc_library_search_book = self.desc_library
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.duration_library_search_book = self.duration_library
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| *value)
            .collect();
//        self.book_progress_search_book = self.book_progress_library
//            .iter()
//            .enumerate()
//            .filter(|(index, _)| index_to_keep.contains(&index))
//            .map(|(_, value)| value.clone())
//            .collect();
//        self.book_progress_search_book_cur_time = self.book_progress_library_cur_time
//            .iter()
//            .enumerate()
//            .filter(|(index, _)| index_to_keep.contains(&index))
//            .map(|(_, value)| value.clone())
//            .collect();
//        self.book_progress_search_book = self.book_progress_library
//            .iter()
//            .enumerate()
//            .filter(|(index, _)| index_to_keep.contains(&index))
//            .map(|(_, value)| value.clone())
//            .collect();

        // apply search filtering for podacst
        self.all_titles_pod_ep_search = self.all_titles_pod_ep
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.all_ids_pod_ep_search = self.all_ids_pod_ep
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.all_subtitles_pod_ep_search = self.all_subtitles_pod_ep
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.all_seasons_pod_ep_search = self.all_seasons_pod_ep
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.all_episodes_pod_ep_search = self.all_episodes_pod_ep
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.all_authors_pod_ep_search = self.all_authors_pod_ep
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.all_descs_pod_ep_search = self.all_descs_pod_ep
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.all_titles_pod_search = self.all_titles_pod
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.all_durations_pod_ep_search = self.all_durations_pod_ep
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.ids_library_pod_search = self.ids_library
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();

        App::render_header(header_area, buf, self.lib_name_type.clone(), &self.username, &self.server_address_pretty, VERSION, &self.update_msg);
        App::render_footer(footer_area, buf, _text_render_footer);
        self.render_list(list_area, buf, render_list_title, titles_search_book_or_pod, &mut self.list_state_search_results.clone());
        if !titles_search_book_or_pod.is_empty() {
            self.render_info_search_book(item_area1, buf, &self.list_state_search_results.clone() );
            self.render_desc_search_book(item_area2, buf, &self.list_state_search_results.clone() );
        }
    }

    /// AppView::PodcastEpisode
    fn render_pod_ep(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, _player_area, _refresh_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(6),
            Constraint::Length(1),
            Constraint::Length(2),
        ]).areas(area);

        let [list_area, item_area1, item_area2] = Layout::vertical([Constraint::Fill(1), Constraint::Length(3), Constraint::Fill(1)]).areas(main_area);


        let text_render_footer = "j/↓, k/↑: move, l/→: play, h: back, Tab: home, R: refresh, S: Settings, Q/Esc: quit\n D: download offline, X: remove offline, '/': search, Scroll desc: J(down) K(up) H(top), g/G: top/bottom";

        App::render_header(header_area, buf, self.lib_name_type.clone(), &self.username, &self.server_address_pretty, VERSION, &self.update_msg);
        App::render_footer(footer_area, buf, text_render_footer);
        let no_episodes_message = "No episodes found for this podcast.\nPress 'h' to go back.";

        if self.is_from_search_pod {
            if self.titles_pod_ep_search.is_empty() {
                log::warn!("render_pod_ep (search): No episodes found.");
                Paragraph::new(no_episodes_message)
                    .centered()
                    .block(Block::new().borders(Borders::TOP).border_style(Style::new().fg(Color::DarkGray)))
                    .render(main_area, buf);
            } else {
                let items_number = self.titles_pod_ep_search.len();
                let render_list_title = format!("Episodes [{} items]", items_number);
                // Only render list/info/desc if episodes exist
                self.render_list(list_area, buf, &render_list_title, &self.titles_pod_ep_search.clone(), &mut self.list_state_pod_ep.clone());
                self.render_info_pod_ep_search(item_area1, buf, &self.list_state_pod_ep.clone() );
                self.render_desc_pod_ep_search(item_area2, buf, &self.list_state_pod_ep.clone() );
            }
        } else {
            if self.titles_pod_ep.is_empty() {
                log::warn!("render_pod_ep (library): No episodes found.");
                Paragraph::new(no_episodes_message)
                    .centered()
                    .block(Block::new().borders(Borders::TOP).border_style(Style::new().fg(Color::DarkGray)))
                    .render(main_area, buf);
            } else {
                let items_number = self.titles_pod_ep.len();
                let render_list_title = format!("Episodes [{} items]", items_number);
                // Only render list/info/desc if episodes exist
                self.render_list(list_area, buf, &render_list_title, &self.titles_pod_ep.clone(), &mut self.list_state_pod_ep.clone());
                self.render_info_pod_ep(item_area1, buf, &self.list_state_pod_ep.clone() );
                self.render_desc_pod_ep(item_area2, buf, &self.list_state_pod_ep.clone() );
            }
        }
    }

    // General functions for rendering 

    fn render_header(area: Rect, buf: &mut Buffer, library_name: String, username: &str, server_address_pretty: &str, version: &str, update_msg: &str) {
        Paragraph::new(library_name)
            .bold()
            .centered()
            .render(area, buf);
        Paragraph::new(format!("👋 Connected as {}\n🔗 {}", &username, &server_address_pretty))
            .not_bold()
            .left_aligned()
            .render(area, buf);
        Paragraph::new(format!("🦜 Toutui v{}\n {}", version, update_msg))
            .right_aligned()
            .render(area, buf);
    }

    fn render_footer(area: Rect, buf: &mut Buffer, text_render_footer: &str) {
        Paragraph::new(text_render_footer)
            .centered()
            .render(area, buf);
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer, render_list_title: &str, render_list_items: &[String], list_state: &mut ListState) {
        let bg_color_header = self.config.colors.header_background_color.clone();
        let fg_color_header = self.config.colors.line_header_color.clone();
        let bg_color_block = self.config.colors.list_background_color.clone();
        let bg_selected = self.config.colors.list_selected_background_color.clone();
        let fg_selected = self.config.colors.list_selected_foreground_color.clone();
        let selected_style: Style = Style::new()
            .bg(Color::Rgb(bg_selected[0], bg_selected[1], bg_selected[2]))  
            .fg(Color::Rgb(fg_selected[0], fg_selected[1], fg_selected[2])) 
            .add_modifier(Modifier::BOLD);

        let header_style: Style = Style::new()
            .fg(Color::Rgb(fg_color_header[0], fg_color_header[1], fg_color_header[2]))
            .bg(Color::Rgb(bg_color_header[0], bg_color_header[1], bg_color_header[2])); 

        let block = Block::new()
            .title(Line::raw(render_list_title.to_string()).centered())
            .borders(Borders::TOP)
            .border_style(header_style)
            .bg(Color::Rgb(bg_color_block[0], bg_color_block[1], bg_color_block[2]));

        let items: Vec<ListItem> = render_list_items
            .iter()
            .enumerate()
            .map(|(i, title)| {
                let color = Self::alternate_colors(i);
                ListItem::new(title.clone()).bg(color)
            })
        .collect();


        let list = List::new(items)
            .block(block)
            .highlight_style(selected_style)
            .highlight_symbol("➤ ")
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, list_state);
    }


    // info about the book or podacst for `Home`
    fn render_info_home(&self, area: Rect, buf: &mut Buffer, list_state: &ListState) {
        let duration_cnt_list_conv = convert_seconds(self.duration_cnt_list.clone());

        if let Some(selected) = list_state.selected() {

            if self.is_podcast {
                let is_offline = self.ids_ep_cnt_list.get(selected)
                    .map(|id| crate::db::crud::get_download(id, &self.username).is_some())
                    .unwrap_or(false);

                Paragraph::new(format!("[{}] - Author: {} - Episode: {} - Duration: {}{}",
                        self.titles_pod_cnt_list[selected],
                        self.authors_pod_cnt_list[selected],
                        self.nums_ep_pod_cnt_list[selected],
                        self.durations_pod_cnt_list[selected],
                        if is_offline { " - [Downloaded]" } else { "" },
                ))
                    .left_aligned()
                    .render(area, buf);
                } else {
                    let is_offline = crate::db::crud::get_download(&self._ids_cnt_list[selected], &self.username).is_some();
                    Paragraph::new(format!("Author: {} - Year: {} - Duration: {}{}\nProgress: {}%, {} {}",
                            self.auth_names_cnt_list[selected],
                            self.pub_year_cnt_list[selected],
                            duration_cnt_list_conv[selected],
                            if is_offline { " - [Downloaded]" } else { "" },
                            self.book_progress_cnt_list[selected][0], // percentage progression
                            convert_seconds_for_prg(self.duration_cnt_list[selected], self.book_progress_cnt_list_cur_time[selected][0]), // time left
                            self.book_progress_cnt_list[selected][1], // is finished
                    ))
                        .left_aligned()
                        .render(area, buf);
            }
        }
    }

    // description of the book or podcast `Home`
    fn render_desc_home(&self, area: Rect, buf: &mut Buffer, list_state: &ListState) {

        if let Some(selected) = list_state.selected() {
            let mut _content: String = String::new();
            if self.is_podcast {
                _content = self.subtitles_pod_cnt_list[selected].clone();
            } else {
                _content = self.desc_cnt_list[selected].clone();
            }

            Paragraph::new(_content.clone())
                .scroll((self.scroll_offset, 0))
                .wrap(Wrap { trim: true })
                .render(area, buf);
            }
    }

    // info about the book or podacst for `Library`
    fn render_info_library(&self, area: Rect, buf: &mut Buffer, list_state: &ListState) {
        let _duration_library_conv = convert_seconds(self.duration_library.clone());

        if let Some(selected) = list_state.selected() {
            if self.is_podcast {
                Paragraph::new(format!("Author: {}", 
                        self.auth_names_library_pod[selected], 
                ))
                    .left_aligned()
                    .render(area, buf);
            } 
            else {
                let is_offline = crate::db::crud::get_download(&self.ids_library[selected], &self.username).is_some();
                Paragraph::new(format!("Author: {} - Year: {}{}", //- Duration: {}\nProgress:{} {}{}",
                        self.auth_names_library[selected],
                        self.published_year_library[selected],
                        if is_offline { " - [Downloaded]" } else { "" },

                        //duration_library_conv[selected],
                        //self.book_progress_library[selected][0], // percentage progression
                        //format!("{}",convert_seconds_for_prg(self.duration_library[selected], self.book_progress_library_cur_time[selected][0])), // time left
                        //self.book_progress_library[selected][1] // is_finished
                        ))
                    .left_aligned()
                    .render(area, buf);
            }
        }
    }

    // description of the book or podcast `Library`
    fn render_desc_library(&self, area: Rect, buf: &mut Buffer, list_state: &ListState) {

        if let Some(selected) = list_state.selected() {

            Paragraph::new(self.desc_library[selected].clone())
                .scroll((self.scroll_offset, 0))
                .wrap(Wrap { trim: true })
                .render(area, buf);
        }
    }

    // info about the podcast for `PodcastEpisode`
    fn render_info_pod_ep(&self, area: Rect, buf: &mut Buffer, list_state: &ListState) {

        // Check if source vectors for podcast title/author are empty before accessing index 0
        if self.titles_pod.is_empty() || self.authors_pod_ep.is_empty() {
            log::error!("render_info_pod_ep: titles_pod or authors_pod_ep is empty. Cannot render episode info.");
            // Render placeholder text or handle appropriately
            Paragraph::new("Error: Podcast metadata missing.")
                .left_aligned()
                .render(area, buf);
            return; // Exit the function early
        }

        let n = self.durations_pod_ep.len();
        // Now safe to access index 0 as we've checked they are not empty
        let duplicated_titles = vec![self.titles_pod[0].clone(); n];
        let duplicated_authors = vec![self.authors_pod_ep[0].clone(); n];

        if let Some(selected) = list_state.selected() {
            log::debug!(
                "render_info_pod_ep: selected={}, titles_pod.len={}, authors_pod_ep.len={}, durations_pod_ep.len={}, episodes_pod_ep.len={}, duplicated_titles.len={}, duplicated_authors.len={}",
                selected,
                self.titles_pod.len(), // Should be >= 1 here
                self.authors_pod_ep.len(), // Should be >= 1 here
                self.durations_pod_ep.len(),
                self.episodes_pod_ep.len(),
                duplicated_titles.len(), // Will be n
                duplicated_authors.len() // Will be n
            );

            // Check if episode-specific vectors are valid for the selected index
            if selected < self.episodes_pod_ep.len() && selected < self.durations_pod_ep.len() {
                 // Also check duplicated vectors, though their length depends on n (durations_pod_ep.len())
                 if selected < duplicated_titles.len() && selected < duplicated_authors.len() {
                    let is_offline = self.ids_pod_ep.get(selected)
                        .map(|id| crate::db::crud::get_download(id, &self.username).is_some())
                        .unwrap_or(false);

                    Paragraph::new(format!("[{}] - Author: {} - Episode: {} - Duration: {} {}",
                            duplicated_titles[selected].trim(),
                            duplicated_authors[selected].trim(),
                            self.episodes_pod_ep[selected].trim(),
                            self.durations_pod_ep[selected].trim(),
                            if is_offline { "- [Downloaded]" } else { "" },
                    ))
                        .left_aligned()
                        .render(area, buf);
                 } else {
                     log::error!("render_info_pod_ep: Index {} out of bounds for duplicated title/author vectors (len={})!", selected, duplicated_titles.len());
                     Paragraph::new("Error: Episode info rendering mismatch.")
                         .left_aligned()
                         .render(area, buf);
                 }
            } else {
                log::error!("render_info_pod_ep: Index {} out of bounds for episode/duration vectors (ep_len={}, dur_len={})!", selected, self.episodes_pod_ep.len(), self.durations_pod_ep.len());
                Paragraph::new("Error: Episode data unavailable or index out of bounds.")
                    .left_aligned()
                    .render(area, buf);
            }
        }
    }
    // info about the podcast for `PodcastEpisode` (from search)
    fn render_info_pod_ep_search(&self, area: Rect, buf: &mut Buffer, list_state: &ListState) {

        let n = self.durations_pod_ep_search.len();
        let duplicated_titles_search = vec![self.titles_pod_search[0].clone(); n];
        let duplicated_authors_search = vec![self.authors_pod_ep_search[0].clone(); n];
        if let Some(selected) = list_state.selected() {

            let is_offline = self.ids_pod_ep_search.get(selected)
                .map(|id| crate::db::crud::get_download(id, &self.username).is_some())
                .unwrap_or(false);

            Paragraph::new(format!("[{}] - Author: {} - Episode: {} - Duration: {} {}",
                    duplicated_titles_search[selected].trim(),
                    duplicated_authors_search[selected].trim(),
                    self.episodes_pod_ep_search[selected].trim(),
                    self.durations_pod_ep_search[selected].trim(),
                    if is_offline { "- [Downloaded]" } else { "" },
            ))
                .left_aligned()
                .render(area, buf);
        }
    }

    // desc of the podcast for `PodcastEpisode`
    fn render_desc_pod_ep(&self, area: Rect, buf: &mut Buffer, list_state: &ListState) {

        if let Some(selected) = list_state.selected() {
            log::debug!("render_desc_pod_ep: selected={}, subtitles_pod_ep.len={}", selected, self.subtitles_pod_ep.len());

            // Check if index is valid for subtitles vector
            if selected < self.subtitles_pod_ep.len() {
                Paragraph::new(self.subtitles_pod_ep[selected].clone())
                    .scroll((self.scroll_offset, 0))
                    .wrap(Wrap { trim: true })
                    .render(area, buf);
            } else {
                log::error!("render_desc_pod_ep: Index {} out of bounds for subtitles_pod_ep (len={})!", selected, self.subtitles_pod_ep.len());
                // Render placeholder text
                Paragraph::new("Error: Episode description unavailable.")
                    .left_aligned()
                    .render(area, buf);
            }
        }
    }
    // desc of the podcast for `PodcastEpisode` (from search)
    fn render_desc_pod_ep_search(&self, area: Rect, buf: &mut Buffer, list_state: &ListState) {

        if let Some(selected) = list_state.selected() {

            Paragraph::new(self.subtitles_pod_ep_search[selected].clone())
                .scroll((self.scroll_offset, 0))
                .wrap(Wrap { trim: true })
                .render(area, buf);
        }
    }

    // info about the book or podacst for `SearchBook`
    fn render_info_search_book(&self, area: Rect, buf: &mut Buffer, list_state: &ListState) {
        let _duration_library_search_book_conv = convert_seconds(self.duration_library_search_book.clone());

        if let Some(selected) = list_state.selected() {
            if self.is_podcast {
                Paragraph::new(format!("Author: {}", 
                        self.auth_names_pod_search_book[selected], 
                ))
                    .left_aligned()
                    .render(area, buf);
            } 
            else {
                let is_offline = self.ids_search_book.get(selected)
                    .map(|id| crate::db::crud::get_download(id, &self.username).is_some())
                    .unwrap_or(false);
                Paragraph::new(format!("Author: {} - Year: {}{}", //- Duration: {}\nProgress:{} {}{}",
                        self.auth_names_search_book[selected],
                        self.published_year_library_search_book[selected],
                        if is_offline { " - [Downloaded]" } else { "" },
                      //  duration_library_search_book_conv[selected],
                      //  self.book_progress_search_book[selected][0], // percentage progression
                      //  format!("{}",convert_seconds_for_prg(self.duration_library_search_book[selected], self.book_progress_search_book_cur_time[selected][0])), // time left
                      //  self.book_progress_search_book[selected][1] // is finished
                        ))
                    .left_aligned()
                    .render(area, buf);
            }
        }
    }

    // description of the book or podcast `SearchBook`
    fn render_desc_search_book(&self, area: Rect, buf: &mut Buffer, list_state: &ListState) {

        if let Some(selected) = list_state.selected() {

            Paragraph::new(self.desc_library_search_book[selected].clone())
                .scroll((self.scroll_offset, 0))
                .wrap(Wrap { trim: true })
                .render(area, buf);
        }
    }

    // info for settings
    fn render_info_settings(&self, area: Rect, buf: &mut Buffer, list_state: &ListState) {

        match list_state.selected() {
            Some(0) => {}
            Some(1) => {}
            Some(2) => {

                Paragraph::new(format!("Toutui v{} - Licence: GPL-3.0 - Contact: albdav.dev@gmail.com\nSource code: {}\nWhat's new:", 
                        VERSION,
                        "https://github.com/AlbanDAVID/Toutui",
                ))
                    .left_aligned()
                    .render(area, buf);
                }
            _ => {}
        }

    }

    
    // desc for settings
    fn render_desc_settings(&self, area: Rect, buf: &mut Buffer, list_state: &ListState) {

        let instructions = "\
Update:
- Quit the app
- If you installed Toutui via yay: yay -S toutui
- If you installed Toutui using the script: toutui --update

Uninstall:
- Quit the app
- If you installed Toutui via yay: yay -R toutui-bin
- If you installed Toutui using the script: toutui --uninstall
";

        match list_state.selected() {

            Some(0) => {}
            Some(1) => {}
            Some(2) => {
                Paragraph::new(self.changelog.clone())
                    .scroll((self.scroll_offset, 0))
                    .wrap(Wrap { trim: true })
                    .render(area, buf);
                }
            Some(3) => {
                Paragraph::new(instructions)
                    .scroll((self.scroll_offset, 0))
                    .wrap(Wrap { trim: true })
                    .render(area, buf);
                }
            _ =>  {}
        }
    }

    // info for settings library
    fn render_info_settings_library(&self, area: Rect, buf: &mut Buffer, list_state: &ListState) {

        if let Some(selected) = list_state.selected() {
                Paragraph::new(format!("Type: {}", 
                        self.media_types[selected], 
                ))
                    .left_aligned()
                    .render(area, buf);
            } 

    }

    fn alternate_colors(i: usize) -> Color {
        let mut color_bg_list = Vec::new();
        let mut color_alt_bg_list = Vec::new();
        if let Ok(cfg) = load_config() {
            color_bg_list = cfg.colors.list_background_color;
            color_alt_bg_list = cfg.colors.list_background_color_alt_row;
        }
        if i.is_multiple_of(2) {
            Color::Rgb(color_bg_list[0], color_bg_list[1], color_bg_list[2])
        } else {
            Color::Rgb(color_alt_bg_list[0], color_alt_bg_list[1], color_alt_bg_list[2])
        }
    }
}
