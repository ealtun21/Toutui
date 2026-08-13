use crate::api::client::ApiClient;
use crate::api::me::update_media_progress::*;
use crate::api::sessions::close_open_session::*;
use crate::db::crud::*;
use crate::logic::offline::remember_progress;
use crate::utils::exit_app::*;
use log::{info, warn};

/// Closes the listening session that the database holds, and sends the last
/// position to the server.
///
/// The function runs before the application starts a new session, and before
/// the application stops.
///
/// The application decodes the audio itself. Therefore this function does not
/// stop a separate program. The caller stops the engine.
///
/// A server that does not answer must not lose the position. The function
/// keeps the position in the table `pending_progress`, and the application
/// sends it when the server answers again. See T-25.
pub async fn sync_session_from_database(
    api: &ApiClient,
    username: String,
    server: String,
    app_quit: bool,
    handle_key: &str,
) {
    match get_listening_session(username.as_str(), server.as_str()) {
        Ok(Some(session)) => {
            if let Err(error) =
                close_session_without_send_prg_data(api, session.id_session.as_str()).await
            {
                warn!(
                    "[sync_session_from_database] the server did not close the session: {}",
                    error
                );
            }

            match handle_key {
                "Q" => info!("[handle_key (Q)][Quit] Session successfully closed"),
                "l" => info!("[handle_key (l)] Session successfully closed"),
                // A key of the view of the accounts. The program starts again
                // after this work. See T-139.
                "the accounts" => {
                    info!("[the accounts] the session closes before the program starts again")
                }
                _ => {}
            }

            let episode = if session.id_pod.is_empty() {
                None
            } else {
                Some(session.id_pod.as_str())
            };

            let result = match (episode, session.is_finished) {
                (Some(episode), true) => {
                    update_media_progress2_pod(
                        api,
                        session.id_item.as_str(),
                        Some(session.current_time),
                        &session.duration,
                        true,
                        episode,
                    )
                    .await
                }
                (Some(episode), false) => {
                    update_media_progress_pod(
                        api,
                        session.id_item.as_str(),
                        Some(session.current_time),
                        &session.duration,
                        episode,
                    )
                    .await
                }
                (None, true) => {
                    update_media_progress2_book(
                        api,
                        session.id_item.as_str(),
                        Some(session.current_time),
                        &session.duration,
                        true,
                    )
                    .await
                }
                (None, false) => {
                    update_media_progress_book(
                        api,
                        session.id_item.as_str(),
                        Some(session.current_time),
                        &session.duration,
                    )
                    .await
                }
            };

            if let Err(error) = result {
                warn!(
                    "[sync_session_from_database] the server did not accept the position: {}",
                    error
                );

                // The server does not answer. The position waits in the
                // database, and the application sends it later.
                if error.is_offline() {
                    remember_progress(
                        &username,
                        &server,
                        session.id_item.as_str(),
                        episode,
                        session.current_time as f64,
                        session.duration.parse::<f64>().unwrap_or(0.0),
                        session.is_finished,
                    );
                }
            }

            let kind = if episode.is_some() { "podcast" } else { "book" };
            let state = if session.is_finished {
                "finished"
            } else {
                "not finished"
            };

            match handle_key {
                "Q" => info!(
                    "[handle_key (Q)][{}][Quit] Item {} closed at {:?}s ({})",
                    kind, session.id_item, session.current_time, state
                ),
                "l" => info!(
                    "[handle_key (l)][{}] Item {} closed at {:?}s ({})",
                    kind, session.id_item, session.current_time, state
                ),
                "the accounts" => info!(
                    "[the accounts][{}] Item {} closed at {:?}s ({})",
                    kind, session.id_item, session.current_time, state
                ),
                _ => {}
            }

            // The session is closed and the position is safe: the server has
            // it, or the table `pending_progress` holds it. Remove the row, so
            // that the application does not send this position again at the
            // next start. A different client can write a newer position, and
            // that position must stay. See T-4.
            let _ = delete_listening_session(username.as_str(), server.as_str());
        }

        Ok(None) => {
            info!("[handle_key] The database holds no session to close");
        }
        Err(e) => {
            info!("[handle_key] Error during fetching session: {:?}", e);
        }
    }

    // The key `Q` must always stop the application.
    //
    // The old code stopped it in one branch only. The branch of `Ok(None)`
    // asked `has_played_before`, and no line of the program gave that value
    // `1` again after a playback began. The branch of `Err` stopped nothing at
    // all. Therefore the key `Q` did nothing in two conditions: after a
    // playback whose row was already gone, and when the database gave an
    // error. The user then had to stop the program by force, and a program
    // that stops by force closes no session. See `6ac5d8` and `fc695f` in
    // `known_bugs.md`.
    //
    // The sync above is the best that the program can do. It must not decide
    // whether the program stops.
    if app_quit {
        let _ = update_has_played_before("1", username.as_str());
        info!("App successfully quit");
        clean_exit();
    }
}
