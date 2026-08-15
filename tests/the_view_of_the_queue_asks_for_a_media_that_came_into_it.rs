//! The view of the queue asks for the place of a media that came into it. See
//! T-237.
//!
//! **The request of the places ran at the key `q`, and it named the media of
//! that moment alone.** The queue changes with no key of this user: the key `X`
//! and the media that comes to its end each read the queue of the disk again
//! (T-147), therefore a second program of the account puts a media in this
//! queue and the line of it comes into the open view. That line held no mark of
//! a place and it said the length of the whole media, while every other line of
//! that same view said the time that is left (T-234).
//!
//! The measurement of the real program v0.8.65 inside tmux, against the sandbox
//! (podman on :13399). The user pressed the key `n` on `A Long Test Book` and on
//! `A Second Book Of Many Hours` of the library `Books`, and then the key `q`. A
//! second program of the account then put `A Book Of Many Hours` in the queue,
//! and the user pressed the key `X` on the line 1:
//!
//! ```text
//! ➤     1. 📕 A Second Book Of Many Hours — Many Hours Author  (6h left)
//!       2. 📕 A Book Of Many Hours — Many Hours Author  (8h)
//! ```
//!
//! The server holds `A Book Of Many Hours` at 7200 seconds of 28800, with the
//! percent 90. **The control of the same run** (the trap 206): the line 1 of
//! that same view said `(6h left)`, therefore the row of the time works; and the
//! Home view of that same program said `90% A Book Of Many Hours`.
//!
//! The same measurement of the corrected program:
//!
//! ```text
//! ➤     1. 📕 A Second Book Of Many Hours — Many Hours Author  (6h left)
//!   90% 2. 📕 A Book Of Many Hours — Many Hours Author  (6h left)
//! ```
//!
//! The log of a proxy of that same run counted the requests (the trap 144): two
//! requests of `/api/me` at the key `q`, **one** after the key `X`, and no
//! request more in the six seconds after it.
//!
//! **The parts of this test stay in one function**: two test functions of one
//! module fight for the slot of that module, and `cargo test` then finds a fault
//! that nextest hides (T-144 and T-157).
//!
//! The function of the rule is pure, therefore this test needs no server and no
//! screen. **Five builds of the fault each fail it**: a rule that says nothing
//! for a media that stands outside, a key of the item alone for an episode, a
//! loop of the view that does not call the rule, a request that keeps no name,
//! and a rule that reads no offline mode.

use std::collections::BTreeSet;
use toutui::logic::playback::PlaybackTarget;
use toutui::logic::queue::{
    a_media_of_the_queue_stands_outside, keep_the_keys_that_the_program_asked,
    the_keys_that_the_program_asked, Entry,
};

/// A book of the queue.
fn book(id: &str) -> Entry {
    Entry {
        target: PlaybackTarget::Book {
            item_id: id.to_string(),
            whole_book_duration: Some(28800.0),
        },
        title: id.to_string(),
        author: "Many Hours Author".to_string(),
        duration: Some(28800.0),
    }
}

/// An episode of the podcast `a-podcast`.
fn episode(id: &str) -> Entry {
    Entry {
        target: PlaybackTarget::Episode {
            item_id: "a-podcast".to_string(),
            episode_id: id.to_string(),
        },
        title: id.to_string(),
        author: "Arthur Gordon Pym".to_string(),
        duration: Some(2336.7),
    }
}

/// Gives the block of a function of a file of the source. See the trap 209.
///
/// A window of a number of characters is a window of the comments of the
/// function after it: the words of a correction take a line out of that window,
/// and the gate then says that the program lost a rule that it holds. The block
/// ends at the comment or at the head of the function that comes after this
/// one.
fn the_block_of(source: &str, head: &str) -> String {
    let start = source
        .find(head)
        .unwrap_or_else(|| panic!("the source holds no function `{}`", head));
    let body = &source[start + head.len()..];

    let end = body
        .find("\n    /// ")
        .into_iter()
        .chain(body.find("\n    pub fn "))
        .chain(body.find("\n    fn "))
        .min()
        .unwrap_or(body.len());

    body[..end].to_string()
}

