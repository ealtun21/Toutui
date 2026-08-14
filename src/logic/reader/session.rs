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

use log::{info, warn};
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
    /// The road of the place of this book. See [`ThePlaceOfTheBook`].
    the_place_of_the_book: ThePlaceOfTheBook,
    /// The file of this book on the disk. The reader writes the time of that
    /// file while the user reads, and the removal of the cache of a second
    /// window then keeps the book. See T-153.
    path: PathBuf,
    /// The time of the last mark of the use.
    said_the_use_at: std::time::Instant,
    sender: Sender<Rendered>,
    receiver: Receiver<Rendered>,
}

/// The time between two sends of the place, while the user reads.
pub const TIME_BETWEEN_SENDS: std::time::Duration = std::time::Duration::from_secs(30);

/// Where the place of the reading of this book goes.
///
/// **A place that the program did not read must not go to the server**, and
/// the two roads that stop the send need two different words for the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThePlaceOfTheBook {
    /// The place goes to the server. This is the road of a book of the server
    /// that the program read.
    GoesToTheServer,
    /// This book is not the book of the server. The server holds one place for
    /// each media, and an item can hold more than one ebook: a send would give
    /// the place of this book to the book of the server. See T-76.
    AnotherBookOfTheItem,
    /// The server did not give the place of this book. The reader stands at the
    /// first page of a book that the server holds at another place, therefore a
    /// send would take the place of the user away. See T-178.
    TheServerDidNotGiveIt,
}

/// The sentence of a reader that sends no place, for the user.
///
/// **The two roads name two different things** (T-91): one is a book of this
/// machine, and the other is a place of the server that the program did not
/// read. The sentence of the second one names the key that asks the server
/// again (T-170).
///
/// The function is pure, therefore a test needs no server.
pub fn the_sentence_of_a_place_that_stays_here(place: ThePlaceOfTheBook) -> Option<&'static str> {
    match place {
        ThePlaceOfTheBook::GoesToTheServer => None,
        ThePlaceOfTheBook::AnotherBookOfTheItem => Some(
            "This is not the book of the server. The place of this book \
             stays on this machine.",
        ),
        ThePlaceOfTheBook::TheServerDidNotGiveIt => Some(
            "The server did not give your place in this book. The program \
             writes no place. Press h and then e to ask again.",
        ),
    }
}

