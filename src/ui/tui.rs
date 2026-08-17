use crate::app::App;
use crate::app::AppView;
use crate::logic::download::progress::{DownloadProgress, DownloadState};
use crate::player::engine::PlaybackStatus;
use crate::ui::cover;
use crate::utils::convert_seconds::*;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    widgets::{
        Block, Borders, Clear, Gauge, HighlightSpacing, List, ListItem, ListState, Paragraph,
        Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget, Wrap,
    },
};
use ratatui_image::StatefulImage;

/// The smallest number of rows of the footer of a view.
///
/// **A footer stands on the rows that it needs** (T-302): this is the floor of
/// `crate::ui::keys::the_rows_of_a_footer`, and no view holds fewer rows than
/// the two that every view held before.
const FOOTER_HEIGHT: u16 = crate::ui::keys::THE_SMALLEST_FOOTER;

/// The number of rows of the header of every view.
///
/// The header says the account, the server, and the library, and the row of the
/// message must not take those rows: the message grows upward over the view
/// alone. See T-171 and T-299.
const HEADER_HEIGHT: u16 = 2;

/// The number of rows of the band of the player, with its row of the buttons.
///
/// **The band stands on the rows that it needs** (T-322): the key `B` takes the
/// row of the buttons away, and `App::the_rows_of_the_band` then gives one row
/// fewer. **The render draws the band for a playback only**, and the views give
/// those rows to the work of the view while nothing plays. See T-104.
#[cfg(test)]
const PLAYER_HEIGHT: u16 = crate::ui::the_band_of_the_player::the_rows_of_the_band(true);

/// The number of pictures of the pages of a PDF that the render keeps. See T-54.
const PICTURES_OF_THE_READER: usize = 8;

// const version
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// init widget for selected AppView
impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // **The key handler reads the width of the screen** (T-320): the frame
        // of the panels holds three shapes of a width, and the keys of the
        // focus of a panel belong to the shape that draws that panel alone. The
        // render is the one place of this program that knows that width.
        self.the_width_of_the_screen = area.width;

        // **One frame asks the server for a number of new covers, and no
        // more** (T-338): this function is the one road of the program to a
        // frame of the screen, therefore the limit of `cover` stands again
        // here. The bands of covers of the Home view draw about 20 cells, and
        // every new cell of a frame was one request of one moment.
        self.covers.a_new_frame();

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

        // The episodes of the podcast that the user opened came. See T-126.
        self.take_the_episodes_that_came();

        // The view of this frame writes the rows of its footer (T-302). A view
        // that draws no footer keeps the two rows that every view held.
        self.rows_of_the_footer = FOOTER_HEIGHT;

        // **The Chapters view is the one view that draws the bar of the book**
        // (T-333), and the areas of the mouse are the areas of the last frame:
        // a bar of the frame before this one would take a click of another
        // view. The view writes the area again while it draws that bar.
        self.the_areas_of_the_mouse.the_bar_of_the_book = Rect::default();

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

        // **The band of the player stands under the work of the view**, and the
        // view of this frame wrote the rows of its footer already: the layout of
        // the band and the layout of the view therefore hold one number of every
        // row. See T-322.
        self.render_the_band_of_the_player(area, buf);

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
///
/// **The footer stands on the rows that it needs** (T-302), therefore the caller
/// measures its own text with `crate::ui::keys::the_rows_of_a_footer` and it
/// gives the answer here. A footer of more than two rows grows upward over the
/// work of the view, in the same way as the message of T-299: a list of the
/// view loses a line, and the key `j` moves the list, therefore no line of it
/// goes out of the reach of the user.
fn the_areas_of_a_view(
    area: Rect,
    rows_of_the_player: u16,
    rows_of_the_footer: u16,
    the_smallest_work: u16,
) -> [Rect; 3] {
    let [header_area, main_area, _band_area, _message_area, footer_area] = the_five_areas(
        area,
        rows_of_the_player,
        rows_of_the_footer,
        the_smallest_work,
    );

    [header_area, main_area, footer_area]
}

/// The area of the band of the player, with its border. See T-322.
///
/// **The band stood at 9 rows above the end of the screen before this stage**,
/// and that number held no footer of more than two rows: `render_player` read
/// the whole screen and it counted backward, therefore a view of a footer of
/// three rows drew its band over its own last line. The band takes the area of
/// the layout of the view now, and the two of them cannot disagree.
///
/// **Fifteen views built their layout themselves** (T-343), with the header,
/// the work of the view, and the footer, and they gave the band no row of it:
/// the band then drew over the last six lines of the reader of an ebook, of the
/// Chapters view, of the view of every key, and of the twelve views beside them.
/// Every view of this program takes its areas of [`the_areas_of_a_view`] now.
fn the_area_of_the_band(
    area: Rect,
    rows_of_the_player: u16,
    rows_of_the_footer: u16,
    the_smallest_work: u16,
) -> Rect {
    the_five_areas(
        area,
        rows_of_the_player,
        rows_of_the_footer,
        the_smallest_work,
    )[2]
}

/// The five areas of the screen of a view: the header, the work of the view,
/// the band of the player, the row of the message, and the footer.
fn the_five_areas(
    area: Rect,
    rows_of_the_player: u16,
    rows_of_the_footer: u16,
    the_smallest_work: u16,
) -> [Rect; 5] {
    let [rows_of_the_header, rows_of_the_message, rows_that_the_footer_keeps] =
        the_rows_around_the_work_of_a_view(area.height, rows_of_the_footer, the_smallest_work);

    Layout::vertical([
        Constraint::Length(rows_of_the_header),
        Constraint::Fill(1),
        Constraint::Length(the_rows_of_the_band_of_a_screen(
            area.height,
            rows_of_the_player,
            rows_of_the_footer,
            the_smallest_work,
        )),
        Constraint::Length(rows_of_the_message),
        Constraint::Length(rows_that_the_footer_keeps),
    ])
    .areas(area)
}

/// The rows of the header, of the row of the message, and of the footer of a
/// screen of few rows. See T-345.
///
/// **The work of a view goes away last** (T-342, T-343, and T-344): the row of
/// the item, the panel of the item, the band of the player, and the two bars of
/// the Chapters view give way before it already. The three parts that stand
/// around the view took their rows of the screen first, because each of them is
/// a `Constraint::Length` and the work of the view is the `Constraint::Fill`:
/// the measurement of 2026-08-17, of the real program v0.8.175 in a terminal of
/// 100 columns, read the header and the footer alone at 5 rows, the footer
/// alone at 3 rows, and **a screen with no letter at all at 1 row**.
///
/// The three parts give way in the sequence of what they say to the user:
///
/// 1. **The header goes away first**, one row at a time. It says the account,
///    the address of the server, the library, and the name of the program, and
///    the settings screen and the view of the accounts say those values too.
/// 2. **The row of the message goes away after it.** A message of the program
///    still reaches the user: `render_the_message` writes it over the work of
///    the view (the trap 39 and T-299), therefore this row is the room that the
///    message does not take away from the view, and not the voice itself.
/// 3. **The footer goes away last**, because it names the keys of the view and
///    no other screen of this program names them.
///
/// A screen of 7 rows and more holds the three of them whole, therefore this
/// rule reaches a terminal of 6 rows and fewer alone. **The rows that the work
/// of the view keeps come of `the_smallest_work`** (T-347), because the panel 4
/// of the frame of the panels holds a border of four sides.
fn the_rows_around_the_work_of_a_view(
    rows_of_the_screen: u16,
    rows_of_the_footer: u16,
    the_smallest_work: u16,
) -> [u16; 3] {
    // The work of the view keeps its border and one line before every part
    // that stands around it.
    let mut room = rows_of_the_screen.saturating_sub(the_smallest_work);

    let footer = rows_of_the_footer.min(room);
    room -= footer;

    let message = 1.min(room);
    room -= message;

    [HEADER_HEIGHT.min(room), message, footer]
}

/// The place of the rows of the message of a frame: the first row, and the
/// number of rows. See T-346.
///
/// **The message stands on the row above the footer, and it grows upward over
/// the work of the view** (T-299 and the trap 39). The two numbers of that
/// place are the rows of the header and the rows of the footer of **this**
/// frame, and [`the_rows_around_the_work_of_a_view`] is the one function that
/// holds them: `render_the_message` read `HEADER_HEIGHT` and the rows that the
/// **text** of the footer wants, and the two of them disagree with the frame on
/// a screen of three rows and fewer.
///
/// The measurement of 2026-08-17, of the real program v0.8.176 inside tmux in a
/// terminal of 100 columns, of the message of the key `Ctrl+o`:
///
/// * At **3 rows** the message stood on the row 0, over the title
///   `Home [20 items]` of the list, and the line of the list under it stayed:
///   the footer keeps one row of such a screen and the message read two.
/// * At **2 rows** and at **1 row** the program said **nothing at all**: the
///   footer of two rows that the message read holds every row of the screen,
///   therefore `the_place_of_a_message` gave `None`.
///
/// A screen of 6 rows and more holds the header, the row of the message, and
/// the footer whole, therefore this rule reaches a terminal of 5 rows and fewer
/// alone. At 5 rows and at 4 rows the header of the frame keeps no row, and the
/// message therefore takes the rows of that header for the whole of its
/// sentence: a message that the screen cuts says nothing (T-299), and the
/// header is away already.
fn the_place_of_the_message_of_a_frame(
    area: Rect,
    rows_of_the_footer: u16,
    rows_that_it_needs: u16,
    the_smallest_work: u16,
) -> Option<(u16, u16)> {
    let [rows_of_the_header, _, rows_that_the_footer_keeps] =
        the_rows_around_the_work_of_a_view(area.height, rows_of_the_footer, the_smallest_work);

    crate::logic::message::the_place_of_a_message(
        area.y,
        area.height,
        rows_of_the_header,
        rows_that_the_footer_keeps,
        rows_that_it_needs,
    )
}

/// The rows that the band of the player keeps on a screen of few rows. See
/// T-343.
///
/// The band says the media that plays, the place of the user, and the keys of
/// the player, and it stands under the work of the view.
///
/// **The work of the view goes away last** (T-342): a `Constraint::Length`
/// stands before a `Constraint::Fill` in the solver of ratatui, therefore the
/// band took its rows away from the screen first and the view took what stayed.
/// The band leaves the view its border and one line (`THE_SMALLEST_LIST`).
///
/// **A band of fewer than [`THE_SMALLEST_BAND`] rows says nothing at all**: its
/// two rows of the border then take the room and the user reads no media and no
/// place. Such a band goes away, and the view takes every row of it.
///
/// A screen that held the whole band keeps it, therefore this rule reaches a
/// terminal of few rows alone.
fn the_rows_of_the_band_of_a_screen(
    rows_of_the_screen: u16,
    rows_of_the_band: u16,
    rows_of_the_footer: u16,
    the_smallest_work: u16,
) -> u16 {
    // The header, the row of the message, the footer, and the smallest work of
    // a view stand before the band.
    let of_the_others = HEADER_HEIGHT
        .saturating_add(1)
        .saturating_add(rows_of_the_footer)
        .saturating_add(the_smallest_work);

    let rows = rows_of_the_band.min(rows_of_the_screen.saturating_sub(of_the_others));

    if rows < THE_SMALLEST_BAND {
        return 0;
    }

    rows
}

/// The rows that the band of the player needs for one word of the media: the
/// two rows of its border, and the row of the title and of the author. See
/// T-343.
const THE_SMALLEST_BAND: u16 = 3;

