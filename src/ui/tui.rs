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
        Block, Borders, Clear, Gauge, HighlightSpacing, List, ListItem, ListState, Paragraph,
        StatefulWidget, Widget, Wrap,
    },
};
use ratatui_image::StatefulImage;

/// The number of rows of the footer of every view.
const FOOTER_HEIGHT: u16 = 2;

/// The number of rows of the panel of the player.
///
/// `render_player` of `src/ui/player_tui.rs` draws 4 rows at 9 rows above the end
/// of the screen, therefore the panel needs 6 rows above the row of the message
/// and the footer. **`main.rs` draws that panel for a playback only**, and the
/// views give those rows to the work of the view while nothing plays. See T-104.
const PLAYER_HEIGHT: u16 = 6;

/// The number of pictures of the pages of a PDF that the render keeps. See T-54.
const PICTURES_OF_THE_READER: usize = 8;

// const version
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// init widget for selected AppView
impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // A live message of the server can take a media away from the shelf of
        // Continue Listening. The render is not asynchronous, therefore the
        // lines change here and the program asks the server for nothing.
        // See T-66.
        self.take_the_media_that_left_away();

        // The program changed a collection or a playlist of the server. See
        // T-84.
        self.take_the_lists();

        // The next page of the library came. The program reads the first page
        // at the start, therefore the cost of the start is the same for a
        // library of every size. See T-70.
        self.take_the_next_page_of_the_library();

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
            AppView::Ebooks => self.render_the_ebooks(area, buf),
            AppView::Downloads => self.render_the_downloads(area, buf),
            AppView::PutInAList => self.render_put_in_a_list(area, buf),
            AppView::SendToEreader => self.render_the_devices_of_an_ereader(area, buf),
            AppView::Keys => self.render_keys(area, buf),
            AppView::Settings => self.render_settings(area, buf),
            AppView::SettingsAccount => self.render_settings_account(area, buf),
            AppView::SettingsLibrary => self.render_settings_library(area, buf),
            AppView::SettingsReader => self.render_settings_reader(area, buf),
            AppView::SettingsAbout => {}
            AppView::SettingsUpdateUninstall => {}
        }

        // The message of the user stands inside the frame, above the footer. A
        // message outside the frame goes away when a view draws its row, and it
        // stays when no view draws it. See T-59 and T-42.
        self.render_the_message(area, buf);

        // The bar goes above the other widgets. Therefore the user sees a
        // download in every view.
        App::render_downloads(area, buf);
    }
}

/// The message for the user. See T-59.
/// Gives the three areas of every view of a list: the header, the work of the
/// view, and the footer. See T-104.
///
/// **The panel of the player takes 6 rows, and it is visible only while a media
/// plays**: `main.rs` draws it for a playback and for nothing else. Every view
/// reserved those 6 rows at every moment, therefore a terminal of 18 rows gave
/// the work of the view 7 rows and it held 6 empty ones (T-99).
///
/// **The decision of the maintainer of 2026-08-12: the rows go to the view while
/// nothing plays.** Every line of the view therefore moves when a playback
/// starts, and it moves back when the playback stops.
///
/// The row above the footer stays at every moment: `render_the_message` writes
/// the message of the program there, and a view that takes that row loses its
/// last line for the six seconds of a message (the trap 39).
fn the_areas_of_a_view(area: Rect, a_media_plays: bool) -> [Rect; 3] {
    let rows_of_the_player = if a_media_plays { PLAYER_HEIGHT } else { 0 };

    let [header_area, main_area, _player_area, _message_area, footer_area] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Fill(1),
        Constraint::Length(rows_of_the_player),
        Constraint::Length(1),
        Constraint::Length(FOOTER_HEIGHT),
    ])
    .areas(area);

    [header_area, main_area, footer_area]
}

/// Gives the three areas of the work of a view of a list: the lines, the row of
/// the item, and the description. See T-99.
///
/// **A terminal that holds few rows must show the list.** The measurement of
/// 2026-08-11, in a terminal of 100 by 18: the area of these three parts held 7
/// rows, the row of the item took 3 of them, and the list therefore held **one
/// line** of 24. The description held "N/A" and 10 rows of the screen were
/// empty.
///
/// A terminal that holds enough rows keeps the split that it had: the row of the
/// item takes 3 rows, and the list and the description take a half each of what
/// stays.
///
/// **The rule is a rule of the screen, and not of this area.** The area takes the
/// 6 rows of the player while nothing plays (T-104), therefore
/// `rows_that_the_player_left` comes away before the comparison. One screen holds
/// one split: a screen of 20 rows gives the list 6 lines, and the split of a
/// large terminal would give it 5. A playback moves no line of the list into the
/// description.
fn the_areas_of_a_list(main_area: Rect, rows_that_the_player_left: u16) -> [Rect; 3] {
    // 13 rows give the list 5 lines with the split of a large terminal. Fewer
    // rows than that give every row to the list: the lines are the work of the
    // view, and a description of one row says almost nothing.
    if main_area.height.saturating_sub(rows_that_the_player_left) <= 12 {
        return Layout::vertical([
            Constraint::Fill(1),
            // The row of the item takes two rows, because its text wraps in a
            // terminal of 80 columns. See T-94.
            Constraint::Length(2),
            Constraint::Length(0),
        ])
        .areas(main_area);
    }

    Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(3),
        Constraint::Fill(1),
    ])
    .areas(main_area)
}

impl App {
    /// Draws the newest message of the program, if one is fresh.
    ///
    /// The message takes one row above the footer. It stands after the view and
    /// before the bar of the downloads: a download is the work that the user
    /// waits for, therefore that bar keeps its rows.
    fn render_the_message(&self, area: Rect, buf: &mut Buffer) {
        let Some(text) = crate::logic::message::for_the_screen() else {
            return;
        };

        // The row of the message stands above the two rows of the footer. A
        // screen that holds no such row draws no message.
        if area.height < FOOTER_HEIGHT + 1 {
            return;
        }

        let row = Rect {
            x: area.x,
            y: area.y + area.height - FOOTER_HEIGHT - 1,
            width: area.width,
            height: 1,
        };

        let background = self.config.colors.header_background_color.clone();
        let (bg_r, bg_g, bg_b) = rgb_parts(&background);
        let letters = self.config.colors.line_header_color.clone();
        let (fg_r, fg_g, fg_b) = rgb_parts(&letters);

        let style = Style::default()
            .bg(Color::Rgb(bg_r, bg_g, bg_b))
            .fg(Color::Rgb(fg_r, fg_g, fg_b))
            .add_modifier(Modifier::BOLD);

        App::draw_the_row_of_the_message(
            row,
            buf,
            &crate::logic::message::one_line(&text, area.width),
            style,
        );
    }

