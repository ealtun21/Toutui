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
//!
//! Every function of [`position`] is pure. The other two parts read the file
//! and they must run in a task, and never on the thread that draws.

pub mod book;
pub mod position;
pub mod render;

pub use book::{Book, ReaderError, TocItem};
pub use position::{fraction, from_ebook_location, from_fraction, to_ebook_location, Position};
pub use render::to_lines;
