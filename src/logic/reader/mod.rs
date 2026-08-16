//! The reader of an ebook. See T-10.
//!
//! Audiobookshelf holds an ebook beside the audiobook. This module opens the
//! EPUB of an item and it gives the text to the screen. The module has three
//! parts.
//!
//! 1. [`book`] opens the file and gives the chapters, with every limit of the
//!    design section 7. A book can come from any source, therefore the module
//!    reads each chapter through a writer that stops at 8 megabytes.
//! 2. [`render`] changes the XHTML of one chapter into lines with a style.
//! 3. [`position`] holds the place where the user reads, and it changes that
//!    place into the two fields of the server.
//! 4. [`session`] holds the book while the user reads it: the chapter, the
//!    line, and the task that renders.
//! 5. [`cache`] holds the files of the ebooks of the disk at or below a limit.
//!    See T-67.
//!
//! Every function of [`position`] is pure. The other two parts read the file
//! and they must run in a task, and never on the thread that draws.

pub mod book;
pub mod cache;
pub mod cfi;
pub mod pdf;
pub mod pdf_of_a_child;
pub mod position;
pub mod render;
pub mod session;

pub use book::{Book, ReaderError, TocItem};
pub use position::{fraction, from_ebook_location, from_fraction, to_ebook_location, Position};
pub use render::to_lines;
pub use session::Reader;

/// The place where the task of the opening puts the book.
///
/// The screen is not asynchronous, therefore a task opens the book and the
/// screen takes it here at the next frame. See T-10.
type OpenedBook = std::sync::Arc<std::sync::Mutex<Option<Result<Reader, String>>>>;

/// Gives the place of the book that a task opens now.
pub fn opened_book() -> OpenedBook {
    static PLACE: std::sync::OnceLock<OpenedBook> = std::sync::OnceLock::new();
    std::sync::Arc::clone(PLACE.get_or_init(|| std::sync::Arc::new(std::sync::Mutex::new(None))))
}

/// Takes the book that the task opened, one time.
pub fn take_the_opened_book() -> Option<Result<Reader, String>> {
    opened_book().lock().ok()?.take()
}

/// The place where the task of the send says that the server took the place.
///
/// The value holds the identity of the media and the place of the user, because
/// the user can open a different book while the request stands.
type ThePlaceThatTheServerTook = std::sync::Arc<std::sync::Mutex<Option<(String, Position)>>>;

/// Gives the place that the task of the send writes.
///
/// **A place that the server did not take must go to the server again**
/// (T-291), therefore the reader says that a place is safe when the answer of
/// the server comes back, and not before it. The screen is not asynchronous,
/// therefore the task writes the place here and the loop of the application
/// gives it to the reader at the next frame.
pub fn the_place_of_the_server() -> ThePlaceThatTheServerTook {
    static PLACE: std::sync::OnceLock<ThePlaceThatTheServerTook> = std::sync::OnceLock::new();
    std::sync::Arc::clone(PLACE.get_or_init(|| std::sync::Arc::new(std::sync::Mutex::new(None))))
}

/// Says that the server took the place of a media.
pub fn say_that_the_server_took_the_place(item_id: String, place: Position) {
    if let Ok(mut box_of_the_place) = the_place_of_the_server().lock() {
        *box_of_the_place = Some((item_id, place));
    }
}

/// Takes the place that the server took, one time.
pub fn take_the_place_that_the_server_took() -> Option<(String, Position)> {
    the_place_of_the_server().lock().ok()?.take()
}