/// The sentence of a book that opened at its first page, for the user.
///
/// The sentence names what the server said (T-91), it says what the program
/// did, and it names the key that asks the server again (T-170). See T-178.
pub fn the_sentence_of_a_place_that_did_not_come(
    error: &crate::api::client::error::ApiError,
) -> String {
    format!(
        "The server did not give your place: {} The program writes no place. \
         Press h and then e to ask again.",
        error
    )
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
        Reader::open_with_the_title(path, item_id, None)
    }

    /// Opens the book, and it takes the title of the media when the file gives no
    /// better one.
    ///
    /// A PDF holds no title in most files, and the name of the file on the disk is
    /// the identity of the item. The title of the media then says more to the
    /// user. See T-54.
    pub fn open_with_the_title(
        path: &Path,
        item_id: &str,
        title_of_the_media: Option<&str>,
    ) -> Result<Reader, ReaderError> {
        let book = Arc::new(Book::open(path)?);
        let (sender, receiver) = channel();

        // The title of the file has more importance for an EPUB book: that form
        // always names the book. A PDF names the file, and the name of the file
        // is the identity of the item.
        let title = match (book.pdf().is_some(), title_of_the_media) {
            (true, Some(title)) if !title.trim().is_empty() => title.to_string(),
            _ => book.title(),
        };

        Ok(Reader {
            title,
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
            the_place_of_the_book: ThePlaceOfTheBook::GoesToTheServer,
            path: path.to_path_buf(),
            // The open of the book wrote the time of the file already
            // (`the_book_is_in_use` of `get_the_ebook_of`), therefore the first
            // mark comes after the time between two marks.
            said_the_use_at: std::time::Instant::now(),
            sender,
            receiver,
        })
    }

    /// Says that the server did not give the place of this book, therefore the
    /// reader stands at the first page and it sends nothing. See T-178.
    pub fn the_server_did_not_give_the_place(&mut self) {
        self.the_place_of_the_book = ThePlaceOfTheBook::TheServerDidNotGiveIt;
    }

    /// Gives the road of the place of this book, for the words of the user.
    pub fn the_place_of_the_book(&self) -> ThePlaceOfTheBook {
        self.the_place_of_the_book
    }

    /// Says that this book is not the book of the server, therefore the place
    /// of the user stays on this machine. See T-76.
    ///
    /// The server holds one place for each media (`ebookLocation`), and not one
    /// place for each file. An item that holds two ebooks would then give the
    /// place of the second book to the first one, and the user would lose their
    /// line.
    pub fn the_place_stays_here(&mut self) {
        self.the_place_of_the_book = ThePlaceOfTheBook::AnotherBookOfTheItem;
    }

    /// Tells if the place of this book goes to the server.
    pub fn sends_the_place(&self) -> bool {
        matches!(
            self.the_place_of_the_book,
            ThePlaceOfTheBook::GoesToTheServer
        )
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
        self.sends_the_place() && wants_to_send(self.sent, self.position(), self.sent_at.elapsed())
    }

    /// Tells if the reader must send the place before it goes away.
    ///
    /// The user leaves the book, or stops the program. The place must go to
    /// the server then, whatever the time of the last send.
    /// Tells if the book is a PDF. One chapter of such a book is one page, and
    /// the screen says "page" and not "chapter". See T-54.
    pub fn holds_pages(&self) -> bool {
        self.book.pdf().is_some()
    }

    /// Gives the picture of the page that the screen shows, if the book is a PDF
    /// and if that page holds one. See T-54.
    ///
    /// The bytes stand behind an `Arc`, therefore the call copies no picture.
    pub fn picture_of_the_page(&self) -> Option<crate::logic::reader::pdf::Picture> {
        let pdf = self.book.pdf()?;
        let page = pdf.page(self.chapter)?;

        page.pictures.first().cloned()
    }

    pub fn wants_to_send_at_the_end(&self) -> bool {
        self.sends_the_place() && self.sent != Some(self.position())
    }

    /// Says that the place went to the server.
    pub fn the_place_went_to_the_server(&mut self) {
        self.sent = Some(self.position());
        self.sent_at = std::time::Instant::now();
    }

    /// Says on the disk that a program of this account reads this book now.
    ///
    /// **The removal of the cache of a second window cannot see the book that
    /// this window reads**: `keep` is a fact of the process. The window B took
    /// a book of 502 megabytes and its 43 megabytes of pages of the disk while
    /// the window A read that book — 545898521 bytes in one key of the user.
    ///
    /// The function writes the time of the file, and that time is the one word
    /// of this program that a different program reads. The removal keeps every
    /// book of a time inside `THE_LIMIT_OF_THE_USE`. See T-153.
    ///
    /// The loop of the application calls this for each turn, and the function
    /// writes the time every `THE_TIME_BETWEEN_THE_MARKS` seconds only.
    pub fn say_that_a_program_reads_this_book(&mut self) {
        if self.said_the_use_at.elapsed() < crate::logic::reader::cache::THE_TIME_BETWEEN_THE_MARKS
        {
            return;
        }

        self.said_the_use_at = std::time::Instant::now();
        crate::logic::reader::cache::the_book_is_in_use(&self.path);
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
/// that the user never opened gives `Ok(None)`.
///
/// Audiobookshelf gives `ebookProgress` as a number and `ebookLocation` as a
/// text. A different client writes an EPUBCFI in that text, and this program
/// then uses the part of the book. See T-10, section 6.1.
///
/// **The reader reads this place and it then writes it**, therefore a fault of
/// this read must stop that write (T-175). The old code held `.ok()?`: every
/// fault gave the beginning of the book, the user read one page, and the loop
/// of the program then sent that beginning to the server. A measurement of
/// 2026-08-14 with `docs/harness/one_method_fails.py`, which answered `500` to
/// `GET /api/me/progress/:id` and which forwarded the `PATCH` of that same
/// path: the server held `Alice in Wonderland` at `toutui:12:300` and 60
/// percent, the reader opened at the chapter 2 of 14 and 0 percent, and the
/// `PATCH` of the key `h` wrote `ebookProgress 0.0041` to the server. **The
/// user lost their place in the book, on every machine of that account**, and
/// no word of the reader said why. See T-178.
///
/// **A status of 404 is the answer of a book that the user never opened**, and
/// such a book has no place: the reader starts at the first page, and its send
/// gives the server the first place of that book.
pub async fn place_of_the_server(
    api: &Arc<ApiClient>,
    item_id: &str,
) -> Result<Option<(String, f64)>, crate::api::client::error::ApiError> {
    let answer: serde_json::Value = crate::app::the_progress_that_the_server_gave(
        api.get_json(&format!("/api/me/progress/{}", item_id)).await,
    )?;

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
        return Ok(None);
    }

    Ok(Some((location, part)))
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

/// Gives the sentence that says why the ebook did not come.
///
/// The endpoint of the ebook answers 404 for a media with no ebook and for an
/// item that does not exist. Therefore this function asks for the item, and it
/// names what that item holds. A request that failed for a different reason
/// keeps its own text.
async fn why_the_book_did_not_come(
    api: &Arc<ApiClient>,
    item_id: &str,
    error: &crate::api::client::error::ApiError,
) -> String {
    if !matches!(error, crate::api::client::error::ApiError::NotFound) {
        return format!("The program did not get the book: {}", error);
    }

    let item: serde_json::Value = match api.get_json(&format!("/api/items/{}", item_id)).await {
        Ok(value) => value,
        Err(_) => return "The server has no ebook for this media.".to_string(),
    };

    let format = item["media"]["ebookFile"]["ebookFormat"]
        .as_str()
        .or_else(|| item["media"]["ebookFormat"].as_str())
        .unwrap_or("");

    the_message_of_the_format(format)
}

/// Gives the sentence for the form of the ebook that the media holds.
///
/// The function is pure, therefore a test needs no server.
pub fn the_message_of_the_format(format: &str) -> String {
    let format = format.trim().to_lowercase();

    match format.as_str() {
        "" => "This media has no ebook. The key `e` needs a media with an EPUB \
               book."
            .to_string(),
        "epub" | "pdf" => format!(
            "The server holds a {} book for this media, and it did not give the \
             file. Try again, or read the log.",
            format.to_uppercase()
        ),
        other => format!(
            "The ebook of this media is a {} file. The reader shows an EPUB book \
             and a PDF book.",
            other.to_uppercase()
        ),
    }
}

/// Gives the name of the file of one ebook on the disk, with no form.
///
/// An item can hold more than one ebook, therefore the name of the item is not
/// enough: the identity of the file of the server comes after it. The book of
/// `media.ebookFile` keeps the name of the item, because the program held that
/// name before T-76 and a user must not get the file a second time.
///
/// The function is pure, therefore a test needs no disk.
pub fn the_name_of_the_book(item_id: &str, ino: Option<&str>) -> String {
    match ino {
        Some(ino) if !ino.is_empty() => format!("{}-{}", item_id, ino),
        _ => item_id.to_string(),
    }
}

/// Says that a file of the directory of the downloads is an ebook of one item.
///
/// The key `X` removes every such file. See T-65 and T-76.
pub fn the_file_is_an_ebook_of_the_item(file_name: &str, item_id: &str) -> bool {
    let Some(rest) = file_name.strip_prefix(item_id) else {
        return false;
    };

    matches!(rest, ".epub" | ".pdf") || {
        // The name of a second ebook of the item: `<item>-<ino>.epub`.
        rest.starts_with('-') && (rest.ends_with(".epub") || rest.ends_with(".pdf"))
    }
}

/// Gives the path of the ebook of one item on the disk.
pub fn ebook_path(username: &str, item_id: &str) -> PathBuf {
    ebook_path_of(username, item_id, None)
}

/// Gives the path of the EPUB book of one ebook of an item. See T-76.
pub fn ebook_path_of(username: &str, item_id: &str, ino: Option<&str>) -> PathBuf {
    crate::logic::download::downloads_base_dir(username)
        .join(format!("{}.epub", the_name_of_the_book(item_id, ino)))
}

/// Gives the path of the PDF of one item on the disk. See T-54.
///
/// The server gives the ebook of every form at one address, therefore the name
/// of the file on the disk comes from the bytes of the answer.
pub fn pdf_path(username: &str, item_id: &str) -> PathBuf {
    pdf_path_of(username, item_id, None)
}

/// Gives the path of the PDF of one ebook of an item. See T-76.
pub fn pdf_path_of(username: &str, item_id: &str, ino: Option<&str>) -> PathBuf {
    crate::logic::download::downloads_base_dir(username)
        .join(format!("{}.pdf", the_name_of_the_book(item_id, ino)))
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
    get_the_ebook_of(api, username, item_id, None).await
}

/// Reads one ebook of an item from the server, if the disk does not hold it.
///
/// `ino` names the file of the server. `None` takes the book that the server
/// opens for the item, and that is `media.ebookFile`. An item can hold more
/// than one ebook, and each of them takes its own name on the disk. See T-76.
pub async fn get_the_ebook_of(
    api: &Arc<ApiClient>,
    username: &str,
    item_id: &str,
    ino: Option<&str>,
) -> Result<PathBuf, String> {
    let path = ebook_path_of(username, item_id, ino);

    // The time of the file is the time of the last use, therefore the cache
    // holds the book that the user reads often. See T-67.
    if path.exists() {
        crate::logic::reader::cache::the_book_is_in_use(&path);
        return Ok(path);
    }

    // A PDF of a visit before this one. See T-54.
    let of_the_pdf = pdf_path_of(username, item_id, ino);

    if of_the_pdf.exists() {
        crate::logic::reader::cache::the_book_is_in_use(&of_the_pdf);
        return Ok(of_the_pdf);
    }

    let directory = path
        .parent()
        .map(|parent| parent.to_path_buf())
        .ok_or_else(|| String::from("The program cannot name the directory of the downloads."))?;

    let name = format!("{}.epub", the_name_of_the_book(item_id, ino));

    // The address of one ebook is the address of the ebook of the item with the
    // identity of the file after it. A measurement of 2026-08-11 gave the PDF
    // for `/ebook` and for `/ebook/<the ino of the PDF>`, and the EPUB book for
    // `/ebook/<the ino of that book>`. See T-76.
    let address = match ino {
        Some(ino) if !ino.is_empty() => format!("/api/items/{}/ebook/{}", item_id, ino),
        _ => format!("/api/items/{}/ebook", item_id),
    };

    if let Err(error) = api.download_to_file(&address, &directory, &name).await {
        // The endpoint of the ebook answers 404 for a media that holds no
        // ebook, and the text "The server does not have this item" then tells
        // the user nothing. One request more names the true cause. See T-52.
        return Err(why_the_book_did_not_come(api, item_id, &error).await);
    }

    // `download_to_file` takes the name of the header, and that name is the
    // name of the file of the server. The reader needs a name that it can find
    // again, therefore the file takes the name of the item.
    let with_the_name_of_the_server = directory.join(&name);

    let came = if with_the_name_of_the_server.exists() {
        with_the_name_of_the_server
    } else {
        path
    };

    // The name of the file must say the form of the file. The server gives the
    // ebook of every form at one address, therefore the bytes decide. See T-54.
    if crate::logic::reader::book::the_file_is_a_pdf(&came) {
        let of_the_pdf = pdf_path_of(username, item_id, ino);

        match std::fs::rename(&came, &of_the_pdf) {
            Ok(()) => {
                hold_the_limit_of_the_cache(username, &of_the_pdf);
                return Ok(of_the_pdf);
            }
            Err(error) => {
                // A name that says `epub` for a PDF is not correct, and the
                // reader still opens it: `Book::open` reads the bytes.
                warn!(
                    "[reader] the program cannot give the file the name of a \
                     PDF: {}",
                    error
                );
            }
        }
    }

    hold_the_limit_of_the_cache(username, &came);

    Ok(came)
}

/// Holds the cache of the ebooks at or below its limit, after a new book came.
///
/// A new book is the one moment when the cache grows, therefore the program
/// looks at the limit here and at no other moment. The book that came now stays.
/// See T-67.
fn hold_the_limit_of_the_cache(username: &str, came: &std::path::Path) {
    // The removal reads the file of the user again: a program of this account
    // that stands in a second window wrote it, or the user wrote it with an
    // editor. See T-142.
    crate::logic::reader::cache::read_the_limit_of_the_configuration_again();

    let (books, bytes) = crate::logic::reader::cache::the_removal(
        username,
        came,
        crate::logic::reader::cache::the_limit(),
    );

    if books == 0 {
        return;
    }

    info!(
        "[reader] the cache of the ebooks gave {} bytes of {} book(s) back.",
        bytes, books
    );

    // **The user must read that the program removed a book of the disk.** The
    // render draws the message in every view, therefore the reader shows it.
    // See T-71.
    crate::logic::message::say(&crate::logic::reader::cache::the_sentence_of_the_cache(
        books, bytes,
    ));
}

#[cfg(test)]
mod tests {
    use super::the_message_of_the_format;
    use super::{the_file_is_an_ebook_of_the_item, the_name_of_the_book};

    /// The book of the server keeps the name of the item, therefore a user of
    /// a version before T-76 gets no file a second time. Every other book of
    /// the item takes the identity of its file after that name.
    #[test]
    fn each_ebook_of_an_item_takes_its_own_name() {
        assert_eq!(the_name_of_the_book("an-item", None), "an-item");
        assert_eq!(the_name_of_the_book("an-item", Some("")), "an-item");
        assert_eq!(
            the_name_of_the_book("an-item", Some("94488")),
            "an-item-94488"
        );
    }

    /// The key `X` removes every ebook of the item, and it removes the file of
    /// no other item. See T-65 and T-76.
    #[test]
    fn the_key_that_removes_finds_every_book_of_the_item() {
        assert!(the_file_is_an_ebook_of_the_item("an-item.epub", "an-item"));
        assert!(the_file_is_an_ebook_of_the_item("an-item.pdf", "an-item"));
        assert!(the_file_is_an_ebook_of_the_item(
            "an-item-94488.epub",
            "an-item"
        ));
        assert!(the_file_is_an_ebook_of_the_item(
            "an-item-6121534.pdf",
            "an-item"
        ));

        assert!(!the_file_is_an_ebook_of_the_item(
            "a-different-item.epub",
            "an-item"
        ));
        assert!(
            !the_file_is_an_ebook_of_the_item("an-item.mp3", "an-item"),
            "an audio file of the key D is not an ebook"
        );
        assert!(
            !the_file_is_an_ebook_of_the_item("an-item", "an-item"),
            "a directory of the audio files is not an ebook"
        );
    }

    /// A media with no ebook must not read "The server does not have this
    /// item". The user of 2026-08-11 met that text, and it says nothing about
    /// the media. See T-52.
    #[test]
    fn the_message_names_the_form_of_the_ebook() {
        let nothing = the_message_of_the_format("");
        assert!(nothing.contains("no ebook"), "{}", nothing);

        // The reader shows a PDF book now, therefore a PDF that did not come is
        // a fault of the request. See T-54.
        let pdf = the_message_of_the_format("pdf");
        assert!(pdf.contains("PDF"), "{}", pdf);
        assert!(!pdf.contains("no ebook"), "{}", pdf);

        // The server names the form in small letters, and a different server
        // can name it in capital letters.
        assert_eq!(the_message_of_the_format("PDF"), pdf);
        assert_eq!(the_message_of_the_format(" pdf "), pdf);

        let epub = the_message_of_the_format("epub");
        assert!(epub.contains("EPUB"), "{}", epub);
        assert!(!epub.contains("no ebook"), "{}", epub);

        // A form that the reader does not show names itself.
        let other = the_message_of_the_format("mobi");
        assert!(other.contains("MOBI"), "{}", other);
    }

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
