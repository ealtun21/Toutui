use crate::api::libraries::get_all_libraries::*;
use crate::api::libraries::get_all_series::*;
use crate::api::libraries::get_library_perso_view::get_the_shelves;
use crate::api::libraries::get_library_perso_view_pod::get_the_shelves_pod;
use crate::api::libraries::get_lists::*;
use crate::api::library_items::get_pod_ep::*;
use crate::api::me::get_media_progress::*;
use crate::api::utils::collect_get_all_books::*;
use crate::api::utils::collect_get_all_libraries::*;
use crate::api::utils::collect_get_media_progress::*;
use crate::api::utils::collect_get_pod_ep::*;
use crate::api::utils::collect_lists::*;
use crate::api::utils::collect_personalized_view::*;
use crate::api::utils::collect_personalized_view_pod::*;
use crate::api::utils::collect_series::*;
use crate::config::*;
use crate::db::crud::*;
use crate::db::database_struct::Database;
use crate::logic::download::{download_with_progress, remove_download, DownloadTarget};
use crate::logic::home_view::{group_home, group_home_pod, HomeRow};
use crate::logic::library_view::{group_library, LibraryRow};
use crate::logic::playback::{play, PlaybackTarget};
use crate::logic::sync_session::sync_session_from_database::*;
use crate::player::engine::PlayerHandle;
use crate::player::integrated::handle_key_player::*;
use crate::utils::changelog::*;
use crate::utils::check_update::*;
use crate::utils::encrypt_token::*;
use color_eyre::Result;
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent, KeyEventKind},
    widgets::ListState,
};

/// The views of the application. The type has no field, therefore a copy
/// costs nothing, and a test can name a view in a list.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AppView {
    Home,
    Library,
    SearchBook,
    PodcastEpisode,
    /// The list of the series of the library. See T-22.
    Series,
    /// The books of one series, in the sequence of the series.
    SeriesBook,
    /// The collections and the playlists of the library. See T-9.
    Lists,
    /// The media of one collection or of one playlist.
    ListEntries,
    /// The reader of an ebook. See T-10.
    Reader,
    /// The statistics of the user. See T-24.
    Stats,
    /// Every session of the user, with pages. See T-24.
    Sessions,
    /// The sequence and the filter of the library. See T-24.
    SortFilter,
    /// The chapters of the media that plays. See T-24.
    Chapters,
    /// The bookmarks of one media. See T-24.
    Bookmarks,
    /// The media that wait in the queue. See T-24.
    Queue,
    /// The podcasts that the server found for the words of the user. See
    /// T-24.
    NewPodcast,
    /// The authors of the library. See T-24.
    Authors,
    /// The ebooks of one media. An item can hold more than one. See T-76.
    Ebooks,
    /// The episodes that the server downloads, and the queue of that work.
    /// See T-81.
    Downloads,
    /// The collections and the playlists that can take the media of the line.
    /// See T-84.
    PutInAList,
    /// The devices of an e-reader that can take the book of the line. See
    /// T-119.
    SendToEreader,
    /// Every key of the program. The key `?` opens it. See T-49.
    Keys,
    Settings,
    SettingsAccount,
    SettingsLibrary,
    SettingsAbout,
    SettingsUpdateUninstall,
    /// The values of the block `[reader]` of `config.toml`. See T-77.
    SettingsReader,
}

/// One picture of a page of a PDF, in the form that the screen draws. See T-54.
pub type PictureOfThePage = ratatui_image::protocol::StatefulProtocol;

pub struct App {
    pub view_state: AppView,
    /// The HTTP client. It holds the addresses of the server and the token.
    pub api: std::sync::Arc<crate::api::client::ApiClient>,
    pub database: Database,
    pub id_selected_lib: String,
    pub token: Option<String>,
    pub should_exit: bool,
    pub list_state_cnt_list: ListState,
    pub list_state_library: ListState,
    pub list_state_search_results: ListState,
    pub list_state_pod_ep: ListState,
    /// The list of the series. See T-22.
    pub list_state_series: ListState,
    /// The list of the books of one series.
    pub list_state_series_book: ListState,
    /// The list of the collections and of the playlists. See T-9.
    pub list_state_lists: ListState,
    /// The list of the media of one collection or of one playlist.
    pub list_state_list_entries: ListState,
    pub list_state_settings: ListState,
    pub list_state_settings_account: ListState,
    pub list_state_settings_library: ListState,
    pub list_state_settings_about: ListState,
    pub list_state_settings_update_uninstall: ListState,
    pub _titles_cnt_list: Vec<String>,
    pub auth_names_cnt_list: Vec<String>,
    pub pub_year_cnt_list: Vec<String>,
    pub duration_cnt_list: Vec<f64>,
    pub desc_cnt_list: Vec<String>,
    pub _ids_cnt_list: Vec<String>,
    pub titles_library: Vec<String>,
    pub ids_library: Vec<String>,
    pub auth_names_library: Vec<String>,
    pub ids_search_book: Vec<String>,
    /// The series of the library, with their books. A podcast library has no
    /// series, thus this list is then empty. See T-22.
    pub series: Vec<SeriesView>,
    /// The lines of the Library view. Every book of a series gives one line.
    /// See T-22.
    pub library_rows: Vec<LibraryRow>,
    /// The number of the page of the library that came last. The first page is
    /// 0. See T-70.
    pub library_page: usize,
    /// The number of items of the library, over every page. The server gives
    /// this value with the first page.
    pub library_total: usize,
    /// The sequence and the filter of the library, for the request of the next
    /// page. See T-70.
    pub library_query: String,
    /// `true` while the program reads the pages that are left, for the key `G`.
    ///
    /// The key `G` means "go to the end of the library". The program holds one
    /// page of 500 items at the start, therefore the end of the lines is not the
    /// end of the library: a user of a library of 2056 items had to press that
    /// key **six** times. The program asks for the page that is left now, and it
    /// takes the last line again at each page. See T-112.
    pub reads_every_page_of_the_library: bool,
    /// `true` while the program reads the pages that the search of a library of
    /// podcasts needs. The line of the user does not move for this work.
    /// See T-125.
    pub reads_the_pages_for_the_search: bool,
    /// The lines of the Home view. A shelf gives a line for its name, and a
    /// line for each of its media. See T-24.
    ///
    /// A media that left the shelf of Continue Listening is absent from this
    /// list. `home_rows_of_the_server` holds every line. See T-66.
    pub home_rows: Vec<HomeRow>,
    /// Every line that the server gave. The program makes `home_rows` from it
    /// each time a live message changes the shelf of Continue Listening, and a
    /// media that comes back on that shelf needs no request. See T-66.
    pub home_rows_of_the_server: Vec<HomeRow>,
    /// `true` for a media of the Home view that stands on the shelf of
    /// Continue Listening. The number is the number of a `HomeRow::Media`.
    /// See T-66.
    pub of_continue_listening: Vec<bool>,
    /// The lines that left the shelf of Continue Listening in `home_rows`. The
    /// value is the number of a `HomeRow::Media`, and **not** the identity of a
    /// media: one media stands on two shelves, and one line of the two goes
    /// away. The program makes the lines again when this list differs from the
    /// list of the live messages. See T-66.
    pub the_media_that_left: std::collections::BTreeSet<usize>,
    /// The user asked for the row of the keys of the player with the key `B`.
    ///
    /// **The render reads no disk** (T-204): `render_player` read this value of
    /// the database at each frame, and a second program of the account that
    /// held the write lock then took the thread of the screen for five seconds
    /// of each of those frames. The key `B` writes this value and the disk
    /// together, and a write that failed keeps the value that the program has.
    pub the_key_bindings_stand: bool,
    /// The speed of the playback of the account, of the disk.
    ///
    /// The row of the player takes it when the engine holds no speed of its
    /// own. **The render reads no disk** (T-204).
    pub the_speed_of_the_account: f32,
    /// The field of the sequence of the library, for the server. An empty
    /// text asks the server for its own sequence. See T-24.
    pub library_sort: String,
    /// The direction of the sequence. `true` puts the largest first.
    pub library_desc: bool,
    /// The filter of the library, of the form `<type>.<base64>`. An empty
    /// text asks for every item.
    pub library_filter: String,
    /// The list of the view of the sequence and of the filter.
    pub list_state_sort_filter: ListState,
    /// The list of the chapters of the media that plays. See T-24.
    pub list_state_chapters: ListState,
    /// The playback and the title of the media whose chapters the view shows.
    ///
    /// **The media that plays changes while that view stands open, and no key
    /// of the user does it**: the media comes to its end, and the queue starts
    /// the media of its front. The program holds the media of the view here,
    /// therefore the line goes to nobody when that media goes away, and the key
    /// `l` seeks in no media that the user did not choose. See T-162.
    pub the_media_of_the_view_of_the_chapters: Option<(u64, String)>,
    /// The list of the bookmarks of one media. See T-24.
    pub list_state_bookmarks: ListState,
    /// The list of the media that wait in the queue. See T-24.
    pub list_state_queue: ListState,
    /// The line of the view of the queue, and the identity and the title of the
    /// media of that line.
    ///
    /// **The queue changes while that view stands open, and no key of the user
    /// does it**: a media that comes to its end takes the media of the front
    /// away, and a second program of the account takes a media out. The program
    /// holds the media of the line of the user here, therefore the cursor goes
    /// with that media, and it goes to nobody when that media leaves the queue.
    /// See T-161.
    pub the_media_of_the_line_of_the_queue: Option<(usize, String, String)>,
    /// The media whose bookmarks the view shows. See T-24.
    pub bookmarks_of: String,
    /// The name of the media whose bookmarks the view shows.
    ///
    /// **The media that plays changes while that view stands open, and no key
    /// of the user does it**: the queue starts the media of its front. The
    /// title of the view names this media, and the key `b` writes a place of
    /// this media alone. See T-163.
    pub bookmarks_of_name: String,
    /// The timer for sleep, if the user set one. See T-24.
    pub sleep: Option<crate::logic::sleep_timer::Timer>,
    /// The choice of the timer, in minutes. `Some(0)` is the end of the
    /// chapter, and `None` is off.
    pub sleep_choice: Option<u64>,
    /// The list of the podcasts that the server found. See T-24.
    pub list_state_new_podcast: ListState,
    /// The list of the authors of the library. See T-24.
    pub list_state_authors: ListState,
    /// The line of the list of the ebooks of one media. See T-76.
    pub list_state_ebooks: ListState,
    /// The line of the values of the block `[reader]`. See T-77.
    pub list_state_settings_reader: ListState,
    /// The view that the search came from. The key `h` gives it back. See T-79.
    pub the_view_before_the_search: AppView,
    /// The view that the key `m` came from. See T-84.
    pub the_view_before_the_list: AppView,
    /// The line of the queue of the downloads of the server. See T-81.
    pub list_state_downloads: ListState,
    /// The episode of the line of the view of the downloads: its number of a
    /// line, its name of `OneDownload::key`, its title, and its podcast.
    ///
    /// **That queue changes while its view stands open, and no key of any user
    /// does it**: the server takes an episode out when it downloaded it, and a
    /// second program of the library empties that queue. The program holds the
    /// episode of the line of the user here, therefore the cursor goes with
    /// that episode, and it goes to nobody when that episode leaves the queue.
    /// See T-166.
    pub the_episode_of_the_line_of_the_downloads: Option<(usize, String, String, String)>,
    /// The first list of the server gave a line to the view of the downloads.
    ///
    /// **The view opens before the answer of the server comes.** A line of the
    /// open therefore stands on nothing at all, and the first list that comes
    /// gives the line 0. A line that went to nobody after that list stays with
    /// nobody: the user chooses the next episode with the keys j and k. See
    /// T-166.
    pub the_downloads_gave_the_first_line: bool,
    /// The line of the lists that can take a media. See T-84.
    pub list_state_put_in_a_list: ListState,
    /// The media that the key `m` puts in a list: its identity, its episode,
    /// and its title. See T-84.
    pub the_media_of_the_list: Option<(String, Option<String>, String)>,
    /// The view that the key `@` came from. See T-119.
    pub the_view_before_the_send: AppView,
    /// The line of the devices of an e-reader. See T-119.
    pub list_state_send_to_ereader: ListState,
    /// The book that the key `@` sends: the identity of the item, and its
    /// title. **An episode of a podcast holds no ebook**, therefore this holds
    /// no episode. See T-119.
    pub the_book_of_the_send: Option<(String, String)>,
    /// The podcast whose queue the key `X` empties, after the question. See
    /// T-81.
    pub confirm_the_empty_queue: Option<String>,
    /// The identity of the list that the key `X` removes at a second press.
    /// **Every user of the server sees a collection**, therefore the program
    /// asks one time. See T-93.
    pub confirm_the_removal_of_the_list: Option<String>,
    /// The line of the view of every key. See T-49.
    pub list_state_keys: ListState,
    /// The view that the user came from, before the list of every key. The key
    /// `?` a second time gives that view back. See T-49.
    pub the_view_before_the_keys: AppView,
    /// The view that the user came from, before the reader of an ebook. The key
    /// `h` gives that view back. The old code always gave the Library view, and
    /// a user of the Home view then lost their line. See T-52.
    pub the_view_before_the_reader: AppView,
    /// The user changed the sequence or the filter. The loop of the program
    /// then makes the application again, in the same way as the key `R`. A
    /// new sequence needs a new request, and every list of the library comes
    /// from that request. See T-24.
    pub must_refresh: bool,
    /// A box of the program wrote on the cells of the view, and the next draw
    /// must write every cell again. See T-89.
    ///
    /// **ratatui writes the cells that changed only**, and it compares with the
    /// buffer that it holds itself. The box of `ask_for_a_text` makes a terminal
    /// of its own, therefore the terminal of the program knows nothing of the
    /// letters that the box wrote over. Those letters stayed on the screen until
    /// a key made the program write the same rows again.
    pub the_screen_must_be_drawn_again: bool,
    /// The view that opened the books of a series. The key `h` then goes
    /// back to that view, and not to the list of the series. The Home view
    /// can open a series too, therefore this is a view and not a yes or a no.
    /// See T-22 and T-24.
    pub series_from: AppView,
    /// The collections and the playlists of the library. See T-9.
    pub lists: Vec<ListView>,
    /// The server did not answer at the start. The application then shows the
    /// media of the disk only. See T-25.
    pub is_offline: bool,
    /// The program did not read the media of the disk of this account in its
    /// database. The offline mode then holds no line, and the view names the disk
    /// and not the server. See T-203.
    pub the_media_of_the_disk_did_not_come: bool,
    /// The number of positions that wait for the server. See T-25.
    ///
    /// `None` is a read of the disk that failed: the header then names no number
    /// at all, because a count of 0 says that every place reached the server
    /// (T-203).
    pub waiting_progress: Option<usize>,
    /// The identity of the server of this account. A user can have an account
    /// on more than one server, and a position must go to the correct server.
    pub server_key: String,
    pub search_query: String,
    pub search_mode: bool,
    pub is_podcast: bool,
    pub all_titles_pod_ep: Vec<Vec<String>>,
    pub all_ids_pod_ep: Vec<Vec<String>>,
    pub all_subtitles_pod_ep: Vec<Vec<String>>,
    pub all_seasons_pod_ep: Vec<Vec<String>>,
    pub all_episodes_pod_ep: Vec<Vec<String>>,
    pub all_authors_pod_ep: Vec<Vec<String>>,
    pub all_descs_pod_ep: Vec<Vec<String>>,
    pub all_titles_pod: Vec<Vec<String>>,
    pub all_durations_pod_ep: Vec<Vec<String>>,
    /// `true` for each podcast whose episodes the program read. See T-126.
    ///
    /// A podcast of no episode and a podcast whose episodes the program did not
    /// read hold the same empty row, and the view says a different sentence for
    /// each of them.
    pub the_episodes_that_came: Vec<bool>,
    pub titles_pod_ep: Vec<String>,
    pub ids_pod_ep: Vec<String>,
    pub ids_pod_ep_search: Vec<String>,
    pub subtitles_pod_ep: Vec<String>,
    pub seasons_pod_ep: Vec<String>,
    pub episodes_pod_ep: Vec<String>,
    pub authors_pod_ep: Vec<String>,
    pub descs_pod_ep: Vec<String>,
    pub titles_pod: Vec<String>,
    pub durations_pod_ep: Vec<String>,
    pub ids_ep_cnt_list: Vec<String>,
    pub all_titles_pod_ep_search: Vec<Vec<String>>,
    pub titles_pod_ep_search: Vec<String>,
    pub is_from_search_pod: bool,
    pub ids_library_pod_search: Vec<String>,
    pub all_ids_pod_ep_search: Vec<Vec<String>>,
    pub libraries_names: Vec<String>,
    pub media_types: Vec<String>,
    pub libraries_ids: Vec<String>,
    pub library_name: String,
    pub media_type: String,
    pub lib_name_type: String,
    pub settings: Vec<String>,
    /// Every account of the database: the name, the address of the server, and
    /// "this account starts the program". See T-124.
    pub the_accounts: Vec<(String, String, bool)>,
    pub username: String,
    pub server_address: String,
    pub server_address_pretty: String,
    pub scroll_offset: u16,
    pub subtitles_pod_cnt_list: Vec<String>,
    pub nums_ep_pod_cnt_list: Vec<String>,
    pub seasons_pod_cnt_list: Vec<String>,
    pub authors_pod_cnt_list: Vec<String>,
    pub descs_pod_cnt_list: Vec<String>,
    pub titles_pod_cnt_list: Vec<String>,
    pub durations_pod_cnt_list: Vec<String>,
    pub published_year_library: Vec<String>,
    pub desc_library: Vec<String>,
    pub duration_library: Vec<f64>,
    pub auth_names_library_pod: Vec<String>,
    pub subtitles_pod_ep_search: Vec<String>,
    pub seasons_pod_ep_search: Vec<String>,
    pub episodes_pod_ep_search: Vec<String>,
    pub authors_pod_ep_search: Vec<String>,
    pub descs_pod_ep_search: Vec<String>,
    pub titles_pod_search: Vec<String>,
    pub durations_pod_ep_search: Vec<String>,
    pub all_subtitles_pod_ep_search: Vec<Vec<String>>,
    pub all_seasons_pod_ep_search: Vec<Vec<String>>,
    pub all_episodes_pod_ep_search: Vec<Vec<String>>,
    pub all_authors_pod_ep_search: Vec<Vec<String>>,
    pub all_descs_pod_ep_search: Vec<Vec<String>>,
    pub all_titles_pod_search: Vec<Vec<String>>,
    pub all_durations_pod_ep_search: Vec<Vec<String>>,
    pub auth_names_pod_search_book: Vec<String>,
    /// The title of each line of the view of the search.
    ///
    /// The reader of a book needs the title of the media (T-54), and the view of
    /// the search held no list of the titles: the reader of a PDF that the user
    /// opened there said the identity of the item. See T-117.
    pub titles_search_book: Vec<String>,
    pub auth_names_search_book: Vec<String>,
    pub published_year_library_search_book: Vec<String>,
    pub desc_library_search_book: Vec<String>,
    pub duration_library_search_book: Vec<f64>,
    pub book_progress_cnt_list: Vec<Vec<String>>,
    pub book_progress_cnt_list_cur_time: Vec<Vec<f64>>,
    //    pub book_progress_library: Vec<Vec<String>>,
    //    pub book_progress_library_cur_time: Vec<Vec<f64>>,
    pub book_progress_search_book: Vec<Vec<String>>,
    pub book_progress_search_book_cur_time: Vec<Vec<f64>>,
    /// The audio engine. The application starts it one time.
    pub player: PlayerHandle,
    /// The reason why the audio engine did not start. The program still shows
    /// the library, and it tells the user that no media can play. See T-46.
    pub audio_fault: Option<String>,
    pub config: ConfigFile,
    pub changelog: String,
    pub update_msg: String,
    /// The pictures of the covers that the render holds. A refresh with the
    /// key `R` makes a new `App` and thus an empty map. The bytes stay in the
    /// store of the process, therefore no request goes to the server a second
    /// time. See T-23.
    pub covers: crate::ui::cover::CoverArt,
    /// The pictures of the pages of a PDF book that the render holds. The key is
    /// the item, the page, and the name of the picture. See T-54.
    pub pictures_of_the_reader: std::collections::HashMap<String, Option<PictureOfThePage>>,
    /// What this account may do on the server. An absent answer gives every
    /// permission. See T-24.
    /// The account of the user: the type and the permissions. The settings
    /// show them, and the key `D` reads the permission of the download. See
    /// T-110.
    pub account: crate::api::me::permissions::TheAccount,
    /// The account that waits for the second press of the key `l`. A log out
    /// forgets a token, therefore the program asks one time. See T-36.
    pub confirm_logout: Option<String>,
    /// The account that waits for the second press of the key `c`. The program
    /// starts again for that key, and a playback stops with the process.
    /// See T-124.
    pub confirm_the_account_that_starts: Option<String>,
    /// The book that the user reads now. See T-10.
    pub reader: Option<crate::logic::reader::Reader>,
    /// A message of the reader for the user, for example the reason why a
    /// book did not open.
    pub reader_message: Option<String>,
    /// The first line of the view of the statistics. The keys `j` and `k`
    /// change it. See T-24.
    pub stats_scroll: u16,
    /// The first line of the view of the sessions.
    pub sessions_scroll: u16,
    /// The largest first line of the view of the sessions.
    pub sessions_scroll_max: u16,
    /// The largest first line of the view of the statistics. The render
    /// writes it, because the render knows the height of the screen. The move
    /// then stops when the last line is visible.
    pub stats_scroll_max: u16,
    /// The program must start again, and the loop must first send the position
    /// of a playback that it stops. See T-139.
    ///
    /// A key of the view of the accounts starts the program again with `exec`,
    /// and **a key handler cannot wait for the server**: the handler is not
    /// asynchronous, and `exec` takes every task of this process away. Therefore
    /// the handler writes the request here, and the loop of `src/main.rs` closes
    /// the session, sends the position, and then starts the program again.
    pub the_program_starts_again: Option<TheProgramStartsAgain>,
}

/// The request of a key that starts the program again. See T-139.
///
/// `variables` holds the variables of the environment that the new program
/// needs, and `message` is the text for a system that has no `exec`.
#[derive(Debug, Clone)]
pub struct TheProgramStartsAgain {
    pub variables: Vec<(String, String)>,
    pub message: String,
}

/// The state of the user that a refresh of the screen must keep. See T-135.
///
/// **A refresh makes a new application**, and every field of the new one starts
/// at its first value. T-131 gave the engine of the playback to that new
/// application, and the timer for sleep stayed behind: the timer of the user
/// went away with no word, and the media that they set to stop played on.
///
/// The engine belongs to the playback, therefore the identity of the playback
/// does not change with a refresh and the timer of the old application measures
/// the same media.
#[derive(Debug, Clone, PartialEq)]
pub struct TheStateThatARefreshKeeps {
    /// The timer for sleep, if the user set one.
    pub sleep: Option<crate::logic::sleep_timer::Timer>,
    /// The choice of the timer, in minutes.
    pub sleep_choice: Option<u64>,
}

/// Init app
impl App {
    /// Gives the state of the user that a refresh must keep. See T-135.
    pub fn the_state_that_a_refresh_keeps(&self) -> TheStateThatARefreshKeeps {
        TheStateThatARefreshKeeps {
            sleep: self.sleep,
            sleep_choice: self.sleep_choice,
        }
    }

    /// Takes the state of the user of the application before this one. See
    /// T-135.
    pub fn keep_the_state_of_the_application_before(
        &mut self,
        of_the_old: TheStateThatARefreshKeeps,
    ) {
        self.sleep = of_the_old.sleep;
        self.sleep_choice = of_the_old.sleep_choice;
    }

    /// Makes the application state.
    ///
    /// The caller gives the HTTP client. The client holds the addresses of the
    /// server and the decrypted token. **A new application starts a new engine
    /// of the sound**, therefore a refresh of the screen takes
    /// `new_with_the_engine`. See T-131.
    pub async fn new(api: std::sync::Arc<crate::api::client::ApiClient>) -> Result<Self> {
        Self::new_with_the_engine(api, None).await
    }

