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
