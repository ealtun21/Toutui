//! The reader of one book, while the user reads it. See T-10.
//!
//! `book.rs` opens the file and gives the text. This module holds what the
//! user does with it: which chapter, which line, and where the file is on the
//! disk.
//!
//! The render of a chapter does not run on the thread that draws. A chapter of
//! Moby Dick needs 3 milliseconds in a release build and 18 milliseconds in a
//! debug build, and a page with 10000 nested tags needs 895 milliseconds.
//! Therefore a task renders the chapter, and the screen shows "Reading…" until
//! the lines come.

use ratatui::text::Line;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

use crate::api::client::ApiClient;
use crate::logic::reader::book::{Book, ReaderError, TocItem};
use crate::logic::reader::position::Position;
use crate::logic::reader::render::to_lines;

/// The time that one chapter may take to render.
///
/// A page with 10000 nested tags needs 895 milliseconds in a debug build, and
/// the time grows with the square of the depth. A page that is worse than that
/// must not hold the reader for ever. See T-10, section 7.2.
const TIME_FOR_ONE_CHAPTER: std::time::Duration = std::time::Duration::from_secs(5);

/// The largest size of an ebook that the program receives.
pub const MAX_EBOOK_BYTES: u64 = 256 * 1024 * 1024;

/// What the task of the render sends back.
struct Rendered {
    chapter: usize,
    width: u16,
    lines: Vec<Line<'static>>,
    message: Option<String>,
}

/// The reader of one book.
pub struct Reader {
    book: Arc<Book>,
    /// The identity of the item on the server. The position goes there.
    pub item_id: String,
    pub title: String,
    pub author: String,
    /// The chapter that the screen shows.
    pub chapter: usize,
    /// The first line that the screen shows.
    pub top_line: usize,
    /// The lines of the chapter, after the render.
    pub lines: Vec<Line<'static>>,
    /// The width that the lines follow. A different width needs a new render.
    rendered_width: u16,
    /// The chapter that the task renders now.
    waiting_for: Option<usize>,
    /// A message for the user: "Reading…", or the reason of a failure.
    pub message: Option<String>,
    /// The table of contents. The key `t` shows it.
    pub contents: Vec<TocItem>,
    pub contents_open: bool,
    /// The place of each chapter in the list of the contents.
    pub contents_line: usize,
    /// The size of each chapter. The part of the book comes from it.
    sizes: Vec<u64>,
    sender: Sender<Rendered>,
    receiver: Receiver<Rendered>,
}

impl std::fmt::Debug for Reader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Reader({}, chapter {} of {})",
            self.title,
            self.chapter + 1,
            self.book.chapter_count()
        )
    }
}

impl Reader {
    /// Opens a book that stands on the disk.
    pub fn open(path: &Path, item_id: &str) -> Result<Reader, ReaderError> {
        let book = Arc::new(Book::open(path)?);
        let (sender, receiver) = channel();

        Ok(Reader {
            title: book.title(),
            author: book.author(),
            contents: book.contents(),
            sizes: Vec::new(),
            book,
            item_id: item_id.to_string(),
            chapter: 0,
            top_line: 0,
            lines: Vec::new(),
            rendered_width: 0,
            waiting_for: None,
            message: None,
            contents_open: false,
            contents_line: 0,
            sender,
            receiver,
        })
    }

    pub fn chapter_count(&self) -> usize {
        self.book.chapter_count()
    }

    /// Takes the lines that the task made, if they are ready.
    ///
    /// The render calls this function for each frame. It never waits.
    pub fn take_the_answer(&mut self) {
        while let Ok(answer) = self.receiver.try_recv() {
            // An answer of an old chapter or of an old width has no use.
            if answer.chapter != self.chapter || answer.width != self.rendered_width {
                continue;
            }

            self.lines = answer.lines;
            self.message = answer.message;
            self.waiting_for = None;

            // A chapter that gives no line is the wrapper of the cover. Every
            // book of the measurement has one. The reader goes past it.
            if self.lines.is_empty()
                && self.message.is_none()
                && self.chapter + 1 < self.chapter_count()
            {
                self.go_to_chapter(self.chapter + 1);
            }
        }
    }