    /// Draws the row of the message over the view that stands below it.
    ///
    /// **The row must hold the message and nothing else.** A `Paragraph` gives
    /// its style to every cell of its area, and it writes its own text only:
    /// every letter that stood on that row before it stays. A measurement of
    /// 2026-08-11 read "CHAPTER IV.  │The Rabbit SeThe server has the place of
    /// the book." in the reader. `Clear` takes the row away first. See T-78.
    fn draw_the_row_of_the_message(row: Rect, buf: &mut Buffer, text: &str, style: Style) {
        Clear.render(row, buf);

        Paragraph::new(text.to_string())
            .centered()
            .style(style)
            .render(row, buf);
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

/// Takes one value of a list of the library for each line of the view of the
/// search.
///
/// The place of a line is the place of its media in the lists of the library.
/// **The lines of the view are not the lines of the library**: the server gives
/// its answer in the sequence of the search, and a filter of the list of the
/// library would give the episodes of a different podcast.
///
/// A line whose media the program did not read has no place. The view of the
/// search drops such a line in a library of podcasts, therefore the value that
/// is empty reaches no screen. See T-113.
fn the_values_at<T: Clone + Default>(list: &[T], places: &[Option<usize>]) -> Vec<T> {
    places
        .iter()
        .map(|place| {
            place
                .and_then(|place| list.get(place))
                .cloned()
                .unwrap_or_default()
        })
        .collect()
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
            // cover. See T-49. The list of the ebooks of one media holds one
            // media, and the cover of that media says nothing about the file
            // that the user takes. See T-76.
            AppView::Keys
            | AppView::Ebooks
            | AppView::SettingsReader
            | AppView::Downloads
            | AppView::PutInAList
            | AppView::SendToEreader => Vec::new(),
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

        // A page of a PDF can hold a picture. The panel of the picture stands at
        // the right of the text, as the panel of the cover art does. See T-54
        // and T-23.
        let picture = self
            .reader
            .as_ref()
            .filter(|reader| !reader.contents_open)
            .and_then(|reader| reader.picture_of_the_page());

        let (text_area, panel) = match &picture {
            Some(_) => cover::split_for_covers(main_area, area.width, cover::picker().font_size()),
            None => (main_area, None),
        };

        let Some(reader) = self.reader.as_mut() else {
            return;
        };

        // The task of the render sends the lines. The screen takes them here,
        // and it never waits for them.
        reader.take_the_answer();

        crate::ui::reader_tui::render(reader, text_area, buf);

        // The reader of the text holds the book, therefore the picture comes
        // after that render: two borrows of `self` must not live at the same
        // time.
        if let (Some(picture), Some(panel)) = (picture, panel) {
            self.render_the_picture_of_the_page(&picture, panel, buf);
        }
    }

    /// Draws one picture of a page of a PDF. See T-54.
    ///
    /// The program makes the form of `ratatui-image` one time for each picture,
    /// and it keeps it. A new form at each frame would read the file of the
    /// picture twenty times in one second.
    fn render_the_picture_of_the_page(
        &mut self,
        picture: &crate::logic::reader::pdf::Picture,
        panel: Rect,
        buf: &mut Buffer,
    ) {
        let key = format!(
            "{}:{}",
            self.reader
                .as_ref()
                .map(|reader| format!("{}:{}", reader.item_id, reader.chapter))
                .unwrap_or_default(),
            picture.name
        );

        if !self.pictures_of_the_reader.contains_key(&key) {
            // A user who reads a book of many pictures must not fill the memory
            // of the machine. The program keeps the pictures of the pages that
            // the user visited, and it forgets them all when they are too many.
            if self.pictures_of_the_reader.len() >= PICTURES_OF_THE_READER {
                self.pictures_of_the_reader.clear();
            }

            let made = image::load_from_memory(&picture.file)
                .ok()
                .map(|image| cover::picker().new_resize_protocol(image));

            if made.is_none() {
                log::info!(
                    "[reader] the program cannot read the picture {} of the page",
                    picture.name
                );
            }

            self.pictures_of_the_reader.insert(key.clone(), made);
        }

        // The form of the real picture gives the box, therefore a picture that is
        // higher than it is wide takes every row of the panel. See T-50.
        let form = if picture.height > 0 {
            picture.width as f32 / picture.height as f32
        } else {
            1.0
        };

        let area = cover::box_of_the_picture(panel, cover::picker().font_size(), form);

        if let Some(Some(made)) = self.pictures_of_the_reader.get_mut(&key) {
            StatefulImage::default().render(area, buf, made);
        }
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

        // The view holds the authors or the narrators, and the title says which
        // list it holds. See T-73.
        let kind = crate::logic::authors::kind();

        let (title, lines) = match &state {
            crate::logic::authors::State::Ready(all) if all.is_empty() => {
                (kind.title_of_nothing(), Vec::new())
            }
            crate::logic::authors::State::Ready(all) => (
                kind.title(all.len()),
                crate::api::libraries::get_authors::lines(all),
            ),
            crate::logic::authors::State::Waiting => {
                ("The program asks the server…".to_string(), Vec::new())
            }
            crate::logic::authors::State::Fault(text) => (kind.title_of_a_fault(text), Vec::new()),
            crate::logic::authors::State::Nothing => {
                ("The program asks the server…".to_string(), Vec::new())
            }
        };

        self.render_header(header_area, buf);
        App::render_footer(
            footer_area,
            buf,
            &crate::ui::keys::footer_with(kind.work_of_the_key_that_opens(), None),
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

/// The lists that take a media. See T-84.
impl App {
    /// AppView::PutInAList rendering
    fn render_put_in_a_list(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, item_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(4),
            Constraint::Length(2),
        ])
        .areas(area);

        // A library holds no list until a user makes the first one. The title
        // says that condition, and it names the two keys that make a list:
        // an empty box says nothing. See T-88.
        let title = match (self.the_media_of_the_list.as_ref(), self.lists.is_empty()) {
            // A server that does not answer takes no new list, and it gave no
            // list either. See T-91.
            (_, true) if self.is_offline => {
                "The server does not answer. A collection and a playlist stand on the server."
                    .to_string()
            }
            (_, true) => {
                "This library holds no collection and no playlist. Press c or p to make one."
                    .to_string()
            }
            (Some((_, _, name)), false) => {
                format!(
                    "Put \"{}\" in a list [{}]",
                    name,
                    crate::ui::keys::items(self.lists.len())
                )
            }
            (None, false) => format!(
                "Put the media in a list [{}]",
                crate::ui::keys::items(self.lists.len())
            ),
        };

        let lines: Vec<String> = self.lists.iter().map(|list| list.line()).collect();

        self.render_header(header_area, buf);
        App::render_footer(
            footer_area,
            buf,
            crate::ui::keys::FOOTER_OF_THE_LISTS_THAT_TAKE_A_MEDIA,
        );
        self.render_list(
            main_area,
            buf,
            &title,
            &lines,
            &mut self.list_state_put_in_a_list.clone(),
        );

        Paragraph::new(crate::ui::keys::THE_LISTS_THAT_TAKE_A_MEDIA)
            .wrap(Wrap { trim: true })
            .render(item_area, buf);
    }
}

/// The devices of an e-reader that take a book. See T-119.
impl App {
    /// AppView::SendToEreader rendering
    fn render_the_devices_of_an_ereader(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, item_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(4),
            Constraint::Length(2),
        ])
        .areas(area);

        let book = self
            .the_book_of_the_send
            .as_ref()
            .map(|(_, title)| title.clone())
            .unwrap_or_default();

        // **An empty list is an answer of the server**, and not a fault: the
        // server holds no device. The view therefore says the reason, and it is
        // a reason that the program has. See T-91.
        let (title, lines) = match crate::logic::the_ereaders::state() {
            crate::logic::the_ereaders::State::Nothing
            | crate::logic::the_ereaders::State::Waiting => (
                "The program asks the server for the devices of an e-reader…".to_string(),
                Vec::new(),
            ),
            crate::logic::the_ereaders::State::Fault(reason) => {
                (format!("The server gave no device: {}", reason), Vec::new())
            }
            crate::logic::the_ereaders::State::Ready(all) if all.is_empty() => (
                "The server holds no device for an e-reader. An administrator of the server \
                 adds one."
                    .to_string(),
                Vec::new(),
            ),
            crate::logic::the_ereaders::State::Ready(all) => (
                match book.is_empty() {
                    true => format!(
                        "Send the ebook to a device [{}]",
                        crate::ui::keys::items(all.len())
                    ),
                    false => format!(
                        "Send \"{}\" to a device [{}]",
                        book,
                        crate::ui::keys::items(all.len())
                    ),
                },
                all.iter().map(|device| device.line()).collect(),
            ),
        };

        self.render_header(header_area, buf);
        App::render_footer(
            footer_area,
            buf,
            crate::ui::keys::FOOTER_OF_THE_DEVICES_OF_AN_EREADER,
        );
        self.render_list(
            main_area,
            buf,
            &title,
            &lines,
            &mut self.list_state_send_to_ereader.clone(),
        );

        Paragraph::new(crate::ui::keys::THE_DEVICES_OF_AN_EREADER)
            .wrap(Wrap { trim: true })
            .render(item_area, buf);
    }
}

