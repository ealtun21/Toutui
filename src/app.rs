use crate::api::utils::collect_personalized_view::*;
use crate::api::utils::collect_personalized_view_pod::*;
use crate::api::utils::collect_get_all_books::*;
use crate::api::utils::collect_get_pod_ep::*;
use crate::api::utils::collect_get_all_libraries::*;
use crate::api::utils::collect_get_media_progress::*;
use crate::api::me::get_media_progress::*;
use crate::api::libraries::get_library_perso_view::*;
use crate::api::libraries::get_library_perso_view_pod::*;
use crate::api::libraries::get_all_books::*;
use crate::api::libraries::get_all_libraries::*;
use crate::api::library_items::get_pod_ep::*;
use crate::config::*;
use crate::db::crud::*;
use crate::db::database_struct::Database;
use color_eyre::Result;
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent, KeyEventKind},
    widgets::ListState,
};
use crate::utils::pop_up_message::*;
use crate::utils::changelog::*;
use crate::utils::encrypt_token::*;
use std::io::stdout;
use crate::logic::sync_session::sync_session_from_database::*;
use crate::player::integrated::handle_key_player::*;
use crate::player::engine::PlayerHandle;
use crate::logic::playback::{play, PlaybackTarget};
use crate::utils::check_update::*;
use crate::logic::download::{DownloadTarget, download_with_progress, remove_download};

pub enum AppView {
    Home,
    Library,
    SearchBook,
    PodcastEpisode,
    Settings,
    SettingsAccount,
    SettingsLibrary,
    SettingsAbout,
    SettingsUpdateUninstall
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
    pub config: ConfigFile,
    pub changelog: String,
    pub update_msg: String,
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

        // init for `Libraries` (get all Libraries (shelf), can be a podcast or book type)
        let all_libraries = get_all_libraries(&api).await?;
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
    let lib_name_type = format!("📖 {} ({})", library_name, media_type);

    // init is_podcast
    let is_podcast = media_type == "podcast";

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

    if is_podcast {
        // init for  `Home` (continue listening) for podcasts
        let continue_listening_pod = get_continue_listening_pod(&api, &id_selected_lib).await?;
        _ids_cnt_list = collect_ids_pod_cnt_list(&continue_listening_pod).await; // id of a podcast
        _titles_cnt_list = collect_titles_cnt_list_pod(&continue_listening_pod).await; // title of podcast ep
        ids_ep_cnt_list = collect_ids_ep_pod_cnt_list(&continue_listening_pod).await; // id of a podcast episode
        subtitles_pod_cnt_list = collect_subtitles_pod_cnt_list(&continue_listening_pod).await;
        nums_ep_pod_cnt_list = collect_nums_ep_pod_cnt_list(&continue_listening_pod).await;
        seasons_pod_cnt_list = collect_seasons_pod_cnt_list(&continue_listening_pod).await;
        authors_pod_cnt_list = collect_authors_pod_cnt_list(&continue_listening_pod).await;
        descs_pod_cnt_list = collect_descs_pod_cnt_list(&continue_listening_pod).await;
        titles_pod_cnt_list = collect_titles_pod_cnt_list(&continue_listening_pod).await; // title of a podcast
        durations_pod_cnt_list = collect_durations_pod_cnt_list(&continue_listening_pod).await;
    }
    else {
        // init for  `Home` (continue listening) for books
        let continue_listening = get_continue_listening(&api, &id_selected_lib).await?;
        _titles_cnt_list = collect_titles_cnt_list(&continue_listening).await;
        auth_names_cnt_list = collect_auth_names_cnt_list(&continue_listening).await;
        pub_year_cnt_list = collect_pub_year_cnt_list(&continue_listening).await;
        duration_cnt_list = collect_duration_cnt_list(&continue_listening).await;
        desc_cnt_list = collect_desc_cnt_list(&continue_listening).await;
        _ids_cnt_list = collect_ids_cnt_list(&continue_listening).await;
        for id in _ids_cnt_list.clone() {
            if let Ok(val) = get_book_progress(&api, &id).await {
                let mut values: Vec<String> = Vec::new();
                let mut values_f64: Vec<f64> = Vec::new();
                values.push(collect_progress_percentage_book(&val).await);
                values.push(collect_is_finished_book(&val).await);
                values_f64.push(collect_current_time_prg(&val).await);
                book_progress_cnt_list.push(values);
                book_progress_cnt_list_cur_time.push(values_f64);
            } else {
                // if the book is not starded, `get book progress` is not fetched
                // so the empty values are handled here : 
                // avoid an out of bound panick
                let mut values: Vec<String> = Vec::new();
                let mut values_f64: Vec<f64> = Vec::new();
                values.push(" N/A".to_string());
                values.push(" N/A".to_string());
                values_f64.push(0.0);
                book_progress_cnt_list.push(values);
                book_progress_cnt_list_cur_time.push(values_f64);
            }}}

