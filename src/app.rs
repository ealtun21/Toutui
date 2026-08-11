use crate::api::libraries::get_all_books::*;
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
use crate::utils::pop_up_message::*;
use color_eyre::Result;
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent, KeyEventKind},
    widgets::ListState,
};
use std::io::stdout;

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
    /// Every key of the program. The key `?` opens it. See T-49.
    Keys,
    Settings,
    SettingsAccount,
    SettingsLibrary,
    SettingsAbout,
    SettingsUpdateUninstall,
}

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
    /// The lines of the Home view. A shelf gives a line for its name, and a
    /// line for each of its media. See T-24.
    pub home_rows: Vec<HomeRow>,
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
    /// The list of the bookmarks of one media. See T-24.
    pub list_state_bookmarks: ListState,
    /// The list of the media that wait in the queue. See T-24.
    pub list_state_queue: ListState,
    /// The media whose bookmarks the view shows. See T-24.
    pub bookmarks_of: String,
    /// The timer for sleep, if the user set one. See T-24.
    pub sleep: Option<crate::logic::sleep_timer::Timer>,
    /// The choice of the timer, in minutes. `Some(0)` is the end of the
    /// chapter, and `None` is off.
    pub sleep_choice: Option<u64>,
    /// The list of the podcasts that the server found. See T-24.
    pub list_state_new_podcast: ListState,
    /// The list of the authors of the library. See T-24.
    pub list_state_authors: ListState,
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
    /// The number of positions that wait for the server. See T-25.
    pub waiting_progress: usize,
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
    pub all_usernames: Vec<String>,
    pub all_server_addresses: Vec<String>,
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
    /// What this account may do on the server. An absent answer gives every
    /// permission. See T-24.
    pub permissions: crate::api::me::permissions::Permissions,
    /// The account that waits for the second press of the key `l`. A log out
    /// forgets a token, therefore the program asks one time. See T-36.
    pub confirm_logout: Option<String>,
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
}