/// The queue of the downloads of the server. See T-81.
impl App {
    /// AppView::Downloads rendering
    fn render_the_downloads(&mut self, area: Rect, buf: &mut Buffer) {
        // The server sends a message at each change of the queue. The view then
        // asks the server again, therefore the list moves while the user looks
        // at it and the user presses no key. See T-81.
        if crate::logic::the_downloads::the_view_must_ask() {
            self.ask_for_the_downloads();
        }

        let [header_area, main_area, item_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(4),
            Constraint::Length(2),
        ])
        .areas(area);

        let (title, lines) = match crate::logic::the_downloads::state() {
            crate::logic::the_downloads::State::Ready(all) if all.is_empty() => (
                "The server downloads no episode. Press E on a podcast to get its new \
                 episodes."
                    .to_string(),
                Vec::new(),
            ),
            crate::logic::the_downloads::State::Ready(all) => (
                format!(
                    "The downloads of the server [{}]",
                    crate::ui::keys::items(all.len())
                ),
                all.iter().map(|one| one.line()).collect(),
            ),
            crate::logic::the_downloads::State::Fault(text) => (
                format!("The server gave no queue of the downloads: {}", text),
                Vec::new(),
            ),
            _ => ("The program asks the server…".to_string(), Vec::new()),
        };

        self.render_header(header_area, buf);
        App::render_footer(footer_area, buf, crate::ui::keys::FOOTER_OF_THE_DOWNLOADS);
        self.render_list(
            main_area,
            buf,
            &title,
            &lines,
            &mut self.list_state_downloads.clone(),
        );

        Paragraph::new(crate::ui::keys::THE_DOWNLOADS_OF_THE_SERVER)
            .wrap(Wrap { trim: true })
            .render(item_area, buf);
    }
}