    //init for `Library ` (all books  or podcasts of a Library (shelf))
    let all_books = get_all_books(&api, &id_selected_lib).await?;
    let titles_library = collect_titles_library(&all_books).await;
    let ids_library = collect_ids_library(&all_books).await;
    let auth_names_library = collect_auth_names_library(&all_books).await; // for a book
    let auth_names_library_pod = collect_auth_names_library_pod(&all_books).await; // for a podcast
    let published_year_library = collect_published_year_library(&all_books).await;
    let desc_library = collect_desc_library(&all_books).await;
    let duration_library = collect_duration_library(&all_books).await;
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
    let search_query = "  ".to_string();
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
    for id_library in ids_library.iter()
    {let podcast_episode = get_pod_ep(&api, id_library.as_str()).await?;
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
    let settings = vec!["Account".to_string(), "Library".to_string(), "About".to_string(), "Update and uninstall".to_string()];

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
    let player = match PlayerHandle::start(token.clone()) {
        Ok(player) => player,
        Err(error) => {
            eprintln!("{}", error);
            return Err(color_eyre::eyre::eyre!(error));
        }
    };

    // Init for check_update
    let update_msg = match check_update().await {
        Some(msg) => msg,
        None => "".to_string(),
    };

    // Init ListeState for `Home` list (continue listening)
    let mut list_state_cnt_list = ListState::default(); // init the ListState ratatui's widget
    list_state_cnt_list.select(Some(0)); // select the first item of the list when app is launch

    // Init ListeState for `Library` list
    let mut list_state_library = ListState::default(); 
    list_state_library.select(Some(0)); 

    // Init ListeState for `SearchBook` list
    let mut list_state_search_results = ListState::default(); 
    list_state_search_results.select(Some(0)); 

    // Init ListState for `PodacastEpisode` list
    let mut list_state_pod_ep = ListState::default();
    list_state_pod_ep.select(Some(0));

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
    })
    }


