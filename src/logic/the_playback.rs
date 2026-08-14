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

        // Every sentence of a view ends with a stop, and no sentence of them is
        // empty. A message with no text takes the message away (`say` calls
        // `forget` for an empty text), therefore an empty sentence here is the
        // fault of T-167 again.
        for why in [
            WhyNot::TheSessionDidNotOpen("a fault"),
            WhyNot::TheMediaDidNotCome("a fault"),
            WhyNot::NoAudioFile,
        ] {
            let text = the_words_of_a_playback_that_did_not_start(why);

            assert!(!text.trim().is_empty(), "{:?}", text);
            assert!(text.len() > 10, "{:?}", text);
        }
    }
}
