use crate::api::client::ApiClient;
use crate::db::crud::*;
use crate::api::sessions::close_open_session::*;
use log::{info, warn};
use crate::api::me::update_media_progress::*;
use crate::utils::exit_app::*;

/// Closes the listening session that the database holds, and sends the last
/// position to the server.
///
/// The function runs before the application starts a new session, and before
/// the application stops.
///
/// The application decodes the audio itself. Therefore this function does not
/// stop a separate program. The caller stops the engine.
pub async fn sync_session_from_database(
    api: &ApiClient,
    username: String,
    app_quit: bool,
    handle_key: &str,
) {
    match get_listening_session() {
        Ok(Some(session)) => {

            if let Err(error) =
                close_session_without_send_prg_data(api, session.id_session.as_str()).await
            {
                warn!("[sync_session_from_database] the server did not close the session: {}", error);
            }

            match handle_key {
                "Q" => info!("[handle_key (Q)][Quit] Session successfully closed"),
                "l" => info!("[handle_key (l)] Session successfully closed"),
                _ => {}
            }

            if session.id_pod.is_empty() {
                if !session.is_finished {
                    if let Err(error) = update_media_progress_book(
                        api,
                        session.id_item.as_str(),
                        Some(session.current_time),
                        &session.duration).await
                    {
                        warn!("[sync_session_from_database] the server did not accept the position: {}", error);
                    }

                    match handle_key {
                        "Q" => info!("[handle_key (Q)][book][Quit] Item {} closed at {:?}s (not finished)", session.id_item, session.current_time),
                        "l" => info!("[handle_key (l)] Item {} closed at {:?}s (not finished)", session.id_item, session.current_time),
                        _ => {}
                    }
                }

                else {
                    let is_finished = true;
                    if let Err(error) = update_media_progress2_book(
                        api,
                        session.id_item.as_str(),
                        Some(session.current_time),
                        &session.duration,
                        is_finished).await
                    {
                        warn!("[sync_session_from_database] the server did not accept the position: {}", error);
                    }

                    match handle_key {
                        "Q" => info!("[handle_key (Q)][book][Quit] Item {} closed at {:?}s (finished)", session.id_item, session.current_time),
                        "l" => info!("[handle_key (l)] Item {} closed at {:?}s (finished)", session.id_item, session.current_time),
                        _ => {}
                    }
                }

            } else {
                if !session.is_finished {
                    if let Err(error) = update_media_progress_pod(
                        api,
                        session.id_item.as_str(),
                        Some(session.current_time),
                        &session.duration,
                        session.id_pod.as_str()).await
                    {
                        warn!("[sync_session_from_database] the server did not accept the position: {}", error);
                    }


                    match handle_key {
                        "Q" => info!("[handle_key (Q)][podcast][Quit] Item {} closed at {:?}s", session.id_pod, session.current_time),
                        "l" => info!("[handle_key (l)] Item {} closed at {:?}s", session.id_pod, session.current_time),
                        _ => {}
                    }
                } else {
                    let is_finished = true;
                    if let Err(error) = update_media_progress2_pod(
                        api,
                        session.id_item.as_str(),
                        Some(session.current_time),
                        &session.duration,
                        is_finished,
                        session.id_pod.as_str()).await
                    {
                        warn!("[sync_session_from_database] the server did not accept the position: {}", error);
                    }

                    match handle_key {
                        "Q" => info!("[handle_key (Q)][podcast][Quit] Item {} closed at {:?}s (finished)", session.id_pod, session.current_time),
                        "l" => info!("[handle_key (l)] Item {} closed at {:?}s (finished)", session.id_pod, session.current_time),
                        _ => {}
                    }
                }
            }

            if app_quit {
                let _ = update_has_played_before("1", username.as_str());
                info!("App successfully quit");
                clean_exit();
            }
        }

        Ok(None) => {
            // The database holds no session. If the user played a media
            // before, the application can stop now.
            if get_has_played_before(username.as_str()) == "1" {
                info!("[handle_key] Quit with no listening session");
                clean_exit();
            } else {
                info!("[handle_key] The first session starts");
            }
        }
        Err(e) => {
            info!("[handle_key] Error during fetching session: {:?}", e);
        }
    }
}

