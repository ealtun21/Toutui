use crate::db::crud::*;
use log::info;

/// Waits until the playback before this playback is complete.
///
/// The loop that follows a playback sets `is_loop_break` to 1 when it stops.
/// A new playback must not start before that moment, because two loops then
/// write the progress at the same time.
///
/// The first playback of a user has no loop before it. Therefore the function
/// does not wait in that condition.
pub fn wait_prev_session_finished(username: String) {
    let message = "Syncing your last listening session. Please wait...";

    let has_played_before = get_has_played_before(&username);
    info!(
        "[wait_prev_session_finished][has_played_before] {}",
        has_played_before
    );

    if has_played_before != "1" {
        let mut is_loop_break = get_is_loop_break(&username);
        info!(
            "[wait_prev_session_finished][is_loop_break] {}",
            is_loop_break
        );

        while is_loop_break != "1" {
            std::thread::sleep(std::time::Duration::from_secs(1));
            is_loop_break = get_is_loop_break(&username);
            crate::logic::message::say(message);
        }
    }

    let _ = update_is_loop_break("0", &username);
    let _ = update_has_played_before("0", &username);

    crate::logic::message::forget();
}