// handle key
pub fn handle_key(&mut self, key: KeyEvent) {
    if key.kind != KeyEventKind::Press {
        return;
    }


    match key.code {
        // PLAYER //
        // toggle playback/pause
        KeyCode::Char(' ') => {
            handle_key_player(" ", &self.player, self.username.as_str());}
        // jump forward
        KeyCode::Char('p') => {
            handle_key_player("p", &self.player, self.username.as_str());}

        // jump backward
        KeyCode::Char('u') => {
            handle_key_player("u", &self.player, self.username.as_str());}

        // next chapter
        KeyCode::Char('P') => {
            handle_key_player("P", &self.player, self.username.as_str());}

        // previous chapter
        KeyCode::Char('U') => {
            handle_key_player("U", &self.player, self.username.as_str());}

        // speed rate up
        KeyCode::Char('O') => {
            handle_key_player("O", &self.player, self.username.as_str());}

        // speed rate down
        KeyCode::Char('I') => {
            handle_key_player("I", &self.player, self.username.as_str());}

        // volume up
        KeyCode::Char('o') => {
            handle_key_player("o", &self.player, self.username.as_str());}

        // volume down
        KeyCode::Char('i') => {
            handle_key_player("i", &self.player, self.username.as_str());}

        // stop the playback
        KeyCode::Char('Y') => {
            handle_key_player("Y", &self.player, self.username.as_str());}

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
            let token = self.token.clone();
            let server_address = self.server_address.clone();
            let username = self.username.clone();

            if let Some((target, title, author)) = self.selected_download() {
                // The map is global. Therefore the bar stays correct when the
                // user refreshes the screen with the key `R`.
                let progress = crate::logic::download::downloads();
                tokio::spawn(async move {
                    download_with_progress(
                        token, target, server_address, username, title, author, progress,
                    )
                    .await;
                });
            }
        }

        // remove the local copy of the selected book or episode
        KeyCode::Char('X') => {
            let username = self.username.clone();

            if let Some((target, _title, _author)) = self.selected_download() {
                if let Some(title) = remove_download(target.key(), &username) {
                    let mut stdout = stdout();
                    let _ = pop_message(&mut stdout, 3, &format!("Removed offline copy of \"{}\".", title));
                }
            }
        }

        KeyCode::Char('/') => {
            let _ = self.search_active();
        }
        KeyCode::Char('S') => {
            self.view_state = AppView::Settings;
        }
        KeyCode::Tab => {
            if self.is_from_search_pod {
                self.is_from_search_pod = false;
            };
            self.toggle_view()
        }

        KeyCode::Char('Q') | KeyCode::Esc => {

            // display message 
            let message_quit = "Exiting the application and syncing data, please hold on.";
            let mut stdout = stdout();
            let _ = pop_message(&mut stdout, 3, message_quit);

            // close and sync session before close the app
            let api = std::sync::Arc::clone(&self.api);
            let username = self.username.clone();

            // Stop the engine before the application syncs and stops.
            self.player.send(crate::player::engine::PlayerCommand::Stop);

            tokio::spawn(async move {
                sync_session_from_database(&api, username, true, "Q").await;
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
                AppView::SettingsAccount => {self.view_state = AppView::Settings} 
                AppView::SettingsLibrary => {self.view_state = AppView::Settings} 
                AppView::SettingsAbout => {self.view_state = AppView::Settings} 
                AppView::SettingsUpdateUninstall => {self.view_state = AppView::Settings} 
                AppView::Settings => {self.view_state = AppView::Home} 
                AppView::PodcastEpisode => {
                    if self.is_from_search_pod {
                        self.view_state = AppView::SearchBook
                    } else {
                        self.view_state = AppView::Library
                    }
                }
                _ => {}
            }
        }        
        KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
            // Clone needed because variables will be used in a spawn
            let api = std::sync::Arc::clone(&self.api);
            let server_address = self.server_address.clone();
            let username = self.username.clone();
            let player = self.player.clone();

            // Init for `Continue Listening` (AppView::Home)
            let ids_cnt_list = self._ids_cnt_list.clone();
            let selected_cnt_list = self.list_state_cnt_list.selected();

            // Init for `Library`
            let ids_library = self.ids_library.clone();
            let selected_library = self.list_state_library.selected();

            // Init for `Search Book`
            let ids_search_book = self.ids_search_book.clone();
            let selected_search_book = self.list_state_search_results.selected();

            // Duration of the whole book, from the `media.duration` field. The playback
            // session gives the duration of the first audio file only, thus a book with
            // many audio files needs this value. See upstream issue #33.
            let whole_book_duration_cnt_list = selected_cnt_list
                .and_then(|i| self.duration_cnt_list.get(i).copied());
            let whole_book_duration_library = selected_library
                .and_then(|i| self.duration_library.get(i).copied());
            let whole_book_duration_search_book = selected_search_book
                .and_then(|i| self.duration_library_search_book.get(i).copied());

            // Init for `PodcastEpisode`
            if self.is_podcast {
                if let Some(index) = selected_library {
                    if let Some(_id_pod) = ids_library.get(index) {
                        self.ids_pod_ep = self.all_ids_pod_ep[index].clone();
                    }}
                if let Some(index) = selected_search_book {
                    // ids_library_pod_search because we need the pod id and he is given by
                    // this variable
                    if let Some(_id_pod) = self.ids_library_pod_search.get(index) {
                        //    println!("{:?}", id_pod);
                        self.ids_pod_ep_search = self.all_ids_pod_ep_search[index].clone();
                        //   println!("{:?}", all_ids_pod_ep_search_clone[index]);
                    }}
            }
            // Init for `SettingsAccount`
            let selected_account = self.list_state_settings_account.selected();

            // Init for `SettingsLibrary`
            let selected_settings_library = self.list_state_settings_library.selected();

            // Now, spawn the async task based on the current view state
            match self.view_state {
                AppView::Home => {
                    if self.is_podcast {
                        // init some variables
                        let _selected_pod_ep = self.list_state_pod_ep.selected();
                        let ids_ep_cnt_list = self.ids_ep_cnt_list.clone();

                        tokio::spawn(async move {
                            if let Some(episode_id) = selected_cnt_list.and_then(|i| ids_ep_cnt_list.get(i)).cloned() {
                                play(
                                    &api,
                                    &player,
                                    PlaybackTarget::Episode {
                                        item_id: ids_cnt_list[selected_cnt_list.unwrap_or(0)].clone(),
                                        episode_id,
                                    },
                                    username,
                                    server_address,
                                )
                                .await;
                            }
                        });
                    } else {


                        tokio::spawn(async move {
                            if let Some(item_id) = selected_cnt_list.and_then(|i| ids_cnt_list.get(i)).cloned() {
                                play(
                                    &api,
                                    &player,
                                    PlaybackTarget::Book {
                                        item_id,
                                        whole_book_duration: whole_book_duration_cnt_list,
                                    },
                                    username,
                                    server_address,
                                )
                                .await;
                            }
                        });

                    }}
                AppView::Settings => {
                    match self.list_state_settings.selected() {
                        Some(0) => self.view_state = AppView::SettingsAccount,
                        Some(1) => self.view_state = AppView::SettingsLibrary,
                        _ => {}
                    }
                }
                AppView::SettingsAccount => {
                    if let Some(index) = selected_account {
                        let usr_to_delete = &self.all_usernames[index];
                        let _ = delete_user(usr_to_delete.as_str());
                    }
                }
                AppView::SettingsLibrary => {
                    if let Some(index) = selected_settings_library {
                        let new_selected_lib = &self.libraries_ids[index];
                        let _ = update_id_selected_lib(new_selected_lib, &self.username);
                    }
                }
                AppView::SettingsAbout => {
                }
                AppView::SettingsUpdateUninstall => {
                }
                AppView::Library => {
                    if self.is_podcast {
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
                        }} else {


                            tokio::spawn(async move {
                                if let Some(item_id) = selected_library.and_then(|i| ids_library.get(i)).cloned() {
                                    play(
                                        &api,
                                        &player,
                                        PlaybackTarget::Book {
                                            item_id,
                                            whole_book_duration: whole_book_duration_library,
                                        },
                                        username,
                                        server_address,
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
                            self.titles_pod_ep_search = self.all_titles_pod_ep_search[index].clone();
                            self.subtitles_pod_ep_search = self.all_subtitles_pod_ep_search[index].clone();
                            self.seasons_pod_ep_search = self.all_seasons_pod_ep_search[index].clone();
                            self.episodes_pod_ep_search = self.all_episodes_pod_ep_search[index].clone();
                            self.authors_pod_ep_search = self.all_authors_pod_ep_search[index].clone();
                            self.descs_pod_ep_search = self.all_descs_pod_ep_search[index].clone();
                            self.titles_pod_search = self.all_titles_pod_search[index].clone();
                            self.durations_pod_ep_search = self.all_durations_pod_ep_search[index].clone();
                            self.list_state_pod_ep.select(Some(0));
                            self.view_state = AppView::PodcastEpisode;
                        }} else {


                            tokio::spawn(async move {
                                if let Some(item_id) = selected_search_book.and_then(|i| ids_search_book.get(i)).cloned() {
                                    play(
                                        &api,
                                        &player,
                                        PlaybackTarget::Book {
                                            item_id,
                                            whole_book_duration: whole_book_duration_search_book,
                                        },
                                        username,
                                        server_address,
                                    )
                                    .await;
                                }
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
                                let all_ids_pod_ep_search_clone = self.all_ids_pod_ep_search.clone();
                                //   println!("{:?}", all_ids_pod_ep_search_clone[index]);
                                let id_pod_clone = id_pod.clone();
                                let selected_pod_ep = self.list_state_pod_ep.selected();

                                tokio::spawn(async move {
                                    if let Some(episode_id) = all_ids_pod_ep_search_clone[index].get(selected_pod_ep.unwrap_or(0)).cloned() {
                                        play(
                                            &api,
                                            &player,
                                            PlaybackTarget::Episode {
                                                item_id: id_pod_clone,
                                                episode_id,
                                            },
                                            username,
                                            server_address,
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
                                    if let Some(episode_id) = all_ids_pod_ep_clone[index].get(selected_pod_ep.unwrap_or(0)).cloned() {
                                        play(
                                            &api,
                                            &player,
                                            PlaybackTarget::Episode {
                                                item_id: id_pod_clone,
                                                episode_id,
                                            },
                                            username,
                                            server_address,
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
            let index = self.list_state_cnt_list.selected()?;

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
            let index = self.list_state_cnt_list.selected()?;

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
            let index = self.list_state_library.selected()?;

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

/// Toggle between Home and Library views
fn toggle_view(&mut self) {
    self.view_state = match self.view_state {
        AppView::Home => AppView::Library,
        AppView::Library => AppView::Home,
        AppView::SearchBook => AppView::Home,
        AppView::PodcastEpisode => AppView::Home,
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
        AppView::Home => { if let Some(selected) = self.list_state_cnt_list.selected() {
            if selected + 1  < self._ids_cnt_list.len() {
                self.list_state_cnt_list.select_next();
            } else {
                self.list_state_cnt_list.select_first();
            }}}
        AppView::Library => { if let Some(selected) = self.list_state_library.selected() {
            if selected + 1  < self.ids_library.len() {
                self.list_state_library.select_next();
            } else {
                self.list_state_library.select_first();
            }}}
        AppView::SearchBook => { if let Some(selected) = self.list_state_search_results.selected() {
            if selected + 1  < self.ids_search_book.len() {
                self.list_state_search_results.select_next();
            } else {
                self.list_state_search_results.select_first();
            }}}
        AppView::PodcastEpisode => { if let Some(selected) = self.list_state_pod_ep.selected() {
            if self.is_from_search_pod {
                if selected + 1  < self.ids_pod_ep_search.len() {
                    self.list_state_pod_ep.select_next();
                } else {
                    self.list_state_pod_ep.select_first();
                }
            } else {
                if selected + 1  < self.ids_pod_ep.len() {
                    self.list_state_pod_ep.select_next();
                } else {
                    self.list_state_pod_ep.select_first();
                }}}}
        AppView::Settings => { if let Some(selected) = self.list_state_settings.selected() {
            if selected + 1  < self.settings.len() {
                self.list_state_settings.select_next();
            } else {
                self.list_state_settings.select_first();
            }}}
        AppView::SettingsAccount => self.list_state_settings_account.select_next(),
        AppView::SettingsLibrary => { if let Some(selected) = self.list_state_settings_library.selected() {
            if selected + 1  < self.media_types.len() {
                self.list_state_settings_library.select_next();
            } else {
                self.list_state_settings_library.select_first();
            }}}
        AppView::SettingsAbout => self.list_state_settings_about.select_next(),
        AppView::SettingsUpdateUninstall => self.list_state_settings_update_uninstall.select_next(),
    }
}

pub fn select_previous(&mut self) {
    match self.view_state {
        AppView::Home => self.list_state_cnt_list.select_previous(),
        AppView::Library => self.list_state_library.select_previous(),
        AppView::SearchBook => self.list_state_search_results.select_previous(),
        AppView::PodcastEpisode => self.list_state_pod_ep.select_previous(),
        AppView::Settings => self.list_state_settings.select_previous(),
        AppView::SettingsAccount => self.list_state_settings_account.select_previous(),
        AppView::SettingsLibrary => self.list_state_settings_library.select_previous(),
        AppView::SettingsAbout => self.list_state_settings_about.select_previous(),
        AppView::SettingsUpdateUninstall => self.list_state_settings_update_uninstall.select_previous(),
    }
}

pub fn select_first(&mut self) {
    match self.view_state {
        AppView::Home => self.list_state_cnt_list.select_first(),
        AppView::Library => self.list_state_library.select_first(),
        AppView::SearchBook => self.list_state_search_results.select_first(),
        AppView::PodcastEpisode => self.list_state_pod_ep.select_first(),
        AppView::Settings => self.list_state_settings.select_first(),
        AppView::SettingsAccount => self.list_state_settings_account.select_first(),
        AppView::SettingsLibrary => self.list_state_settings_library.select_first(),
        AppView::SettingsAbout => self.list_state_settings_about.select_first(),
        AppView::SettingsUpdateUninstall => self.list_state_settings_update_uninstall.select_first(),
    }
}

pub fn select_last(&mut self) {
    match self.view_state {
        AppView::Home => {
            let last_index = self._ids_cnt_list.len() - 1;
            self.list_state_cnt_list.select(Some(last_index));
        }            
        AppView::Library => {
            let last_index = self.ids_library.len() - 1;
            self.list_state_library.select(Some(last_index));
        }            
        AppView::SearchBook => {
            let last_index = self.ids_search_book.len() - 1;
            self.list_state_search_results.select(Some(last_index));
        }            
        AppView::PodcastEpisode => {
            if self.is_from_search_pod {
                let last_index = self.ids_pod_ep_search.len() - 1;
                self.list_state_pod_ep.select(Some(last_index));
            } else {
                let last_index = self.ids_pod_ep.len() - 1;
                self.list_state_pod_ep.select(Some(last_index));
            }}            
        AppView::Settings => {
            let last_index = self.settings.len() - 1;
            self.list_state_settings.select(Some(last_index));
        }            
        AppView::SettingsAccount => self.list_state_settings_account.select_last(),
        AppView::SettingsLibrary => {
            let last_index = self.media_types.len() - 1;
            self.list_state_settings_library.select(Some(last_index));
        }            
        AppView::SettingsAbout => self.list_state_settings_about.select_last(),
        AppView::SettingsUpdateUninstall => self.list_state_settings_update_uninstall.select_last(),
    }
}

}