/// Init app
impl App {
    /// Makes the application state.
    ///
    /// The caller gives the HTTP client. The client holds the addresses of the
    /// server and the decrypted token.
    pub async fn new(api: std::sync::Arc<crate::api::client::ApiClient>) -> Result<Self> {
        // init config
        let config = load_config()?;

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
                println!("Error: {}", e);
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
        let (mut library_sort, library_desc, library_filter) =
            crate::db::crud::get_library_sort(&username);

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

        let library_query =
            crate::logic::sort_filter::query(&library_sort, library_desc, &library_filter);

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

            for group in _ids_cnt_list.clone().chunks(AT_THE_SAME_TIME).enumerate() {
                let (group_number, ids) = group;
                let mut tasks = tokio::task::JoinSet::new();

                for (inside, id) in ids.iter().enumerate() {
                    let place = group_number * AT_THE_SAME_TIME + inside;
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

        // The series, the collections, the playlists, and the items do not
        // need each other. The old code asked for them one after the other,
        // therefore a slow server made the user wait for the sum of the four.
        // They go together now, and the wait is the longest of the four. See
        // T-40.
        crate::utils::startup::set("the series, the lists, and every item");

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
                    // application. The user then sees an empty list.
                    log::warn!("[app] the server did not give the series: {}", error);
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

            get_all_books(&api, &id_selected_lib, &library_query)
                .await
                .unwrap_or_else(|error| {
                    log::warn!("[app] the server did not give the items: {}", error);
                    crate::api::libraries::get_all_books::Root::default()
                })
        };

        let ask_for_the_permissions = async {
            if is_offline {
                return crate::api::me::permissions::Permissions::default();
            }

            crate::api::me::permissions::get_permissions(&api)
                .await
                .unwrap_or_else(|error| {
                    log::warn!("[app] the server did not give the permissions: {}", error);
                    crate::api::me::permissions::Permissions::default()
                })
        };

        let (series, collections, playlists, all_books, permissions) = tokio::join!(
            ask_for_the_series,
            ask_for_the_collections,
            ask_for_the_playlists,
            ask_for_the_items,
            ask_for_the_permissions,
        );

        let lists = collect_lists(&collections, &playlists);

        let downloads = if is_offline {
            get_all_downloads(&username, &server_key)
        } else {
            Vec::new()
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

        if is_podcast {
            for id_library in ids_library.iter() {
                crate::utils::startup::set("the episodes of the podcasts");
                let podcast_episode = get_pod_ep(&api, id_library.as_str()).await?;
                let title = collect_titles_pod_ep(&podcast_episode).await;
                all_titles_pod_ep.push(title);
                let id = collect_ids_pod_ep(&podcast_episode).await;
                all_ids_pod_ep.push(id);
                let sub = collect_subtitles_pod_ep(&podcast_episode).await;
                all_subtitles_pod_ep.push(sub);
                let seasons = collect_seasons_pod_ep(&podcast_episode).await;
                all_seasons_pod_ep.push(seasons);
                let numep = collect_episodes_pod_ep(&podcast_episode).await;
                all_episodes_pod_ep.push(numep);
                let authors = collect_authors_pod_ep(&podcast_episode).await;
                all_authors_pod_ep.push(authors);
                let desc = collect_descs_pod_ep(&podcast_episode).await;
                all_descs_pod_ep.push(desc);
                let title_pod = collect_titles_pod(&podcast_episode).await;
                all_titles_pod.push(title_pod);
                let duration = collect_durations_pod_ep(&podcast_episode).await;
                all_durations_pod_ep.push(duration);
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
        ];

        // init for `SettingsAccount`
        let mut all_usernames: Vec<String> = Vec::new();
        let mut all_server_addresses: Vec<String> = Vec::new();
        if let Some(var_username) = database.default_usr.first() {
            all_usernames.push(var_username.clone());
        }
        if let Some(var_server_address) = database.default_usr.get(1) {
            all_server_addresses.push(var_server_address.clone());
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

        let token_of_the_engine = token.clone();
        let start_of_the_engine =
            tokio::task::spawn_blocking(move || PlayerHandle::start(token_of_the_engine));

        let outcome =
            match tokio::time::timeout(TIME_FOR_THE_SOUND_DEVICE, start_of_the_engine).await {
                Ok(Ok(outcome)) => outcome,
                Ok(Err(error)) => Err(format!("the thread of the sound device stopped: {}", error)),
                Err(_) => Err(format!(
                    "the sound device did not answer in {} seconds",
                    TIME_FOR_THE_SOUND_DEVICE.as_secs()
                )),
            };

        let (player, audio_fault) = match outcome {
            Ok(player) => (player, None),
            Err(error) => {
                log::error!("[app] the audio engine did not start: {}", error);
                let (player, receiver) = PlayerHandle::without_engine();

                // Nothing reads the commands of a player with no engine. A
                // thread takes them and drops them, so that a key of the
                // playback does not fill the memory.
                std::thread::spawn(move || while receiver.recv().is_ok() {});

                (player, Some(error.to_string()))
            }
        };

        let waiting_progress = count_pending_progress(&username, &server_key);

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
            series,
            library_rows,
            home_rows,
            library_sort,
            library_desc,
            library_filter,
            list_state_sort_filter,
            list_state_chapters: ListState::default(),
            list_state_bookmarks: ListState::default(),
            list_state_queue: ListState::default(),
            bookmarks_of: String::new(),
            sleep: None,
            sleep_choice: None,
            list_state_new_podcast: ListState::default(),
            list_state_authors: ListState::default(),
            list_state_keys: ListState::default(),
            the_view_before_the_keys: AppView::Home,
            the_view_before_the_reader: AppView::Home,
            must_refresh: false,
            series_from: AppView::Series,
            lists,
            is_offline,
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
            all_usernames,
            all_server_addresses,
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
            permissions,
            confirm_logout: None,
            reader: None,
            reader_message: None,
            stats_scroll: 0,
            sessions_scroll: 0,
            sessions_scroll_max: 0,
            stats_scroll_max: 0,
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

            // The key that tells the server to get the new episodes of a
            // podcast. See T-24.
            KeyCode::Char('E') => self.get_the_new_episodes(),

            // The key that looks for a new podcast. See T-24.
            KeyCode::Char('A') => self.look_for_a_podcast(),

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
                handle_key_player(" ", &self.player, self.username.as_str());
            }
            // jump forward
            KeyCode::Char('p') => {
                handle_key_player("p", &self.player, self.username.as_str());
            }

            // jump backward
            KeyCode::Char('u') => {
                handle_key_player("u", &self.player, self.username.as_str());
            }

            // next chapter
            KeyCode::Char('P') => {
                handle_key_player("P", &self.player, self.username.as_str());
            }

            // previous chapter
            KeyCode::Char('U') => {
                handle_key_player("U", &self.player, self.username.as_str());
            }

            // speed rate up
            KeyCode::Char('O') => {
                handle_key_player("O", &self.player, self.username.as_str());
            }

            // speed rate down
            KeyCode::Char('I') => {
                handle_key_player("I", &self.player, self.username.as_str());
            }

            // volume up
            KeyCode::Char('o') => {
                handle_key_player("o", &self.player, self.username.as_str());
            }

            // volume down
            KeyCode::Char('i') => {
                handle_key_player("i", &self.player, self.username.as_str());
            }

            // stop the playback
            KeyCode::Char('Y') => {
                handle_key_player("Y", &self.player, self.username.as_str());
            }

            // show key bindings
            KeyCode::Char('B') => {
                let value = get_is_show_key_bindings(self.username.as_str());
                if value == "0" {
                    let _ = update_is_show_key_bindings("1", self.username.as_str());
                } else if value == "1" {
                    let _ = update_is_show_key_bindings("0", self.username.as_str());
                }
            }

            // END PLAYER //

            // download the selected book or episode for offline listening
            KeyCode::Char('D') => {
                // The server refuses a download for an account that may not
                // download, and it gives an error of the protocol. The user
                // reads a sentence instead. See T-24.
                if !self.permissions.download {
                    let mut stdout = stdout();
                    let _ = clear_message(&mut stdout, 3);
                    let _ = pop_message(&mut stdout, 3, crate::api::me::permissions::no_download());
                    return;
                }

                let token = self.token.clone();
                let server_address = self.server_address.clone();
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

            // remove the local copy of the selected book or episode
            KeyCode::Char('X') => {
                let username = self.username.clone();

                if let Some((target, _title, _author)) = self.selected_download() {
                    if let Some(title) = remove_download(target.key(), &username) {
                        let mut stdout = stdout();
                        let _ = pop_message(
                            &mut stdout,
                            3,
                            &format!("Removed offline copy of \"{}\".", title),
                        );
                    }
                }
            }

            // show the series of the library
            KeyCode::Char('s') => {
                if !self.is_podcast {
                    match self.view_state {
                        AppView::Home | AppView::Library | AppView::SearchBook => {
                            self.list_state_series.select(Some(0));
                            self.scroll_offset = 0;
                            self.view_state = AppView::Series;
                        }
                        _ => {}
                    }
                }
            }

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
                let mut stdout = stdout();

                if state.status == crate::player::engine::PlaybackStatus::Stopped
                    || !crate::logic::sync_session::force_sync::ask(state.playback_id)
                {
                    let _ = clear_message(&mut stdout, 3);
                    let _ = pop_message(&mut stdout, 3, "Sync: nothing plays now.");
                } else {
                    let _ = clear_message(&mut stdout, 3);
                    let _ =
                        pop_message(&mut stdout, 3, "Sync: the application sends the position…");

                    // The answer comes from the loop of the playback. This
                    // task waits for it, and it stops after a short time when
                    // no answer comes.
                    tokio::spawn(async move {
                        for _ in 0..40 {
                            tokio::time::sleep(std::time::Duration::from_millis(200)).await;

                            if let Some(text) =
                                crate::logic::sync_session::force_sync::take_report()
                            {
                                let mut stdout = std::io::stdout();
                                let _ = clear_message(&mut stdout, 3);
                                let _ = pop_message(&mut stdout, 3, text.as_str());
                                return;
                            }
                        }

                        let mut stdout = std::io::stdout();
                        let _ = clear_message(&mut stdout, 3);
                        let _ = pop_message(&mut stdout, 3, "Sync: the playback gave no answer.");
                    });
                }
            }
            KeyCode::Tab => {
                if self.is_from_search_pod {
                    self.is_from_search_pod = false;
                };
                self.toggle_view()
            }

            // `Esc` inside the list of every key closes that list. A key that
            // stops the whole program must not stand alone in a view that the
            // user opened to read. See T-49.
            KeyCode::Esc if matches!(self.view_state, AppView::Keys) => {
                self.view_state = self.the_view_before_the_keys;
            }

            KeyCode::Char('Q') | KeyCode::Esc => {
                // display message
                let message_quit = "Exiting the application and syncing data, please hold on.";
                let mut stdout = stdout();
                let _ = pop_message(&mut stdout, 3, message_quit);

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
                if self.is_podcast {
                    if let Some(index) = selected_library {
                        if let Some(_id_pod) = ids_library.get(index) {
                            self.ids_pod_ep = self.all_ids_pod_ep[index].clone();
                        }
                    }
                    if let Some(index) = selected_search_book {
                        // ids_library_pod_search because we need the pod id and he is given by
                        // this variable
                        if let Some(_id_pod) = self.ids_library_pod_search.get(index) {
                            //    println!("{:?}", id_pod);
                            self.ids_pod_ep_search = self.all_ids_pod_ep_search[index].clone();
                            //   println!("{:?}", all_ids_pod_ep_search_clone[index]);
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
                        Some(0) => self.view_state = AppView::SettingsAccount,
                        Some(1) => self.view_state = AppView::SettingsLibrary,
                        _ => {}
                    },
                    // The list can be shorter than the selection: the user
                    // removes an account, and the list of the accounts keeps
                    // its old length until the next refresh. An index of a
                    // vector stops the program. `get` does not. See T-41.
                    AppView::SettingsAccount => {
                        if let Some(usr_to_delete) =
                            selected_account.and_then(|index| self.all_usernames.get(index))
                        {
                            let usr_to_delete = usr_to_delete.clone();

                            // A log out forgets the token of a server, and the
                            // user then gives their password again. Therefore
                            // the program asks one time. Any key that is not
                            // `l` stops the question. See T-36.
                            if self.confirm_logout.as_deref() != Some(usr_to_delete.as_str()) {
                                self.confirm_logout = Some(usr_to_delete.clone());

                                let mut stdout = stdout();
                                let _ = clear_message(&mut stdout, 3);
                                let _ = pop_message(
                                    &mut stdout,
                                    3,
                                    format!(
                                        "Press l again to log out of \"{}\". Any other key stops this.",
                                        usr_to_delete
                                    )
                                    .as_str(),
                                );

                                return;
                            }

                            self.confirm_logout = None;
                            let _ = delete_user(usr_to_delete.as_str());

                            // The list must follow the change at once.
                            self.all_usernames.retain(|name| name != &usr_to_delete);

                            let last = self.all_usernames.len().saturating_sub(1);
                            if self.all_usernames.is_empty() {
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
                            let _ = update_id_selected_lib(new_selected_lib, &self.username);
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
                            if let Some(index) = selected_library {
                                self.titles_pod_ep = self.all_titles_pod_ep[index].clone();
                                self.subtitles_pod_ep = self.all_subtitles_pod_ep[index].clone();
                                self.seasons_pod_ep = self.all_seasons_pod_ep[index].clone();
                                self.episodes_pod_ep = self.all_episodes_pod_ep[index].clone();
                                self.authors_pod_ep = self.all_authors_pod_ep[index].clone();
                                self.descs_pod_ep = self.all_descs_pod_ep[index].clone();
                                self.titles_pod = self.all_titles_pod[index].clone();
                                self.durations_pod_ep = self.all_durations_pod_ep[index].clone();
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
                                self.titles_pod_ep_search =
                                    self.all_titles_pod_ep_search[index].clone();
                                self.subtitles_pod_ep_search =
                                    self.all_subtitles_pod_ep_search[index].clone();
                                self.seasons_pod_ep_search =
                                    self.all_seasons_pod_ep_search[index].clone();
                                self.episodes_pod_ep_search =
                                    self.all_episodes_pod_ep_search[index].clone();
                                self.authors_pod_ep_search =
                                    self.all_authors_pod_ep_search[index].clone();
                                self.descs_pod_ep_search =
                                    self.all_descs_pod_ep_search[index].clone();
                                self.titles_pod_search = self.all_titles_pod_search[index].clone();
                                self.durations_pod_ep_search =
                                    self.all_durations_pod_ep_search[index].clone();
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
                    AppView::Lists => {
                        if self
                            .selected_list()
                            .is_some_and(|list| !list.entries.is_empty())
                        {
                            self.list_state_list_entries.select(Some(0));
                            self.scroll_offset = 0;
                            self.view_state = AppView::ListEntries;
                        }
                    }
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
                                    let all_ids_pod_ep_clone = self.all_ids_pod_ep.clone();
                                    self.ids_pod_ep = all_ids_pod_ep_clone[index].clone();
                                    let id_pod_clone = id_pod.clone();
                                    let selected_pod_ep = self.list_state_pod_ep.selected();
                                    tokio::spawn(async move {
                                        if let Some(episode_id) = all_ids_pod_ep_clone[index]
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
        let mut stdout = stdout();
        let _ = clear_message(&mut stdout, 3);

        let Some(item_id) = self.selected_item_id() else {
            let _ = pop_message(&mut stdout, 3, "No media is selected.");
            return;
        };

        if self.is_offline {
            let _ = pop_message(&mut stdout, 3, "The server does not answer.");
            return;
        }

        let _ = pop_message(&mut stdout, 3, "The mark of the media goes to the server…");
        let api = std::sync::Arc::clone(&self.api);

        tokio::spawn(async move {
            let text = mark_the_media(&api, &item_id).await;

            let mut stdout = std::io::stdout();
            let _ = clear_message(&mut stdout, 3);
            let _ = pop_message(&mut stdout, 3, text.as_str());
        });
    }

    /// Takes the selected media away from the shelf of Continue Listening,
    /// or puts it back. See T-24.
    ///
    /// The field `hideFromContinueListening` of `PATCH /api/me/progress/:id`
    /// does this work. A user who does not want a book on the Home view had
    /// no way to take it away: the book stayed until they finished it.
    pub fn toggle_the_shelf_of_continue_listening(&mut self) {
        let mut stdout = stdout();
        let _ = clear_message(&mut stdout, 3);

        let Some(item_id) = self.selected_item_id() else {
            let _ = pop_message(&mut stdout, 3, "No media is selected.");
            return;
        };

        if self.is_offline {
            let _ = pop_message(&mut stdout, 3, "The server does not answer.");
            return;
        }

        let _ = pop_message(&mut stdout, 3, "The change goes to the server…");
        let api = std::sync::Arc::clone(&self.api);

        tokio::spawn(async move {
            let text = hide_the_media(&api, &item_id).await;

            let mut stdout = std::io::stdout();
            let _ = clear_message(&mut stdout, 3);
            let _ = pop_message(&mut stdout, 3, text.as_str());
        });
    }

    /// Shows the chapters of the media that plays. See T-24.
    ///
    /// The engine holds the chapters already: it uses them for the keys `P`
    /// and `U`. The user could not see them, and they could not go to a
    /// chapter by its name.
    pub fn show_the_chapters(&mut self) {
        let mut stdout = stdout();
        let _ = clear_message(&mut stdout, 3);

        let state = self.player.state();

        if state.status == crate::player::engine::PlaybackStatus::Stopped {
            let _ = pop_message(&mut stdout, 3, "No media plays now.");
            return;
        }

        if state.chapters.is_empty() {
            let _ = pop_message(&mut stdout, 3, "This media has no chapter.");
            return;
        }

        // The selection starts at the chapter that plays.
        let now = crate::logic::chapters::chapter_at(&state.chapters, state.position);

        self.list_state_chapters.select(Some(now.unwrap_or(0)));
        self.scroll_offset = 0;
        self.view_state = AppView::Chapters;
    }

    /// Goes to the chapter that the user selected.
    pub fn go_to_the_chapter(&mut self) {
        let state = self.player.state();

        let Some(index) = self.list_state_chapters.selected() else {
            return;
        };

        let Some(chapter) = state.chapters.get(index) else {
            return;
        };

        self.player
            .send(crate::player::engine::PlayerCommand::SeekTo(chapter.start));

        let mut stdout = stdout();
        let _ = clear_message(&mut stdout, 3);
        let _ = pop_message(
            &mut stdout,
            3,
            &format!("The playback goes to \"{}\".", chapter.title),
        );
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
        let mut stdout = stdout();
        let _ = clear_message(&mut stdout, 3);

        if self.is_offline {
            let _ = pop_message(&mut stdout, 3, "The server does not answer.");
            return;
        }

        if self.id_selected_lib.is_empty() {
            let _ = pop_message(&mut stdout, 3, "No library is selected.");
            return;
        }

        let api = std::sync::Arc::clone(&self.api);
        let library = self.id_selected_lib.clone();

        let _ = pop_message(&mut stdout, 3, "The server examines the library…");

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

            let mut stdout = std::io::stdout();
            let _ = clear_message(&mut stdout, 3);
            let _ = pop_message(&mut stdout, 3, text.as_str());
        });
    }

    /// Shows the authors of the library, and asks the server for them.
    ///
    /// A library of podcasts has no author. See T-24.
    pub fn show_the_authors(&mut self) {
        let mut stdout = stdout();
        let _ = clear_message(&mut stdout, 3);

        if !matches!(
            self.view_state,
            AppView::Home | AppView::Library | AppView::SearchBook | AppView::Authors
        ) {
            return;
        }

        if self.is_podcast {
            let _ = pop_message(&mut stdout, 3, "A library of podcasts has no author.");
            return;
        }

        self.list_state_authors.select(Some(0));
        self.scroll_offset = 0;
        self.view_state = AppView::Authors;

        if self.is_offline {
            crate::logic::authors::keep(crate::logic::authors::State::Fault(
                "the server does not answer".to_string(),
            ));
            return;
        }

        // The authors of a library do not change while the program runs.
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
            let state = match crate::api::libraries::get_authors::get_authors(&api, &library).await
            {
                Ok(all) => crate::logic::authors::State::Ready(all),
                Err(error) => {
                    log::warn!("[authors] the server gave no author: {}", error);
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

        self.library_filter = crate::logic::sort_filter::filter_value("authors", &author.id);

        let _ = crate::db::crud::update_library_sort(
            &self.username,
            &self.library_sort,
            self.library_desc,
            &self.library_filter,
        );

        self.must_refresh = true;
    }

    /// Tells the server to get the episodes that it does not hold. See T-24.
    ///
    /// The key `D` copies a media to the disk of the user. This key is a
    /// different work: the server gets the file and it puts it in the library
    /// of the server, therefore every client can play it.
    pub fn get_the_new_episodes(&mut self) {
        let mut stdout = stdout();
        let _ = clear_message(&mut stdout, 3);

        if !self.is_podcast {
            let _ = pop_message(&mut stdout, 3, "This library holds books.");
            return;
        }

        if self.is_offline {
            let _ = pop_message(&mut stdout, 3, "The server does not answer.");
            return;
        }

        // The view of the episodes belongs to one podcast, and the Library
        // view gives the podcast of the line.
        let item_id = match self.view_state {
            AppView::PodcastEpisode => self.podcast_of_the_episodes(),
            _ => self.selected_item_id(),
        };

        let Some(item_id) = item_id else {
            let _ = pop_message(&mut stdout, 3, "No podcast is selected.");
            return;
        };

        let api = std::sync::Arc::clone(&self.api);
        let _ = pop_message(&mut stdout, 3, "The server reads the feed…");

        tokio::spawn(async move {
            let text = ask_the_server_for_the_episodes(&api, &item_id).await;

            let mut stdout = std::io::stdout();
            let _ = clear_message(&mut stdout, 3);
            let _ = pop_message(&mut stdout, 3, text.as_str());
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
        let mut stdout = stdout();
        let _ = clear_message(&mut stdout, 3);

        if !matches!(
            self.view_state,
            AppView::Home | AppView::Library | AppView::SearchBook | AppView::NewPodcast
        ) {
            return;
        }

        if !self.is_podcast {
            let _ = pop_message(
                &mut stdout,
                3,
                "This library holds books. Choose a library of podcasts with S.",
            );
            return;
        }

        if self.is_offline {
            let _ = pop_message(&mut stdout, 3, "The server does not answer.");
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
        let mut stdout = stdout();
        let _ = clear_message(&mut stdout, 3);

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
            let _ = pop_message(&mut stdout, 3, "This answer of the server holds no feed.");
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
            let _ = clear_message(&mut stdout, 3);
            let _ = pop_message(&mut stdout, 3, "The program added no podcast.");
            return;
        }

        let api = std::sync::Arc::clone(&self.api);
        let library = self.id_selected_lib.clone();
        let feed_url = found.feed_url.clone();
        let title = found.title.clone();

        let _ = clear_message(&mut stdout, 3);
        let _ = pop_message(&mut stdout, 3, "The server reads the feed…");

        tokio::spawn(async move {
            let text = add_a_podcast(&api, &library, &feed_url, &title).await;

            let mut stdout = std::io::stdout();
            let _ = clear_message(&mut stdout, 3);
            let _ = pop_message(&mut stdout, 3, text.as_str());
        });
    }

    /// Moves the timer for sleep to its next choice. See T-24.
    ///
    /// The key gives 5, 10, 15, 30, 45, and 60 minutes, the end of the
    /// chapter, and then off. The volume falls in the last 30 seconds, and
    /// the playback then pauses.
    pub fn change_the_timer_for_sleep(&mut self) {
        use crate::logic::sleep_timer as sleep;

        let mut stdout = stdout();
        let _ = clear_message(&mut stdout, 3);

        let state = self.player.state();

        if state.status == crate::player::engine::PlaybackStatus::Stopped {
            let _ = pop_message(&mut stdout, 3, "No media plays now.");
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
            let _ = pop_message(&mut stdout, 3, "The timer for sleep is off.");
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
                    let _ = pop_message(&mut stdout, 3, "This media has no chapter.");
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

        let _ = pop_message(
            &mut stdout,
            3,
            &format!("The playback stops after {}.", sleep::label_of(choice)),
        );
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

                let mut stdout = stdout();
                let _ = clear_message(&mut stdout, 3);
                let _ = pop_message(&mut stdout, 3, "The timer for sleep stopped the playback.");
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
        let mut stdout = stdout();
        let _ = clear_message(&mut stdout, 3);

        let state = self.player.state();

        if state.status == crate::player::engine::PlaybackStatus::Stopped {
            let _ = pop_message(&mut stdout, 3, "No media plays now.");
            return;
        }

        if self.is_offline {
            let _ = pop_message(&mut stdout, 3, "The server does not answer.");
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
        let _ = clear_message(&mut stdout, 3);
        let _ = pop_message(&mut stdout, 3, "The bookmark goes to the server…");

        tokio::spawn(async move {
            let text =
                match crate::api::me::bookmarks::add_bookmark(&api, &item_id, place, &name).await {
                    Ok(()) => format!("The bookmark \"{}\" is on the server.", name),
                    Err(error) => format!("The server did not take the bookmark: {}", error),
                };

            // A bookmark that came now must stand in the view.
            crate::logic::bookmarks::forget();

            let mut stdout = std::io::stdout();
            let _ = clear_message(&mut stdout, 3);
            let _ = pop_message(&mut stdout, 3, text.as_str());
        });
    }

    /// Shows the bookmarks of a media, and asks the server for them.
    ///
    /// The media that plays comes first, because a user who listens looks for
    /// a place of that media. A media that plays no media gives the media of
    /// the line that the user selected.
    pub fn show_the_bookmarks(&mut self) {
        let mut stdout = stdout();
        let _ = clear_message(&mut stdout, 3);

        let state = self.player.state();

        let item_id = if state.status != crate::player::engine::PlaybackStatus::Stopped {
            state.item_id.clone()
        } else {
            match self.selected_item_id() {
                Some(id) => id,
                None => {
                    let _ =
                        pop_message(&mut stdout, 3, "No media plays, and no media is selected.");
                    return;
                }
            }
        };

        self.bookmarks_of = item_id.clone();
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
        let mut stdout = stdout();
        let _ = clear_message(&mut stdout, 3);

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
            let _ = pop_message(
                &mut stdout,
                3,
                "Play this media first, and the bookmark then gives its place.",
            );
            return;
        }

        self.player
            .send(crate::player::engine::PlayerCommand::SeekTo(bookmark.time));

        let _ = pop_message(
            &mut stdout,
            3,
            &format!("The playback goes to \"{}\".", bookmark.title),
        );
    }

    /// Removes the bookmark that the user selected. See T-24.
    pub fn remove_the_bookmark(&mut self) {
        let mut stdout = stdout();
        let _ = clear_message(&mut stdout, 3);

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
            let _ = pop_message(&mut stdout, 3, "The server does not answer.");
            return;
        }

        let api = std::sync::Arc::clone(&self.api);
        let item_id = bookmark.library_item_id.clone();
        let name = bookmark.title.clone();
        let time = bookmark.time;

        let _ = pop_message(&mut stdout, 3, "The program removes the bookmark…");

        tokio::spawn(async move {
            let text = match crate::api::me::bookmarks::remove_bookmark(&api, &item_id, time).await
            {
                Ok(()) => format!("The bookmark \"{}\" is not on the server now.", name),
                Err(error) => format!("The server did not remove the bookmark: {}", error),
            };

            crate::logic::bookmarks::forget();

            let mut stdout = std::io::stdout();
            let _ = clear_message(&mut stdout, 3);
            let _ = pop_message(&mut stdout, 3, text.as_str());
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
                    Ok(data) => crate::logic::sort_filter::from_the_server::State::Ready(
                        crate::api::libraries::get_filter_data::choices(&data),
                    ),
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

        let _ = crate::db::crud::update_library_sort(
            &self.username,
            &self.library_sort,
            self.library_desc,
            &self.library_filter,
        );

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

            crate::logic::search::from_the_server::keep(
                crate::logic::search::from_the_server::Answer {
                    words,
                    items: crate::api::libraries::search_library::items_of(&answer),
                    names: crate::api::libraries::search_library::names_of(&answer),
                },
            );
        });
    }

    /// Gives the identity of the item that the user selected, in any view of
    /// media.
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

    /// Opens the ebook of the item that the user selected. See T-10.
    ///
    /// The program keeps the file in the directory of the downloads. Therefore
    /// a second visit needs no request, and the reader also works with no
    /// server.
    pub fn open_the_ebook(&mut self) {
        let Some(item_id) = self.selected_item_id() else {
            return;
        };

        // A book that the reader holds already needs no work.
        if self
            .reader
            .as_ref()
            .is_some_and(|reader| reader.item_id == item_id)
        {
            if !matches!(self.view_state, AppView::Reader) {
                self.the_view_before_the_reader = self.view_state;
            }

            self.view_state = AppView::Reader;
            return;
        }

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
            let outcome =
                match crate::logic::reader::session::get_the_ebook(&api, &username, &item_id).await
                {
                    Ok(path) => crate::logic::reader::Reader::open(&path, &item_id)
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

                    // The user reads the same book on a different machine. The
                    // program opens the book where they stopped. See T-10,
                    // section 6.
                    if let Some((location, part)) =
                        crate::logic::reader::session::place_of_the_server(&api, &item_id).await
                    {
                        reader.go_to_the_place_of_the_server(&location, part);
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
            _ => {}
        }
    }

    /// Sends the place of the reader to the server. See T-10, section 6.
    pub fn send_the_place_of_the_reader(&mut self) {
        let Some(reader) = self.reader.as_ref() else {
            return;
        };

        let item_id = reader.item_id.clone();
        let location = reader.location_text();
        let part = reader.fraction();
        let api = std::sync::Arc::clone(&self.api);

        // The reader remembers the place that it sent. It then sends nothing
        // while the user reads the same line.
        if let Some(reader) = self.reader.as_mut() {
            reader.the_place_went_to_the_server();
        }

        let mut stdout = stdout();
        let _ = clear_message(&mut stdout, 3);
        let _ = pop_message(&mut stdout, 3, "The place of the book goes to the server…");

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

            let mut stdout = std::io::stdout();
            let _ = clear_message(&mut stdout, 3);
            let _ = pop_message(&mut stdout, 3, text.as_str());
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
                let index = self.list_state_search_results.selected()?;
                let id = self.ids_search_book.get(index)?.clone();
                let in_library = self.ids_library.iter().position(|x| x == &id)?;

                Some((
                    DownloadTarget::Book { item_id: id },
                    self.titles_library.get(in_library)?.clone(),
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
        let mut stdout = stdout();
        let _ = clear_message(&mut stdout, 3);

        let Some(entry) = self.selected_media() else {
            let _ = pop_message(&mut stdout, 3, "This line holds no media.");
            return;
        };

        let title = entry.title.clone();
        let place = crate::logic::queue::add(entry);

        let _ = pop_message(
            &mut stdout,
            3,
            &format!(
                "\"{}\" is number {} of the queue. Press q to see the queue.",
                title, place
            ),
        );
    }

    /// Shows every key of the program. The key is `?`. See T-49.
    ///
    /// The key a second time gives the view of the user back. Therefore the
    /// list is a look at the keys, and it takes no place of the work.
    pub fn show_every_key(&mut self) {
        let mut stdout = stdout();
        let _ = clear_message(&mut stdout, 3);

        if matches!(self.view_state, AppView::Keys) {
            self.view_state = self.the_view_before_the_keys;
            return;
        }

        self.the_view_before_the_keys = self.view_state;
        self.list_state_keys.select(Some(0));
        self.view_state = AppView::Keys;
    }

    /// Shows the media that wait in the queue. The key is `q`.
    pub fn show_the_queue(&mut self) {
        let mut stdout = stdout();
        let _ = clear_message(&mut stdout, 3);

        let count = crate::logic::queue::len();

        // The selection must stand inside the list. An empty queue has no
        // line to select.
        self.list_state_queue.select(if count == 0 {
            None
        } else {
            Some(self.list_state_queue.selected().unwrap_or(0).min(count - 1))
        });

        self.scroll_offset = 0;
        self.view_state = AppView::Queue;
    }

    /// Takes the selected media out of the queue. The key is `X` inside the
    /// view of the queue.
    pub fn remove_from_the_queue(&mut self) {
        let mut stdout = stdout();
        let _ = clear_message(&mut stdout, 3);

        let Some(index) = self.list_state_queue.selected() else {
            return;
        };

        let Some(entry) = crate::logic::queue::take_at(index) else {
            return;
        };

        self.list_state_queue
            .select(crate::logic::queue::snapshot().selection_after_a_remove(index));

        let _ = pop_message(
            &mut stdout,
            3,
            &format!("\"{}\" is not in the queue now.", entry.title),
        );
    }

    /// Starts the selected media of the queue now. The key is `l` inside the
    /// view of the queue.
    ///
    /// The media goes out of the queue: it plays, therefore it does not wait.
    /// The media that plays now stops, in the same way as the key `l` in every
    /// other view.
    pub fn start_the_media_of_the_queue(&mut self) {
        let mut stdout = stdout();
        let _ = clear_message(&mut stdout, 3);

        let Some(index) = self.list_state_queue.selected() else {
            return;
        };

        let Some(entry) = crate::logic::queue::take_at(index) else {
            return;
        };

        self.list_state_queue
            .select(crate::logic::queue::snapshot().selection_after_a_remove(index));

        let api = std::sync::Arc::clone(&self.api);
        let player = self.player.clone();
        let username = self.username.clone();
        let server_address = self.server_address.clone();
        let server_key = self.server_key.clone();

        tokio::spawn(async move {
            play(
                &api,
                &player,
                entry.target,
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
            AppView::Settings => AppView::Home,
            AppView::SettingsAccount => AppView::Home,
            AppView::SettingsLibrary => AppView::Home,
            AppView::SettingsAbout => AppView::Home,
            AppView::SettingsUpdateUninstall => AppView::Home,
        };
    }

    /// Select functions that apply to both views
    /// all select functions are from ListState widget
    pub fn select_next(&mut self) {
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
            AppView::Lists => {
                if let Some(selected) = self.list_state_lists.selected() {
                    if selected + 1 < self.lists.len() {
                        self.list_state_lists.select_next();
                    } else {
                        self.list_state_lists.select_first();
                    }
                }
            }
            AppView::ListEntries => {
                if let Some(selected) = self.list_state_list_entries.selected() {
                    if selected + 1 < self.selected_list().map_or(0, |l| l.entries.len()) {
                        self.list_state_list_entries.select_next();
                    } else {
                        self.list_state_list_entries.select_first();
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
                    if selected + 1 < self.all_usernames.len() {
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
            AppView::Keys => self.list_state_keys.select_previous(),
            AppView::Settings => self.list_state_settings.select_previous(),
            AppView::SettingsAccount => self.list_state_settings_account.select_previous(),
            AppView::SettingsLibrary => self.list_state_settings_library.select_previous(),
            AppView::SettingsAbout => self.list_state_settings_about.select_previous(),
            AppView::SettingsUpdateUninstall => {
                self.list_state_settings_update_uninstall.select_previous()
            }
        }
    }

    pub fn select_first(&mut self) {
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
            AppView::Keys => self.list_state_keys.select_first(),
            AppView::Settings => self.list_state_settings.select_first(),
            AppView::SettingsAccount => self.list_state_settings_account.select_first(),
            AppView::SettingsLibrary => self.list_state_settings_library.select_first(),
            AppView::SettingsAbout => self.list_state_settings_about.select_first(),
            AppView::SettingsUpdateUninstall => {
                self.list_state_settings_update_uninstall.select_first()
            }
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
            AppView::Library => {
                let last_index = self.library_rows.len().saturating_sub(1);
                self.list_state_library.select(Some(last_index));
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

/// Changes the mark "finished" of one media on the server.
///
/// The function reads the condition of the media, and it then sends the
/// opposite. It gives the text for the user.
///
/// **A media that goes to "not finished" loses its position.** A measurement
/// against an Audiobookshelf 2.36.0 on 2026-08-11 sent `isFinished: false` and
/// read `currentTime: 0` and `progress: 0` back. The server does that, and
/// this program tells the user. See T-24.
pub async fn mark_the_media(
    api: &std::sync::Arc<crate::api::client::ApiClient>,
    item_id: &str,
) -> String {
    let answer: serde_json::Value =
        match api.get_json(&format!("/api/me/progress/{}", item_id)).await {
            Ok(answer) => answer,
            // A media that never played has no progress, and the server gives an
            // error. Such a media is not finished.
            Err(_) => serde_json::json!({}),
        };

    let was_finished = answer
        .get("isFinished")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);

    let body = serde_json::json!({ "isFinished": !was_finished });

    match api
        .patch_json(&format!("/api/me/progress/{}", item_id), &body)
        .await
    {
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
pub async fn hide_the_media(
    api: &std::sync::Arc<crate::api::client::ApiClient>,
    item_id: &str,
) -> String {
    let answer: serde_json::Value =
        match api.get_json(&format!("/api/me/progress/{}", item_id)).await {
            Ok(answer) => answer,
            Err(_) => serde_json::json!({}),
        };

    let was_hidden = answer
        .get("hideFromContinueListening")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);

    let body = serde_json::json!({ "hideFromContinueListening": !was_hidden });

    match api
        .patch_json(&format!("/api/me/progress/{}", item_id), &body)
        .await
    {
        Ok(()) => message_of_the_shelf(!was_hidden),
        Err(error) => format!("The server did not take the change: {}", error),
    }
}

/// Gives the text that the user reads after a change of the shelf.
pub fn message_of_the_shelf(hidden: bool) -> String {
    if hidden {
        "The media is away from Continue Listening now. Press R to see the change.".to_string()
    } else {
        "The media is on Continue Listening again. Press R to see the change.".to_string()
    }
}

/// Gives the text that the user reads after a change of the mark.
pub fn message_of_the_mark(finished: bool) -> String {
    if finished {
        "The media is finished now. Press R to see the change.".to_string()
    } else {
        "The media is not finished now, and its position went back to the \
         start. Press R to see the change."
            .to_string()
    }
}