    /// Makes the application state, and it keeps the engine that plays.
    ///
    /// **The key `R` and every key that refreshes the screen make a new
    /// application** (the sequence of the library, and the key that takes the
    /// next library of T-66). `App::new` started a new engine of the sound for
    /// each of them: the old engine kept the playback and no key of the user
    /// reached it, therefore the row of the player went away while the media
    /// played, and the key `Space` stopped nothing. A measurement of 2026-08-12
    /// pressed `R` at the minute 2 of a book of 30 minutes, and the book played
    /// to its end with no row on the screen. See T-131.
    ///
    /// `engine` holds the handle of the engine and the fault of the sound
    /// device, when a program that runs already gives them.
    pub async fn new_with_the_engine(
        api: std::sync::Arc<crate::api::client::ApiClient>,
        engine: Option<(PlayerHandle, Option<String>)>,
    ) -> Result<Self> {
        // A new application reads the first page of the library again. A page
        // of the library before it belongs to a different filter, a different
        // library, or a different server. See T-70.
        crate::logic::library_pages::forget();

        // init config
        let config = load_config()?;

        // **The limit of the cache of the ebooks stands in a slot of a module**,
        // because the task that removes a book holds no `App` (T-72). The start
        // of the program writes that slot, and **a new application must write it
        // again**: the key `R` reads `config.toml` again, and a second program of
        // this account or an editor of the user can hold a different value in
        // that file. See T-142.
        crate::logic::reader::cache::keep_the_limit_of_the_configuration(
            config.reader.ebook_cache_mb,
        );

        // init database from Database struct
        crate::utils::startup::set("the database");
        let database = Database::new().await?;

        // init changelog
        let changelog = changelog();

        // retrieve crypted token from database
        let mut token: String = String::new();
        if let Some(var_token) = database.default_usr.get(2) {
            token = var_token.clone();
        }
        match decrypt_token(token.as_str()) {
            Ok(decrypted_token) => {
                token = decrypted_token;
                //info!("Token successfully decrypted")
            }
            Err(e) => {
                // **A view of ratatui writes no line to the terminal.** A
                // `println!` here stood on the cells of the frame, and the
                // program drew no cell of that row again. The first request
                // with the empty token gives the login screen, and that screen
                // says what the user must do. See T-133.
                log::error!("[app] the token of the database has no plain form: {}", e);
            }
        }

        // init server_address
        let mut _server_address: String = String::new();
        if let Some(var_server_address) = database.default_usr.get(1) {
            _server_address = var_server_address.clone();
        }

        // init id_selected_lib
        let mut id_selected_lib: String = String::new();
        if let Some(var_id_selected_lib) = database.default_usr.get(5) {
            id_selected_lib = var_id_selected_lib.clone();
        }

        // init current username
        let mut username: String = String::new();
        if let Some(var_username) = database.default_usr.first() {
            username = var_username.clone();
        }

        // init server address (without prefix)
        let mut server_address: String = String::new();
        let mut server_address_pretty: String = String::new();
        if let Some(var_server_address) = database.default_usr.get(1) {
            server_address = var_server_address.clone();

            // Remove "http://" or "https://"
            if let Some(stripped) = server_address.strip_prefix("http://") {
                server_address_pretty = stripped.to_string();
            } else if let Some(stripped) = server_address.strip_prefix("https://") {
                server_address_pretty = stripped.to_string();
            }
        }

        // The sequence and the filter of the library. The choice belongs to
        // the account, therefore it stays after the program stops. A field
        // that this build does not know goes away: the server takes a name of
        // a field that does not exist, and it then gives an unspecified
        // sequence. See T-24.
        //
        // **A read of that row that failed is not an account that chose
        // nothing** (T-209). The three reads below stand on the row of `users`
        // of this account — the row of the accounts of T-199 — therefore a fault
        // of one of them is the fault of the accounts: the start stops with the
        // words that name the database, and the key `R` keeps the application of
        // the user (T-205).
        let (mut library_sort, library_desc, library_filter) =
            crate::db::crud::get_library_sort(&username)
                .map_err(|error| crate::db::TheAccountsDidNotCome(error.to_string()))?;

        // **The render reads no disk** (T-204). The two values of the row of the
        // player come of the disk here, at the start and at every refresh with
        // the key `R`, and the render then draws with no call of the database.
        let the_key_bindings_stand = crate::db::crud::get_is_show_key_bindings(&username)
            .map_err(|error| crate::db::TheAccountsDidNotCome(error.to_string()))?;
        let the_speed_of_the_account = crate::db::crud::get_speed_rate(&username)
            .map_err(|error| crate::db::TheAccountsDidNotCome(error.to_string()))?;

        // **A copy of the disk of a media is a label of six views**, and that
        // label held a read of the database of each frame (T-203). The box of
        // the process holds it now, and this call fills it for the account of
        // this program. See T-204.
        crate::logic::the_copies_of_the_disk::read_the_disk(&username);

        // A user can have an account on more than one server. The identity of the
        // server keeps the positions of one server separate from the positions of
        // a different server.
        let server_key = crate::config::server_key(&config.servers, &server_address);

        // init for `Libraries` (get all Libraries (shelf), can be a podcast or book type)
        //
        // This is the first request. A server that does not answer starts the
        // offline mode: the application then makes its lists from the media on
        // the disk, and it sends no other request. See T-25.
        crate::utils::startup::set("the libraries of the server");
        let (all_libraries, is_offline) = match get_all_libraries(&api).await {
            Ok(value) => (value, false),
            Err(error) if error.is_offline() => {
                log::warn!(
                    "[app] the server does not answer: {}. The offline mode starts.",
                    error
                );
                (
                    crate::api::libraries::get_all_libraries::Root::default(),
                    true,
                )
            }
            Err(error) => return Err(error.into()),
        };
        // The server answers. Therefore the application sends every position that
        // waited during the offline mode, before it asks for anything more. A
        // request that fails later must not stop this. See T-25.
        if !is_offline {
            crate::logic::offline::flush_pending_progress(&api, &username, &server_key).await;
        }

        let libraries_names = collect_library_names(&all_libraries).await; // all the libraries names of the user ex : {name1, name2}
        let media_types = collect_media_types(&all_libraries).await; // all media type of libraries ex : {book, podcast}
        let libraries_ids = collect_library_ids(&all_libraries).await; // all all libraries ids
        let mut library_name = String::new(); // library name of the selected library
        let mut media_type = String::new(); // media type of the selected library

        // **An account may lose a library while the program of that account
        // holds it.** The server answers 403 for every request of that library,
        // and the program then showed a view of no line with the words "This
        // library holds no media": the header said "📖  ()", the key Shift+Tab
        // said "This server holds one library" and it moved to nothing, and no
        // key gave the user the library that they may read. A start after it
        // gave the same screen, therefore the account was locked out of the
        // program for ever. See T-136.
        if let Some(place) = crate::logic::library_pages::the_library_that_the_program_must_take(
            &libraries_ids,
            &id_selected_lib,
        ) {
            let of_the_account = libraries_ids[place].clone();
            let name = libraries_names.get(place).cloned().unwrap_or_default();

            log::warn!(
                "[app] the library {} is not a library of the account {}. The program takes {} ({}).",
                id_selected_lib,
                username,
                name,
                of_the_account
            );

            // This write stands behind no key of the user, therefore its fault
            // takes a line of the log and no word for the user (T-177 and
            // T-205). The program shows the library of the account with no row
            // of the disk, and the next start does this work again.
            if let Err(error) = update_id_selected_lib(&of_the_account, &username) {
                log::error!(
                    "[app] the program did not write the library {} of {}: {}. \
                     The next start reads the library of the row again.",
                    of_the_account,
                    username,
                    error
                );
            }

            id_selected_lib = of_the_account;

            crate::logic::message::say(&format!(
                "Your account cannot read the library of this program. It shows \"{}\" now.",
                name
            ));
        }

        let target = id_selected_lib.clone();

        // retrieve name and mediatype of the current librarie
        if let Some(index) = libraries_ids.iter().position(|x| x == &target) {
            library_name = libraries_names[index].clone();
            media_type = media_types[index].clone();
        }
        // The offline mode shows the media of the disk, and not a library of the
        // server. The header must say that.
        let lib_name_type = if is_offline {
            "📴 Offline: the media on the disk".to_string()
        } else {
            format!("📖 {} ({})", library_name, media_type)
        };

        // init is_podcast
        let is_podcast = media_type == "podcast";

        // A field of a library of books has no meaning in a library of
        // podcasts. The program then asks the server with no sequence.
        if !crate::logic::sort_filter::is_a_field_of_the_program(&library_sort, is_podcast) {
            library_sort = String::new();
        }

        // **The server groups the books of a series**, and the program showed one
        // line for such a group already (T-22). A measurement of 2026-08-12
        // against the sandbox compared the two answers: 14 items with no
        // parameter, and **10 items with the parameter** — the same 10 lines
        // that `group_library` makes, in the same sequence, and with the same
        // series of one book.
        //
        // The parameter therefore takes work away and it changes no screen:
        // one page of the answer holds 500 **lines** now, and `total` counts the
        // lines that the user reads. The title of the view of T-70 is then
        // exact.
        //
        // `group_library` stays: it gives the line of a series the place of that
        // series in `App::series`, and the view reads the books, the
        // description, and the cover there.
        //
        // A library of podcasts holds no series, therefore it takes no
        // parameter.
        let library_query = format!(
            "{}{}",
            crate::logic::sort_filter::query(&library_sort, library_desc, &library_filter),
            if is_podcast { "" } else { "&collapseseries=1" }
        );

        // init for `Home` (continue listening)
        let mut _titles_cnt_list: Vec<String> = Vec::new();
        let mut auth_names_cnt_list: Vec<String> = Vec::new();
        let mut pub_year_cnt_list: Vec<String> = Vec::new();
        let mut duration_cnt_list: Vec<f64> = Vec::new();
        let mut desc_cnt_list: Vec<String> = Vec::new();
        let mut _ids_cnt_list: Vec<String> = Vec::new();
        let mut ids_ep_cnt_list: Vec<String> = Vec::new();
        let mut subtitles_pod_cnt_list: Vec<String> = Vec::new();
        let mut nums_ep_pod_cnt_list: Vec<String> = Vec::new();
        let mut seasons_pod_cnt_list: Vec<String> = Vec::new();
        let mut authors_pod_cnt_list: Vec<String> = Vec::new();
        let mut descs_pod_cnt_list: Vec<String> = Vec::new();
        let mut titles_pod_cnt_list: Vec<String> = Vec::new();
        let mut durations_pod_cnt_list: Vec<String> = Vec::new();
        let mut book_progress_cnt_list: Vec<Vec<String>> = Vec::new();
        let mut book_progress_cnt_list_cur_time: Vec<Vec<f64>> = Vec::new();

        // **The account of the token, beside the shelves.** The answer holds
        // the permissions (T-110) and the position of every media of the
        // account (T-127): the start asked one request for each media of the
        // Home view before, and 29 media of a server of 500 milliseconds cost
        // 2.1 seconds of a start of 3.8. The task runs while the program asks
        // for the shelves.
        // The account of the token, for every branch below. The books read the
        // positions of that answer, and the other branches take the
        // permissions only.
        let mut the_account = crate::api::me::permissions::TheAccount::default();

        let mut the_account_of_the_token = Some({
            let api = std::sync::Arc::clone(&api);

            tokio::spawn(async move {
                if is_offline {
                    return (
                        crate::api::me::permissions::TheAccount::default(),
                        Vec::new(),
                        false,
                    );
                }

                match crate::api::me::permissions::the_account_of_the_token(&api).await {
                    Ok((account, positions)) => (account, positions, true),
                    Err(error) => {
                        log::warn!("[app] the server did not give the account: {}", error);
                        (
                            crate::api::me::permissions::TheAccount::default(),
                            Vec::new(),
                            false,
                        )
                    }
                }
            })
        });

        // **The series, the collections, the playlists, and the items need the
        // shelves of the Home view for nothing**, therefore they do not wait for
        // that answer. The four go together already (T-40), and a measurement of
        // 2026-08-12 with a proxy of 500 milliseconds read three rounds of
        // requests in the log of that proxy: the libraries, then the shelves and
        // the account, then the four. **The four stand in the second round now**,
        // and the first frame of that server takes 1.5 seconds of 2.0. See T-129.
        crate::utils::startup::set("the shelves, the series, the lists, and every item");

        // **A new start of this library takes the faults of the start before it
        // away.** The key `R`, the key `S`, and a new sequence of the library
        // all make this application again, and a request that answers now must
        // take no sentence of the request before it. See T-170, and the same
        // rule of T-168 and of T-169.
        crate::logic::the_requests_of_the_start::forget_the_faults_of(&id_selected_lib);

        let the_four_requests = {
            let api = std::sync::Arc::clone(&api);
            let id_selected_lib = id_selected_lib.clone();
            let library_query = library_query.clone();

            tokio::spawn(async move {
                // **A new request of this library takes the fault of the
                // request before it away.** The key `S` gives the program a new
                // library, and it comes back to the library of a fault later.
                // See T-169, and the same rule of T-168.
                crate::logic::the_lists::forget_the_fault_of(&id_selected_lib);

                let ask_for_the_series = async {
                    // A podcast library has no series, thus the application sends no
                    // request for it. See T-22.
                    if is_podcast || is_offline {
                        return Vec::new();
                    }

                    match get_all_series(&api, &id_selected_lib).await {
                        Ok(root) => collect_series(&root),
                        Err(error) => {
                            // A server that does not give the series must not stop the
                            // application. **The view of the series says why it
                            // holds no line** (T-91 and T-170): it said "This
                            // library has no series." for a library of series
                            // before that item.
                            log::warn!("[app] the server did not give the series: {}", error);
                            crate::logic::the_requests_of_the_start::keep_the_fault(
                                &id_selected_lib,
                                crate::logic::the_requests_of_the_start::TheRequest::Series,
                                &error.to_string(),
                            );
                            Vec::new()
                        }
                    }
                };

                let ask_for_the_collections = async {
                    // A podcast library has no collection, and it can have a playlist.
                    // See T-9.
                    if is_podcast || is_offline {
                        return CollectionRoot::default();
                    }

                    get_all_collections(&api, &id_selected_lib)
                        .await
                        .unwrap_or_else(|error| {
                            log::warn!("[app] the server did not give the collections: {}", error);
                            // **The view must say why it holds no line** (T-91).
                            // The server answers, therefore `is_offline` holds
                            // `false` and the words of the offline mode never
                            // come: the view said "This library has no
                            // collection and no playlist" for a library of one
                            // collection and of one playlist. See T-169.
                            crate::logic::the_lists::keep_the_fault(
                                &id_selected_lib,
                                &error.to_string(),
                            );
                            CollectionRoot::default()
                        })
                };

                let ask_for_the_playlists = async {
                    if is_offline {
                        return PlaylistRoot::default();
                    }

                    get_all_playlists(&api, &id_selected_lib)
                        .await
                        .unwrap_or_else(|error| {
                            log::warn!("[app] the server did not give the playlists: {}", error);
                            // See the collections above, and T-169.
                            crate::logic::the_lists::keep_the_fault(
                                &id_selected_lib,
                                &error.to_string(),
                            );
                            PlaylistRoot::default()
                        })
                };

                let ask_for_the_items = async {
                    // The offline mode makes this list from the media on the disk. A
                    // media that the disk does not hold cannot play, thus the list
                    // must not show it. See T-25.
                    if is_offline {
                        return crate::api::libraries::get_all_books::Root::default();
                    }

                    // **The first page, and not every page.** `get_all_books` read
                    // every page of the library before the first frame: a library of
                    // 2056 items made five requests, and a library of 250000 items made
                    // 500 of them. The program asks for the page after this one when
                    // the user comes near the end of the lines that it holds. See T-70.
                    crate::api::libraries::get_all_books::get_one_page_of_books(
                        &api,
                        &id_selected_lib,
                        &library_query,
                        0,
                    )
                    .await
                    .unwrap_or_else(|error| {
                        log::warn!("[app] the server did not give the items: {}", error);
                        // **The Library view says why it holds no line** (T-91
                        // and T-170): it said "This library holds no media.
                        // Press L to tell the server to examine the library."
                        // for a library of 17 books, and that key does no work
                        // of this fault (T-118).
                        crate::logic::the_requests_of_the_start::keep_the_fault(
                            &id_selected_lib,
                            crate::logic::the_requests_of_the_start::TheRequest::Items,
                            &error.to_string(),
                        );
                        crate::api::libraries::get_all_books::Root::default()
                    })
                };

                tokio::join!(
                    ask_for_the_series,
                    ask_for_the_collections,
                    ask_for_the_playlists,
                    ask_for_the_items,
                )
            })
        };

        // The shelves of the Home view. The lines of that view need the
        // shelves and the series together, therefore the program keeps the
        // answer here and it makes the lines when it holds the series. See
        // T-24.
        let mut shelves: Vec<crate::api::libraries::get_library_perso_view::Root> = Vec::new();
        let mut shelves_pod: Vec<crate::api::libraries::get_library_perso_view_pod::Root> =
            Vec::new();

        if is_offline {
            // The server gives no "continue listening" list. The view Library
            // holds the media of the disk, thus the Home view stays empty and the
            // application starts in the Library view.
        } else if is_podcast {
            // init for  `Home` (continue listening) for podcasts
            crate::utils::startup::set("the shelves of the Home view");
            // A server that gives an answer that the program cannot read must
            // not stop the program. The Home view is then empty, and every
            // other view works. See T-41.
            shelves_pod = get_the_shelves_pod(&api, &id_selected_lib)
                .await
                .unwrap_or_else(|error| {
                    log::warn!("[app] the server did not give the shelves: {}", error);
                    // **The Home view says why it holds no shelf** (T-91 and
                    // T-170).
                    crate::logic::the_requests_of_the_start::keep_the_fault(
                        &id_selected_lib,
                        crate::logic::the_requests_of_the_start::TheRequest::Shelves,
                        &error.to_string(),
                    );
                    Default::default()
                });
            _ids_cnt_list = collect_ids_pod_cnt_list(&shelves_pod).await; // id of a podcast
            _titles_cnt_list = collect_titles_cnt_list_pod(&shelves_pod).await; // title of podcast ep
            ids_ep_cnt_list = collect_ids_ep_pod_cnt_list(&shelves_pod).await; // id of a podcast episode
            subtitles_pod_cnt_list = collect_subtitles_pod_cnt_list(&shelves_pod).await;
            nums_ep_pod_cnt_list = collect_nums_ep_pod_cnt_list(&shelves_pod).await;
            seasons_pod_cnt_list = collect_seasons_pod_cnt_list(&shelves_pod).await;
            authors_pod_cnt_list = collect_authors_pod_cnt_list(&shelves_pod).await;
            descs_pod_cnt_list = collect_descs_pod_cnt_list(&shelves_pod).await;
            titles_pod_cnt_list = collect_titles_pod_cnt_list(&shelves_pod).await; // title of a podcast
            durations_pod_cnt_list = collect_durations_pod_cnt_list(&shelves_pod).await;
        } else {
            // init for  `Home` (continue listening) for books
            crate::utils::startup::set("the shelves of the Home view");
            // A server that gives an answer that the program cannot read must
            // not stop the program. See T-41.
            shelves = get_the_shelves(&api, &id_selected_lib)
                .await
                .unwrap_or_else(|error| {
                    log::warn!("[app] the server did not give the shelves: {}", error);
                    // **The Home view says why it holds no shelf** (T-91 and
                    // T-170).
                    crate::logic::the_requests_of_the_start::keep_the_fault(
                        &id_selected_lib,
                        crate::logic::the_requests_of_the_start::TheRequest::Shelves,
                        &error.to_string(),
                    );
                    Default::default()
                });
            _titles_cnt_list = collect_titles_cnt_list(&shelves).await;
            auth_names_cnt_list = collect_auth_names_cnt_list(&shelves).await;
            pub_year_cnt_list = collect_pub_year_cnt_list(&shelves).await;
            duration_cnt_list = collect_duration_cnt_list(&shelves).await;
            desc_cnt_list = collect_desc_cnt_list(&shelves).await;
            _ids_cnt_list = collect_ids_cnt_list(&shelves).await;
            // The position of each book needs its own request. The old code
            // sent them one after the other, therefore the start of the
            // program took the time of one request for each book of the list.
            // A server with a delay of 300 milliseconds and a list of ten
            // books then needed three seconds. The requests go together now,
            // eight at a time, and the answers keep the sequence of the list.
            // See T-40.
            const AT_THE_SAME_TIME: usize = 8;

            let count_of_the_list = _ids_cnt_list.len();
            let mut answers: Vec<Option<(Vec<String>, Vec<f64>)>> = vec![None; count_of_the_list];
            let mut done = 0;

            // **One request holds every position.** `GET /api/me` gives
            // `mediaProgress` for every media of the account, and the program
            // asks that endpoint for the permissions already: a media of that
            // answer needs no request of its own. See T-127.
            let (account_of_the_token, the_positions, the_answer_came) =
                match the_account_of_the_token.take() {
                    Some(task) => task.await.unwrap_or_else(|error| {
                        log::warn!("[app] the task of the account stopped: {}", error);
                        (
                            crate::api::me::permissions::TheAccount::default(),
                            Vec::new(),
                            false,
                        )
                    }),
                    None => (
                        crate::api::me::permissions::TheAccount::default(),
                        Vec::new(),
                        false,
                    ),
                };

            the_account = account_of_the_token;

            let mut the_media_that_need_a_request: Vec<(usize, String)> = Vec::new();

            for (place, id) in _ids_cnt_list.iter().enumerate() {
                match crate::logic::the_positions::the_position_of_a_media(&the_positions, id) {
                    Some(row) => {
                        answers[place] = Some((
                            vec![
                                collect_progress_percentage_book(row).await,
                                collect_is_finished_book(row).await,
                            ],
                            vec![collect_current_time_prg(row).await],
                        ));
                        done += 1;
                    }
                    // **The answer of the account holds every media that this
                    // account played**, therefore a book of no row played
                    // never: `GET /api/me/progress/:id` answers 404 for it, and
                    // the line says "N/A" either way. The program asks for such
                    // a book only when that answer did not come. See T-127.
                    None if the_answer_came => done += 1,
                    None => the_media_that_need_a_request.push((place, id.clone())),
                }
            }

            log::info!(
                "[app] the answer of the account holds the position of {} media of {}. \
                 The program asks the server for {}.",
                the_positions.len(),
                count_of_the_list,
                the_media_that_need_a_request.len()
            );

            for group in the_media_that_need_a_request.chunks(AT_THE_SAME_TIME) {
                let mut tasks = tokio::task::JoinSet::new();

                for (place, id) in group.iter() {
                    let place = *place;
                    let api = std::sync::Arc::clone(&api);
                    let id = id.clone();

                    tasks.spawn(async move {
                        let answer = match get_book_progress(&api, &id).await {
                            Ok(value) => Some((
                                vec![
                                    collect_progress_percentage_book(&value).await,
                                    collect_is_finished_book(&value).await,
                                ],
                                vec![collect_current_time_prg(&value).await],
                            )),
                            // A book that never played has no progress. The
                            // server gives an error, and that is not a fault.
                            Err(_) => None,
                        };

                        (place, answer)
                    });
                }

                while let Some(finished) = tasks.join_next().await {
                    done += 1;
                    crate::utils::startup::set_part(
                        "the position of each book of that list",
                        done,
                        count_of_the_list,
                    );

                    if let Ok((place, answer)) = finished {
                        if let Some(slot) = answers.get_mut(place) {
                            *slot = answer;
                        }
                    }
                }
            }

            for answer in answers {
                match answer {
                    Some((values, values_f64)) => {
                        book_progress_cnt_list.push(values);
                        book_progress_cnt_list_cur_time.push(values_f64);
                    }
                    // A book that never played gives no progress. The lists
                    // must still hold one row for each book, because the
                    // screen reads them by the number of the row.
                    None => {
                        book_progress_cnt_list.push(vec![" N/A".to_string(), " N/A".to_string()]);
                        book_progress_cnt_list_cur_time.push(vec![0.0]);
                    }
                }
            }
        }

        // **The account of the token came with the shelves** (T-127). A branch
        // that did not read it takes it here, and that wait is the wait of a
        // request that ran already.
        if let Some(task) = the_account_of_the_token.take() {
            the_account = task
                .await
                .unwrap_or_else(|error| {
                    log::warn!("[app] the task of the account stopped: {}", error);
                    (
                        crate::api::me::permissions::TheAccount::default(),
                        Vec::new(),
                        false,
                    )
                })
                .0;
        }

        let account = the_account;

        // The answers of the four requests that started beside the shelves. A
        // task that stopped gives the empty answer of each of them, and every
        // view then says that the server gave nothing (T-91). See T-129.
        let (series, collections, playlists, all_books) =
            the_four_requests.await.unwrap_or_else(|error| {
                log::warn!(
                    "[app] the task of the series and of the lists stopped: {}",
                    error
                );
                (
                    Vec::new(),
                    CollectionRoot::default(),
                    PlaylistRoot::default(),
                    crate::api::libraries::get_all_books::Root::default(),
                )
            });

        let lists = collect_lists(&collections, &playlists);

        // **A read of the disk that failed is not an account with no download**
        // (T-203). The offline mode of T-25 holds the media of the disk alone: a
        // read that gave nothing therefore takes every line of every view away,
        // and the Library view said that the **server** gave no media. The line
        // of the log holds the fault, and the view names the disk.
        let (downloads, the_media_of_the_disk_did_not_come) = if is_offline {
            match get_all_downloads(&username, &server_key) {
                Ok(rows) => (rows, false),
                Err(error) => {
                    log::error!(
                        "[app] the program did not read the downloads of the disk: {}. \
                         The offline mode holds no media of this account.",
                        error
                    );

                    (Vec::new(), true)
                }
            }
        } else {
            (Vec::new(), false)
        };

        let titles_library = if is_offline {
            downloads.iter().map(|row| row.title.clone()).collect()
        } else {
            collect_titles_library(&all_books).await
        };

        let ids_library: Vec<String> = if is_offline {
            downloads.iter().map(|row| row.key.clone()).collect()
        } else {
            collect_ids_library(&all_books).await
        };

        // Every book of a series gives one line of the Library view. See T-22.
        let library_rows = group_library(&ids_library, &series);

        // The lines of the Home view: a line for the name of each shelf, and
        // a line for each media of that shelf. See T-24.
        let home_rows = if is_podcast {
            group_home_pod(&shelves_pod)
        } else {
            group_home(&shelves, &series)
        };

        // The shelf of each media of the Home view. A live message takes a
        // media away from the shelf of Continue Listening, and it must take
        // nothing away from the other shelves. See T-66.
        let of_continue_listening = if is_podcast {
            crate::logic::home_view::the_media_of_continue_listening_pod(&shelves_pod)
        } else {
            crate::logic::home_view::the_media_of_continue_listening(&shelves)
        };

        let auth_names_library = if is_offline {
            downloads.iter().map(|row| row.author.clone()).collect()
        } else {
            collect_auth_names_library(&all_books).await
        };

        let duration_library: Vec<f64> = if is_offline {
            downloads.iter().map(|row| row.duration).collect()
        } else {
            collect_duration_library(&all_books).await
        };

        let desc_library: Vec<String> = if is_offline {
            downloads
                .iter()
                .map(|_| "This media plays from the disk. The server does not answer.".to_string())
                .collect()
        } else {
            collect_desc_library(&all_books).await
        };

        let published_year_library = if is_offline {
            downloads.iter().map(|_| "N/A".to_string()).collect()
        } else {
            collect_published_year_library(&all_books).await
        };

        let auth_names_library_pod = if is_offline {
            downloads.iter().map(|row| row.author.clone()).collect()
        } else {
            collect_auth_names_library_pod(&all_books).await // for a podcast
        };
        //    let mut book_progress_library: Vec<Vec<String>> = Vec::new();
        //    let mut book_progress_library_cur_time: Vec<Vec<f64>> = Vec::new();
        //    if !is_podcast{
        //        for id in _ids_cnt_list.clone() {
        //            if let Ok(val) = get_book_progress(&token, &id, server_address.clone()).await {
        //                let mut values: Vec<String> = Vec::new();
        //                let mut values_f64: Vec<f64> = Vec::new();
        //                values.push(format!(" {}%,",collect_progress_percentage_book(&val).await));
        //                values.push(format!(" {}",collect_is_finished_book(&val).await));
        //                values_f64.push(collect_current_time_prg(&val).await);
        //                book_progress_library.push(values);
        //                book_progress_library_cur_time.push(values_f64);
        //
        //            } else {
        //                // if the book is not starded, `get book progress` is not fetched
        //                // so the empty values are handled here :
        //                // avoid an out of bound panick
        //                let mut values: Vec<String> = Vec::new();
        //                let mut values_f64: Vec<f64> = Vec::new();
        //                values.push(format!(" Not started yet"));
        //                values.push(format!(""));
        //                values_f64.push(0.0);
        //                book_progress_library.push(values);
        //                book_progress_library_cur_time.push(values_f64);
        //            }
        //        }
        //    }

        // init for `SearchBook`

        let ids_search_book: Vec<String> = Vec::new();
        let _auth_names_pod_search_book: Vec<String> = Vec::new();
        let _auth_names_search_book: Vec<String> = Vec::new();
        let _published_year_library_search_book: Vec<String> = Vec::new();
        let _desc_library_search_book: Vec<String> = Vec::new();
        let auth_names_search_book: Vec<String> = Vec::new();
        let auth_names_pod_search_book: Vec<String> = Vec::new();
        let published_year_library_search_book: Vec<String> = Vec::new();
        let desc_library_search_book: Vec<String> = Vec::new();
        let duration_library_search_book: Vec<f64> = Vec::new();
        let book_progress_search_book: Vec<Vec<String>> = Vec::new();
        let book_progress_search_book_cur_time: Vec<Vec<f64>> = Vec::new();
        let search_mode = false;
        // No words yet. The view of the search never comes before the key
        // `/`, and that key writes the words of the user here. See T-24.
        let search_query = String::new();
        let all_titles_pod_ep_search: Vec<Vec<String>> = Vec::new(); // init in tui.rs in render search book function
        let all_ids_pod_ep_search: Vec<Vec<String>> = Vec::new();
        let all_subtitles_pod_ep_search: Vec<Vec<String>> = Vec::new();
        let all_seasons_pod_ep_search: Vec<Vec<String>> = Vec::new();
        let all_episodes_pod_ep_search: Vec<Vec<String>> = Vec::new();
        let all_authors_pod_ep_search: Vec<Vec<String>> = Vec::new();
        let all_descs_pod_ep_search: Vec<Vec<String>> = Vec::new();
        let all_titles_pod_search: Vec<Vec<String>> = Vec::new();
        let all_durations_pod_ep_search: Vec<Vec<String>> = Vec::new();
        let titles_pod_ep_search: Vec<String> = Vec::new();
        let ids_library_pod_search: Vec<String> = Vec::new(); // library because we take index of library
        let subtitles_pod_ep_search: Vec<String> = Vec::new();
        let seasons_pod_ep_search: Vec<String> = Vec::new();
        let episodes_pod_ep_search: Vec<String> = Vec::new();
        let authors_pod_ep_search: Vec<String> = Vec::new();
        let descs_pod_ep_search: Vec<String> = Vec::new();
        let titles_pod_search: Vec<String> = Vec::new();
        let durations_pod_ep_search: Vec<String> = Vec::new();
        let is_from_search_pod = false;

        //init for `PodcastEpisode`
        let mut all_titles_pod_ep: Vec<Vec<String>> = Vec::new(); // fetch titles for all podcast episodes. Ex: {titles_pod1_ep1, title_pod1_ep2}, {titles_pod2_ep1, title_pod2_ep2}
        let mut all_ids_pod_ep: Vec<Vec<String>> = Vec::new();
        let mut all_subtitles_pod_ep: Vec<Vec<String>> = Vec::new();
        let mut all_seasons_pod_ep: Vec<Vec<String>> = Vec::new();
        let mut all_episodes_pod_ep: Vec<Vec<String>> = Vec::new();
        let mut all_authors_pod_ep: Vec<Vec<String>> = Vec::new();
        let mut all_descs_pod_ep: Vec<Vec<String>> = Vec::new();
        let mut all_titles_pod: Vec<Vec<String>> = Vec::new(); // fetch title of a podcast (not episode)
        let mut all_durations_pod_ep: Vec<Vec<String>> = Vec::new();
        let titles_pod_ep: Vec<String> = Vec::new(); // fetch episode titles for a podcast. {titles_pod1_ep1, title_pod1_ep2}
        let ids_pod_ep: Vec<String> = Vec::new();
        let ids_pod_ep_search: Vec<String> = Vec::new();
        let subtitles_pod_ep: Vec<String> = Vec::new();
        let seasons_pod_ep: Vec<String> = Vec::new();
        let episodes_pod_ep: Vec<String> = Vec::new();
        let authors_pod_ep: Vec<String> = Vec::new();
        let descs_pod_ep: Vec<String> = Vec::new();
        let titles_pod: Vec<String> = Vec::new();
        let durations_pod_ep: Vec<String> = Vec::new();

        // **The start makes no request for a podcast now.** It read the
        // episodes of every podcast of the page, one request after the other:
        // a library of 520 podcasts gave 500 requests, and the first frame took
        // 11.9 seconds with a server of 20 milliseconds. The program reads the
        // episodes of one podcast when the user opens it (T-126), and this rule
        // is the rule of T-70 for the pages of the library.
        //
        // The lists hold one row for each item of the library, and every row is
        // empty. A row that no request filled stands in
        // `the_episodes_that_came`.
        let mut the_episodes_that_came: Vec<bool> = Vec::new();

        if is_podcast {
            for _ in ids_library.iter() {
                all_titles_pod_ep.push(Vec::new());
                all_ids_pod_ep.push(Vec::new());
                all_subtitles_pod_ep.push(Vec::new());
                all_seasons_pod_ep.push(Vec::new());
                all_episodes_pod_ep.push(Vec::new());
                all_authors_pod_ep.push(Vec::new());
                all_descs_pod_ep.push(Vec::new());
                all_titles_pod.push(Vec::new());
                all_durations_pod_ep.push(Vec::new());
                the_episodes_that_came.push(false);
            }
        }
        // init for `Settings`
        // The names say what the entry does. The user of the report of
        // 2026-08-10 did not find the way to leave a server, because the
        // entry said "Account" only. See T-36.
        let settings = vec![
            "Accounts and log out".to_string(),
            "Library: choose the library".to_string(),
            "About and changelog".to_string(),
            "Update and uninstall".to_string(),
            // The user changed this value with an editor before T-77.
            "The reader: the cache of the ebooks".to_string(),
        ];

        // **The view of the accounts holds every account of the database.** It
        // held the account of the start alone before T-124, therefore a user
        // who had two accounts read one line. The database of an older program
        // gives the account of the start only, and that account then makes the
        // one line as it did before.
        let mut the_accounts = crate::db::crud::select_every_usr().unwrap_or_default();
        if the_accounts.is_empty() {
            if let Some(var_username) = database.default_usr.first() {
                the_accounts.push((
                    var_username.clone(),
                    database.default_usr.get(1).cloned().unwrap_or_default(),
                    true,
                ));
            }
        }

        // init variables for for scrolling into description section
        let scroll_offset = 0;

        // Default view_state at launch
        let mut view_state = AppView::Home; // By default, Home will be the first AppView launched when the app start
        if _ids_cnt_list.is_empty() {
            view_state = AppView::Library; // If `Home` is empty (no book or podcast to continue)
        }

        // Start the audio engine. The application decodes the audio itself,
        // thus the token stays in the memory of the process and `ps aux` does
        // not show it. See T-5.
        // A machine with no sound card must still show the library.
        //
        // The old code stopped the whole program when the engine did not
        // start. A user on a machine with no sound device, or with a
        // configuration of ALSA that does not work, could then not read their
        // library, not download a book, and not see their progress. The
        // program keeps every function that needs no sound now, and it tells
        // the user why no playback starts. See T-46.
        crate::utils::startup::set("the sound device");

        // The sound device also gets a limit of time.
        //
        // `PlayerHandle::start` opens the sound card. ALSA can wait for ever
        // there: a device that a different program holds, a server of sound
        // that does not answer, or a device of Bluetooth that sleeps. The old
        // code had no limit, and the program then drew nothing and never
        // stopped. The program waits five seconds now, and it goes on with no
        // sound. See T-46.
        const TIME_FOR_THE_SOUND_DEVICE: std::time::Duration = std::time::Duration::from_secs(5);

        // **The engine of a program that plays already stays.** A new engine
        // takes the sound device a second time, and the playback of the user
        // then belongs to no key of the program: the row of the player went
        // away, and the key `Space` stopped nothing. See T-131.
        let (player, audio_fault) = match engine {
            Some((player, audio_fault)) => (player, audio_fault),
            None => {
                let token_of_the_engine = token.clone();
                let start_of_the_engine =
                    tokio::task::spawn_blocking(move || PlayerHandle::start(token_of_the_engine));

                let outcome = match tokio::time::timeout(
                    TIME_FOR_THE_SOUND_DEVICE,
                    start_of_the_engine,
                )
                .await
                {
                    Ok(Ok(outcome)) => outcome,
                    Ok(Err(error)) => {
                        Err(format!("the thread of the sound device stopped: {}", error))
                    }
                    Err(_) => Err(format!(
                        "the sound device did not answer in {} seconds",
                        TIME_FOR_THE_SOUND_DEVICE.as_secs()
                    )),
                };

                match outcome {
                    Ok(player) => (player, None),
                    Err(error) => {
                        log::error!("[app] the audio engine did not start: {}", error);
                        let (player, receiver) = PlayerHandle::without_engine();

                        // Nothing reads the commands of a player with no engine.
                        // A thread takes them and drops them, so that a key of
                        // the playback does not fill the memory.
                        std::thread::spawn(move || while receiver.recv().is_ok() {});

                        (player, Some(error.to_string()))
                    }
                }
            }
        };

        // **A read of the disk that failed is not a count of 0** (T-203). The
        // header of the offline mode says "N positions wait", and a count of 0
        // takes those words away: the user then reads that every place of the
        // playback reached the server.
        let waiting_progress = match count_pending_progress(&username, &server_key) {
            Ok(count) => Some(count),
            Err(error) => {
                log::error!(
                    "[app] the program did not count the positions that wait: {}. \
                     The header of the screen names no number of them.",
                    error
                );

                None
            }
        };

        // Init for check_update
        //
        // The offline mode sends no request to GitHub. The check costs time when
        // no network is available.
        let update_msg = if is_offline {
            String::new()
        } else {
            match check_update().await {
                Some(msg) => msg,
                None => "".to_string(),
            }
        };

        // Init ListeState for `Home` list (the shelves)
        //
        // The first line of the Home view is the name of a shelf, and a name
        // is not a line of the user. Therefore the selection starts at the
        // first media. See T-24.
        let mut list_state_cnt_list = ListState::default();
        list_state_cnt_list.select(crate::logic::home_view::first_line(&home_rows));

        // Init ListState for the view of the sequence and of the filter. The
        // first line is the name of a group, therefore the selection starts
        // at the line after it. See T-24.
        let mut list_state_sort_filter = ListState::default();
        list_state_sort_filter.select(Some(1));

        // Init ListeState for `Library` list
        let mut list_state_library = ListState::default();
        list_state_library.select(Some(0));

        // Init ListeState for `SearchBook` list
        let mut list_state_search_results = ListState::default();
        list_state_search_results.select(Some(0));

        // Init ListState for `PodacastEpisode` list
        let mut list_state_pod_ep = ListState::default();
        list_state_pod_ep.select(Some(0));

        // Init ListState for the two lists of the series
        let mut list_state_series = ListState::default();
        list_state_series.select(Some(0));

        let mut list_state_series_book = ListState::default();
        list_state_series_book.select(Some(0));

        // Init ListState for the collections and the playlists
        let mut list_state_lists = ListState::default();
        list_state_lists.select(Some(0));

        let mut list_state_list_entries = ListState::default();
        list_state_list_entries.select(Some(0));

        // Init ListState for `Settings` list
        let mut list_state_settings = ListState::default();
        list_state_settings.select(Some(0));

        // Init ListState for `SettingsAccount` list
        let mut list_state_settings_account = ListState::default();
        list_state_settings_account.select(Some(0));

        // Init ListState for `SettingsLibrary` list
        let mut list_state_settings_library = ListState::default();
        list_state_settings_library.select(Some(0));

        // Init ListState for `SettingsAbout` list
        let mut list_state_settings_about = ListState::default();
        list_state_settings_about.select(Some(0));

        // Init ListState for `SettingsUpdateUninstall` list
        let mut list_state_settings_update_uninstall = ListState::default();
        list_state_settings_update_uninstall.select(Some(0));

        Ok(Self {
            api,
            database,
            id_selected_lib,
            token: Some(token),
            should_exit: false,
            list_state_cnt_list,
            list_state_library,
            list_state_search_results,
            list_state_pod_ep,
            list_state_series,
            list_state_series_book,
            list_state_lists,
            list_state_list_entries,
            list_state_settings,
            list_state_settings_account,
            list_state_settings_library,
            list_state_settings_about,
            list_state_settings_update_uninstall,
            _titles_cnt_list,
            auth_names_cnt_list,
            pub_year_cnt_list,
            duration_cnt_list,
            desc_cnt_list,
            _ids_cnt_list,
            view_state,
            titles_library,
            ids_library,
            auth_names_library,
            ids_search_book,
            titles_search_book: Vec::new(),
            series,
            library_rows,
            library_page: 0,
            library_total: all_books.total.unwrap_or_default().max(0) as usize,
            library_query,
            reads_every_page_of_the_library: false,
            reads_the_pages_for_the_search: false,
            home_rows_of_the_server: home_rows.clone(),
            home_rows,
            of_continue_listening,
            the_media_that_left: std::collections::BTreeSet::new(),
            the_key_bindings_stand,
            the_speed_of_the_account,
            library_sort,
            library_desc,
            library_filter,
            list_state_sort_filter,
            list_state_chapters: ListState::default(),
            the_media_of_the_view_of_the_chapters: None,
            list_state_bookmarks: ListState::default(),
            list_state_queue: ListState::default(),
            the_media_of_the_line_of_the_queue: None,
            bookmarks_of: String::new(),
            bookmarks_of_name: String::new(),
            sleep: None,
            sleep_choice: None,
            list_state_new_podcast: ListState::default(),
            list_state_authors: ListState::default(),
            list_state_ebooks: ListState::default(),
            list_state_settings_reader: ListState::default(),
            the_view_before_the_search: AppView::Library,
            the_view_before_the_list: AppView::Library,
            list_state_downloads: ListState::default(),
            the_episode_of_the_line_of_the_downloads: None,
            the_downloads_gave_the_first_line: false,
            list_state_put_in_a_list: ListState::default(),
            the_media_of_the_list: None,
            the_view_before_the_send: AppView::Library,
            list_state_send_to_ereader: ListState::default(),
            the_book_of_the_send: None,
            confirm_the_empty_queue: None,
            confirm_the_removal_of_the_list: None,
            list_state_keys: ListState::default(),
            the_view_before_the_keys: AppView::Home,
            the_view_before_the_reader: AppView::Home,
            must_refresh: false,
            the_screen_must_be_drawn_again: false,
            series_from: AppView::Series,
            lists,
            is_offline,
            the_media_of_the_disk_did_not_come,
            waiting_progress,
            server_key,
            search_mode,
            search_query,
            is_podcast,
            all_titles_pod_ep,
            all_ids_pod_ep,
            titles_pod_ep,
            ids_pod_ep,
            ids_pod_ep_search,
            ids_ep_cnt_list,
            all_titles_pod_ep_search,
            titles_pod_ep_search,
            is_from_search_pod,
            ids_library_pod_search,
            all_ids_pod_ep_search,
            libraries_names,
            libraries_ids,
            media_types,
            library_name,
            media_type,
            lib_name_type,
            settings,
            the_accounts,
            username,
            server_address,
            server_address_pretty,
            scroll_offset,
            subtitles_pod_cnt_list,
            nums_ep_pod_cnt_list,
            seasons_pod_cnt_list,
            authors_pod_cnt_list,
            descs_pod_cnt_list,
            titles_pod_cnt_list,
            durations_pod_cnt_list,
            published_year_library,
            desc_library,
            duration_library,
            auth_names_library_pod,
            all_subtitles_pod_ep,
            all_seasons_pod_ep,
            all_episodes_pod_ep,
            all_authors_pod_ep,
            all_descs_pod_ep,
            all_titles_pod,
            all_durations_pod_ep,
            the_episodes_that_came,
            subtitles_pod_ep,
            seasons_pod_ep,
            episodes_pod_ep,
            authors_pod_ep,
            descs_pod_ep,
            titles_pod,
            durations_pod_ep,
            subtitles_pod_ep_search,
            seasons_pod_ep_search,
            episodes_pod_ep_search,
            authors_pod_ep_search,
            descs_pod_ep_search,
            titles_pod_search,
            durations_pod_ep_search,
            all_subtitles_pod_ep_search,
            all_seasons_pod_ep_search,
            all_episodes_pod_ep_search,
            all_authors_pod_ep_search,
            all_descs_pod_ep_search,
            all_titles_pod_search,
            all_durations_pod_ep_search,
            auth_names_pod_search_book,
            auth_names_search_book,
            published_year_library_search_book,
            desc_library_search_book,
            duration_library_search_book,
            book_progress_cnt_list,
            book_progress_cnt_list_cur_time,
            //       book_progress_library,
            //       book_progress_library_cur_time,
            book_progress_search_book,
            book_progress_search_book_cur_time,
            player,
            config,
            changelog,
            update_msg,
            covers: crate::ui::cover::CoverArt::new(),
            pictures_of_the_reader: std::collections::HashMap::new(),
            account,
            confirm_logout: None,
            confirm_the_account_that_starts: None,
            reader: None,
            reader_message: None,
            stats_scroll: 0,
            sessions_scroll: 0,
            sessions_scroll_max: 0,
            stats_scroll_max: 0,
            the_program_starts_again: None,
            audio_fault,
        })
    }

