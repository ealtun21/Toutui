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
use crate::logic::reader::cfi::{self, CfiPlace, TextPlace};
use crate::logic::reader::position::Position;
use crate::logic::reader::render::{letters_of_each_line, to_lines};

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
    /// Every text of the chapter, with the path of the EPUBCFI that names it.
    /// The place of the user in the form of the web reader needs it.
    places: Vec<TextPlace>,
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
    /// The number of letters of each line. The place of the user in the form of
    /// the web reader counts the letters. See `cfi` and T-10.
    letters_of_each_line: Vec<usize>,
    /// Every text of the chapter, with the path of its EPUBCFI.
    places: Vec<TextPlace>,
    /// A place of the web reader that waits for the render of its chapter.
    ///
    /// The line of an EPUBCFI needs the lines of the chapter, and the render is
    /// not immediate. The reader therefore holds the place here, and it takes
    /// the line when the lines come.
    waiting_place: Option<CfiPlace>,
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
    /// The place that the server holds. The reader sends nothing when the
    /// place did not change.
    sent: Option<Position>,
    /// The time of the last send. The reader sends every 30 seconds at the
    /// most, and not for each line that the user reads.
    sent_at: std::time::Instant,
    sender: Sender<Rendered>,
    receiver: Receiver<Rendered>,
}

