use crate::app::App;
use crate::app::AppView;
use crate::config::*;
use crate::logic::download::progress::DownloadState;
use crate::player::engine::PlaybackStatus;
use crate::ui::cover;
use crate::utils::convert_seconds::*;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::Line,
    widgets::{
        Block, Borders, Gauge, HighlightSpacing, List, ListItem, ListState, Paragraph,
        StatefulWidget, Widget, Wrap,
    },
};
use ratatui_image::StatefulImage;

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
            AppView::Lists => self.render_lists(area, buf),
            AppView::ListEntries => self.render_list_entries(area, buf),
            AppView::Reader => self.render_reader(area, buf),
            AppView::Stats => self.render_stats(area, buf),
            AppView::Sessions => self.render_sessions(area, buf),
            AppView::SortFilter => self.render_sort_filter(area, buf),
            AppView::Chapters => self.render_chapters(area, buf),
            AppView::Bookmarks => self.render_bookmarks(area, buf),
            AppView::Queue => self.render_queue(area, buf),
            AppView::NewPodcast => self.render_new_podcast(area, buf),
            AppView::Authors => self.render_authors(area, buf),
            AppView::Keys => self.render_keys(area, buf),
            AppView::Settings => self.render_settings(area, buf),
            AppView::SettingsAccount => self.render_settings_account(area, buf),
            AppView::SettingsLibrary => self.render_settings_library(area, buf),
            AppView::SettingsAbout => {}
            AppView::SettingsUpdateUninstall => {}
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

/// Reads one text of a list of the screen.
///
/// A list of the screen can be shorter than the selection. An example: the
/// user removes an account, and the list keeps its old length until the next
/// refresh. Another example: a library gives 40 items, and the list of the
/// authors gives 39, because one item has no author. An index of a vector then
/// stops the whole program, and a panic inside `Widget::render` gives the user
/// no screen at all. This function gives a text instead. See T-41.
fn at(list: &[String], index: usize) -> &str {
    list.get(index).map(|value| value.as_str()).unwrap_or("N/A")
}

/// Reads one number of a list of the screen. See `at`.
fn at_number(list: &[f64], index: usize) -> f64 {
    list.get(index).copied().unwrap_or(0.0)
}

/// Reads one text of a list of lists of the screen. See `at`.
fn at_part(list: &[Vec<String>], index: usize, part: usize) -> &str {
    list.get(index)
        .and_then(|row| row.get(part))
        .map(|value| value.as_str())
        .unwrap_or("N/A")
}

/// Reads one number of a list of lists of the screen. See `at`.
fn at_number_part(list: &[Vec<f64>], index: usize, part: usize) -> f64 {
    list.get(index)
        .and_then(|row| row.get(part))
        .copied()
        .unwrap_or(0.0)
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

/// The cover art. See T-23.
impl App {
    /// Gives the identities of the media that the panel of the covers shows.
    ///
    /// A view of one medium gives one identity. A view of a series, of a
    /// collection, or of a playlist gives the identity of each of its media,
    /// therefore the panel looks like a shelf.
    fn cover_ids(&self) -> Vec<String> {
        let one = |value: Option<&String>| value.cloned().into_iter().collect::<Vec<String>>();

        match self.view_state {
            // The view of the keys holds no media, therefore it shows no
            // cover. See T-49.
            AppView::Keys => Vec::new(),
            // A line of a series shows the cover of each of its books, in
            // the same way as the Library view. See T-22 and T-24.
            AppView::Home if self.selected_home_series().is_some() => self
                .selected_home_series()
                .map(|series| {
                    series
                        .books
                        .iter()
                        .take(cover::SHELF_MAX)
                        .map(|book| book.id.clone())
                        .collect()
                })
                .unwrap_or_default(),

            AppView::Home => one(self
                .selected_home_item()
                .and_then(|index| self._ids_cnt_list.get(index))),

            // A line of a series shows the cover of each of its books. See
            // T-22.
            AppView::Library if self.selected_library_series().is_some() => self
                .selected_library_series()
                .map(|series| {
                    series
                        .books
                        .iter()
                        .take(cover::SHELF_MAX)
                        .map(|book| book.id.clone())
                        .collect()
                })
                .unwrap_or_default(),

            // An episode has no cover of its own. The cover of the podcast
            // stands for every episode of that podcast.
            AppView::Library | AppView::PodcastEpisode => one(self
                .selected_library_item()
                .and_then(|index| self.ids_library.get(index))),

            AppView::SearchBook => {
                let index = self.list_state_search_results.selected();
                let ids = if self.is_podcast {
                    &self.ids_library_pod_search
                } else {
                    &self.ids_search_book
                };
                one(index.and_then(|index| ids.get(index)))
            }

            AppView::Series => self
                .selected_series()
                .map(|series| {
                    series
                        .books
                        .iter()
                        .take(cover::SHELF_MAX)
                        .map(|book| book.id.clone())
                        .collect()
                })
                .unwrap_or_default(),

            AppView::SeriesBook => one(self.selected_series_book().map(|book| &book.id)),

            AppView::Lists => self
                .selected_list()
                .map(|list| {
                    list.entries
                        .iter()
                        .take(cover::SHELF_MAX)
                        .map(|entry| entry.id.clone())
                        .collect()
                })
                .unwrap_or_default(),

            AppView::ListEntries => one(self.selected_list_entry().map(|entry| &entry.id)),

            AppView::Reader
            | AppView::Stats
            | AppView::Sessions
            | AppView::SortFilter
            | AppView::Chapters
            | AppView::Bookmarks
            | AppView::Queue
            | AppView::NewPodcast
            | AppView::Authors
            | AppView::Settings
            | AppView::SettingsAccount
            | AppView::SettingsLibrary
            | AppView::SettingsAbout
            | AppView::SettingsUpdateUninstall => Vec::new(),
        }
    }

    /// Draws the panel of the covers.
    ///
    /// The function draws nothing when the screen is too narrow, because
    /// `split_for_covers` then gives no panel.
    fn render_covers(&mut self, panel: Option<Rect>, buf: &mut Buffer) {
        let Some(panel) = panel else {
            return;
        };

        let playback = self.player.state();
        let playing = if playback.status == PlaybackStatus::Stopped || playback.item_id.is_empty() {
            None
        } else {
            Some(playback.item_id.clone())
        };

        // The selection needs no second cover when it is the media that
        // plays. The panel then shows one cover, and that cover is large.
        let selected: Vec<String> = self
            .cover_ids()
            .into_iter()
            .filter(|id| !id.is_empty() && Some(id.as_str()) != playing.as_deref())
            .take(cover::SHELF_MAX)
            .collect();

        // The large cover takes the form of its own picture. Therefore a cover
        // that is higher than it is wide takes the whole height of the panel.
        // See T-50.
        let large = playing
            .as_deref()
            .or(selected.first().map(|id| id.as_str()))
            .and_then(|id| self.covers.form_of(id));

        let plan = cover::plan_covers(
            panel,
            cover::picker().font_size(),
            playing.is_some(),
            selected.len(),
            large,
        );

        let api = std::sync::Arc::clone(&self.api);

        if let (Some(area), Some(id)) = (plan.playing, playing.as_deref()) {
            if let Some(picture) = self.covers.picture(&api, id) {
                StatefulImage::default().render(area, buf, picture);
            }
        }

        for (area, id) in plan.shelf.iter().zip(selected.iter()) {
            if let Some(picture) = self.covers.picture(&api, id) {
                StatefulImage::default().render(*area, buf, picture);
            }
        }
    }
}

/// The reader of an ebook. See T-10.
impl App {
    /// Draws the reader. The whole screen belongs to the book, because a book
    /// needs every line that the terminal has.
    fn render_reader(&mut self, area: Rect, buf: &mut Buffer) {
        // The task that opens a book puts it in a place of the process. The
        // screen takes it here.
        self.take_the_book();

        // A view with no book keeps two lines for the footer. A screen that
        // names no key looks like a program that stopped: the user of
        // 2026-08-11 had to start the program again. See T-52.
        if self.reader.is_none() {
            let [header_area, main_area, footer_area] = Layout::vertical([
                Constraint::Length(2),
                Constraint::Fill(1),
                Constraint::Length(2),
            ])
            .areas(area);

            let message = self
                .reader_message
                .clone()
                .unwrap_or_else(|| "No book is open.".to_string());

            self.render_header(header_area, buf);
            App::render_footer(footer_area, buf, crate::ui::keys::FOOTER_OF_A_FAULT);

            Paragraph::new(message)
                .centered()
                .wrap(Wrap { trim: true })
                .block(
                    Block::default()
                        .borders(Borders::TOP)
                        .title("The reader of the ebook")
                        .title_alignment(Alignment::Center),
                )
                .render(main_area, buf);

            return;
        }

        let [header_area, main_area] =
            Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]).areas(area);

        self.render_header(header_area, buf);

        let Some(reader) = self.reader.as_mut() else {
            return;
        };

        // The task of the render sends the lines. The screen takes them here,
        // and it never waits for them.
        reader.take_the_answer();

        crate::ui::reader_tui::render(reader, main_area, buf);
    }
}

