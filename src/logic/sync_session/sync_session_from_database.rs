use crate::api::client::ApiClient;
use crate::api::me::update_media_progress::*;
use crate::api::sessions::close_open_session::*;
use crate::db::crud::*;
use crate::logic::offline::{remember_progress, the_place_can_wait};
use crate::logic::sync_session::the_rows_that_the_disk_kept::{
    the_row_of_a_closed_session_goes_away, the_server_holds_this_session_already,
};
use crate::utils::exit_app::*;
use log::{info, warn};

/// Closes **every** listening session that the database holds for this program,
/// and sends the last position of each of them to the server.
///
/// A program that died leaves its row, and the rule of T-140 gives that row to
/// the next program of the account. **The function closed one row and it
/// removed every row before T-145**: a user who started the program again
/// inside `THE_LIMIT_OF_THE_HEARTBEAT` seconds lost the place of the program
/// that died, because that row was too young for the close and old enough for
/// the removal.
///
/// The row of a program that died goes first, and the row of this program goes
/// last: two rows of one media then leave the newest position on the server.
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
    match get_the_sessions_to_close(username.as_str(), server.as_str()) {
        Ok(sessions) if sessions.is_empty() => {
            info!("[handle_key] The database holds no session to close");
        }

        Ok(sessions) => {
            for session in sessions {
                // **A place that this program gave to the server goes to that
                // server no second time** (T-207). The removal of the row can
                // fail, and the row of the disk then says "this place waits for
                // the server" for a place that the server holds already: the
                // program sent 646 seconds over the 6000 seconds of a second
                // client of the account, and the book of the user lost 89
                // minutes.
                if the_server_holds_this_session_already(session.id_session.as_str()) {
                    info!(
                        "[handle_key] the server holds the place of the session {} already. The \
                         disk kept its row, and this program sends it no second time.",
                        session.id_session
                    );

                    // A disk that answers again takes that row away, and the
                    // condition of the box goes away with it.
                    the_row_of_a_closed_session_goes_away(session.id_session.as_str());

                    continue;
                }

                close_one_session(api, &username, &server, session, handle_key).await;
            }
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
        // **A caller that reads no answer of its write says nothing at all**
        // (T-207). The program stops after this line, therefore no word can
        // reach the user and the log holds the fault (T-177). The program that
        // starts after this one writes the same mark again.
        if let Err(error) = update_has_played_before("1", username.as_str()) {
            warn!(
                "[handle_key (Q)] the disk did not take the mark of the quit of {}: {}.",
                username, error
            );
        }

        info!("App successfully quit");
        clean_exit();
    }
}

/// Closes one session on the server, sends its position, and removes its row.
///
/// **The row goes away after the position is safe**: the server holds it, or
/// the table `pending_progress` holds it. The function removes the row of this
/// session alone, therefore a row that no request carried stays for the next
/// program. See T-145.
async fn close_one_session(
    api: &ApiClient,
    username: &str,
    server: &str,
    session: crate::db::database_struct::ListeningSession,
    handle_key: &str,
) {
    if let Err(error) = close_session_without_send_prg_data(api, session.id_session.as_str()).await
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

        // **The server did not take the place of the user. That place waits in
        // the database, and the application sends it later.**
        //
        // The old code asked `is_offline`, therefore a server that **answered**
        // with a fault threw the place away: the row of the session goes away
        // after this block, and no row of `pending_progress` held it. A
        // measurement of 2026-08-14 with `docs/harness/one_path_fails.py` of the
        // path `/api/me/progress` lost 1234 seconds of a book of eight hours,
        // and the log said "closed at 1234s". `the_place_can_wait` names the two
        // faults that mean "this place reaches this server never". See T-189.
        if the_place_can_wait(&error) {
            remember_progress(
                username,
                server,
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

    // The session is closed and the position is safe: the server has it, or the
    // table `pending_progress` holds it. Remove the row, so that the
    // application does not send this position again at the next start. A
    // different client can write a newer position, and that position must
    // stay. See T-4.
    //
    // **The row of this session goes away, and no other row.** A blunt removal
    // takes the row of a program that died away with no request, and the place
    // of that user is then gone for ever. See T-145.
    //
    // **A removal that the disk refused is no removal** (T-207). The caller of
    // this line read no answer, therefore a disk that takes no write left a row
    // of a place that the server holds already.
    the_row_of_a_closed_session_goes_away(session.id_session.as_str());
}