/// The ebooks of one media. See T-76.
impl App {
    /// AppView::Ebooks rendering
    fn render_the_ebooks(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, item_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(4),
            Constraint::Length(2),
        ])
        .areas(area);

        let (title, lines) = match crate::logic::the_ebooks::state() {
            crate::logic::the_ebooks::State::Ready(all) if all.is_empty() => {
                ("This media has no ebook.".to_string(), Vec::new())
            }
            crate::logic::the_ebooks::State::Ready(all) => (
                format!(
                    "The books of this media [{}]",
                    crate::ui::keys::items(all.len())
                ),
                all.iter().map(|one| one.line()).collect(),
            ),
            crate::logic::the_ebooks::State::Fault(text) => (
                format!("The server gave no list of the books: {}", text),
                Vec::new(),
            ),
            _ => ("The program asks the server…".to_string(), Vec::new()),
        };

        self.render_header(header_area, buf);
        App::render_footer(
            footer_area,
            buf,
            &crate::ui::keys::footer_with("read this book", None),
        );
        self.render_list(
            main_area,
            buf,
            &title,
            &lines,
            &mut self.list_state_ebooks.clone(),
        );

        Paragraph::new(
            "The server holds one place for each media, and not one place for \
             each book. Therefore the program sends the place of the book of \
             the server only, and it keeps the place of every other book on \
             this machine.",
        )
        .wrap(Wrap { trim: true })
        .render(item_area, buf);
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
                format!(
                    "A new podcast [{}]",
                    crate::ui::keys::counted(all.len(), "answer")
                ),
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
                format!("The bookmarks [{}]", crate::ui::keys::items(all.len())),
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
            format!("The queue [{}]", crate::ui::keys::items(lines.len()))
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

        // The list holds no line for three reasons, and the title must name the
        // right one. A user who presses `C` with no media reads a different
        // sentence from a user whose media holds no chapter. See T-59.
        let title = if !lines.is_empty() {
            format!(
                "The chapters of \"{}\" [{}]",
                state.title,
                crate::ui::keys::items(lines.len())
            )
        } else if state.status == PlaybackStatus::Stopped {
            "No media plays now. A media that plays gives its chapters. Press h to go back."
                .to_string()
        } else {
            format!("\"{}\" holds no chapter. Press h to go back.", state.title)
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
        let [header_area, main_area, footer_area] = the_areas_of_a_view(area, self.a_media_plays());

        // The panel of the covers stands at the right of the list and of the
        // description. It is always visible. See T-23.
        let (main_area, cover_panel) =
            cover::split_for_covers(main_area, area.width, cover::picker().font_size());
        self.render_covers(cover_panel, buf);

        let [list_area, item_area1, item_area2] =
            the_areas_of_a_list(main_area, self.the_rows_that_the_player_left());

        // Every line starts with a mark: the media that plays, a media that
        // the user finished, or the part that the user heard. See T-44.
        let lines = self.home_lines();
        let count = self
            .home_rows
            .iter()
            .filter(|row| row.is_a_line_of_the_user())
            .count();
        let render_list_title = format!("Home [{}]", crate::ui::keys::items(count));

        // A library of podcasts has no series and no ebook. The footer of
        // that library must not name a key that does nothing.
        let text_render_footer = if self.is_podcast {
            crate::ui::keys::FOOTER_OF_A_LIBRARY_OF_PODCASTS
        } else {
            crate::ui::keys::FOOTER_OF_A_LIBRARY_OF_BOOKS
        };

        self.render_header(header_area, buf);
        App::render_footer(footer_area, buf, text_render_footer);

        // **A view says why it holds no line.** The Home view of a library with
        // no media drew an empty list and no word at all. See T-103 and T-91.
        if lines.is_empty() {
            App::render_the_reason(
                main_area,
                buf,
                crate::ui::keys::the_text_of_the_home_view_with_no_line(self.is_offline),
            );
            return;
        }

        self.render_list(
            list_area,
            buf,
            &render_list_title,
            &lines,
            &mut self.list_state_cnt_list.clone(),
        );
        self.render_info_home(item_area1, buf);
        self.render_desc_home(item_area2, buf);
    }

    /// Says if a media plays now. See T-104.
    ///
    /// The panel of the player stands on the screen for a playback only, and the
    /// areas of a view therefore hold its 6 rows for a playback only. A playback
    /// that a pause holds is a playback: the panel stays, and the user reads the
    /// place of that media.
    fn a_media_plays(&self) -> bool {
        self.player.state().status != PlaybackStatus::Stopped
    }

    /// The rows of the panel of the player that the view holds now. See T-104.
    ///
    /// The split of the work of a view is a rule of the screen (T-99), therefore
    /// `the_areas_of_a_list` takes these rows away before it compares.
    fn the_rows_that_the_player_left(&self) -> u16 {
        if self.a_media_plays() {
            return 0;
        }

        PLAYER_HEIGHT
    }

    /// Draws the sentence of a view that holds no line. See T-103.
    ///
    /// Every view of the program that can hold no line says why, and this
    /// function holds the shape of that screen in one place.
    fn render_the_reason(area: Rect, buf: &mut Buffer, text: &str) {
        Paragraph::new(text)
            .centered()
            .wrap(Wrap { trim: true })
            .block(
                Block::new()
                    .borders(Borders::TOP)
                    .border_style(Style::new().fg(Color::DarkGray)),
            )
            .render(area, buf);
    }

    /// AppView::Library rendering
    fn render_library(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, footer_area] = the_areas_of_a_view(area, self.a_media_plays());

        // The panel of the covers stands at the right of the list and of the
        // description. It is always visible. See T-23.
        let (main_area, cover_panel) =
            cover::split_for_covers(main_area, area.width, cover::picker().font_size());
        self.render_covers(cover_panel, buf);

        let [list_area, item_area1, item_area2] =
            the_areas_of_a_list(main_area, self.the_rows_that_the_player_left());

        // Every book of a series gives one line. See T-22.
        let lines = self.library_lines();
        // The title says the sequence and the filter, because a user who
        // does not see every item must know why. See T-24.
        let render_list_title = format!(
            "Library [{}]{}{}",
            // **The program holds the pages that it read, and the server told
            // how many items the library holds.** A title that says the number
            // of the lines only is not false, and it says less than the program
            // knows. See T-70 and T-91.
            crate::ui::keys::the_lines_of_the_library(
                lines.len(),
                self.ids_library.len(),
                self.library_total
            ),
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

        // **A filter that hides every media is not a library with no media**,
        // and a server that does not answer is neither. See T-103 and T-91.
        if lines.is_empty() {
            App::render_the_reason(
                main_area,
                buf,
                crate::ui::keys::the_text_of_the_library_view_with_no_line(
                    self.is_offline,
                    !self.library_filter.is_empty(),
                    self.is_podcast,
                ),
            );
            return;
        }

        self.render_list(
            list_area,
            buf,
            &render_list_title,
            &lines,
            &mut self.list_state_library.clone(),
        );
        self.render_info_library(item_area1, buf);
        self.render_desc_library(item_area2, buf);
    }

    /// AppView::Series rendering: the list of the series of the library.
    fn render_series(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, footer_area] = the_areas_of_a_view(area, self.a_media_plays());

        // The panel of the covers stands at the right of the list and of the
        // description. It is always visible. See T-23.
        let (main_area, cover_panel) =
            cover::split_for_covers(main_area, area.width, cover::picker().font_size());
        self.render_covers(cover_panel, buf);

        let [list_area, item_area1, item_area2] =
            the_areas_of_a_list(main_area, self.the_rows_that_the_player_left());

        let text_render_footer = crate::ui::keys::FOOTER_OF_A_LIST;

        self.render_header(header_area, buf);
        App::render_footer(footer_area, buf, text_render_footer);

        if self.series.is_empty() {
            // A server that does not answer gives no series, and that is not a
            // library with no series. See T-91.
            let text = if self.is_offline {
                "The server gave no series: the server does not answer.\nPress h to go back."
            } else {
                "This library has no series.\nPress h to go back."
            };

            Paragraph::new(text)
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
        let render_list_title = format!("Series [{}]", crate::ui::keys::items(self.series.len()));

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
                "{} - {} - Duration: {}",
                series.name,
                crate::ui::keys::counted(books, "book"),
                convert_seconds(vec![seconds])
                    .first()
                    .cloned()
                    .unwrap_or_default(),
            ))
            .wrap(Wrap { trim: true })
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
        let [header_area, main_area, footer_area] = the_areas_of_a_view(area, self.a_media_plays());

        // The panel of the covers stands at the right of the list and of the
        // description. It is always visible. See T-23.
        let (main_area, cover_panel) =
            cover::split_for_covers(main_area, area.width, cover::picker().font_size());
        self.render_covers(cover_panel, buf);

        let [list_area, item_area1, item_area2] =
            the_areas_of_a_list(main_area, self.the_rows_that_the_player_left());

        let text_render_footer = crate::ui::keys::FOOTER_OF_A_LIST_OF_MEDIA;

        self.render_header(header_area, buf);
        App::render_footer(footer_area, buf, text_render_footer);

        let Some(series) = self.selected_series() else {
            return;
        };

        let name = series.name.clone();
        let lines: Vec<String> = series.books.iter().map(|book| book.line()).collect();
        let render_list_title = format!("{} [{}]", name, crate::ui::keys::items(lines.len()));

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
            .wrap(Wrap { trim: true })
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
        let [header_area, main_area, footer_area] = the_areas_of_a_view(area, self.a_media_plays());

        // The panel of the covers stands at the right of the list and of the
        // description. It is always visible. See T-23.
        let (main_area, cover_panel) =
            cover::split_for_covers(main_area, area.width, cover::picker().font_size());
        self.render_covers(cover_panel, buf);

        let [list_area, item_area1, item_area2] =
            the_areas_of_a_list(main_area, self.the_rows_that_the_player_left());

        // The keys `r` and `X` of this view need a footer of its own. See T-93.
        let text_render_footer = crate::ui::keys::FOOTER_OF_THE_LISTS;

        self.render_header(header_area, buf);
        App::render_footer(footer_area, buf, text_render_footer);

        if self.lists.is_empty() {
            // **The program knows nothing of the lists of a server that does
            // not answer.** The offline sweep of 2026-08-11 read "This library
            // has no collection and no playlist" with the server stopped, and
            // that sentence is not true: no request gave an answer. The view of
            // the authors held this rule already. See T-91.
            let text = if self.is_offline {
                "The server gave no collection and no playlist: the server does not answer.\n\
                 Press h to go back."
            } else {
                "This library has no collection and no playlist.\nPress h to go back."
            };

            Paragraph::new(text)
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
        let render_list_title = format!(
            "Collections and playlists [{}]",
            crate::ui::keys::items(self.lists.len())
        );

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
                "{} - {} - Duration: {}",
                list.kind.name(),
                crate::ui::keys::items(list.entries.len()),
                convert_seconds(vec![seconds])
                    .first()
                    .cloned()
                    .unwrap_or_default(),
            ))
            .wrap(Wrap { trim: true })
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
        let [header_area, main_area, footer_area] = the_areas_of_a_view(area, self.a_media_plays());

        // The panel of the covers stands at the right of the list and of the
        // description. It is always visible. See T-23.
        let (main_area, cover_panel) =
            cover::split_for_covers(main_area, area.width, cover::picker().font_size());
        self.render_covers(cover_panel, buf);

        let [list_area, item_area1, item_area2] =
            the_areas_of_a_list(main_area, self.the_rows_that_the_player_left());

        let text_render_footer = crate::ui::keys::FOOTER_OF_THE_MEDIA_OF_A_LIST;

        self.render_header(header_area, buf);
        App::render_footer(footer_area, buf, text_render_footer);

        let Some(list) = self.selected_list() else {
            return;
        };

        let lines: Vec<String> = list.entries.iter().map(|entry| entry.line()).collect();
        let render_list_title = format!(
            "{} [{}]",
            list.name.clone(),
            crate::ui::keys::items(lines.len())
        );

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
            .wrap(Wrap { trim: true })
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
        let [header_area, main_area, footer_area] = the_areas_of_a_view(area, self.a_media_plays());

        let [list_area, item_area1, item_area2] =
            the_areas_of_a_list(main_area, self.the_rows_that_the_player_left());

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
        let [header_area, main_area, footer_area] = the_areas_of_a_view(area, self.a_media_plays());

        let [list_area, item_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Fill(1)]).areas(main_area);

        // **The title and the footer name every key of this view.** A key that
        // a view holds and that no text names is a key that no user finds.
        // See T-124 and T-79.
        let render_list_title = "Accounts — a: add, c: this account starts, l: log out";
        // **The area of the footer holds two rows, and 80 columns give 160
        // cells** (T-90). The words of the log out stand in the text of the
        // view: the footer names the keys.
        let text_render_footer = "h: back, a: add an account, c: this account starts, \
             l/→: log out,\n Tab: home, R: refresh, Q/Esc: quit.";

        self.render_header(header_area, buf);
        App::render_footer(footer_area, buf, text_render_footer);
        self.render_list(
            list_area,
            buf,
            render_list_title,
            &self.the_lines_of_the_accounts(),
            &mut self.list_state_settings_account.clone(),
        );

        // **The type of the account and its permissions.** `GET /api/me` gives
        // them, and no screen of the program showed one of them: a user whose
        // account may not download read the message of the key `D` and nothing
        // else. See T-110.
        //
        // The offline mode says that the program knows nothing of the account:
        // no request answered, therefore the values below are the values of a
        // program that asked nothing (T-91).
        let lines = if self.is_offline {
            vec![
                "The server does not answer, therefore this screen holds no \
                 permission of your account."
                    .to_string(),
            ]
        } else {
            crate::api::me::permissions::the_lines_of_the_account(&self.username, &self.account)
        };

        Paragraph::new(lines.join("\n"))
            .wrap(Wrap { trim: true })
            .render(item_area, buf);
    }

    /// AppView::SettingsReader rendering. See T-77.
    fn render_settings_reader(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, item_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(5),
            Constraint::Length(2),
        ])
        .areas(area);

        let now = self.megabytes_of_the_cache();

        let lines: Vec<String> = crate::logic::reader::cache::THE_VALUES_OF_THE_SETTINGS
            .iter()
            .map(|value| crate::logic::reader::cache::line_of_a_value(*value, now))
            .collect();

        let title = format!("The cache of the ebooks — {} MB now", now);

        self.render_header(header_area, buf);
        App::render_footer(
            footer_area,
            buf,
            &crate::ui::keys::footer_with("write this value in config.toml", None),
        );
        self.render_list(
            main_area,
            buf,
            &title,
            &lines,
            &mut self.list_state_settings_reader.clone(),
        );

        Paragraph::new(crate::ui::keys::THE_CACHE_OF_THE_EBOOKS)
            .wrap(Wrap { trim: true })
            .render(item_area, buf);
    }

    /// AppView::SettingsLibrary rendering
    fn render_settings_library(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, footer_area] = the_areas_of_a_view(area, self.a_media_plays());

        let [list_area, item_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Fill(1)]).areas(main_area);

        let items_number = self.libraries_names.len();
        let render_list_title = format!(
            "Settings Library [{}]",
            crate::ui::keys::items(items_number)
        );

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

    /// Gives the lines of the search that the program can make by itself.
    ///
    /// The answer of the server comes some hundred milliseconds after the key
    /// Enter. The program looks in the titles that it holds while the user
    /// waits, and the title of the view says where the lines come from. See
    /// T-24 and T-113.
    fn the_titles_of_this_program_that_hold_the_words(&self) -> Vec<crate::logic::search::Found> {
        let words = self.search_query.to_lowercase();

        self.titles_library
            .iter()
            .enumerate()
            .filter(|(_, title)| title.to_lowercase().contains(&words))
            .map(|(place, title)| crate::logic::search::Found {
                id: at(&self.ids_library, place).to_string(),
                title: title.clone(),
                author: at(&self.auth_names_library, place).to_string(),
                author_of_a_podcast: at(&self.auth_names_library_pod, place).to_string(),
                year: at(&self.published_year_library, place).to_string(),
                description: at(&self.desc_library, place).to_string(),
                duration: at_number(&self.duration_library, place),
                place: Some(place),
            })
            .collect()
    }

    /// AppView::SearchBook rendering
    fn render_search_book(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, footer_area] = the_areas_of_a_view(area, self.a_media_plays());

        // The panel of the covers stands at the right of the list and of the
        // description. It is always visible. See T-23.
        let (main_area, cover_panel) =
            cover::split_for_covers(main_area, area.width, cover::picker().font_size());
        self.render_covers(cover_panel, buf);

        let [list_area, item_area1, item_area2] =
            the_areas_of_a_list(main_area, self.the_rows_that_the_player_left());

        let from_the_server = crate::logic::search::from_the_server::answer_for(&self.search_query);

        // The title says where the answer comes from, it names the author and the
        // narrator of the answer, and **it says why the view holds no line**.
        // `the_title_of_the_search` holds every case. See T-24 and T-70.
        //
        // The number of the lines comes below, therefore the program makes the
        // title after the lines.

        // The keys of this view are not the keys of the Library: `h` goes back
        // to the view that the search came from, and `/` searches again. See
        // T-79.
        let _text_render_footer = crate::ui::keys::FOOTER_OF_THE_SEARCH;

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
        // **The answer of the server holds every value of its lines**, therefore
        // a media of a page that the program did not read gives a line too. The
        // program looks in the titles that it holds while the answer comes.
        // See T-113.
        let mut found: Vec<crate::logic::search::Found> = match &from_the_server {
            Some(answer) => answer
                .media
                .iter()
                .map(|one| crate::logic::search::Found {
                    place: self.ids_library.iter().position(|id| id == &one.id),
                    ..one.clone()
                })
                .collect(),
            None => self.the_titles_of_this_program_that_hold_the_words(),
        };

        // **A library of podcasts reads the episodes of a media with the place
        // of that media in the lists of the library.** A podcast of a page that
        // the program did not read therefore gives no line, and one page holds
        // 500 podcasts. See T-113.
        if self.is_podcast {
            let before = found.len();

            found.retain(|one| one.place.is_some());

            if found.len() != before {
                log::info!(
                    "[search] the program did not read {} podcast(s) of the answer",
                    before - found.len()
                );
            }
        }

        // The reader of a book takes the title of the line (T-117), therefore the
        // titles of the view stand in the application and not in this function.
        self.titles_search_book = found.iter().map(|one| one.title.clone()).collect();

        let titles_search_book_or_pod: &[String] = &self.titles_search_book.clone();

        let render_list_title = crate::logic::search::the_title_of_the_search(
            &self.search_query,
            from_the_server.is_some(),
            from_the_server
                .as_ref()
                .map(|answer| answer.names.as_slice())
                .unwrap_or(&[]),
            titles_search_book_or_pod.len(),
        );
        let render_list_title = render_list_title.as_str();

        // Every list of the view holds one value for each line of the view.
        self.ids_search_book = found.iter().map(|one| one.id.clone()).collect();
        self.auth_names_pod_search_book = found
            .iter()
            .map(|one| one.author_of_a_podcast.clone())
            .collect();
        self.auth_names_search_book = found.iter().map(|one| one.author.clone()).collect();
        self.published_year_library_search_book =
            found.iter().map(|one| one.year.clone()).collect();
        self.desc_library_search_book = found.iter().map(|one| one.description.clone()).collect();
        self.duration_library_search_book = found.iter().map(|one| one.duration).collect();
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

        // **The lists of a podcast come from the place of the media in the lists
        // of the library**, and they hold one value for each line of the view: a
        // list in the sequence of the library would give the episodes of a
        // different podcast. See T-113.
        let places: Vec<Option<usize>> = found.iter().map(|one| one.place).collect();

        // apply search filtering for podacst
        self.all_titles_pod_ep_search = the_values_at(&self.all_titles_pod_ep, &places);
        self.all_ids_pod_ep_search = the_values_at(&self.all_ids_pod_ep, &places);
        self.all_subtitles_pod_ep_search = the_values_at(&self.all_subtitles_pod_ep, &places);
        self.all_seasons_pod_ep_search = the_values_at(&self.all_seasons_pod_ep, &places);
        self.all_episodes_pod_ep_search = the_values_at(&self.all_episodes_pod_ep, &places);
        self.all_authors_pod_ep_search = the_values_at(&self.all_authors_pod_ep, &places);
        self.all_descs_pod_ep_search = the_values_at(&self.all_descs_pod_ep, &places);
        self.all_titles_pod_search = the_values_at(&self.all_titles_pod, &places);
        self.all_durations_pod_ep_search = the_values_at(&self.all_durations_pod_ep, &places);
        self.ids_library_pod_search = the_values_at(&self.ids_library, &places);

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
        let [header_area, main_area, footer_area] = the_areas_of_a_view(area, self.a_media_plays());

        // The panel of the covers stands at the right of the list and of the
        // description. It is always visible. See T-23.
        let (main_area, cover_panel) =
            cover::split_for_covers(main_area, area.width, cover::picker().font_size());
        self.render_covers(cover_panel, buf);

        let [list_area, item_area1, item_area2] =
            the_areas_of_a_list(main_area, self.the_rows_that_the_player_left());

        let text_render_footer = crate::ui::keys::FOOTER_OF_A_LIST_OF_MEDIA;

        self.render_header(header_area, buf);
        App::render_footer(footer_area, buf, text_render_footer);
        let no_episodes_message = "This podcast has no episode.\nPress h to go back.";

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
                let render_list_title =
                    format!("Episodes [{}]", crate::ui::keys::items(items_number));
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
                let render_list_title =
                    format!("Episodes [{}]", crate::ui::keys::items(items_number));
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

        // **The address of the pool, and not the address of the login.** A pool
        // of two addresses moves between them, and the header named the address
        // that the user gave at the login for ever. See T-105 and T-107.
        let active = self.api.pool().active();

        let connection = crate::ui::keys::the_lines_of_the_connection(
            &self.username,
            active.as_deref(),
            &self.server_address_pretty,
            self.is_offline,
            area.width,
        );

        // The audio engine did not start. The user reads the library, and no
        // media plays. See T-46.
        let connection = match &self.audio_fault {
            Some(_) => format!("{}\n🔇 No sound device: no media can play", connection),
            None => connection,
        };

        Paragraph::new(connection)
            .not_bold()
            .wrap(Wrap { trim: true })
            .left_aligned()
            .render(area, buf);

        let notice = if self.is_offline {
            let waiting = match self.waiting_progress {
                0 => String::new(),
                1 => " - 1 position waits".to_string(),
                count => format!(" - {} wait", crate::ui::keys::counted(count, "position")),
            };

            format!("R: try the server again{}", waiting)
        } else if active.is_none() {
            // No address answers, and the lists still come from the server. The
            // key `R` gives the media of the disk. See T-107.
            crate::ui::keys::THE_SERVER_DOES_NOT_ANSWER.to_string()
        } else if crate::logic::live::the_lists_are_old() {
            // A different client changed the metadata of an item. That value
            // stands in many lists, therefore one line cannot hold the
            // correction. See T-47.
            "R: the server has newer data".to_string()
        } else {
            self.update_msg.clone()
        };

        Paragraph::new(format!(
            "{}\n {}",
            crate::ui::keys::the_name_of_the_program(VERSION, area.width),
            notice
        ))
        .right_aligned()
        .render(area, buf);
    }

    /// Draws the footer of a view.
    ///
    /// **The footer holds two rows, and it writes on one of them only.** A
    /// terminal of 80 columns therefore lost the end of every footer: the Home
    /// view of a library of podcasts ended with "?: every k", and the keys `?`
    /// and `Q` are the two keys that a lost user needs. See T-90 and T-52.
    ///
    /// The text wraps now. A wide terminal draws one row, as it did before,
    /// and a terminal of 80 columns draws two.
    fn render_footer(area: Rect, buf: &mut Buffer, text_render_footer: &str) {
        Paragraph::new(text_render_footer)
            .wrap(Wrap { trim: true })
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
                "{} - {} - Duration: {}",
                series.name,
                crate::ui::keys::counted(series.books.len(), "book"),
                convert_seconds(vec![seconds])
                    .first()
                    .cloned()
                    .unwrap_or_default(),
            ))
            .wrap(Wrap { trim: true })
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
                .wrap(Wrap { trim: true })
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
                .wrap(Wrap { trim: true })
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
                "{} - {} - Duration: {}",
                series.name,
                crate::ui::keys::counted(series.books.len(), "book"),
                convert_seconds(vec![seconds])
                    .first()
                    .cloned()
                    .unwrap_or_default(),
            ))
            .wrap(Wrap { trim: true })
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
                .wrap(Wrap { trim: true })
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
                .wrap(Wrap { trim: true })
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
                .wrap(Wrap { trim: true })
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
                    .wrap(Wrap { trim: true })
                    .left_aligned()
                    .render(area, buf);
                } else {
                    log::error!("render_info_pod_ep: Index {} out of bounds for duplicated title/author vectors (len={})!", selected, duplicated_titles.len());
                    Paragraph::new("Error: Episode info rendering mismatch.")
                        .wrap(Wrap { trim: true })
                        .left_aligned()
                        .render(area, buf);
                }
            } else {
                log::error!("render_info_pod_ep: Index {} out of bounds for episode/duration vectors (ep_len={}, dur_len={})!", selected, self.episodes_pod_ep.len(), self.durations_pod_ep.len());
                Paragraph::new("Error: Episode data unavailable or index out of bounds.")
                    .wrap(Wrap { trim: true })
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
            .wrap(Wrap { trim: true })
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
                    .wrap(Wrap { trim: true })
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
                .wrap(Wrap { trim: true })
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
                .wrap(Wrap { trim: true })
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
                .wrap(Wrap { trim: true })
                .left_aligned()
                .render(area, buf);
            }
            _ => {}
        }
    }

    // desc for settings
    //
    // A sweep of every view of 2026-08-11 read "the program
    // forgets the token" with a run of 22 spaces inside the line. The two texts
    // below held the space of an old wrap of the source, and `Wrap` keeps a
    // space that stands inside a line. They stand as constants now, and a test
    // holds them to one space. See `THE_ACCOUNTS` and `THE_LIBRARIES`.
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
                Paragraph::new(crate::ui::keys::THE_ACCOUNTS)
                    .wrap(Wrap { trim: true })
                    .render(area, buf);
            }
            Some(1) => {
                Paragraph::new(crate::ui::keys::THE_LIBRARIES)
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
            // The cache of the ebooks. See T-77.
            Some(4) => {
                Paragraph::new(crate::ui::keys::THE_CACHE_OF_THE_EBOOKS)
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
                .wrap(Wrap { trim: true })
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The row of the message must hold the message and nothing else.
    ///
    /// A measurement of the real program on 2026-08-11 read this row of the
    /// reader:
    ///
    /// ```text
    /// CHAPTER IV.       │The Rabbit SeThe server has the place of the book.
    /// ```
    ///
    /// A `Paragraph` gives its style to every cell of its area, and it writes
    /// its own text only. Every letter of the view stayed. See T-78.
    #[test]
    fn the_row_of_the_message_takes_the_letters_of_the_view_away() {
        let row = Rect {
            x: 0,
            y: 0,
            width: 40,
            height: 1,
        };

        let mut buf = Buffer::empty(row);

        // The view wrote a line of text on that row.
        buf.set_string(
            0,
            0,
            "CHAPTER IV.    The Rabbit Sends in a Bill",
            Style::default(),
        );

        App::draw_the_row_of_the_message(
            row,
            &mut buf,
            "The server has the place.",
            Style::default(),
        );

        let line: String = (0..row.width)
            .map(|column| buf[(column, 0)].symbol().to_string())
            .collect();

        assert_eq!(line.trim(), "The server has the place.");
        assert!(
            !line.contains("CHAPTER"),
            "the row holds the text of the view: \"{}\"",
            line
        );
    }

    /// **The 6 rows of the player go to the view while nothing plays.** They
    /// stood empty at every moment before, and a terminal of 18 rows gave the
    /// work of the view 7 rows of the 18. The decision of the maintainer of
    /// 2026-08-12. See T-104 and T-99.
    #[test]
    fn the_view_takes_the_rows_of_the_player_while_nothing_plays() {
        let screen = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 18,
        };

        let [header, main, footer] = the_areas_of_a_view(screen, true);
        let [header_of_no_playback, main_of_no_playback, footer_of_no_playback] =
            the_areas_of_a_view(screen, false);

        assert_eq!(main.height, 7, "the areas of a playback do not change");
        assert_eq!(
            main_of_no_playback.height,
            main.height + PLAYER_HEIGHT,
            "the view must take the rows of the player"
        );

        // **The header and the footer stay where they stand.** The row of the
        // message stands above the footer at every moment, therefore a message
        // takes no line of the view away (the trap 39).
        assert_eq!(header, header_of_no_playback);
        assert_eq!(footer, footer_of_no_playback);
        assert_eq!(footer.height, FOOTER_HEIGHT);
        assert_eq!(
            main_of_no_playback.y + main_of_no_playback.height,
            footer.y - 1,
            "the row of the message stands between the view and the footer"
        );
    }

    /// **The reflow of the rows of the player must take no line of the list
    /// away.** One screen holds one split of the work of a view: the split is a
    /// rule of the screen (T-99), and the area of that work grows by 6 rows while
    /// nothing plays (T-104).
    ///
    /// The first form of T-104 compared the area, and not the screen. A screen of
    /// 20 rows then gave the list **5** lines while nothing played and **6** while
    /// a media played: the reflow took one line of the work away, and it gave 6
    /// rows to a description of "No description available".
    #[test]
    fn one_screen_holds_one_split_of_the_work_of_a_view() {
        for height in 6..60 {
            let screen = Rect {
                x: 0,
                y: 0,
                width: 100,
                height,
            };

            let [_, of_a_playback, _] = the_areas_of_a_view(screen, true);
            let [_, of_no_playback, _] = the_areas_of_a_view(screen, false);

            let [list_of_a_playback, ..] = the_areas_of_a_list(of_a_playback, 0);
            let [list_of_no_playback, ..] = the_areas_of_a_list(of_no_playback, PLAYER_HEIGHT);

            assert!(
                list_of_no_playback.height >= list_of_a_playback.height,
                "a screen of {} rows gives the list {} lines while nothing plays, \
                 and {} while a media plays",
                height,
                list_of_no_playback.height,
                list_of_a_playback.height
            );
        }
    }

    /// A terminal that holds fewer rows than the parts of a view must give an
    /// answer, and it must not stop the program.
    #[test]
    fn a_terminal_of_five_rows_gives_the_areas_of_a_view() {
        for height in 0..=6 {
            let screen = Rect {
                x: 0,
                y: 0,
                width: 40,
                height,
            };

            for a_media_plays in [true, false] {
                let [header, main, footer] = the_areas_of_a_view(screen, a_media_plays);

                assert!(header.y + header.height <= screen.height.max(1) + 1);
                assert!(main.y >= header.y);
                assert!(footer.y + footer.height <= screen.y + screen.height + FOOTER_HEIGHT);
            }
        }
    }
}
