use crate::db::crud::*;
use log::{error, info, warn};

/// The longest time that a playback waits for the playback before it.
///
/// **A wait with no end is the fault of T-35, and it came back with T-158.**
/// The loop of a playback writes `is_loop_break` at its end, therefore a
/// program that dies inside that loop writes it never. 30 seconds is the time
/// of this fork for a program that stood still: the row of a session (T-140),
/// the lock of a download (T-148), and the book of a reader (T-153) all hold
/// it.
const THE_LONGEST_WAIT: std::time::Duration = std::time::Duration::from_secs(30);

/// Waits until the playback before this playback is complete.
///
/// The loop that follows a playback sets `is_loop_break` to 1 when it stops.
/// A new playback must not start before that moment, because two loops then
/// write the progress at the same time.
///
/// The first playback of a user has no loop before it. Therefore the function
/// does not wait in that condition.
///
/// **The account of this program can stand in no row of the disk.** A second
/// program of one account logs out with the key `l` of the view of the
/// accounts, and the row of `users` then goes away while this program runs
/// (T-155). The two reads gave a text of a fault for such an account, and that
/// text is not `1`: the wait therefore had no end, because no loop of a
/// playback of a row that does not exist can ever write `is_loop_break`. **A
/// row that no account holds means that no loop stands before this playback.**
/// See T-158.
pub fn wait_prev_session_finished(username: String) {
    the_wait_of_a_playback(username, THE_LONGEST_WAIT)
}

/// The work of `wait_prev_session_finished`, with the longest wait of its
/// caller. A test gives a shorter time here: a test of 30 seconds holds a
/// session of continuous integration for nothing.
pub fn the_wait_of_a_playback(username: String, longest_wait: std::time::Duration) {
    let message = "Syncing your last listening session. Please wait...";

    // **A read that failed is not an account that stands in no row** (T-202).
    // The old line took `None` of a fault of the database for that condition:
    // the program said to the user that their account is gone — a reason that the
    // program does not have (T-91) — and it did not wait at all, therefore two
    // loops of a playback of one account could run at one time (T-158 and T-140).
    //
    // **A fault of this read takes the road of a playback that waits for
    // nothing**, and it says nothing to the user. The value of the wait stands in
    // the database, therefore a database that says nothing can never end that
    // wait: the program would hold the user for the whole limit of time and say
    // "Syncing your last listening session. Please wait..." for it, and the
    // playback after it does not start at all — **a playback whose row of the
    // session reaches no disk stops with a word of its own** (T-201). A line of
    // the log names the fault, and this read holds no key of the user (T-177).
    let has_played_before = match get_has_played_before(&username) {
        Ok(value) => value,
        Err(error) => {
            error!(
                "[wait_prev_session_finished] the program did not read the account {}: {}. This \
                 playback waits for no loop before it.",
                username, error
            );

            Some("1".to_string())
        }
    };

    info!(
        "[wait_prev_session_finished][has_played_before] {:?}",
        has_played_before
    );

    let Some(has_played_before) = has_played_before else {
        warn!(
            "[wait_prev_session_finished] the account {} stands in no row of the disk. \
             No loop of a playback holds this one, therefore it does not wait.",
            username
        );
        crate::logic::message::say(
            crate::logic::the_accounts::the_text_of_an_account_that_is_gone(&username).as_str(),
        );

        return;
    };

    if has_played_before != "1" {
        // **A fault of this read is not the value of the row** (T-202). The wait
        // below goes on while the value is not `1`, therefore a fault takes the
        // road of a wait that goes on, and the limit of time of that wait holds
        // it.
        let mut is_loop_break = the_value_of_the_loop(&username);
        info!(
            "[wait_prev_session_finished][is_loop_break] {:?}",
            is_loop_break
        );

        let start = std::time::Instant::now();

        while is_loop_break.as_deref() != Some("1") {
            if start.elapsed() >= longest_wait {
                warn!(
                    "[wait_prev_session_finished] the playback before this one wrote no end in \
                     {} s. This playback starts.",
                    longest_wait.as_secs()
                );
                break;
            }

            std::thread::sleep(std::time::Duration::from_secs(1));
            is_loop_break = the_value_of_the_loop(&username);
            crate::logic::message::say(message);
        }
    }

    let _ = update_is_loop_break("0", &username);
    let _ = update_has_played_before("0", &username);

    crate::logic::message::forget();
}

/// Gives the value of `is_loop_break` of an account, and a fault of the database
/// keeps the wait. See T-202.
///
/// **A read that failed is not the value `1`.** The value `1` says that the loop
/// of the playback before this one wrote its end, and a fault of the database says
/// nothing at all: the wait therefore goes on, and the limit of time of that wait
/// holds it.
fn the_value_of_the_loop(username: &str) -> Option<String> {
    match get_is_loop_break(username) {
        Ok(value) => value,
        Err(error) => {
            error!(
                "[wait_prev_session_finished] the program did not read the loop of the account \
                 {}: {}. The wait goes on.",
                username, error
            );

            Some("0".to_string())
        }
    }
}
