//! A place of a book that the server did not take goes to the server again.
//! See T-291.
//!
//! **The reader sends the place of the user on three roads**: the rule of the
//! time of every 30 seconds, the key `s`, and the key `h` that leaves the
//! book. The old code said that the place went to the server **before** the
//! request:
//!
//! ```text
//! if let Some(reader) = self.reader.as_mut() {
//!     reader.the_place_went_to_the_server();     // sent = Some(position())
//! }
//!
//! tokio::spawn(async move {
//!     let text = match api.patch_json(…).await {
//!         Ok(()) => "The server has the place of the book.",
//!         Err(error) => format!("The server did not take the place: {}", error),
//!     };
//!     …
//! });
//! ```
//!
//! The two rules of the send each ask `sent != Some(position())`, therefore a
//! request that the server refused took the place of the user out of both of
//! them: **the program said the fault one time, and it sent that place never
//! again.**
//!
//! The measurement of the real program v0.8.119 inside tmux against the
//! sandbox, with `docs/harness/one_method_fails.py` on the port 13500 and the
//! rule `PATCH:/api/me/progress/8fda6e43-…`. The server held
//! `Alice in Wonderland` at `ebookLocation toutui:12:300`, and the reader
//! opened at `chapter 13 of 14`. The user pressed `n` and then `s`:
//!
//! | The moment | v0.8.119 | The correction |
//! |---|---|---|
//! | The key `s` | `500 PATCH /api/me/progress/…` | `500 PATCH /api/me/progress/…` |
//! | The words of the program | `The server did not take the place: The server reported a fault. Status 500.` | the same |
//! | 45 seconds in the book | **no request at all** | a second `PATCH` |
//! | The key `h` that leaves the book | **no request at all** | a third `PATCH` |
//! | The server after it | **`toutui:12:300`, the place of chapter 13** | the place of chapter 14 |
//!
//! **The user read chapter 14 of 14, and the server kept chapter 13 on every
//! machine of that account.** The rule of T-212 is the same shape: a value of
//! the user that reached no machine must keep the one copy that holds it.
//!
//! **This test needs no sandbox and no server**: the fault stands in the two
//! rules of the send and in the box of the process that carries the answer of
//! the server back to the reader.

use std::time::Duration;
use toutui::logic::reader::session::{wants_to_send, TIME_BETWEEN_SENDS};
use toutui::logic::reader::{
    say_that_the_server_took_the_place, take_the_place_that_the_server_took, Position, Reader,
};

/// The media of this measurement.
const THE_MEDIA: &str = "8fda6e43-0728-46ad-98bc-4c8634e299ad";

/// A place that the server did not take stays unsent, and a place that the
/// server took stops the next request.
///
/// **The parts of this test stay in one function**: two test functions of one
/// binary take a thread each, and the box of the process holds one slot for
/// the whole binary (T-144 and T-157).
#[test]
fn a_place_that_the_server_did_not_take_goes_again() {
    let path = std::path::Path::new("tests/data/alice.epub");
    let mut reader = Reader::open(path, THE_MEDIA).expect("the book must open");

    assert!(
        reader.sends_the_place(),
        "the book of the server sends its place"
    );

    // The user reads a chapter after the chapter of the server.
    reader.next_chapter();
    let the_place_of_the_user = reader.position();

    assert_ne!(
        the_place_of_the_user,
        Position::default(),
        "the user must stand at a place that the server does not hold"
    );

    // The send stands. **The reader holds no place of the server yet**: the
    // answer of the server did not come.
    reader.the_place_goes_to_the_server();

    assert_eq!(
        reader.the_place_that_the_server_holds(),
        None,
        "a request that stands is no place of the server"
    );

    // The road of the fault: the server refused that request, and the answer
    // never came. The two rules of the send must ask for that place again.
    assert!(
        reader.wants_to_send_at_the_end(),
        "the key h must send a place that the server did not take"
    );

    assert!(
        wants_to_send(
            reader.the_place_that_the_server_holds(),
            reader.position(),
            TIME_BETWEEN_SENDS + Duration::from_secs(1),
        ),
        "the rule of the time must send a place that the server did not take"
    );

    // The road of the answer that came. The task of the send writes the place
    // in the box of the process, and the loop of the application takes it one
    // time.
    say_that_the_server_took_the_place(THE_MEDIA.to_string(), the_place_of_the_user);

    let (the_media_of_the_box, the_place_of_the_box) =
        take_the_place_that_the_server_took().expect("the box must hold the place of the send");

    assert_eq!(
        the_media_of_the_box, THE_MEDIA,
        "the box names the media, because the user can open a different book \
         while the request stands"
    );
    assert_eq!(the_place_of_the_box, the_place_of_the_user);
    assert!(
        take_the_place_that_the_server_took().is_none(),
        "the loop takes the place of the box one time"
    );

    reader.the_place_went_to_the_server(the_place_of_the_box);

    assert!(
        !reader.wants_to_send_at_the_end(),
        "the key h sends no place that the server holds already"
    );
    assert!(
        !wants_to_send(
            reader.the_place_that_the_server_holds(),
            reader.position(),
            TIME_BETWEEN_SENDS + Duration::from_secs(1),
        ),
        "the rule of the time sends no place that the server holds already"
    );

    // **The place of the send is not the place of this moment**: the user
    // reads more lines while the request stands, and the server holds no
    // place of those lines.
    reader.next_chapter();

    assert!(
        reader.wants_to_send_at_the_end(),
        "the server holds the place of the send alone, therefore the lines \
         after it must go to the server too"
    );

    // A place of a different media says nothing of this reader. The
    // application asks the identity of the box before it gives that place to
    // the reader.
    say_that_the_server_took_the_place("a-media-of-no-reader".to_string(), reader.position());

    let (the_media_of_another_book, _) =
        take_the_place_that_the_server_took().expect("the box holds the place of that send");

    assert_ne!(
        the_media_of_another_book, THE_MEDIA,
        "a place of one book must not stop the send of another book"
    );
}