    /// Starts the render of the chapter for a width, if that is necessary.
    pub fn render_for(&mut self, width: u16) {
        if width == 0 {
            return;
        }

        let same =
            width == self.rendered_width && self.waiting_for.is_none() && !self.lines.is_empty();

        if same || self.waiting_for == Some(self.chapter) && width == self.rendered_width {
            return;
        }

        self.rendered_width = width;
        self.waiting_for = Some(self.chapter);
        self.message = Some("Reading…".to_string());

        let book = Arc::clone(&self.book);
        let sender = self.sender.clone();
        let chapter = self.chapter;

        tokio::spawn(async move {
            let work = tokio::task::spawn_blocking(move || match book.chapter_xhtml(chapter) {
                Ok(xhtml) => (to_lines(&xhtml, width), None),
                Err(error) => (Vec::new(), Some(error.to_string())),
            });

            let answer = match tokio::time::timeout(TIME_FOR_ONE_CHAPTER, work).await {
                Ok(Ok((lines, message))) => Rendered {
                    chapter,
                    width,
                    lines,
                    message,
                },
                Ok(Err(_)) => Rendered {
                    chapter,
                    width,
                    lines: Vec::new(),
                    message: Some("This chapter did not open.".to_string()),
                },
                Err(_) => Rendered {
                    chapter,
                    width,
                    lines: Vec::new(),
                    message: Some("This chapter is too complex.".to_string()),
                },
            };

            let _ = sender.send(answer);
        });
    }

    /// Goes to a chapter. The screen starts at its first line.
    pub fn go_to_chapter(&mut self, chapter: usize) {
        if chapter >= self.chapter_count() {
            return;
        }

        self.chapter = chapter;
        self.top_line = 0;
        self.lines = Vec::new();
        self.rendered_width = 0;
        self.waiting_for = None;
        self.message = Some("Reading…".to_string());
    }

    pub fn next_chapter(&mut self) {
        if self.chapter + 1 < self.chapter_count() {
            self.go_to_chapter(self.chapter + 1);
        }
    }

    pub fn previous_chapter(&mut self) {
        if self.chapter > 0 {
            self.go_to_chapter(self.chapter - 1);
        }
    }

    /// Moves the first line of the screen.
    pub fn scroll(&mut self, by: i64, height: u16) {
        self.top_line = new_top_line(self.top_line, by, self.lines.len(), height);
    }

    pub fn to_the_start(&mut self) {
        self.top_line = 0;
    }

    pub fn to_the_end(&mut self, height: u16) {
        self.top_line = last_top_line(self.lines.len(), height);
    }

    /// Gives the place of the user, for the server.
    pub fn position(&self) -> Position {
        Position {
            spine: self.chapter,
            line: self.top_line,
        }
    }

    /// Gives the part of the book that the user read, from 0 to 1.
    pub fn fraction(&self) -> f64 {
        crate::logic::reader::position::fraction(&self.sizes, self.position(), self.lines.len())
    }

    /// Reads the size of every chapter. The part of the book needs it.
    ///
    /// `rbook` gives no size without a read, therefore this work takes 18
    /// milliseconds for a book of twelve chapters in a debug build. The caller
    /// runs it one time, in a task.
    pub fn measure_the_chapters(&mut self) {
        if self.sizes.is_empty() {
            self.sizes = self.book.chapter_sizes();
        }
    }