/// The time between two sends of the place, while the user reads.
pub const TIME_BETWEEN_SENDS: std::time::Duration = std::time::Duration::from_secs(30);

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
            letters_of_each_line: Vec::new(),
            places: Vec::new(),
            waiting_place: None,
            rendered_width: 0,
            waiting_for: None,
            message: None,
            contents_open: false,
            contents_line: 0,
            sent: None,
            sent_at: std::time::Instant::now(),
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

            self.letters_of_each_line = letters_of_each_line(&answer.lines);
            self.lines = answer.lines;
            self.places = answer.places;
            self.message = answer.message;
            self.waiting_for = None;

            // A place of the web reader waited for these lines. The reader now
            // knows the letters of each line, therefore it knows the line.
            if let Some(place) = self.waiting_place.take() {
                if !self.lines.is_empty() {
                    let letters = cfi::letters_before(&self.places, &place);
                    self.top_line = cfi::line_of_letters(&self.letters_of_each_line, letters);
                } else {
                    // The chapter gave no line. The reader keeps the place, and
                    // the next render of the same chapter uses it.
                    self.waiting_place = Some(place);
                }
            }

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
                // The walk over the tree runs beside the render, in the same
                // task. A measurement on 2026-08-11 gave 2 milliseconds for the
                // longest chapter of Moby Dick in a debug build, and the render
                // of that chapter needs 18 milliseconds.
                Ok(xhtml) => (to_lines(&xhtml, width), cfi::text_places(&xhtml), None),
                Err(error) => (Vec::new(), Vec::new(), Some(error.to_string())),
            });

            let answer = match tokio::time::timeout(TIME_FOR_ONE_CHAPTER, work).await {
                Ok(Ok((lines, places, message))) => Rendered {
                    chapter,
                    width,
                    lines,
                    places,
                    message,
                },
                Ok(Err(_)) => Rendered {
                    chapter,
                    width,
                    lines: Vec::new(),
                    places: Vec::new(),
                    message: Some("This chapter did not open.".to_string()),
                },
                Err(_) => Rendered {
                    chapter,
                    width,
                    lines: Vec::new(),
                    places: Vec::new(),
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
        self.letters_of_each_line = Vec::new();
        self.places = Vec::new();
        self.waiting_place = None;
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

    /// Gives the text of `ebookLocation` for the server.
    ///
    /// The reader writes an EPUBCFI, therefore the web reader of Audiobookshelf
    /// and a telephone open the same paragraph. See `cfi` and T-10.
    ///
    /// A chapter that gives no text gives no EPUBCFI. The reader then writes
    /// its own form `toutui:<spine>:<line>`, and it keeps the place of the
    /// user.
    pub fn location_text(&self) -> String {
        let letters = cfi::letters_before_line(&self.letters_of_each_line, self.top_line);
        cfi::to_epubcfi(self.chapter, &self.places, letters)
            .unwrap_or_else(|| crate::logic::reader::position::to_ebook_location(self.position()))
    }

    /// Puts the user at a place that the server gave.
    ///
    /// Three forms come from the server, and the program takes the first one
    /// that it understands.
    ///
    /// 1. An EPUBCFI. The web reader writes it, and this reader also writes it.
    ///    The reader takes the chapter now, and it takes the line when the
    ///    render of that chapter gives the lines.
    /// 2. The older form of this program, `toutui:<spine>:<line>`. A server
    ///    holds such a text from a version before 0.7.8.
    /// 3. Nothing that the program understands. The part of the book then gives
    ///    the chapter.
    pub fn go_to_the_place_of_the_server(&mut self, location: &str, ebook_fraction: f64) {
        if let Some(place) = cfi::parse_epubcfi(location) {
            let spine = place.spine.min(self.chapter_count().saturating_sub(1));
            self.go_to_chapter(spine);
            self.waiting_place = Some(place);
            return;
        }

        let place =
            crate::logic::reader::position::from_ebook_location(location).unwrap_or_else(|| {
                crate::logic::reader::position::from_fraction(&self.sizes, ebook_fraction)
            });

        self.go_to_chapter(place.spine.min(self.chapter_count().saturating_sub(1)));
        self.top_line = place.line;
    }
}

impl Reader {
    /// Tells if the reader must send the place now.
    ///
    /// The reader sends when the place changed and 30 seconds went by. A user
    /// who reads a page each ten seconds therefore makes one request each 30
    /// seconds, and not one for each page.
    pub fn wants_to_send(&self) -> bool {
        wants_to_send(self.sent, self.position(), self.sent_at.elapsed())
    }

    /// Tells if the reader must send the place before it goes away.
    ///
    /// The user leaves the book, or stops the program. The place must go to
    /// the server then, whatever the time of the last send.
    pub fn wants_to_send_at_the_end(&self) -> bool {
        self.sent != Some(self.position())
    }

    /// Says that the place went to the server.
    pub fn the_place_went_to_the_server(&mut self) {
        self.sent = Some(self.position());
        self.sent_at = std::time::Instant::now();
    }
}

/// The rule of the send. See `Reader::wants_to_send`.
pub fn wants_to_send(
    sent: Option<Position>,
    now: Position,
    since_the_last_send: std::time::Duration,
) -> bool {
    if sent == Some(now) {
        return false;
    }

    // The first place of a book goes at once. The user then sees that the
    // program holds their place.
    if sent.is_none() {
        return since_the_last_send >= TIME_BETWEEN_SENDS;
    }

    since_the_last_send >= TIME_BETWEEN_SENDS
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

/// Asks the server where the user stopped reading.
///
/// The answer gives the text of the place and the part of the book. A book
/// that the user never opened gives nothing.
///
/// Audiobookshelf gives `ebookProgress` as a number and `ebookLocation` as a
/// text. A different client writes an EPUBCFI in that text, and this program
/// then uses the part of the book. See T-10, section 6.1.
pub async fn place_of_the_server(api: &Arc<ApiClient>, item_id: &str) -> Option<(String, f64)> {
    let answer: serde_json::Value = api
        .get_json(&format!("/api/me/progress/{}", item_id))
        .await
        .ok()?;

    let location = answer
        .get("ebookLocation")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_string();

    let part = answer
        .get("ebookProgress")
        .and_then(number_of)
        .unwrap_or(0.0);

    if location.is_empty() && part <= 0.0 {
        return None;
    }

    Some((location, part))
}

/// Reads a number that the server gives as a number or as a text.
///
/// Audiobookshelf gives `currentTime` as a text. A reader that takes a number
/// only finds nothing there.
fn number_of(value: &serde_json::Value) -> Option<f64> {
    match value {
        serde_json::Value::Number(number) => number.as_f64(),
        serde_json::Value::String(text) => text.parse::<f64>().ok(),
        _ => None,
    }
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

    fn a_place(spine: usize, line: usize) -> Position {
        Position { spine, line }
    }

    #[test]
    fn the_reader_sends_nothing_when_the_place_did_not_change() {
        let now = a_place(2, 40);
        assert!(!wants_to_send(
            Some(now),
            now,
            std::time::Duration::from_secs(600)
        ));
    }

    #[test]
    fn the_reader_waits_for_the_time_between_two_sends() {
        assert!(!wants_to_send(
            Some(a_place(2, 10)),
            a_place(2, 40),
            std::time::Duration::from_secs(5)
        ));
        assert!(wants_to_send(
            Some(a_place(2, 10)),
            a_place(2, 40),
            TIME_BETWEEN_SENDS
        ));
    }

    #[test]
    fn a_book_that_sent_nothing_yet_also_waits() {
        // A book that the user opens and leaves at once must make no request.
        assert!(!wants_to_send(
            None,
            a_place(0, 0),
            std::time::Duration::from_secs(1)
        ));
        assert!(wants_to_send(
            None,
            a_place(0, 0),
            std::time::Duration::from_secs(31)
        ));
    }

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
    fn a_number_of_the_server_comes_as_a_number_or_as_a_text() {
        assert_eq!(number_of(&serde_json::json!(0.42)), Some(0.42));
        assert_eq!(number_of(&serde_json::json!("0.42")), Some(0.42));
        assert_eq!(number_of(&serde_json::json!("not a number")), None);
        assert_eq!(number_of(&serde_json::json!(null)), None);
    }

    #[test]
    fn the_path_of_an_ebook_holds_the_identity_of_the_item() {
        let path = ebook_path("a-user", "an-item");
        assert!(path.to_string_lossy().ends_with("an-item.epub"));
        assert!(path.to_string_lossy().contains("a-user"));
    }
}
