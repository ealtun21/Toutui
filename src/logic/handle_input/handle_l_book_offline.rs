use crate::player::vlc::start_vlc::*;
use crate::player::vlc::fetch_vlc_data::*;
use crate::player::vlc::exec_nc::*;
use crate::player::vlc::quit_vlc::*;
use crate::utils::pop_up_message::*;
use std::io::stdout;
use log::{info, error};
use crate::db::crud::*;

/// Play a library item straight from a locally downloaded file, without contacting
/// the Audiobookshelf server. Progress is persisted locally (in the `downloads`
/// table) instead of being synced through a server play session.
pub async fn handle_l_book_offline(
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
) {

    // need to pkill VLC for macos users
    pkill_vlc();

    info!("[handle_l_book_offline] Playing offline item {} from {}", id_item, file_path);

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

    loop {
        match fetch_vlc_data(port.clone(), address_player.clone()).await {
            Ok(Some(data_fetched_from_vlc)) => {

                // persist progress locally so offline playback resumes where it left off
                let _ = update_download_current_time(id_item.as_str(), username.as_str(), data_fetched_from_vlc);

                // Important, sleep time to 1s minimum, otherwise connection to vlc player will not have time to connect
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                match fetch_vlc_is_playing(port.clone(), address_player.clone()).await {
                    Ok(true) => {}
                    // `Ok(false)` means the track reached the end (VLC still open)
                    Ok(false) => {
                        info!("[handle_l_book_offline][Finished] Track finished");
                        let _ = update_download_current_time(id_item.as_str(), username.as_str(), 0);
                        let _ = update_is_loop_break("1", username.as_str());
                        let _ = update_is_vlc_running("0", username.as_str());
                        break;
                    }
                    // `Err` means VLC was closed by the user
                    Err(_) => {
                        info!("[handle_l_book_offline][Quit]");
                        let _ = update_is_loop_break("1", username.as_str());
                        let _ = update_is_vlc_running("0", username.as_str());
                        break;
                    }
                }
            }
            Ok(None) => {
                info!("[handle_l_book_offline][None]");
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