    /// Puts the user at a place that the server gave.
    pub fn go_to_the_place_of_the_server(&mut self, location: &str, ebook_fraction: f64) {
        // A place that this program wrote names the chapter and the line. A
        // place of a different client is an EPUBCFI, and this program does not
        // read that form. The part of the book then gives the chapter.
        let place =
            crate::logic::reader::position::from_ebook_location(location).unwrap_or_else(|| {
                crate::logic::reader::position::from_fraction(&self.sizes, ebook_fraction)
            });

        self.go_to_chapter(place.spine.min(self.chapter_count().saturating_sub(1)));
        self.top_line = place.line;
    }
}

/// Gives the first line of the screen after a move.
///
/// The function never goes past the end of the chapter, and never before its
/// first line.
pub fn new_top_line(top: usize, by: i64, lines: usize, height: u16) -> usize {
    let last = last_top_line(lines, height);
    let wanted = top as i64 + by;

    wanted.clamp(0, last as i64) as usize
}

/// Gives the first line of the screen at the end of a chapter.
///
/// The last screen holds the last lines, therefore the user reads the end of
/// the chapter and no empty space.
pub fn last_top_line(lines: usize, height: u16) -> usize {
    lines.saturating_sub(usize::from(height).max(1))
}

/// Gives the path of the ebook of one item on the disk.
pub fn ebook_path(username: &str, item_id: &str) -> PathBuf {
    crate::logic::download::downloads_base_dir(username).join(format!("{}.epub", item_id))
}

/// Reads the ebook of one item from the server, if the disk does not hold it.
///
/// The program keeps the file, therefore a second visit needs no request and
/// the reader also works with no server.
pub async fn get_the_ebook(
    api: &Arc<ApiClient>,
    username: &str,
    item_id: &str,
) -> Result<PathBuf, String> {
    let path = ebook_path(username, item_id);

    if path.exists() {
        return Ok(path);
    }

    let directory = path
        .parent()
        .map(|parent| parent.to_path_buf())
        .ok_or_else(|| String::from("The program cannot name the directory of the downloads."))?;

    let name = format!("{}.epub", item_id);

    api.download_to_file(&format!("/api/items/{}/ebook", item_id), &directory, &name)
        .await
        .map_err(|error| format!("The program did not get the book: {}", error))?;

    // `download_to_file` takes the name of the header, and that name is the
    // name of the file of the server. The reader needs a name that it can find
    // again, therefore the file takes the name of the item.
    let with_the_name_of_the_server = directory.join(&name);

    if with_the_name_of_the_server.exists() {
        return Ok(with_the_name_of_the_server);
    }

    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_screen_never_goes_past_the_end() {
        // 100 lines in a screen of 20 lines: the last screen starts at 80.
        assert_eq!(last_top_line(100, 20), 80);
        assert_eq!(new_top_line(70, 100, 100, 20), 80);
        assert_eq!(new_top_line(80, 1, 100, 20), 80);
    }

    #[test]
    fn the_screen_never_goes_before_the_first_line() {
        assert_eq!(new_top_line(5, -100, 100, 20), 0);
        assert_eq!(new_top_line(0, -1, 100, 20), 0);
    }

    #[test]
    fn a_chapter_that_is_shorter_than_the_screen_starts_at_the_first_line() {
        assert_eq!(last_top_line(5, 20), 0);
        assert_eq!(new_top_line(0, 10, 5, 20), 0);
    }

    #[test]
    fn a_screen_of_no_height_gives_no_fault() {
        assert_eq!(last_top_line(100, 0), 99);
        assert_eq!(new_top_line(0, 5, 100, 0), 5);
    }

    #[test]
    fn a_chapter_with_no_line_stays_at_the_first_line() {
        assert_eq!(last_top_line(0, 20), 0);
        assert_eq!(new_top_line(0, 10, 0, 20), 0);
    }

    #[test]
    fn the_path_of_an_ebook_holds_the_identity_of_the_item() {
        let path = ebook_path("a-user", "an-item");
        assert!(path.to_string_lossy().ends_with("an-item.epub"));
        assert!(path.to_string_lossy().contains("a-user"));
    }
}
