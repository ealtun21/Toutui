use crate::api::client::ApiClient;
use crate::player::vlc::start_vlc::*;
use crate::player::vlc::fetch_vlc_data::*;
use crate::player::vlc::exec_nc::*;
use crate::player::vlc::quit_vlc::*;
use crate::utils::pop_up_message::*;
use crate::api::me::update_media_progress::*;
use std::io::stdout;
use log::{info, error, warn};
use crate::db::crud::*;

/// Play a library item straight from a locally downloaded file, without requiring
/// the Audiobookshelf server to start the session. Progress is always persisted
/// locally (in the `downloads` table) so offline listening resumes correctly.
/// If the server is reachable, progress is also best-effort pushed to it (same
/// endpoint the streaming path uses) so other Audiobookshelf clients stay in
/// sync; if it isn't reachable the push just fails silently.
#[allow(clippy::too_many_arguments)]
pub async fn handle_l_book_offline(
    api: &ApiClient,
    port: String,
    address_player: String,
    program: String,
    is_cvlc_term: String,
    username: String,
    id_item: String,
    file_path: String,
    current_time_start: u32,
    title: String,
    author: String,
    duration: f64,
) {

    // need to pkill VLC for macos users
    pkill_vlc();

    info!("[handle_l_book_offline] Playing offline item {} from {}", id_item, file_path);

    let duration_str = duration.to_string();
    let current_time_str = current_time_start.to_string();

    let port_clone = port.clone();
    let address_player_clone = address_player.clone();
    let file_path_clone = file_path.clone();
    let title_clone = title.clone();
    let author_clone = author.clone();
    let program_clone = program.clone();
    let username_clone = username.clone();

    // start_vlc_offline is launched in a spawn to allow fetch_vlc_data to start at the same time
    tokio::spawn(async move {
        info!("[handle_l_book_offline][start_vlc_offline] VLC successfully launched");
        start_vlc_offline(
            &current_time_str,
            &port_clone,
            address_player_clone,
            &file_path_clone,
            title_clone.clone(), // artist
            title_clone, // subtitle
            author_clone,
            program_clone,
            username_clone,
        ).await;
    });

    if is_cvlc_term == "1" {
        let port_clone = port.clone();
        let address_player_clone = address_player.clone();
        tokio::spawn(async move {
            exec_nc(&port_clone, address_player_clone).await;
        });
    }

    // clear loading message (from app.rs) when vlc is launched
    let mut stdout = stdout();
    let _ = clear_message(&mut stdout, 3);

    // Important, sleep time to 1s minimum otherwise connection to vlc player will not have time to connect
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let _ = update_is_vlc_running("1", username.as_str());

    // sync progress to the server (if reachable) every ~10s, like the streaming path
    let mut trigger = 1;

    loop {
        match fetch_vlc_data(port.clone(), address_player.clone()).await {
            Ok(Some(data_fetched_from_vlc)) => {

                // persist progress locally so offline playback resumes where it left off
                let _ = update_download_current_time(id_item.as_str(), username.as_str(), data_fetched_from_vlc);

                // Important, sleep time to 1s minimum, otherwise connection to vlc player will not have time to connect
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                match fetch_vlc_is_playing(port.clone(), address_player.clone()).await {
                    Ok(true) => {
                        // This function plays a local file. There is no server session, thus there is
                        // no session id and no /sync request. Only /progress can write the progress
                        // here. This is different from the streaming path of upstream issue #35.
                        // best-effort push to the server every ~10 seconds
                        if trigger >= 10 {
                            if let Err(error) = update_media_progress_book(api, id_item.as_str(), Some(data_fetched_from_vlc), &duration_str).await {
                                warn!("[handle_l_book_offline] the server did not accept the position: {}", error);
                            }
                            trigger = 0;
                        }
                        trigger += 1;
                    }
                    // `Ok(false)` means the track reached the end (VLC still open)
                    Ok(false) => {
                        info!("[handle_l_book_offline][Finished] Track finished");
                        let _ = update_download_current_time(id_item.as_str(), username.as_str(), 0);
                        if let Err(error) = update_media_progress2_book(api, id_item.as_str(), Some(data_fetched_from_vlc), &duration_str, true).await {
                            warn!("[handle_l_book_offline] the server did not accept the position: {}", error);
                        }
                        let _ = update_is_loop_break("1", username.as_str());
                        let _ = update_is_vlc_running("0", username.as_str());
                        break;
                    }
                    // `Err` means VLC was closed by the user
                    Err(_) => {
                        info!("[handle_l_book_offline][Quit]");
                        if let Err(error) = update_media_progress_book(api, id_item.as_str(), Some(data_fetched_from_vlc), &duration_str).await {
                            warn!("[handle_l_book_offline] the server did not accept the position: {}", error);
                        }
                        let _ = update_is_loop_break("1", username.as_str());
                        let _ = update_is_vlc_running("0", username.as_str());
                        break;
                    }
                }
            }
            Ok(None) => {
                info!("[handle_l_book_offline][None]");
                if let Err(error) = update_media_progress_book(api, id_item.as_str(), Some(current_time_start), &duration_str).await {
                    warn!("[handle_l_book_offline] the server did not accept the position: {}", error);
                }
                let _ = update_is_loop_break("1", username.as_str());
                let _ = update_is_vlc_running("0", username.as_str());
                break;
            }
            Err(e) => {
                error!("[handle_l_book_offline][Err(e)] {}", e);
                let _ = update_is_loop_break("1", username.as_str());
                let _ = update_is_vlc_running("0", username.as_str());
                break;
            }
        }
    }
}