/// The statistics of the user. See T-24.
impl App {
    /// AppView::Stats rendering
    fn render_stats(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(2),
        ])
        .areas(area);

        self.render_header(header_area, buf);
        App::render_footer(
            footer_area,
            buf,
            "j/k: move  T: ask the server again  h: back  ?: every key  Q: quit",
        );

        // The task of the request writes the answer, and the screen takes it
        // here. The screen never waits for the server.
        let state = crate::logic::stats::state();

        self.stats_scroll_max =
            crate::ui::stats_tui::render(&state, self.stats_scroll, main_area, buf);

        // A screen that becomes higher shows more lines. The first line then
        // stands after the last one, and the user sees nothing.
        if self.stats_scroll > self.stats_scroll_max {
            self.stats_scroll = self.stats_scroll_max;
        }
    }
}

/// Every session of the user, with pages. See T-24.
impl App {
    /// AppView::Sessions rendering
    fn render_sessions(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(2),
        ])
        .areas(area);

        self.render_header(header_area, buf);
        App::render_footer(
            footer_area,
            buf,
            "j/k: move  W: ask the server again  h: back  ?: every key  Q: quit",
        );

        let state = crate::logic::sessions_view::state();

        self.sessions_scroll_max =
            crate::ui::sessions_tui::render(&state, self.sessions_scroll, main_area, buf);

        // A screen that becomes higher shows more lines. The first line then
        // stands after the last one, and the user sees nothing.
        if self.sessions_scroll > self.sessions_scroll_max {
            self.sessions_scroll = self.sessions_scroll_max;
        }
    }
}

/// The authors of the library. See T-24.
impl App {
    /// AppView::Authors rendering
    fn render_authors(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, item_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(4),
            Constraint::Length(2),
        ])
        .areas(area);

        let state = crate::logic::authors::state();

        let (title, lines) = match &state {
            crate::logic::authors::State::Ready(all) if all.is_empty() => {
                ("This library has no author.".to_string(), Vec::new())
            }
            crate::logic::authors::State::Ready(all) => (
                format!("The authors [{} items]", all.len()),
                crate::api::libraries::get_authors::lines(all),
            ),
            crate::logic::authors::State::Waiting => {
                ("The program asks the server…".to_string(), Vec::new())
            }
            crate::logic::authors::State::Fault(text) => {
                (format!("The server gave no author: {}", text), Vec::new())
            }
            crate::logic::authors::State::Nothing => {
                ("The program asks the server…".to_string(), Vec::new())
            }
        };

        self.render_header(header_area, buf);
        App::render_footer(
            footer_area,
            buf,
            &crate::ui::keys::footer_with("the books of this author", None),
        );
        self.render_list(
            main_area,
            buf,
            &title,
            &lines,
            &mut self.list_state_authors.clone(),
        );

        let all = crate::logic::authors::authors();

        if let Some(one) = self
            .list_state_authors
            .selected()
            .and_then(|index| all.get(index))
        {
            Paragraph::new(crate::api::libraries::get_authors::description_of(one))
                .scroll((self.scroll_offset, 0))
                .wrap(Wrap { trim: true })
                .render(item_area, buf);
        }
    }
}

/// A new podcast. See T-24.
impl App {
    /// AppView::NewPodcast rendering
    fn render_new_podcast(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, item_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(4),
            Constraint::Length(2),
        ])
        .areas(area);

        let state = crate::logic::new_podcast::state();

        let (title, lines) = match &state {
            crate::logic::new_podcast::State::Ready(all) if all.is_empty() => (
                "The server found no podcast. Press A and write other words.".to_string(),
                Vec::new(),
            ),
            crate::logic::new_podcast::State::Ready(all) => (
                format!("A new podcast [{} answers]", all.len()),
                crate::api::podcasts::lines(all),
            ),
            crate::logic::new_podcast::State::Waiting => {
                ("The server looks for the podcast…".to_string(), Vec::new())
            }
            crate::logic::new_podcast::State::Fault(text) => {
                (format!("The server found nothing: {}", text), Vec::new())
            }
            crate::logic::new_podcast::State::Nothing => (
                "Press A and write the name of a podcast.".to_string(),
                Vec::new(),
            ),
        };

        self.render_header(header_area, buf);
        App::render_footer(
            footer_area,
            buf,
            "j/k: move  l: add the podcast  A: other words  h: back  ?: every key  Q: quit",
        );
        self.render_list(
            main_area,
            buf,
            &title,
            &lines,
            &mut self.list_state_new_podcast.clone(),
        );

        // The description of the answer that the user selected.
        let all = crate::logic::new_podcast::found();

        if let Some(one) = self
            .list_state_new_podcast
            .selected()
            .and_then(|index| all.get(index))
        {
            let text = if one.description_plain.is_empty() {
                one.feed_url.clone()
            } else {
                one.description_plain.clone()
            };

            Paragraph::new(text)
                .scroll((self.scroll_offset, 0))
                .wrap(Wrap { trim: true })
                .render(item_area, buf);
        }
    }
}