#[test]
fn the_view_of_the_queue_asks_for_a_media_that_came_into_it() {
    // ---------------------------------------------------------------------
    // The rule: a media of the queue that the request did not name.
    // ---------------------------------------------------------------------

    let asked: BTreeSet<String> = ["a-book".to_string(), "a-second-book".to_string()]
        .into_iter()
        .collect();

    // Every media of the queue stands in the names of the request, therefore
    // the program asks the server nothing. The view of the queue calls this
    // rule at every frame, and a rule that says `true` here gives one request
    // for each frame.
    assert!(
        !a_media_of_the_queue_stands_outside(&[book("a-book"), book("a-second-book")], &asked),
        "the request named every media of this queue"
    );

    // **A media came into the queue after the request** (T-237). The line of it
    // said the length of the whole media and it held no mark of a place.
    assert!(
        a_media_of_the_queue_stands_outside(&[book("a-second-book"), book("a-third-book")], &asked),
        "the request of the key q named the media of that moment alone, \
         therefore the program must ask the server for the media that came \
         into the queue after it"
    );

    // An empty queue holds no media that stands outside: the key `X` of the
    // last line of the view must give no request.
    assert!(
        !a_media_of_the_queue_stands_outside(&[], &asked),
        "an empty queue holds no media at all"
    );

    // **The name of an episode holds the episode after the item** (T-223 and
    // T-229): two episodes of one podcast hold the identity of that podcast,
    // therefore a name of the item alone would say that the request named the
    // second episode already.
    let of_one_episode: BTreeSet<String> = ["a-podcast/the-first-episode".to_string()]
        .into_iter()
        .collect();

    assert!(
        !a_media_of_the_queue_stands_outside(&[episode("the-first-episode")], &of_one_episode),
        "the request named this episode"
    );
    assert!(
        a_media_of_the_queue_stands_outside(&[episode("the-second-episode")], &of_one_episode),
        "the request named the first episode of this podcast, and not the \
         second one"
    );

    // ---------------------------------------------------------------------
    // The box of the names of the request.
    // ---------------------------------------------------------------------

    keep_the_keys_that_the_program_asked(asked.clone());
    assert_eq!(the_keys_that_the_program_asked(), asked);

    // **The list takes the place of the list that came before it**: a media
    // that left the queue and that came back into it stands in no row of the
    // box of the places, therefore the program must ask the server for it
    // again.
    let of_the_second_request: BTreeSet<String> =
        ["a-third-book".to_string()].into_iter().collect();

    keep_the_keys_that_the_program_asked(of_the_second_request.clone());
    assert_eq!(the_keys_that_the_program_asked(), of_the_second_request);
    assert!(
        a_media_of_the_queue_stands_outside(&[book("a-book")], &the_keys_that_the_program_asked()),
        "the names of the request before this one must reach no later frame"
    );

    // ---------------------------------------------------------------------
    // The two places of `src/app.rs` that hold the rule.
    // ---------------------------------------------------------------------

    let source = include_str!("../src/app.rs");

    // **The names go to the box before the request** (T-237): the answer comes
    // in a task of its own, and the loop of the view would else ask the server
    // again at each frame until that answer came.
    let of_the_request = the_block_of(
        source,
        "fn ask_the_server_for_the_places_of_the_queue(&mut self) {",
    );

    assert!(
        of_the_request.contains("keep_the_keys_that_the_program_asked"),
        "the request must name the media that it asked for, and it must do it \
         before the task of that request"
    );

    // **The loop of the view asks for a media that came into the queue**: the
    // key `X` and the media that comes to its end each read the queue of the
    // disk again, and no key of this user names the media that came in.
    let of_the_loop = the_block_of(
        source,
        "pub fn the_line_of_the_queue_holds_its_media(&mut self) {",
    );

    assert!(
        of_the_loop.contains("a_media_of_the_queue_stands_outside"),
        "the loop of the view of the queue must ask the server for the place \
         of a media that came into the queue after the request of the key q"
    );
    assert!(
        of_the_loop.contains("self.is_offline"),
        "the offline mode asks the server nothing at all, therefore the names \
         of the box stay empty there and this rule must not read them"
    );
}
