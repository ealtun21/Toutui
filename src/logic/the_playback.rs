//! The words of a playback that did not start. See T-167.
//!
//! **A playback that did not start said nothing to the user.** The key `l` wrote
//! "Loading the media..." and the three faults of `logic::playback::play_media`
//! wrote to the log alone: the six seconds of that message went by, the row of
//! the message became empty, no media played, and the program never said why.
//!
//! A measurement of 2026-08-14 with the sandbox and tmux: the user stood in the
//! view of the episodes of "Letters of Two Brides" with the cursor on
//! "Letter 5", a second program took that episode out of the podcast
//! (`DELETE /api/podcasts/:id/episode/:episode?hard=1`), and the key `l` gave
//! "Loading the media..." and then **nothing at all**. The log held
//! `[play] the server did not start the session: The server does not have this
//! item.` The Home view gave the same answer for the same episode, therefore
//! this is a fault of the playback and not a fault of one view.
//!
//! **The program does not hold the title of the media at that moment.** The
//! title comes from the answer of the session, and that answer is the thing
//! that did not come. Therefore the text names what the program knows, and it
//! names no media (T-91).

/// Why a playback did not start.
///
/// Each of these is one fault of `logic::playback::play_media` that gives
/// `Outcome::Fault` before the engine holds the media.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhyNot<'a> {
    /// The server did not open the session of the playback. A media that a
    /// second program took away gives this one.
    TheSessionDidNotOpen(&'a str),
    /// The server opened the session, and it did not give the media after it.
    TheMediaDidNotCome(&'a str),
    /// The media of the server holds no audio file.
    NoAudioFile,
    /// The answer of the session names no session. See T-182.
    TheSessionHasNoIdentity,
    /// The answer of the session gave no place, and the read of the place of
    /// that media came back with a fault. See T-182.
    ThePlaceDidNotCome(&'a str),
    /// The disk did not take the row of the session of this playback. See T-201.
    ///
    /// **That row is the one copy of the place of the user for a program that
    /// dies** (T-140, T-145, and T-152), and the row of the player of the screen
    /// reads it too: a playback with no row plays the audio, and the screen says
    /// nothing at all of the media.
    TheDiskDidNotTakeTheSession(&'a str),
}

/// What the program does with the answer of a session of a playback. See
/// T-182.
#[derive(Debug, Clone, PartialEq)]
pub enum TheStartOfAPlayback {
    /// The answer holds the place of the user, and the playback starts at it.
    ItStartsAt(f64),
    /// The answer holds a session, and it gave no place. The program asks the
    /// server for the place of that media.
    TheProgramAsksForThePlace,
    /// The answer names no session. The playback does not start.
    TheSessionHasNoIdentity,
}

/// Reads the answer of the session of a playback.
///
/// `of_the_session` is the list of `collect_info_item` of
/// `src/api/library_items/play_lib_item_or_pod.rs`, and **a text of no
/// character is a value that the server did not give**.
///
/// **A session that the server did not name is no session** (T-182). Every sync
/// of the playback and the close at the end name the session in the path of the
/// request, therefore a session of no name gives `/api/session//sync`: a
/// measurement of 2026-08-14 read
/// `the server did not accept the sync: The server does not have this item.` at
/// each sync of a book of eight hours, and the same sentence at the close. The
/// row of the database took that name too, and `id_session` is the key of the
/// table `listening_session`: a second program of the same account wrote no row
/// of its own, and its position stood in the row of the first program.
///
/// The function is pure, therefore a test needs no server and no engine.
pub fn the_start_of_a_playback(of_the_session: &[String]) -> TheStartOfAPlayback {
    let names_a_session = of_the_session
        .get(3)
        .is_some_and(|name| !name.trim().is_empty());

    if !names_a_session {
        return TheStartOfAPlayback::TheSessionHasNoIdentity;
    }

    match of_the_session
        .first()
        .and_then(|text| the_place_of_the_session(text))
    {
        Some(place) => TheStartOfAPlayback::ItStartsAt(place),
        None => TheStartOfAPlayback::TheProgramAsksForThePlace,
    }
}

/// The place of the start of a playback, of the answer of the session.
///
/// **A place that the answer of the server does not hold is not the place 0**
/// (T-182). `currentTime` of `POST /api/items/:id/play` took the default 0.0,
/// therefore a server that does not hold that field started the book of the
/// user at its first second, the loop of the playback wrote that first second
/// to the server, and the place of the user went away with no word: a
/// measurement of 2026-08-14 took the book of eight hours of the sandbox from
/// 12000 seconds to 1096 seconds.
///
/// A text of no number gives the same answer as a text of no character: the
/// program does not have the place, and it must ask the server for it.
///
/// The function is pure, therefore a test needs no server and no engine.
pub fn the_place_of_the_session(of_the_answer: &str) -> Option<f64> {
    let text = of_the_answer.trim();

    if text.is_empty() {
        return None;
    }

    text.parse::<f64>().ok().filter(|place| place.is_finite())
}

/// The place of a media whose read came back with a fault. See T-182.
///
/// **The status 404 is the answer of a media that never played**, and the place
/// of such a media is 0. That 0 is a measurement, and it is not a default.
///
/// Every other fault gives nothing: the program does not have the place of the
/// user, therefore the playback does not start. A playback that starts at 0
/// gives that 0 to the server at the next sync, and the place of the user goes
/// away. This is the rule of T-175 and of T-178 for the playback.
///
/// The function is pure, therefore a test needs no server.
pub fn the_place_of_a_media_that_never_played(
    error: &crate::api::client::error::ApiError,
) -> Option<f64> {
    match error {
        crate::api::client::error::ApiError::NotFound => Some(0.0),
        _ => None,
    }
}

/// Gives the sentence that the user reads.
///
/// The function is pure, therefore a test needs no server and no engine.
///
/// **The answer of a key stands above every view** (T-164): the user pressed
/// `l`, and the media of the queue that comes after it writes `The queue starts
/// "…"` to the same slot. Therefore the caller writes this text with
/// `message::say`, and not with `message::say_in`.
pub fn the_words_of_a_playback_that_did_not_start(why: WhyNot<'_>) -> String {
    match why {
        WhyNot::TheSessionDidNotOpen(error) => {
            format!("The server did not start the playback: {}", error)
        }
        WhyNot::TheMediaDidNotCome(error) => {
            format!("The server did not give the media: {}", error)
        }
        WhyNot::NoAudioFile => "This media has no audio file.".to_string(),
        // The program cannot send the position of a session that has no name,
        // and it cannot close that session. See T-182.
        WhyNot::TheSessionHasNoIdentity => "The session of the server has no identity.".to_string(),
        WhyNot::ThePlaceDidNotCome(error) => {
            format!("The server did not give the place of this media: {}", error)
        }
        // The row of the disk holds the place of the user for a program that
        // dies, and the row of the player of the screen reads it. A playback
        // with no such row keeps no place. See T-201.
        WhyNot::TheDiskDidNotTakeTheSession(error) => {
            format!(
                "The program did not keep the session on its disk: {}. Stop a second Toutui, and \
                 press the key again.",
                error.trim_end_matches('.')
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The three faults of a playback that did not start each give one
    /// sentence, and no sentence of them is empty.
    ///
    /// **The parts of this test stay in one function**: two test functions of
    /// one module fight for the box of the process (T-144 and T-157).
    #[test]
    fn every_fault_of_a_playback_says_one_sentence() {
        // The fault of the measurement of T-167: a second program took the
        // episode out of the podcast.
        let text = the_words_of_a_playback_that_did_not_start(WhyNot::TheSessionDidNotOpen(
            "The server does not have this item.",
        ));

        assert_eq!(
            text,
            "The server did not start the playback: The server does not have this item."
        );

        let text = the_words_of_a_playback_that_did_not_start(WhyNot::TheMediaDidNotCome(
            "The server does not have this item.",
        ));

        assert_eq!(
            text,
            "The server did not give the media: The server does not have this item."
        );

        assert_eq!(
            the_words_of_a_playback_that_did_not_start(WhyNot::NoAudioFile),
            "This media has no audio file."
        );

        // The fault of the measurement of T-201: a second program of one account
        // held the database, therefore the row of the session reached no disk.
        // The sentence names the key of the work of that fault (T-170).
        let text = the_words_of_a_playback_that_did_not_start(WhyNot::TheDiskDidNotTakeTheSession(
            "database is locked",
        ));

        assert_eq!(
            text,
            "The program did not keep the session on its disk: database is locked. Stop a second \
             Toutui, and press the key again."
        );

        // Every sentence of a view ends with a stop, and no sentence of them is
        // empty. A message with no text takes the message away (`say` calls
        // `forget` for an empty text), therefore an empty sentence here is the
        // fault of T-167 again.
        for why in [
            WhyNot::TheSessionDidNotOpen("a fault"),
            WhyNot::TheMediaDidNotCome("a fault"),
            WhyNot::NoAudioFile,
            // The two faults of the answer of the session of T-182.
            WhyNot::TheSessionHasNoIdentity,
            WhyNot::ThePlaceDidNotCome("a fault"),
        ] {
            let text = the_words_of_a_playback_that_did_not_start(why);

            assert!(!text.trim().is_empty(), "{:?}", text);
            assert!(text.len() > 10, "{:?}", text);
        }
    }
}