/// The bookmarks of one media. See T-24.
impl App {
    /// AppView::Bookmarks rendering
    fn render_bookmarks(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(2),
        ])
        .areas(area);

        // A write and a remove forget the answer. The view then asks the
        // server again, and the new list comes at the frame after it.
        self.take_the_bookmarks_again();

        let state = crate::logic::bookmarks::state();

        let (title, lines) = match &state {
            crate::logic::bookmarks::State::Ready(all) if all.is_empty() => (
                "This media has no bookmark. Press b while it plays.".to_string(),
                Vec::new(),
            ),
            crate::logic::bookmarks::State::Ready(all) => (
                format!("The bookmarks [{} items]", all.len()),
                crate::api::me::bookmarks::lines(all),
            ),
            crate::logic::bookmarks::State::Waiting => {
                ("The program asks the server…".to_string(), Vec::new())
            }
            crate::logic::bookmarks::State::Fault(text) => {
                (format!("The server gave no bookmark: {}", text), Vec::new())
            }
            crate::logic::bookmarks::State::Nothing => {
                ("The program asks the server…".to_string(), Vec::new())
            }
        };

        self.render_header(header_area, buf);
        App::render_footer(
            footer_area,
            buf,
            &crate::ui::keys::footer_with("go to the place", Some("remove the bookmark")),
        );
        self.render_list(
            main_area,
            buf,
            &title,
            &lines,
            &mut self.list_state_bookmarks.clone(),
        );
    }
}

/// The media that wait in the queue. See T-24.
impl App {
    /// AppView::Queue rendering
    fn render_queue(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(2),
        ])
        .areas(area);

        let queue = crate::logic::queue::snapshot();
        let lines = queue.lines();

        let title = if lines.is_empty() {
            "The queue is empty. Press n on a media to put it in the queue.".to_string()
        } else {
            format!("The queue [{} items]", lines.len())
        };

        self.render_header(header_area, buf);
        App::render_footer(
            footer_area,
            buf,
            &crate::ui::keys::footer_with("play it now", Some("take it out")),
        );
        self.render_list(
            main_area,
            buf,
            &title,
            &lines,
            &mut self.list_state_queue.clone(),
        );
    }
}

/// The chapters of the media that plays. See T-24.
impl App {
    /// AppView::Chapters rendering
    fn render_chapters(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(2),
        ])
        .areas(area);

        let state = self.player.state();
        let lines = crate::logic::chapters::lines(&state.chapters, state.position);

        // The media stopped while the view was open. The list is then empty,
        // and the screen says why.
        let title = if lines.is_empty() {
            "No media plays now. Press h to go back.".to_string()
        } else {
            format!(
                "The chapters of \"{}\" [{} items]",
                state.title,
                lines.len()
            )
        };

        self.render_header(header_area, buf);
        App::render_footer(
            footer_area,
            buf,
            &crate::ui::keys::footer_with("go to the chapter", None),
        );
        self.render_list(
            main_area,
            buf,
            &title,
            &lines,
            &mut self.list_state_chapters.clone(),
        );
    }
}

/// The sequence and the filter of the library. See T-24.
impl App {
    /// AppView::SortFilter rendering
    fn render_sort_filter(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(2),
        ])
        .areas(area);

        let rows = self.sort_filter_rows();

        let lines: Vec<String> = rows
            .iter()
            .map(|row| {
                crate::logic::sort_filter::line_of(
                    row,
                    &self.library_sort,
                    self.library_desc,
                    &self.library_filter,
                )
            })
            .collect();

        // The answer of the server adds lines. A selection that stands after
        // the last line must come back to a line of the user.
        let flags: Vec<bool> = rows.iter().map(|row| row.is_a_line_of_the_user()).collect();
        let selected = self.list_state_sort_filter.selected().unwrap_or(0);

        if flags.get(selected) != Some(&true) {
            self.list_state_sort_filter
                .select(crate::logic::list_moves::first(&flags));
        }

        let title = format!(
            "The sequence and the filter — {}{}",
            crate::logic::sort_filter::label_of(&self.library_sort, self.is_podcast),
            if self.library_desc {
                ", the largest first"
            } else {
                ""
            }
        );

        self.render_header(header_area, buf);
        App::render_footer(footer_area, buf, crate::ui::keys::FOOTER_OF_A_LIST);
        self.render_list(
            main_area,
            buf,
            &title,
            &lines,
            &mut self.list_state_sort_filter.clone(),
        );
    }
}