/// The rows of the two bars of the Chapters view that the list of that view
/// leaves them: the bar of the book, the bar of the chapter, and one row of
/// nothing under the two of them. See T-330.5 and T-343.
///
/// **The list of a view goes away last** (T-342): a `Constraint::Length` stands
/// before a `Constraint::Fill` in the solver of ratatui, therefore the bars took
/// their three rows away from the panel first and the list took what stayed. The
/// bars leave the list its border and one line (`THE_SMALLEST_LIST`).
///
/// A panel of 5 rows and more keeps the three rows that it had, therefore this
/// rule reaches a terminal of few rows alone.
fn the_rows_of_the_bars_of_the_chapters(rows_of_the_panel: u16, the_bars_stand: bool) -> u16 {
    if !the_bars_stand {
        return 0;
    }

    // The bar of the book stands first, the bar of the chapter after it, and
    // the row of nothing last.
    rows_of_the_panel.saturating_sub(THE_SMALLEST_LIST).min(3)
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
///
/// **The row of the item took the last lines of the list** (T-342). A
/// `Constraint::Length` stands before a `Constraint::Fill` in the solver of
/// ratatui, therefore the two rows of the row of the item came away from the
/// screen first and the list took what stayed. The measurement of 2026-08-17,
/// of a terminal of 100 by 8: this area held 3 rows, the row of the item took
/// 2 of them, and the list held its border and **no line at all** while its
/// title said `Library [500 items of 2056]`.
///
/// **The list is the work of the view, and it goes away last**: the row of the
/// item takes no row that the list needs for its border and one line
/// (`the_smallest_work`). A screen that held the two rows of that row keeps
/// them, therefore this rule reaches a terminal of 8 rows and fewer alone.
fn the_areas_of_a_list(
    main_area: Rect,
    rows_that_the_player_left: u16,
    the_smallest_work: u16,
) -> [Rect; 3] {
    // 13 rows give the list 5 lines with the split of a large terminal. Fewer
    // rows than that give every row to the list: the lines are the work of the
    // view, and a description of one row says almost nothing.
    if main_area.height.saturating_sub(rows_that_the_player_left) <= 12 {
        return Layout::vertical([
            Constraint::Fill(1),
            // The row of the item takes two rows, because its text wraps in a
            // terminal of 80 columns. See T-94. It takes fewer of them, and
            // none at all, before the list loses its one line. See T-342.
            Constraint::Length(the_rows_of_the_row_of_the_item(
                main_area.height,
                the_smallest_work,
            )),
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

/// The rows that a list of a border at the top keeps for itself: that border
/// with the title, and one line. See T-342.
///
/// A list of this shape stands in a `Block` of `Borders::TOP`, therefore a list
/// of one line needs two rows. **The panel 4 of the frame of the panels holds a
/// border of four sides**, and it therefore needs one row more: that number is
/// [`the_smallest_work_of_a_view`], and no function of the rows of a screen
/// reads this constant by itself (T-347).
const THE_SMALLEST_LIST: u16 = 2;

/// The rows that the work of a view needs for its border and one line. See
/// T-347.
///
/// **The panel 4 of the frame of the panels holds a border of four sides**
/// (T-320), and the block of every other view of this program holds the one
/// border at the top that it had: a list of one line therefore needs three rows
/// of the frame and two rows outside it.
///
/// The measurement of 2026-08-17, of the real program v0.8.177 inside tmux at
/// **160 columns**, where the frame of the panels stands: the panel 4 of the
/// Home view held its two rows of the border and **no line at all** at 6 rows
/// and fewer, and at 8 rows the panel of the item under it still held a row of
/// its own. Every measurement of the rows of a screen before this one ran at
/// 100 columns, where the frame does not stand (T-342 to T-346).
fn the_smallest_work_of_a_view(the_frame_of_the_panels_stands: bool) -> u16 {
    if the_frame_of_the_panels_stands {
        // The border of the panel 4 takes one row above the lines and one row
        // under them.
        return THE_SMALLEST_LIST + 1;
    }

    THE_SMALLEST_LIST
}

/// The rows of the row of the item under a list of few rows. See T-342.
///
/// The row of the item says the author, the year, the length, and the place of
/// the user of the line of the cursor. **It says nothing at all while the list
/// holds no line**, because the cursor then stands on no line that the user
/// sees: it therefore takes its two rows out of what the list does not need.
///
/// A view of 4 rows and more keeps the two rows that it had.
fn the_rows_of_the_row_of_the_item(rows_of_the_view: u16, the_smallest_work: u16) -> u16 {
    // The row of the item takes two rows, because its text wraps in a terminal
    // of 80 columns. See T-94.
    the_rows_of_the_panel_of_the_item(rows_of_the_view, 2, the_smallest_work)
}

/// The rows of the panel of the item under a list of few rows. See T-344.
///
/// **The list of a view is the work of that view, and it goes away last**
/// (T-342). Seven views of this program hold the panel of the item under their
/// list as a `Constraint::Length` of 4 or of 5 with the list as a
/// `Constraint::Fill`, therefore the solver of ratatui gave that panel its rows
/// first and the list took what stayed. The measurement of 2026-08-17, of the
/// real program v0.8.174 in a terminal of 100 columns and 8 rows: the Authors
/// view of a library of nine authors said `No description available` and
/// **nothing else** — no title, and no author at all.
///
/// The panel says the words of the line of the cursor, and **it says nothing at
/// all while the list holds no line**, because the cursor then stands on no
/// line that the user sees: it therefore takes the rows that it wants out of
/// what the list does not need. A view that held the whole panel keeps it,
/// therefore this rule reaches a terminal of few rows alone.
fn the_rows_of_the_panel_of_the_item(
    rows_of_the_view: u16,
    rows_that_it_wants: u16,
    the_smallest_work: u16,
) -> u16 {
    rows_of_the_view
        .saturating_sub(the_smallest_work)
        .min(rows_that_it_wants)
}

impl App {
    /// The rows that the work of this view needs for its border and one line.
    /// See T-347.
    ///
    /// The panel 4 of the frame of the panels draws a border of four sides, and
    /// `render_the_list_of_the_panel_4` draws that panel while
    /// [`App::the_frame_of_the_panels_stands`] answers `true`: the two functions
    /// therefore read one condition.
    fn the_smallest_work_of_the_view(&self) -> u16 {
        the_smallest_work_of_a_view(self.the_frame_of_the_panels_stands())
    }

    /// The three areas of this view: the header, the work of the view, and the
    /// footer. See T-343 and T-347.
    ///
    /// Every view of this program takes its areas here, therefore the band of
    /// the player, the row of the message, and the work of the view cannot
    /// disagree on one row of the screen.
    fn the_areas_of_this_view(&self, area: Rect, rows_of_the_footer: u16) -> [Rect; 3] {
        the_areas_of_a_view(
            area,
            self.the_rows_of_the_band(),
            rows_of_the_footer,
            self.the_smallest_work_of_the_view(),
        )
    }

    /// Draws the newest message of the program, if one is fresh.
    ///
    /// The message stands after the view and before the bar of the downloads: a
    /// download is the work that the user waits for, therefore that bar keeps
    /// its rows.
    ///
    /// **The message stands on the rows that it needs** (T-299). The row of the
    /// message held one row and it cut every sentence that was longer than the
    /// screen: the measurement of 2026-08-16 logged out of the account that
    /// starts the program, and the Home view of the account after it said
    /// `… Log in again with the same name and the same…` at 160 columns. The
    /// road back of that sentence stood outside the screen.
    ///
    /// The last row of the message stays where one row of a message stood, and
    /// the rows before it grow upward over the view (the trap 39). The header of
    /// the screen keeps its rows: a message that needs more rows than that room
    /// loses its end to `in_the_rows`, and the log holds the whole of it.
    fn render_the_message(&self, area: Rect, buf: &mut Buffer) {
        // **The message names the view of the user**: a rule of the loop writes
        // a message of a view with no key of the user, and that message waits
        // for its view. See T-164.
        let Some(text) = crate::logic::message::for_the_screen(self.view_state) else {
            return;
        };

        // The rows of the message stand above the rows of the footer. A screen
        // that holds no such row draws no message.
        let rows_that_it_needs = crate::logic::message::the_rows_of_a_message(&text, area.width);

        let Some((y, rows)) = the_place_of_the_message_of_a_frame(
            area,
            self.rows_of_the_footer,
            rows_that_it_needs,
            self.the_smallest_work_of_the_view(),
        ) else {
            return;
        };

        let row = Rect {
            x: area.x,
            y,
            width: area.width,
            height: rows,
        };

        let style = Style::default()
            .bg(self.config.colors.header_background())
            .fg(self.config.colors.line_header())
            .add_modifier(Modifier::BOLD);

        App::draw_the_row_of_the_message(
            row,
            buf,
            &crate::logic::message::in_the_rows(&text, area.width, rows),
            style,
        );
    }

    /// Draws the rows of the message over the view that stands below them.
    ///
    /// **The rows must hold the message and nothing else.** A `Paragraph` gives
    /// its style to every cell of its area, and it writes its own text only:
    /// every letter that stood on that row before it stays. A measurement of
    /// 2026-08-11 read "CHAPTER IV.  │The Rabbit SeThe server has the place of
    /// the book." in the reader. `Clear` takes the rows away first. See T-78.
    ///
    /// The wrap of the paragraph gives the message its second row and every row
    /// after it, and `the_rows_of_a_message` measured them already (T-299).
    fn draw_the_row_of_the_message(row: Rect, buf: &mut Buffer, text: &str, style: Style) {
        Clear.render(row, buf);

        Paragraph::new(text.to_string())
            .centered()
            .wrap(Wrap { trim: true })
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

            let label = the_label_of_a_download(item);

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

/// Gives the label of the copy of the disk of one media. See T-203 and T-204.
///
/// **A read of the disk that failed is not a media with no copy on the disk**
/// (T-203), and **a read of the disk that stands inside the render is a read of
/// every frame** (T-204): this row asked the database at each frame, and a
/// second program of the account that holds the write lock then took the thread
/// of the screen for five seconds of each of those frames.
///
/// The box of `crate::logic::the_copies_of_the_disk` holds the answer of the
/// disk, and the program reads the disk at the moments that it needs it.
fn the_copy_of_the_disk(key: &str) -> &'static str {
    crate::ui::keys::the_label_of_the_copy_of_the_disk(
        crate::logic::the_copies_of_the_disk::the_copy_of_this_media(key),
    )
}

/// The words of the bar of one download.
///
/// **A total of 0 is a total that the server did not give** (T-179): every file
/// of the plan then holds the size 0, and the old label said
/// "0.0 MB / 0.0 MB" while the program wrote the bytes of the book. The label
/// holds the bytes of the disk alone in that condition, because a bar cannot
/// show a part of a whole that the program does not have.
fn the_label_of_a_download(item: &DownloadProgress) -> String {
    let done = megabytes(item.bytes_done);

    let whole = match item.bytes_total {
        0 => done,
        total => format!("{} / {}", done, megabytes(total)),
    };

    if item.file_count > 1 {
        return format!(
            " ⬇ {}  file {}/{}  {} ",
            shorten(&item.title, 28),
            item.file_index,
            item.file_count,
            whole,
        );
    }

    format!(" ⬇ {}  {} ", shorten(&item.title, 34), whole)
}

/// Changes a number of bytes to a text in megabytes.
///
/// The function stands in `crate::ui::keys` since T-284, because the reader
/// says a size too and it holds no part of this file.
fn megabytes(bytes: u64) -> String {
    crate::ui::keys::megabytes(bytes)
}

/// Makes a text shorter, with three points for the end that goes away.
///
/// **`crate::logic::message::in_one_row` is the one maker of a text of one row
/// of this program** (T-304).
fn shorten(text: &str, width: usize) -> String {
    crate::logic::message::in_one_row(text, u16::try_from(width).unwrap_or(u16::MAX))
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

    /// Every line of the facts of the design, for the panel 5 of the cover.
    /// See T-325 and T-326.
    ///
    /// **The facts of the design belong to a book of the Library view and to a
    /// book of the Home view.** The two views draw the panel 4 of the list and
    /// the panel 5 of the cover, and the answer of the personalized view holds
    /// the same six facts as the answer of the items of a library: a panel of
    /// the Home view that said `Author: … - Year: N/A - Duration: 30m` over two
    /// rows, with 15 rows of no character under it, was the fault of T-326.
    ///
    /// The other views of this program keep the two lines of today: the lists
    /// of a row of a search, of a collection, and of the episodes of a podcast
    /// hold no narrator, no genre, and no ebook, therefore a panel of the
    /// design there would say the same two lines over more rows.
    ///
    /// `width` is the width of the inside of the panel, and it gives the bar of
    /// the progress its cells.
    ///
    /// The function gives `None` for every view and for every row that the
    /// facts of the design do not reach, and the panel then keeps the rows that
    /// it held before this stage.
    fn the_lines_of_the_facts_of_the_panel(&self, width: u16) -> Option<Vec<String>> {
        // **A library of podcasts holds no fact of this design**: its lists of
        // a row hold the episode and not the narrator, the genre, or the ebook.
        if self.is_podcast {
            return None;
        }

        // A line of a series says the number of the books and the whole length,
        // and no book of it stands under the cursor. The two views each hold
        // such a line: the Library view groups the books of a series, and the
        // shelf `recent-series` of the Home view is a shelf of series.
        let (facts, length, of_the_disk, author, year, place, the_day_of_the_start) =
            match self.view_state {
                AppView::Library => {
                    if self.selected_library_series().is_some() {
                        return None;
                    }

                    let selected = self.selected_library_item()?;

                    (
                        self.the_facts_library.get(selected)?,
                        self.duration_library.get(selected).copied(),
                        at(&self.ids_library, selected),
                        at(&self.auth_names_library, selected),
                        at(&self.published_year_library, selected),
                        self.the_place_of_the_panel_of_the_library(selected),
                        self.the_day_of_the_start_of_the_library(selected),
                    )
                }
                AppView::Home => {
                    if self.selected_home_series().is_some() {
                        return None;
                    }

                    let selected = self.selected_home_item()?;

                    (
                        self.the_facts_home.get(selected)?,
                        self.duration_cnt_list.get(selected).copied(),
                        at(&self._ids_cnt_list, selected),
                        at(&self.auth_names_cnt_list, selected),
                        at(&self.pub_year_cnt_list, selected),
                        self.the_place_of_the_panel_of_the_home_view(selected),
                        self.the_day_of_the_start_of_the_home_view(selected),
                    )
                }
                _ => return None,
            };

        let length = length
            .map(|seconds| convert_seconds(vec![seconds]))
            .and_then(|words| words.first().cloned())
            .unwrap_or_default();

        // **The label of the copy of the disk holds the words of a line of many
        // facts**: it starts with ` - `, because it stands after the length in
        // the line of today. A line of one fact takes the words alone.
        let of_the_disk = the_copy_of_the_disk(of_the_disk)
            .trim_start()
            .trim_start_matches("- ");

        Some(crate::logic::the_facts_of_a_media::the_lines_of_the_facts(
            &crate::logic::the_facts_of_a_media::TheMediaOfThePanel {
                facts,
                author,
                year,
                length: &length,
                of_the_disk,
                percent: &place.percent,
                the_time_that_is_left: &place.the_time_that_is_left,
                the_end: &place.the_end,
                the_day_of_the_start: &the_day_of_the_start,
            },
            width,
        ))
    }

    /// The text of the description that the panel 5 draws for the media of the
    /// cursor. See T-330.3.
    ///
    /// **The picture takes every row that the facts and the description
    /// leave**, therefore the layout of the panel needs the text before the
    /// render of it. The eight views of `render_covers` each hold a render of
    /// their own for that text, and this function names the same source for
    /// each of them.
    ///
    /// A view that draws no such description gives an empty text, and the
    /// picture then takes every row that the facts leave.
    fn the_description_of_the_panel(&self) -> String {
        match self.view_state {
            AppView::Home => {
                if let Some(series) = self.selected_home_series() {
                    return series.description_for_the_screen();
                }

                let Some(selected) = self.selected_home_item() else {
                    return String::new();
                };

                if self.is_podcast {
                    crate::logic::the_panel_of_a_line::the_description_of_a_podcast(
                        at(&self.subtitles_pod_cnt_list, selected),
                        at(&self.descs_pod_cnt_list, selected),
                    )
                } else {
                    at(&self.desc_cnt_list, selected).to_string()
                }
            }
            AppView::Library => {
                if let Some(series) = self.selected_library_series() {
                    return series.description_for_the_screen();
                }

                self.selected_library_item()
                    .and_then(|index| self.desc_library.get(index).cloned())
                    .unwrap_or_default()
            }
            AppView::Series => self
                .selected_series()
                .map(|series| series.description_for_the_screen())
                .unwrap_or_default(),
            AppView::SeriesBook => self
                .selected_series_book()
                .map(|book| book.description_for_the_screen())
                .unwrap_or_default(),
            AppView::Lists => self
                .selected_list()
                .map(|list| list.description.clone())
                .unwrap_or_default(),
            AppView::ListEntries => self
                .selected_list_entry()
                .map(|entry| entry.description.clone())
                .unwrap_or_default(),
            AppView::SearchBook => self
                .list_state_search_results
                .selected()
                .map(|selected| at(&self.desc_library_search_book, selected).to_string())
                .unwrap_or_default(),
            AppView::PodcastEpisode => self
                .list_state_pod_ep
                .selected()
                .map(|selected| {
                    crate::logic::the_panel_of_a_line::the_description_of_a_podcast(
                        at(&self.subtitles_pod_ep, selected),
                        at(&self.descs_pod_ep, selected),
                    )
                })
                .unwrap_or_default(),
            _ => String::new(),
        }
    }

    /// The media of the panel 5 of the cover: the media that plays, and the
    /// media of the list around the cursor.
    ///
    /// **The selection needs no second cover when it is the media that
    /// plays.** The panel then shows one cover, and that cover is large.
    ///
    /// The render of the panel reads this, **and the layout of the frame reads
    /// it before the render** (T-348): the width of the panel comes of the
    /// picture that stands in it, therefore `split_for_covers` must know
    /// whether a picture comes at all.
    fn the_media_of_the_panel_of_the_cover(&self) -> (Option<String>, Vec<String>) {
        let playback = self.player.state();
        let playing = if playback.status == PlaybackStatus::Stopped || playback.item_id.is_empty() {
            None
        } else {
            Some(playback.item_id.clone())
        };

        let selected: Vec<String> = self
            .cover_ids()
            .into_iter()
            .filter(|id| !id.is_empty() && Some(id.as_str()) != playing.as_deref())
            .take(cover::SHELF_MAX)
            .collect();

        (playing, selected)
    }

    /// Says if one media of the panel 5 has a picture that the panel can draw.
    ///
    /// **A picture that the program did not ask for yet is not a media with no
    /// picture** (T-319): `cover::no_picture_comes` reads the two states of the
    /// store that no second request asks for, and no other value.
    fn a_picture_comes_of(playing: &Option<String>, selected: &[String]) -> bool {
        playing
            .iter()
            .chain(selected.iter())
            .any(|id| !cover::no_picture_comes(id))
    }

    /// Says if a picture stands in the panel 5 of the cover of this frame.
    ///
    /// **The width of that panel comes of the height of the picture in it**
    /// (T-50), therefore the layout needs this answer before the render of the
    /// panel. A panel of the words alone takes no such limit (T-348).
    fn a_picture_comes_in_the_panel_of_the_cover(&self) -> bool {
        let (playing, selected) = self.the_media_of_the_panel_of_the_cover();
        Self::a_picture_comes_of(&playing, &selected)
    }

    /// Says if the panel 5 of the cover holds the identity of one media at
    /// least. See T-354.
    ///
    /// **A media with no cover is a media of this panel** (T-319): the words of
    /// it stand in the panel, therefore this answer is not
    /// `a_picture_comes_in_the_panel_of_the_cover`. A view of no media at all
    /// gives `false`, and the panel then takes no column of the screen.
    fn a_media_of_the_panel_of_the_cover_comes(&self) -> bool {
        let (playing, selected) = self.the_media_of_the_panel_of_the_cover();

        playing.is_some() || !selected.is_empty()
    }

    /// Draws the panel 5 of the cover, and it gives the area of the facts of
    /// the media and the area of the description of it. See T-319.
    ///
    /// The function draws nothing and it gives `None` when the screen is too
    /// narrow, because `split_for_covers` then gives no panel. It gives `None`
    /// for a panel that is not tall too: the words of the media then stay under
    /// the list, where they stood before this stage.
    ///
    /// **The panel of the covers of T-23 stood in the air**: it had no border,
    /// no title, and no number, therefore no key and no click of the user could
    /// name it, and the rows under the picture held nothing at all. The frame
    /// of T-320 gives it the border of a panel now, and the words of the media
    /// fill the rows that the picture leaves.
    fn render_covers(&mut self, panel: Option<Rect>, buf: &mut Buffer) -> Option<(Rect, Rect)> {
        let Some(column) = panel else {
            // **A frame that draws no panel takes no click of it** (T-316): the
            // areas of the last frame are the screen that the user clicked.
            self.the_areas_of_the_mouse.the_panel_of_the_cover = Rect::default();
            self.render_the_gallery(None, buf);
            return None;
        };

        // **The panel 6 of the gallery stands under the panel 5 of the cover**
        // (T-327), and it belongs to the frame of the panels: the Home view and
        // the Library view are the two views that hold that frame, therefore
        // every other view keeps the column that it had.
        let (playing, selected) = self.the_media_of_the_panel_of_the_cover();

        // **A picture that never comes must take no row of the screen**
        // (T-319): the server holds some media with no cover at all, and the
        // panel of those media held 50 columns and 41 rows of nothing.
        let a_picture_comes = Self::a_picture_comes_of(&playing, &selected);

        // **The facts of the design take the rows that they need** (T-325): the
        // panel of a book of the Library view says one fact of one line, and
        // three rows hold no such list.
        //
        // **The gallery reads this number too** (T-350), therefore the panel 5
        // keeps the rows of the whole of its facts before the panel 6 takes
        // any. The width of the panel comes of `split_for_covers` and the
        // gallery divides the height alone, therefore the words of the panel
        // wrap at this width whether the gallery stands or not.
        let of_the_facts = self
            .the_lines_of_the_facts_of_the_panel(column.width.saturating_sub(2))
            .map(|lines| lines.len() as u16)
            .unwrap_or(crate::ui::the_panel_of_the_cover::THE_ROWS_OF_THE_FACTS);

        // **The picture takes every row that the facts and the description
        // leave** (T-330.3), therefore the panel needs the rows of the text of
        // the description before it draws anything.
        //
        // **The gallery reads this number too** (T-353), therefore it stands
        // before the two panels: the width of the panel is the width of the
        // column, because `the_two_panels` divides the height alone.
        let of_the_description = u16::try_from(
            crate::logic::the_scroll_of_a_panel::the_number_of_the_lines(
                &self.the_description_of_the_panel(),
                column.width.saturating_sub(2),
            ),
        )
        .unwrap_or(u16::MAX);

        let of_a_cell = crate::ui::the_panel_of_the_gallery::THE_WIDTHS_OF_A_CELL
            [self.the_size_of_a_cell_of_the_gallery];
        let (panel, gallery) = if self.the_frame_of_the_panels_stands() {
            crate::ui::the_panel_of_the_gallery::the_two_panels(
                column,
                of_a_cell,
                cover::picker().font_size(),
                a_picture_comes,
                of_the_facts,
                of_the_description,
            )
        } else {
            (column, None)
        };

        let it_holds_the_focus =
            self.the_panel_of_the_focus == crate::ui::frame::ThePanel::TheCover;
        let block = crate::ui::frame::a_panel(
            crate::ui::frame::ThePanel::TheCover.the_number(),
            "Cover",
            it_holds_the_focus,
        );
        let inside = block.inner(panel);
        block.render(panel, buf);

        self.the_areas_of_the_mouse.the_panel_of_the_cover = panel;

        // The large cover takes the form of its own picture. Therefore a cover
        // that is higher than it is wide takes the whole height of the panel.
        // See T-50.
        let large = playing
            .as_deref()
            .or(selected.first().map(|id| id.as_str()))
            .and_then(|id| self.covers.form_of(id));

        let parts = crate::ui::the_panel_of_the_cover::the_parts_of_the_panel(
            inside,
            a_picture_comes,
            of_the_facts,
            of_the_description,
        );

        let api = std::sync::Arc::clone(&self.api);

        if let Some(area) = parts.cover {
            let plan = cover::plan_covers(
                area,
                cover::picker().font_size(),
                playing.is_some(),
                selected.len(),
                large,
            );

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

        self.render_the_gallery(gallery, buf);

        parts
            .the_words_stand_here()
            .then_some((parts.facts, parts.description))
    }

    /// Draws the title of a media in the rows of the picture of a cell. See
    /// T-339.
    ///
    /// **The cell of a band and the cell of the gallery are one picture in one
    /// border** (T-336), therefore the two of them say a title in one way. The
    /// words take the quiet style of the program, because the title of a cell
    /// is not the title of a panel.
    fn render_the_title_of_a_cell(title: &str, of_the_picture: Rect, buf: &mut Buffer) {
        if title.is_empty() || of_the_picture.width == 0 || of_the_picture.height == 0 {
            return;
        }

        Paragraph::new(crate::ui::the_panel_of_the_gallery::the_title_of_a_cell(
            title,
            of_the_picture,
        ))
        .wrap(Wrap { trim: true })
        .style(crate::ui::theme::a_quiet_text())
        .render(of_the_picture, buf);
    }

    /// Draws the panel 6 of the gallery of the covers. See T-327.
    ///
    /// **The gallery is the list of the panel 4 and no list of its own**: the
    /// grid holds the media around the cursor, the cell of the cursor takes the
    /// heavy border of the focus, and a click of a cell moves that cursor.
    ///
    /// **A cell holds the picture and its border alone** (T-330.4): the row of
    /// the percentage and the row of the title went away, because the panel 5
    /// says the facts of the media of the cursor already.
    ///
    /// **A panel that the frame did not draw takes no click of it** (T-316),
    /// therefore the area of no panel and the grid of no cell go in the state of
    /// the last frame together.
    fn render_the_gallery(&mut self, panel: Option<Rect>, buf: &mut Buffer) {
        use crate::ui::the_panel_of_the_gallery::{plan_the_gallery, TheGallery};

        let Some(panel) = panel else {
            self.the_areas_of_the_mouse.the_panel_of_the_gallery = Rect::default();
            self.the_gallery_of_the_last_frame = TheGallery::default();
            return;
        };

        let it_holds_the_focus =
            self.the_panel_of_the_focus == crate::ui::frame::ThePanel::TheGallery;
        let block = crate::ui::frame::a_panel(
            crate::ui::frame::ThePanel::TheGallery.the_number(),
            "Gallery",
            it_holds_the_focus,
        );
        let inside = block.inner(panel);
        block.render(panel, buf);

        self.the_areas_of_the_mouse.the_panel_of_the_gallery = panel;

        let the_media = self.the_media_of_the_gallery();
        let the_cursor = self.the_media_of_the_cursor_of_the_gallery(&the_media);
        let of_a_cell = crate::ui::the_panel_of_the_gallery::THE_WIDTHS_OF_A_CELL
            [self.the_size_of_a_cell_of_the_gallery];
        let font = cover::picker().font_size();

        let plan = plan_the_gallery(inside, of_a_cell, font, the_media.len(), the_cursor);
        let api = std::sync::Arc::clone(&self.api);

        for cell in &plan.cells {
            let Some(media) = the_media.get(cell.the_media) else {
                continue;
            };

            // **The cell of the cursor says which media the panel 5 shows**:
            // the two panels of the column then say one media, and a user who
            // reads the facts of the panel 5 finds its cover in the grid.
            //
            // **The border of the cell of the cursor is heavy and bright, and
            // the border of every other cell is thin and dim** (T-330.4): a
            // colour alone is not the mark of the focus, because a terminal of
            // a theme of few colours draws the two of them near together.
            let of_the_cursor = cell.the_media == the_cursor;
            let border = if of_the_cursor {
                Block::new()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Thick)
                    .border_style(
                        Style::default()
                            .fg(crate::ui::theme::THE_ACCENT)
                            .add_modifier(Modifier::BOLD),
                    )
            } else {
                Block::new()
                    .borders(Borders::ALL)
                    .border_style(crate::ui::theme::a_quiet_text())
            };
            border.render(cell.the_box, buf);

            // The picture keeps the form of the cover, therefore it stands in
            // the middle of the rows that the cell gives it.
            //
            // **A cell that no picture reaches holds the title of its media**
            // (T-339): the terminal of no protocol of pictures, the
            // `TOUTUI_NO_COVERS` of the user, and a media that the server holds
            // with no cover each gave a border and nothing at all.
            if media.id.is_empty() || cover::no_picture_comes(&media.id) {
                App::render_the_title_of_a_cell(&media.the_title, cell.the_picture, buf);
                continue;
            }

            let form = self.covers.form_of(&media.id).unwrap_or(1.0);
            let area = cover::box_of_the_picture(cell.the_picture, font, form);

            if area.width > 0 && area.height > 0 {
                if let Some(picture) = self.covers.picture(&api, &media.id) {
                    StatefulImage::default().render(area, buf, picture);
                }
            }
        }

        self.the_gallery_of_the_last_frame = plan;
    }

    /// The three areas of the work of a view of a list, with the panel 5 of the
    /// cover beside it. See T-319.
    ///
    /// **The words of a media stand in one place of one frame**: a panel 5 that
    /// says the facts and the description takes them away from the area under
    /// the list, and the list then holds every row of its column. A panel that
    /// is too small for them, and a screen that draws no panel at all, leave
    /// them under the list.
    fn the_areas_of_a_list_and_the_panel(
        &self,
        main_area: Rect,
        the_words: Option<(Rect, Rect)>,
    ) -> [Rect; 3] {
        match the_words {
            Some((facts, description)) => [main_area, facts, description],
            None => the_areas_of_a_list(
                main_area,
                self.the_rows_that_the_player_left(),
                self.the_smallest_work_of_the_view(),
            ),
        }
    }
}

/// The reader of an ebook. See T-10.
impl App {
    /// Draws the reader. Every line that the view of a list gives its list goes
    /// to the book, because a book needs every line that the terminal has.
    ///
    /// **The band of the player stands under the book** (T-343): the reader
    /// took the whole screen under its header, therefore the band drew over six
    /// lines of the page and the line under it went on with the text. The
    /// footer of the reader stands in the footer of the frame now, and the row
    /// of the message stands above it.
    fn render_reader(&mut self, area: Rect, buf: &mut Buffer) {
        // The task that opens a book puts it in a place of the process. The
        // screen takes it here.
        self.take_the_book();

        // A view with no book keeps two lines for the footer. A screen that
        // names no key looks like a program that stopped: the user of
        // 2026-08-11 had to start the program again. See T-52.
        if self.reader.is_none() {
            // **The footer stands on the rows that it needs** (T-302).
            let text_render_footer = crate::ui::keys::FOOTER_OF_A_FAULT;
            let rows_of_the_footer = self.the_rows_of_the_footer(text_render_footer, area);

            let [header_area, main_area, footer_area] =
                self.the_areas_of_this_view(area, rows_of_the_footer);

            let message = self
                .reader_message
                .clone()
                .unwrap_or_else(|| "No book is open.".to_string());

            self.render_header(header_area, buf);
            App::render_footer(footer_area, buf, text_render_footer);

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

        // **The footer of the reader stands on the rows that it needs** (T-301),
        // and the band of the player stands over the rows that stay (T-343):
        // the reader took the whole screen under its header, therefore the band
        // drew over six lines of the book.
        let keys = self
            .reader
            .as_ref()
            .map(|reader| {
                crate::ui::reader_tui::footer_of(reader.contents_open, reader.holds_pages())
            })
            .unwrap_or(crate::ui::keys::FOOTER_OF_THE_READER);
        let rows_of_the_footer = self.the_rows_of_the_footer(keys, area);

        let [header_area, main_area, footer_area] =
            self.the_areas_of_this_view(area, rows_of_the_footer);

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
            // **The picture of a page of a PDF stands always** (T-54): this
            // arm holds a picture already, therefore the panel takes the limit
            // of the height of T-50.
            Some(_) => {
                cover::split_for_covers(main_area, area.width, cover::picker().font_size(), true)
            }
            None => (main_area, None),
        };

        let Some(reader) = self.reader.as_mut() else {
            return;
        };

        // The task of the render sends the lines. The screen takes them here,
        // and it never waits for them.
        reader.take_the_answer();

        crate::ui::reader_tui::render(reader, text_area, footer_area, buf);

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
        // **The footer stands on the rows that it needs** (T-302): the
        // number of its rows is the number that the wrap of its text needs.
        let text_render_footer = crate::ui::keys::FOOTER_OF_THE_STATISTICS;
        let rows_of_the_footer = self.the_rows_of_the_footer(text_render_footer, area);

        let [header_area, main_area, footer_area] =
            self.the_areas_of_this_view(area, rows_of_the_footer);

        self.render_header(header_area, buf);
        App::render_footer(footer_area, buf, text_render_footer);

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
        // **The footer stands on the rows that it needs** (T-302): the
        // number of its rows is the number that the wrap of its text needs.
        let text_render_footer = crate::ui::keys::FOOTER_OF_THE_SESSIONS;
        let rows_of_the_footer = self.the_rows_of_the_footer(text_render_footer, area);

        let [header_area, main_area, footer_area] =
            self.the_areas_of_this_view(area, rows_of_the_footer);

        self.render_header(header_area, buf);
        App::render_footer(footer_area, buf, text_render_footer);

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

/// The panel of a description of every view. See T-252.
impl App {
    /// Draws the panel of a description, and it keeps the scroll of the user
    /// inside the text of that panel.
    ///
    /// **A panel that scrolled past its last line holds no line at all**, and
    /// the user cannot tell it from a media whose description the server did
    /// not give (T-252). The fourteen panels of this file each read
    /// `self.scroll_offset`, and the key `J` moved that value with no limit.
    ///
    /// The render is the one road to the length of the text: it holds the text
    /// and the size of the panel, and the key of the user holds neither.
    /// Therefore this function keeps the largest scroll of the panel in the box
    /// of `crate::logic::the_scroll_of_a_panel`, and the key reads that box.
    ///
    /// **A panel that holds more text than its rows says so** (T-253): the bar
    /// of the scroll stands at the right of the text, and it takes one
    /// character of the width. A panel that holds the whole of its text takes
    /// no bar and it keeps the whole width.
    ///
    /// **The two ends of the bar name the keys that move the panel** (T-254):
    /// the letter of the key that moves it up stands at the top of the bar, and
    /// the letter of the key that moves it down stands at the foot of it.
    fn render_a_description(&self, area: Rect, buf: &mut Buffer, text: &str) {
        let panel = crate::logic::the_scroll_of_a_panel::the_panel_of_the_render(
            self.scroll_offset,
            text,
            area.width,
            area.height,
        );

        let [text_area, bar_area] = Layout::horizontal([
            Constraint::Length(panel.width_of_the_text),
            Constraint::Fill(1),
        ])
        .areas(area);

        Paragraph::new(text.to_string())
            .scroll((panel.scroll, 0))
            .wrap(Wrap { trim: true })
            .render(text_area, buf);

        if panel.the_bar_comes() {
            // The state of the bar counts the lines of the text that stand
            // above the panel, and not the lines of the text: the thumb of the
            // bar then reaches the foot of it at the last line of the panel.
            let mut state =
                ScrollbarState::new(usize::from(panel.last)).position(usize::from(panel.scroll));

            // **The two ends of the bar name the keys that move the panel**
            // (T-254). The footer of the Home view and of the Library view
            // holds 116 of the 130 characters of the gate of the footers,
            // therefore those words fit in no footer. A bar of few rows keeps
            // its track and it takes no letter.
            let (up, down) = if panel.the_letters_come() {
                (
                    Some(crate::logic::the_scroll_of_a_panel::THE_LETTER_OF_THE_KEY_UP),
                    Some(crate::logic::the_scroll_of_a_panel::THE_LETTER_OF_THE_KEY_DOWN),
                )
            } else {
                (None, None)
            };

            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(up)
                .end_symbol(down)
                .track_symbol(Some("│"))
                .thumb_symbol("█")
                .render(bar_area, buf, &mut state);
        }
    }
}

/// The authors of the library. See T-24.
impl App {
    /// AppView::Authors rendering
    fn render_authors(&mut self, area: Rect, buf: &mut Buffer) {
        // **The footer stands on the rows that it needs** (T-302): the
        // number of its rows is the number that the wrap of its text needs.
        // The view holds the authors or the narrators, and the title says which
        // list it holds. See T-73.
        let kind = crate::logic::authors::kind();
        let text_render_footer =
            crate::ui::keys::footer_with(kind.work_of_the_key_that_opens(), None);
        let rows_of_the_footer = self.the_rows_of_the_footer(&text_render_footer, area);

        let [header_area, work_area, footer_area] =
            self.the_areas_of_this_view(area, rows_of_the_footer);

        // **The list of a view goes away last** (T-342 and T-344): the panel
        // of the item takes no row that the list needs for its border and one
        // line.
        let [main_area, item_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(the_rows_of_the_panel_of_the_item(
                work_area.height,
                4,
                self.the_smallest_work_of_the_view(),
            )),
        ])
        .areas(work_area);

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
        App::render_footer(footer_area, buf, &text_render_footer);
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
            self.render_a_description(
                item_area,
                buf,
                &crate::api::libraries::get_authors::description_of(one),
            );
        }
    }
}

/// The lists that take a media. See T-84.
impl App {
    /// AppView::PutInAList rendering
    fn render_put_in_a_list(&mut self, area: Rect, buf: &mut Buffer) {
        // **The footer stands on the rows that it needs** (T-302): the
        // number of its rows is the number that the wrap of its text needs.
        let text_render_footer = crate::ui::keys::FOOTER_OF_THE_LISTS_THAT_TAKE_A_MEDIA;
        let rows_of_the_footer = self.the_rows_of_the_footer(text_render_footer, area);

        let [header_area, work_area, footer_area] =
            self.the_areas_of_this_view(area, rows_of_the_footer);

        // **The list of a view goes away last** (T-342 and T-344): the panel
        // of the item takes no row that the list needs for its border and one
        // line.
        let [main_area, item_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(the_rows_of_the_panel_of_the_item(
                work_area.height,
                4,
                self.the_smallest_work_of_the_view(),
            )),
        ])
        .areas(work_area);

        // A library holds no list until a user makes the first one. The title
        // says that condition, and it names the two keys that make a list:
        // an empty box says nothing. See T-88.
        let title = match (self.the_media_of_the_list.as_ref(), self.lists.is_empty()) {
            // A server that does not answer takes no new list, and it gave no
            // list either. See T-91. **A request that came back with a fault is
            // a third condition**, and this title says it too (T-169).
            (_, true) => crate::logic::the_lists::the_title_of_no_list(
                self.is_offline,
                crate::logic::the_lists::the_fault_of(&self.id_selected_lib).as_deref(),
            ),
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
        App::render_footer(footer_area, buf, text_render_footer);
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
        // **The footer stands on the rows that it needs** (T-302): the
        // number of its rows is the number that the wrap of its text needs.
        let text_render_footer = crate::ui::keys::FOOTER_OF_THE_DEVICES_OF_AN_EREADER;
        let rows_of_the_footer = self.the_rows_of_the_footer(text_render_footer, area);

        let [header_area, work_area, footer_area] =
            self.the_areas_of_this_view(area, rows_of_the_footer);

        // **The list of a view goes away last** (T-342 and T-344): the panel
        // of the item takes no row that the list needs for its border and one
        // line.
        let [main_area, item_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(the_rows_of_the_panel_of_the_item(
                work_area.height,
                4,
                self.the_smallest_work_of_the_view(),
            )),
        ])
        .areas(work_area);

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
        App::render_footer(footer_area, buf, text_render_footer);
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

        // **The footer stands on the rows that it needs** (T-302): the
        // number of its rows is the number that the wrap of its text needs.
        let text_render_footer = crate::ui::keys::FOOTER_OF_THE_DOWNLOADS;
        let rows_of_the_footer = self.the_rows_of_the_footer(text_render_footer, area);

        let [header_area, work_area, footer_area] =
            self.the_areas_of_this_view(area, rows_of_the_footer);

        // **The list of a view goes away last** (T-342 and T-344): the panel
        // of the item takes no row that the list needs for its border and one
        // line.
        let [main_area, item_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(the_rows_of_the_panel_of_the_item(
                work_area.height,
                4,
                self.the_smallest_work_of_the_view(),
            )),
        ])
        .areas(work_area);

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
        App::render_footer(footer_area, buf, text_render_footer);
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
        // **The footer stands on the rows that it needs** (T-302): the
        // number of its rows is the number that the wrap of its text needs.
        let text_render_footer = crate::ui::keys::footer_with("read this book", None);
        let rows_of_the_footer = self.the_rows_of_the_footer(&text_render_footer, area);

        let [header_area, work_area, footer_area] =
            self.the_areas_of_this_view(area, rows_of_the_footer);

        // **The list of a view goes away last** (T-342 and T-344): the panel
        // of the item takes no row that the list needs for its border and one
        // line.
        let [main_area, item_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(the_rows_of_the_panel_of_the_item(
                work_area.height,
                4,
                self.the_smallest_work_of_the_view(),
            )),
        ])
        .areas(work_area);

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
        App::render_footer(footer_area, buf, &text_render_footer);
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
        // **The footer stands on the rows that it needs** (T-302): the
        // number of its rows is the number that the wrap of its text needs.
        let text_render_footer = crate::ui::keys::FOOTER_OF_A_NEW_PODCAST;
        let rows_of_the_footer = self.the_rows_of_the_footer(text_render_footer, area);

        let [header_area, work_area, footer_area] =
            self.the_areas_of_this_view(area, rows_of_the_footer);

        // **The list of a view goes away last** (T-342 and T-344): the panel
        // of the item takes no row that the list needs for its border and one
        // line.
        let [main_area, item_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(the_rows_of_the_panel_of_the_item(
                work_area.height,
                4,
                self.the_smallest_work_of_the_view(),
            )),
        ])
        .areas(work_area);

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
        App::render_footer(footer_area, buf, text_render_footer);
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

            self.render_a_description(item_area, buf, &text);
        }
    }
}

/// The bookmarks of one media. See T-24.
impl App {
    /// AppView::Bookmarks rendering
    fn render_bookmarks(&mut self, area: Rect, buf: &mut Buffer) {
        // **The footer stands on the rows that it needs** (T-302): the
        // number of its rows is the number that the wrap of its text needs.
        let text_render_footer =
            crate::ui::keys::footer_with("go to the place", Some("remove the bookmark"));
        let rows_of_the_footer = self.the_rows_of_the_footer(&text_render_footer, area);

        let [header_area, main_area, footer_area] =
            self.the_areas_of_this_view(area, rows_of_the_footer);

        // A write and a remove forget the answer. The view then asks the
        // server again, and the new list comes at the frame after it.
        self.take_the_bookmarks_again();

        let state = crate::logic::bookmarks::state();

        // **The title names the media of the view**, and not the media that
        // plays: the queue starts the media of its front with no key of the
        // user, and a title of "The bookmarks" alone leaves the user with no
        // way to tell whose places they read. See T-163.
        let name = self.bookmarks_of_name.clone();
        // **A bookmark of Audiobookshelf names an item and no episode**
        // (T-223), therefore the list of a podcast holds the places of every
        // episode of it and the title names the podcast.
        let of_a_podcast = self.bookmarks_of_a_podcast;

        let (title, lines) = match &state {
            crate::logic::bookmarks::State::Ready(all) if all.is_empty() => (
                crate::logic::bookmarks::the_title_of_no_bookmark(&name, of_a_podcast),
                Vec::new(),
            ),
            crate::logic::bookmarks::State::Ready(all) => (
                crate::logic::bookmarks::the_title(
                    &name,
                    &crate::ui::keys::items(all.len()),
                    of_a_podcast,
                ),
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
        App::render_footer(footer_area, buf, &text_render_footer);
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
        // **The footer stands on the rows that it needs** (T-302): the
        // number of its rows is the number that the wrap of its text needs.
        let text_render_footer = crate::ui::keys::footer_with("play it now", Some("take it out"));
        let rows_of_the_footer = self.the_rows_of_the_footer(&text_render_footer, area);

        let [header_area, main_area, footer_area] =
            self.the_areas_of_this_view(area, rows_of_the_footer);

        let queue = crate::logic::queue::snapshot();
        let lines = self.queue_lines(queue.entries());

        let title = if lines.is_empty() {
            "The queue is empty. Press n on a media to put it in the queue.".to_string()
        } else {
            format!("The queue [{}]", crate::ui::keys::items(lines.len()))
        };

        self.render_header(header_area, buf);
        App::render_footer(footer_area, buf, &text_render_footer);
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
        // **The footer stands on the rows that it needs** (T-302): the
        // number of its rows is the number that the wrap of its text needs.
        let text_render_footer = crate::ui::keys::footer_with("go to the chapter", None);
        let rows_of_the_footer = self.the_rows_of_the_footer(&text_render_footer, area);

        let [header_area, main_area, footer_area] =
            self.the_areas_of_this_view(area, rows_of_the_footer);

        let state = self.player.state();

        // **The two bars stand over the table** (T-330.5, and the note of
        // `docs/mockups/mockup-7.md`): the bar of the whole book, with a mark at
        // each boundary of a chapter, and the bar of the chapter of the cursor
        // under it. They are a pure function, therefore they take a test with no
        // screen at all.
        let the_bars = crate::logic::chapters::the_bars_of_the_view(
            area.width.saturating_sub(2),
            &state.chapters,
            state.position,
            state.duration,
            self.list_state_chapters.selected(),
            state.status == PlaybackStatus::Stopped,
        );

        let rows_of_the_bars =
            the_rows_of_the_bars_of_the_chapters(main_area.height, the_bars.is_some());

        let [bars_area, main_area] =
            Layout::vertical([Constraint::Length(rows_of_the_bars), Constraint::Fill(1)])
                .areas(main_area);

        // **The table of the times takes the width of a line of the list**
        // (T-330.5): the block of this view holds one border at the top,
        // therefore the width of the panel is the width of the area, and the
        // bar of the scroll and the sign of the cursor take their columns of it
        // before the columns of the table divide what stays.
        let the_rows_of_the_lines = main_area.height.saturating_sub(2);
        let of_a_line = crate::logic::the_scroll_of_a_list::the_list_of_the_render(
            state.chapters.len(),
            main_area.width,
            the_rows_of_the_lines,
        )
        .width_of_the_lines
        .saturating_sub(crate::ui::the_list_of_a_view::THE_SIGN_OF_THE_CURSOR);

        let lines = crate::logic::chapters::lines(&state.chapters, state.position, of_a_line);
        let the_header_of_the_columns = crate::logic::chapters::the_header_of_the_table(
            crate::logic::chapters::the_columns_of_the_table(of_a_line, &state.chapters),
        );

        // The header holds the three sentences of this view, and it is a pure
        // function of `crate::logic::chapters`: **the two headers of a media
        // name the episode of a podcast** (T-227), and a pure function takes a
        // test.
        let title = crate::logic::chapters::the_header_of_the_view(
            &state.title,
            state.episode_title.as_deref(),
            lines.len(),
            state.status == PlaybackStatus::Stopped,
        );

        self.render_header(header_area, buf);

        if let Some(the_bars) = the_bars {
            // **The render gives the cells of the bar of the book back**
            // (T-333), and a click of one of those cells moves the media to the
            // place that the cell holds.
            self.the_areas_of_the_mouse.the_bar_of_the_book =
                crate::ui::the_bars_of_the_chapters::render(
                    bars_area,
                    buf,
                    &the_bars,
                    crate::logic::chapters::the_columns_of_the_name(state.chapters.len()),
                );
        }

        App::render_footer(footer_area, buf, &text_render_footer);

        // **The line of the user goes to the render itself and not a copy of
        // it** (T-330.5): ratatui writes the offset of the list while it draws
        // it, and a copy takes that offset to nowhere. The map of the mouse
        // then read the offset 0 at every frame, and a click of a row of a list
        // that scrolled gave the chapter of that row of the **first** screen of
        // the list — the key `G` of the book of 70 chapters gave the rows 35 to
        // 70, and a click of the second row of them gave the chapter 2.
        //
        // The state comes out of `self` for the render, because the render
        // takes the colours of `self` beside it, and it goes back after the map
        // of the mouse reads the offset that the render wrote.
        let mut the_line_of_the_user = std::mem::take(&mut self.list_state_chapters);

        let the_lines_of_the_list = crate::ui::the_list_of_a_view::render_the_list_with_a_header(
            main_area,
            buf,
            &self.config.colors,
            &title,
            &lines,
            the_header_of_the_columns.as_deref(),
            &mut the_line_of_the_user,
        );

        self.the_areas_of_the_list_of_the_mouse(
            main_area,
            the_lines_of_the_list,
            lines.len(),
            &the_line_of_the_user,
        );

        self.list_state_chapters = the_line_of_the_user;
    }
}

/// The sequence and the filter of the library. See T-24.
impl App {
    /// AppView::SortFilter rendering
    fn render_sort_filter(&mut self, area: Rect, buf: &mut Buffer) {
        // **The footer stands on the rows that it needs** (T-302): the
        // number of its rows is the number that the wrap of its text needs.
        let text_render_footer = crate::ui::keys::FOOTER_OF_A_LIST;
        let rows_of_the_footer = self.the_rows_of_the_footer(text_render_footer, area);

        let [header_area, main_area, footer_area] =
            self.the_areas_of_this_view(area, rows_of_the_footer);

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
        App::render_footer(footer_area, buf, text_render_footer);
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
        // **The footer stands on the rows that it needs** (T-302): the
        // number of its rows is the number that the wrap of its text needs.
        let text_render_footer = crate::ui::keys::FOOTER_OF_THE_KEYS;
        let rows_of_the_footer = self.the_rows_of_the_footer(text_render_footer, area);

        let [header_area, main_area, footer_area] =
            self.the_areas_of_this_view(area, rows_of_the_footer);

        let lines = crate::ui::keys::lines();

        self.render_header(header_area, buf);
        App::render_footer(footer_area, buf, text_render_footer);
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
        // **The footer stands on the rows that it needs** (T-302): the
        // number of its rows is the number that the wrap of its text needs.
        let of_the_table = crate::ui::keys::the_footer_of_the_home_view(self.is_podcast, false);
        let of_the_bands = crate::ui::keys::the_footer_of_the_home_view(self.is_podcast, true);
        // **The footer names the keys of the panel that holds the focus**
        // (T-320), and it names no panel at all in a terminal that holds no
        // frame of the panels: a footer must not promise a key that the view
        // does not hold (T-143).
        let the_frame_stands = self.the_frame_of_the_panels_stands();
        let the_stack_stands = self.the_stack_of_the_panels_stands();
        let the_focus = self.the_panel_of_the_focus;
        let of_the_table = crate::ui::keys::the_footer_of_a_panel(
            of_the_table,
            the_frame_stands,
            the_stack_stands,
            the_focus,
        );
        let of_the_bands = crate::ui::keys::the_footer_of_a_panel(
            of_the_bands,
            the_frame_stands,
            the_stack_stands,
            the_focus,
        );

        // **The footer takes the rows of the longer of the two texts, and the
        // panel 4 then holds the same rows in the two shapes** (T-336). The
        // shape of that panel comes of its own height, therefore a footer of a
        // height of its own would decide the shape that names it, and a screen
        // of one row more or less would then say the keys of one shape over the
        // other one, which is the fault of T-143. The cost is one row of the
        // panel, at a width where the two texts wrap in a different way.
        let rows_of_the_footer = self
            .the_rows_of_the_footer(&of_the_table, area)
            .max(self.the_rows_of_the_footer(&of_the_bands, area));
        let [header_area, main_area, footer_area] =
            self.the_areas_of_this_view(area, rows_of_the_footer);

        // **The stack of the panels of the frame stands at the left** (T-320),
        // and it takes its 34 columns of a screen of three columns alone.
        let main_area = self.the_stack_of_the_panels(main_area, buf);

        // The panel of the covers stands at the right of the list and of the
        // description. It is always visible. See T-23.
        //
        // **The width of the work of the view is the width that this function
        // reads, and not the width of the screen** (T-320): the stack takes 34
        // columns away, and a panel of a cover of the width of the whole screen
        // would then stand over the list.
        // **The width of the panel of the cover comes of the picture in it**
        // (T-50), therefore the layout needs to know whether a picture comes
        // at all before it cuts the area (T-348).
        let a_picture_comes = self.a_picture_comes_in_the_panel_of_the_cover();

        // Every line starts with a mark: the media that plays, a media that
        // the user finished, or the part that the user heard. See T-44.
        //
        // **The two panels of the covers read this number** (T-354), therefore
        // the lines of the view stand before the layout: a view with no line
        // holds no media of a picture and no media of a cell.
        let lines = self.home_lines();
        let (main_area, cover_panel) = if cover::the_panels_of_the_covers_stand(
            lines.len(),
            self.a_media_of_the_panel_of_the_cover_comes(),
        ) {
            cover::split_for_covers(
                main_area,
                main_area.width,
                cover::picker().font_size(),
                a_picture_comes,
            )
        } else {
            (main_area, None)
        };
        let the_words_of_the_panel = self.render_covers(cover_panel, buf);

        let [list_area, item_area1, item_area2] =
            self.the_areas_of_a_list_and_the_panel(main_area, the_words_of_the_panel);

        let count = self
            .home_rows
            .iter()
            .filter(|row| row.is_a_line_of_the_user())
            .count();
        let render_list_title = format!("Home [{}]", crate::ui::keys::items(count));

        self.render_header(header_area, buf);

        // **A view says why it holds no line.** The Home view of a library with
        // no media drew an empty list and no word at all. See T-103 and T-91.
        if lines.is_empty() {
            self.the_bands_of_the_last_frame = Default::default();
            App::render_footer(footer_area, buf, &of_the_table);
            App::render_the_reason(
                main_area,
                buf,
                &crate::ui::keys::the_text_of_the_home_view_with_no_line(
                    self.is_offline,
                    crate::logic::the_requests_of_the_start::the_fault_of(
                        &self.id_selected_lib,
                        crate::logic::the_requests_of_the_start::TheRequest::Shelves,
                    )
                    .as_deref(),
                ),
            );
            return;
        }

        // **The bands of the covers stand in the place of the table** (T-331 and
        // T-336), and a panel that has no room for one whole band draws the
        // table of today.
        //
        // **The footer says the keys of the shape that this frame drew**
        // (T-143), therefore it comes after the panel 4 and not before it.
        let the_bands_stand =
            self.render_the_bands_of_the_panel_4(list_area, buf, &render_list_title);

        App::render_footer(
            footer_area,
            buf,
            crate::ui::keys::the_footer_of_a_panel(
                crate::ui::keys::the_footer_of_the_home_view(self.is_podcast, the_bands_stand),
                the_frame_stands,
                the_stack_stands,
                the_focus,
            )
            .as_str(),
        );

        if !the_bands_stand {
            self.render_the_list_of_the_panel_4(
                list_area,
                buf,
                &render_list_title,
                &lines,
                Some(&self.home_table_rows()),
                &mut self.list_state_cnt_list.clone(),
            );
        }

        self.render_info_home(item_area1, buf, the_words_of_the_panel.is_some());
        self.render_desc_home(item_area2, buf);
    }

    /// Draws the bands of the covers of the Home view in the panel 4. See T-336.
    ///
    /// It gives `true` when the bands stand, and `false` when the panel has no
    /// room for one whole band: the Home view then draws the table of today,
    /// which is the decision 5 of the maintainer.
    ///
    /// **The bands stand where the table stands** (T-321): the table of the
    /// panel 4 needs the frame of the panels, therefore a screen under 120
    /// columns keeps the list of one column that it had.
    fn render_the_bands_of_the_panel_4(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        title: &str,
    ) -> bool {
        use crate::ui::the_panel_of_the_bands::{plan_the_bands, the_row_of_a_title};

        if !self.the_frame_of_the_panels_stands() {
            self.the_bands_of_the_last_frame = Default::default();
            return false;
        }

        let it_holds_the_focus = self.the_panel_of_the_focus == crate::ui::frame::ThePanel::TheList;
        let block = crate::ui::frame::a_panel(
            crate::ui::frame::ThePanel::TheList.the_number(),
            title,
            it_holds_the_focus,
        );
        let inside = block.inner(area);

        let of_a_cell = crate::ui::the_panel_of_the_gallery::THE_WIDTHS_OF_A_CELL
            [self.the_size_of_a_cell_of_the_gallery];
        let font = cover::picker().font_size();
        let bands = crate::logic::the_bands_of_the_home::the_bands(&self.home_rows);
        let the_line = self.list_state_cnt_list.selected().unwrap_or(0);
        let plan = plan_the_bands(
            inside,
            of_a_cell,
            font,
            &bands,
            the_line,
            &self.the_offsets_of_the_bands,
        );

        // **The offsets of the state take the offsets that the frame drew**
        // (T-337): the plan moves the band of the cursor and it holds a band of
        // an old answer of the server inside its cells, therefore a wheel of the
        // mouse after that frame must start at the band that the user sees.
        for band in &plan.bands {
            self.the_offset_of_a_band_goes_to(band.the_band, band.the_first_cell);
        }

        if !plan.stands() {
            self.the_bands_of_the_last_frame = plan;
            return false;
        }

        block.render(area, buf);

        // **The areas of the mouse are the areas of the last frame**: a view
        // that draws no line of a list must take the lines of the frame before
        // it away, or a click of a cell reads a row that no frame drew (T-321).
        self.the_areas_of_the_list_of_the_mouse(
            area,
            Rect::default(),
            0,
            &self.list_state_cnt_list.clone(),
        );

        let api = std::sync::Arc::clone(&self.api);
        let the_titles = self.home_table_rows();

        for band in &plan.bands {
            Paragraph::new(the_row_of_a_title(band, band.the_title.width))
                .style(crate::ui::theme::a_quiet_text())
                .render(band.the_title, buf);

            for cell in &band.cells {
                // **The border of the cell of the cursor is heavy and bright,
                // and the border of every other cell is thin and dim** (T-327):
                // a colour alone is not the mark of the focus.
                let border = if cell.the_line == the_line {
                    Block::new()
                        .borders(Borders::ALL)
                        .border_type(ratatui::widgets::BorderType::Thick)
                        .border_style(
                            Style::default()
                                .fg(crate::ui::theme::THE_ACCENT)
                                .add_modifier(Modifier::BOLD),
                        )
                } else {
                    Block::new()
                        .borders(Borders::ALL)
                        .border_style(crate::ui::theme::a_quiet_text())
                };
                border.render(cell.the_box, buf);

                // **A cell that no picture reaches holds the title of its
                // media** (T-339): the terminal of no protocol of pictures, the
                // `TOUTUI_NO_COVERS` of the user, and a media that the server
                // holds with no cover each gave a border and nothing at all, and
                // the user then read no name of a media in the whole panel.
                let id = self.the_identity_of_a_line_of_the_home_view(cell.the_line);
                let id = id.filter(|id| !cover::no_picture_comes(id));

                let Some(id) = id else {
                    let title = the_titles
                        .get(cell.the_line)
                        .map(|row| row.title.as_str())
                        .unwrap_or_default();

                    App::render_the_title_of_a_cell(title, cell.the_picture, buf);
                    continue;
                };

                let form = self.covers.form_of(&id).unwrap_or(1.0);
                let of_the_picture = cover::box_of_the_picture(cell.the_picture, font, form);

                if of_the_picture.width > 0 && of_the_picture.height > 0 {
                    if let Some(picture) = self.covers.picture(&api, &id) {
                        StatefulImage::default().render(of_the_picture, buf, picture);
                    }
                }
            }
        }

        self.the_bands_of_the_last_frame = plan;
        true
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

    /// The rows of the band of the player that the screen holds now. See T-104
    /// and T-322.
    ///
    /// **The band stands on the rows that it needs** (T-322): the key `B` takes
    /// the row of the buttons away, and the work of the view then takes that
    /// row.
    fn the_rows_of_the_band(&self) -> u16 {
        if !self.a_media_plays() {
            return 0;
        }

        crate::ui::the_band_of_the_player::the_rows_of_the_band(self.the_key_bindings_stand)
    }

    /// The rows of the panel of the player that the view holds now. See T-104.
    ///
    /// The split of the work of a view is a rule of the screen (T-99), therefore
    /// `the_areas_of_a_list` takes these rows away before it compares.
    fn the_rows_that_the_player_left(&self) -> u16 {
        if self.a_media_plays() {
            return 0;
        }

        crate::ui::the_band_of_the_player::the_rows_of_the_band(self.the_key_bindings_stand)
    }

    /// Draws the band of the player, and it writes the areas of the click of
    /// the user. See T-322.
    ///
    /// **The band belongs to the render of the frame and not to the loop of the
    /// program**: `src/main.rs` drew it before the view, at 9 rows above the end
    /// of the screen, therefore the band and the layout of the view held two
    /// numbers of one row. The view writes `self.rows_of_the_footer` before this
    /// function runs, therefore the band reads the footer of this frame.
    fn render_the_band_of_the_player(&mut self, area: Rect, buf: &mut Buffer) {
        // **A screen of few rows gives the band no row at all** (T-343): the
        // work of the view goes away last, therefore this area holds no row
        // when the view needs every one of them.
        let band = the_area_of_the_band(
            area,
            self.the_rows_of_the_band(),
            self.rows_of_the_footer,
            self.the_smallest_work_of_the_view(),
        );

        if band.height == 0 {
            // **A frame that draws no band takes no click of it** (T-316).
            self.the_areas_of_the_mouse.the_band_of_the_player = Rect::default();
            self.the_areas_of_the_mouse.the_bar_of_the_seek = Rect::default();
            self.the_areas_of_the_mouse.the_length_of_the_media = 0;
            return;
        }

        let state = self.player.state();
        let position = state.position.max(0.0) as u32;
        let length = state.duration.max(0.0) as u32;

        let words = crate::ui::player_tui::TheWordsOfTheBand {
            // **The render of the band gives every end of a line one space**
            // (T-312), therefore these three texts of the server go to it as
            // the server gave them.
            title: crate::logic::media_name::the_name_of_the_media(
                &state.title,
                state.episode_title.as_deref(),
            ),
            author: state.author.clone(),
            chapter: crate::ui::the_band_of_the_player::the_words_of_the_chapter(
                &state.chapters,
                state.position,
                state.chapter_title.as_deref().unwrap_or("No chapter"),
            ),
            it_plays: matches!(
                state.status,
                PlaybackStatus::Playing | PlaybackStatus::Stalled
            ),
            position,
            length,
            the_chapter: crate::ui::the_band_of_the_player::the_place_in_the_chapter(
                &state.chapters,
                state.position,
            ),
            speed: format!(
                "{:.2}",
                if state.speed > 0.0 {
                    state.speed
                } else {
                    self.the_speed_of_the_account
                }
            ),
            volume: crate::player::integrated::player_info::the_volume_of_the_row(state.volume),
            notice: crate::logic::playback::the_place_of_the_disk::the_notice_of_the_player(
                state.notice.clone(),
            ),
            sleep: self.text_of_the_timer_for_sleep(),
            the_buttons_stand: self.the_key_bindings_stand,
        };

        let bar = crate::ui::player_tui::render_the_band(
            band,
            buf,
            &words,
            &self.config.colors.player_background_color.clone(),
        );

        self.the_areas_of_the_mouse.the_band_of_the_player = band;
        self.the_areas_of_the_mouse.the_bar_of_the_seek = bar;
        self.the_areas_of_the_mouse.the_length_of_the_media = length;
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
        // **The footer stands on the rows that it needs** (T-302): the
        // number of its rows is the number that the wrap of its text needs.
        let of_the_view = if self.is_podcast {
            crate::ui::keys::FOOTER_OF_A_LIBRARY_OF_PODCASTS
        } else {
            crate::ui::keys::FOOTER_OF_A_LIBRARY_OF_BOOKS
        };
        // **The footer names the keys of the panel that holds the focus**
        // (T-320), and it names no panel at all in a terminal that holds no
        // frame of the panels: a footer must not promise a key that the view
        // does not hold (T-143).
        let text_render_footer = crate::ui::keys::the_footer_of_a_panel(
            of_the_view,
            self.the_frame_of_the_panels_stands(),
            self.the_stack_of_the_panels_stands(),
            self.the_panel_of_the_focus,
        );
        let text_render_footer = text_render_footer.as_str();
        let rows_of_the_footer = self.the_rows_of_the_footer(text_render_footer, area);
        let [header_area, main_area, footer_area] =
            self.the_areas_of_this_view(area, rows_of_the_footer);

        // **The stack of the panels of the frame stands at the left** (T-320),
        // and it takes its 34 columns of a screen of three columns alone.
        let main_area = self.the_stack_of_the_panels(main_area, buf);

        // The panel of the covers stands at the right of the list and of the
        // description. It is always visible. See T-23.
        //
        // **The width of the work of the view is the width that this function
        // reads, and not the width of the screen** (T-320).
        // **The width of the panel of the cover comes of the picture in it**
        // (T-50), therefore the layout needs to know whether a picture comes
        // at all before it cuts the area (T-348).
        let a_picture_comes = self.a_picture_comes_in_the_panel_of_the_cover();

        // Every book of a series gives one line. See T-22.
        //
        // **The two panels of the covers read this number** (T-354), therefore
        // the lines of the view stand before the layout: a view with no line
        // holds no media of a picture and no media of a cell.
        let lines = self.library_lines();
        let (main_area, cover_panel) = if cover::the_panels_of_the_covers_stand(
            lines.len(),
            self.a_media_of_the_panel_of_the_cover_comes(),
        ) {
            cover::split_for_covers(
                main_area,
                main_area.width,
                cover::picker().font_size(),
                a_picture_comes,
            )
        } else {
            (main_area, None)
        };
        let the_words_of_the_panel = self.render_covers(cover_panel, buf);

        let [list_area, item_area1, item_area2] =
            self.the_areas_of_a_list_and_the_panel(main_area, the_words_of_the_panel);
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

        self.render_header(header_area, buf);
        App::render_footer(footer_area, buf, text_render_footer);

        // **A filter that hides every media is not a library with no media**,
        // and a server that does not answer is neither. See T-103 and T-91.
        if lines.is_empty() {
            App::render_the_reason(
                main_area,
                buf,
                &crate::ui::keys::the_text_of_the_library_view_with_no_line(
                    self.is_offline,
                    self.the_media_of_the_disk_did_not_come,
                    !self.library_filter.is_empty(),
                    self.is_podcast,
                    crate::logic::the_requests_of_the_start::the_fault_of(
                        &self.id_selected_lib,
                        crate::logic::the_requests_of_the_start::TheRequest::Items,
                    )
                    .as_deref(),
                ),
            );
            return;
        }

        self.render_the_list_of_the_panel_4(
            list_area,
            buf,
            &render_list_title,
            &lines,
            Some(&self.library_table_rows()),
            &mut self.list_state_library.clone(),
        );
        self.render_info_library(item_area1, buf, the_words_of_the_panel.is_some());
        self.render_desc_library(item_area2, buf);
    }

    /// AppView::Series rendering: the list of the series of the library.
    fn render_series(&mut self, area: Rect, buf: &mut Buffer) {
        // **The footer stands on the rows that it needs** (T-302): the
        // number of its rows is the number that the wrap of its text needs.
        let text_render_footer = crate::ui::keys::FOOTER_OF_A_LIST;
        let rows_of_the_footer = self.the_rows_of_the_footer(text_render_footer, area);
        let [header_area, main_area, footer_area] =
            self.the_areas_of_this_view(area, rows_of_the_footer);

        // The panel of the covers stands at the right of the list and of the
        // description. It is always visible. See T-23.
        // **The width of the panel of the cover comes of the picture in it**
        // (T-50), therefore the layout needs to know whether a picture comes
        // at all before it cuts the area (T-348).
        let a_picture_comes = self.a_picture_comes_in_the_panel_of_the_cover();
        let (main_area, cover_panel) = cover::split_for_covers(
            main_area,
            area.width,
            cover::picker().font_size(),
            a_picture_comes,
        );
        let the_words_of_the_panel = self.render_covers(cover_panel, buf);

        let [list_area, item_area1, item_area2] =
            self.the_areas_of_a_list_and_the_panel(main_area, the_words_of_the_panel);

        self.render_header(header_area, buf);
        App::render_footer(footer_area, buf, text_render_footer);

        if self.series.is_empty() {
            // A server that does not answer gives no series, and that is not a
            // library with no series. See T-91. **A request that came back with
            // a fault is a third condition**, and `is_offline` does not hold it
            // (T-170).
            let text = crate::logic::the_requests_of_the_start::the_reason_of_no_series(
                self.is_offline,
                crate::logic::the_requests_of_the_start::the_fault_of(
                    &self.id_selected_lib,
                    crate::logic::the_requests_of_the_start::TheRequest::Series,
                )
                .as_deref(),
            );

            App::render_the_reason(main_area, buf, &text);
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

            self.render_a_description(item_area2, buf, &series.description_for_the_screen());
        }
    }

    /// AppView::SeriesBook rendering: the books of one series.
    fn render_series_book(&mut self, area: Rect, buf: &mut Buffer) {
        // **The footer stands on the rows that it needs** (T-302): the
        // number of its rows is the number that the wrap of its text needs.
        let text_render_footer = crate::ui::keys::FOOTER_OF_A_LIST_OF_MEDIA;
        let rows_of_the_footer = self.the_rows_of_the_footer(text_render_footer, area);
        let [header_area, main_area, footer_area] =
            self.the_areas_of_this_view(area, rows_of_the_footer);

        // The panel of the covers stands at the right of the list and of the
        // description. It is always visible. See T-23.
        // **The width of the panel of the cover comes of the picture in it**
        // (T-50), therefore the layout needs to know whether a picture comes
        // at all before it cuts the area (T-348).
        let a_picture_comes = self.a_picture_comes_in_the_panel_of_the_cover();
        let (main_area, cover_panel) = cover::split_for_covers(
            main_area,
            area.width,
            cover::picker().font_size(),
            a_picture_comes,
        );
        let the_words_of_the_panel = self.render_covers(cover_panel, buf);

        let [list_area, item_area1, item_area2] =
            self.the_areas_of_a_list_and_the_panel(main_area, the_words_of_the_panel);

        self.render_header(header_area, buf);
        App::render_footer(footer_area, buf, text_render_footer);

        let Some(series) = self.selected_series() else {
            return;
        };

        let name = series.name.clone();

        // **The line of this view said no place of the user at all** (T-243).
        // The box of the places of the account reaches it with no request.
        let lines: Vec<String> = self.series_book_lines();
        let render_list_title = format!("{} [{}]", name, crate::ui::keys::items(lines.len()));

        self.render_list(
            list_area,
            buf,
            &render_list_title,
            &lines,
            &mut self.list_state_series_book.clone(),
        );

        if let Some(book) = self.selected_series_book() {
            let of_the_disk = the_copy_of_the_disk(&book.id);

            // **The panel of a book of this view said the author and the
            // length alone** (T-243), while the panel of that same book of the
            // Home view of that same run said the place of the user.
            let place = self.the_place_of_the_panel_of_a_series_book(book);

            Paragraph::new(crate::ui::keys::the_panel_of_a_media(
                &format!(
                    "Author: {} - Duration: {}{}",
                    book.author,
                    convert_seconds(vec![book.duration])
                        .first()
                        .cloned()
                        .unwrap_or_default(),
                    of_the_disk,
                ),
                &format!(
                    "Progress: {}%, {} {}",
                    place.percent,               // percentage progression
                    place.the_time_that_is_left, // time left
                    place.the_end,               // is finished
                ),
            ))
            .wrap(Wrap { trim: true })
            .left_aligned()
            .render(item_area1, buf);

            self.render_a_description(item_area2, buf, &book.description_for_the_screen());
        }
    }

    /// AppView::Lists rendering: the collections and the playlists.
    fn render_lists(&mut self, area: Rect, buf: &mut Buffer) {
        // **The footer stands on the rows that it needs** (T-302): the
        // number of its rows is the number that the wrap of its text needs.
        let text_render_footer = crate::ui::keys::FOOTER_OF_THE_LISTS;
        let rows_of_the_footer = self.the_rows_of_the_footer(text_render_footer, area);
        let [header_area, main_area, footer_area] =
            self.the_areas_of_this_view(area, rows_of_the_footer);

        // The panel of the covers stands at the right of the list and of the
        // description. It is always visible. See T-23.
        // **The width of the panel of the cover comes of the picture in it**
        // (T-50), therefore the layout needs to know whether a picture comes
        // at all before it cuts the area (T-348).
        let a_picture_comes = self.a_picture_comes_in_the_panel_of_the_cover();
        let (main_area, cover_panel) = cover::split_for_covers(
            main_area,
            area.width,
            cover::picker().font_size(),
            a_picture_comes,
        );
        let the_words_of_the_panel = self.render_covers(cover_panel, buf);

        let [list_area, item_area1, item_area2] =
            self.the_areas_of_a_list_and_the_panel(main_area, the_words_of_the_panel);

        self.render_header(header_area, buf);
        App::render_footer(footer_area, buf, text_render_footer);

        if self.lists.is_empty() {
            // **The program knows nothing of the lists of a server that does
            // not answer.** The offline sweep of 2026-08-11 read "This library
            // has no collection and no playlist" with the server stopped, and
            // that sentence is not true: no request gave an answer. The view of
            // the authors held this rule already. See T-91.
            //
            // **A request that came back with a fault is a fourth condition**,
            // and `is_offline` does not hold it: the server answers, and the
            // program has no list of it. See T-169.
            let what_the_server_said = crate::logic::the_lists::the_fault_of(&self.id_selected_lib);

            let text = crate::logic::the_lists::the_reason_of_no_list(
                self.is_offline,
                what_the_server_said.as_deref(),
            );

            // **The sentence of the fault names what the server said**,
            // therefore it is longer than the two sentences before it. A
            // paragraph with no wrap cuts the words at the edge of the panel:
            // the measurement of T-169 read "The server reported a fault.
            // Status" with the number outside the screen.
            Paragraph::new(text)
                .centered()
                .wrap(Wrap { trim: true })
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

            self.render_a_description(item_area2, buf, &list.description.clone());
        }
    }

    /// AppView::ListEntries rendering: the media of one collection or of one
    /// playlist.
    fn render_list_entries(&mut self, area: Rect, buf: &mut Buffer) {
        // **The footer stands on the rows that it needs** (T-302): the
        // number of its rows is the number that the wrap of its text needs.
        let text_render_footer = crate::ui::keys::FOOTER_OF_THE_MEDIA_OF_A_LIST;
        let rows_of_the_footer = self.the_rows_of_the_footer(text_render_footer, area);
        let [header_area, main_area, footer_area] =
            self.the_areas_of_this_view(area, rows_of_the_footer);

        // The panel of the covers stands at the right of the list and of the
        // description. It is always visible. See T-23.
        // **The width of the panel of the cover comes of the picture in it**
        // (T-50), therefore the layout needs to know whether a picture comes
        // at all before it cuts the area (T-348).
        let a_picture_comes = self.a_picture_comes_in_the_panel_of_the_cover();
        let (main_area, cover_panel) = cover::split_for_covers(
            main_area,
            area.width,
            cover::picker().font_size(),
            a_picture_comes,
        );
        let the_words_of_the_panel = self.render_covers(cover_panel, buf);

        let [list_area, item_area1, item_area2] =
            self.the_areas_of_a_list_and_the_panel(main_area, the_words_of_the_panel);

        self.render_header(header_area, buf);
        App::render_footer(footer_area, buf, text_render_footer);

        let Some(list) = self.selected_list() else {
            return;
        };

        // **The line of this view said no place of the user at all** (T-243).
        // The key of the place names the episode after the item (T-223),
        // therefore a line of an episode holds the place of its own episode.
        let name = list.name.clone();
        let lines: Vec<String> = self.list_entry_lines();
        let render_list_title = format!("{} [{}]", name, crate::ui::keys::items(lines.len()));

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
            let of_the_disk = the_copy_of_the_disk(&key);

            // **The panel of a media of this view said the author and the
            // length alone** (T-243), while the panel of that same media of the
            // Home view of that same run said the place of the user.
            let place = self.the_place_of_the_panel_of_a_list_entry(entry);

            Paragraph::new(crate::ui::keys::the_panel_of_a_media(
                &format!(
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
                    of_the_disk,
                ),
                &format!(
                    "Progress: {}%, {} {}",
                    place.percent,               // percentage progression
                    place.the_time_that_is_left, // time left
                    place.the_end,               // is finished
                ),
            ))
            .wrap(Wrap { trim: true })
            .left_aligned()
            .render(item_area1, buf);

            self.render_a_description(item_area2, buf, &entry.description.clone());
        }
    }

    /// AppView::Settings rendering
    fn render_settings(&mut self, area: Rect, buf: &mut Buffer) {
        // **The footer stands on the rows that it needs** (T-302): the
        // number of its rows is the number that the wrap of its text needs.
        let text_render_footer = crate::ui::keys::FOOTER_OF_A_LIST;
        let rows_of_the_footer = self.the_rows_of_the_footer(text_render_footer, area);
        let [header_area, main_area, footer_area] =
            self.the_areas_of_this_view(area, rows_of_the_footer);

        let [list_area, item_area1, item_area2] = the_areas_of_a_list(
            main_area,
            self.the_rows_that_the_player_left(),
            self.the_smallest_work_of_the_view(),
        );

        let render_list_title = "Settings";

        self.render_header(header_area, buf);
        App::render_footer(footer_area, buf, text_render_footer);
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
        // **The footer stands on the rows that it needs** (T-302): the
        // number of its rows is the number that the wrap of its text needs.
        let text_render_footer = crate::ui::keys::FOOTER_OF_THE_ACCOUNTS;
        let rows_of_the_footer = self.the_rows_of_the_footer(text_render_footer, area);
        let [header_area, main_area, footer_area] =
            self.the_areas_of_this_view(area, rows_of_the_footer);

        let [list_area, item_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Fill(1)]).areas(main_area);

        // **The title and the footer name every key of this view.** A key that
        // a view holds and that no text names is a key that no user finds.
        // See T-124 and T-79.
        let render_list_title = "Accounts — a: add, c: this account starts, l: log out";
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
        // **The footer stands on the rows that it needs** (T-302): the
        // number of its rows is the number that the wrap of its text needs.
        let text_render_footer =
            crate::ui::keys::footer_with("write this value in config.toml", None);
        let rows_of_the_footer = self.the_rows_of_the_footer(&text_render_footer, area);

        let [header_area, work_area, footer_area] =
            self.the_areas_of_this_view(area, rows_of_the_footer);

        // **The list of a view goes away last** (T-342 and T-344): the panel
        // of the item takes no row that the list needs for its border and one
        // line.
        let [main_area, item_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(the_rows_of_the_panel_of_the_item(
                work_area.height,
                5,
                self.the_smallest_work_of_the_view(),
            )),
        ])
        .areas(work_area);

        let now = self.megabytes_of_the_cache();

        let lines: Vec<String> = crate::logic::reader::cache::THE_VALUES_OF_THE_SETTINGS
            .iter()
            .map(|value| crate::logic::reader::cache::line_of_a_value(*value, now))
            .collect();

        let title = format!("The cache of the ebooks — {} MB now", now);

        self.render_header(header_area, buf);
        App::render_footer(footer_area, buf, &text_render_footer);
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
        // **The footer stands on the rows that it needs** (T-302): the
        // number of its rows is the number that the wrap of its text needs.
        let text_render_footer = crate::ui::keys::FOOTER_OF_THE_LIBRARY_OF_THE_USER;
        let rows_of_the_footer = self.the_rows_of_the_footer(text_render_footer, area);
        let [header_area, main_area, footer_area] =
            self.the_areas_of_this_view(area, rows_of_the_footer);

        let [list_area, item_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Fill(1)]).areas(main_area);

        let items_number = self.libraries_names.len();
        let render_list_title = format!(
            "Settings Library [{}]",
            crate::ui::keys::items(items_number)
        );

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
        // **The footer stands on the rows that it needs** (T-302): the
        // number of its rows is the number that the wrap of its text needs.
        let text_render_footer = crate::ui::keys::FOOTER_OF_THE_SEARCH;
        let rows_of_the_footer = self.the_rows_of_the_footer(text_render_footer, area);
        let [header_area, main_area, footer_area] =
            self.the_areas_of_this_view(area, rows_of_the_footer);

        // The panel of the covers stands at the right of the list and of the
        // description. It is always visible. See T-23.
        // **The width of the panel of the cover comes of the picture in it**
        // (T-50), therefore the layout needs to know whether a picture comes
        // at all before it cuts the area (T-348).
        let a_picture_comes = self.a_picture_comes_in_the_panel_of_the_cover();
        let (main_area, cover_panel) = cover::split_for_covers(
            main_area,
            area.width,
            cover::picker().font_size(),
            a_picture_comes,
        );
        let the_words_of_the_panel = self.render_covers(cover_panel, buf);

        let [list_area, item_area1, item_area2] =
            self.the_areas_of_a_list_and_the_panel(main_area, the_words_of_the_panel);

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
        //
        // **The program reads those pages now** (T-125). A library of 520
        // podcasts said "The server found nothing" for a podcast that the
        // server found, and that is a reason that the program does not have
        // (T-91). The line comes when the page of that media comes, therefore
        // the title says what the program does while the user waits.
        let mut the_podcasts_that_come = 0;

        if self.is_podcast {
            let before = found.len();

            found.retain(|one| one.place.is_some());
            the_podcasts_that_come = before - found.len();

            if the_podcasts_that_come > 0 {
                self.the_search_reads_the_pages_that_are_left();
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
            the_podcasts_that_come,
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
        self.all_the_lengths_of_the_episodes_search =
            the_values_at(&self.all_the_lengths_of_the_episodes, &places);
        // The place of the user of each episode of each podcast of this view.
        // See T-245.
        self.all_pod_ep_places_search = the_values_at(&self.all_pod_ep_places, &places);
        self.ids_library_pod_search = the_values_at(&self.ids_library, &places);

        // **The line of a book of this view held no mark of the percent**
        // (T-242): the panel of it says the place of the user (T-241), and the
        // line of that same book of the Home view of that same frame said
        // `84% A Book Of Many Hours`.
        //
        // **A line of a library of podcasts holds more than one media**
        // (T-221), therefore such a line keeps the title of the server.
        let the_lines_of_the_search: Vec<String> = if self.is_podcast {
            titles_search_book_or_pod.to_vec()
        } else {
            self.search_book_lines(titles_search_book_or_pod)
        };

        self.render_header(header_area, buf);
        App::render_footer(footer_area, buf, text_render_footer);
        self.render_list(
            list_area,
            buf,
            render_list_title,
            &the_lines_of_the_search,
            &mut self.list_state_search_results.clone(),
        );
        if !titles_search_book_or_pod.is_empty() {
            self.render_info_search_book(item_area1, buf, &self.list_state_search_results.clone());
            self.render_desc_search_book(item_area2, buf, &self.list_state_search_results.clone());
        }
    }

    /// AppView::PodcastEpisode
    fn render_pod_ep(&mut self, area: Rect, buf: &mut Buffer) {
        // **The footer stands on the rows that it needs** (T-302): the
        // number of its rows is the number that the wrap of its text needs.
        let text_render_footer = crate::ui::keys::FOOTER_OF_A_LIST_OF_MEDIA;
        let rows_of_the_footer = self.the_rows_of_the_footer(text_render_footer, area);
        let [header_area, main_area, footer_area] =
            self.the_areas_of_this_view(area, rows_of_the_footer);

        // The panel of the covers stands at the right of the list and of the
        // description. It is always visible. See T-23.
        // **The width of the panel of the cover comes of the picture in it**
        // (T-50), therefore the layout needs to know whether a picture comes
        // at all before it cuts the area (T-348).
        let a_picture_comes = self.a_picture_comes_in_the_panel_of_the_cover();
        let (main_area, cover_panel) = cover::split_for_covers(
            main_area,
            area.width,
            cover::picker().font_size(),
            a_picture_comes,
        );
        let the_words_of_the_panel = self.render_covers(cover_panel, buf);

        let [list_area, item_area1, item_area2] =
            self.the_areas_of_a_list_and_the_panel(main_area, the_words_of_the_panel);

        self.render_header(header_area, buf);
        App::render_footer(footer_area, buf, text_render_footer);
        // **A view must not give a reason that the program does not have**
        // (T-91). The view said "This podcast has no episode" while the
        // program had not read one episode of that podcast: a podcast of a
        // page after the first met that sentence, and it holds one episode.
        // See T-126.
        let the_place_of_the_podcast = if self.is_from_search_pod {
            self.list_state_search_results
                .selected()
                .and_then(|line| self.ids_library_pod_search.get(line))
                .and_then(|id| self.ids_library.iter().position(|one| one == id))
        } else {
            self.selected_library_item()
        };

        // A place that these lists do not hold is a podcast that the program
        // did not read: the answer of the server did not come for it.
        let the_episodes_came = the_place_of_the_podcast
            .and_then(|place| self.the_episodes_that_came.get(place))
            .copied()
            .unwrap_or(false);

        // The request of this podcast did not come back, and the view said that
        // the program gets the episodes for ever. See T-168.
        let what_the_server_said =
            the_place_of_the_podcast.and_then(crate::logic::the_episodes::the_fault_of);

        let no_episodes_message = format!(
            "{}\nPress h to go back.",
            crate::logic::the_episodes::the_reason_of_no_episode(
                the_episodes_came && !crate::logic::the_episodes::asks(),
                self.is_offline,
                what_the_server_said.as_deref(),
            )
        );
        let no_episodes_message = no_episodes_message.as_str();

        if self.is_from_search_pod {
            if self.titles_pod_ep_search.is_empty() {
                // **The sentence of this view holds the words of the server,
                // and a terminal of 80 columns cuts it** (T-278). The widget of
                // it stands in a function of its own, with the `wrap`.
                crate::ui::the_message_of_a_view::render_the_message(
                    no_episodes_message,
                    main_area,
                    buf,
                );
            } else {
                let items_number = self.titles_pod_ep_search.len();
                let render_list_title =
                    format!("Episodes [{}]", crate::ui::keys::items(items_number));
                // Only render list/info/desc if episodes exist
                self.render_list(
                    list_area,
                    buf,
                    &render_list_title,
                    &self.pod_ep_lines_search(),
                    &mut self.list_state_pod_ep.clone(),
                );
                self.render_info_pod_ep_search(item_area1, buf, &self.list_state_pod_ep.clone());
                self.render_desc_pod_ep_search(item_area2, buf, &self.list_state_pod_ep.clone());
            }
        } else {
            if self.titles_pod_ep.is_empty() {
                // The same rule of T-278, for the road of the view that no
                // search opened.
                crate::ui::the_message_of_a_view::render_the_message(
                    no_episodes_message,
                    main_area,
                    buf,
                );
            } else {
                let items_number = self.titles_pod_ep.len();
                let render_list_title =
                    format!("Episodes [{}]", crate::ui::keys::items(items_number));
                // Only render list/info/desc if episodes exist
                self.render_list(
                    list_area,
                    buf,
                    &render_list_title,
                    &self.pod_ep_lines(),
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
    ///
    /// **Each part of a row stands whole, or it does not stand at all** (T-340):
    /// the three parts of a row are three texts of one row, and a part that is
    /// too long wrote on the letters of its neighbour at every width under
    /// about 54 columns. `crate::ui::the_row_of_the_header` holds the rule and
    /// the measurement of the fault.
    fn render_header(&self, area: Rect, buf: &mut Buffer) {
        // **The address of the pool, and not the address of the login.** A pool
        // of two addresses moves between them, and the header named the address
        // that the user gave at the login for ever. See T-105 and T-107.
        let active = self.api.pool().active();

        // **A server that answers `500` is not a server that is away.** The
        // header said "does not answer" for a server that answered `curl` in
        // 1.4 milliseconds, and it offered the media of the disk to that user.
        // See T-171.
        let the_server_reports_a_fault = self.api.pool().every_address_answers_with_a_fault();

        let connection = crate::ui::keys::the_lines_of_the_connection(
            &self.username,
            active.as_deref(),
            &self.server_address_pretty,
            self.is_offline,
            the_server_reports_a_fault,
            area.width,
        );

        // The audio engine did not start. The user reads the library, and no
        // media plays. See T-46.
        let connection = match &self.audio_fault {
            Some(_) => format!("{}\n🔇 No sound device: no media can play", connection),
            None => connection,
        };

        // The three rows of the header, each of them a row of its own: the
        // account, the address, and the sound device that did not start. **A
        // paragraph of the three of them over the whole area writes each row
        // over the width of the screen**, and the parts beside them then have
        // no room of their own to stand in (T-340).
        let mut the_rows_of_the_connection = connection.lines();
        let the_account = the_rows_of_the_connection.next().unwrap_or("").to_string();
        let the_address = the_rows_of_the_connection.next().unwrap_or("").to_string();
        let the_sound = the_rows_of_the_connection.next().unwrap_or("").to_string();

        let notice = if self.is_offline {
            // **A read of the disk that failed names no number** (T-203): a count
            // of 0 says that every place of the user reached the server already.
            let waiting = match self.waiting_progress {
                None | Some(0) => String::new(),
                Some(1) => " - 1 position waits".to_string(),
                Some(count) => format!(" - {} wait", crate::ui::keys::counted(count, "position")),
            };

            format!("R: try the server again{}", waiting)
        } else if active.is_none() && the_server_reports_a_fault {
            // The server answers, and its answer holds a fault. The media of
            // the disk are not the road of this user: the key `R` asks the
            // server again, and it gives the lists of the server. See T-171 and
            // T-170.
            crate::ui::keys::THE_SERVER_REPORTS_A_FAULT.to_string()
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

        // **The notice carries a space of its own at the left of it**, and that
        // space is the first column of the gap of T-329: the part at the right
        // therefore holds one column more than the letters of the notice, and
        // every screen that stood before T-340 stands in the same shape.
        let the_notice = if notice.is_empty() {
            String::new()
        } else {
            format!(" {}", notice)
        };

        let the_version = crate::ui::keys::the_name_of_the_program(VERSION, area.width);

        // **The first row of the header loses the name of the program first,
        // and the account after it** (T-340). The name of the library is the
        // last part to go away, because it names what the list under it holds
        // and the key of the next library changes it; the account comes before
        // it, because this program holds more than one account (T-124); and the
        // name of the program says the least of the three, and the settings
        // screen says it at every width already.
        let of_the_first_row = crate::ui::the_row_of_the_header::the_places_of_a_row(
            area.width,
            crate::ui::the_row_of_the_header::the_columns(&the_account),
            crate::ui::the_row_of_the_header::the_columns(&self.lib_name_type),
            crate::ui::the_row_of_the_header::the_columns(&the_version),
            [
                crate::ui::the_row_of_the_header::ThePart::AtTheRight,
                crate::ui::the_row_of_the_header::ThePart::AtTheLeft,
                crate::ui::the_row_of_the_header::ThePart::InTheMiddle,
            ],
        );

        // **The second row loses the notice of the key `R` before the address**
        // (T-340): the notice names a key that the footer of every view names
        // too, and an address that the row cuts says an address that the user
        // does not have — the measurement of that row at 40 columns read
        // `🔗 localhost:133`. The words of the sequence and of the filter are
        // the middle of this row, and they go away first by the rule of T-329
        // that `draw_the_words_of_the_sequence` holds.
        let of_the_second_row = crate::ui::the_row_of_the_header::the_places_of_a_row(
            area.width,
            crate::ui::the_row_of_the_header::the_columns(&the_address),
            0,
            crate::ui::the_row_of_the_header::the_columns(&the_notice),
            [
                crate::ui::the_row_of_the_header::ThePart::InTheMiddle,
                crate::ui::the_row_of_the_header::ThePart::AtTheRight,
                crate::ui::the_row_of_the_header::ThePart::AtTheLeft,
            ],
        );

        App::draw_a_part_of_the_header(area, buf, 0, of_the_first_row.at_the_left, &the_account);
        App::draw_a_part_of_the_header(area, buf, 1, of_the_second_row.at_the_left, &the_address);
        App::draw_a_part_of_the_header(area, buf, 0, of_the_first_row.at_the_right, &the_version);
        App::draw_a_part_of_the_header(area, buf, 1, of_the_second_row.at_the_right, &the_notice);

        if let Some(column) = of_the_first_row.in_the_middle {
            let of_the_library = Rect {
                x: area.x.saturating_add(column),
                y: area.y,
                width: crate::ui::the_row_of_the_header::the_columns(&self.lib_name_type),
                height: 1,
            };

            if area.height >= 1 && of_the_library.right() <= area.right() {
                Paragraph::new(self.lib_name_type.clone())
                    .bold()
                    .left_aligned()
                    .render(of_the_library, buf);
            }
        }

        // The sound device that did not start takes the third row of the
        // header, and no part stands beside it. See T-46.
        if !the_sound.is_empty() && area.height >= 3 {
            Paragraph::new(the_sound)
                .not_bold()
                .wrap(Wrap { trim: true })
                .left_aligned()
                .render(
                    Rect {
                        x: area.x,
                        y: area.y.saturating_add(2),
                        width: area.width,
                        height: area.height.saturating_sub(2),
                    },
                    buf,
                );
        }

        // The columns of the two neighbours of the words of the sequence and of
        // the filter, which is the rule of T-329. **A part that the row took
        // away holds no column at all**, therefore the words then take the room
        // that it left.
        let the_columns_of_the_address = of_the_second_row
            .at_the_left
            .map(|_| usize::from(crate::ui::the_row_of_the_header::the_columns(&the_address)))
            .unwrap_or(0);

        let the_columns_of_the_notice = of_the_second_row
            .at_the_right
            .map(|_| usize::from(crate::ui::the_row_of_the_header::the_columns(&the_notice)))
            .unwrap_or(0);

        // **The header keeps the words of the sequence and of the filter for a
        // screen that draws no stack** (T-318, and the decision 3 of the road
        // of the panels): the panel 2 and the panel 3 say those two values at
        // 120 columns and up, and a terminal under that width held no word of
        // them at all — the user read them in the view of the key `f` alone,
        // and that view hides the list that it describes.
        //
        // **The words come after the two parts beside them** (T-329), because
        // they take the room that those two leave: the address of the server
        // stands at the left of this row and the notice of the key `R` stands
        // at the right of it, and a centred paragraph over the whole area wrote
        // on the letters of the address at every width under 80 columns and at
        // 84 columns with a sequence of its own.
        if !self.the_stack_of_the_panels_stands()
            && matches!(self.view_state, AppView::Home | AppView::Library)
            && !self.is_offline
        {
            let of_the_server = match crate::logic::sort_filter::from_the_server::state() {
                crate::logic::sort_filter::from_the_server::State::Ready(choices) => choices,
                _ => Vec::new(),
            };

            let the_words =
                crate::ui::the_panels_of_the_stack::the_words_of_the_sequence_and_the_filter(
                    self.is_podcast,
                    &self.library_sort,
                    self.library_desc,
                    &self.library_filter,
                    &of_the_server,
                );

            App::draw_the_words_of_the_sequence(
                area,
                buf,
                &the_words,
                the_columns_of_the_address,
                the_columns_of_the_notice,
            );
        }
    }

    /// Draws one part of one row of the header, at the column that
    /// `crate::ui::the_row_of_the_header` gave it. See T-340.
    ///
    /// **A part of `None` does not stand at all**: the row had no room for it
    /// with a gap of two columns from its neighbours, and a text that the row
    /// cuts says nothing to the user (T-91).
    ///
    /// **This function stands beside `render_header`** for the reason of
    /// `draw_the_words_of_the_sequence`: a test of it needs a `Buffer` and no
    /// `App`, no terminal, and no server at all.
    fn draw_a_part_of_the_header(
        area: Rect,
        buf: &mut Buffer,
        the_row: u16,
        the_column: Option<u16>,
        text: &str,
    ) {
        let Some(the_column) = the_column else {
            return;
        };

        if text.is_empty() || area.height <= the_row {
            return;
        }

        let of_the_part = Rect {
            x: area.x.saturating_add(the_column),
            y: area.y.saturating_add(the_row),
            width: crate::ui::the_row_of_the_header::the_columns(text),
            height: 1,
        };

        // The row of the header holds every column of this part already, and a
        // part that leaves the area of the header draws nothing at all.
        if of_the_part.right() > area.right() {
            return;
        }

        Paragraph::new(text.to_string())
            .not_bold()
            .left_aligned()
            .render(of_the_part, buf);
    }

    /// Draws the words of the sequence and of the filter on the second row of
    /// the header, between the address at the left and the notice at the right.
    ///
    /// **This function stands beside `render_header` and not inside it** for the
    /// reason of `draw_the_row_of_the_message`: a test of it needs a `Buffer`
    /// and no `App`, no terminal, and no server at all. See T-329.
    ///
    /// `at_the_left` and `at_the_right` are the columns of the two neighbours of
    /// the words on that row.
    fn draw_the_words_of_the_sequence(
        area: Rect,
        buf: &mut Buffer,
        the_words: &str,
        at_the_left: usize,
        at_the_right: usize,
    ) {
        if area.height < 2 {
            return;
        }

        let Some(column) = crate::ui::the_panels_of_the_stack::the_column_of_the_words(
            area.width,
            u16::try_from(at_the_left).unwrap_or(u16::MAX),
            u16::try_from(at_the_right).unwrap_or(u16::MAX),
            u16::try_from(crate::logic::message::the_columns_of(the_words)).unwrap_or(u16::MAX),
        ) else {
            return;
        };

        let row = Rect {
            x: area.x.saturating_add(column),
            y: area.y.saturating_add(1),
            width: area.width.saturating_sub(column),
            height: 1,
        };

        Paragraph::new(the_words.to_string())
            .not_bold()
            .left_aligned()
            .render(row, buf);
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
    /// Gives the number of rows of the footer of a view, and it keeps that
    /// number for the message of the same frame.
    ///
    /// **A footer stands on the rows that it needs** (T-302). The row of the
    /// message grows upward from the row above the footer (T-299), therefore
    /// `render_the_message` reads the same number: a message that read the two
    /// rows of the old footer would draw over the keys of a narrow terminal.
    fn the_rows_of_the_footer(&mut self, keys: &str, area: Rect) -> u16 {
        let rows = crate::ui::keys::the_rows_of_a_footer(keys, area.width, area.height);
        self.rows_of_the_footer = rows;
        rows
    }

    /// Draws the footer of a view.
    ///
    /// **A footer that needs more rows than the view gives it loses its end to
    /// three points** (T-302, and the rule of T-299 for a message). The caller
    /// gives the rows of `the_rows_of_the_footer`, therefore that road comes
    /// for a view of few rows alone.
    fn render_footer(area: Rect, buf: &mut Buffer, text_render_footer: &str) {
        Paragraph::new(crate::logic::message::in_the_rows(
            text_render_footer,
            area.width,
            area.height,
        ))
        .wrap(Wrap { trim: true })
        .centered()
        .render(area, buf);
    }

    /// Draws the stack of the panels of the frame at the left of the work of a
    /// view, and gives the area of the work that stays. See T-320.
    ///
    /// **The stack comes with the three columns alone**
    /// (`crate::ui::frame::the_shape_of`), therefore this function gives the
    /// whole area back for a narrow terminal and it draws nothing at all: the
    /// screen of a terminal under 120 columns is the screen of today.
    ///
    /// **The stack holds three panels** (T-318): the panel 1 of the views, the
    /// panel 2 of the sequence, and the panel 3 of the filter. A stack that is
    /// too short loses the panel 3 first and the panel 2 after it, and the
    /// panel 1 keeps the rows that it needs
    /// (`the_panels_of_the_stack::the_three_panels`).
    fn the_stack_of_the_panels(&mut self, main_area: Rect, buf: &mut Buffer) -> Rect {
        let (stack, work) = crate::ui::frame::the_stack_and_the_work(
            main_area,
            crate::ui::frame::the_shape_of(main_area.width),
            self.the_stack_is_hidden,
        );

        let Some(stack) = stack else {
            // **A frame that draws no stack takes no click of a stack**
            // (T-316): the areas of the last frame are the screen that the user
            // clicked, therefore a panel that the frame did not draw must hold
            // no cell of that screen at all.
            self.the_areas_of_the_mouse.the_panel_of_the_views = Rect::default();
            self.the_areas_of_the_mouse.the_lines_of_the_views = Rect::default();
            self.the_areas_of_the_mouse.the_panel_of_the_sequence = Rect::default();
            self.the_areas_of_the_mouse.the_lines_of_the_sequence = Rect::default();
            self.the_areas_of_the_mouse.the_panel_of_the_filter = Rect::default();
            self.the_areas_of_the_mouse.the_lines_of_the_filter = Rect::default();

            return work;
        };

        let of_the_sequence = self.the_rows_of_the_panel_2();
        let of_the_filter = crate::ui::the_panels_of_the_stack::the_rows_of_the_filter();

        let (stack, the_sequence, the_filter) =
            crate::ui::the_panels_of_the_stack::the_three_panels(
                stack,
                crate::ui::the_panels_of_the_stack::the_height_of_a_panel(of_the_sequence.len()),
                crate::ui::the_panels_of_the_stack::the_height_of_a_panel(of_the_filter.len()),
            );

        self.render_a_panel_of_the_stack(
            the_sequence,
            crate::ui::frame::ThePanel::TheSequence,
            "Sequence",
            &of_the_sequence,
            buf,
        );
        self.render_a_panel_of_the_stack(
            the_filter,
            crate::ui::frame::ThePanel::TheFilter,
            "Filter",
            &of_the_filter,
            buf,
        );

        let it_holds_the_focus =
            self.the_panel_of_the_focus == crate::ui::frame::ThePanel::TheViews;

        let block = crate::ui::frame::a_panel(
            crate::ui::frame::ThePanel::TheViews.the_number(),
            "Views",
            it_holds_the_focus,
        );
        let inner = block.inner(stack);
        block.render(stack, buf);

        let lines: Vec<ListItem> = crate::ui::frame::the_lines_of_the_views(inner.width)
            .into_iter()
            .map(ListItem::new)
            .collect();

        // **The row of the cursor of a panel that does not hold the focus is
        // quiet** (the section (c) of `docs/mockups/mockup-1.md`): one accent
        // alone stands on the screen, and it belongs to the panel of the focus.
        let of_the_cursor = if it_holds_the_focus {
            Style::new()
                .bg(crate::ui::theme::THE_ACCENT)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::new().add_modifier(Modifier::BOLD)
        };

        // **The render gets a copy of the state** (the code before T-316 did
        // the same): the panel 1 keeps the line of the user, and ratatui writes
        // the offset of the panel into this copy while it draws the list.
        let mut of_the_render = self.the_line_of_the_views;

        StatefulWidget::render(
            List::new(lines)
                .highlight_style(of_the_cursor)
                .highlight_symbol("➤ ")
                .highlight_spacing(HighlightSpacing::Always),
            inner,
            buf,
            &mut of_the_render,
        );

        // The areas that a report of the mouse reads. See T-316.
        self.the_areas_of_the_mouse.the_panel_of_the_views = stack;
        self.the_areas_of_the_mouse.the_lines_of_the_views = inner;
        self.the_areas_of_the_mouse.the_offset_of_the_views = of_the_render.offset();
        self.the_areas_of_the_mouse.the_views = crate::ui::frame::THE_VIEWS.len();

        work
    }

    /// Draws the panel 2 of the sequence or the panel 3 of the filter, and it
    /// writes the areas of that panel for the mouse. See T-318.
    ///
    /// **A panel with no area holds no cell of the screen at all**: the stack
    /// of a terminal that is not tall loses the panel 3 first and the panel 2
    /// after it, and a click of the mouse and a digit of the focus then name
    /// nothing (T-79).
    fn render_a_panel_of_the_stack(
        &mut self,
        area: Option<Rect>,
        the_panel: crate::ui::frame::ThePanel,
        name: &str,
        rows: &[crate::logic::sort_filter::Row],
        buf: &mut Buffer,
    ) {
        let Some(area) = area else {
            self.the_areas_of_a_panel_of_the_stack(the_panel, Rect::default(), Rect::default(), 0);
            return;
        };

        let it_holds_the_focus = self.the_panel_of_the_focus == the_panel;

        let block = crate::ui::frame::a_panel(the_panel.the_number(), name, it_holds_the_focus);
        let inner = block.inner(area);
        block.render(area, buf);

        let lines: Vec<ListItem> = crate::ui::the_panels_of_the_stack::the_lines_of_a_panel(
            rows,
            inner.width,
            &self.library_sort,
            self.library_desc,
            &self.library_filter,
        )
        .into_iter()
        .map(ListItem::new)
        .collect();

        // **The row of the cursor of a panel that does not hold the focus is
        // quiet** (the section (c) of `docs/mockups/mockup-1.md`): one accent
        // alone stands on the screen, and it belongs to the panel of the focus.
        let of_the_cursor = if it_holds_the_focus {
            Style::new()
                .bg(crate::ui::theme::THE_ACCENT)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::new().add_modifier(Modifier::BOLD)
        };

        let mut of_the_render = self.the_line_of_a_panel_of_the_stack(the_panel);

        StatefulWidget::render(
            List::new(lines)
                .highlight_style(of_the_cursor)
                .highlight_symbol("➤ ")
                .highlight_spacing(HighlightSpacing::Always),
            inner,
            buf,
            &mut of_the_render,
        );

        self.the_areas_of_a_panel_of_the_stack(the_panel, area, inner, rows.len());
    }

    /// Writes the areas of the last frame of the panel 2 or of the panel 3.
    /// See T-318.
    fn the_areas_of_a_panel_of_the_stack(
        &mut self,
        the_panel: crate::ui::frame::ThePanel,
        area: Rect,
        inner: Rect,
        lines: usize,
    ) {
        if the_panel == crate::ui::frame::ThePanel::TheSequence {
            self.the_areas_of_the_mouse.the_panel_of_the_sequence = area;
            self.the_areas_of_the_mouse.the_lines_of_the_sequence = inner;
            self.the_areas_of_the_mouse.the_sequences = lines;
        } else {
            self.the_areas_of_the_mouse.the_panel_of_the_filter = area;
            self.the_areas_of_the_mouse.the_lines_of_the_filter = inner;
            self.the_areas_of_the_mouse.the_filters = lines;
        }
    }

    fn render_list(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        render_list_title: &str,
        render_list_items: &[String],
        list_state: &mut ListState,
    ) {
        // **The render of a list stands in a module of its own** (T-256): a
        // private method of `App` reaches no test, therefore the bar of the
        // scroll of T-255 stood on the measurement of tmux alone.
        //
        // **The colour of each line stands there too** (T-257): the colours of
        // the user come of `self.config`, which `App` read at its start and at
        // the key `R`, and no line of a frame opens `config.toml` again.
        let the_lines = crate::ui::the_list_of_a_view::render_the_list(
            area,
            buf,
            &self.config.colors,
            render_list_title,
            render_list_items,
            list_state,
        );

        self.the_areas_of_the_list_of_the_mouse(
            area,
            the_lines,
            render_list_items.len(),
            list_state,
        );
    }

    /// Draws the list of a view inside the panel 4 of the frame of the panels.
    /// See T-320.
    ///
    /// **The panel 4 stands with the stack alone**: a terminal under 120
    /// columns holds no frame of the panels, and the list of it therefore keeps
    /// the block of one border at the top that it had.
    fn render_the_list_of_the_panel_4(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        render_list_title: &str,
        render_list_items: &[String],
        the_rows: Option<&[crate::ui::the_table_of_a_view::ARowOfTheTable]>,
        list_state: &mut ListState,
    ) {
        let the_panel = if self.the_frame_of_the_panels_stands() {
            Some((
                crate::ui::frame::ThePanel::TheList.the_number(),
                self.the_panel_of_the_focus == crate::ui::frame::ThePanel::TheList,
            ))
        } else {
            None
        };

        // **The table stands inside the panel 4 alone** (T-321): a view that
        // draws no panel draws the list of today, and the rows of the table
        // then reach no frame.
        let the_rows = the_rows.filter(|_| the_panel.is_some());

        let (the_lines, the_header) = crate::ui::the_list_of_a_view::render_the_table_of_a_panel(
            area,
            buf,
            &self.config.colors,
            crate::ui::the_list_of_a_view::TheContentOfAPanel {
                the_panel,
                title: render_list_title,
                lines: render_list_items,
                the_rows,
                the_header_of_the_columns: None,
            },
            list_state,
        );

        self.the_areas_of_the_list_of_the_mouse(
            area,
            the_lines,
            render_list_items.len(),
            list_state,
        );
        self.the_areas_of_the_mouse.the_header_of_the_list = the_header;
    }

    /// Writes the area of the list of the view into the areas that a report of
    /// the mouse reads. See T-316.
    ///
    /// **ratatui writes the offset of the list while it draws it**, therefore
    /// this function comes after the render and it needs no second measurement
    /// of the place of the user. That offset is the line of the list that
    /// stands on the first row of the panel, and a click of a row therefore
    /// needs it and the area together.
    fn the_areas_of_the_list_of_the_mouse(
        &mut self,
        the_panel: Rect,
        the_lines_of_the_list: Rect,
        the_lines: usize,
        list_state: &ListState,
    ) {
        self.the_areas_of_the_mouse.the_panel_of_the_list = the_panel;
        self.the_areas_of_the_mouse.the_lines_of_the_list = the_lines_of_the_list;
        self.the_areas_of_the_mouse.the_offset_of_the_list = list_state.offset();

        // **A view that draws no table holds no row of a header** (T-321): the
        // areas of the mouse are the areas of the **last** frame, therefore a
        // header of the frame before this one would take a click of a line.
        self.the_areas_of_the_mouse.the_header_of_the_list = Rect::default();
        self.the_areas_of_the_mouse.the_lines = the_lines;
    }

    // info about the book or podacst for `Home`
    fn render_info_home(&self, area: Rect, buf: &mut Buffer, in_the_panel: bool) {
        let duration_cnt_list_conv = convert_seconds(self.duration_cnt_list.clone());

        // **The facts of the design stand in the panel 5 alone** (T-325 and
        // T-326): the area under the list holds three rows in the layout of a
        // list, and a list of ten facts does not read there.
        if in_the_panel {
            if let Some(lines) = self.the_lines_of_the_facts_of_the_panel(area.width) {
                Paragraph::new(lines.join("\n"))
                    .left_aligned()
                    .render(area, buf);
                return;
            }
        }

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
                let of_the_disk = self
                    .ids_ep_cnt_list
                    .get(selected)
                    .map(|id| the_copy_of_the_disk(id))
                    .unwrap_or("");

                // **The panel of a line of a book says the place of the user,
                // and this one said nothing at all** (T-228). The two episodes
                // of `Arthur Gordon Pym` of the sandbox stood at 80 percent and
                // at 10 percent of the server.
                //
                // **The panel of an episode says the time that is left too**
                // (T-244): this paragraph named the percent and the mark of the
                // end alone, and the panel of that same episode of the view of a
                // playlist of that same run said `Progress: 100%, 0m left,
                // Finished`.
                //
                // **The panel of a media that plays reads the engine of this
                // program** (T-239): see
                // `App::the_place_of_the_panel_of_the_home_view`.
                let place = self.the_place_of_the_panel_of_the_home_view(selected);

                Paragraph::new(crate::ui::keys::the_panel_of_a_media(
                    &format!(
                        "[{}] - Author: {} - Episode: {} - Duration: {}{}",
                        at(&self.titles_pod_cnt_list, selected),
                        at(&self.authors_pod_cnt_list, selected),
                        at(&self.nums_ep_pod_cnt_list, selected),
                        at(&self.durations_pod_cnt_list, selected),
                        of_the_disk,
                    ),
                    &format!(
                        "Progress: {}%, {} {}",
                        place.percent, place.the_time_that_is_left, place.the_end,
                    ),
                ))
                .wrap(Wrap { trim: true })
                .left_aligned()
                .render(area, buf);
            } else {
                let of_the_disk = the_copy_of_the_disk(at(&self._ids_cnt_list, selected));

                // **The panel of a media that plays reads the engine of this
                // program** (T-239). The panel said `Progress: 37%, 5h left`
                // while the row of the player of that same frame said
                // `▶ 4:13:12 / 8:00:00 | Left: 3:46:48 (53%)`.
                let place = self.the_place_of_the_panel_of_the_home_view(selected);

                Paragraph::new(crate::ui::keys::the_panel_of_a_media(
                    &format!(
                        "Author: {} - Year: {} - Duration: {}{}",
                        at(&self.auth_names_cnt_list, selected),
                        at(&self.pub_year_cnt_list, selected),
                        at(&duration_cnt_list_conv, selected),
                        of_the_disk,
                    ),
                    &format!(
                        "Progress: {}%, {} {}",
                        place.percent,               // percentage progression
                        place.the_time_that_is_left, // time left
                        place.the_end,               // is finished
                    ),
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
            self.render_a_description(area, buf, &series.description_for_the_screen());
            return;
        }

        if let Some(selected) = self.selected_home_item() {
            let mut _content: String = String::new();
            if self.is_podcast {
                // **The panel of a podcast says the description of that podcast
                // when the episode holds no subtitle** (T-250). The program
                // collected `descs_pod_cnt_list` and no render read it.
                _content = crate::logic::the_panel_of_a_line::the_description_of_a_podcast(
                    at(&self.subtitles_pod_cnt_list, selected),
                    at(&self.descs_pod_cnt_list, selected),
                );
            } else {
                _content = at(&self.desc_cnt_list, selected).to_string();
            }

            self.render_a_description(area, buf, &_content.clone());
        }
    }

    // info about the book or podacst for `Library`
    fn render_info_library(&self, area: Rect, buf: &mut Buffer, in_the_panel: bool) {
        let duration_library_conv = convert_seconds(self.duration_library.clone());

        // **The facts of the design stand in the panel 5 alone** (T-325): the
        // area under the list holds three rows in the layout of a list, and a
        // list of ten facts does not read there.
        if in_the_panel {
            if let Some(lines) = self.the_lines_of_the_facts_of_the_panel(area.width) {
                Paragraph::new(lines.join("\n"))
                    .left_aligned()
                    .render(area, buf);
                return;
            }
        }

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
                let of_the_disk = the_copy_of_the_disk(at(&self.ids_library, selected));

                // **The panel of a book of this view said no place of the
                // user** (T-241): it named the author and the year, while the
                // panel of that same book of the Home view of that same frame
                // said the percent, the time that is left, and the mark of the
                // end.
                let place = self.the_place_of_the_panel_of_the_library(selected);

                Paragraph::new(crate::ui::keys::the_panel_of_a_media(
                    &format!(
                        "Author: {} - Year: {} - Duration: {}{}",
                        at(&self.auth_names_library, selected),
                        at(&self.published_year_library, selected),
                        at(&duration_library_conv, selected),
                        of_the_disk,
                    ),
                    &format!(
                        "Progress: {}%, {} {}",
                        place.percent,               // percentage progression
                        place.the_time_that_is_left, // time left
                        place.the_end,               // is finished
                    ),
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
            self.render_a_description(area, buf, &text);
        }
    }

    // info about the podcast for `PodcastEpisode`
    fn render_info_pod_ep(&self, area: Rect, buf: &mut Buffer, list_state: &ListState) {
        let Some(selected) = list_state.selected() else {
            return;
        };

        let of_the_disk = self
            .ids_pod_ep
            .get(selected)
            .map(|id| the_copy_of_the_disk(id))
            .unwrap_or("");

        // **The panel of a line of this view said nothing of the place of the
        // user** (T-229).
        //
        // **The panel of an episode says the time that is left too** (T-244):
        // the length of the episode stands in `the_lengths_of_the_episodes`
        // (T-236) and the place of the user in seconds stands in
        // `pod_ep_places`.
        //
        // **The panel of a media that plays reads the engine of this program**
        // (T-239).
        let place = self.the_place_of_the_panel_of_the_episodes(selected);

        // **A line that the lists of this view do not hold says the words of a
        // value that the program does not have** (T-288). This road held three
        // branches of the length of its lists, and each of them drew the words
        // of a program in the place of the panel and wrote one line of the log
        // at every frame.
        Paragraph::new(crate::logic::the_panel_of_a_line::the_panel_of_an_episode(
            at(&self.titles_pod, 0),
            at(&self.authors_pod_ep, 0),
            &self.episodes_pod_ep,
            &self.durations_pod_ep,
            selected,
            of_the_disk,
            &place,
        ))
        .wrap(Wrap { trim: true })
        .left_aligned()
        .render(area, buf);
    }
    // info about the podcast for `PodcastEpisode` (from search)
    fn render_info_pod_ep_search(&self, area: Rect, buf: &mut Buffer, list_state: &ListState) {
        let Some(selected) = list_state.selected() else {
            return;
        };

        let of_the_disk = self
            .ids_pod_ep_search
            .get(selected)
            .map(|id| the_copy_of_the_disk(id))
            .unwrap_or("");

        // The panel of this view says the place of the user too. See
        // T-229, T-239 for the media that plays, and T-244 for the time that
        // is left.
        let place = self.the_place_of_the_panel_of_the_episodes_of_a_search(selected);

        // **The two panels of this view say one thing for one condition**
        // (T-288): the copy of the name of the podcast of this road stood in a
        // list of the length of the lengths of the episodes, therefore a line
        // above that length lost the name of its podcast too.
        Paragraph::new(crate::logic::the_panel_of_a_line::the_panel_of_an_episode(
            at(&self.titles_pod_search, 0),
            at(&self.authors_pod_ep_search, 0),
            &self.episodes_pod_ep_search,
            &self.durations_pod_ep_search,
            selected,
            of_the_disk,
            &place,
        ))
        .wrap(Wrap { trim: true })
        .left_aligned()
        .render(area, buf);
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
                // **The panel of an episode says the description of that
                // episode when the episode holds no subtitle** (T-251). The
                // program collected `descs_pod_ep` and no render read it.
                self.render_a_description(
                    area,
                    buf,
                    &crate::logic::the_panel_of_a_line::the_description_of_a_podcast(
                        at(&self.subtitles_pod_ep, selected),
                        at(&self.descs_pod_ep, selected),
                    ),
                );
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
            // The view of the episodes of a search holds the same panel. See
            // T-251, and T-246 for the lists of that view.
            self.render_a_description(
                area,
                buf,
                &crate::logic::the_panel_of_a_line::the_description_of_a_podcast(
                    at(&self.subtitles_pod_ep_search, selected),
                    at(&self.descs_pod_ep_search, selected),
                ),
            );
        }
    }

    // info about the book or podacst for `SearchBook`
    fn render_info_search_book(&self, area: Rect, buf: &mut Buffer, list_state: &ListState) {
        let duration_library_search_book_conv =
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
                let of_the_disk = self
                    .ids_search_book
                    .get(selected)
                    .map(|id| the_copy_of_the_disk(id))
                    .unwrap_or("");
                // **The panel of a book of the view of the search said no place
                // of the user** (T-241). The rule of the Library view is the
                // rule of this view: the two panels take one function.
                let place = self.the_place_of_the_panel_of_the_search_book(selected);

                Paragraph::new(crate::ui::keys::the_panel_of_a_media(
                    &format!(
                        "Author: {} - Year: {} - Duration: {}{}",
                        at(&self.auth_names_search_book, selected),
                        at(&self.published_year_library_search_book, selected),
                        at(&duration_library_search_book_conv, selected),
                        of_the_disk,
                    ),
                    &format!(
                        "Progress: {}%, {} {}",
                        place.percent,               // percentage progression
                        place.the_time_that_is_left, // time left
                        place.the_end,               // is finished
                    ),
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
            self.render_a_description(area, buf, at(&self.desc_library_search_book, selected));
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
                self.render_a_description(area, buf, &self.changelog.clone());
            }
            Some(3) => {
                self.render_a_description(area, buf, instructions);
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
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One download of the map of the progress.
    fn a_download(done: u64, total: u64, files: usize) -> DownloadProgress {
        DownloadProgress {
            key: "item-1".to_string(),
            title: "Alice in Wonderland".to_string(),
            file_index: 1,
            file_count: files,
            bytes_done: done,
            bytes_total: total,
            state: DownloadState::Running,
        }
    }

    /// **A total of 0 is a total that the server did not give** (T-179), and
    /// the bar must not say that the whole book holds 0.0 MB while the program
    /// writes its bytes.
    #[test]
    fn the_label_of_a_download_of_no_total_holds_the_bytes_of_the_disk() {
        let words = the_label_of_a_download(&a_download(20_554, 0, 1));

        assert!(words.contains("Alice in Wonderland"), "{words}");
        assert!(!words.contains('/'), "the label names no total: {words}");
        assert!(words.contains("0.0 MB"), "{words}");

        let of_many = the_label_of_a_download(&a_download(20_554, 0, 3));

        assert!(of_many.contains("file 1/3"), "{of_many}");
        assert!(
            !of_many.contains("MB / "),
            "the label names no total: {of_many}"
        );
    }

    /// A total that the server gave stays in the label.
    #[test]
    fn the_label_of_a_download_holds_the_total_of_the_server() {
        let words = the_label_of_a_download(&a_download(1_048_576, 2_097_152, 1));

        assert!(words.contains("1.0 MB / 2.0 MB"), "{words}");
    }

    /// The words of the sequence and of the filter must leave every letter of
    /// the address of the server. See T-329.
    ///
    /// A measurement of the real program v0.8.158 at 84 columns read the second
    /// row of the header:
    ///
    /// ```text
    /// 🔗 localhost:13399title, the largest first ▣ The media that you finished
    /// ```
    ///
    /// A `Paragraph` of the whole area writes its text at the middle of that
    /// area, and the address of the row before it holds those columns already.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn the_words_of_the_sequence_leave_the_address_whole() {
        let area = Rect {
            x: 0,
            y: 0,
            width: 84,
            height: 3,
        };

        let mut buf = Buffer::empty(area);

        // The header wrote the address of the server on the second row.
        buf.set_string(0, 1, "🔗 localhost:13399", Style::default());

        App::draw_the_words_of_the_sequence(
            area,
            &mut buf,
            "⇅ The title, the largest first ▣ The media that you finished",
            18,
            0,
        );

        let row: String = (0..area.width)
            .map(|column| buf[(column, 1)].symbol().to_string())
            .collect();

        // **A mark of two columns takes two cells of the buffer**, and the
        // second of them holds no symbol at all: a row that this test joins
        // therefore holds one space more than the screen (the trap 245).
        assert!(
            row.contains(
                "localhost:13399  ⇅ The title, the largest first ▣ The media that you finished"
            ),
            "the address stays whole, and the words stand beside it: {row:?}"
        );

        // A row that has no room for the whole of the words holds none of them,
        // and the address of it stays whole.
        let narrow = Rect {
            x: 0,
            y: 0,
            width: 60,
            height: 3,
        };

        let mut buf = Buffer::empty(narrow);
        buf.set_string(0, 1, "🔗 localhost:13399", Style::default());

        App::draw_the_words_of_the_sequence(
            narrow,
            &mut buf,
            "⇅ The title, the largest first ▣ The media that you finished",
            18,
            0,
        );

        let row: String = (0..narrow.width)
            .map(|column| buf[(column, 1)].symbol().to_string())
            .collect();

        assert!(
            row.trim_end().ends_with("localhost:13399"),
            "a narrow row holds the address alone: {row:?}"
        );
    }

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

        // **The message stands on the rows that it has** (T-299). A message of
        // more letters than the width of the screen went to the three points
        // before, and it now wraps over the rows of its area.
        let rows = Rect {
            x: 0,
            y: 0,
            width: 20,
            height: 3,
        };

        let mut buf = Buffer::empty(rows);
        let sentence = "The disk keeps the copies of that account.";

        App::draw_the_row_of_the_message(rows, &mut buf, sentence, Style::default());

        let words: String = (0..rows.height)
            .map(|row| {
                (0..rows.width)
                    .map(|column| buf[(column, row)].symbol().to_string())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join(" ");
        let words = words.split_whitespace().collect::<Vec<_>>().join(" ");

        assert_eq!(words, sentence, "the message did not wrap over its rows");
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

        let [header, main, footer] =
            the_areas_of_a_view(screen, PLAYER_HEIGHT, FOOTER_HEIGHT, THE_SMALLEST_LIST);
        let [header_of_no_playback, main_of_no_playback, footer_of_no_playback] =
            the_areas_of_a_view(screen, 0, FOOTER_HEIGHT, THE_SMALLEST_LIST);

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

            let [_, of_a_playback, _] =
                the_areas_of_a_view(screen, PLAYER_HEIGHT, FOOTER_HEIGHT, THE_SMALLEST_LIST);
            let [_, of_no_playback, _] =
                the_areas_of_a_view(screen, 0, FOOTER_HEIGHT, THE_SMALLEST_LIST);

            let [list_of_a_playback, ..] = the_areas_of_a_list(of_a_playback, 0, THE_SMALLEST_LIST);
            let [list_of_no_playback, ..] =
                the_areas_of_a_list(of_no_playback, PLAYER_HEIGHT, THE_SMALLEST_LIST);

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

    /// **The list of a view goes away last** (T-342).
    ///
    /// The measurement of the real program v0.8.172 inside tmux, in a terminal
    /// of 100 columns and 8 rows: the Library view said
    /// `Library [500 items of 2056]` in the title of its border, the row of the
    /// item said `Author: N/A - Year: N/A - Duration: 0m` and
    /// `Progress:  N/A%,   N/A` on the two rows under it, and **no one of the
    /// 500 items stood on the screen**. The row of the item is a
    /// `Constraint::Length` and the list is a `Constraint::Fill`, therefore the
    /// solver of ratatui gave the row of the item its two rows first.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn the_list_of_a_view_keeps_its_line_before_the_row_of_the_item() {
        // A view of 4 rows and more keeps the two rows that it had, therefore
        // every screen that stood before T-342 stands in the same shape.
        for rows_of_the_view in 4..=12u16 {
            assert_eq!(
                the_rows_of_the_row_of_the_item(rows_of_the_view, THE_SMALLEST_LIST),
                2,
                "the view of {rows_of_the_view} rows must keep the row of the item"
            );
        }

        // A view of fewer rows gives the list its border and one line first.
        assert_eq!(the_rows_of_the_row_of_the_item(3, THE_SMALLEST_LIST), 1);
        assert_eq!(the_rows_of_the_row_of_the_item(2, THE_SMALLEST_LIST), 0);
        assert_eq!(the_rows_of_the_row_of_the_item(1, THE_SMALLEST_LIST), 0);
        assert_eq!(the_rows_of_the_row_of_the_item(0, THE_SMALLEST_LIST), 0);

        // The areas themselves, of a screen of 100 columns. A screen of 8 rows
        // gives the work of a view 3 rows: the header takes 2, the row of the
        // message takes 1, and the footer takes 2. **A screen of 6 rows gives
        // that work 2 rows** (T-345): the header of such a screen keeps one row
        // alone, therefore the list holds its border and one line there too.
        for (rows_of_the_screen, rows_of_the_list) in
            [(8u16, 2u16), (7, 2), (6, 2), (12, 5), (13, 6)]
        {
            let screen = Rect {
                x: 0,
                y: 0,
                width: 100,
                height: rows_of_the_screen,
            };

            let [_, main_area, _] =
                the_areas_of_a_view(screen, 0, FOOTER_HEIGHT, THE_SMALLEST_LIST);
            let [list, of_the_item, _] = the_areas_of_a_list(main_area, 0, THE_SMALLEST_LIST);

            assert_eq!(
                list.height, rows_of_the_list,
                "a screen of {rows_of_the_screen} rows must give the list \
                 {rows_of_the_list} rows, and it gives {}",
                list.height
            );
            assert!(
                list.height >= THE_SMALLEST_LIST || list.height == main_area.height,
                "a screen of {rows_of_the_screen} rows gives the list \
                 {} rows of the {} rows of the work of the view",
                list.height,
                main_area.height
            );
            assert_eq!(
                list.height + of_the_item.height,
                main_area.height,
                "the two parts must hold every row of the work of the view"
            );
        }
    }

    /// **The panel of the item of the seven views goes away before the list**
    /// (T-344).
    ///
    /// The measurement of the real program v0.8.174 inside tmux, in a terminal
    /// of 100 columns and 8 rows, of the library `Books` of nine authors: the
    /// Authors view of the key `a` said `No description available` and
    /// **nothing else** — no title of its border, and no author at all. At 10
    /// rows and at 11 rows it said `The authors [9 items]` and no author. The
    /// Home view of the same screen kept its title and one line, because
    /// T-342 corrected it already.
    ///
    /// Seven views hold that panel as a `Constraint::Length` of 4 or of 5:
    /// the Authors view, the view of the lists that take a media, the view of
    /// the devices of an e-reader, the view of the downloads of the server,
    /// the view of the ebooks of a media, the view of a new podcast, and the
    /// settings of the reader.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn the_list_of_the_seven_views_keeps_its_line_before_the_panel_of_the_item() {
        // A view of 6 rows and more keeps the four rows that it had, therefore
        // every screen that stood before T-344 stands in the same shape.
        for rows_of_the_view in 6..=40u16 {
            assert_eq!(
                the_rows_of_the_panel_of_the_item(rows_of_the_view, 4, THE_SMALLEST_LIST),
                4,
                "the view of {rows_of_the_view} rows must keep the panel of the item"
            );
        }

        // A view of fewer rows gives the list its border and one line first.
        assert_eq!(
            the_rows_of_the_panel_of_the_item(5, 4, THE_SMALLEST_LIST),
            3
        );
        assert_eq!(
            the_rows_of_the_panel_of_the_item(4, 4, THE_SMALLEST_LIST),
            2
        );
        assert_eq!(
            the_rows_of_the_panel_of_the_item(3, 4, THE_SMALLEST_LIST),
            1
        );
        assert_eq!(
            the_rows_of_the_panel_of_the_item(2, 4, THE_SMALLEST_LIST),
            0
        );
        assert_eq!(
            the_rows_of_the_panel_of_the_item(1, 4, THE_SMALLEST_LIST),
            0
        );
        assert_eq!(
            the_rows_of_the_panel_of_the_item(0, 4, THE_SMALLEST_LIST),
            0
        );

        // The settings of the reader hold a panel of five rows, and that panel
        // takes the same road.
        assert_eq!(
            the_rows_of_the_panel_of_the_item(7, 5, THE_SMALLEST_LIST),
            5
        );
        assert_eq!(
            the_rows_of_the_panel_of_the_item(6, 5, THE_SMALLEST_LIST),
            4
        );
        assert_eq!(
            the_rows_of_the_panel_of_the_item(3, 5, THE_SMALLEST_LIST),
            1
        );
        assert_eq!(
            the_rows_of_the_panel_of_the_item(2, 5, THE_SMALLEST_LIST),
            0
        );

        // The rule of T-342 is the same rule with two rows.
        for rows_of_the_view in 0..=40u16 {
            assert_eq!(
                the_rows_of_the_row_of_the_item(rows_of_the_view, THE_SMALLEST_LIST),
                the_rows_of_the_panel_of_the_item(rows_of_the_view, 2, THE_SMALLEST_LIST),
                "the row of the item of {rows_of_the_view} rows takes the same road"
            );
        }

        // The areas themselves, of a screen of 100 columns. A screen of 8 rows
        // gives the work of a view 3 rows: the header takes 2, the row of the
        // message takes 1, and the footer takes 2. The list of every one of
        // them keeps its border and one line.
        for (rows_of_the_screen, rows_of_the_list) in
            [(8u16, 2u16), (9, 2), (10, 2), (11, 2), (12, 3), (13, 4)]
        {
            let screen = Rect {
                x: 0,
                y: 0,
                width: 100,
                height: rows_of_the_screen,
            };

            let [_, work_area, _] =
                the_areas_of_a_view(screen, 0, FOOTER_HEIGHT, THE_SMALLEST_LIST);

            let [list, of_the_item] = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(the_rows_of_the_panel_of_the_item(
                    work_area.height,
                    4,
                    THE_SMALLEST_LIST,
                )),
            ])
            .areas(work_area);

            assert_eq!(
                list.height, rows_of_the_list,
                "a screen of {rows_of_the_screen} rows must give the list \
                 {rows_of_the_list} rows, and it gives {}",
                list.height
            );
            assert!(
                list.height >= THE_SMALLEST_LIST || list.height == work_area.height,
                "a screen of {rows_of_the_screen} rows gives the list \
                 {} rows of the {} rows of the work of the view",
                list.height,
                work_area.height
            );
            assert_eq!(
                list.height + of_the_item.height,
                work_area.height,
                "the two parts must hold every row of the work of the view"
            );
        }
    }

    /// **The band of the player and the bars of the Chapters view go away
    /// before the work of a view** (T-343, and the decision of T-342).
    ///
    /// The measurement of the real program v0.8.173 inside tmux, in a terminal
    /// of 100 columns and 8 rows, with a book of 70 chapters that plays: the
    /// Chapters view said `The chapters of "A Second Book Of Many Hours" [70
    /// items]` in the title of its border and **no one of the 70 chapters stood
    /// on the screen**, because the two bars took the three rows of the panel
    /// first. The band of the player took three rows of that same screen.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn the_band_and_the_bars_go_away_before_the_work_of_a_view() {
        // A screen of many rows keeps the whole band, therefore every screen
        // that stood before T-343 stands in the same shape.
        for rows_of_the_screen in 13..=45u16 {
            assert_eq!(
                the_rows_of_the_band_of_a_screen(
                    rows_of_the_screen,
                    PLAYER_HEIGHT,
                    FOOTER_HEIGHT,
                    THE_SMALLEST_LIST
                ),
                PLAYER_HEIGHT,
                "a screen of {rows_of_the_screen} rows must keep the whole band"
            );
        }

        // A screen of fewer rows gives the work of the view its border and one
        // line first: the header takes 2 rows, the row of the message takes 1,
        // and the footer takes the rows of its text.
        assert_eq!(
            the_rows_of_the_band_of_a_screen(12, PLAYER_HEIGHT, FOOTER_HEIGHT, THE_SMALLEST_LIST),
            5
        );
        assert_eq!(
            the_rows_of_the_band_of_a_screen(11, PLAYER_HEIGHT, FOOTER_HEIGHT, THE_SMALLEST_LIST),
            4
        );
        assert_eq!(
            the_rows_of_the_band_of_a_screen(10, PLAYER_HEIGHT, FOOTER_HEIGHT, THE_SMALLEST_LIST),
            3
        );

        // **A band of fewer than THE_SMALLEST_BAND rows says nothing at all**:
        // the two rows of its border take the room of the view, and the user
        // reads no media and no place.
        for rows_of_the_screen in 0..=9u16 {
            assert_eq!(
                the_rows_of_the_band_of_a_screen(
                    rows_of_the_screen,
                    PLAYER_HEIGHT,
                    FOOTER_HEIGHT,
                    THE_SMALLEST_LIST
                ),
                0,
                "a screen of {rows_of_the_screen} rows must give the band no row"
            );
        }

        // The areas themselves: the work of every view keeps its border and one
        // line while the screen holds the header, the message, and the footer.
        for rows_of_the_screen in 0..=45u16 {
            let screen = Rect {
                x: 0,
                y: 0,
                width: 100,
                height: rows_of_the_screen,
            };

            let [_, main_area, _] =
                the_areas_of_a_view(screen, PLAYER_HEIGHT, FOOTER_HEIGHT, THE_SMALLEST_LIST);
            let band =
                the_area_of_the_band(screen, PLAYER_HEIGHT, FOOTER_HEIGHT, THE_SMALLEST_LIST);

            let of_the_others = HEADER_HEIGHT + 1 + FOOTER_HEIGHT;

            assert!(
                main_area.height
                    >= THE_SMALLEST_LIST.min(rows_of_the_screen.saturating_sub(of_the_others)),
                "a screen of {rows_of_the_screen} rows gives the work of the view \
                 {} rows",
                main_area.height
            );

            // **The band takes no row of the work of the view** (T-343): the
            // band drew over the last lines of every view that built its
            // layout itself, therefore the two areas must not meet.
            assert!(
                band.height == 0 || band.y >= main_area.y + main_area.height,
                "the band of a screen of {rows_of_the_screen} rows stands over \
                 the work of the view"
            );
        }

        // The two bars of the Chapters view take the same road.
        for rows_of_the_panel in 5..=45u16 {
            assert_eq!(
                the_rows_of_the_bars_of_the_chapters(rows_of_the_panel, true),
                3,
                "a panel of {rows_of_the_panel} rows must keep the three rows \
                 of the bars"
            );
        }

        assert_eq!(the_rows_of_the_bars_of_the_chapters(4, true), 2);
        assert_eq!(the_rows_of_the_bars_of_the_chapters(3, true), 1);
        assert_eq!(the_rows_of_the_bars_of_the_chapters(2, true), 0);
        assert_eq!(the_rows_of_the_bars_of_the_chapters(0, true), 0);

        // A media of no chapter gives no bar at all, at every number of rows.
        assert_eq!(the_rows_of_the_bars_of_the_chapters(45, false), 0);
    }

    /// **The row of the message of a frame stands above the footer of that same
    /// frame** (T-346, and the decision of T-345).
    ///
    /// `render_the_message` read `HEADER_HEIGHT` and the rows that the **text**
    /// of the footer wants, and [`the_rows_around_the_work_of_a_view`] gives the
    /// rows that the screen gave them: the two numbers disagree at 3 rows and
    /// fewer. The measurement of the real program v0.8.176 inside tmux, in a
    /// terminal of 100 columns, of the message of the key `Ctrl+o`: at 3 rows
    /// the message stood over the title `Home [20 items]` of the list while the
    /// row above the footer stayed free, and **at 2 rows and at 1 row the
    /// program said nothing at all**.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn the_row_of_the_message_stands_above_the_footer_of_the_frame() {
        for rows_of_the_screen in 1..=45u16 {
            let screen = Rect {
                x: 0,
                y: 0,
                width: 100,
                height: rows_of_the_screen,
            };

            let [_, _, _, message_area, footer_area] =
                the_five_areas(screen, 0, FOOTER_HEIGHT, THE_SMALLEST_LIST);

            // **A screen of one row and more says the message of the program.**
            let (y, rows) =
                the_place_of_the_message_of_a_frame(screen, FOOTER_HEIGHT, 1, THE_SMALLEST_LIST)
                    .unwrap_or_else(|| {
                        panic!("a screen of {rows_of_the_screen} rows says no message at all")
                    });

            assert!(rows >= 1, "a message of no row says nothing");

            // The last row of the message stands right above the footer of this
            // frame, and the footer keeps every row that the frame gave it.
            assert_eq!(
                y + rows,
                footer_area.y,
                "the message of a screen of {rows_of_the_screen} rows stands \
                 {rows} rows at the row {y}, and the footer of it stands at the \
                 row {}",
                footer_area.y
            );

            // The message stays inside the screen, and it takes no row of the
            // header of the frame.
            assert!(y + rows <= rows_of_the_screen);

            // A screen that holds the row of the message writes the message
            // there and it grows no further while one row is enough.
            if message_area.height == 1 {
                assert_eq!(
                    y, message_area.y,
                    "a screen of {rows_of_the_screen} rows holds the row of the \
                     message at {}, and the message stands at {y}",
                    message_area.y
                );
            }
        }

        // A message of many rows grows upward over the work of the view, and it
        // stops at the header of the frame. A screen of 45 rows holds the two
        // rows of its header.
        let tall = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 45,
        };

        assert_eq!(
            the_place_of_the_message_of_a_frame(tall, FOOTER_HEIGHT, 4, THE_SMALLEST_LIST),
            Some((39, 4))
        );
        assert_eq!(
            the_place_of_the_message_of_a_frame(tall, FOOTER_HEIGHT, 100, THE_SMALLEST_LIST),
            Some((HEADER_HEIGHT, 41))
        );

        // A footer of three rows (T-302) takes the same road: the message reads
        // the rows of the footer of the frame and not the rows of a fixed one.
        let short = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 5,
        };

        assert_eq!(
            the_place_of_the_message_of_a_frame(short, 3, 1, THE_SMALLEST_LIST),
            Some((1, 1))
        );
    }

    /// **The header, the row of the message, and the footer go away before the
    /// work of a view** (T-345, and the decision of T-342).
    ///
    /// The measurement of the real program v0.8.175 inside tmux, in a terminal
    /// of 100 columns: at 5 rows the Home view held the two rows of the header,
    /// one blank row, and the two rows of the footer, and **no title of the list
    /// and no line of it at all**; at 3 rows it held the footer alone; and at 1
    /// row it held **no letter at all**.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn the_work_of_a_view_goes_away_after_the_parts_around_it() {
        // A screen of 7 rows and more keeps the three parts whole, therefore
        // every screen that stood before T-345 stands in the same shape.
        for rows_of_the_screen in 7..=45u16 {
            assert_eq!(
                the_rows_around_the_work_of_a_view(
                    rows_of_the_screen,
                    FOOTER_HEIGHT,
                    THE_SMALLEST_LIST
                ),
                [HEADER_HEIGHT, 1, FOOTER_HEIGHT],
                "a screen of {rows_of_the_screen} rows must keep the three parts"
            );
        }

        // The header goes away first, one row at a time; the row of the message
        // after it; and the footer last.
        assert_eq!(
            the_rows_around_the_work_of_a_view(6, FOOTER_HEIGHT, THE_SMALLEST_LIST),
            [1, 1, 2]
        );
        assert_eq!(
            the_rows_around_the_work_of_a_view(5, FOOTER_HEIGHT, THE_SMALLEST_LIST),
            [0, 1, 2]
        );
        assert_eq!(
            the_rows_around_the_work_of_a_view(4, FOOTER_HEIGHT, THE_SMALLEST_LIST),
            [0, 0, 2]
        );
        assert_eq!(
            the_rows_around_the_work_of_a_view(3, FOOTER_HEIGHT, THE_SMALLEST_LIST),
            [0, 0, 1]
        );
        assert_eq!(
            the_rows_around_the_work_of_a_view(2, FOOTER_HEIGHT, THE_SMALLEST_LIST),
            [0, 0, 0]
        );
        assert_eq!(
            the_rows_around_the_work_of_a_view(1, FOOTER_HEIGHT, THE_SMALLEST_LIST),
            [0, 0, 0]
        );
        assert_eq!(
            the_rows_around_the_work_of_a_view(0, FOOTER_HEIGHT, THE_SMALLEST_LIST),
            [0, 0, 0]
        );

        // A footer of three rows takes the same road: it keeps its rows while
        // the work of the view holds its border and one line.
        assert_eq!(
            the_rows_around_the_work_of_a_view(8, 3, THE_SMALLEST_LIST),
            [2, 1, 3]
        );
        assert_eq!(
            the_rows_around_the_work_of_a_view(5, 3, THE_SMALLEST_LIST),
            [0, 0, 3]
        );
        assert_eq!(
            the_rows_around_the_work_of_a_view(4, 3, THE_SMALLEST_LIST),
            [0, 0, 2]
        );

        // The areas themselves: the work of every view keeps its border and one
        // line while the screen holds two rows at all.
        for rows_of_the_screen in 0..=45u16 {
            let screen = Rect {
                x: 0,
                y: 0,
                width: 100,
                height: rows_of_the_screen,
            };

            let [_, main_area, _] =
                the_areas_of_a_view(screen, PLAYER_HEIGHT, FOOTER_HEIGHT, THE_SMALLEST_LIST);

            assert!(
                main_area.height >= THE_SMALLEST_LIST.min(rows_of_the_screen),
                "a screen of {rows_of_the_screen} rows gives the work of the view \
                 {} rows",
                main_area.height
            );
        }

        // The rows of the work of a view of the smallest screens.
        for (rows_of_the_screen, rows_of_the_view) in
            [(1, 1), (2, 2), (3, 2), (4, 2), (5, 2), (6, 2)]
        {
            let screen = Rect {
                x: 0,
                y: 0,
                width: 100,
                height: rows_of_the_screen,
            };

            let [_, main_area, _] =
                the_areas_of_a_view(screen, PLAYER_HEIGHT, FOOTER_HEIGHT, THE_SMALLEST_LIST);

            assert_eq!(
                main_area.height, rows_of_the_view,
                "the work of a view of a screen of {rows_of_the_screen} rows"
            );
        }
    }

    /// **The footer of a view stands on the rows that it needs** (T-302).
    ///
    /// The measurement of the real program v0.8.130 inside tmux, in a terminal
    /// of 40 columns and 30 rows: the Home view said `j/k: move  l: play or
    /// open  Tab: home/library  S-Tab: the next library` on its two rows, and
    /// the keys `/: search`, `R: refresh`, `?: every key`, and `Q: quit` stood
    /// outside the screen. The user of that terminal read no road to the table
    /// of the keys and no road out of the program.
    ///
    /// The footer grows upward over the work of the view, in the same way as
    /// the message of T-299: the list of the view loses a line, and the key `j`
    /// moves the list.
    #[test]
    fn the_footer_of_a_view_stands_on_the_rows_that_it_needs() {
        let keys = crate::ui::keys::FOOTER_OF_A_LIBRARY_OF_BOOKS;

        for (width, rows_that_it_needs) in [(40u16, 4u16), (80, 2), (160, 2)] {
            let screen = Rect {
                x: 0,
                y: 0,
                width,
                height: 30,
            };

            let rows = crate::ui::keys::the_rows_of_a_footer(keys, width, screen.height);

            assert_eq!(rows, rows_that_it_needs, "the rows at {} columns", width);

            let [header, main, footer] = the_areas_of_a_view(screen, 0, rows, THE_SMALLEST_LIST);

            assert_eq!(footer.height, rows_that_it_needs);
            assert_eq!(
                footer.y + footer.height,
                screen.height,
                "the footer keeps the last row of the screen"
            );
            assert_eq!(
                header.height, HEADER_HEIGHT,
                "the header of the screen keeps its rows"
            );
            assert_eq!(
                main.y + main.height,
                footer.y - 1,
                "the row of the message stands between the view and the footer"
            );

            // Every key of the footer stands on the screen.
            let mut buf = Buffer::empty(screen);
            App::render_footer(footer, &mut buf, keys);

            let words: String = (footer.y..footer.y + footer.height)
                .map(|row| {
                    (0..footer.width)
                        .map(|column| buf[(column, row)].symbol().to_string())
                        .collect::<String>()
                })
                .collect::<Vec<_>>()
                .join(" ");
            let words = words.split_whitespace().collect::<Vec<_>>().join(" ");

            assert_eq!(
                words,
                keys.split_whitespace().collect::<Vec<_>>().join(" "),
                "the footer lost its end at {} columns",
                width
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

            for rows_of_the_band in [PLAYER_HEIGHT, PLAYER_HEIGHT - 1, 0] {
                let [header, main, footer] =
                    the_areas_of_a_view(screen, rows_of_the_band, FOOTER_HEIGHT, THE_SMALLEST_LIST);

                assert!(header.y + header.height <= screen.height.max(1) + 1);
                assert!(main.y >= header.y);
                assert!(footer.y + footer.height <= screen.y + screen.height + FOOTER_HEIGHT);
            }
        }
    }

    /// **The panel of the frame of the panels keeps its border of four sides
    /// and one line** (T-347, and the decision of T-342).
    ///
    /// Every measurement of the rows of a screen before this one ran at 100
    /// columns, where the frame of the panels does not stand (T-342 to T-346).
    /// The measurement of the real program v0.8.177 inside tmux at **160
    /// columns**, of the Home view of the library `Large`: the panel 4 said
    /// `4 Home [20 items]` in the title of its border and it held **no line at
    /// all** at 8, 6, 5, 4, 3, and 2 rows, and at 8 rows the panel of the item
    /// under it still held a row of its own.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn the_panel_of_the_frame_keeps_its_border_of_four_sides_and_one_line() {
        // The panel 4 draws a border above its lines and one under them, and
        // every other view of this program draws the one border at the top that
        // it had.
        assert_eq!(the_smallest_work_of_a_view(true), 3);
        assert_eq!(the_smallest_work_of_a_view(false), THE_SMALLEST_LIST);

        for the_frame_stands in [true, false] {
            let the_smallest_work = the_smallest_work_of_a_view(the_frame_stands);

            // The rows of the border of the work of the view. A block of
            // `Borders::TOP` takes one row, and the panel 4 takes two.
            let of_the_border = if the_frame_stands { 2 } else { 1 };

            for rows_of_the_screen in 1..=45u16 {
                let screen = Rect {
                    x: 0,
                    y: 0,
                    width: 160,
                    height: rows_of_the_screen,
                };

                let [_, work_area, _] =
                    the_areas_of_a_view(screen, 0, FOOTER_HEIGHT, the_smallest_work);

                // **The work of the view keeps its border and one line while
                // the screen holds the rows of them at all** (T-342 and T-345).
                let of_the_line = work_area.height.saturating_sub(of_the_border);

                assert!(
                    of_the_line >= 1 || rows_of_the_screen < the_smallest_work,
                    "a screen of {rows_of_the_screen} rows gives the work of \
                     the view {} rows, and its border takes {of_the_border} of \
                     them: the list holds no line at all",
                    work_area.height
                );

                // **The panel of the item goes away before the list** (T-344),
                // therefore it takes no row that the border and the line of the
                // list need.
                let [list, of_the_item, _] = the_areas_of_a_list(work_area, 0, the_smallest_work);

                // A view of 13 rows and more keeps the split of a large
                // terminal, where the description takes the rows under the
                // panel of the item.
                if work_area.height <= 12 {
                    assert_eq!(
                        list.height + of_the_item.height,
                        work_area.height,
                        "the list and the panel of the item hold every row of \
                         the work of a screen of {rows_of_the_screen} rows"
                    );
                }
                assert!(
                    list.height.saturating_sub(of_the_border) >= 1
                        || rows_of_the_screen < the_smallest_work,
                    "a screen of {rows_of_the_screen} rows gives the list {} \
                     rows and the panel of the item {} rows",
                    list.height,
                    of_the_item.height
                );

                // **The band of the player goes away before the work of the
                // view** (T-343), and it reads the same number.
                let band =
                    the_area_of_the_band(screen, PLAYER_HEIGHT, FOOTER_HEIGHT, the_smallest_work);
                let [_, work_of_a_playback, _] =
                    the_areas_of_a_view(screen, PLAYER_HEIGHT, FOOTER_HEIGHT, the_smallest_work);

                assert!(
                    work_of_a_playback.height.saturating_sub(of_the_border) >= 1
                        || rows_of_the_screen < the_smallest_work,
                    "a screen of {rows_of_the_screen} rows with a band of {} \
                     rows gives the work of the view {} rows",
                    band.height,
                    work_of_a_playback.height
                );
            }
        }

        // The numbers of the frame of the panels, of a screen of few rows and a
        // footer of two rows: the work of the view keeps three rows while the
        // screen holds them, and the footer, the row of the message, and the
        // header take what stays (T-345).
        assert_eq!(
            the_rows_around_the_work_of_a_view(8, FOOTER_HEIGHT, 3),
            [2, 1, 2]
        );
        assert_eq!(
            the_rows_around_the_work_of_a_view(6, FOOTER_HEIGHT, 3),
            [0, 1, 2]
        );
        assert_eq!(
            the_rows_around_the_work_of_a_view(5, FOOTER_HEIGHT, 3),
            [0, 0, 2]
        );
        assert_eq!(
            the_rows_around_the_work_of_a_view(4, FOOTER_HEIGHT, 3),
            [0, 0, 1]
        );
        assert_eq!(
            the_rows_around_the_work_of_a_view(3, FOOTER_HEIGHT, 3),
            [0, 0, 0]
        );
    }
}
