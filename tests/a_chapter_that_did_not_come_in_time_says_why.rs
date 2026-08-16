//! T-280: a render of a chapter that did not come back says what the program
//! measured.
//!
//! The measurement, of the real program v0.8.108 inside tmux against the
//! sandbox, of a book of three chapters whose second chapter holds 40000
//! nested `<div>`. That book stood in the cache of the ebooks of the account
//! `toutuitest`, under the name of the item of `Alice in Wonderland`, because
//! a book of the cache costs no request of the server.
//!
//! The key `e` gave `Reading…` for five seconds, and the screen then said
//!
//! ```text
//! This chapter is too complex.
//! ```
//!
//! and the file of the log held no line of the reader at all.
//!
//! The three faults of that one sentence:
//!
//! 1. **The program measured a time, and it did not measure the chapter.** A
//!    machine that is busy and a disk that is slow give that same limit of
//!    five seconds. "Too complex" is therefore a reason that the program does
//!    not have (T-91).
//! 2. **The sentence names no key.** The view of the reader holds `n` for the
//!    chapter after this one, `p` for the chapter before it, and `h` to leave
//!    the book, and each of the three does the work of this fault (T-170).
//! 3. **The arm takes no line of the log** (T-177): the maintainer of a
//!    machine that is slow reads nothing of it.
//!
//! The arm beside it, for a render whose thread died, said `This chapter did
//! not open.` and it dropped the `JoinError` that holds the reason.
//!
//! These tests need no book, no terminal, and no network: the two functions
//! make the text of the message.

use toutui::logic::reader::session::{
    the_message_of_the_render_that_died, the_message_of_the_render_that_took_too_long,
};

/// The keys that the view of the reader holds for this fault. The footer of
/// that view says `n/p: chapter` and `h: leave the book`.
const THE_KEYS: [&str; 3] = ["n", "p", "h"];

#[test]
fn a_render_that_went_past_its_limit_says_what_the_program_measured() {
    let text = the_message_of_the_render_that_took_too_long(1);

    // The fault: the sentence before this correction. The program measured a
    // time of five seconds, and it did not measure the chapter.
    assert!(
        !text.contains("too complex"),
        "the sentence says a reason that the program does not have: {text}"
    );

    // What the program has: the limit of time that went by.
    assert!(
        text.contains("5 seconds"),
        "the sentence does not say the limit of time: {text}"
    );

    // The two conditions that give that limit, and neither of them as a fact.
    assert!(
        text.contains("can have very many tags") && text.contains("machine can be busy"),
        "the sentence does not name the two conditions: {text}"
    );

    // The keys of the view that the user reads this text in (T-170).
    for key in THE_KEYS {
        assert!(
            text.contains(&format!("Press {key} ")) || text.contains(&format!("or {key} ")),
            "the sentence does not name the key {key}: {text}"
        );
    }

    assert!(
        text.contains("log"),
        "the sentence does not name the file of the log: {text}"
    );
}

#[test]
fn a_render_whose_thread_died_holds_the_reason_of_the_machine() {
    // A `JoinError` comes of the runtime alone, therefore the test makes one
    // with a task that panics.
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("the test needs a runtime");

    let fault = runtime.block_on(async {
        let guard = toutui::utils::exit_app::ExpectedPanic::new();
        let answer = tokio::spawn(async { panic!("the render of the test") }).await;
        drop(guard);
        answer.expect_err("a task that panics gives a fault of the join")
    });

    let text = the_message_of_the_render_that_died(1, &fault);

    // The fault: the sentence before this correction dropped the reason.
    assert!(
        !text.contains("This chapter did not open."),
        "the sentence is the one of before the correction: {text}"
    );
    assert!(
        text.contains(
            &fault
                .to_string()
                .trim_end()
                .trim_end_matches('.')
                .to_string()
        ),
        "the sentence does not hold the reason of the machine: {text}"
    );

    for key in THE_KEYS {
        assert!(
            text.contains(&format!("Press {key} ")) || text.contains(&format!("or {key} ")),
            "the sentence does not name the key {key}: {text}"
        );
    }
}

/// The two sentences say no reason as a fact, and each of them names a road.
#[test]
fn the_two_sentences_of_a_render_that_did_not_come_back_name_a_road() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("the test needs a runtime");
    let fault = runtime.block_on(async {
        let guard = toutui::utils::exit_app::ExpectedPanic::new();
        let answer = tokio::spawn(async { panic!("the render of the test") }).await;
        drop(guard);
        answer.expect_err("a task that panics gives a fault of the join")
    });

    for text in [
        the_message_of_the_render_that_took_too_long(0),
        the_message_of_the_render_that_died(0, &fault),
    ] {
        assert!(
            text.contains("Press h to leave the book."),
            "the sentence does not give the road back: {text}"
        );
        // A sentence of one word says nothing. Each of these holds the fault,
        // the road, and the log.
        assert!(text.len() > 120, "the sentence is too short: {text}");
    }
}
