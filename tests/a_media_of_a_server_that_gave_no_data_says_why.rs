//! T-279: a media whose data the server did not give says why.
//!
//! The endpoint of the ebook answers 404 for a media with no ebook and for an
//! item that does not exist, therefore the reader asks the server for the item
//! and it names what that item holds. **A fault of that second request is not a
//! media with no ebook**: the program then says nothing of the book, and the
//! sentence "The server has no ebook for this media." is a reason that the
//! program does not have (T-91).
//!
//! The measurement of the real program v0.8.107 inside tmux against the
//! sandbox: the harness `docs/harness/a_status_of_some_paths.py` gave the
//! status 404 to `/api/items/<the id>/ebook` and the status 500 to
//! `/api/items/<the id>` of Alice in Wonderland, and the key `e` of that book
//! said "The server has no ebook for this media." **The book was good**: the
//! sandbox holds the EPUB of it.
//!
//! The parts of this test need no server and no disk.

use toutui::api::client::error::ApiError;
use toutui::logic::reader::session::{
    the_message_of_the_format, the_message_of_the_item_that_did_not_come,
};

/// The words that the program must not say for a fault of the server.
const THE_WORDS_OF_NO_EBOOK: &str = "no ebook";

#[test]
fn a_fault_of_the_item_says_what_the_server_said() {
    let message = the_message_of_the_item_that_did_not_come("an-id", &ApiError::Server(500));

    assert!(
        !message.contains(THE_WORDS_OF_NO_EBOOK),
        "a fault of the server says that the media holds no ebook: {}",
        message
    );
    assert!(
        message.contains("The server reported a fault. Status 500"),
        "the message drops what the server said: {}",
        message
    );
    assert!(
        message.contains("Press h to go back."),
        "the message names no key of the view of the reader: {}",
        message
    );
    assert!(
        !message.contains("500.."),
        "the message holds two periods after the fault: {}",
        message
    );
}

#[test]
fn every_fault_that_is_not_the_media_says_what_the_server_said() {
    for fault in [
        ApiError::Unreachable,
        ApiError::Timeout,
        ApiError::Unauthorized,
        ApiError::Forbidden,
        ApiError::Server(502),
        ApiError::Decode("a body of no JSON".to_string()),
    ] {
        let message = the_message_of_the_item_that_did_not_come("an-id", &fault);

        assert!(
            !message.contains(THE_WORDS_OF_NO_EBOOK),
            "the fault {:?} says that the media holds no ebook: {}",
            fault,
            message
        );

        let said = fault.to_string();
        let said = said.trim_end().trim_end_matches('.');
        assert!(
            message.contains(said),
            "the fault {:?} loses its reason: {}",
            fault,
            message
        );
    }
}

#[test]
fn the_status_404_of_the_item_is_the_media_that_the_server_does_not_hold() {
    let message = the_message_of_the_item_that_did_not_come("an-id", &ApiError::NotFound);

    assert!(
        message.contains("The server does not hold this media."),
        "the status 404 of the item names no media: {}",
        message
    );
    assert!(
        !message.contains(THE_WORDS_OF_NO_EBOOK),
        "the status 404 of the item says that the media holds no ebook: {}",
        message
    );
}

#[test]
fn an_item_that_the_server_gave_keeps_the_words_of_the_form() {
    // The road of an item that came back is the road of T-76, and this item
    // leaves it as it stands: a media with no ebook keeps its own sentence.
    assert!(
        the_message_of_the_format("").contains("This media has no ebook."),
        "the media with no ebook lost its sentence"
    );
    assert!(
        the_message_of_the_format("epub").contains("EPUB"),
        "the media of an EPUB lost the form of its book"
    );
}