    // handle key
    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        // A key that is not `l` stops the question of the log out. The user
        // then never logs out because a key came at a wrong moment. See T-36.
        if !matches!(
            key.code,
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter
        ) {
            self.confirm_logout = None;
        }

        // The same rule for the queue of the downloads of the server (T-81),
        // and for the list that the key `X` removes (T-93).
        if !matches!(key.code, KeyCode::Char('X')) {
            self.confirm_the_empty_queue = None;
            self.confirm_the_removal_of_the_list = None;
        }

        // The same rule for the account that starts the program: that key
        // starts the program again, and a playback stops with the process.
        // See T-124.
        if !matches!(key.code, KeyCode::Char('c')) {
            self.confirm_the_account_that_starts = None;
        }

        match key.code {
            // The keys of the reader of an ebook come first. The reader uses
            // the same letters as the lists and as the player, and it uses
            // them for a different work. See T-10.
            // The key `Q` stops the program in every view. The reader took
            // every key before this rule, therefore a user of the reader could
            // not stop the program at all. See T-52.
            code if matches!(self.view_state, AppView::Reader)
                && !matches!(code, KeyCode::Char('Q')) =>
            {
                self.handle_key_of_the_reader(code);
            }

            // The keys of the view of the accounts. They stand before the key
            // `a` of the authors and the key `c` of the lists, therefore that
            // view takes them and every other view keeps its own key.
            // See T-124.
            KeyCode::Char('a') if matches!(self.view_state, AppView::SettingsAccount) => {
                self.add_an_account()
            }
            KeyCode::Char('c') if matches!(self.view_state, AppView::SettingsAccount) => {
                self.this_account_starts()
            }

            // The key that opens the ebook of the item that the user selected.
            KeyCode::Char('e') => self.open_the_ebook(),

            // The key that marks a media as finished, or not finished.
            // See T-24.
            KeyCode::Char('M') => self.toggle_the_mark_of_finished(),

            // The key that takes a media away from the shelf of Continue
            // Listening, or puts it back. See T-24.
            KeyCode::Char('N') => self.toggle_the_shelf_of_continue_listening(),

            // The key that shows the chapters of the media that plays.
            // See T-24.
            KeyCode::Char('C') => self.show_the_chapters(),

            // The key that tells the server to examine the library. See
            // T-24.
            KeyCode::Char('L') => self.scan_the_library(),

            // The key that shows the authors of the library. See T-24.
            KeyCode::Char('a') => self.show_the_authors(),

            // The key that shows the narrators of the library. See T-73.
            KeyCode::Char('v') => self.show_the_narrators(),

            // The key that tells the server to get the new episodes of a
            // podcast. See T-24.
            KeyCode::Char('E') => self.get_the_new_episodes(),

            // The key that looks for a new podcast. See T-24.
            KeyCode::Char('A') => self.look_for_a_podcast(),

            // The key that shows the episodes that the server downloads, and
            // the queue of that work. See T-81.
            KeyCode::Char('d') => self.show_the_downloads_of_the_server(),

            // The key that puts the media of the line in a collection or in a
            // playlist. See T-84.
            KeyCode::Char('m') => self.show_the_lists_that_take_the_media(),

            // The key that sends the book of the line to an e-reader. The
            // letter of an address of e-mail is the mark of that work, and it
            // takes no letter of the alphabet from a view. See T-119.
            KeyCode::Char('@') => self.show_the_devices_of_an_ereader(),

            // The key that gives a collection or a playlist a new name. See
            // T-93. It works in the view of the lists only.
            KeyCode::Char('r') if matches!(self.view_state, AppView::Lists) => {
                self.give_the_list_of_the_line_a_new_name()
            }

            // The key of the description of a list. See T-100. It stands
            // before the key `D` of the copy on the disk, and a list holds no
            // copy on the disk.
            KeyCode::Char('D') if matches!(self.view_state, AppView::Lists) => {
                self.give_the_list_of_the_line_a_new_description()
            }

            // The keys that make a new collection and a new playlist. See
            // T-88. They stand before the keys `c` and `p` of the program,
            // therefore this view takes them. The key `p` of the player and the
            // key `c` of the collections do their work in every other view.
            KeyCode::Char('c') if matches!(self.view_state, AppView::PutInAList) => {
                self.make_a_new_list(crate::api::utils::collect_lists::ListKind::Collection)
            }
            KeyCode::Char('p') if matches!(self.view_state, AppView::PutInAList) => {
                self.make_a_new_list(crate::api::utils::collect_lists::ListKind::Playlist)
            }

            // The key of the timer for sleep. See T-24.
            KeyCode::Char('t') => self.change_the_timer_for_sleep(),

            // The key that writes a bookmark at the place of the playback.
            // See T-24.
            KeyCode::Char('b') => self.write_a_bookmark(),

            // The key that shows the bookmarks of a media. See T-24.
            KeyCode::Char('V') => self.show_the_bookmarks(),

            // The key that shows the time that the user listened. See T-24.
            KeyCode::Char('T') => self.show_the_statistics(),

            // The key that shows every session of the user, with pages. The
            // key `T` shows the five last sessions only. See T-24.
            KeyCode::Char('W') => self.show_the_sessions(),

            // The key that chooses the sequence and the filter. See T-24.
            // The key `n` puts the selected media at the end of the queue,
            // and the key `q` shows the queue. See T-24.
            KeyCode::Char('n') => self.add_to_the_queue(),

            KeyCode::Char('q') => self.show_the_queue(),

            KeyCode::Char('f') => self.show_the_sequence_and_the_filter(),

            // PLAYER //
            // toggle playback/pause
            KeyCode::Char(' ') => {
                handle_key_player(
                    " ",
                    &self.player,
                    self.username.as_str(),
                    self.server_key.as_str(),
                );
            }
            // jump forward
            KeyCode::Char('p') => {
                handle_key_player(
                    "p",
                    &self.player,
                    self.username.as_str(),
                    self.server_key.as_str(),
                );
            }

            // jump backward
            KeyCode::Char('u') => {
                handle_key_player(
                    "u",
                    &self.player,
                    self.username.as_str(),
                    self.server_key.as_str(),
                );
            }

            // next chapter
            KeyCode::Char('P') => {
                handle_key_player(
                    "P",
                    &self.player,
                    self.username.as_str(),
                    self.server_key.as_str(),
                );
            }

            // previous chapter
            KeyCode::Char('U') => {
                handle_key_player(
                    "U",
                    &self.player,
                    self.username.as_str(),
                    self.server_key.as_str(),
                );
            }

            // speed rate up
            KeyCode::Char('O') => {
                handle_key_player(
                    "O",
                    &self.player,
                    self.username.as_str(),
                    self.server_key.as_str(),
                );
            }

            // speed rate down
            KeyCode::Char('I') => {
                handle_key_player(
                    "I",
                    &self.player,
                    self.username.as_str(),
                    self.server_key.as_str(),
                );
            }

            // volume up
            KeyCode::Char('o') => {
                handle_key_player(
                    "o",
                    &self.player,
                    self.username.as_str(),
                    self.server_key.as_str(),
                );
            }

            // volume down
            KeyCode::Char('i') => {
                handle_key_player(
                    "i",
                    &self.player,
                    self.username.as_str(),
                    self.server_key.as_str(),
                );
            }

            // stop the playback
            KeyCode::Char('Y') => {
                handle_key_player(
                    "Y",
                    &self.player,
                    self.username.as_str(),
                    self.server_key.as_str(),
                );
            }

            // show key bindings
            //
            // **A key that reads a state of the disk and that then writes it**
            // (the shape of T-175) did nothing at all when that read failed: the
            // value was neither "0" nor "1", therefore no branch wrote the disk,
            // and no word told the user. The `App` holds that value now (T-204),
            // and a write that failed keeps it and says why.
            KeyCode::Char('B') => {
                let value = !self.the_key_bindings_stand;

                match update_is_show_key_bindings(
                    if value { "1" } else { "0" },
                    self.username.as_str(),
                ) {
                    Ok(()) => self.the_key_bindings_stand = value,
                    Err(why) => {
                        log::error!(
                            "[key B] the program did not write the row of the keys of the player: {}",
                            why
                        );
                        crate::logic::message::say(
                            crate::ui::keys::THE_KEYS_OF_THE_PLAYER_DID_NOT_REACH_THE_DISK,
                        );
                    }
                }
            }

            // END PLAYER //

            // download the selected book or episode for offline listening
            KeyCode::Char('D') => {
                // The server refuses a download for an account that may not
                // download, and it gives an error of the protocol. The user
                // reads a sentence instead. See T-24.
                if !self.account.permissions.download {
                    crate::logic::message::say(crate::api::me::permissions::no_download());
                    return;
                }

                let token = self.token.clone();
                // **The download goes to the address that answers**, and not to
                // the address of the login: a user away from home holds the
                // address of the house in their row of the database, and every
                // other request of the program takes the pool already. See
                // T-149, and T-105 and T-128 for the rule of the pool.
                let server_address = crate::logic::download::the_address_of_the_download(
                    self.api.pool().an_address(),
                    &self.server_address,
                );
                let username = self.username.clone();
                let server_key = self.server_key.clone();

                if let Some((target, title, author)) = self.selected_download() {
                    // The map is global. Therefore the bar stays correct when the
                    // user refreshes the screen with the key `R`.
                    let progress = crate::logic::download::downloads();
                    tokio::spawn(async move {
                        download_with_progress(
                            token,
                            target,
                            server_address,
                            username,
                            title,
                            author,
                            server_key,
                            progress,
                        )
                        .await;
                    });
                }
            }

            // The key `X` removes a bookmark inside the view of the
            // bookmarks. Every other view removes the local copy. See T-24.
            KeyCode::Char('X') if matches!(self.view_state, AppView::Queue) => {
                self.remove_from_the_queue();
            }

            KeyCode::Char('X') if matches!(self.view_state, AppView::Bookmarks) => {
                self.remove_the_bookmark()
            }

            KeyCode::Char('X') if matches!(self.view_state, AppView::Downloads) => {
                self.empty_the_queue_of_the_downloads()
            }

            // The key that takes the media of the line out of the list that
            // holds it. See T-84.
            KeyCode::Char('X') if matches!(self.view_state, AppView::Lists) => {
                self.remove_the_list_of_the_line()
            }

            KeyCode::Char('X') if matches!(self.view_state, AppView::ListEntries) => {
                self.take_the_media_out_of_the_list()
            }

            // The sequence of the media inside a collection or a playlist. The
            // keys stand in the view of the media of a list only: no other view
            // of the program holds a sequence that a user writes. See T-102.
            KeyCode::Char('<') if matches!(self.view_state, AppView::ListEntries) => {
                self.move_the_media_of_the_list(false)
            }

            KeyCode::Char('>') if matches!(self.view_state, AppView::ListEntries) => {
                self.move_the_media_of_the_list(true)
            }

            // remove the local copy of the selected book or episode
            KeyCode::Char('X') => {
                let username = self.username.clone();

                if let Some((target, title_of_the_line, _author)) = self.selected_download() {
                    // **A download that runs holds its files** (T-150). A
                    // removal of them gives that writer a fault, and it gives
                    // the user "Download failed" for a download that works:
                    // that is the shape of T-148 from the other side.
                    let work = crate::logic::download::the_work_of_the_key_that_removes(
                        crate::logic::download::this_program_downloads(target.key()),
                        crate::logic::download::a_program_downloads(target.key(), &username),
                        // **A media that plays from the disk keeps its files.**
                        // An offline playback stands in no session of the
                        // server, and the place of that playback in
                        // `pending_progress` moves at each second (T-152): that
                        // moment is the one word of it that a second program of
                        // this account reads. See T-156.
                        // **A read of the disk that failed is not a media that no
                        // program plays** (T-203): this key then removes nothing.
                        crate::db::crud::a_program_keeps_the_place_of_this_media(
                            &username,
                            target.item_id(),
                            target.episode_id().unwrap_or_default(),
                        )
                        .map_err(|_| ()),
                    );

                    use crate::logic::download::TheWorkOfTheKeyThatRemoves as TheWork;

                    match work {
                        TheWork::AProgramPlaysItFromTheDisk => {
                            crate::logic::message::say(
                                &crate::logic::download::text_of_the_media_that_plays_from_the_disk(
                                    &title_of_the_line,
                                ),
                            );
                        }
                        TheWork::ThisProgramDownloads | TheWork::ADifferentProgramDownloads => {
                            crate::logic::message::say(
                                &crate::logic::download::text_of_the_download_that_runs(
                                    &title_of_the_line,
                                    work == TheWork::ThisProgramDownloads,
                                ),
                            );
                        }
                        // **The program does not know which program holds these
                        // files**, therefore it removes none of them. See T-203.
                        TheWork::TheDatabaseDidNotAnswer => {
                            crate::logic::message::say(
                                &crate::logic::download::text_of_the_database_that_did_not_answer(
                                    &title_of_the_line,
                                ),
                            );
                        }
                        TheWork::TakeTheDisk => {
                            // The audio of the download, and the ebook that the
                            // reader keeps. **The reader kept its file for ever
                            // before T-65**, and a PDF of a scan holds some
                            // hundred megabytes.
                            use crate::logic::download::TheRemovalOfADownload as TheRemoval;

                            match remove_download(target.key(), &username) {
                                TheRemoval::TheDiskAndTheDatabase(title, of_the_audio) => {
                                    let of_the_ebook =
                                        crate::logic::download::remove_the_ebook_of_the_item(
                                            target.item_id(),
                                            &username,
                                        );

                                    crate::logic::message::say(
                                        &crate::logic::download::text_of_the_removal(
                                            &title.unwrap_or(title_of_the_line),
                                            &of_the_audio,
                                            of_the_ebook,
                                        ),
                                    );
                                }
                                // The database says nothing, therefore the ebook of
                                // the cache stays too: the two copies of one media
                                // go away together, and the key `X` again does that
                                // work.
                                TheRemoval::TheDatabaseSaidNothing => {
                                    crate::logic::message::say(
                                        &crate::logic::download::text_of_the_database_that_did_not_answer(
                                            &title_of_the_line,
                                        ),
                                    );
                                }
                                TheRemoval::TheRowsOfTheDatabaseStay => {
                                    crate::logic::message::say(
                                        &crate::logic::download::text_of_the_rows_that_stay(
                                            &title_of_the_line,
                                        ),
                                    );
                                }
                            }
                        }
                    }
                }
            }

            // show the series of the library
            KeyCode::Char('s') => match self.view_state {
                AppView::Home | AppView::Library | AppView::SearchBook => {
                    // **A key that does nothing must say why.** The key `a` of
                    // a library of podcasts says "A library of podcasts has no
                    // author", and this key said nothing at all. See T-83.
                    if self.is_podcast {
                        crate::logic::message::say("A library of podcasts has no series.");
                    } else {
                        self.list_state_series.select(Some(0));
                        self.scroll_offset = 0;
                        self.view_state = AppView::Series;
                    }
                }
                _ => {}
            },

            // show the collections and the playlists
            KeyCode::Char('c') => match self.view_state {
                AppView::Home | AppView::Library | AppView::SearchBook => {
                    self.list_state_lists.select(Some(0));
                    self.scroll_offset = 0;
                    self.view_state = AppView::Lists;
                }
                _ => {}
            },

            KeyCode::Char('/') => {
                let _ = self.search_active();
            }
            KeyCode::Char('S') => {
                self.view_state = AppView::Settings;
            }

            // The list of every key. The footer of a view names the keys of
            // the work of that view only, therefore this list holds the rest.
            // See T-49.
            KeyCode::Char('?') => self.show_every_key(),

            // The key that forces the sync. See T-32 and upstream issue #37.
            //
            // The design named the key `S`, and `S` was not free: it opens the
            // settings. `F` reads as "force the sync".
            //
            // The key writes a flag only. The loop of the playback sends the
            // position at its next second, because that loop holds the
            // listened time. The command does not close the session.
            KeyCode::Char('F') => {
                let state = self.player.state();

                if state.status == crate::player::engine::PlaybackStatus::Stopped
                    || !crate::logic::sync_session::force_sync::ask(state.playback_id)
                {
                    crate::logic::message::say("Sync: nothing plays now.");
                } else {
                    crate::logic::message::say("Sync: the application sends the position…");

                    // The answer comes from the loop of the playback. This
                    // task waits for it, and it stops after a short time when
                    // no answer comes.
                    tokio::spawn(async move {
                        for _ in 0..40 {
                            tokio::time::sleep(std::time::Duration::from_millis(200)).await;

                            if let Some(text) =
                                crate::logic::sync_session::force_sync::take_report()
                            {
                                crate::logic::message::say(text.as_str());
                                return;
                            }
                        }

                        crate::logic::message::say("Sync: the playback gave no answer.");
                    });
                }
            }
            KeyCode::Tab => {
                if self.is_from_search_pod {
                    self.is_from_search_pod = false;
                };
                self.toggle_view()
            }

            // The key that takes the next library of the server. crossterm
            // gives Shift+Tab as its own code, therefore this handler needs no
            // work for a modifier (the trap 58). See T-66.
            KeyCode::BackTab => self.take_the_next_library(),

            // `Esc` inside the list of every key closes that list. A key that
            // stops the whole program must not stand alone in a view that the
            // user opened to read. See T-49.
            KeyCode::Esc if matches!(self.view_state, AppView::Keys) => {
                self.view_state = self.the_view_before_the_keys;
            }

            KeyCode::Char('Q') | KeyCode::Esc => {
                // display message
                let message_quit = "Exiting the application and syncing data, please hold on.";
                crate::logic::message::say(message_quit);

                // close and sync session before close the app
                let api = std::sync::Arc::clone(&self.api);
                let username = self.username.clone();
                let server_key = self.server_key.clone();

                // Stop the engine before the application syncs and stops.
                self.player.send(crate::player::engine::PlayerCommand::Stop);

                tokio::spawn(async move {
                    sync_session_from_database(&api, username, server_key, true, "Q").await;
                });
            }

            KeyCode::Char('j') | KeyCode::Down => {
                self.select_next();
                self.scroll_offset = 0;
            }
            // scroll up into description section
            KeyCode::Char('J') => self.scroll_offset += 1,
            // go start description section
            KeyCode::Char('H') => self.scroll_offset = 0,
            KeyCode::Char('k') | KeyCode::Up => {
                self.select_previous();
                self.scroll_offset = 0;
            }

            // scroll down into description section
            KeyCode::Char('K') => {
                if usize::from(self.scroll_offset) > 0 {
                    self.scroll_offset -= 1;
                }
            }
            KeyCode::Char('g') | KeyCode::Home => {
                self.select_first();
                self.scroll_offset = 0;
            }
            KeyCode::Char('G') | KeyCode::End => {
                self.select_last();
                self.scroll_offset = 0;
            }
            KeyCode::Char('h') => {
                // To return to a page
                match self.view_state {
                    AppView::Keys => self.view_state = self.the_view_before_the_keys,
                    AppView::SettingsAccount => self.view_state = AppView::Settings,
                    AppView::SettingsLibrary => self.view_state = AppView::Settings,
                    AppView::SettingsAbout => self.view_state = AppView::Settings,
                    AppView::SettingsUpdateUninstall => self.view_state = AppView::Settings,
                    // The footer of this view says "h: back", and the key did
                    // nothing: the user then pressed `Esc`, and that key stops
                    // the program. See T-143.
                    AppView::SettingsReader => self.view_state = AppView::Settings,
                    AppView::Settings => self.view_state = AppView::Home,
                    // The view of the statistics goes back to Home, as the
                    // settings do. See T-24.
                    AppView::Stats | AppView::Sessions => self.view_state = AppView::Home,
                    AppView::SortFilter => self.view_state = AppView::Library,
                    // The view of the chapters goes back to the Home view.
                    AppView::Chapters => self.view_state = AppView::Home,
                    AppView::Bookmarks => self.view_state = AppView::Home,
                    AppView::Queue => self.view_state = AppView::Home,
                    AppView::NewPodcast => self.view_state = AppView::Library,
                    AppView::Authors => self.view_state = AppView::Library,
                    // The view of the search goes back to the view that opened
                    // it. A sweep of 2026-08-11 pressed `h` there, and the
                    // screen did not move. See T-79.
                    AppView::SearchBook => self.view_state = self.the_view_before_the_search,
                    // The user came from the reader, and the reader holds the
                    // book that they read now. See T-76.
                    AppView::Ebooks => self.view_state = AppView::Reader,
                    AppView::Downloads => self.view_state = AppView::Library,
                    AppView::PutInAList => self.view_state = self.the_view_before_the_list,
                    AppView::SendToEreader => self.view_state = self.the_view_before_the_send,
                    AppView::PodcastEpisode => {
                        if self.is_from_search_pod {
                            self.view_state = AppView::SearchBook
                        } else {
                            self.view_state = AppView::Library
                        }
                    }
                    AppView::Series => {
                        self.scroll_offset = 0;
                        self.view_state = AppView::Library
                    }
                    AppView::SeriesBook => {
                        self.scroll_offset = 0;
                        // The key `h` goes back to the view that opened the
                        // series. See T-22.
                        self.view_state = self.series_from;
                        self.series_from = AppView::Series;
                    }
                    AppView::Lists => {
                        self.scroll_offset = 0;
                        self.view_state = AppView::Library
                    }
                    AppView::ListEntries => {
                        self.scroll_offset = 0;
                        self.view_state = AppView::Lists
                    }
                    _ => {}
                }
            }
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                // Clone needed because variables will be used in a spawn
                let api = std::sync::Arc::clone(&self.api);
                let server_address = self.server_address.clone();
                let username = self.username.clone();
                let server_key = self.server_key.clone();
                let player = self.player.clone();

                // Init for the Home view.
                //
                // The Home view holds the name of a shelf and a series, and
                // neither of them is a media. Therefore this number is the
                // place of the media in the lists, and not the place of the
                // line on the screen. See T-24.
                let ids_cnt_list = self._ids_cnt_list.clone();
                let selected_cnt_list = self.selected_home_item();

                // Init for `Library`
                let ids_library = self.ids_library.clone();
                let selected_library = self.list_state_library.selected();

                // Init for `Search Book`
                let ids_search_book = self.ids_search_book.clone();
                let selected_search_book = self.list_state_search_results.selected();

                // Duration of the whole book, from the `media.duration` field. The playback
                // session gives the duration of the first audio file only, thus a book with
                // many audio files needs this value. See upstream issue #33.
                let whole_book_duration_cnt_list =
                    selected_cnt_list.and_then(|i| self.duration_cnt_list.get(i).copied());
                let whole_book_duration_library =
                    selected_library.and_then(|i| self.duration_library.get(i).copied());
                let whole_book_duration_search_book = selected_search_book
                    .and_then(|i| self.duration_library_search_book.get(i).copied());

                // Init for `PodcastEpisode`
                // **`get` gives nothing for a line that these lists do not
                // hold, and an index of a vector stops the program.** A podcast
                // of a page that the program did not read has no row here.
                // See T-126 and T-41.
                if self.is_podcast {
                    if let Some(index) = selected_library {
                        if ids_library.get(index).is_some() {
                            self.ids_pod_ep =
                                self.all_ids_pod_ep.get(index).cloned().unwrap_or_default();
                        }
                    }
                    if let Some(index) = selected_search_book {
                        // ids_library_pod_search because we need the pod id and he is given by
                        // this variable
                        if self.ids_library_pod_search.get(index).is_some() {
                            self.ids_pod_ep_search = self
                                .all_ids_pod_ep_search
                                .get(index)
                                .cloned()
                                .unwrap_or_default();
                        }
                    }
                }
                // Init for `SettingsAccount`
                let selected_account = self.list_state_settings_account.selected();

                // Init for `SettingsLibrary`
                let selected_settings_library = self.list_state_settings_library.selected();

                // Now, spawn the async task based on the current view state
                match self.view_state {
                    // The view of the keys is a list of text. No line of it
                    // holds a media, therefore the key `l` does nothing.
                    AppView::Keys => {}
                    AppView::Home => {
                        // A line of a series opens the books of that series,
                        // in the same way as the Library view. See T-22.
                        if let Some(index) = self.selected_home_row().and_then(|row| row.series()) {
                            if self.series.get(index).is_some_and(|s| !s.books.is_empty()) {
                                self.list_state_series.select(Some(index));
                                self.list_state_series_book.select(Some(0));
                                self.scroll_offset = 0;
                                self.series_from = AppView::Home;
                                self.view_state = AppView::SeriesBook;
                            }
                        } else if self.is_podcast {
                            // init some variables
                            let _selected_pod_ep = self.list_state_pod_ep.selected();
                            let ids_ep_cnt_list = self.ids_ep_cnt_list.clone();

                            tokio::spawn(async move {
                                if let Some(episode_id) = selected_cnt_list
                                    .and_then(|i| ids_ep_cnt_list.get(i))
                                    .cloned()
                                {
                                    play(
                                        &api,
                                        &player,
                                        PlaybackTarget::Episode {
                                            item_id: ids_cnt_list[selected_cnt_list.unwrap_or(0)]
                                                .clone(),
                                            episode_id,
                                        },
                                        username,
                                        server_address,
                                        server_key,
                                    )
                                    .await;
                                }
                            });
                        } else {
                            tokio::spawn(async move {
                                if let Some(item_id) =
                                    selected_cnt_list.and_then(|i| ids_cnt_list.get(i)).cloned()
                                {
                                    play(
                                        &api,
                                        &player,
                                        PlaybackTarget::Book {
                                            item_id,
                                            whole_book_duration: whole_book_duration_cnt_list,
                                        },
                                        username,
                                        server_address,
                                        server_key,
                                    )
                                    .await;
                                }
                            });
                        }
                    }
                    AppView::Settings => match self.list_state_settings.selected() {
                        // The view reads the accounts of the disk when it opens:
                        // a second program of this account adds and removes a
                        // row while this window stands open. See T-155.
                        Some(0) => {
                            self.the_accounts_come_from_the_disk();
                            self.view_state = AppView::SettingsAccount;
                        }
                        Some(1) => self.view_state = AppView::SettingsLibrary,
                        Some(4) => self.show_the_settings_of_the_reader(),
                        _ => {}
                    },
                    AppView::SettingsReader => self.take_the_value_of_the_cache(),
                    // The list can be shorter than the selection: the user
                    // removes an account, and the list of the accounts keeps
                    // its old length until the next refresh. An index of a
                    // vector stops the program. `get` does not. See T-41.
                    AppView::SettingsAccount => {
                        if let Some(usr_to_delete) = selected_account
                            .and_then(|index| self.the_accounts.get(index))
                            .map(|(name, _, _)| name.clone())
                        {
                            // A log out forgets the token of a server, and the
                            // user then gives their password again. Therefore
                            // the program asks one time. Any key that is not
                            // `l` stops the question. See T-36.
                            if self.confirm_logout.as_deref() != Some(usr_to_delete.as_str()) {
                                self.confirm_logout = Some(usr_to_delete.clone());

                                crate::logic::message::say(&format!(
                                    "Press l again to log out of \"{}\". Any other key stops this.",
                                    usr_to_delete
                                ));

                                return;
                            }

                            self.confirm_logout = None;

                            // **The disk is the truth, and the key acts on the
                            // account of its own line** (T-147). A second
                            // program of this account can remove that account
                            // while this view stands: the key then changed no
                            // row and it said nothing at all (T-79), and the
                            // rule of the log out read a list of a phantom.
                            // See T-155.
                            self.the_accounts_come_from_the_disk();

                            if matches!(
                                crate::logic::the_accounts::the_account_of_the_line(
                                    &self.the_accounts,
                                    &usr_to_delete
                                ),
                                crate::logic::the_accounts::TheAccountOfTheLine::ItIsGone
                            ) {
                                crate::logic::message::say(
                                    &crate::logic::the_accounts::the_text_of_an_account_that_is_gone(
                                        &usr_to_delete,
                                    ),
                                );

                                return;
                            }

                            // **A log out that removed no row is no log out**
                            // (T-200). The old code read the answer of that
                            // work with `let _ =`, and the words of the fault
                            // came from the module of the database: the user
                            // read "Error connecting to the database.", and the
                            // program then took the account of the line for an
                            // account that went away.
                            if let Err(error) = delete_user(usr_to_delete.as_str()) {
                                log::error!(
                                    "[the accounts] the program did not remove the account {}: {}",
                                    usr_to_delete,
                                    error
                                );

                                crate::logic::message::say(
                                    "The program did not remove the account. Stop a second \
                                     Toutui, and press the key again.",
                                );

                                return;
                            }

                            // **A log out of the account that starts leaves the
                            // program with no account of a start.** The first
                            // account that stays takes that work, and the
                            // program starts again with it: every list of the
                            // screen comes from one account. With no account at
                            // all the program starts again too, and the login
                            // screen of a first start comes. See T-124 and
                            // T-123.
                            let what_comes_now =
                                crate::logic::the_accounts::the_account_after_a_log_out(
                                    &self.the_accounts,
                                    &usr_to_delete,
                                );

                            // The list must follow the change at once.
                            self.the_accounts
                                .retain(|(name, _, _)| name != &usr_to_delete);

                            match what_comes_now {
                                crate::logic::the_accounts::AfterALogOut::ThisAccountStarts(
                                    name,
                                ) => {
                                    self.start_the_program_with_this_account(&name);
                                    return;
                                }
                                crate::logic::the_accounts::AfterALogOut::TheLoginScreen => {
                                    self.the_login_screen_comes();
                                    return;
                                }
                                crate::logic::the_accounts::AfterALogOut::TheViewOnly => {}
                            }

                            let last = self.the_accounts.len().saturating_sub(1);
                            if self.the_accounts.is_empty() {
                                self.list_state_settings_account.select(None);
                            } else if selected_account.unwrap_or(0) > last {
                                self.list_state_settings_account.select(Some(last));
                            }
                        }
                    }
                    AppView::SettingsLibrary => {
                        if let Some(new_selected_lib) = selected_settings_library
                            .and_then(|index| self.libraries_ids.get(index))
                        {
                            // **A write of the disk that failed is no new
                            // library** (T-205). See `take_the_next_library`.
                            if let Err(error) =
                                update_id_selected_lib(new_selected_lib, &self.username)
                            {
                                log::error!(
                                    "[the library of the settings] the program did not write \
                                     the library of {}: {}",
                                    self.username,
                                    error
                                );

                                crate::logic::message::say(
                                    crate::ui::keys::THE_LIBRARY_DID_NOT_REACH_THE_DISK,
                                );

                                return;
                            }

                            // **The screen must follow the choice.** The old
                            // code wrote the row of the database only, and every
                            // list of the screen stayed: the user read
                            // "Books (book)" in the header after they took the
                            // library of the podcasts, and the key `R` or a new
                            // start of the program did the work. See T-82.
                            let name = selected_settings_library
                                .and_then(|index| self.libraries_names.get(index))
                                .cloned()
                                .unwrap_or_default();

                            crate::logic::message::say(&format!(
                                "The program shows the library \"{}\" now.",
                                name
                            ));

                            self.must_refresh = true;
                        }
                    }
                    AppView::SettingsAbout => {}
                    AppView::SettingsUpdateUninstall => {}
                    // The reader has its own keys. See T-10.
                    AppView::Reader => {}
                    // The view of the statistics holds no line to open.
                    AppView::Stats | AppView::Sessions => {}
                    AppView::SortFilter => self.apply_the_sequence_or_the_filter(),
                    AppView::Chapters => self.go_to_the_chapter(),
                    AppView::Bookmarks => self.go_to_the_bookmark(),
                    AppView::Queue => self.start_the_media_of_the_queue(),
                    AppView::NewPodcast => self.add_the_podcast(),
                    AppView::Authors => self.show_the_books_of_the_author(),
                    AppView::Ebooks => self.open_the_ebook_of_the_line(),
                    // The server owns this work, therefore no line of this
                    // view opens. The key `X` empties the queue. See T-81.
                    AppView::Downloads => {}
                    AppView::PutInAList => self.put_the_media_in_the_list(),
                    AppView::SendToEreader => self.send_the_book_to_an_ereader(),
                    AppView::Library => {
                        // A line of a series opens the books of that series.
                        // See T-22.
                        if let Some(index) =
                            self.selected_library_row().and_then(|row| row.series())
                        {
                            if self.series.get(index).is_some_and(|s| !s.books.is_empty()) {
                                self.list_state_series.select(Some(index));
                                self.list_state_series_book.select(Some(0));
                                self.scroll_offset = 0;
                                self.series_from = AppView::Library;
                                self.view_state = AppView::SeriesBook;
                            }
                        } else if self.is_podcast {
                            // **An index of a vector stops the program, and
                            // `get` does not** (T-41). A podcast of a page that
                            // the program did not read holds no row of these
                            // lists, and the key `l` of that line stopped the
                            // program: the sweep of a library of 520 podcasts
                            // of 2026-08-12 measured it. See T-126.
                            if let Some(index) = selected_library {
                                self.take_the_episodes_of_the_line(index);
                                self.list_state_pod_ep.select(Some(0));
                                self.view_state = AppView::PodcastEpisode;
                            }
                        } else {
                            tokio::spawn(async move {
                                if let Some(item_id) =
                                    selected_library.and_then(|i| ids_library.get(i)).cloned()
                                {
                                    play(
                                        &api,
                                        &player,
                                        PlaybackTarget::Book {
                                            item_id,
                                            whole_book_duration: whole_book_duration_library,
                                        },
                                        username,
                                        server_address,
                                        server_key,
                                    )
                                    .await;
                                }
                            });
                        }
                    }
                    AppView::SearchBook => {
                        if self.is_podcast {
                            self.is_from_search_pod = true;
                            if let Some(index) = selected_search_book {
                                // The lists of this view come from the place of
                                // the media in the lists of the library (T-113),
                                // and `get` gives nothing for a line that they
                                // do not hold. See T-126 and T-41.
                                let at = |list: &Vec<Vec<String>>| {
                                    list.get(index).cloned().unwrap_or_default()
                                };

                                self.titles_pod_ep_search = at(&self.all_titles_pod_ep_search);
                                self.subtitles_pod_ep_search =
                                    at(&self.all_subtitles_pod_ep_search);
                                self.seasons_pod_ep_search = at(&self.all_seasons_pod_ep_search);
                                self.episodes_pod_ep_search = at(&self.all_episodes_pod_ep_search);
                                self.authors_pod_ep_search = at(&self.all_authors_pod_ep_search);
                                self.descs_pod_ep_search = at(&self.all_descs_pod_ep_search);
                                self.titles_pod_search = at(&self.all_titles_pod_search);
                                self.durations_pod_ep_search =
                                    at(&self.all_durations_pod_ep_search);

                                // **The program reads the episodes of a podcast
                                // when the user opens it**, and the view of the
                                // search opens one too. See T-126.
                                if let Some(place) =
                                    self.ids_library_pod_search.get(index).and_then(|id| {
                                        self.ids_library.iter().position(|one| one == id)
                                    })
                                {
                                    self.ask_the_server_for_the_episodes(place);
                                }

                                self.list_state_pod_ep.select(Some(0));
                                self.view_state = AppView::PodcastEpisode;
                            }
                        } else {
                            tokio::spawn(async move {
                                if let Some(item_id) = selected_search_book
                                    .and_then(|i| ids_search_book.get(i))
                                    .cloned()
                                {
                                    play(
                                        &api,
                                        &player,
                                        PlaybackTarget::Book {
                                            item_id,
                                            whole_book_duration: whole_book_duration_search_book,
                                        },
                                        username,
                                        server_address,
                                        server_key,
                                    )
                                    .await;
                                }
                            });
                        }
                    }
                    // The series gives the books of the series.
                    AppView::Series => {
                        if let Some(index) = self.list_state_series.selected() {
                            if self.series.get(index).is_some_and(|s| !s.books.is_empty()) {
                                self.list_state_series_book.select(Some(0));
                                self.scroll_offset = 0;
                                self.series_from = AppView::Series;
                                self.view_state = AppView::SeriesBook;
                            }
                        }
                    }
                    // A book of a series plays in the same way as a book of the
                    // library.
                    AppView::SeriesBook => {
                        if let Some(book) = self.selected_series_book() {
                            let item_id = book.id.clone();
                            let duration = book.duration;

                            tokio::spawn(async move {
                                play(
                                    &api,
                                    &player,
                                    PlaybackTarget::Book {
                                        item_id,
                                        whole_book_duration: Some(duration),
                                    },
                                    username,
                                    server_address,
                                    server_key,
                                )
                                .await;
                            });
                        }
                    }
                    // A collection or a playlist gives its media.
                    AppView::Lists => match self.selected_list() {
                        None => crate::logic::message::say("No list is selected."),
                        Some(list) if !list.entries.is_empty() => {
                            self.list_state_list_entries.select(Some(0));
                            self.scroll_offset = 0;
                            self.view_state = AppView::ListEntries;
                        }
                        Some(_) => {}
                    },
                    // A medium of a list is a book or an episode.
                    AppView::ListEntries => {
                        if let Some(entry) = self.selected_list_entry() {
                            let item_id = entry.id.clone();
                            let duration = entry.duration;

                            let target = match entry.episode_id.clone() {
                                Some(episode_id) => PlaybackTarget::Episode {
                                    item_id,
                                    episode_id,
                                },
                                None => PlaybackTarget::Book {
                                    item_id,
                                    whole_book_duration: Some(duration),
                                },
                            };

                            tokio::spawn(async move {
                                play(&api, &player, target, username, server_address, server_key)
                                    .await;
                            });
                        }
                    }
                    AppView::PodcastEpisode => {
                        if self.is_from_search_pod {
                            // we need the index of selected_search_book to feet after with
                            // ids_library_pod_search
                            if let Some(index) = selected_search_book {
                                // ids_library_pod_search because we need the pod id and he is given by
                                // this variable
                                if let Some(id_pod) = self.ids_library_pod_search.get(index) {
                                    //    println!("{:?}", id_pod);
                                    let all_ids_pod_ep_search_clone =
                                        self.all_ids_pod_ep_search.clone();
                                    //   println!("{:?}", all_ids_pod_ep_search_clone[index]);
                                    let id_pod_clone = id_pod.clone();
                                    let selected_pod_ep = self.list_state_pod_ep.selected();

                                    tokio::spawn(async move {
                                        if let Some(episode_id) = all_ids_pod_ep_search_clone[index]
                                            .get(selected_pod_ep.unwrap_or(0))
                                            .cloned()
                                        {
                                            play(
                                                &api,
                                                &player,
                                                PlaybackTarget::Episode {
                                                    item_id: id_pod_clone,
                                                    episode_id,
                                                },
                                                username,
                                                server_address,
                                                server_key,
                                            )
                                            .await;
                                        }
                                    });
                                }
                            }
                        } else {
                            // selected_livrary ids_library because we need the pod id and he is given by
                            // these variables
                            // we also need the index of selected library to feet after with
                            // ids_library
                            if let Some(index) = selected_library {
                                if let Some(id_pod) = ids_library.get(index) {
                                    let of_the_podcast: Vec<String> =
                                        self.all_ids_pod_ep.get(index).cloned().unwrap_or_default();
                                    self.ids_pod_ep = of_the_podcast.clone();
                                    let id_pod_clone = id_pod.clone();
                                    let selected_pod_ep = self.list_state_pod_ep.selected();
                                    tokio::spawn(async move {
                                        if let Some(episode_id) = of_the_podcast
                                            .get(selected_pod_ep.unwrap_or(0))
                                            .cloned()
                                        {
                                            play(
                                                &api,
                                                &player,
                                                PlaybackTarget::Episode {
                                                    item_id: id_pod_clone,
                                                    episode_id,
                                                },
                                                username,
                                                server_address,
                                                server_key,
                                            )
                                            .await;
                                        }
                                    });
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        // **The key moved the list, therefore the user can be near the end of
        // the lines that the program holds.** The function itself decides, and
        // it asks the server for one page at the most at a time. See T-70.
        self.ask_for_the_next_page_of_the_library();
    }

    /// Marks the selected media as finished, or as not finished. See T-24.
    ///
    /// The program sent `isFinished` at the end of a playback only. A user who
    /// leaves a book in the middle could not take it out of the list Continue
    /// Listening.
    ///
    /// The task asks the server for the condition of the media first, and it
    /// then sends the opposite. The user therefore presses one key, and the
    /// key does the right work in every view.
    pub fn toggle_the_mark_of_finished(&mut self) {
        let Some((item_id, episode_id)) = self.selected_place() else {
            crate::logic::message::say(self.words_of_a_line_with_no_place());
            return;
        };

        if self.is_offline {
            crate::logic::message::say("The server does not answer.");
            return;
        }

        crate::logic::message::say("The mark of the media goes to the server…");
        let api = std::sync::Arc::clone(&self.api);

        tokio::spawn(async move {
            let text = mark_the_media(&api, &item_id, episode_id.as_deref()).await;

            crate::logic::message::say(text.as_str());
        });
    }

    /// Takes the selected media away from the shelf of Continue Listening,
    /// or puts it back. See T-24.
    ///
    /// The field `hideFromContinueListening` of `PATCH /api/me/progress/:id`
    /// does this work. A user who does not want a book on the Home view had
    /// no way to take it away: the book stayed until they finished it.
    pub fn toggle_the_shelf_of_continue_listening(&mut self) {
        let Some((item_id, episode_id)) = self.selected_place() else {
            crate::logic::message::say(self.words_of_a_line_with_no_place());
            return;
        };

        if self.is_offline {
            crate::logic::message::say("The server does not answer.");
            return;
        }

        crate::logic::message::say("The change goes to the server…");
        let api = std::sync::Arc::clone(&self.api);

        tokio::spawn(async move {
            let text = hide_the_media(&api, &item_id, episode_id.as_deref()).await;

            crate::logic::message::say(text.as_str());
        });
    }

    /// Shows the chapters of the media that plays. See T-24.
    ///
    /// The engine holds the chapters already: it uses them for the keys `P`
    /// and `U`. The user could not see them, and they could not go to a
    /// chapter by its name.
    pub fn show_the_chapters(&mut self) {
        let state = self.player.state();

        // **The view opens for every answer.** A message that stands outside the
        // buffer of ratatui goes away with the next frame of a view that draws
        // that row. The user then presses a key and reads nothing. The view says
        // why it holds no line. See T-42, T-59, and T-134.
        //
        // The selection starts at the chapter that plays.
        let now = crate::logic::chapters::chapter_at(&state.chapters, state.position);

        self.list_state_chapters.select(Some(now.unwrap_or(0)));

        // The view opens with the media of this moment, therefore the program
        // reads the media that plays again. See T-162.
        self.the_media_of_the_view_of_the_chapters = None;

        self.scroll_offset = 0;
        self.view_state = AppView::Chapters;
    }

    /// Holds the media whose chapters the view shows, and it takes the line
    /// away when that media does not play.
    ///
    /// **The loop of the program calls this at each frame**, because the media
    /// that plays changes with no key of this user: the media comes to its end,
    /// and the queue starts the media of its front. The view then draws the
    /// chapters of another media and the line keeps the number of the line,
    /// therefore the key `l` seeks in a media that the user did not choose. The
    /// measurement of 2026-08-14 took a book of eight hours from 4:50:35 to
    /// 5:33:20, and the server holds that place. See T-162, and T-160 and T-161
    /// for the same rule of two other views.
    pub fn the_view_of_the_chapters_holds_its_media(&mut self) {
        if !matches!(self.view_state, AppView::Chapters) {
            self.the_media_of_the_view_of_the_chapters = None;
            return;
        }

        let state = self.player.state();

        // A playback that stopped is no media of a chapter.
        let of_the_player = if state.status == crate::player::engine::PlaybackStatus::Stopped {
            None
        } else {
            Some((state.playback_id, state.title.clone()))
        };

        let of_the_program = self
            .the_media_of_the_view_of_the_chapters
            .as_ref()
            .map(|(playback, _)| *playback);

        match crate::logic::chapters::what_the_media_of_the_chapters_is(
            of_the_program,
            of_the_player.as_ref().map(|(playback, _)| *playback),
        ) {
            // The media of the user plays, and the line stays with its chapter.
            crate::logic::chapters::TheMediaOfTheChapters::ItStillPlays => {}
            // **No key of the view may reach a media that the user did not
            // choose**, therefore the line goes to nobody and the program says
            // which media went away.
            crate::logic::chapters::TheMediaOfTheChapters::ItWentAway => {
                let title = self
                    .the_media_of_the_view_of_the_chapters
                    .take()
                    .map(|(_, title)| title)
                    .unwrap_or_default();

                self.list_state_chapters.select(None);

                crate::logic::message::say_in(
                    AppView::Chapters,
                    crate::logic::chapters::the_text_of_the_media_that_went_away(&title).as_str(),
                );
            }
            // The view opened, or the user chose a chapter of the media that
            // plays now with the keys j and k.
            crate::logic::chapters::TheMediaOfTheChapters::TheProgramReadsItAgain => {
                self.the_media_of_the_view_of_the_chapters = of_the_player;
            }
        }
    }

    /// Goes to the chapter that the user selected.
    pub fn go_to_the_chapter(&mut self) {
        let state = self.player.state();

        let Some(index) = self.list_state_chapters.selected() else {
            // **The media of the view can go away with no key of this user**
            // (T-162), and the line then stands on nobody. A key that does
            // nothing must say why (T-79).
            crate::logic::message::say("No line is selected.");
            return;
        };

        let Some(chapter) = state.chapters.get(index) else {
            return;
        };

        self.player
            .send(crate::player::engine::PlayerCommand::SeekTo(chapter.start));

        crate::logic::message::say(&format!("The playback goes to \"{}\".", chapter.title));
    }

    /// Tells the server to examine the library again. See T-24.
    ///
    /// A user who puts a file in the directory of the library must open the
    /// web page today. `POST /api/libraries/:id/scan` gives `200`, and the
    /// server then reads the directory.
    ///
    /// The scan runs on the server and it takes its own time. Therefore the
    /// program says that the work started, and the user presses `R` after a
    /// moment.
    pub fn scan_the_library(&mut self) {
        if self.is_offline {
            crate::logic::message::say("The server does not answer.");
            return;
        }

        if self.id_selected_lib.is_empty() {
            crate::logic::message::say("No library is selected.");
            return;
        }

        let api = std::sync::Arc::clone(&self.api);
        let library = self.id_selected_lib.clone();

        crate::logic::message::say("The server examines the library…");

        tokio::spawn(async move {
            let text = match api
                .post_no_content(
                    &format!("/api/libraries/{}/scan", library),
                    &serde_json::json!({}),
                )
                .await
            {
                Ok(()) => {
                    "The server examines the library now. Press R after a moment.".to_string()
                }
                Err(error) => format!("The server did not start the examination: {}", error),
            };

            crate::logic::message::say(text.as_str());
        });
    }

    /// Shows the authors of the library, and asks the server for them.
    ///
    /// A library of podcasts has no author. See T-24.
    pub fn show_the_authors(&mut self) {
        self.show_the_names(crate::logic::authors::Kind::Authors);
    }

    /// Shows the narrators of the library. See T-73.
    ///
    /// A narrator of the server holds the shape of an author, therefore one view
    /// and one function hold the two lists.
    pub fn show_the_narrators(&mut self) {
        self.show_the_names(crate::logic::authors::Kind::Narrators);
    }

    /// Asks the server for the next page of the library, when the user comes
    /// near the end of the lines that the program holds. See T-70.
    ///
    /// The program reads the first page at the start. Therefore the cost of the
    /// start is the same for a library of 12 items and for a library of 250000.
    /// **The search of the server stays the authority** for a title of a page
    /// that the program did not read.
    pub fn ask_for_the_next_page_of_the_library(&mut self) {
        if self.is_offline {
            return;
        }

        // The position of the item of the line that the user selected. A line
        // of a series holds the position of its first book.
        let selected = self
            .list_state_library
            .selected()
            .and_then(|line| self.library_rows.get(line))
            .map(|row| row.item())
            .unwrap_or(0);

        if !crate::logic::library_pages::wants_the_next_page(
            self.ids_library.len(),
            self.library_total,
            selected,
            crate::logic::library_pages::asks(),
        ) {
            return;
        }

        self.ask_for_one_page_now();
    }

    /// Asks the server for the page after the page that came last. See T-125.
    ///
    /// **The line of the user decides for the key that moves, and it decides
    /// for nothing else.** The key `G` and the search of a library of podcasts
    /// both need every page, and the line of the user says nothing of that
    /// need: this function holds the request itself, and
    /// `ask_for_the_next_page_of_the_library` holds the rule of the line.
    fn ask_for_one_page_now(&mut self) {
        if self.is_offline
            || crate::logic::library_pages::asks()
            || self.ids_library.len() >= self.library_total
        {
            return;
        }

        crate::logic::library_pages::keep_the_flag(true);

        let api = std::sync::Arc::clone(&self.api);
        let library = self.id_selected_lib.clone();
        let query = self.library_query.clone();
        let number = self.library_page + 1;

        tokio::spawn(async move {
            let answer = crate::api::libraries::get_all_books::get_one_page_of_books(
                &api, &library, &query, number,
            )
            .await;

            let root = match answer {
                Ok(root) => root,
                Err(error) => {
                    log::warn!("[library] the server gave no page {}: {}", number, error);
                    // **The key of the user said nothing at all**: the key `G`
                    // waited for the end of a library of 2056 items, 500 of
                    // them stood on the screen, and no word came. See T-168.
                    crate::logic::library_pages::keep_the_fault(error.to_string().as_str());
                    crate::logic::library_pages::keep_the_flag(false);
                    return;
                }
            };

            let count = root.results.as_ref().map(|all| all.len()).unwrap_or(0);
            log::info!("[library] the page {} gives {} item(s)", number, count);

            // The lists of the screen come from the task, because these seven
            // functions are asynchronous and the render is not.
            crate::logic::library_pages::keep(crate::logic::library_pages::Page {
                number,
                total: root.total.unwrap_or_default().max(0) as usize,
                titles: collect_titles_library(&root).await,
                ids: collect_ids_library(&root).await,
                authors: collect_auth_names_library(&root).await,
                authors_of_a_podcast: collect_auth_names_library_pod(&root).await,
                durations: collect_duration_library(&root).await,
                descriptions: collect_desc_library(&root).await,
                years: collect_published_year_library(&root).await,
            });

            crate::logic::library_pages::keep_the_flag(false);
        });
    }

    /// Puts the page that came in the lists of the library. See T-70.
    ///
    /// The render calls this at each frame, and it does the work one time: the
    /// box holds one page, and `take` leaves it empty. A page that is not the
    /// page after the page that came last goes away, because a new library or a
    /// new filter makes every page before it wrong.
    pub fn take_the_next_page_of_the_library(&mut self) {
        // **A page that did not come is the answer of a key of the user**, and
        // that key said nothing at all: the key `G` stopped inside a library of
        // 2056 items with no word, and the work of that key stood open for
        // ever. The answer of a key belongs to no view and it stands above them
        // all, therefore `say` writes it and not `say_in` (T-164). See T-168.
        if let Some(fault) = crate::logic::library_pages::take_the_fault() {
            self.reads_every_page_of_the_library = false;
            self.reads_the_pages_for_the_search = false;

            crate::logic::message::say(
                &crate::logic::library_pages::the_words_of_a_page_that_did_not_come(&fault),
            );
        }

        let Some(page) = crate::logic::library_pages::take() else {
            return;
        };

        if page.number != self.library_page + 1 {
            return;
        }

        self.library_page = page.number;
        self.library_total = page.total;

        // A library of podcasts holds one row of the episodes for each item,
        // and every row of a new page is empty: the program reads the episodes
        // of a podcast when the user opens it. See T-126.
        if self.is_podcast {
            for _ in 0..page.ids.len() {
                self.all_titles_pod_ep.push(Vec::new());
                self.all_ids_pod_ep.push(Vec::new());
                self.all_subtitles_pod_ep.push(Vec::new());
                self.all_seasons_pod_ep.push(Vec::new());
                self.all_episodes_pod_ep.push(Vec::new());
                self.all_authors_pod_ep.push(Vec::new());
                self.all_descs_pod_ep.push(Vec::new());
                self.all_titles_pod.push(Vec::new());
                self.all_durations_pod_ep.push(Vec::new());
                self.the_episodes_that_came.push(false);
            }
        }

        self.titles_library.extend(page.titles);
        self.ids_library.extend(page.ids);
        self.auth_names_library.extend(page.authors);
        self.auth_names_library_pod
            .extend(page.authors_of_a_podcast);
        self.duration_library.extend(page.durations);
        self.desc_library.extend(page.descriptions);
        self.published_year_library.extend(page.years);

        // **The lines of the view grow, and no line of them moves.** Every book
        // of a series gives one line, and the function reads the items in their
        // sequence: the lines of the pages before this one are the same lines.
        self.library_rows = group_library(&self.ids_library, &self.series);

        // The key `G` waits for the end of the library. The new lines stand
        // after the line of that key, therefore the key takes the last line
        // again. See T-112.
        if self.reads_every_page_of_the_library {
            self.take_the_last_line_of_the_library();

            if self.ids_library.len() >= self.library_total {
                self.reads_every_page_of_the_library = false;
            } else {
                // **No key comes between two pages.** The user pressed `G` one
                // time, therefore the page that is left needs no key at all.
                self.ask_for_one_page_now();
            }
        } else if self.reads_the_pages_for_the_search {
            // The search of a library of podcasts reads the pages that are
            // left, and it moves no line of the library. See T-125.
            if self.ids_library.len() >= self.library_total {
                self.reads_the_pages_for_the_search = false;
                log::info!(
                    "[search] the program holds every page of the library now: {} item(s)",
                    self.ids_library.len()
                );
            } else {
                self.ask_for_one_page_now();
            }
        }
    }

    /// Puts the episodes of the podcast of the line in the lists of the view,
    /// and it asks the server for them when the program does not hold them.
    /// See T-126.
    ///
    /// **The start of the program read the episodes of every podcast**, one
    /// request after the other: 500 requests of a library of 520 podcasts, and
    /// a first frame of 11.9 seconds with a server of 20 milliseconds. The
    /// program reads one podcast now, and it reads it when the user opens that
    /// podcast.
    pub fn take_the_episodes_of_the_line(&mut self, place: usize) {
        self.titles_pod_ep = self
            .all_titles_pod_ep
            .get(place)
            .cloned()
            .unwrap_or_default();
        self.subtitles_pod_ep = self
            .all_subtitles_pod_ep
            .get(place)
            .cloned()
            .unwrap_or_default();
        self.seasons_pod_ep = self
            .all_seasons_pod_ep
            .get(place)
            .cloned()
            .unwrap_or_default();
        self.episodes_pod_ep = self
            .all_episodes_pod_ep
            .get(place)
            .cloned()
            .unwrap_or_default();
        self.authors_pod_ep = self
            .all_authors_pod_ep
            .get(place)
            .cloned()
            .unwrap_or_default();
        self.descs_pod_ep = self
            .all_descs_pod_ep
            .get(place)
            .cloned()
            .unwrap_or_default();
        self.titles_pod = self.all_titles_pod.get(place).cloned().unwrap_or_default();
        self.durations_pod_ep = self
            .all_durations_pod_ep
            .get(place)
            .cloned()
            .unwrap_or_default();

        self.ask_the_server_for_the_episodes(place);
    }

    /// Asks the server for the episodes of one podcast. See T-126.
    ///
    /// The program asks one time for each podcast: the answer stays in the
    /// lists of the library, therefore a second visit needs no request. A
    /// podcast of no episode gives an empty answer, and the flag of that
    /// podcast then says that the program asked.
    pub fn ask_the_server_for_the_episodes(&mut self, place: usize) {
        if self.is_offline || crate::logic::the_episodes::asks() {
            return;
        }

        if self
            .the_episodes_that_came
            .get(place)
            .copied()
            .unwrap_or(false)
        {
            return;
        }

        let Some(id) = self.ids_library.get(place).cloned() else {
            return;
        };

        crate::logic::the_episodes::keep_the_flag(true);

        // A request of this podcast runs now. The fault of a request before it
        // is not the truth of this moment. See T-168.
        crate::logic::the_episodes::forget_the_fault_of(place);

        let api = std::sync::Arc::clone(&self.api);

        tokio::spawn(async move {
            let answer = get_pod_ep(&api, id.as_str()).await;

            let podcast = match answer {
                Ok(podcast) => podcast,
                Err(error) => {
                    log::warn!(
                        "[podcast] the server gave no episode of the podcast {}: {}",
                        id,
                        error
                    );
                    // **The view said nothing of this fault**: it told the user
                    // that the program gets the episodes, and no episode ever
                    // came. See T-168.
                    crate::logic::the_episodes::keep_the_fault(place, error.to_string().as_str());
                    crate::logic::the_episodes::keep_the_flag(false);
                    return;
                }
            };

            crate::logic::the_episodes::keep(crate::logic::the_episodes::Episodes {
                place,
                id,
                titles: collect_titles_pod_ep(&podcast).await,
                ids: collect_ids_pod_ep(&podcast).await,
                subtitles: collect_subtitles_pod_ep(&podcast).await,
                seasons: collect_seasons_pod_ep(&podcast).await,
                numbers: collect_episodes_pod_ep(&podcast).await,
                authors: collect_authors_pod_ep(&podcast).await,
                descriptions: collect_descs_pod_ep(&podcast).await,
                titles_of_the_podcast: collect_titles_pod(&podcast).await,
                durations: collect_durations_pod_ep(&podcast).await,
            });

            crate::logic::the_episodes::keep_the_flag(false);
        });
    }

    /// Puts the episodes that came in the lists of the library. See T-126.
    ///
    /// The render calls this at each frame, and it does the work one time. The
    /// user stands in the view of the episodes of that podcast, therefore the
    /// lists of the view take the answer too.
    pub fn take_the_episodes_that_came(&mut self) {
        let Some(episodes) = crate::logic::the_episodes::take() else {
            return;
        };

        let place = episodes.place;

        // A new library or a new filter moves every line. The answer of a
        // podcast that stands at a different place now goes away.
        if self.ids_library.get(place) != Some(&episodes.id) {
            return;
        }

        let keep = |list: &mut Vec<Vec<String>>, values: Vec<String>| {
            if let Some(row) = list.get_mut(place) {
                *row = values;
            }
        };

        keep(&mut self.all_titles_pod_ep, episodes.titles.clone());
        keep(&mut self.all_ids_pod_ep, episodes.ids.clone());
        keep(&mut self.all_subtitles_pod_ep, episodes.subtitles.clone());
        keep(&mut self.all_seasons_pod_ep, episodes.seasons.clone());
        keep(&mut self.all_episodes_pod_ep, episodes.numbers.clone());
        keep(&mut self.all_authors_pod_ep, episodes.authors.clone());
        keep(&mut self.all_descs_pod_ep, episodes.descriptions.clone());
        keep(
            &mut self.all_titles_pod,
            episodes.titles_of_the_podcast.clone(),
        );
        keep(&mut self.all_durations_pod_ep, episodes.durations.clone());

        if let Some(flag) = self.the_episodes_that_came.get_mut(place) {
            *flag = true;
        }

        // The user waits in the view of the episodes of that podcast. The view
        // of the search holds its own lists, and the line of the search names
        // the same podcast.
        if !matches!(self.view_state, AppView::PodcastEpisode) {
            return;
        }

        if self.is_from_search_pod {
            let of_the_line = self
                .list_state_search_results
                .selected()
                .and_then(|line| self.ids_library_pod_search.get(line));

            if of_the_line != Some(&episodes.id) {
                return;
            }

            self.titles_pod_ep_search = episodes.titles;
            self.ids_pod_ep_search = episodes.ids;
            self.subtitles_pod_ep_search = episodes.subtitles;
            self.seasons_pod_ep_search = episodes.seasons;
            self.episodes_pod_ep_search = episodes.numbers;
            self.authors_pod_ep_search = episodes.authors;
            self.descs_pod_ep_search = episodes.descriptions;
            self.titles_pod_search = episodes.titles_of_the_podcast;
            self.durations_pod_ep_search = episodes.durations;
            return;
        }

        self.titles_pod_ep = episodes.titles;
        self.ids_pod_ep = episodes.ids;
        self.subtitles_pod_ep = episodes.subtitles;
        self.seasons_pod_ep = episodes.seasons;
        self.episodes_pod_ep = episodes.numbers;
        self.authors_pod_ep = episodes.authors;
        self.descs_pod_ep = episodes.descriptions;
        self.titles_pod = episodes.titles_of_the_podcast;
        self.durations_pod_ep = episodes.durations;
    }

    /// The search found a podcast of a page that the program did not read.
    /// See T-125.
    ///
    /// **The lists of the episodes of a podcast come from the place of that
    /// media in the lists of the library** (T-113), therefore a podcast of a
    /// page that the program did not read gave no line at all: a library of 520
    /// podcasts said "The server found nothing" for a podcast that the server
    /// found. The program reads the pages that are left now, and the line comes
    /// with its episodes. The user asked for that media, therefore the cost of
    /// the requests is theirs (T-112).
    pub fn the_search_reads_the_pages_that_are_left(&mut self) {
        if self.is_offline
            || self.reads_the_pages_for_the_search
            || self.ids_library.len() >= self.library_total
        {
            return;
        }

        log::info!(
            "[search] the server found a podcast of a page that the program did not read. \
             The program reads the {} item(s) that are left.",
            self.library_total - self.ids_library.len()
        );

        self.reads_the_pages_for_the_search = true;
        self.ask_for_one_page_now();
    }

    /// Takes the last line of the Library view.
    ///
    /// The key `G` and the page that comes after that key both need it. See
    /// T-112.
    fn take_the_last_line_of_the_library(&mut self) {
        let last_line = self.library_rows.len().saturating_sub(1);
        self.list_state_library.select(Some(last_line));
    }

    /// Shows the authors of the library, or its narrators. See T-24 and T-73.
    fn show_the_names(&mut self, kind: crate::logic::authors::Kind) {
        if !matches!(
            self.view_state,
            AppView::Home | AppView::Library | AppView::SearchBook | AppView::Authors
        ) {
            return;
        }

        if self.is_podcast {
            crate::logic::message::say(&kind.message_of_a_library_of_podcasts());
            return;
        }

        // A new list forgets the answer of the list that came before it.
        crate::logic::authors::keep_the_kind(kind);

        self.list_state_authors.select(Some(0));
        self.scroll_offset = 0;
        self.view_state = AppView::Authors;

        if self.is_offline {
            crate::logic::authors::keep(crate::logic::authors::State::Fault(
                "the server does not answer".to_string(),
            ));
            return;
        }

        // The names of a library do not change while the program runs.
        // Therefore the program asks one time, and the key `R` asks again.
        if !matches!(
            crate::logic::authors::state(),
            crate::logic::authors::State::Nothing
        ) {
            return;
        }

        crate::logic::authors::keep(crate::logic::authors::State::Waiting);

        let api = std::sync::Arc::clone(&self.api);
        let library = self.id_selected_lib.clone();

        tokio::spawn(async move {
            let answer = match kind {
                crate::logic::authors::Kind::Authors => {
                    crate::api::libraries::get_authors::get_authors(&api, &library).await
                }
                crate::logic::authors::Kind::Narrators => {
                    crate::api::libraries::get_authors::get_narrators(&api, &library).await
                }
            };

            let state = match answer {
                Ok(all) => {
                    log::info!(
                        "[authors] the server gave {} name(s) of {:?}",
                        all.len(),
                        kind
                    );
                    crate::logic::authors::State::Ready(all)
                }
                Err(error) => {
                    log::warn!("[authors] the server gave no name of {:?}: {}", kind, error);
                    crate::logic::authors::State::Fault(error.to_string())
                }
            };

            crate::logic::authors::keep(state);
        });
    }

    /// Shows the books of the author that the user selected. See T-24.
    ///
    /// The work is the filter of the library, and the program holds it
    /// already. The library then shows the books of that author, and the key
    /// `f` takes the filter away.
    pub fn show_the_books_of_the_author(&mut self) {
        let all = crate::logic::authors::authors();

        let Some(author) = self
            .list_state_authors
            .selected()
            .and_then(|index| all.get(index))
        else {
            return;
        };

        let of_the_old = self.the_sequence_of_the_library();

        self.library_filter = crate::logic::authors::kind().filter_of(author);

        if !self.the_disk_takes_the_sequence_of_the_library(of_the_old) {
            return;
        }

        self.must_refresh = true;
    }

    /// The sequence, the direction, and the filter of the library of this
    /// account. See T-205.
    fn the_sequence_of_the_library(&self) -> (String, bool, String) {
        (
            self.library_sort.clone(),
            self.library_desc,
            self.library_filter.clone(),
        )
    }

    /// Writes the sequence and the filter of the library of this account, and it
    /// gives `false` when the disk did not take them. See T-205.
    ///
    /// **A write of the disk that failed is no new sequence.** The old line was
    /// `let _ = update_library_sort(...)`: a database that a second Toutui of
    /// this account held (T-140) took nothing, the program then asked the server
    /// for the library again, and `App::new` read the sequence of the row of
    /// before. The user read a list of the sequence of before with no word at
    /// all, and a key of the user that writes the disk takes a sentence (T-199).
    ///
    /// The values of the application go back to the values of the disk, because
    /// the row of the account is the truth of this sequence: a screen that holds
    /// a sequence that no row holds says the sequence of nobody.
    fn the_disk_takes_the_sequence_of_the_library(
        &mut self,
        of_the_old: (String, bool, String),
    ) -> bool {
        let of_the_disk = crate::db::crud::update_library_sort(
            &self.username,
            &self.library_sort,
            self.library_desc,
            &self.library_filter,
        );

        if let Err(error) = of_the_disk {
            log::error!(
                "[the sequence of the library] the program did not write the sequence of {}: {}",
                self.username,
                error
            );

            self.library_sort = of_the_old.0;
            self.library_desc = of_the_old.1;
            self.library_filter = of_the_old.2;

            crate::logic::message::say(crate::ui::keys::THE_SEQUENCE_DID_NOT_REACH_THE_DISK);

            return false;
        }

        true
    }

    /// Shows the lists that can take the media of the line. See T-84.
    ///
    /// The program read the collections and the playlists, and it changed none
    /// of them: a user who wanted a book in a playlist opened the web page of
    /// the server.
    pub fn show_the_lists_that_take_the_media(&mut self) {
        if !matches!(
            self.view_state,
            AppView::Home
                | AppView::Library
                | AppView::SearchBook
                | AppView::SeriesBook
                | AppView::ListEntries
                | AppView::PodcastEpisode
        ) {
            return;
        }

        // `selected_download` names the media of every view that holds one, and
        // it names the episode of a podcast. A line of a podcast itself holds no
        // media for this work: a list of the server takes a book or an episode.
        let Some((target, title, _author)) = self.selected_download() else {
            crate::logic::message::say("This line holds no book and no episode.");
            return;
        };

        let item_id = target.item_id().to_string();

        // An episode of a podcast stands in a playlist, and never in a
        // collection: a collection holds books. See T-84.
        let episode_id = match &target {
            crate::logic::download::DownloadTarget::Episode { episode_id, .. } => {
                Some(episode_id.clone())
            }
            crate::logic::download::DownloadTarget::Book { .. } => None,
        };

        // A library that holds no list opens this view too: the keys `c` and
        // `p` make the first list of that library. See T-88. The view held a
        // message of one row before that work, and that message said "The web
        // page of the server makes one".
        self.the_media_of_the_list = Some((item_id, episode_id, title));
        self.the_view_before_the_list = self.view_state;
        self.list_state_put_in_a_list.select(Some(0));
        self.view_state = AppView::PutInAList;
    }

    /// Puts the media in the list of the line. See T-84.
    pub fn put_the_media_in_the_list(&mut self) {
        let Some((item_id, episode_id, title)) = self.the_media_of_the_list.clone() else {
            return;
        };

        let Some(list) = self
            .list_state_put_in_a_list
            .selected()
            .and_then(|line| self.lists.get(line))
            .cloned()
        else {
            return;
        };

        // A collection holds books. The server refuses an episode, therefore
        // the program says it before the request.
        if episode_id.is_some()
            && matches!(
                list.kind,
                crate::api::utils::collect_lists::ListKind::Collection
            )
        {
            crate::logic::message::say(
                "A collection holds books only. Take a playlist for an episode of a podcast.",
            );
            return;
        }

        let api = std::sync::Arc::clone(&self.api);
        let kind = list.kind;
        let list_id = list.id.clone();
        let name = list.name.clone();
        let library = self.id_selected_lib.clone();

        self.view_state = self.the_view_before_the_list;

        tokio::spawn(async move {
            let text = match crate::api::lists::put_in_the_list(
                &api,
                kind,
                &list_id,
                &item_id,
                episode_id.as_deref(),
            )
            .await
            {
                Ok(came) => crate::api::lists::the_sentence_of_the_work(kind, &name, &title, came),
                Err(error) => format!("The server did not take the media: {}", error),
            };

            crate::logic::message::say(&text);

            // The lines of the screen come after the write. A question that
            // goes with the write gives the list of the moment before it.
            crate::logic::the_lists::ask(&api, &library).await;
        });
    }

    /// Shows the devices of an e-reader that can take the book of the line.
    /// See T-119.
    ///
    /// **The program asks the server at this key.** `POST /api/authorize` gives
    /// the devices of this account, and the server filters that list itself:
    /// `GET /api/emails/settings` answers `404` for an account that is not an
    /// administrator, and it can therefore never give the list to a user.
    pub fn show_the_devices_of_an_ereader(&mut self) {
        if !matches!(
            self.view_state,
            AppView::Home
                | AppView::Library
                | AppView::SearchBook
                | AppView::SeriesBook
                | AppView::ListEntries
                | AppView::PodcastEpisode
        ) {
            return;
        }

        let Some((target, title, _author)) = self.selected_download() else {
            crate::logic::message::say("This line holds no book.");
            return;
        };

        // **The server sends `media.ebookFile`, and a podcast holds none.** The
        // endpoint answers `404` with "Ebook file not found" for an episode, and
        // the program says the reason before the request.
        let item_id = match &target {
            crate::logic::download::DownloadTarget::Book { item_id, .. } => item_id.clone(),
            crate::logic::download::DownloadTarget::Episode { .. } => {
                crate::logic::message::say(
                    "An episode of a podcast holds no ebook. The server sends a book.",
                );
                return;
            }
        };

        // A device stands on the server, and the list comes from the server.
        // A view must not give a reason that the program does not have (T-91),
        // and this program has this one.
        if self.is_offline {
            crate::logic::message::say(
                "The server does not answer. The devices of an e-reader stand on the server.",
            );
            return;
        }

        self.the_book_of_the_send = Some((item_id, title));
        self.the_view_before_the_send = self.view_state;
        self.list_state_send_to_ereader.select(Some(0));
        self.view_state = AppView::SendToEreader;

        self.ask_for_the_devices_of_an_ereader();
    }

    /// Asks the server for the devices of this account. See T-119.
    pub fn ask_for_the_devices_of_an_ereader(&mut self) {
        let api = std::sync::Arc::clone(&self.api);

        crate::logic::the_ereaders::ask();

        tokio::spawn(async move {
            let state = match crate::api::ereaders::the_devices_of_the_account(&api).await {
                Ok(all) => crate::logic::the_ereaders::State::Ready(all),
                Err(error) => crate::logic::the_ereaders::State::Fault(error.to_string()),
            };

            crate::logic::the_ereaders::keep(state);
        });
    }

    /// Sends the book to the device of the line. See T-119.
    ///
    /// **The whole work stands on the server**: it reads the file, it makes the
    /// e-mail, and it gives the bytes to an SMTP server. A book of 480
    /// megabytes took 36 seconds of the sandbox, therefore the request holds a
    /// time limit of its own (`THE_TIME_OF_A_SEND`) and this program says that
    /// a big book takes some minutes.
    pub fn send_the_book_to_an_ereader(&mut self) {
        let Some((item_id, title)) = self.the_book_of_the_send.clone() else {
            return;
        };

        // **The footer names the key `l`, therefore that key must say
        // something.** A key that does nothing is a fault of its own (T-79),
        // and this view holds no line while the program waits for the server or
        // while the server holds no device.
        let Some(device) = self
            .list_state_send_to_ereader
            .selected()
            .and_then(|line| crate::logic::the_ereaders::devices().get(line).cloned())
        else {
            let reason = match crate::logic::the_ereaders::state() {
                crate::logic::the_ereaders::State::Nothing
                | crate::logic::the_ereaders::State::Waiting => {
                    "The program waits for the devices of the server.".to_string()
                }
                crate::logic::the_ereaders::State::Fault(reason) => {
                    format!("The server gave no device: {}", reason)
                }
                crate::logic::the_ereaders::State::Ready(_) => {
                    "The server holds no device for an e-reader. An administrator of the \
                     server adds one."
                        .to_string()
                }
            };

            crate::logic::message::say(&reason);
            return;
        };

        let api = std::sync::Arc::clone(&self.api);
        let name = device.name.clone();

        self.view_state = self.the_view_before_the_send;

        crate::logic::message::say(&format!(
            "The server sends \"{}\" to {}. A big book takes some minutes.",
            title, name
        ));

        tokio::spawn(async move {
            let text = match crate::api::ereaders::send_the_ebook(&api, &item_id, &name).await {
                Ok(end) => crate::api::ereaders::the_sentence_of_the_send(&title, &name, &end),
                Err(error) => format!("The server did not send \"{}\": {}", title, error),
            };

            crate::logic::message::say(&text);
        });
    }

    /// Makes a collection or a playlist, and it puts the media in it.
    /// See T-88.
    ///
    /// **The server refuses a collection with no book**, therefore this work
    /// starts from a media: the key `m` names one, and the keys `c` and `p` of
    /// that view make the list.
    pub fn make_a_new_list(&mut self, kind: crate::api::utils::collect_lists::ListKind) {
        let Some((item_id, episode_id, title)) = self.the_media_of_the_list.clone() else {
            return;
        };

        // A list stands on the server, therefore a server that does not answer
        // takes no new list. The program says it before it asks for a name.
        // See T-91.
        if self.is_offline {
            crate::logic::message::say(
                "The server does not answer. A collection and a playlist stand on the server.",
            );
            return;
        }

        // A collection holds books. The server refuses an episode, therefore
        // the program says it before it asks for a name.
        if episode_id.is_some()
            && matches!(kind, crate::api::utils::collect_lists::ListKind::Collection)
        {
            crate::logic::message::say(
                "A collection holds books only. Make a playlist for an episode of a podcast.",
            );
            return;
        }

        let question = format!(
            "The name of the new {} (Enter, or Esc)",
            kind.name().to_lowercase()
        );

        let Ok(Some(name)) = self.ask_for_a_text(&question) else {
            return;
        };

        let name = name.trim().to_string();

        // The server answers 400 for a name of no letter. The program says the
        // reason, and it makes no request.
        if name.is_empty() {
            crate::logic::message::say("A collection and a playlist need a name.");
            return;
        }

        // The server takes two lists of one name, and the user cannot tell the
        // two lines apart. See T-88.
        if crate::api::lists::a_list_holds_that_name(&self.lists, kind, &name) {
            crate::logic::message::say(&crate::api::lists::the_sentence_of_the_name_that_exists(
                kind, &name,
            ));
            return;
        }

        let api = std::sync::Arc::clone(&self.api);
        let library = self.id_selected_lib.clone();

        self.view_state = self.the_view_before_the_list;

        tokio::spawn(async move {
            let text = match crate::api::lists::make_the_list(
                &api,
                kind,
                &library,
                &name,
                &item_id,
                episode_id.as_deref(),
            )
            .await
            {
                Ok(_) => crate::api::lists::the_sentence_of_the_new_list(kind, &name, &title),
                Err(error) => format!("The server did not make the list: {}", error),
            };

            crate::logic::message::say(&text);

            // The lines of the screen come after the write. See T-84.
            crate::logic::the_lists::ask(&api, &library).await;
        });
    }

    /// Removes the collection or the playlist of the line. See T-93.
    ///
    /// **The program asks one time.** Every user of the server sees a
    /// collection, therefore a key that removes one by mistake takes it away
    /// from every one of them. The question names the kind of the list and the
    /// number of its media.
    pub fn remove_the_list_of_the_line(&mut self) {
        let Some(list) = self.selected_list().cloned() else {
            crate::logic::message::say("No list is selected.");
            return;
        };

        // A list stands on the server. See T-91.
        if self.is_offline {
            crate::logic::message::say(
                "The server does not answer. A collection and a playlist stand on the server.",
            );
            return;
        }

        if self.confirm_the_removal_of_the_list.as_deref() != Some(list.id.as_str()) {
            self.confirm_the_removal_of_the_list = Some(list.id.clone());

            crate::logic::message::say(&crate::api::lists::the_question_of_the_removal(
                list.kind,
                &list.name,
                list.entries.len(),
            ));

            return;
        }

        self.confirm_the_removal_of_the_list = None;

        let api = std::sync::Arc::clone(&self.api);
        let kind = list.kind;
        let list_id = list.id.clone();
        let name = list.name.clone();
        let library = self.id_selected_lib.clone();

        // The line of the list goes away, therefore the line of the user must
        // hold a list that stays: the list below it, or the list above it when
        // this one is the last. `take_the_lists` then follows that list to its
        // place in the answer of the server, and it says nothing at all — the
        // user pressed the key that removed this list, and the answer of that
        // key names it already. See T-165.
        let count = self.lists.len();

        if let Some(line) = self.list_state_lists.selected() {
            let next = if line + 1 < count {
                Some(line + 1)
            } else {
                line.checked_sub(1)
            };

            self.list_state_lists.select(next);
        }

        tokio::spawn(async move {
            let text = match crate::api::lists::remove_the_list(&api, kind, &list_id).await {
                Ok(()) => crate::api::lists::the_sentence_of_the_removal(kind, &name),
                Err(error) => format!("The server did not remove the list: {}", error),
            };

            crate::logic::message::say(&text);

            // The lines of the screen come after the write. See T-84.
            crate::logic::the_lists::ask(&api, &library).await;
        });
    }

    /// Gives the collection or the playlist of the line a new name. See T-93.
    ///
    /// **The server does not examine the name of this request**, and it does
    /// examine it when it makes a list: a `PATCH` of a collection with a name of
    /// no letter gives a collection with no name. Therefore the program holds
    /// the two rules of the name here, and they are the rules of T-88.
    pub fn give_the_list_of_the_line_a_new_name(&mut self) {
        let Some(list) = self.selected_list().cloned() else {
            crate::logic::message::say("No list is selected.");
            return;
        };

        if self.is_offline {
            crate::logic::message::say(
                "The server does not answer. A collection and a playlist stand on the server.",
            );
            return;
        }

        let question = format!(
            "The new name of the {} \"{}\" (Enter, or Esc)",
            list.kind.name().to_lowercase(),
            list.name
        );

        let Ok(Some(name)) = self.ask_for_a_text(&question) else {
            return;
        };

        let name = name.trim().to_string();

        if name.is_empty() {
            crate::logic::message::say("A collection and a playlist need a name.");
            return;
        }

        // The list keeps its own name, therefore a name that this list holds
        // already is not a name of a different list.
        if crate::api::lists::a_different_list_holds_that_name(
            &self.lists,
            list.kind,
            &name,
            &list.id,
        ) {
            crate::logic::message::say(&crate::api::lists::the_sentence_of_the_name_that_exists(
                list.kind, &name,
            ));
            return;
        }

        if name == list.name {
            return;
        }

        let api = std::sync::Arc::clone(&self.api);
        let kind = list.kind;
        let list_id = list.id.clone();
        let old = list.name.clone();
        let library = self.id_selected_lib.clone();

        tokio::spawn(async move {
            let text = match crate::api::lists::give_the_list_a_new_name(
                &api, kind, &list_id, &name,
            )
            .await
            {
                Ok(()) => crate::api::lists::the_sentence_of_the_new_name(kind, &old, &name),
                Err(error) => format!("The server did not take the new name: {}", error),
            };

            crate::logic::message::say(&text);

            crate::logic::the_lists::ask(&api, &library).await;
        });
    }

    /// Gives the collection or the playlist of the line a new description.
    /// See T-100.
    ///
    /// **A description of no letter takes the description away.** The server
    /// takes that value, therefore the program needs no rule of its own here:
    /// a list with no description is a list that the user made that way.
    pub fn give_the_list_of_the_line_a_new_description(&mut self) {
        let Some(list) = self.selected_list().cloned() else {
            crate::logic::message::say("No list is selected.");
            return;
        };

        if self.is_offline {
            crate::logic::message::say(
                "The server does not answer. A collection and a playlist stand on the server.",
            );
            return;
        }

        let question = format!(
            "The description of the {} \"{}\" (Enter, or Esc)",
            list.kind.name().to_lowercase(),
            list.name
        );

        let Ok(Some(description)) = self.ask_for_a_text(&question) else {
            return;
        };

        let description = description.trim().to_string();

        if description == list.description {
            return;
        }

        let api = std::sync::Arc::clone(&self.api);
        let kind = list.kind;
        let list_id = list.id.clone();
        let name = list.name.clone();
        let library = self.id_selected_lib.clone();
        let gone = description.is_empty();

        tokio::spawn(async move {
            let text = match crate::api::lists::give_the_list_a_new_description(
                &api,
                kind,
                &list_id,
                &description,
            )
            .await
            {
                Ok(()) => crate::api::lists::the_sentence_of_the_new_description(kind, &name, gone),
                Err(error) => format!("The server did not take the description: {}", error),
            };

            crate::logic::message::say(&text);

            crate::logic::the_lists::ask(&api, &library).await;
        });
    }

    /// Takes the media of the line out of the list that holds it. See T-84.
    pub fn take_the_media_out_of_the_list(&mut self) {
        let Some(list) = self.selected_list().cloned() else {
            crate::logic::message::say("No list is selected.");
            return;
        };

        let Some(entry) = self.selected_list_entry().cloned() else {
            crate::logic::message::say("No media is selected.");
            return;
        };

        let api = std::sync::Arc::clone(&self.api);
        let kind = list.kind;
        let list_id = list.id.clone();
        let name = list.name.clone();
        let title = entry.title.clone();
        let item_id = entry.id.clone();
        let episode_id = entry.episode_id.clone();
        let library = self.id_selected_lib.clone();

        tokio::spawn(async move {
            let text = match crate::api::lists::take_out_of_the_list(
                &api,
                kind,
                &list_id,
                &item_id,
                episode_id.as_deref(),
            )
            .await
            {
                Ok(()) => format!(
                    "\"{}\" is not in the {} \"{}\" now.",
                    title,
                    kind.name().to_lowercase(),
                    name
                ),
                Err(error) => format!("The server did not take the media out: {}", error),
            };

            crate::logic::message::say(&text);

            // The lines of the screen come after the write.
            crate::logic::the_lists::ask(&api, &library).await;
        });
    }

    /// Moves the media of the line one place up or down inside its collection or
    /// its playlist. See T-102.
    ///
    /// **The screen holds the new sequence before the answer of the server.** The
    /// user presses the key more than one time to move a media some lines, and a
    /// screen that waits for the server between two keys moves the wrong line.
    /// The request goes with every media of the list, and the task asks the
    /// server for the lists after the write: an answer that differs takes the
    /// place of the sequence of the screen.
    pub fn move_the_media_of_the_list(&mut self, down: bool) {
        let Some(place) = self.list_state_lists.selected() else {
            return;
        };

        let Some(line) = self.list_state_list_entries.selected() else {
            return;
        };

        let Some(list) = self.lists.get(place) else {
            return;
        };

        let Some(moved) = crate::api::lists::the_sequence_that_moved(&list.entries, line, down)
        else {
            crate::logic::message::say(&crate::api::lists::the_sentence_of_a_line_that_stays(down));
            return;
        };

        let kind = list.kind;
        let list_id = list.id.clone();
        let name = list.name.clone();
        let to = if down { line + 1 } else { line - 1 };
        let title = moved[to].title.clone();

        // The screen takes the new sequence now, and the selection follows the
        // media that moved.
        self.lists[place].entries = moved.clone();
        self.list_state_list_entries.select(Some(to));

        let api = std::sync::Arc::clone(&self.api);
        let library = self.id_selected_lib.clone();

        tokio::spawn(async move {
            let text =
                match crate::api::lists::give_the_list_a_new_sequence(&api, kind, &list_id, &moved)
                    .await
                {
                    Ok(()) => {
                        crate::api::lists::the_sentence_of_the_new_sequence(kind, &name, &title, to)
                    }
                    Err(error) => format!("The server did not take the new sequence: {}", error),
                };

            crate::logic::message::say(&text);

            // The lines of the screen come after the write. See the trap 40.
            crate::logic::the_lists::ask(&api, &library).await;
        });
    }

    /// Asks the server for the collections and the playlists again. See T-84.
    ///
    /// The program changed a list of the server, therefore the lines of the
    /// screen are old. The task writes the answer in `logic::the_lists`, and
    /// the render takes it at the next frame.
    pub fn ask_for_the_lists(&mut self) {
        let api = std::sync::Arc::clone(&self.api);
        let library = self.id_selected_lib.clone();

        tokio::spawn(async move {
            crate::logic::the_lists::ask(&api, &library).await;
        });
    }

    /// Takes the lists that the task asked for, if they came. See T-84.
    ///
    /// The render calls this at each frame.
    ///
    /// **The line of the user holds a list, and not a number of a line**: this
    /// function is the one door of a change of `self.lists`, and a list that a
    /// second program of the account removed moved the list below it under
    /// that line with no word at all. See T-165, and the same rule of T-160,
    /// of T-161, of T-162, and of T-163.
    pub fn take_the_lists(&mut self) {
        let Some(lists) = crate::logic::the_lists::take() else {
            return;
        };

        let the_list_of_the_line = self
            .selected_list()
            .map(|list| (list.id.clone(), list.kind, list.name.clone()));

        self.lists = lists;

        match crate::logic::the_lists::what_the_line_of_the_lists_holds(
            the_list_of_the_line.as_ref().map(|(id, _, _)| id.as_str()),
            &self.lists,
        ) {
            crate::logic::the_lists::TheLineOfTheLists::TheSameList(place) => {
                self.list_state_lists.select(Some(place));
            }
            crate::logic::the_lists::TheLineOfTheLists::NoLine => {}
            crate::logic::the_lists::TheLineOfTheLists::ThatListWentAway => {
                let Some((_, kind, name)) = the_list_of_the_line else {
                    return;
                };

                self.list_state_lists.select(None);
                self.list_state_list_entries.select(None);

                // A rule of the loop writes this message with no key of the
                // user, therefore it belongs to the view of the lists and to
                // no other view. See T-164.
                let text = if matches!(self.view_state, AppView::ListEntries) {
                    self.view_state = AppView::Lists;
                    crate::logic::the_lists::the_text_of_the_media_of_a_list_that_went_away(
                        kind, &name,
                    )
                } else {
                    crate::logic::the_lists::the_text_of_the_list_that_went_away(kind, &name)
                };

                crate::logic::message::say_in(AppView::Lists, &text);

                return;
            }
        }

        // The list of the line can hold fewer media than it held before, and
        // the selection must stay inside it. See T-41.
        let count = self
            .selected_list()
            .map(|list| list.entries.len())
            .unwrap_or(0);

        if count == 0 {
            self.list_state_list_entries.select(None);
        } else if self.list_state_list_entries.selected().unwrap_or(0) >= count {
            self.list_state_list_entries.select(Some(count - 1));
        }
    }

    /// Shows the episodes that the server downloads, and the queue. See T-81.
    ///
    /// The key `E` gives the server the episodes of a feed that it does not
    /// hold, and the server does that work alone. **The program showed nothing
    /// of it before this view**: a user who pressed `E` on a feed of 57 episodes
    /// read one message and no more.
    pub fn show_the_downloads_of_the_server(&mut self) {
        if !matches!(
            self.view_state,
            AppView::Home | AppView::Library | AppView::SearchBook | AppView::PodcastEpisode
        ) {
            return;
        }

        if !self.is_podcast {
            crate::logic::message::say(
                "This library holds books. The server downloads the episodes of a podcast only.",
            );
            return;
        }

        crate::logic::the_downloads::forget();

        // **The view opens before the answer of the server comes**, therefore
        // it opens with no line at all: a line of the open stands on nothing,
        // and the queue of the server can be empty. The first list that comes
        // gives the line. See T-166.
        self.list_state_downloads.select(None);
        self.the_episode_of_the_line_of_the_downloads = None;
        self.the_downloads_gave_the_first_line = false;
        self.confirm_the_empty_queue = None;
        self.view_state = AppView::Downloads;
    }

    /// Asks the server for the queue of the downloads of the library.
    ///
    /// The render calls this at the first frame of the view, at each message of
    /// the server, and after the time of `logic::the_downloads`. See T-81.
    pub fn ask_for_the_downloads(&mut self) {
        let api = std::sync::Arc::clone(&self.api);
        let library = self.id_selected_lib.clone();

        if !matches!(
            crate::logic::the_downloads::state(),
            crate::logic::the_downloads::State::Ready(_)
        ) {
            crate::logic::the_downloads::keep(crate::logic::the_downloads::State::Waiting);
        }

        tokio::spawn(async move {
            let state = match crate::api::podcasts::the_downloads::the_downloads_of_the_library(
                &api, &library,
            )
            .await
            {
                Ok(all) => crate::logic::the_downloads::State::Ready(all),
                Err(error) => crate::logic::the_downloads::State::Fault(error.to_string()),
            };

            crate::logic::the_downloads::keep(state);
        });
    }

    /// Holds the episode that the user chose in the view of the downloads, and
    /// it takes the line away when that episode leaves the queue of the server.
    ///
    /// **The loop of the program calls this at each frame**, because that queue
    /// changes with no key of any user: the server takes an episode out when it
    /// downloaded it, and a second program of the library empties the queue.
    /// The lines kept the number of the line, therefore an episode that the
    /// user did not choose moved under the cursor with no word at all — the key
    /// `X` then named the podcast of that episode, and the two presses emptied
    /// a queue that the user never chose. See T-166, and the same rule of
    /// T-161 for the queue of the media.
    pub fn the_line_of_the_downloads_holds_its_episode(&mut self) {
        if !matches!(self.view_state, AppView::Downloads) {
            self.the_episode_of_the_line_of_the_downloads = None;
            return;
        }

        let all = crate::logic::the_downloads::downloads();

        let of_the_user = self.list_state_downloads.selected();
        let of_the_program = self
            .the_episode_of_the_line_of_the_downloads
            .as_ref()
            .map(|(line, key, _, _)| (*line, key.as_str()));

        match crate::logic::the_downloads::what_the_line_of_the_downloads_holds(
            &all,
            of_the_program,
            of_the_user,
        ) {
            // The episode of the user stands in the queue, and the cursor goes
            // with it.
            crate::logic::the_downloads::TheLineOfTheDownloads::ItStandsAt(place) => {
                self.list_state_downloads.select(Some(place));

                if let Some(held) = self.the_episode_of_the_line_of_the_downloads.as_mut() {
                    held.0 = place;
                }
            }
            // **No key may reach an episode that the user did not choose**,
            // therefore the line goes to nobody and the program says which
            // episode went away.
            crate::logic::the_downloads::TheLineOfTheDownloads::ItWentAway => {
                let (_, _, title, podcast) = self
                    .the_episode_of_the_line_of_the_downloads
                    .take()
                    .unwrap_or_default();

                self.list_state_downloads.select(None);

                // The mark of the confirmation goes away with the line: the
                // second press of the key `X` must reach no podcast at all.
                self.confirm_the_empty_queue = None;

                // A rule of the loop writes this message with no key of the
                // user, therefore it belongs to the view of the downloads and
                // to no other view. See T-164.
                crate::logic::message::say_in(
                    AppView::Downloads,
                    crate::logic::the_downloads::the_text_of_the_episode_that_went_away(
                        &title, &podcast,
                    )
                    .as_str(),
                );
            }
            // The user moved the cursor, and that key is their choice.
            crate::logic::the_downloads::TheLineOfTheDownloads::TheUserChoseAnother => {
                // The first list of the server gives the line 0. A line that
                // went to nobody after it stays with nobody.
                let of_the_user = match of_the_user {
                    Some(line) => Some(line),
                    None if !all.is_empty() && !self.the_downloads_gave_the_first_line => Some(0),
                    None => None,
                };

                if !all.is_empty() {
                    self.the_downloads_gave_the_first_line = true;
                }

                self.the_episode_of_the_line_of_the_downloads = of_the_user.and_then(|line| {
                    all.get(line)
                        .map(|one| (line, one.key(), one.title.clone(), one.podcast.clone()))
                });

                // The queue of the server can be empty, and a line that reaches
                // no episode is a line of nobody.
                self.list_state_downloads.select(
                    self.the_episode_of_the_line_of_the_downloads
                        .as_ref()
                        .map(|(line, _, _, _)| *line),
                );
            }
        }
    }

    /// Empties the queue of the podcast of the line. See T-81.
    ///
    /// **The program asks one time.** The queue holds the work of the server,
    /// and a key that stops it by mistake costs the user every episode of that
    /// queue. The key `E` gives them back.
    pub fn empty_the_queue_of_the_downloads(&mut self) {
        let all = crate::logic::the_downloads::downloads();

        let Some(one) = self
            .list_state_downloads
            .selected()
            .and_then(|line| all.get(line))
            .cloned()
        else {
            // **The episode of the line can leave the queue with no key of any
            // user** (T-166), and the queue of the server can be empty. The
            // line then stands on nobody, and a key that does nothing must say
            // why (T-79). The footer promises this key (T-143).
            crate::logic::message::say("No episode is selected.");
            return;
        };

        if self.confirm_the_empty_queue.as_deref() != Some(one.item_id.as_str()) {
            self.confirm_the_empty_queue = Some(one.item_id.clone());

            crate::logic::message::say(&format!(
                "Press X again to empty the queue of \"{}\". Any other key stops this.",
                one.podcast
            ));

            return;
        }

        self.confirm_the_empty_queue = None;

        let api = std::sync::Arc::clone(&self.api);
        let podcast = one.podcast.clone();
        let item_id = one.item_id.clone();

        tokio::spawn(async move {
            let text =
                match crate::api::podcasts::the_downloads::empty_the_queue(&api, &item_id).await {
                    Ok(()) => format!(
                        "The queue of \"{}\" is empty now. The episode that downloads goes on.",
                        podcast
                    ),
                    Err(error) => format!("The server did not empty the queue: {}", error),
                };

            crate::logic::message::say(&text);
            crate::logic::the_downloads::note_that_the_queue_changed();
        });
    }

    /// Tells the server to get the episodes that it does not hold. See T-24.
    ///
    /// The key `D` copies a media to the disk of the user. This key is a
    /// different work: the server gets the file and it puts it in the library
    /// of the server, therefore every client can play it.
    pub fn get_the_new_episodes(&mut self) {
        if !self.is_podcast {
            crate::logic::message::say("This library holds books.");
            return;
        }

        if self.is_offline {
            crate::logic::message::say("The server does not answer.");
            return;
        }

        // The view of the episodes belongs to one podcast, and the Library
        // view gives the podcast of the line.
        let item_id = match self.view_state {
            AppView::PodcastEpisode => self.podcast_of_the_episodes(),
            _ => self.selected_item_id(),
        };

        let Some(item_id) = item_id else {
            crate::logic::message::say("No podcast is selected.");
            return;
        };

        let api = std::sync::Arc::clone(&self.api);
        crate::logic::message::say("The server reads the feed…");

        tokio::spawn(async move {
            let text = ask_the_server_for_the_episodes(&api, &item_id).await;

            crate::logic::message::say(text.as_str());
        });
    }

    /// Gives the identity of the podcast whose episodes the view shows.
    fn podcast_of_the_episodes(&self) -> Option<String> {
        let ids = if self.is_from_search_pod {
            &self.ids_library_pod_search
        } else {
            &self.ids_library
        };

        let index = if self.is_from_search_pod {
            self.list_state_search_results.selected()
        } else {
            self.selected_library_item()
        };

        index.and_then(|index| ids.get(index)).cloned()
    }

    /// Looks for a new podcast, and shows what the server found. See T-24.
    ///
    /// The key operates in a library of podcasts only: a library of books
    /// cannot hold a podcast. The server asks iTunes, therefore the search
    /// needs the network of the server.
    pub fn look_for_a_podcast(&mut self) {
        if !matches!(
            self.view_state,
            AppView::Home | AppView::Library | AppView::SearchBook | AppView::NewPodcast
        ) {
            return;
        }

        if !self.is_podcast {
            crate::logic::message::say(
                "This library holds books. Choose a library of podcasts with S.",
            );
            return;
        }

        if self.is_offline {
            crate::logic::message::say("The server does not answer.");
            return;
        }

        let Ok(Some(words)) = self.ask_for_a_text("The name of the podcast (Enter, or Esc)") else {
            return;
        };

        if words.trim().is_empty() {
            return;
        }

        crate::logic::new_podcast::keep(crate::logic::new_podcast::State::Waiting);
        self.list_state_new_podcast.select(Some(0));
        self.scroll_offset = 0;
        self.view_state = AppView::NewPodcast;

        let api = std::sync::Arc::clone(&self.api);
        let words = words.trim().to_string();

        tokio::spawn(async move {
            let state = match crate::api::podcasts::search_podcast(&api, &words).await {
                Ok(all) => crate::logic::new_podcast::State::Ready(all),
                Err(error) => {
                    log::warn!("[podcast] the server found nothing: {}", error);
                    crate::logic::new_podcast::State::Fault(error.to_string())
                }
            };

            crate::logic::new_podcast::keep(state);
        });
    }

    /// Writes the podcast that the user selected in the library. See T-24.
    ///
    /// The work needs two requests: the server reads the feed, and the server
    /// then writes the podcast. **This request changes the library of the
    /// server**, therefore the program asks the user one time before it
    /// sends.
    pub fn add_the_podcast(&mut self) {
        let all = crate::logic::new_podcast::found();

        let Some(found) = self
            .list_state_new_podcast
            .selected()
            .and_then(|index| all.get(index))
            .cloned()
        else {
            return;
        };

        if found.feed_url.is_empty() {
            crate::logic::message::say("This answer of the server holds no feed.");
            return;
        }

        // The request writes a new directory in the library of the server.
        // The user says yes one time.
        let question = format!(
            "Add \"{}\" to the library? Write yes, and then Enter.",
            found.title
        );

        let Ok(Some(answer)) = self.ask_for_a_text(&question) else {
            return;
        };

        if answer.trim().to_lowercase() != "yes" {
            crate::logic::message::say("The program added no podcast.");
            return;
        }

        let api = std::sync::Arc::clone(&self.api);
        let library = self.id_selected_lib.clone();
        let feed_url = found.feed_url.clone();
        let title = found.title.clone();

        crate::logic::message::say("The server reads the feed…");

        tokio::spawn(async move {
            let text = add_a_podcast(&api, &library, &feed_url, &title).await;

            crate::logic::message::say(text.as_str());
        });
    }

    /// Moves the timer for sleep to its next choice. See T-24.
    ///
    /// The key gives 5, 10, 15, 30, 45, and 60 minutes, the end of the
    /// chapter, and then off. The volume falls in the last 30 seconds, and
    /// the playback then pauses.
    pub fn change_the_timer_for_sleep(&mut self) {
        use crate::logic::sleep_timer as sleep;

        let state = self.player.state();

        if state.status == crate::player::engine::PlaybackStatus::Stopped {
            crate::logic::message::say("No media plays now.");
            return;
        }

        self.sleep_choice = sleep::next_choice(self.sleep_choice);

        // The volume of the user comes from the state, and not from a timer
        // that falls already. A second press during the fall would keep the
        // small volume for ever.
        let volume = match &self.sleep {
            Some(timer) => timer.volume,
            None => state.volume,
        };

        let Some(choice) = self.sleep_choice else {
            self.stop_the_timer_for_sleep(volume);
            crate::logic::message::say("The timer for sleep is off.");
            return;
        };

        let wait = if choice == 0 {
            // The end of the chapter. The book plays at a speed, therefore
            // the time of the clock is not the time of the book.
            let end = crate::logic::chapters::chapter_at(&state.chapters, state.position)
                .and_then(|index| state.chapters.get(index))
                .map(|chapter| chapter.end - state.position);

            match end {
                Some(seconds) => sleep::clock_time_of(seconds, state.speed),
                None => {
                    self.sleep_choice = None;
                    self.stop_the_timer_for_sleep(volume);
                    crate::logic::message::say("This media has no chapter.");
                    return;
                }
            }
        } else {
            std::time::Duration::from_secs(choice * 60)
        };

        self.sleep = Some(sleep::Timer {
            ends_at: std::time::Instant::now() + wait,
            volume,
            playback_id: state.playback_id,
            label: sleep::label_of(choice),
        });

        // A press during the fall must give the volume of the user back.
        self.player
            .send(crate::player::engine::PlayerCommand::SetVolume(volume));

        crate::logic::message::say(&format!(
            "The playback stops after {}.",
            sleep::label_of(choice)
        ));
    }

    /// Stops the timer, and gives the volume of the user back.
    fn stop_the_timer_for_sleep(&mut self, volume: f32) {
        if self.sleep.take().is_some() {
            self.player
                .send(crate::player::engine::PlayerCommand::SetVolume(volume));
        }
    }

    /// Does the work of the timer for sleep. The loop of the program calls
    /// this at each frame. See T-24.
    pub fn tick_the_timer_for_sleep(&mut self) {
        use crate::logic::sleep_timer as sleep;

        let Some(timer) = self.sleep else {
            return;
        };

        let state = self.player.state();

        match sleep::action_for(
            &timer,
            state.status,
            state.playback_id,
            std::time::Instant::now(),
        ) {
            sleep::Action::Nothing => {}
            sleep::Action::Off(volume) => {
                self.sleep_choice = None;
                self.stop_the_timer_for_sleep(volume);
            }
            sleep::Action::Volume(value) => {
                self.player
                    .send(crate::player::engine::PlayerCommand::SetVolume(value));
            }
            sleep::Action::Sleep(volume) => {
                self.player
                    .send(crate::player::engine::PlayerCommand::Pause);
                self.sleep_choice = None;
                self.stop_the_timer_for_sleep(volume);

                crate::logic::message::say("The timer for sleep stopped the playback.");
            }
        }
    }

    /// Gives the text of the timer for the player, if a timer runs.
    pub fn text_of_the_timer_for_sleep(&self) -> Option<String> {
        self.sleep
            .as_ref()
            .map(|timer| crate::logic::sleep_timer::text_of(timer, std::time::Instant::now()))
    }

    /// Writes a bookmark at the place of the playback. See T-24.
    ///
    /// The key operates while a media plays: a bookmark holds a place, and a
    /// media that does not play has no place. The program asks the user for a
    /// name, and an empty name gives the name of the place.
    pub fn write_a_bookmark(&mut self) {
        let state = self.player.state();
        let plays = state.status != crate::player::engine::PlaybackStatus::Stopped;

        // **The view of the bookmarks holds the media that the user opened**,
        // and the media that plays changes with no key of the user: the queue
        // starts the media of its front (T-24). A bookmark of this view
        // therefore belongs to the media of this view, and no other media.
        // See T-163, and T-160, T-161, and T-162 for three other views.
        if matches!(self.view_state, AppView::Bookmarks) && !self.bookmarks_of.is_empty() {
            let of_the_player = if plays {
                Some(state.item_id.as_str())
            } else {
                None
            };

            if crate::logic::bookmarks::what_the_media_of_the_bookmarks_is(
                &self.bookmarks_of,
                of_the_player,
            ) == crate::logic::bookmarks::TheMediaOfTheBookmarks::ItDoesNotPlay
            {
                crate::logic::message::say(
                    &crate::logic::bookmarks::the_text_of_the_media_that_does_not_play(
                        &self.bookmarks_of_name,
                    ),
                );
                return;
            }
        }

        if !plays {
            crate::logic::message::say("No media plays now.");
            return;
        }

        if self.is_offline {
            crate::logic::message::say("The server does not answer.");
            return;
        }

        let place = state.position;
        let item_id = state.item_id.clone();

        // The user presses Esc. The program then writes nothing.
        let Ok(Some(name)) = self.ask_for_a_text("The name of the bookmark (Enter, or Esc)") else {
            return;
        };

        let name = if name.trim().is_empty() {
            crate::api::me::bookmarks::default_title(place)
        } else {
            name.trim().to_string()
        };

        let api = std::sync::Arc::clone(&self.api);
        crate::logic::message::say("The bookmark goes to the server…");

        tokio::spawn(async move {
            let text =
                match crate::api::me::bookmarks::add_bookmark(&api, &item_id, place, &name).await {
                    Ok(()) => format!("The bookmark \"{}\" is on the server.", name),
                    Err(error) => format!("The server did not take the bookmark: {}", error),
                };

            // A bookmark that came now must stand in the view.
            crate::logic::bookmarks::forget();

            crate::logic::message::say(text.as_str());
        });
    }

    /// Shows the bookmarks of a media, and asks the server for them.
    ///
    /// The media that plays comes first, because a user who listens looks for
    /// a place of that media. A media that plays no media gives the media of
    /// the line that the user selected.
    pub fn show_the_bookmarks(&mut self) {
        let state = self.player.state();

        let (item_id, name) = if state.status != crate::player::engine::PlaybackStatus::Stopped {
            (state.item_id.clone(), state.title.clone())
        } else {
            match self.selected_item_id() {
                Some(id) => (id, self.selected_item_title().unwrap_or_default()),
                None => {
                    crate::logic::message::say("No media plays, and no media is selected.");
                    return;
                }
            }
        };

        self.bookmarks_of = item_id.clone();
        // The title of the view names this media, and the key `b` of this view
        // writes a place of this media alone: the media that plays changes with
        // no key of the user. See T-163.
        self.bookmarks_of_name = name;
        self.list_state_bookmarks.select(Some(0));
        self.scroll_offset = 0;
        self.view_state = AppView::Bookmarks;

        if self.is_offline {
            crate::logic::bookmarks::keep(crate::logic::bookmarks::State::Fault(
                "the server does not answer".to_string(),
            ));
            return;
        }

        self.ask_the_server_for_the_bookmarks(item_id);
    }

    /// Asks the server for the bookmarks of one media.
    fn ask_the_server_for_the_bookmarks(&self, item_id: String) {
        crate::logic::bookmarks::keep(crate::logic::bookmarks::State::Waiting);

        let api = std::sync::Arc::clone(&self.api);

        tokio::spawn(async move {
            let state = match crate::api::me::bookmarks::get_bookmarks(&api).await {
                Ok(all) => crate::logic::bookmarks::State::Ready(
                    crate::api::me::bookmarks::of_item(&all, &item_id),
                ),
                Err(error) => {
                    log::warn!("[bookmarks] the server gave no bookmark: {}", error);
                    crate::logic::bookmarks::State::Fault(error.to_string())
                }
            };

            crate::logic::bookmarks::keep(state);
        });
    }

    /// Goes to the place of the bookmark that the user selected.
    pub fn go_to_the_bookmark(&mut self) {
        let all = crate::logic::bookmarks::bookmarks();

        let Some(bookmark) = self
            .list_state_bookmarks
            .selected()
            .and_then(|index| all.get(index))
        else {
            return;
        };

        let state = self.player.state();

        // The bookmark belongs to a media that does not play. The engine
        // cannot go to a place of a media that it does not hold.
        if state.status == crate::player::engine::PlaybackStatus::Stopped
            || state.item_id != bookmark.library_item_id
        {
            crate::logic::message::say(
                "Play this media first, and the bookmark then gives its place.",
            );
            return;
        }

        self.player
            .send(crate::player::engine::PlayerCommand::SeekTo(bookmark.time));

        crate::logic::message::say(&format!("The playback goes to \"{}\".", bookmark.title));
    }

    /// Removes the bookmark that the user selected. See T-24.
    pub fn remove_the_bookmark(&mut self) {
        let all = crate::logic::bookmarks::bookmarks();

        let Some(bookmark) = self
            .list_state_bookmarks
            .selected()
            .and_then(|index| all.get(index))
            .cloned()
        else {
            return;
        };

        if self.is_offline {
            crate::logic::message::say("The server does not answer.");
            return;
        }

        let api = std::sync::Arc::clone(&self.api);
        let item_id = bookmark.library_item_id.clone();
        let name = bookmark.title.clone();
        let time = bookmark.time;

        crate::logic::message::say("The program removes the bookmark…");

        tokio::spawn(async move {
            let text = match crate::api::me::bookmarks::remove_bookmark(&api, &item_id, time).await
            {
                Ok(()) => format!("The bookmark \"{}\" is not on the server now.", name),
                Err(error) => format!("The server did not remove the bookmark: {}", error),
            };

            crate::logic::bookmarks::forget();

            crate::logic::message::say(text.as_str());
        });
    }

    /// Asks the server for the bookmarks again, if the view lost them.
    ///
    /// A write and a remove forget the answer. The render then calls this,
    /// and the new list comes at the frame after it.
    pub fn take_the_bookmarks_again(&mut self) {
        if !matches!(self.view_state, AppView::Bookmarks) || self.is_offline {
            return;
        }

        if matches!(
            crate::logic::bookmarks::state(),
            crate::logic::bookmarks::State::Nothing
        ) && !self.bookmarks_of.is_empty()
        {
            self.ask_the_server_for_the_bookmarks(self.bookmarks_of.clone());
        }
    }

    /// Shows the time that the user listened, and asks the server for it.
    ///
    /// The request goes at every press of the key. The numbers change while
    /// the user listens, therefore an old answer would be wrong. See T-24.
    pub fn show_the_statistics(&mut self) {
        self.stats_scroll = 0;
        self.view_state = AppView::Stats;

        if self.is_offline {
            crate::logic::stats::keep(crate::logic::stats::State::Fault(
                "The server does not answer. The program works with the disk only.".to_string(),
            ));
            return;
        }

        crate::logic::stats::keep(crate::logic::stats::State::Waiting);

        let api = std::sync::Arc::clone(&self.api);
        let library = self.id_selected_lib.clone();
        let library_name = self.library_name.clone();

        tokio::spawn(async move {
            let year_number = crate::api::stats::this_year();

            // The three requests go together. They ask three different paths,
            // therefore the view waits for the slowest one and not for the sum
            // of the three.
            let (listening, of_the_library, of_the_year) = tokio::join!(
                crate::api::me::listening_stats::get_listening_stats(&api),
                async {
                    if library.is_empty() {
                        return None;
                    }
                    crate::api::stats::get_library_stats(&api, &library)
                        .await
                        .ok()
                },
                async {
                    crate::api::stats::get_year_stats(&api, year_number)
                        .await
                        .ok()
                },
            );

            // The time of the user is the important answer. A fault of one of
            // the two other requests takes its group away only: a user with no
            // permission for the statistics of the year keeps the rest of the
            // view. See `logic::stats::Statistics`.
            let state = match listening {
                Ok(listening) => {
                    if of_the_library.is_none() && !library.is_empty() {
                        log::warn!("[stats] the server gave no number for the library {library}");
                    }
                    if of_the_year.is_none() {
                        log::warn!("[stats] the server gave no number for the year {year_number}");
                    }
                    crate::logic::stats::State::Ready(Box::new(crate::logic::stats::Statistics {
                        listening,
                        library: of_the_library,
                        library_name,
                        year: of_the_year,
                        year_number,
                    }))
                }
                Err(error) => {
                    log::warn!("[stats] the server gave no statistics: {}", error);
                    crate::logic::stats::State::Fault(error.to_string())
                }
            };

            crate::logic::stats::keep(state);
        });
    }

    /// Shows every session of the user, and asks the server for the first page.
    ///
    /// The view of the key `T` shows the five last sessions. This view shows
    /// the whole history, and it reads the next page when the user comes near
    /// the end. See T-24.
    pub fn show_the_sessions(&mut self) {
        self.sessions_scroll = 0;
        self.view_state = AppView::Sessions;

        if self.is_offline {
            crate::logic::sessions_view::keep(crate::logic::sessions_view::State::Fault(
                "The server does not answer. The program works with the disk only.".to_string(),
            ));
            return;
        }

        crate::logic::sessions_view::keep(crate::logic::sessions_view::State::Waiting);

        let api = std::sync::Arc::clone(&self.api);

        tokio::spawn(async move {
            let per_page = crate::api::me::sessions::PER_PAGE;
            let state = match crate::api::me::sessions::get_sessions(&api, 0, per_page).await {
                Ok(page) => crate::logic::sessions_view::State::Ready(Box::new(
                    crate::logic::sessions_view::Loaded::first(page),
                )),
                Err(error) => {
                    log::warn!("[sessions] the server gave no session: {}", error);
                    crate::logic::sessions_view::State::Fault(error.to_string())
                }
            };

            crate::logic::sessions_view::keep(state);
        });
    }

    /// Asks the server for the next page of the sessions, if that is necessary.
    ///
    /// The move calls this at each step down. `a_task_asks` gives `true` one
    /// time only for one page, therefore a user who holds the key `j` makes one
    /// request and not fifty.
    pub fn read_the_next_page_of_the_sessions(&mut self) {
        if self.is_offline {
            return;
        }

        let state = crate::logic::sessions_view::state();
        let crate::logic::sessions_view::State::Ready(loaded) = &state else {
            return;
        };

        let lines = usize::from(self.sessions_scroll_max) + 1;
        if !loaded.wants_the_next_page(usize::from(self.sessions_scroll), lines) {
            return;
        }

        // The mark stops a second task for the same page.
        if !crate::logic::sessions_view::a_task_asks() {
            return;
        }

        let api = std::sync::Arc::clone(&self.api);
        let next = loaded.page + 1;

        tokio::spawn(async move {
            let per_page = crate::api::me::sessions::PER_PAGE;
            match crate::api::me::sessions::get_sessions(&api, next, per_page).await {
                Ok(page) => crate::logic::sessions_view::add_a_page(page),
                Err(error) => {
                    log::warn!("[sessions] the page {} did not come: {}", next, error);
                    crate::logic::sessions_view::the_page_did_not_come();
                }
            }
        });
    }

    /// Gives the lines of the view of the sequence and of the filter.
    ///
    /// The function is cheap: it makes about twenty lines from a list that
    /// the program holds. The key handler and the render both call it, thus
    /// the program keeps no list that could disagree with the screen.
    pub fn sort_filter_rows(&self) -> Vec<crate::logic::sort_filter::Row> {
        use crate::logic::sort_filter::from_the_server::State;

        let (filters, note) = match crate::logic::sort_filter::from_the_server::state() {
            State::Ready(filters) => (filters, None),
            State::Waiting => (
                Vec::new(),
                Some("The program asks the server for the authors and the series…".to_string()),
            ),
            State::Fault(text) => (
                Vec::new(),
                Some(format!("The server gave no author and no series: {}", text)),
            ),
            State::Nothing => (Vec::new(), None),
        };

        crate::logic::sort_filter::rows(self.is_podcast, &filters, note)
    }

    /// Gives one value for each line of that view: `true` for a line that the
    /// user can select.
    fn lines_of_the_sort_filter_view(&self) -> Vec<bool> {
        self.sort_filter_rows()
            .iter()
            .map(|row| row.is_a_line_of_the_user())
            .collect()
    }

    /// Shows the sequence and the filter of the library, and asks the server
    /// for the authors and the series. See T-24.
    pub fn show_the_sequence_and_the_filter(&mut self) {
        if !matches!(
            self.view_state,
            AppView::Home | AppView::Library | AppView::SearchBook | AppView::SortFilter
        ) {
            return;
        }

        self.view_state = AppView::SortFilter;
        self.scroll_offset = 0;

        let lines = self.lines_of_the_sort_filter_view();
        self.list_state_sort_filter
            .select(crate::logic::list_moves::first(&lines));

        // The offline mode holds the media of the disk, and no request goes
        // to the server. The sequence of the server then changes nothing.
        if self.is_offline {
            crate::logic::sort_filter::from_the_server::keep(
                crate::logic::sort_filter::from_the_server::State::Fault(
                    "the server does not answer".to_string(),
                ),
            );
            return;
        }

        // The answer of a request before this one is still correct, because
        // the library did not change. The program then asks one time.
        if !matches!(
            crate::logic::sort_filter::from_the_server::state(),
            crate::logic::sort_filter::from_the_server::State::Nothing
        ) {
            return;
        }

        crate::logic::sort_filter::from_the_server::keep(
            crate::logic::sort_filter::from_the_server::State::Waiting,
        );

        let api = std::sync::Arc::clone(&self.api);
        let library = self.id_selected_lib.clone();

        tokio::spawn(async move {
            let state =
                match crate::api::libraries::get_filter_data::get_filter_data(&api, &library).await
                {
                    Ok(data) => {
                        // The answer of `filterdata` holds no tag. Therefore the
                        // program asks `GET /api/tags` and it puts those tags in
                        // the same list. See T-60.
                        let tags = crate::api::libraries::get_filter_data::get_the_tags(&api).await;
                        let data =
                            crate::api::libraries::get_filter_data::with_the_tags(data, tags);

                        crate::logic::sort_filter::from_the_server::State::Ready(
                            crate::api::libraries::get_filter_data::choices(&data),
                        )
                    }
                    Err(error) => {
                        log::warn!("[sort] the server gave no filter data: {}", error);
                        crate::logic::sort_filter::from_the_server::State::Fault(error.to_string())
                    }
                };

            crate::logic::sort_filter::from_the_server::keep(state);
        });
    }

    /// Takes the choice of the user of the view of the sequence.
    ///
    /// The sequence and the filter belong to the request of the items, and
    /// every list of the library comes from that request. Therefore the
    /// program makes the application again, in the same way as the key `R`.
    pub fn apply_the_sequence_or_the_filter(&mut self) {
        use crate::logic::sort_filter::Row;

        let Some(index) = self.list_state_sort_filter.selected() else {
            return;
        };

        let rows = self.sort_filter_rows();

        let Some(row) = rows.get(index) else {
            return;
        };

        let of_the_old = self.the_sequence_of_the_library();

        match row {
            Row::Title(_) | Row::Note(_) => return,
            Row::Sort { field, .. } => {
                // The same field a second time changes the direction. The
                // user then needs one key for "the newest first".
                if self.library_sort == *field {
                    self.library_desc = !self.library_desc;
                } else {
                    self.library_sort = field.clone();
                }
            }
            Row::Direction => self.library_desc = !self.library_desc,
            Row::NoFilter => self.library_filter = String::new(),
            Row::Filter { value, .. } => {
                // The same filter a second time removes it.
                if self.library_filter == *value {
                    self.library_filter = String::new();
                } else {
                    self.library_filter = value.clone();
                }
            }
        }

        if !self.the_disk_takes_the_sequence_of_the_library(of_the_old) {
            return;
        }

        self.must_refresh = true;
    }

    /// Asks the server for the media that agree with the words of the user.
    ///
    /// The render is not asynchronous, therefore a task asks and the render
    /// takes the answer at the next frame. See T-24.
    pub fn ask_the_server_to_search(&mut self) {
        crate::logic::search::from_the_server::forget();

        if self.is_offline || self.search_query.trim().is_empty() {
            return;
        }

        let api = std::sync::Arc::clone(&self.api);
        let library = self.id_selected_lib.clone();
        let words = self.search_query.clone();

        tokio::spawn(async move {
            let answer =
                match crate::api::libraries::search_library::search_library(&api, &library, &words)
                    .await
                {
                    Ok(answer) => answer,
                    Err(error) => {
                        log::warn!("[search] the server did not search: {}", error);
                        return;
                    }
                };

            let mut items = crate::api::libraries::search_library::media_of(&answer);

            // **The group of the books of the server holds no name of an author.**
            // A user who writes the name of an author therefore saw the name in
            // the header and no book at all. The filter of the library gives the
            // books of that name. See T-70.
            for named in crate::api::libraries::search_library::the_names_to_ask(&answer) {
                match crate::api::libraries::get_all_books::get_all_books(
                    &api,
                    &library,
                    &named.query,
                )
                .await
                {
                    Ok(root) => {
                        let found = root.results.unwrap_or_default();

                        log::info!(
                            "[search] the name \"{}\" gives {} book(s) of the library",
                            named.name,
                            found.len()
                        );

                        for item in found {
                            let Some(id) = item.id.clone() else {
                                continue;
                            };

                            if items.iter().any(|one| one.id.as_deref() == Some(&id)) {
                                continue;
                            }

                            items.push(item);
                        }
                    }
                    Err(error) => log::warn!(
                        "[search] the server gave no book of the name \"{}\": {}",
                        named.name,
                        error
                    ),
                }
            }

            crate::logic::search::from_the_server::keep(
                crate::logic::search::from_the_server::Answer {
                    words,
                    // **Every value of a line comes from the answer**, therefore
                    // the view shows a media of a page that the program did not
                    // read. See T-113.
                    media: crate::logic::search::the_media_that_the_server_found(&items),
                    names: crate::api::libraries::search_library::names_of(&answer),
                },
            );
        });
    }

    /// Gives the identity of the item that the user selected, in any view of
    /// media.
    /// Gives the title of the media that the user selected.
    ///
    /// A PDF holds no title in most files, and the name of the file on the disk
    /// is the identity of the item. Therefore the reader takes the title of the
    /// server for such a book. See T-54.
    pub fn selected_item_title(&self) -> Option<String> {
        match self.view_state {
            AppView::Home => self
                .selected_home_item()
                .and_then(|index| self._titles_cnt_list.get(index))
                .cloned(),
            AppView::Library => self
                .selected_library_item()
                .and_then(|index| self.titles_library.get(index))
                .cloned(),
            // **The view of the search holds the titles of its lines now.** The
            // answer of the server carries the title of every media (T-113), and
            // the reader of a PDF said the identity of the item before: a
            // measurement of 2026-08-12 read
            // "27c55369-b048-4d68-9e70-17653b4d618f — page 1 of 150". See T-117
            // and T-54.
            AppView::SearchBook => self
                .list_state_search_results
                .selected()
                .and_then(|line| self.titles_search_book.get(line))
                .cloned(),
            AppView::SeriesBook => self.selected_series_book().map(|book| book.title.clone()),
            AppView::ListEntries => self.selected_list_entry().map(|entry| entry.title.clone()),
            _ => None,
        }
    }

    pub fn selected_item_id(&self) -> Option<String> {
        match self.view_state {
            AppView::Home => self
                .selected_home_item()
                .and_then(|index| self._ids_cnt_list.get(index))
                .cloned(),
            AppView::Library => self
                .selected_library_item()
                .and_then(|index| self.ids_library.get(index))
                .cloned(),
            AppView::SearchBook => self
                .list_state_search_results
                .selected()
                .and_then(|index| self.ids_search_book.get(index))
                .cloned(),
            AppView::SeriesBook => self.selected_series_book().map(|book| book.id.clone()),
            AppView::ListEntries => self.selected_list_entry().map(|entry| entry.id.clone()),
            _ => None,
        }
    }

    /// Shows the list of the ebooks of the media that the reader holds.
    ///
    /// **An item can hold more than one ebook.** The key `e` opens the book of
    /// `media.ebookFile`, and the key `e` inside the reader gives the list of
    /// every book of that media. See T-76.
    pub fn show_the_ebooks_of_the_media(&mut self) {
        let Some(item_id) = self.reader.as_ref().map(|reader| reader.item_id.clone()) else {
            return;
        };

        crate::logic::the_ebooks::ask_for(&item_id);
        self.list_state_ebooks.select(Some(0));
        self.view_state = AppView::Ebooks;

        let api = std::sync::Arc::clone(&self.api);

        tokio::spawn(async move {
            let state =
                match crate::api::library_items::the_ebooks::the_ebooks_of_the_item(&api, &item_id)
                    .await
                {
                    Ok(all) => crate::logic::the_ebooks::State::Ready(all),
                    Err(error) => crate::logic::the_ebooks::State::Fault(error.to_string()),
                };

            crate::logic::the_ebooks::keep(&item_id, state);
        });
    }

    /// Opens the ebook of the line that the user selected. See T-76.
    pub fn open_the_ebook_of_the_line(&mut self) {
        let all = crate::logic::the_ebooks::ebooks();

        let Some(one) = self
            .list_state_ebooks
            .selected()
            .and_then(|line| all.get(line))
            .cloned()
        else {
            return;
        };

        let item_id = crate::logic::the_ebooks::item_id();

        if item_id.is_empty() {
            return;
        }

        let title = self
            .reader
            .as_ref()
            .map(|reader| reader.title.clone())
            .filter(|title| !title.trim().is_empty());

        // The book of the server keeps the shape of T-10: the place of the user
        // goes to the server. Every other book keeps its place on this machine,
        // because the server holds one place for each media.
        let ino = if one.is_the_book_of_the_server {
            None
        } else {
            Some(one.ino.clone())
        };

        self.get_the_book(item_id, title, ino);
    }

    /// The lines of the view of the accounts. See T-124.
    ///
    /// One line for each account of the database. The account that starts the
    /// program holds the mark, and the address of the server stands beside the
    /// name: two accounts of one name on two servers are two accounts.
    /// Reads the accounts of the database again. See T-155.
    ///
    /// **The list of the accounts came of `App::new` alone**, therefore the view
    /// of a window that stands open showed an account that a second program of
    /// this account removed, and it hid an account that a second program added.
    /// The disk is the truth, and the program reads it at the moment of the use:
    /// that is the rule of T-142, of T-147, and of T-148.
    ///
    /// The line of the user keeps its place when the list holds it still, and it
    /// goes to the last line of the list when the list became shorter.
    pub fn the_accounts_come_from_the_disk(&mut self) {
        // **A read that failed is not a database with no account** (T-199). The
        // list of this window stays on both roads, and a fault takes a line of
        // the log: the two conditions are not one, and the log of the
        // maintainer must say which of them came.
        let of_the_disk = match crate::db::crud::select_every_usr() {
            Ok(rows) => rows,
            Err(error) => {
                log::error!(
                    "[the accounts] the program did not read the accounts of the disk: {}. The \
                     lines of this view stay.",
                    error
                );

                return;
            }
        };

        if of_the_disk.is_empty() {
            // A database of no account is the database of a login that runs. The
            // list of this window then says more than the disk does, and no key
            // of this view needs a list of no line.
            return;
        }

        self.the_accounts = of_the_disk;

        let last = self.the_accounts.len() - 1;
        match self.list_state_settings_account.selected() {
            Some(line) if line > last => self.list_state_settings_account.select(Some(last)),
            Some(_) => {}
            None => self.list_state_settings_account.select(Some(0)),
        }
    }

    pub fn the_lines_of_the_accounts(&self) -> Vec<String> {
        self.the_accounts
            .iter()
            .map(|(name, address, starts)| {
                crate::logic::the_accounts::the_line_of_an_account(name, address, *starts)
            })
            .collect()
    }

    /// The key `a` of the view of the accounts: it adds an account. See T-124.
    ///
    /// **The login screen needs a terminal that no view holds** (T-123),
    /// therefore the program starts again and the new program shows that
    /// screen. Every account of the database stays: the login writes a new row,
    /// and that row takes the start of the program.
    pub fn add_an_account(&mut self) {
        log::info!("[the accounts] the user adds an account. The program starts again.");

        self.the_program_starts_again = Some(TheProgramStartsAgain {
            variables: vec![
                (
                    crate::logic::the_accounts::THE_PROGRAM_ADDS_AN_ACCOUNT.to_string(),
                    "1".to_string(),
                ),
                (
                    crate::logic::auth::auth_input::THE_ADDRESS_OF_THE_LOGIN.to_string(),
                    self.server_address.clone(),
                ),
            ],
            message: "This system cannot start the program again. Stop the program, and start it \
                      again with the variable TOUTUI_ADD_AN_ACCOUNT=1."
                .to_string(),
        });
    }

    /// The account of this program stands in no row of the disk, therefore the
    /// program starts again. See T-159.
    ///
    /// **A second program of one account logs out with the key `l` of the view
    /// of the accounts** (T-124), and the row of `users` then goes away while
    /// this program runs (T-155). Every key that refreshes the screen makes a
    /// new application (T-131), and that application took the account of the
    /// disk: **the header said "👋 Connected as " with no name at all**, the
    /// token had no plain form, every write of the account changed 0 rows — the
    /// library of the key `S` among them — and the program went on with the
    /// token of the account that logged out, because the client of the start
    /// holds that token.
    ///
    /// **The rule of the start is the rule here** (T-136): the program that
    /// starts again takes the account of the disk, and it draws the login
    /// screen when no account stays. The login screen says why. `exec` gives
    /// that program the terminal of this one, and the loop of `src/main.rs`
    /// sends the place of the playback before it (T-139).
    pub fn the_account_of_this_program_is_gone(&mut self) {
        log::warn!(
            "[the accounts] the account {} stands in no row of the disk. The program starts again.",
            self.username
        );

        let _ = crate::db::crud::update_login_err(
            crate::logic::the_accounts::the_text_of_an_account_that_is_gone(&self.username)
                .as_str(),
        );

        self.the_program_starts_again = Some(TheProgramStartsAgain {
            variables: vec![(
                crate::logic::auth::auth_input::THE_ADDRESS_OF_THE_LOGIN.to_string(),
                self.server_address.clone(),
            )],
            message: crate::logic::the_accounts::the_text_of_an_account_that_is_gone(
                &self.username,
            ),
        });
    }

    /// The login screen comes, because the program holds no account. See T-124.
    ///
    /// The user logged out of the one account of the program. The database
    /// holds no row of an account now, therefore the program that starts again
    /// draws the login screen of a first start.
    pub fn the_login_screen_comes(&mut self) {
        log::info!(
            "[the accounts] no account stays. The program starts again, and the login screen comes."
        );

        self.the_program_starts_again = Some(TheProgramStartsAgain {
            variables: vec![(
                crate::logic::auth::auth_input::THE_ADDRESS_OF_THE_LOGIN.to_string(),
                self.server_address.clone(),
            )],
            message: "The program removed the account. Stop the program, and start it again: it \
                      asks you for a server, a name, and a password then."
                .to_string(),
        });
    }

    /// The key `c` of the view of the accounts: the account of the line starts
    /// the program. See T-124.
    ///
    /// The program asks one time, because it starts again: a playback stops
    /// with the process. Any key that is not `c` stops the question, and that
    /// is the rule of the log out (T-36).
    pub fn this_account_starts(&mut self) {
        let Some(name) = self
            .list_state_settings_account
            .selected()
            .and_then(|line| self.the_accounts.get(line))
            .map(|(name, _, _)| name.clone())
        else {
            return;
        };

        // **The disk is the truth**: a second program of this account can
        // remove the account of this line while this view stands. The key then
        // took the mark of the start from every account and it gave that mark
        // to nobody, and the program showed the login screen at every start
        // after it. See T-155.
        self.the_accounts_come_from_the_disk();

        if matches!(
            crate::logic::the_accounts::the_account_of_the_line(&self.the_accounts, &name),
            crate::logic::the_accounts::TheAccountOfTheLine::ItIsGone
        ) {
            crate::logic::message::say(
                &crate::logic::the_accounts::the_text_of_an_account_that_is_gone(&name),
            );

            return;
        }

        if self
            .the_accounts
            .iter()
            .any(|(one, _, starts)| *one == name && *starts)
        {
            crate::logic::message::say(&format!(
                "The program starts with the account {} already.",
                name
            ));
            return;
        }

        if self.confirm_the_account_that_starts.as_deref() != Some(name.as_str()) {
            self.confirm_the_account_that_starts = Some(name.clone());

            crate::logic::message::say(&format!(
                "Press c again to start with the account \"{}\". The program starts again, and a \
                 playback stops.",
                name
            ));

            return;
        }

        self.confirm_the_account_that_starts = None;
        self.start_the_program_with_this_account(&name);
    }

    /// Writes the account of the start, and starts the program again.
    ///
    /// **Every list of the program comes from one account**, therefore a change
    /// of the account is the work of `App::new` and of every task: the new
    /// process does that work, and no state of this process crosses it. The key
    /// `c` and a log out of the account that starts both come here. See T-124.
    pub fn start_the_program_with_this_account(&mut self, name: &str) {
        match crate::db::crud::make_this_account_the_default(name) {
            Err(error) => {
                log::error!(
                    "[the accounts] the account {} cannot start: {}",
                    name,
                    error
                );
                crate::logic::message::say("The program cannot write the account of the start.");
                return;
            }

            // **The program must not start again for an account of no row.**
            // The write gave the mark of the start to nobody before T-155, and
            // the login screen then came at every start with a good account on
            // the disk.
            Ok(0) => {
                log::warn!(
                    "[the accounts] the database holds no account {}. The program stays.",
                    name
                );
                crate::logic::message::say(
                    &crate::logic::the_accounts::the_text_of_an_account_that_is_gone(name),
                );
                self.the_accounts_come_from_the_disk();
                return;
            }

            Ok(_) => {}
        }

        log::info!(
            "[the accounts] the account {} starts the program. The program starts again.",
            name
        );

        self.the_program_starts_again = Some(TheProgramStartsAgain {
            variables: Vec::new(),
            message: "This system cannot start the program again. Stop the program, and start it \
                      again: it takes the account then."
                .to_string(),
        });
    }

    /// Shows the values of the block `[reader]` of `config.toml`. See T-77.
    ///
    /// The line of the value that the program uses now stands selected, and the
    /// user reads which value that is.
    ///
    /// **The view says the value of the file**, therefore it reads the file here:
    /// a second program of this account writes that file too, and a title that
    /// says "512 MB now" for a file of 4096 lies to the user. See T-142.
    pub fn show_the_settings_of_the_reader(&mut self) {
        self.take_the_limit_of_the_cache_of_the_file();

        let now = self.megabytes_of_the_cache();

        let line = crate::logic::reader::cache::THE_VALUES_OF_THE_SETTINGS
            .iter()
            .position(|value| *value == now)
            .unwrap_or(0);

        self.list_state_settings_reader.select(Some(line));
        self.view_state = AppView::SettingsReader;
    }

    /// Reads the limit of the cache of the ebooks of `config.toml` again, and it
    /// gives that value to the program. See T-142.
    ///
    /// **One account can hold two programs**, and both of them write this file
    /// (T-140 holds the same rule for the row of a listening session). The value
    /// goes to two places, therefore they cannot disagree: `self.config` for the
    /// screen, and the slot of the module for the task that removes a book.
    ///
    /// A file that the program cannot read changes nothing: the program then
    /// keeps the value that it holds.
    fn take_the_limit_of_the_cache_of_the_file(&mut self) {
        let Ok(of_the_file) = load_config() else {
            return;
        };

        self.config.reader.ebook_cache_mb = of_the_file.reader.ebook_cache_mb;

        crate::logic::reader::cache::keep_the_limit_of_the_configuration(
            of_the_file.reader.ebook_cache_mb,
        );
    }

    /// Gives the cache of the ebooks of `config.toml`, in megabytes.
    ///
    /// The value 0 means that the file names no value, therefore the program
    /// uses its own. See T-72.
    pub fn megabytes_of_the_cache(&self) -> u64 {
        match self.config.reader.ebook_cache_mb {
            0 => crate::logic::reader::cache::LIMIT_OF_THE_CACHE / (1024 * 1024),
            value => value,
        }
    }

    /// Writes the value of the cache of the ebooks that the user took. See T-77.
    ///
    /// The write keeps every comment of the file, and the program uses the new
    /// value at once: the next book that comes holds the cache to it, and the
    /// user starts the program no second time.
    pub fn take_the_value_of_the_cache(&mut self) {
        let Some(megabytes) = self
            .list_state_settings_reader
            .selected()
            .and_then(|line| crate::logic::reader::cache::THE_VALUES_OF_THE_SETTINGS.get(line))
            .copied()
        else {
            return;
        };

        match crate::config::write_the_value("reader", "ebook_cache_mb", &megabytes.to_string()) {
            Ok(()) => {
                self.config.reader.ebook_cache_mb = megabytes;
                crate::logic::reader::cache::keep_the_limit_of_the_configuration(megabytes);

                crate::logic::message::say(&format!(
                    "The cache of the ebooks holds {} MB now. config.toml has the value.",
                    megabytes
                ));
            }
            Err(error) => crate::logic::message::say(&format!(
                "The program did not write config.toml: {}",
                error
            )),
        }
    }

    /// Opens the ebook of the item that the user selected. See T-10.
    ///
    /// The program keeps the file in the directory of the downloads. Therefore
    /// a second visit needs no request, and the reader also works with no
    /// server.
    pub fn open_the_ebook(&mut self) {
        let Some(item_id) = self.selected_item_id() else {
            return;
        };

        // The title comes from the view of the user, therefore it must come
        // before the view changes to the reader. See T-54.
        let title = self.selected_item_title();

        // A book that the reader holds already needs no work.
        if self
            .reader
            .as_ref()
            .is_some_and(|reader| reader.item_id == item_id && reader.sends_the_place())
        {
            if !matches!(self.view_state, AppView::Reader) {
                self.the_view_before_the_reader = self.view_state;
            }

            self.view_state = AppView::Reader;
            return;
        }

        self.get_the_book(item_id, title, None);
    }

    /// Gets one book of an item, and it opens the reader on it.
    ///
    /// `ino` names the file of the server. `None` takes the book that the
    /// server opens for the media. See T-10 and T-76.
    fn get_the_book(&mut self, item_id: String, title: Option<String>, ino: Option<String>) {
        self.reader = None;
        self.reader_message = Some("The program gets the book…".to_string());

        // The key `h` of the reader gives this view back. See T-52.
        if !matches!(self.view_state, AppView::Reader) {
            self.the_view_before_the_reader = self.view_state;
        }

        self.view_state = AppView::Reader;

        let api = std::sync::Arc::clone(&self.api);
        let username = self.username.clone();
        let answer = crate::logic::reader::opened_book();

        tokio::spawn(async move {
            let outcome = match crate::logic::reader::session::get_the_ebook_of(
                &api,
                &username,
                &item_id,
                ino.as_deref(),
            )
            .await
            {
                Ok(path) => crate::logic::reader::Reader::open_with_the_title(
                    &path,
                    &item_id,
                    title.as_deref(),
                )
                .map_err(|error| error.to_string()),
                Err(message) => Err(message),
            };

            let outcome = match outcome {
                Ok(mut reader) => {
                    // The size of each chapter gives the part of the book, and
                    // the place of the server needs it. The work reads the
                    // file, therefore it runs here and not on the thread that
                    // draws.
                    reader.measure_the_chapters();

                    if ino.is_some() {
                        // This book is not the book of the server. The place of
                        // the server belongs to the book of `media.ebookFile`,
                        // therefore the reader neither reads that place nor
                        // writes it. See T-76.
                        reader.the_place_stays_here();
                    } else {
                        // The user reads the same book on a different machine.
                        // The program opens the book where they stopped. See
                        // T-10, section 6.
                        match crate::logic::reader::session::place_of_the_server(&api, &item_id)
                            .await
                        {
                            Ok(Some((location, part))) => {
                                reader.go_to_the_place_of_the_server(&location, part);
                            }
                            // The user never opened this book. It starts at its
                            // first page, and the send of that place is the
                            // truth.
                            Ok(None) => {}
                            // **The reader stands at the first page of a book
                            // that the server holds at another place.** A send
                            // would take the place of the user away, on every
                            // machine of the account. See T-178.
                            Err(error) => {
                                reader.the_server_did_not_give_the_place();
                                crate::logic::message::say_in(
                                    AppView::Reader,
                                    &crate::logic::reader::session::the_sentence_of_a_place_that_did_not_come(
                                        &error,
                                    ),
                                );
                            }
                        }
                    }

                    Ok(reader)
                }
                Err(message) => Err(message),
            };

            if let Ok(mut place) = answer.lock() {
                *place = Some(outcome);
            }
        });
    }

    /// Sends the place of the reader when the rule of the time says so.
    ///
    /// The loop of the application calls this for each turn. See T-10.
    pub fn send_the_place_of_the_reader_if_it_is_time(&mut self) {
        let wants = self
            .reader
            .as_ref()
            .is_some_and(|reader| reader.wants_to_send());

        if wants {
            self.send_the_place_of_the_reader();
        }
    }

    /// Says on the disk that this program reads its book now.
    ///
    /// The loop of the application calls this for each turn. **A second window
    /// of this account removes the books of the cache with no key of this
    /// window**, and `keep` of that removal names the book of that window
    /// alone: the time of the file is the one word that the two programs
    /// share. See T-153.
    pub fn say_that_this_program_reads_its_book(&mut self) {
        if let Some(reader) = self.reader.as_mut() {
            reader.say_that_a_program_reads_this_book();
        }
    }

    /// Takes the book that the task opened, if it is ready.
    pub fn take_the_book(&mut self) {
        let Some(outcome) = crate::logic::reader::take_the_opened_book() else {
            return;
        };

        match outcome {
            Ok(reader) => {
                self.reader_message = None;
                self.reader = Some(reader);
            }
            Err(message) => {
                self.reader_message = Some(message);
                self.reader = None;
            }
        }
    }

    /// The keys of the reader of an ebook. See T-10.
    fn handle_key_of_the_reader(&mut self, code: KeyCode) {
        // The height of the text is the height of the screen, less the line of
        // the header, the two lines of the keys, and the two lines of the
        // header of the application.
        let height = crossterm::terminal::size()
            .map(|(_, rows)| rows.saturating_sub(5))
            .unwrap_or(20);

        // The list of every key opens over the reader. The reader uses `?` for
        // no work of its own, therefore the key holds the same meaning in every
        // view. See T-49 and T-52.
        if matches!(code, KeyCode::Char('?')) {
            self.show_every_key();
            return;
        }

        // The key `h` leaves the reader always. `Esc` leaves it when the
        // contents are closed, and it closes the contents when they are open.
        // A view with a fault holds no reader, therefore this rule stands
        // before every rule that needs one. See T-52.
        let contents_are_open = self
            .reader
            .as_ref()
            .is_some_and(|reader| reader.contents_open);

        if matches!(code, KeyCode::Char('h'))
            || (matches!(code, KeyCode::Esc) && !contents_are_open)
        {
            // The place goes to the server before the user leaves the book.
            let wants = self
                .reader
                .as_ref()
                .is_some_and(|reader| reader.wants_to_send_at_the_end());

            if wants {
                self.send_the_place_of_the_reader();
            }

            self.view_state = self.the_view_before_the_reader;
            return;
        }

        let Some(reader) = self.reader.as_mut() else {
            return;
        };

        if reader.contents_open {
            match code {
                KeyCode::Char('j') | KeyCode::Down => {
                    if reader.contents_line + 1 < reader.contents.len() {
                        reader.contents_line += 1;
                    }
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    reader.contents_line = reader.contents_line.saturating_sub(1);
                }
                KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                    if let Some(chapter) = reader
                        .contents
                        .get(reader.contents_line)
                        .and_then(|entry| entry.spine_index)
                    {
                        reader.go_to_chapter(chapter);
                    }
                    reader.contents_open = false;
                }
                KeyCode::Char('t') | KeyCode::Esc => reader.contents_open = false,
                _ => {}
            }

            return;
        }

        match code {
            KeyCode::Char('j') | KeyCode::Down => reader.scroll(1, height),
            KeyCode::Char('k') | KeyCode::Up => reader.scroll(-1, height),
            KeyCode::Char(' ') | KeyCode::PageDown => reader.scroll(i64::from(height), height),
            KeyCode::Char('b') | KeyCode::PageUp => reader.scroll(-i64::from(height), height),
            KeyCode::Char('n') => reader.next_chapter(),
            KeyCode::Char('p') => reader.previous_chapter(),
            KeyCode::Char('g') => reader.to_the_start(),
            KeyCode::Char('G') => reader.to_the_end(height),
            KeyCode::Char('t') => {
                reader.contents_open = true;
                reader.contents_line = 0;
            }
            KeyCode::Char('s') => self.send_the_place_of_the_reader(),
            // An item can hold more than one ebook. See T-76.
            KeyCode::Char('e') => self.show_the_ebooks_of_the_media(),
            _ => {}
        }
    }

    /// Sends the place of the reader to the server. See T-10, section 6.
    pub fn send_the_place_of_the_reader(&mut self) {
        let Some(reader) = self.reader.as_ref() else {
            return;
        };

        // **A place that this program did not read must not go to the server.**
        // The book of another file of the item holds one road (T-76), and a
        // read of the place that came back with a fault holds the other one
        // (T-178). The two roads say two different things.
        if let Some(text) = crate::logic::reader::session::the_sentence_of_a_place_that_stays_here(
            reader.the_place_of_the_book(),
        ) {
            crate::logic::message::say(text);
            return;
        }

        let item_id = reader.item_id.clone();
        let location = reader.location_text();
        let part = reader.fraction();
        let api = std::sync::Arc::clone(&self.api);

        // The reader remembers the place that it sent. It then sends nothing
        // while the user reads the same line.
        if let Some(reader) = self.reader.as_mut() {
            reader.the_place_went_to_the_server();
        }

        crate::logic::message::say("The place of the book goes to the server…");

        tokio::spawn(async move {
            let body = serde_json::json!({
                "ebookLocation": location,
                "ebookProgress": part,
            });

            let text = match api
                .patch_json(&format!("/api/me/progress/{}", item_id), &body)
                .await
            {
                Ok(()) => "The server has the place of the book.".to_string(),
                Err(error) => format!("The server did not take the place: {}", error),
            };

            crate::logic::message::say(text.as_str());
        });
    }

    /// Gives the line that the user selected in the view `Library`.
    pub fn selected_library_row(&self) -> Option<&LibraryRow> {
        self.library_rows.get(self.list_state_library.selected()?)
    }

    /// Gives the position of the selection in the lists of the library.
    ///
    /// A line of a series gives the position of its first book, because the
    /// lists of the library hold one row for each book and no row for a
    /// series.
    pub fn selected_library_item(&self) -> Option<usize> {
        Some(self.selected_library_row()?.item())
    }

    /// Takes the media that left the shelf of Continue Listening away from the
    /// lines of the Home view.
    ///
    /// The server keeps a media that the user finished, and a media that the
    /// user hid, away from that shelf. A live message says that one of the two
    /// happened, and this function then makes the lines again. It does nothing
    /// when the list did not change, therefore the sync of the playback of the
    /// program itself costs one comparison of two small lists. See T-66.
    ///
    /// The render calls this, because the render is not asynchronous.
    pub fn take_the_media_that_left_away(&mut self) {
        let away = crate::logic::live::the_media_away_from_continue_listening();

        // **The list holds the number of the line, and not the identity of the
        // media.** One media stands on two shelves: a measurement of
        // 2026-08-11 showed a book on Continue Listening and on Recently Added
        // together. A list of the identities took both lines away, and the
        // server gives the second one. Each shelf gives its own number.
        let mut that_left: std::collections::BTreeSet<usize> = std::collections::BTreeSet::new();

        for (item, on_the_shelf) in self.of_continue_listening.iter().enumerate() {
            if !on_the_shelf {
                continue;
            }

            if let Some(id) = self._ids_cnt_list.get(item) {
                if away.contains(id) {
                    that_left.insert(item);
                }
            }
        }

        if that_left == self.the_media_that_left {
            return;
        }

        let selected = self.selected_home_row().cloned();

        // **A media that goes away from under the line of the user takes the
        // next key of that user with it.** The lines keep the number of the
        // line, therefore the media below moves under the cursor with no word
        // at all. See T-160.
        let went_away = crate::logic::home_view::the_media_of_the_line_that_went_away(
            &self.home_rows,
            self.list_state_cnt_list.selected(),
            |item| that_left.contains(&item),
        );

        self.the_media_that_left = that_left;

        self.home_rows = crate::logic::home_view::without_the_media_that_left(
            &self.home_rows_of_the_server,
            |item| self.the_media_that_left.contains(&item),
        );

        // **The media of the line of the user went away, therefore no line is
        // selected.** No key of the selection can then reach a media that the
        // user did not choose, and the message names the media that went away.
        // See T-160.
        if let Some(item) = went_away {
            self.list_state_cnt_list.select(None);

            if let Some(title) = self
                ._titles_cnt_list
                .get(item)
                .filter(|one| !one.is_empty())
            {
                crate::logic::message::say_in(
                    AppView::Home,
                    crate::logic::home_view::the_text_of_the_media_that_went_away(title).as_str(),
                );
            }

            return;
        }

        // The user keeps the line that they selected. A line that went away
        // gives the line above it, and never the top of the view.
        let place = selected.and_then(|row| self.home_rows.iter().position(|one| *one == row));

        let place =
            match place {
                Some(place) => Some(place),
                None => {
                    let old = self.list_state_cnt_list.selected().unwrap_or(0);
                    let at = old.min(self.home_rows.len().saturating_sub(1));

                    if self.home_rows.get(at).is_some_and(|row| {
                        crate::logic::home_view::HomeRow::is_a_line_of_the_user(row)
                    }) {
                        Some(at)
                    } else {
                        crate::logic::home_view::previous_line(&self.home_rows, at)
                    }
                }
            };

        self.list_state_cnt_list.select(place);
    }

    /// Gives the line that the user selected in the view `Home`. See T-24.
    pub fn selected_home_row(&self) -> Option<&HomeRow> {
        self.home_rows.get(self.list_state_cnt_list.selected()?)
    }

    /// Gives the position of the selected media in the lists of the Home
    /// view. A line of a shelf and a line of a series give nothing.
    pub fn selected_home_item(&self) -> Option<usize> {
        self.selected_home_row()?.item()
    }

    /// Gives the series of the selected line of the view `Home`, if that line
    /// is a series.
    pub fn selected_home_series(&self) -> Option<&SeriesView> {
        self.series.get(self.selected_home_row()?.series()?)
    }

    /// Gives the series of the selected line of the view `Library`, if that
    /// line is a series.
    pub fn selected_library_series(&self) -> Option<&SeriesView> {
        self.series.get(self.selected_library_row()?.series()?)
    }

    /// Gives the identity of the media that plays now, if one plays.
    pub fn playing_item(&self) -> Option<String> {
        let state = self.player.state();

        if state.status == crate::player::engine::PlaybackStatus::Stopped
            || state.item_id.is_empty()
        {
            None
        } else {
            Some(state.item_id)
        }
    }

    /// Gives the text of each line of the view `Home`.
    ///
    /// Every line starts with a mark: the media that plays, a media that the
    /// user finished, or the part that the user heard. See T-44.
    pub fn home_lines(&self) -> Vec<String> {
        let playing = self.playing_item();

        self.home_rows
            .iter()
            .map(|row| match row {
                // The name of a shelf. A media of that shelf stands at a
                // column after it, therefore the user reads the shape.
                HomeRow::Shelf { label } => crate::ui::marks::shelf(label),

                // A line of a series holds more than one book, therefore it
                // gets no mark of a position. See T-44 and T-22.
                HomeRow::Series { series } => crate::ui::marks::line(
                    &crate::ui::marks::of_library(false),
                    &self
                        .series
                        .get(*series)
                        .map(|series| series.line())
                        .unwrap_or_default(),
                ),

                HomeRow::Media { item } => {
                    let progress = self.book_progress_cnt_list.get(*item);
                    let percent = progress.and_then(|row| row.first()).map(|s| s.as_str());
                    let finished = progress.and_then(|row| row.get(1)).map(|s| s.as_str());

                    let plays_now = self
                        ._ids_cnt_list
                        .get(*item)
                        .zip(playing.as_ref())
                        .is_some_and(|(id, playing)| id == playing);

                    // A live message of the server gives a newer position than
                    // the request of the start. A different client of the same
                    // account moved in this book, and the mark then shows the
                    // new place at the next frame. See T-47.
                    let live = self
                        ._ids_cnt_list
                        .get(*item)
                        .and_then(|id| crate::logic::live::progress_of(id));

                    let mark = match &live {
                        Some(live) => {
                            crate::ui::marks::of_progress(&live.percent, &live.finished, plays_now)
                        }
                        None => crate::ui::marks::of_progress(
                            percent.unwrap_or(""),
                            finished.unwrap_or(""),
                            plays_now,
                        ),
                    };

                    crate::ui::marks::line(
                        &mark,
                        self._titles_cnt_list
                            .get(*item)
                            .map(|title| title.as_str())
                            .unwrap_or_default(),
                    )
                }
            })
            .collect()
    }

    /// Gives the text of each line of the view `Library`.
    pub fn library_lines(&self) -> Vec<String> {
        let playing = self.playing_item();

        self.library_rows
            .iter()
            .map(|row| match row.series() {
                // A line of a series holds more than one book, therefore it
                // gets no mark of a position. See T-44.
                Some(index) => crate::ui::marks::line(
                    &crate::ui::marks::of_library(false),
                    &self
                        .series
                        .get(index)
                        .map(|series| series.line())
                        .unwrap_or_default(),
                ),
                None => {
                    let plays_now = self
                        .ids_library
                        .get(row.item())
                        .zip(playing.as_ref())
                        .is_some_and(|(id, playing)| id == playing);

                    crate::ui::marks::line(
                        &crate::ui::marks::of_library(plays_now),
                        self.titles_library
                            .get(row.item())
                            .map(|title| title.as_str())
                            .unwrap_or_default(),
                    )
                }
            })
            .collect()
    }

    /// Gives the series that the user selected in the view `Series`.
    pub fn selected_series(&self) -> Option<&SeriesView> {
        self.series.get(self.list_state_series.selected()?)
    }

    /// Gives the book that the user selected in the view `SeriesBook`.
    pub fn selected_series_book(&self) -> Option<&SeriesBookView> {
        self.selected_series()?
            .books
            .get(self.list_state_series_book.selected()?)
    }

    /// Gives the collection or the playlist that the user selected.
    pub fn selected_list(&self) -> Option<&ListView> {
        self.lists.get(self.list_state_lists.selected()?)
    }

    /// Gives the medium that the user selected in the view `ListEntries`.
    pub fn selected_list_entry(&self) -> Option<&ListEntry> {
        self.selected_list()?
            .entries
            .get(self.list_state_list_entries.selected()?)
    }

    /// Gives the item that the keys `D` and `X` operate on.
    ///
    /// The function gives the target of the download, the title, and the author.
    /// The title of an episode is the title of the episode, and its author is the
    /// title of the podcast.
    ///
    /// The function gives `None` when the view has no item that the application
    /// can download. A podcast in the view `Library` is an example: the user must
    /// open the podcast and select one episode.
    pub fn selected_download(&self) -> Option<(DownloadTarget, String, String)> {
        match self.view_state {
            AppView::Home if self.is_podcast => {
                let index = self.selected_home_item()?;

                Some((
                    DownloadTarget::Episode {
                        item_id: self._ids_cnt_list.get(index)?.clone(),
                        episode_id: self.ids_ep_cnt_list.get(index)?.clone(),
                    },
                    self._titles_cnt_list.get(index)?.clone(),
                    self.titles_pod_cnt_list.get(index)?.clone(),
                ))
            }
            AppView::Home => {
                // A line of a series holds more than one book. The user opens
                // the series with the key `l` and downloads one book there.
                let index = self.selected_home_item()?;

                Some((
                    DownloadTarget::Book {
                        item_id: self._ids_cnt_list.get(index)?.clone(),
                    },
                    self._titles_cnt_list.get(index)?.clone(),
                    self.auth_names_cnt_list.get(index)?.clone(),
                ))
            }
            AppView::Library if self.is_podcast => None,
            AppView::Library => {
                // A line of a series holds more than one book. The user opens
                // the series with the key `l` and downloads one book there.
                // See T-22.
                let LibraryRow::Book { item: index } = *self.selected_library_row()? else {
                    return None;
                };

                Some((
                    DownloadTarget::Book {
                        item_id: self.ids_library.get(index)?.clone(),
                    },
                    self.titles_library.get(index)?.clone(),
                    self.auth_names_library.get(index)?.clone(),
                ))
            }
            AppView::SearchBook if self.is_podcast => None,
            AppView::SearchBook => {
                // **Every list of this view holds one value for each line of
                // it** (T-113 and T-117), and the lists of the library hold the
                // rows of the answer of the library alone: one page holds 500
                // items (T-70) and the server searches the whole library, and a
                // library of books takes `&collapseseries=1`, therefore a book of
                // a series of more than one book stands in no row of them.
                //
                // This arm read `titles_library` with the place of the media in
                // `ids_library`, therefore a media that stands in no row of the
                // library gave nothing at all: the keys `D` and `X` said no
                // word and wrote no line of the log, the key `n` said "This line
                // holds no media.", and the key `m` said "This line holds no book
                // and no episode." The key `l` of that same line played the book.
                // See T-218, and T-79 for the rule of a key that does nothing.
                let index = self.list_state_search_results.selected()?;

                Some((
                    DownloadTarget::Book {
                        item_id: self.ids_search_book.get(index)?.clone(),
                    },
                    self.titles_search_book.get(index)?.clone(),
                    self.auth_names_search_book.get(index)?.clone(),
                ))
            }
            AppView::SeriesBook => {
                let book = self.selected_series_book()?;

                Some((
                    DownloadTarget::Book {
                        item_id: book.id.clone(),
                    },
                    book.title.clone(),
                    book.author.clone(),
                ))
            }
            AppView::ListEntries => {
                let entry = self.selected_list_entry()?;

                let target = match entry.episode_id.clone() {
                    Some(episode_id) => DownloadTarget::Episode {
                        item_id: entry.id.clone(),
                        episode_id,
                    },
                    None => DownloadTarget::Book {
                        item_id: entry.id.clone(),
                    },
                };

                Some((target, entry.title.clone(), entry.author.clone()))
            }
            AppView::PodcastEpisode => {
                let episode = self.list_state_pod_ep.selected()?;

                // The two ways into this view hold the episodes in two different
                // lists.
                let (item_id, episode_id, title, podcast) = if self.is_from_search_pod {
                    let podcast = self.list_state_search_results.selected()?;

                    (
                        self.ids_library_pod_search.get(podcast)?.clone(),
                        self.ids_pod_ep_search.get(episode)?.clone(),
                        self.titles_pod_ep_search.get(episode)?.clone(),
                        self.titles_pod_search.first().cloned(),
                    )
                } else {
                    let podcast = self.list_state_library.selected()?;

                    (
                        self.ids_library.get(podcast)?.clone(),
                        self.ids_pod_ep.get(episode)?.clone(),
                        self.titles_pod_ep.get(episode)?.clone(),
                        self.titles_pod.first().cloned(),
                    )
                };

                Some((
                    DownloadTarget::Episode {
                        item_id,
                        episode_id,
                    },
                    title.trim().to_string(),
                    podcast.unwrap_or_default().trim().to_string(),
                ))
            }
            _ => None,
        }
    }

    /// Gives the length of the selected media, in seconds.
    ///
    /// The function gives nothing when the view holds the length as a text
    /// only. The view of the episodes of a podcast is such a view: the server
    /// gives the seconds, and `collect_durations_pod_ep` makes the text at
    /// once.
    fn selected_length(&self) -> Option<f64> {
        match self.view_state {
            // A library of podcasts holds the lengths of the episodes in the
            // lists of the podcasts, and not in this list.
            AppView::Home if self.is_podcast => None,
            AppView::Home => self
                .selected_home_item()
                .and_then(|index| self.duration_cnt_list.get(index).copied()),
            AppView::Library => self
                .selected_library_item()
                .and_then(|index| self.duration_library.get(index).copied()),
            AppView::SearchBook => self
                .list_state_search_results
                .selected()
                .and_then(|index| self.duration_library_search_book.get(index).copied()),
            AppView::SeriesBook => self.selected_series_book().map(|book| book.duration),
            AppView::ListEntries => self.selected_list_entry().map(|entry| entry.duration),
            _ => None,
        }
    }

    /// Gives the media of the keys `M` and `N`, with its episode. See T-219.
    ///
    /// **The place of an episode of a podcast stands at
    /// `/api/me/progress/:item/:episode`** (T-182 and T-188), and the two keys
    /// read `selected_item_id` before: that function gives the identity of the
    /// **item** alone, therefore an episode of a podcast took the path of its
    /// podcast. A measurement of 2026-08-14 against the sandbox, of the line
    /// `Chapter 01` of the shelf Continue Listening of a library of podcasts:
    /// `GET /api/me/progress/:item` answered with the place of `Chapter 00`,
    /// and `PATCH` of that same path answered
    /// `400 Library item is not a book`. The key of the user did nothing at
    /// all, and the words of the program named a fault of the server.
    ///
    /// `selected_download` holds the item **and** the episode of every view
    /// that shows an episode: the Home view of a library of podcasts, the view
    /// of the episodes of a podcast, and the view of the media of a collection
    /// or of a playlist.
    pub fn selected_place(&self) -> Option<(String, Option<String>)> {
        match self.selected_download()?.0 {
            DownloadTarget::Book { item_id } => Some((item_id, None)),
            DownloadTarget::Episode {
                item_id,
                episode_id,
            } => Some((item_id, Some(episode_id))),
        }
    }

    /// Gives the words of a line that holds no place of the user. See T-219.
    ///
    /// **A podcast holds no place, and its episodes hold one each.** The line
    /// of a podcast of the Library view and of the view of the search
    /// therefore names the key that opens its episodes (T-83 for the rule, and
    /// T-170 for the key of the sentence). Every other line with no media says
    /// that it holds none.
    pub fn words_of_a_line_with_no_place(&self) -> &'static str {
        if self.selected_item_id().is_some() {
            "A podcast holds no place. Press l for its episodes."
        } else {
            "No media is selected."
        }
    }

    /// Gives the media that the user selected, for the queue. See T-24.
    ///
    /// `selected_download` gives the media of every view that holds one media,
    /// with its name and the name of its author. The queue needs the same
    /// media. Therefore this function changes the target of the download to
    /// the target of a playback, and it adds the length.
    ///
    /// A view that holds no media gives nothing. A podcast in the view
    /// `Library` is an example: the user opens the podcast and selects one
    /// episode.
    pub fn selected_media(&self) -> Option<crate::logic::queue::Entry> {
        let (target, title, author) = self.selected_download()?;

        let target = match target {
            DownloadTarget::Book { item_id } => PlaybackTarget::Book {
                item_id,
                whole_book_duration: self.selected_length(),
            },
            DownloadTarget::Episode {
                item_id,
                episode_id,
            } => PlaybackTarget::Episode {
                item_id,
                episode_id,
            },
        };

        Some(crate::logic::queue::Entry {
            target,
            title,
            author,
            duration: self.selected_length(),
        })
    }

    /// Puts the selected media at the end of the queue. The key is `n`.
    ///
    /// The key does not change the media that plays. The queue starts the next
    /// media when the media that plays comes to its end. The key `q` shows the
    /// queue, and `l` in that view starts a media now.
    pub fn add_to_the_queue(&mut self) {
        let Some(entry) = self.selected_media() else {
            crate::logic::message::say("This line holds no media.");
            return;
        };

        let title = entry.title.clone();

        // **The disk is the truth of the queue** (T-147), therefore a program that
        // did not read the disk puts no media in it (T-202), and a program whose
        // disk did not take the write puts no media in it either (T-206): the
        // sentence below said that the media is number 1 of the queue while the
        // disk of the account held no row at all.
        let place = match crate::logic::queue::add(entry) {
            Ok(place) => place,
            Err(fault) => {
                crate::logic::message::say(
                    &crate::logic::queue::the_words_of_a_queue_that_the_disk_did_not_hold(
                        fault, "n",
                    ),
                );

                return;
            }
        };

        crate::logic::message::say(&format!(
            "\"{}\" is number {} of the queue. Press q to see the queue.",
            title, place
        ));
    }

    /// Shows every key of the program. The key is `?`. See T-49.
    ///
    /// The key a second time gives the view of the user back. Therefore the
    /// list is a look at the keys, and it takes no place of the work.
    pub fn show_every_key(&mut self) {
        if matches!(self.view_state, AppView::Keys) {
            self.view_state = self.the_view_before_the_keys;
            return;
        }

        self.the_view_before_the_keys = self.view_state;
        self.list_state_keys.select(Some(0));
        self.view_state = AppView::Keys;
    }

    /// Shows the media that wait in the queue. The key is `q`.
    ///
    /// **The view takes the queue of the disk first** (T-147). A second program
    /// of the account writes the same rows, therefore the queue of this process
    /// can be older than the queue of the user.
    pub fn show_the_queue(&mut self) {
        crate::logic::queue::read_the_queue_again();

        let count = crate::logic::queue::len();

        // The selection must stand inside the list. An empty queue has no
        // line to select.
        self.list_state_queue.select(if count == 0 {
            None
        } else {
            Some(self.list_state_queue.selected().unwrap_or(0).min(count - 1))
        });

        // The view opens with the queue of this moment, therefore the program
        // reads the media of the line of the user again. See T-161.
        self.the_media_of_the_line_of_the_queue = None;

        self.scroll_offset = 0;
        self.view_state = AppView::Queue;
    }

    /// Holds the media that the user chose in the view of the queue, and it
    /// takes the line away when that media leaves the queue.
    ///
    /// **The loop of the program calls this at each frame**, because the queue
    /// changes with no key of this user: the media that plays comes to its end
    /// and the queue takes the media of the front away, and a second program of
    /// the account takes a media out with the key `X`. The lines keep the number
    /// of the line, therefore a media that the user did not choose moved under
    /// the cursor with no word at all — the key `X` then took that media out of
    /// the queue, and the key `l` played it and stopped the media that plays.
    /// See T-161, and T-160 for the same rule of the Home view.
    pub fn the_line_of_the_queue_holds_its_media(&mut self) {
        if !matches!(self.view_state, AppView::Queue) {
            self.the_media_of_the_line_of_the_queue = None;
            return;
        }

        let queue = crate::logic::queue::snapshot();
        let entries = queue.entries();
        let of_the_user = self.list_state_queue.selected();

        let of_the_program = self
            .the_media_of_the_line_of_the_queue
            .as_ref()
            .map(|(line, key, _)| (*line, key.as_str()));

        match crate::logic::queue::what_the_line_of_the_user_holds(
            entries,
            of_the_program,
            of_the_user,
        ) {
            // The media of the user stands in the queue, and the cursor goes
            // with it.
            crate::logic::queue::TheLineOfTheUser::ItStandsAt(place) => {
                self.list_state_queue.select(Some(place));

                if let Some(held) = self.the_media_of_the_line_of_the_queue.as_mut() {
                    held.0 = place;
                }
            }
            // **No key of the selection may reach a media that the user did not
            // choose**, therefore the line goes to nobody and the program says
            // which media went away.
            crate::logic::queue::TheLineOfTheUser::ItWentAway => {
                let title = self
                    .the_media_of_the_line_of_the_queue
                    .take()
                    .map(|(_, _, title)| title)
                    .unwrap_or_default();

                self.list_state_queue.select(None);

                crate::logic::message::say_in(
                    AppView::Queue,
                    crate::logic::queue::the_text_of_the_media_that_went_away(&title).as_str(),
                );
            }
            // The user moved the cursor, and that key is their choice.
            crate::logic::queue::TheLineOfTheUser::TheUserChoseAnother => {
                self.the_media_of_the_line_of_the_queue = of_the_user.and_then(|line| {
                    entries
                        .get(line)
                        .map(|entry| (line, entry.key(), entry.title.clone()))
                });
            }
        }
    }

    /// Takes the selected media out of the queue. The key is `X` inside the
    /// view of the queue.
    pub fn remove_from_the_queue(&mut self) {
        let Some(index) = self.list_state_queue.selected() else {
            // **The media of the line can leave the queue with no key of this
            // user** (T-161), and the line then stands on nobody. A key that
            // does nothing must say why (T-79).
            crate::logic::message::say("No media is selected.");
            return;
        };

        // The identity of the line holds the media when a second program
        // changed the queue under this view. See T-147.
        let of_the_line = crate::logic::queue::snapshot()
            .entries()
            .get(index)
            .map(|entry| (entry.key(), entry.title.clone()));

        let Some((key, title_of_the_line)) = of_the_line else {
            return;
        };

        // **A different program of the account can take that media out first**,
        // and this key then takes nothing. The key said nothing at all before
        // T-151, therefore the user could not tell one road from the other.
        // **A media that a second program took out and a disk that says nothing
        // are two conditions** (T-202): the sentence of this key says that the
        // media waits no more, and the media of a disk that says nothing waits
        // still.
        let entry = match crate::logic::queue::take_the_media(index, &key) {
            Ok(entry) => entry,
            Err(fault) => {
                crate::logic::message::say(
                    &crate::logic::queue::the_words_of_a_queue_that_the_disk_did_not_hold(
                        fault, "X",
                    ),
                );

                return;
            }
        };

        self.list_state_queue
            .select(crate::logic::queue::snapshot().selection_after_a_remove(index));

        // The media of the line went out with this key of the user, therefore
        // the program reads the media of the new line at the next frame. See
        // T-161.
        self.the_media_of_the_line_of_the_queue = None;

        if let Some(text) = crate::logic::queue::text_of_the_key_that_takes(
            Some(&title_of_the_line),
            entry.as_ref().map(|entry| entry.title.as_str()),
        ) {
            crate::logic::message::say(&text);
        }
    }

    /// Starts the selected media of the queue now. The key is `l` inside the
    /// view of the queue.
    ///
    /// The media goes out of the queue: it plays, therefore it does not wait.
    /// The media that plays now stops, in the same way as the key `l` in every
    /// other view.
    pub fn start_the_media_of_the_queue(&mut self) {
        let Some(index) = self.list_state_queue.selected() else {
            // The rule of the key `X` above, and the same reason. See T-79 and
            // T-161.
            crate::logic::message::say("No media is selected.");
            return;
        };

        // The identity of the line holds the media when a second program
        // changed the queue under this view. See T-147.
        let Some(key) = crate::logic::queue::snapshot()
            .entries()
            .get(index)
            .map(|entry| entry.key())
        else {
            return;
        };

        let entry = match crate::logic::queue::take_the_media(index, &key) {
            Ok(Some(entry)) => entry,
            Ok(None) => return,
            Err(fault) => {
                crate::logic::message::say(
                    &crate::logic::queue::the_words_of_a_queue_that_the_disk_did_not_hold(
                        fault, "l",
                    ),
                );

                return;
            }
        };

        self.list_state_queue
            .select(crate::logic::queue::snapshot().selection_after_a_remove(index));

        // The media of the line went out with this key of the user, therefore
        // the program reads the media of the new line at the next frame. See
        // T-161.
        self.the_media_of_the_line_of_the_queue = None;

        let api = std::sync::Arc::clone(&self.api);
        let player = self.player.clone();
        let username = self.username.clone();
        let server_address = self.server_address.clone();
        let server_key = self.server_key.clone();

        // The media stands outside the queue now. A playback that does not
        // start gives it back to the queue, therefore this key gives the whole
        // entry and not the target alone. See T-146.
        tokio::spawn(async move {
            crate::logic::playback::play_the_media_of_the_queue(
                &api,
                &player,
                entry,
                username,
                server_address,
                server_key,
            )
            .await;
        });
    }

    /// Toggle between Home and Library views
    fn toggle_view(&mut self) {
        self.view_state = match self.view_state {
            AppView::Home => AppView::Library,
            AppView::Library => AppView::Home,
            AppView::SearchBook => AppView::Home,
            AppView::Keys => AppView::Home,
            AppView::PodcastEpisode => AppView::Home,
            AppView::Series => AppView::Home,
            AppView::SeriesBook => AppView::Home,
            AppView::Lists => AppView::Home,
            AppView::ListEntries => AppView::Home,
            AppView::Reader => AppView::Home,
            AppView::Stats | AppView::Sessions => AppView::Home,
            AppView::SortFilter => AppView::Library,
            AppView::Chapters => AppView::Home,
            AppView::Bookmarks => AppView::Home,
            AppView::Queue => AppView::Home,
            AppView::NewPodcast => AppView::Library,
            AppView::Authors => AppView::Library,
            AppView::Ebooks => AppView::Reader,
            AppView::Downloads => AppView::Library,
            AppView::PutInAList => AppView::Library,
            AppView::SendToEreader => AppView::Library,
            AppView::Settings => AppView::Home,
            AppView::SettingsAccount => AppView::Home,
            AppView::SettingsLibrary => AppView::Home,
            AppView::SettingsAbout => AppView::Home,
            AppView::SettingsUpdateUninstall => AppView::Home,
            AppView::SettingsReader => AppView::Settings,
        };
    }

    /// Takes the next library of the server, in the Home view and in the
    /// Library view. See T-66.
    ///
    /// **The Home view shows the shelves of one library**, therefore a user of
    /// two libraries read the shelf of Continue Listening of one of them only.
    /// The settings hold the same work behind three keys (`S`, the line
    /// "Library", and `l`), and this key does it with one.
    ///
    /// **The two views share one footer**, therefore the key works in both: a
    /// key that a footer names and that does nothing is a fault of its own
    /// (T-79).
    ///
    /// The program holds one library at every moment, and no request of the
    /// start changes: the refresh makes the application again with the new
    /// library, as the settings do (T-82).
    pub fn take_the_next_library(&mut self) {
        if !matches!(self.view_state, AppView::Home | AppView::Library) {
            return;
        }

        if self.is_offline {
            crate::logic::message::say(
                "The server does not answer, therefore the program holds one library.",
            );
            return;
        }

        let Some(next) = crate::logic::library_pages::the_next_library(
            &self.libraries_ids,
            &self.id_selected_lib,
        ) else {
            crate::logic::message::say("This server holds one library.");
            return;
        };

        let name = self.libraries_names.get(next).cloned().unwrap_or_default();

        let Some(id) = self.libraries_ids.get(next) else {
            return;
        };

        // **A write of the disk that failed is no new library** (T-205). The old
        // line was `let _ = update_id_selected_lib(...)`: a database that a
        // second Toutui of this account held (T-140) took the row of nobody, the
        // program said that it shows the other library now, and the refresh
        // after it read the row of the library of before. **A view never says a
        // reason that the program does not have** (T-91), and a key of the user
        // that writes the disk takes a sentence (T-199).
        if let Err(error) = update_id_selected_lib(id, &self.username) {
            log::error!(
                "[the next library] the program did not write the library of {}: {}",
                self.username,
                error
            );

            crate::logic::message::say(crate::ui::keys::THE_NEXT_LIBRARY_DID_NOT_REACH_THE_DISK);

            return;
        }

        crate::logic::message::say(&format!("The program shows the library \"{}\" now.", name));

        self.must_refresh = true;
    }

    /// Select functions that apply to both views
    /// all select functions are from ListState widget
    pub fn select_next(&mut self) {
        // A move of the user stops the wait of the key `G`. See T-112.
        self.reads_every_page_of_the_library = false;

        match self.view_state {
            AppView::Home => {
                // The name of a shelf is not a line of the user, therefore
                // the move goes over it. See T-24.
                let from = self.list_state_cnt_list.selected().unwrap_or(0);
                self.list_state_cnt_list
                    .select(crate::logic::home_view::next_line(&self.home_rows, from));
            }
            AppView::Library => {
                if let Some(selected) = self.list_state_library.selected() {
                    if selected + 1 < self.library_rows.len() {
                        self.list_state_library.select_next();
                    } else {
                        self.list_state_library.select_first();
                    }
                }
            }
            AppView::SearchBook => {
                if let Some(selected) = self.list_state_search_results.selected() {
                    if selected + 1 < self.ids_search_book.len() {
                        self.list_state_search_results.select_next();
                    } else {
                        self.list_state_search_results.select_first();
                    }
                }
            }
            AppView::PodcastEpisode => {
                if let Some(selected) = self.list_state_pod_ep.selected() {
                    if self.is_from_search_pod {
                        if selected + 1 < self.ids_pod_ep_search.len() {
                            self.list_state_pod_ep.select_next();
                        } else {
                            self.list_state_pod_ep.select_first();
                        }
                    } else {
                        if selected + 1 < self.ids_pod_ep.len() {
                            self.list_state_pod_ep.select_next();
                        } else {
                            self.list_state_pod_ep.select_first();
                        }
                    }
                }
            }
            AppView::Series => {
                if let Some(selected) = self.list_state_series.selected() {
                    if selected + 1 < self.series.len() {
                        self.list_state_series.select_next();
                    } else {
                        self.list_state_series.select_first();
                    }
                }
            }
            AppView::SeriesBook => {
                if let Some(selected) = self.list_state_series_book.selected() {
                    if selected + 1 < self.selected_series().map_or(0, |s| s.books.len()) {
                        self.list_state_series_book.select_next();
                    } else {
                        self.list_state_series_book.select_first();
                    }
                }
            }
            AppView::Lists => match self.list_state_lists.selected() {
                // The line holds nobody: the list of that line went away, and
                // the text of T-165 says that this key selects one.
                None => {
                    if !self.lists.is_empty() {
                        self.list_state_lists.select_first();
                    }
                }
                Some(selected) => {
                    if selected + 1 < self.lists.len() {
                        self.list_state_lists.select_next();
                    } else {
                        self.list_state_lists.select_first();
                    }
                }
            },
            AppView::ListEntries => {
                let count = self.selected_list().map_or(0, |l| l.entries.len());

                match self.list_state_list_entries.selected() {
                    None => {
                        if count > 0 {
                            self.list_state_list_entries.select_first();
                        }
                    }
                    Some(selected) => {
                        if selected + 1 < count {
                            self.list_state_list_entries.select_next();
                        } else {
                            self.list_state_list_entries.select_first();
                        }
                    }
                }
            }
            // The reader has its own keys. See T-10.
            AppView::Reader => {}
            // The keys `j` and `k` move the view of the statistics, because
            // that view holds no list. The move stops at the last line.
            AppView::Sessions => {
                if self.sessions_scroll < self.sessions_scroll_max {
                    self.sessions_scroll += 1;
                }
                // The view holds a part of the sessions only. A user who comes
                // near the end of that part starts the read of the next page.
                self.read_the_next_page_of_the_sessions();
            }
            AppView::Stats => {
                if self.stats_scroll < self.stats_scroll_max {
                    self.stats_scroll += 1;
                }
            }
            AppView::SortFilter => {
                let lines = self.lines_of_the_sort_filter_view();
                let from = self.list_state_sort_filter.selected().unwrap_or(0);
                self.list_state_sort_filter
                    .select(crate::logic::list_moves::next(&lines, from));
            }
            AppView::Chapters => {
                let count = self.player.state().chapters.len();
                let from = self.list_state_chapters.selected().unwrap_or(0);

                if from + 1 < count {
                    self.list_state_chapters.select(Some(from + 1));
                } else {
                    self.list_state_chapters.select(Some(0));
                }
            }
            AppView::Bookmarks => {
                let count = crate::logic::bookmarks::bookmarks().len();
                let from = self.list_state_bookmarks.selected().unwrap_or(0);

                if from + 1 < count {
                    self.list_state_bookmarks.select(Some(from + 1));
                } else {
                    self.list_state_bookmarks.select(Some(0));
                }
            }
            // An empty queue holds no line, therefore the move selects
            // nothing. The key `G` in an empty list stopped the program one
            // time. See T-24 and the empty library of `docs/TEST-SERVER.md`.
            AppView::Queue => {
                let count = crate::logic::queue::len();
                let from = self.list_state_queue.selected().unwrap_or(0);

                self.list_state_queue.select(if count == 0 {
                    None
                } else if from + 1 < count {
                    Some(from + 1)
                } else {
                    Some(0)
                });
            }
            AppView::NewPodcast => {
                let count = crate::logic::new_podcast::found().len();
                let from = self.list_state_new_podcast.selected().unwrap_or(0);

                if from + 1 < count {
                    self.list_state_new_podcast.select(Some(from + 1));
                } else {
                    self.list_state_new_podcast.select(Some(0));
                }
            }
            // The view of the keys is a list of text. The move goes to the
            // next line, and the list scrolls. See T-49.
            AppView::Keys => {
                let count = crate::ui::keys::lines().len();
                let from = self.list_state_keys.selected().unwrap_or(0);

                self.list_state_keys.select(if from + 1 < count {
                    Some(from + 1)
                } else {
                    Some(0)
                });
            }
            AppView::Authors => {
                let count = crate::logic::authors::authors().len();
                let from = self.list_state_authors.selected().unwrap_or(0);

                if from + 1 < count {
                    self.list_state_authors.select(Some(from + 1));
                } else {
                    self.list_state_authors.select(Some(0));
                }
            }
            AppView::Ebooks => {
                let count = crate::logic::the_ebooks::ebooks().len();
                let from = self.list_state_ebooks.selected().unwrap_or(0);

                if from + 1 < count {
                    self.list_state_ebooks.select(Some(from + 1));
                } else {
                    self.list_state_ebooks.select(Some(0));
                }
            }
            AppView::PutInAList => {
                let count = self.lists.len();
                let from = self.list_state_put_in_a_list.selected().unwrap_or(0);

                if from + 1 < count {
                    self.list_state_put_in_a_list.select(Some(from + 1));
                } else {
                    self.list_state_put_in_a_list.select(Some(0));
                }
            }
            AppView::SendToEreader => {
                let count = crate::logic::the_ereaders::devices().len();
                let from = self.list_state_send_to_ereader.selected().unwrap_or(0);

                if from + 1 < count {
                    self.list_state_send_to_ereader.select(Some(from + 1));
                } else {
                    self.list_state_send_to_ereader.select(Some(0));
                }
            }
            AppView::Downloads => {
                // The line of this view can stand on nobody: the episode of it
                // left the queue of the server. See T-166.
                let count = crate::logic::the_downloads::downloads().len();
                let line = crate::logic::the_downloads::the_line_of_the_move(
                    self.list_state_downloads.selected(),
                    count,
                    true,
                );

                self.list_state_downloads.select(line);
            }
            AppView::SettingsReader => {
                let count = crate::logic::reader::cache::THE_VALUES_OF_THE_SETTINGS.len();
                let from = self.list_state_settings_reader.selected().unwrap_or(0);

                if from + 1 < count {
                    self.list_state_settings_reader.select(Some(from + 1));
                } else {
                    self.list_state_settings_reader.select(Some(0));
                }
            }
            AppView::Settings => {
                if let Some(selected) = self.list_state_settings.selected() {
                    if selected + 1 < self.settings.len() {
                        self.list_state_settings.select_next();
                    } else {
                        self.list_state_settings.select_first();
                    }
                }
            }
            // The list of the accounts holds one line for each account. The
            // old code moved past the end, and the key `l` then found no
            // account and did nothing. See T-41.
            AppView::SettingsAccount => {
                if let Some(selected) = self.list_state_settings_account.selected() {
                    if selected + 1 < self.the_accounts.len() {
                        self.list_state_settings_account.select_next();
                    } else {
                        self.list_state_settings_account.select_first();
                    }
                }
            }
            AppView::SettingsLibrary => {
                if let Some(selected) = self.list_state_settings_library.selected() {
                    if selected + 1 < self.media_types.len() {
                        self.list_state_settings_library.select_next();
                    } else {
                        self.list_state_settings_library.select_first();
                    }
                }
            }
            AppView::SettingsAbout => self.list_state_settings_about.select_next(),
            AppView::SettingsUpdateUninstall => {
                self.list_state_settings_update_uninstall.select_next()
            }
        }
    }

    pub fn select_previous(&mut self) {
        // A move of the user stops the wait of the key `G`. See T-112.
        self.reads_every_page_of_the_library = false;

        match self.view_state {
            AppView::Home => {
                let from = self.list_state_cnt_list.selected().unwrap_or(0);
                self.list_state_cnt_list
                    .select(crate::logic::home_view::previous_line(
                        &self.home_rows,
                        from,
                    ));
            }
            AppView::Library => self.list_state_library.select_previous(),
            AppView::SearchBook => self.list_state_search_results.select_previous(),
            AppView::PodcastEpisode => self.list_state_pod_ep.select_previous(),
            AppView::Series => self.list_state_series.select_previous(),
            AppView::SeriesBook => self.list_state_series_book.select_previous(),
            AppView::Lists => self.list_state_lists.select_previous(),
            AppView::ListEntries => self.list_state_list_entries.select_previous(),
            AppView::Reader => {}
            AppView::Stats => self.stats_scroll = self.stats_scroll.saturating_sub(1),
            AppView::Sessions => self.sessions_scroll = self.sessions_scroll.saturating_sub(1),
            AppView::SortFilter => {
                let lines = self.lines_of_the_sort_filter_view();
                let from = self.list_state_sort_filter.selected().unwrap_or(0);
                self.list_state_sort_filter
                    .select(crate::logic::list_moves::previous(&lines, from));
            }
            AppView::Chapters => self.list_state_chapters.select_previous(),
            AppView::Bookmarks => self.list_state_bookmarks.select_previous(),
            AppView::Queue => self.list_state_queue.select_previous(),
            AppView::NewPodcast => self.list_state_new_podcast.select_previous(),
            AppView::Authors => self.list_state_authors.select_previous(),
            AppView::Ebooks => self.list_state_ebooks.select_previous(),
            // The line of this view can stand on nobody: the episode of it left
            // the queue of the server. See T-166.
            AppView::Downloads => {
                let count = crate::logic::the_downloads::downloads().len();
                let line = crate::logic::the_downloads::the_line_of_the_move(
                    self.list_state_downloads.selected(),
                    count,
                    false,
                );

                self.list_state_downloads.select(line);
            }
            AppView::PutInAList => self.list_state_put_in_a_list.select_previous(),
            AppView::SendToEreader => self.list_state_send_to_ereader.select_previous(),
            AppView::Keys => self.list_state_keys.select_previous(),
            AppView::Settings => self.list_state_settings.select_previous(),
            AppView::SettingsAccount => self.list_state_settings_account.select_previous(),
            AppView::SettingsLibrary => self.list_state_settings_library.select_previous(),
            AppView::SettingsAbout => self.list_state_settings_about.select_previous(),
            AppView::SettingsUpdateUninstall => {
                self.list_state_settings_update_uninstall.select_previous()
            }
            AppView::SettingsReader => self.list_state_settings_reader.select_previous(),
        }
    }

    pub fn select_first(&mut self) {
        // The key `g` goes to the first line, therefore the wait of the key `G`
        // stops. See T-112.
        self.reads_every_page_of_the_library = false;

        match self.view_state {
            AppView::Home => self
                .list_state_cnt_list
                .select(crate::logic::home_view::first_line(&self.home_rows)),
            AppView::Library => self.list_state_library.select_first(),
            AppView::SearchBook => self.list_state_search_results.select_first(),
            AppView::PodcastEpisode => self.list_state_pod_ep.select_first(),
            AppView::Series => self.list_state_series.select_first(),
            AppView::SeriesBook => self.list_state_series_book.select_first(),
            AppView::Lists => self.list_state_lists.select_first(),
            AppView::ListEntries => self.list_state_list_entries.select_first(),
            AppView::Reader => {}
            AppView::Stats => self.stats_scroll = 0,
            AppView::Sessions => self.sessions_scroll = 0,
            AppView::SortFilter => {
                let lines = self.lines_of_the_sort_filter_view();
                self.list_state_sort_filter
                    .select(crate::logic::list_moves::first(&lines));
            }
            AppView::Chapters => self.list_state_chapters.select_first(),
            AppView::Bookmarks => self.list_state_bookmarks.select_first(),
            AppView::Queue => self.list_state_queue.select_first(),
            AppView::NewPodcast => self.list_state_new_podcast.select_first(),
            AppView::Authors => self.list_state_authors.select_first(),
            AppView::Ebooks => self.list_state_ebooks.select_first(),
            AppView::Downloads => self.list_state_downloads.select_first(),
            AppView::PutInAList => self.list_state_put_in_a_list.select_first(),
            AppView::SendToEreader => self.list_state_send_to_ereader.select_first(),
            AppView::Keys => self.list_state_keys.select_first(),
            AppView::Settings => self.list_state_settings.select_first(),
            AppView::SettingsAccount => self.list_state_settings_account.select_first(),
            AppView::SettingsLibrary => self.list_state_settings_library.select_first(),
            AppView::SettingsAbout => self.list_state_settings_about.select_first(),
            AppView::SettingsUpdateUninstall => {
                self.list_state_settings_update_uninstall.select_first()
            }
            AppView::SettingsReader => self.list_state_settings_reader.select_first(),
        }
    }

    pub fn select_last(&mut self) {
        match self.view_state {
            AppView::Keys => {
                let last = crate::ui::keys::lines().len().saturating_sub(1);
                self.list_state_keys.select(Some(last));
            }
            AppView::Home => self
                .list_state_cnt_list
                .select(crate::logic::home_view::last_line(&self.home_rows)),
            // **The end of the lines is not the end of the library.** The
            // program holds one page at the start, therefore this key must read
            // the pages that are left. See T-112 and T-70.
            AppView::Library => {
                self.take_the_last_line_of_the_library();

                self.reads_every_page_of_the_library =
                    !self.is_offline && self.ids_library.len() < self.library_total;

                if self.reads_every_page_of_the_library {
                    self.ask_for_the_next_page_of_the_library();
                }
            }
            AppView::SearchBook => {
                let last_index = self.ids_search_book.len().saturating_sub(1);
                self.list_state_search_results.select(Some(last_index));
            }
            AppView::PodcastEpisode => {
                if self.is_from_search_pod {
                    let last_index = self.ids_pod_ep_search.len().saturating_sub(1);
                    self.list_state_pod_ep.select(Some(last_index));
                } else {
                    let last_index = self.ids_pod_ep.len().saturating_sub(1);
                    self.list_state_pod_ep.select(Some(last_index));
                }
            }
            AppView::Series => {
                let last_index = self.series.len().saturating_sub(1);
                self.list_state_series.select(Some(last_index));
            }
            AppView::SeriesBook => {
                let last_index = self
                    .selected_series()
                    .map_or(0, |series| series.books.len())
                    .saturating_sub(1);
                self.list_state_series_book.select(Some(last_index));
            }
            AppView::Lists => {
                let last_index = self.lists.len().saturating_sub(1);
                self.list_state_lists.select(Some(last_index));
            }
            AppView::ListEntries => {
                let last_index = self
                    .selected_list()
                    .map_or(0, |list| list.entries.len())
                    .saturating_sub(1);
                self.list_state_list_entries.select(Some(last_index));
            }
            AppView::Reader => {}
            AppView::Stats => self.stats_scroll = self.stats_scroll_max,
            AppView::Sessions => {
                self.sessions_scroll = self.sessions_scroll_max;
                self.read_the_next_page_of_the_sessions();
            }
            AppView::SortFilter => {
                let lines = self.lines_of_the_sort_filter_view();
                self.list_state_sort_filter
                    .select(crate::logic::list_moves::last(&lines));
            }
            AppView::Chapters => {
                let last = self.player.state().chapters.len().saturating_sub(1);
                self.list_state_chapters.select(Some(last));
            }
            AppView::Bookmarks => {
                let last = crate::logic::bookmarks::bookmarks().len().saturating_sub(1);
                self.list_state_bookmarks.select(Some(last));
            }
            AppView::Queue => {
                let count = crate::logic::queue::len();

                self.list_state_queue
                    .select(if count == 0 { None } else { Some(count - 1) });
            }
            AppView::NewPodcast => {
                let last = crate::logic::new_podcast::found().len().saturating_sub(1);
                self.list_state_new_podcast.select(Some(last));
            }
            AppView::Authors => {
                let last = crate::logic::authors::authors().len().saturating_sub(1);
                self.list_state_authors.select(Some(last));
            }
            AppView::Ebooks => {
                let last = crate::logic::the_ebooks::ebooks().len().saturating_sub(1);
                self.list_state_ebooks.select(Some(last));
            }
            AppView::Downloads => {
                let last = crate::logic::the_downloads::downloads()
                    .len()
                    .saturating_sub(1);
                self.list_state_downloads.select(Some(last));
            }
            AppView::PutInAList => {
                let last = self.lists.len().saturating_sub(1);
                self.list_state_put_in_a_list.select(Some(last));
            }
            AppView::SendToEreader => {
                let last = crate::logic::the_ereaders::devices()
                    .len()
                    .saturating_sub(1);
                self.list_state_send_to_ereader.select(Some(last));
            }
            AppView::SettingsReader => {
                let last = crate::logic::reader::cache::THE_VALUES_OF_THE_SETTINGS
                    .len()
                    .saturating_sub(1);
                self.list_state_settings_reader.select(Some(last));
            }
            AppView::Settings => {
                let last_index = self.settings.len().saturating_sub(1);
                self.list_state_settings.select(Some(last_index));
            }
            AppView::SettingsAccount => self.list_state_settings_account.select_last(),
            AppView::SettingsLibrary => {
                let last_index = self.media_types.len().saturating_sub(1);
                self.list_state_settings_library.select(Some(last_index));
            }
            AppView::SettingsAbout => self.list_state_settings_about.select_last(),
            AppView::SettingsUpdateUninstall => {
                self.list_state_settings_update_uninstall.select_last()
            }
        }
    }
}

/// Gives the progress that the server holds for one media, or the fault that
/// stops a write of it.
///
/// **The keys `M` and `N` read a state and they then write the opposite of
/// it.** A read that did not come back leaves the program with no state, and
/// the old code read every fault as "the server has no progress for this
/// media": the two keys then wrote the same value at every press.
///
/// **A status of 404 is the answer of a media that never played**, and such a
/// media is not finished and it is not away from the shelf Continue Listening.
/// A measurement of 2026-08-14 against the sandbox: `GET /api/me/progress/:id`
/// of a book that no reader read gives `404 Not Found`.
///
/// **Every other fault stops the write.** A measurement of 2026-08-14 with
/// `docs/harness/one_method_fails.py`, which answered `500` to
/// `GET /api/me/progress/:id` and which forwarded the `PATCH` of the same path:
/// the user stood on a media that the server held as finished, they pressed
/// `M`, the program wrote `isFinished: true` one more time, and it said
/// `The media is finished now.` The key of the user did the opposite of its
/// work, and the words of the program named a state that it did not read. The
/// key `N` gave the same answer for a media that stood away from the shelf.
/// See T-175.
pub fn the_progress_that_the_server_gave(
    answer: Result<serde_json::Value, crate::api::client::error::ApiError>,
) -> Result<serde_json::Value, crate::api::client::error::ApiError> {
    match answer {
        Ok(answer) => Ok(answer),
        // The server has no progress for this media, and it says so. Such a
        // media is not finished, and it is not away from the shelf.
        Err(crate::api::client::error::ApiError::NotFound) => Ok(serde_json::json!({})),
        Err(error) => Err(error),
    }
}

/// Gives the text of a mark that the program did not change. See T-175.
///
/// The sentence names what the server said (T-91), it says that the program
/// changed nothing, and it names the key that does this work again (T-170).
pub fn message_of_no_mark(error: &crate::api::client::error::ApiError) -> String {
    format!(
        "The server did not give the mark: {} The program changed nothing. \
         Press M to ask the server again.",
        error
    )
}

/// Gives the text of a shelf that the program did not change. See T-175.
pub fn message_of_no_shelf(error: &crate::api::client::error::ApiError) -> String {
    format!(
        "The server did not give the state of this media: {} The program \
         changed nothing. Press N to ask the server again.",
        error
    )
}

/// Changes the mark "finished" of one media on the server.
///
/// The function reads the condition of the media, and it then sends the
/// opposite. It gives the text for the user.
///
/// **A media that goes to "not finished" loses its position.** A measurement
/// against an Audiobookshelf 2.36.0 on 2026-08-11 sent `isFinished: false` and
/// read `currentTime: 0` and `progress: 0` back. The server does that, and
/// this program tells the user. See T-24.
///
/// **An episode of a podcast holds its own place**, and the path of that place
/// names the episode after the item (T-182 and T-188). A path of the item alone
/// answers with the place of one episode of that podcast, and the server refuses
/// every write of it with `400 Library item is not a book`. See T-219.
pub async fn mark_the_media(
    api: &std::sync::Arc<crate::api::client::ApiClient>,
    item_id: &str,
    episode_id: Option<&str>,
) -> String {
    let path = crate::api::me::get_media_progress::the_path_of_the_place(item_id, episode_id);

    let answer: serde_json::Value =
        match the_progress_that_the_server_gave(api.get_json(&path).await) {
            Ok(answer) => answer,
            Err(error) => return message_of_no_mark(&error),
        };

    let was_finished = answer
        .get("isFinished")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);

    let body = serde_json::json!({ "isFinished": !was_finished });

    match api.patch_json(&path, &body).await {
        Ok(()) => message_of_the_mark(!was_finished),
        Err(error) => format!("The server did not take the mark: {}", error),
    }
}

/// Asks the server to get the episodes of a feed that it does not hold.
///
/// The function gives the text that the user reads. See T-24.
pub async fn ask_the_server_for_the_episodes(
    api: &std::sync::Arc<crate::api::client::ApiClient>,
    item_id: &str,
) -> String {
    let item: serde_json::Value = match api.get_json(&format!("/api/items/{}", item_id)).await {
        Ok(value) => value,
        Err(error) => return format!("The server did not give the podcast: {}", error),
    };

    let Some(feed_url) = item["media"]["metadata"]["feedUrl"].as_str() else {
        return "This podcast holds no address of a feed.".to_string();
    };

    let feed = match crate::api::podcasts::get_feed(api, feed_url).await {
        Ok(value) => value,
        Err(error) => return format!("The server did not read the feed: {}", error),
    };

    let held: Vec<serde_json::Value> = item["media"]["episodes"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    let asked = crate::api::podcasts::missing(&feed.episodes, &held);

    if asked.is_empty() {
        return format!(
            "The server holds every episode of the feed ({} of {}).",
            held.len(),
            feed.episodes.len()
        );
    }

    match crate::api::podcasts::download_episodes(api, item_id, &asked).await {
        Ok(()) => format!(
            "The server gets {} episode(s). Press R after a moment.",
            asked.len()
        ),
        Err(error) => format!("The server did not take the request: {}", error),
    }
}

/// Reads a feed and writes the podcast in the library. See T-24.
///
/// The function gives the text that the user reads. It needs the folder of
/// the library, therefore it asks the server for the library first.
pub async fn add_a_podcast(
    api: &std::sync::Arc<crate::api::client::ApiClient>,
    library_id: &str,
    feed_url: &str,
    title: &str,
) -> String {
    #[derive(serde::Deserialize)]
    struct Library {
        #[serde(default)]
        folders: Vec<Folder>,
    }

    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Folder {
        #[serde(default)]
        id: String,
        #[serde(default)]
        full_path: String,
    }

    let library: Library = match api
        .get_json(&format!("/api/libraries/{}", library_id))
        .await
    {
        Ok(value) => value,
        Err(error) => return format!("The server did not give the library: {}", error),
    };

    // The server makes the directory of the podcast inside a folder of the
    // library. A library with no folder cannot hold a new podcast.
    let Some(folder) = library.folders.first() else {
        return "This library has no folder. The web page adds one.".to_string();
    };

    let feed = match crate::api::podcasts::get_feed(api, feed_url).await {
        Ok(value) => value,
        Err(error) => return format!("The server did not read the feed: {}", error),
    };

    let body = crate::api::podcasts::body_for(&feed, library_id, &folder.id, &folder.full_path);

    match crate::api::podcasts::create_podcast(api, &body).await {
        Ok(_) => format!(
            "\"{}\" is in the library now, with {} episode(s). Press R to see it.",
            title,
            feed.episodes.len()
        ),
        // A `400` comes when the library holds that podcast already: the
        // server cannot make a directory that exists. A measurement on
        // 2026-08-11 gave that answer for a second add of one podcast.
        Err(crate::api::client::error::ApiError::Server(400)) => format!(
            "The server refused \"{}\". The library can hold that podcast already.",
            title
        ),
        Err(error) => format!("The server did not add the podcast: {}", error),
    }
}

/// Takes a media away from the shelf of Continue Listening, or puts it back.
///
/// The function reads the state first, therefore the key is a change of the
/// state and not one direction only. A media that never played has no
/// progress, and the server then gives an error to the first request; such a
/// media does not stand on the shelf, and the program writes the field
/// anyway. See T-24.
/// **An episode of a podcast holds its own place**, and the path of that place
/// names the episode after the item (T-182 and T-188). See T-219.
pub async fn hide_the_media(
    api: &std::sync::Arc<crate::api::client::ApiClient>,
    item_id: &str,
    episode_id: Option<&str>,
) -> String {
    let path = crate::api::me::get_media_progress::the_path_of_the_place(item_id, episode_id);

    let answer: serde_json::Value =
        match the_progress_that_the_server_gave(api.get_json(&path).await) {
            Ok(answer) => answer,
            Err(error) => return message_of_no_shelf(&error),
        };

    let was_hidden = answer
        .get("hideFromContinueListening")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);

    let body = serde_json::json!({ "hideFromContinueListening": !was_hidden });

    match api.patch_json(&path, &body).await {
        Ok(()) => message_of_the_shelf(!was_hidden),
        Err(error) => format!("The server did not take the change: {}", error),
    }
}

/// Gives the text that the user reads after a change of the shelf.
///
/// The message asks for no key: the server answers this request with a live
/// message, and the line of the Home view goes away or comes back at the next
/// frame. A measurement of 2026-08-11 shows both directions. See T-66.
pub fn message_of_the_shelf(hidden: bool) -> String {
    if hidden {
        "The media is away from Continue Listening now.".to_string()
    } else {
        "The media is on Continue Listening again.".to_string()
    }
}

/// Gives the text that the user reads after a change of the mark.
///
/// A media that the user finished leaves the shelf of Continue Listening by
/// itself. The mark of every other list needs the key `R`. See T-66.
pub fn message_of_the_mark(finished: bool) -> String {
    if finished {
        "The media is finished now.".to_string()
    } else {
        "The media is not finished now, and its position went back to the \
         start."
            .to_string()
    }
}