/// Rendering logic
impl App {
    /// Draws every key of the program. The key `?` opens this view. See T-49.
    ///
    /// The footer of a view names the keys of the work of that view only. A
    /// footer with every key needed two lines of more than 300 characters, and
    /// a terminal of 80 columns showed a part of them only.
    fn render_keys(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(2),
        ])
        .areas(area);

        let lines = crate::ui::keys::lines();

        self.render_header(header_area, buf);
        App::render_footer(footer_area, buf, crate::ui::keys::FOOTER_OF_THE_KEYS);
        self.render_list(
            main_area,
            buf,
            "Every key of the program",
            &lines,
            &mut self.list_state_keys.clone(),
        );
    }

    /// AppView::Home rendering
    fn render_home(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, _player_area, _refresh_area, footer_area] =
            Layout::vertical([
                Constraint::Length(2),
                Constraint::Fill(1),
                Constraint::Length(6),
                Constraint::Length(1),
                Constraint::Length(2),
            ])
            .areas(area);

        // The panel of the covers stands at the right of the list and of the
        // description. It is always visible. See T-23.
        let (main_area, cover_panel) =
            cover::split_for_covers(main_area, area.width, cover::picker().font_size());
        self.render_covers(cover_panel, buf);

        let [list_area, item_area1, item_area2] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Fill(1),
        ])
        .areas(main_area);

        // Every line starts with a mark: the media that plays, a media that
        // the user finished, or the part that the user heard. See T-44.
        let lines = self.home_lines();
        let count = self
            .home_rows
            .iter()
            .filter(|row| row.is_a_line_of_the_user())
            .count();
        let render_list_title = format!("Home [{} items]", count);

        // A library of podcasts has no series and no ebook. The footer of
        // that library must not name a key that does nothing.
        let text_render_footer = if self.is_podcast {
            crate::ui::keys::FOOTER_OF_A_LIBRARY_OF_PODCASTS
        } else {
            crate::ui::keys::FOOTER_OF_A_LIBRARY_OF_BOOKS
        };

        self.render_header(header_area, buf);
        App::render_footer(footer_area, buf, text_render_footer);
        self.render_list(
            list_area,
            buf,
            &render_list_title,
            &lines,
            &mut self.list_state_cnt_list.clone(),
        );
        if !lines.is_empty() {
            self.render_info_home(item_area1, buf);
            self.render_desc_home(item_area2, buf);
        }
    }

    /// AppView::Library rendering
    fn render_library(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, _player_area, _refresh_area, footer_area] =
            Layout::vertical([
                Constraint::Length(2),
                Constraint::Fill(1),
                Constraint::Length(6),
                Constraint::Length(1),
                Constraint::Length(2),
            ])
            .areas(area);

        // The panel of the covers stands at the right of the list and of the
        // description. It is always visible. See T-23.
        let (main_area, cover_panel) =
            cover::split_for_covers(main_area, area.width, cover::picker().font_size());
        self.render_covers(cover_panel, buf);

        let [list_area, item_area1, item_area2] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Fill(1),
        ])
        .areas(main_area);

        // Every book of a series gives one line. See T-22.
        let lines = self.library_lines();
        // The title says the sequence and the filter, because a user who
        // does not see every item must know why. See T-24.
        let render_list_title = format!(
            "Library [{} items]{}{}",
            lines.len(),
            if self.library_sort.is_empty() {
                String::new()
            } else {
                format!(
                    " — {}{}",
                    crate::logic::sort_filter::label_of(&self.library_sort, self.is_podcast),
                    if self.library_desc {
                        ", the largest first"
                    } else {
                        ""
                    }
                )
            },
            if self.library_filter.is_empty() {
                ""
            } else {
                " — a filter is on (f)"
            }
        );

        let mut _text_render_footer = "";
        if self.is_podcast {
            _text_render_footer = crate::ui::keys::FOOTER_OF_A_LIBRARY_OF_PODCASTS
        } else {
            _text_render_footer = crate::ui::keys::FOOTER_OF_A_LIBRARY_OF_BOOKS;
        }

        self.render_header(header_area, buf);
        App::render_footer(footer_area, buf, _text_render_footer);
        self.render_list(
            list_area,
            buf,
            &render_list_title,
            &lines,
            &mut self.list_state_library.clone(),
        );
        if !lines.is_empty() {
            self.render_info_library(item_area1, buf);
            self.render_desc_library(item_area2, buf);
        }
    }

    /// AppView::Series rendering: the list of the series of the library.
    fn render_series(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, _player_area, _refresh_area, footer_area] =
            Layout::vertical([
                Constraint::Length(2),
                Constraint::Fill(1),
                Constraint::Length(6),
                Constraint::Length(1),
                Constraint::Length(2),
            ])
            .areas(area);

        // The panel of the covers stands at the right of the list and of the
        // description. It is always visible. See T-23.
        let (main_area, cover_panel) =
            cover::split_for_covers(main_area, area.width, cover::picker().font_size());
        self.render_covers(cover_panel, buf);

        let [list_area, item_area1, item_area2] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Fill(1),
        ])
        .areas(main_area);

        let text_render_footer = crate::ui::keys::FOOTER_OF_A_LIST;

        self.render_header(header_area, buf);
        App::render_footer(footer_area, buf, text_render_footer);

        if self.series.is_empty() {
            Paragraph::new("This library has no series.\nPress 'h' to go back.")
                .centered()
                .block(
                    Block::new()
                        .borders(Borders::TOP)
                        .border_style(Style::new().fg(Color::DarkGray)),
                )
                .render(main_area, buf);
            return;
        }

        let lines: Vec<String> = self.series.iter().map(|series| series.line()).collect();
        let render_list_title = format!("Series [{} items]", self.series.len());

        self.render_list(
            list_area,
            buf,
            &render_list_title,
            &lines,
            &mut self.list_state_series.clone(),
        );

        if let Some(series) = self.selected_series() {
            let books = series.books.len();
            let seconds: f64 = series.books.iter().map(|book| book.duration).sum();

            Paragraph::new(format!(
                "{} - {} book(s) - Duration: {}",
                series.name,
                books,
                convert_seconds(vec![seconds])
                    .first()
                    .cloned()
                    .unwrap_or_default(),
            ))
            .left_aligned()
            .render(item_area1, buf);

            Paragraph::new(series.description_for_the_screen())
                .scroll((self.scroll_offset, 0))
                .wrap(Wrap { trim: true })
                .render(item_area2, buf);
        }
    }

    /// AppView::SeriesBook rendering: the books of one series.
    fn render_series_book(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, _player_area, _refresh_area, footer_area] =
            Layout::vertical([
                Constraint::Length(2),
                Constraint::Fill(1),
                Constraint::Length(6),
                Constraint::Length(1),
                Constraint::Length(2),
            ])
            .areas(area);

        // The panel of the covers stands at the right of the list and of the
        // description. It is always visible. See T-23.
        let (main_area, cover_panel) =
            cover::split_for_covers(main_area, area.width, cover::picker().font_size());
        self.render_covers(cover_panel, buf);

        let [list_area, item_area1, item_area2] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Fill(1),
        ])
        .areas(main_area);

        let text_render_footer = crate::ui::keys::FOOTER_OF_A_LIST_OF_MEDIA;

        self.render_header(header_area, buf);
        App::render_footer(footer_area, buf, text_render_footer);

        let Some(series) = self.selected_series() else {
            return;
        };

        let name = series.name.clone();
        let lines: Vec<String> = series.books.iter().map(|book| book.line()).collect();
        let render_list_title = format!("{} [{} items]", name, lines.len());

        self.render_list(
            list_area,
            buf,
            &render_list_title,
            &lines,
            &mut self.list_state_series_book.clone(),
        );

        if let Some(book) = self.selected_series_book() {
            let is_offline = crate::db::crud::get_download(&book.id, &self.username).is_some();

            Paragraph::new(format!(
                "Author: {} - Duration: {}{}",
                book.author,
                convert_seconds(vec![book.duration])
                    .first()
                    .cloned()
                    .unwrap_or_default(),
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

    /// AppView::Lists rendering: the collections and the playlists.
    fn render_lists(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, _player_area, _refresh_area, footer_area] =
            Layout::vertical([
                Constraint::Length(2),
                Constraint::Fill(1),
                Constraint::Length(6),
                Constraint::Length(1),
                Constraint::Length(2),
            ])
            .areas(area);

        // The panel of the covers stands at the right of the list and of the
        // description. It is always visible. See T-23.
        let (main_area, cover_panel) =
            cover::split_for_covers(main_area, area.width, cover::picker().font_size());
        self.render_covers(cover_panel, buf);

        let [list_area, item_area1, item_area2] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Fill(1),
        ])
        .areas(main_area);

        let text_render_footer = crate::ui::keys::FOOTER_OF_A_LIST;

        self.render_header(header_area, buf);
        App::render_footer(footer_area, buf, text_render_footer);

        if self.lists.is_empty() {
            Paragraph::new(
                "This library has no collection and no playlist.\nPress 'h' to go back.",
            )
            .centered()
            .block(
                Block::new()
                    .borders(Borders::TOP)
                    .border_style(Style::new().fg(Color::DarkGray)),
            )
            .render(main_area, buf);
            return;
        }

        let lines: Vec<String> = self.lists.iter().map(|list| list.line()).collect();
        let render_list_title = format!("Collections and playlists [{} items]", self.lists.len());

        self.render_list(
            list_area,
            buf,
            &render_list_title,
            &lines,
            &mut self.list_state_lists.clone(),
        );

        if let Some(list) = self.selected_list() {
            let seconds: f64 = list.entries.iter().map(|entry| entry.duration).sum();

            Paragraph::new(format!(
                "{} - {} item(s) - Duration: {}",
                list.kind.name(),
                list.entries.len(),
                convert_seconds(vec![seconds])
                    .first()
                    .cloned()
                    .unwrap_or_default(),
            ))
            .left_aligned()
            .render(item_area1, buf);

            Paragraph::new(list.description.clone())
                .scroll((self.scroll_offset, 0))
                .wrap(Wrap { trim: true })
                .render(item_area2, buf);
        }
    }

    /// AppView::ListEntries rendering: the media of one collection or of one
    /// playlist.
    fn render_list_entries(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, _player_area, _refresh_area, footer_area] =
            Layout::vertical([
                Constraint::Length(2),
                Constraint::Fill(1),
                Constraint::Length(6),
                Constraint::Length(1),
                Constraint::Length(2),
            ])
            .areas(area);

        // The panel of the covers stands at the right of the list and of the
        // description. It is always visible. See T-23.
        let (main_area, cover_panel) =
            cover::split_for_covers(main_area, area.width, cover::picker().font_size());
        self.render_covers(cover_panel, buf);

        let [list_area, item_area1, item_area2] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Fill(1),
        ])
        .areas(main_area);

        let text_render_footer = crate::ui::keys::FOOTER_OF_A_LIST_OF_MEDIA;

        self.render_header(header_area, buf);
        App::render_footer(footer_area, buf, text_render_footer);

        let Some(list) = self.selected_list() else {
            return;
        };

        let lines: Vec<String> = list.entries.iter().map(|entry| entry.line()).collect();
        let render_list_title = format!("{} [{} items]", list.name.clone(), lines.len());

        self.render_list(
            list_area,
            buf,
            &render_list_title,
            &lines,
            &mut self.list_state_list_entries.clone(),
        );

        if let Some(entry) = self.selected_list_entry() {
            // The download of an episode has the identity of the episode.
            let key = entry.episode_id.clone().unwrap_or_else(|| entry.id.clone());
            let is_offline = crate::db::crud::get_download(&key, &self.username).is_some();

            Paragraph::new(format!(
                "{} - Author: {} - Duration: {}{}",
                if entry.is_episode() {
                    "Episode"
                } else {
                    "Book"
                },
                entry.author,
                convert_seconds(vec![entry.duration])
                    .first()
                    .cloned()
                    .unwrap_or_default(),
                if is_offline { " - [Downloaded]" } else { "" },
            ))
            .left_aligned()
            .render(item_area1, buf);

            Paragraph::new(entry.description.clone())
                .scroll((self.scroll_offset, 0))
                .wrap(Wrap { trim: true })
                .render(item_area2, buf);
        }
    }

    /// AppView::Settings rendering
    fn render_settings(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, _player_area, _refresh_area, footer_area] =
            Layout::vertical([
                Constraint::Length(2),
                Constraint::Fill(1),
                Constraint::Length(6),
                Constraint::Length(1),
                Constraint::Length(2),
            ])
            .areas(area);

        let [list_area, item_area1, item_area2] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Fill(1),
        ])
        .areas(main_area);

        let render_list_title = "Settings";

        // Every line of the settings takes the same keys.
        let _text_render_footer = crate::ui::keys::FOOTER_OF_A_LIST;

        self.render_header(header_area, buf);
        App::render_footer(footer_area, buf, _text_render_footer);
        self.render_list(
            list_area,
            buf,
            render_list_title,
            &self.settings.clone(),
            &mut self.list_state_settings.clone(),
        );
        self.render_info_settings(item_area1, buf, &self.list_state_settings.clone());
        self.render_desc_settings(item_area2, buf, &self.list_state_settings.clone());
    }

    /// AppView::SettingsAccount rendering
    fn render_settings_account(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, _player_area, _refresh_area, footer_area] =
            Layout::vertical([
                Constraint::Length(2),
                Constraint::Fill(1),
                Constraint::Length(6),
                Constraint::Length(1),
                Constraint::Length(2),
            ])
            .areas(area);

        let [list_area, _item_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Fill(1)]).areas(main_area);

        let render_list_title = "Accounts — l: log out of the account";
        let text_render_footer =
            "h: back, l/→: log out of this account (the program forgets its token),\n Tab: home, R: refresh, Q/Esc: quit.";

        self.render_header(header_area, buf);
        App::render_footer(footer_area, buf, text_render_footer);
        self.render_list(
            list_area,
            buf,
            render_list_title,
            &self.all_usernames.clone(),
            &mut self.list_state_settings_account.clone(),
        );
        //self.render_selected_item(item_area, buf, &self.titles_library.clone(), self.auth_names_library.clone());
    }

    /// AppView::SettingsLibrary rendering
    fn render_settings_library(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, _player_area, _refresh_area, footer_area] =
            Layout::vertical([
                Constraint::Length(2),
                Constraint::Fill(1),
                Constraint::Length(6),
                Constraint::Length(1),
                Constraint::Length(2),
            ])
            .areas(area);

        let [list_area, item_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Fill(1)]).areas(main_area);

        let items_number = self.libraries_names.len();
        let render_list_title = format!("Settings Library [{} items]", items_number);

        let text_render_footer =
            "h: back, l/→: change library,\n Tab: home, R: refresh, Q/Esc: quit.";

        self.render_header(header_area, buf);
        App::render_footer(footer_area, buf, text_render_footer);
        self.render_list(
            list_area,
            buf,
            &render_list_title,
            &self.libraries_names.clone(),
            &mut self.list_state_settings_library.clone(),
        );
        self.render_info_settings_library(
            item_area,
            buf,
            &self.list_state_settings_library.clone(),
        );
    }

    /// AppView::SearchBook rendering
    fn render_search_book(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, _player_area, _refresh_area, footer_area] =
            Layout::vertical([
                Constraint::Length(2),
                Constraint::Fill(1),
                Constraint::Length(6),
                Constraint::Length(1),
                Constraint::Length(2),
            ])
            .areas(area);

        // The panel of the covers stands at the right of the list and of the
        // description. It is always visible. See T-23.
        let (main_area, cover_panel) =
            cover::split_for_covers(main_area, area.width, cover::picker().font_size());
        self.render_covers(cover_panel, buf);

        let [list_area, item_area1, item_area2] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Fill(1),
        ])
        .areas(main_area);

        let from_the_server = crate::logic::search::from_the_server::answer_for(&self.search_query);

        // The title tells the user where the answer comes from, and it names
        // the author or the narrator that the server found. A search for the
        // name of an author gives no book when the library holds no book of
        // that name, and the user must not think that the program did
        // nothing. See T-24.
        let render_list_title = match &from_the_server {
            Some(answer) if !answer.names.is_empty() => format!(
                "Search result [the server also found: {}]",
                answer.names.join(", ")
            ),
            Some(_) => "Search result [from the server]".to_string(),
            None => "Search result [the titles of this program]".to_string(),
        };
        let render_list_title = render_list_title.as_str();

        let mut _text_render_footer = "";
        if self.is_podcast {
            _text_render_footer = crate::ui::keys::FOOTER_OF_A_LIBRARY_OF_PODCASTS;
        } else {
            _text_render_footer = crate::ui::keys::FOOTER_OF_A_LIBRARY_OF_BOOKS;
        }

        if self.search_mode {
            if let Ok(query) = self.search_active() {
                self.search_query = query.to_string();
                self.search_mode = false;
            }
        }

        // init variables for search result (search by a book by title)
        // The answer of the server comes first, and the titles of this
        // program come while the user waits for it.
        //
        // The server finds an author, a series, a narrator, a tag, and a
        // genre. This program looks in the titles only, therefore a user who
        // writes the name of an author finds nothing here. See T-24.
        let idx_and_titles: Vec<(usize, String)> = match &from_the_server {
            Some(answer) => answer
                .items
                .iter()
                .filter_map(|id| self.ids_library.iter().position(|one| one == id))
                .filter_map(|index| {
                    self.titles_library
                        .get(index)
                        .map(|title| (index, title.clone()))
                })
                .collect(),
            None => self
                .titles_library
                .iter()
                .enumerate()
                .filter(|(_, x)| x.to_lowercase().contains(&self.search_query.to_lowercase()))
                .map(|(index, title)| (index, title.clone()))
                .collect(),
        };

        let mut titles_search_book_or_pod: Vec<String> = Vec::new();
        let mut index_to_keep: Vec<usize> = Vec::new();
        for (index, title) in idx_and_titles {
            titles_search_book_or_pod.push(title.to_string());
            index_to_keep.push(index)
        }

        let titles_search_book_or_pod: &[String] = &titles_search_book_or_pod;

        // apply search filtering for book
        self.ids_search_book = self
            .ids_library
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.auth_names_pod_search_book = self
            .auth_names_library_pod
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.auth_names_search_book = self
            .auth_names_library
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.published_year_library_search_book = self
            .published_year_library
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.desc_library_search_book = self
            .desc_library
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.duration_library_search_book = self
            .duration_library
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
        self.all_titles_pod_ep_search = self
            .all_titles_pod_ep
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.all_ids_pod_ep_search = self
            .all_ids_pod_ep
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.all_subtitles_pod_ep_search = self
            .all_subtitles_pod_ep
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.all_seasons_pod_ep_search = self
            .all_seasons_pod_ep
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.all_episodes_pod_ep_search = self
            .all_episodes_pod_ep
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.all_authors_pod_ep_search = self
            .all_authors_pod_ep
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.all_descs_pod_ep_search = self
            .all_descs_pod_ep
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.all_titles_pod_search = self
            .all_titles_pod
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.all_durations_pod_ep_search = self
            .all_durations_pod_ep
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.ids_library_pod_search = self
            .ids_library
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();

        self.render_header(header_area, buf);
        App::render_footer(footer_area, buf, _text_render_footer);
        self.render_list(
            list_area,
            buf,
            render_list_title,
            titles_search_book_or_pod,
            &mut self.list_state_search_results.clone(),
        );
        if !titles_search_book_or_pod.is_empty() {
            self.render_info_search_book(item_area1, buf, &self.list_state_search_results.clone());
            self.render_desc_search_book(item_area2, buf, &self.list_state_search_results.clone());
        }
    }

    /// AppView::PodcastEpisode
    fn render_pod_ep(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, _player_area, _refresh_area, footer_area] =
            Layout::vertical([
                Constraint::Length(2),
                Constraint::Fill(1),
                Constraint::Length(6),
                Constraint::Length(1),
                Constraint::Length(2),
            ])
            .areas(area);

        // The panel of the covers stands at the right of the list and of the
        // description. It is always visible. See T-23.
        let (main_area, cover_panel) =
            cover::split_for_covers(main_area, area.width, cover::picker().font_size());
        self.render_covers(cover_panel, buf);

        let [list_area, item_area1, item_area2] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Fill(1),
        ])
        .areas(main_area);

        let text_render_footer = crate::ui::keys::FOOTER_OF_A_LIST_OF_MEDIA;

        self.render_header(header_area, buf);
        App::render_footer(footer_area, buf, text_render_footer);
        let no_episodes_message = "No episodes found for this podcast.\nPress 'h' to go back.";

        if self.is_from_search_pod {
            if self.titles_pod_ep_search.is_empty() {
                log::warn!("render_pod_ep (search): No episodes found.");
                Paragraph::new(no_episodes_message)
                    .centered()
                    .block(
                        Block::new()
                            .borders(Borders::TOP)
                            .border_style(Style::new().fg(Color::DarkGray)),
                    )
                    .render(main_area, buf);
            } else {
                let items_number = self.titles_pod_ep_search.len();
                let render_list_title = format!("Episodes [{} items]", items_number);
                // Only render list/info/desc if episodes exist
                self.render_list(
                    list_area,
                    buf,
                    &render_list_title,
                    &self.titles_pod_ep_search.clone(),
                    &mut self.list_state_pod_ep.clone(),
                );
                self.render_info_pod_ep_search(item_area1, buf, &self.list_state_pod_ep.clone());
                self.render_desc_pod_ep_search(item_area2, buf, &self.list_state_pod_ep.clone());
            }
        } else {
            if self.titles_pod_ep.is_empty() {
                log::warn!("render_pod_ep (library): No episodes found.");
                Paragraph::new(no_episodes_message)
                    .centered()
                    .block(
                        Block::new()
                            .borders(Borders::TOP)
                            .border_style(Style::new().fg(Color::DarkGray)),
                    )
                    .render(main_area, buf);
            } else {
                let items_number = self.titles_pod_ep.len();
                let render_list_title = format!("Episodes [{} items]", items_number);
                // Only render list/info/desc if episodes exist
                self.render_list(
                    list_area,
                    buf,
                    &render_list_title,
                    &self.titles_pod_ep.clone(),
                    &mut self.list_state_pod_ep.clone(),
                );
                self.render_info_pod_ep(item_area1, buf, &self.list_state_pod_ep.clone());
                self.render_desc_pod_ep(item_area2, buf, &self.list_state_pod_ep.clone());
            }
        }
    }

    // General functions for rendering

    /// Draws the two lines at the top of the screen.
    ///
    /// The offline mode says that the server does not answer, and it gives the
    /// number of positions that wait for the server. See T-25.
    fn render_header(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(self.lib_name_type.clone())
            .bold()
            .centered()
            .render(area, buf);

        let connection = if self.is_offline {
            format!(
                "📴 Offline as {}\n🔗 {} does not answer",
                self.username, self.server_address_pretty
            )
        } else {
            format!(
                "👋 Connected as {}\n🔗 {}",
                self.username, self.server_address_pretty
            )
        };

        // The audio engine did not start. The user reads the library, and no
        // media plays. See T-46.
        let connection = match &self.audio_fault {
            Some(_) => format!("{}\n🔇 No sound device: no media can play", connection),
            None => connection,
        };

        Paragraph::new(connection)
            .not_bold()
            .left_aligned()
            .render(area, buf);

        let notice = if self.is_offline {
            let waiting = match self.waiting_progress {
                0 => String::new(),
                1 => " - 1 position waits".to_string(),
                count => format!(" - {} positions wait", count),
            };

            format!("R: try the server again{}", waiting)
        } else if crate::logic::live::the_lists_are_old() {
            // A different client changed the metadata of an item. That value
            // stands in many lists, therefore one line cannot hold the
            // correction. See T-47.
            "R: the server has newer data".to_string()
        } else {
            self.update_msg.clone()
        };

        Paragraph::new(format!("🦜 Toutui v{}\n {}", VERSION, notice))
            .right_aligned()
            .render(area, buf);
    }

    fn render_footer(area: Rect, buf: &mut Buffer, text_render_footer: &str) {
        Paragraph::new(text_render_footer)
            .centered()
            .render(area, buf);
    }

    fn render_list(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        render_list_title: &str,
        render_list_items: &[String],
        list_state: &mut ListState,
    ) {
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
            .fg(Color::Rgb(
                fg_color_header[0],
                fg_color_header[1],
                fg_color_header[2],
            ))
            .bg(Color::Rgb(
                bg_color_header[0],
                bg_color_header[1],
                bg_color_header[2],
            ));

        let block = Block::new()
            .title(Line::raw(render_list_title.to_string()).centered())
            .borders(Borders::TOP)
            .border_style(header_style)
            .bg(Color::Rgb(
                bg_color_block[0],
                bg_color_block[1],
                bg_color_block[2],
            ));

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
    fn render_info_home(&self, area: Rect, buf: &mut Buffer) {
        let duration_cnt_list_conv = convert_seconds(self.duration_cnt_list.clone());

        // A line of a series tells the number of the books and the whole
        // length, in the same way as the Library view. See T-22.
        if let Some(series) = self.selected_home_series() {
            let seconds: f64 = series.books.iter().map(|book| book.duration).sum();

            Paragraph::new(format!(
                "{} - {} book(s) - Duration: {}",
                series.name,
                series.books.len(),
                convert_seconds(vec![seconds])
                    .first()
                    .cloned()
                    .unwrap_or_default(),
            ))
            .left_aligned()
            .render(area, buf);
            return;
        }

        if let Some(selected) = self.selected_home_item() {
            if self.is_podcast {
                let is_offline = self
                    .ids_ep_cnt_list
                    .get(selected)
                    .map(|id| crate::db::crud::get_download(id, &self.username).is_some())
                    .unwrap_or(false);

                Paragraph::new(format!(
                    "[{}] - Author: {} - Episode: {} - Duration: {}{}",
                    at(&self.titles_pod_cnt_list, selected),
                    at(&self.authors_pod_cnt_list, selected),
                    at(&self.nums_ep_pod_cnt_list, selected),
                    at(&self.durations_pod_cnt_list, selected),
                    if is_offline { " - [Downloaded]" } else { "" },
                ))
                .left_aligned()
                .render(area, buf);
            } else {
                let is_offline = crate::db::crud::get_download(
                    at(&self._ids_cnt_list, selected),
                    &self.username,
                )
                .is_some();
                Paragraph::new(format!(
                    "Author: {} - Year: {} - Duration: {}{}\nProgress: {}%, {} {}",
                    at(&self.auth_names_cnt_list, selected),
                    at(&self.pub_year_cnt_list, selected),
                    at(&duration_cnt_list_conv, selected),
                    if is_offline { " - [Downloaded]" } else { "" },
                    at_part(&self.book_progress_cnt_list, selected, 0), // percentage progression
                    convert_seconds_for_prg(
                        at_number(&self.duration_cnt_list, selected),
                        at_number_part(&self.book_progress_cnt_list_cur_time, selected, 0)
                    ), // time left
                    at_part(&self.book_progress_cnt_list, selected, 1), // is finished
                ))
                .left_aligned()
                .render(area, buf);
            }
        }
    }

    // description of the book or podcast `Home`
    fn render_desc_home(&self, area: Rect, buf: &mut Buffer) {
        if let Some(series) = self.selected_home_series() {
            Paragraph::new(series.description_for_the_screen())
                .scroll((self.scroll_offset, 0))
                .wrap(Wrap { trim: true })
                .render(area, buf);
            return;
        }

        if let Some(selected) = self.selected_home_item() {
            let mut _content: String = String::new();
            if self.is_podcast {
                _content = at(&self.subtitles_pod_cnt_list, selected).to_string();
            } else {
                _content = at(&self.desc_cnt_list, selected).to_string();
            }

            Paragraph::new(_content.clone())
                .scroll((self.scroll_offset, 0))
                .wrap(Wrap { trim: true })
                .render(area, buf);
        }
    }

    // info about the book or podacst for `Library`
    fn render_info_library(&self, area: Rect, buf: &mut Buffer) {
        let _duration_library_conv = convert_seconds(self.duration_library.clone());

        // A line of a series tells the number of the books and the whole
        // length. See T-22.
        if let Some(series) = self.selected_library_series() {
            let seconds: f64 = series.books.iter().map(|book| book.duration).sum();

            Paragraph::new(format!(
                "{} - {} book(s) - Duration: {}",
                series.name,
                series.books.len(),
                convert_seconds(vec![seconds])
                    .first()
                    .cloned()
                    .unwrap_or_default(),
            ))
            .left_aligned()
            .render(area, buf);
            return;
        }

        if let Some(selected) = self.selected_library_item() {
            if self.is_podcast {
                Paragraph::new(format!(
                    "Author: {}",
                    at(&self.auth_names_library_pod, selected),
                ))
                .left_aligned()
                .render(area, buf);
            } else {
                let is_offline =
                    crate::db::crud::get_download(at(&self.ids_library, selected), &self.username)
                        .is_some();
                Paragraph::new(format!(
                    "Author: {} - Year: {}{}", //- Duration: {}\nProgress:{} {}{}",
                    at(&self.auth_names_library, selected),
                    at(&self.published_year_library, selected),
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
    fn render_desc_library(&self, area: Rect, buf: &mut Buffer) {
        let text = match self.selected_library_series() {
            Some(series) => Some(series.description_for_the_screen()),
            None => self
                .selected_library_item()
                .and_then(|index| self.desc_library.get(index).cloned()),
        };

        if let Some(text) = text {
            Paragraph::new(text)
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
        let duplicated_titles = vec![at(&self.titles_pod, 0).to_string(); n];
        let duplicated_authors = vec![at(&self.authors_pod_ep, 0).to_string(); n];

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
                    let is_offline = self
                        .ids_pod_ep
                        .get(selected)
                        .map(|id| crate::db::crud::get_download(id, &self.username).is_some())
                        .unwrap_or(false);

                    Paragraph::new(format!(
                        "[{}] - Author: {} - Episode: {} - Duration: {} {}",
                        at(&duplicated_titles, selected).trim(),
                        at(&duplicated_authors, selected).trim(),
                        at(&self.episodes_pod_ep, selected).trim(),
                        at(&self.durations_pod_ep, selected).trim(),
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
        let duplicated_titles_search = vec![at(&self.titles_pod_search, 0).to_string(); n];
        let duplicated_authors_search = vec![at(&self.authors_pod_ep_search, 0).to_string(); n];
        if let Some(selected) = list_state.selected() {
            let is_offline = self
                .ids_pod_ep_search
                .get(selected)
                .map(|id| crate::db::crud::get_download(id, &self.username).is_some())
                .unwrap_or(false);

            Paragraph::new(format!(
                "[{}] - Author: {} - Episode: {} - Duration: {} {}",
                at(&duplicated_titles_search, selected).trim(),
                at(&duplicated_authors_search, selected).trim(),
                at(&self.episodes_pod_ep_search, selected).trim(),
                at(&self.durations_pod_ep_search, selected).trim(),
                if is_offline { "- [Downloaded]" } else { "" },
            ))
            .left_aligned()
            .render(area, buf);
        }
    }

    // desc of the podcast for `PodcastEpisode`
    fn render_desc_pod_ep(&self, area: Rect, buf: &mut Buffer, list_state: &ListState) {
        if let Some(selected) = list_state.selected() {
            log::debug!(
                "render_desc_pod_ep: selected={}, subtitles_pod_ep.len={}",
                selected,
                self.subtitles_pod_ep.len()
            );

            // Check if index is valid for subtitles vector
            if selected < self.subtitles_pod_ep.len() {
                Paragraph::new(at(&self.subtitles_pod_ep, selected).to_string())
                    .scroll((self.scroll_offset, 0))
                    .wrap(Wrap { trim: true })
                    .render(area, buf);
            } else {
                log::error!(
                    "render_desc_pod_ep: Index {} out of bounds for subtitles_pod_ep (len={})!",
                    selected,
                    self.subtitles_pod_ep.len()
                );
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
            Paragraph::new(at(&self.subtitles_pod_ep_search, selected).to_string())
                .scroll((self.scroll_offset, 0))
                .wrap(Wrap { trim: true })
                .render(area, buf);
        }
    }

    // info about the book or podacst for `SearchBook`
    fn render_info_search_book(&self, area: Rect, buf: &mut Buffer, list_state: &ListState) {
        let _duration_library_search_book_conv =
            convert_seconds(self.duration_library_search_book.clone());

        if let Some(selected) = list_state.selected() {
            if self.is_podcast {
                Paragraph::new(format!(
                    "Author: {}",
                    at(&self.auth_names_pod_search_book, selected),
                ))
                .left_aligned()
                .render(area, buf);
            } else {
                let is_offline = self
                    .ids_search_book
                    .get(selected)
                    .map(|id| crate::db::crud::get_download(id, &self.username).is_some())
                    .unwrap_or(false);
                Paragraph::new(format!(
                    "Author: {} - Year: {}{}", //- Duration: {}\nProgress:{} {}{}",
                    at(&self.auth_names_search_book, selected),
                    at(&self.published_year_library_search_book, selected),
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
            Paragraph::new(at(&self.desc_library_search_book, selected).to_string())
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
                Paragraph::new(format!(
                    "Toutui v{} - Licence: GPL-3.0 - Contact: {}\nSource code: {}\nWhat's new:",
                    VERSION,
                    "https://github.com/ealtun21/Toutui/issues",
                    "https://github.com/ealtun21/Toutui",
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
            Some(0) => {
                Paragraph::new(
                    "The accounts that this program holds.\n\n                     The key l on an account logs out of it: the program                      forgets the token of that server, and it asks for the                      password again at the next start.\n\n                     A program that holds more than one account starts with                      the account that is the default one.",
                )
                .wrap(Wrap { trim: true })
                .render(area, buf);
            }
            Some(1) => {
                Paragraph::new(
                    "The libraries of this server.\n\n                     The key l on a library makes it the library that the                      program shows.",
                )
                .wrap(Wrap { trim: true })
                .render(area, buf);
            }
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
            _ => {}
        }
    }

    // info for settings library
    fn render_info_settings_library(&self, area: Rect, buf: &mut Buffer, list_state: &ListState) {
        if let Some(selected) = list_state.selected() {
            Paragraph::new(format!("Type: {}", at(&self.media_types, selected),))
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
            Color::Rgb(
                color_alt_bg_list[0],
                color_alt_bg_list[1],
                color_alt_bg_list[2],
            )
        }
    }
}
