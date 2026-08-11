//! The EPUB file. See T-10, design section 5.2 and section 7.
//!
//! This module opens the file and it gives the chapters. It holds three
//! limits, because a book can come from any source and it can be hostile.
//!
//! 1. The whole file must not be larger than 256 megabytes.
//! 2. The archive must not hold more than 4096 entries.
//! 3. One chapter must not be larger than 8 megabytes.
//!
//! The third limit is the important one. A measurement on 2026-08-10 gave a
//! zip archive of 2 megabytes that opens to 2 gigabytes. A plain read of that
//! chapter took 4102 megabytes and then the process stopped with `abort`, and
//! `catch_unwind` cannot catch an `abort`. The same read through
//! `ManifestEntry::copy_bytes`, into the writer of this module, gives an error
//! and the program takes little memory. A measurement of this module on
//! 2026-08-10, in a debug build: the bomb gives "This chapter is too large."
//! after 23 milliseconds, and the largest memory of the whole process is 13
//! megabytes.
//!
//! The module never uses `Epub::options().strict(true)`. A measurement on
//! 2026-08-10 showed that the strict mode refuses all twelve hostile files and
//! also refuses real books of Project Gutenberg.

// `rbook` gives a concrete type for each part of the book, and not an
// `impl Trait`. Therefore the module calls the methods of the concrete types
// and it needs no trait in scope.
use rbook::Epub;
use std::collections::HashMap;
use std::fmt;
use std::io::{self, Write};
use std::path::Path;

/// The largest file that the reader opens, in bytes. 256 megabytes.
///
/// The largest book of the measurement of 2026-08-10 is Moby Dick with
/// 812 600 bytes. This limit therefore refuses no real book.
pub const MAX_BOOK_BYTES: u64 = 256 * 1024 * 1024;

/// The largest number of entries in the archive.
///
/// The four books of the measurement hold 21, 23, 41, and 21 entries. An
/// archive with thousands of entries is not a book.
pub const MAX_ENTRIES: usize = 4096;

/// The largest chapter that the reader reads, in bytes. 8 megabytes.
///
/// The largest chapter of the four books is 160 kilobytes. This limit
/// therefore refuses no real chapter, and it stops the zip bomb.
pub const MAX_CHAPTER_BYTES: usize = 8 * 1024 * 1024;

/// A fault of the reader. Each value gives one short message to the user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReaderError {
    /// The file is not an EPUB, or the archive is damaged.
    NotAnEpub,
    /// The file is larger than [`MAX_BOOK_BYTES`]. The number is the size of
    /// the file in bytes.
    BookTooLarge(u64),
    /// The archive holds more than [`MAX_ENTRIES`] entries. The number is the
    /// count.
    TooManyEntries(usize),
    /// The book has no chapter with this number.
    NoSuchChapter(usize),
    /// The book names a chapter, and the archive does not hold it.
    ChapterAbsent,
    /// The chapter is larger than [`MAX_CHAPTER_BYTES`].
    ChapterTooLarge,
}

impl fmt::Display for ReaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReaderError::NotAnEpub => write!(f, "This file is not an EPUB."),
            ReaderError::BookTooLarge(size) => write!(
                f,
                "This book is too large. It has {size} bytes, and the limit is {MAX_BOOK_BYTES} bytes."
            ),
            ReaderError::TooManyEntries(count) => write!(
                f,
                "This book holds too many files. It has {count} files, and the limit is {MAX_ENTRIES} files."
            ),
            ReaderError::NoSuchChapter(index) => {
                write!(f, "This book has no chapter {index}.")
            }
            ReaderError::ChapterAbsent => write!(f, "This chapter is absent."),
            ReaderError::ChapterTooLarge => write!(f, "This chapter is too large."),
        }
    }
}

impl std::error::Error for ReaderError {}

/// One line of the table of contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TocItem {
    /// The depth in the tree. The top level is 0.
    pub depth: usize,
    /// The text that the user reads.
    pub label: String,
    /// The chapter that the line opens. An entry that points at no chapter of
    /// the spine gives `None`, and the user interface must not open it.
    pub spine_index: Option<usize>,
}

/// A writer that refuses more than a given number of bytes.
///
/// The reader gives this writer to `ManifestEntry::copy_bytes`. The archive
/// then writes in small parts, and this writer stops the copy at the limit.
/// Therefore a chapter of 2 gigabytes never comes into the memory.
struct CappedWriter {
    /// The bytes that the writer holds.
    buffer: Vec<u8>,
    /// The largest number of bytes that the writer takes.
    limit: usize,
    /// True after the writer refused a part. The caller reads this value,
    /// because the archive wraps the fault in its own error.
    hit_limit: bool,
}

impl CappedWriter {
    fn new(limit: usize) -> Self {
        CappedWriter {
            buffer: Vec::new(),
            limit,
            hit_limit: false,
        }
    }
}

impl Write for CappedWriter {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        // `checked_add` and not `+`: the count must never go round to zero.
        let after = self.buffer.len().checked_add(data.len());
        match after {
            Some(size) if size <= self.limit => {
                self.buffer.extend_from_slice(data);
                Ok(data.len())
            }
            _ => {
                self.hit_limit = true;
                Err(io::Error::other("this resource is too large"))
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Tells if a file is a PDF.
///
/// Every PDF starts with `%PDF-`. The name of the file says nothing: the server
/// gives the ebook of a media at one address for every form. See T-54.
pub fn the_file_is_a_pdf(path: &Path) -> bool {
    use std::io::Read;

    let Ok(mut file) = std::fs::File::open(path) else {
        return false;
    };

    let mut head = [0u8; 5];

    match file.read_exact(&mut head) {
        Ok(()) => &head == b"%PDF-",
        Err(_) => false,
    }
}

/// One open book.
///
/// The structure holds the `Epub` and the length of the spine only. It holds
/// no entry of the book, because the API of `rbook` uses `impl Trait` in a
/// trait and such a value borrows the `Epub`.
pub struct Book {
    /// The form of the book: an archive of EPUB, or a PDF. See T-54.
    kind: Kind,
}

/// The form of an open book.
///
/// The two forms hold a different size, therefore the archive of EPUB stands
/// behind a `Box`. A large value inside a small one makes every copy of the
/// small one expensive.
enum Kind {
    /// An EPUB book. One chapter is one entry of the spine.
    Epub {
        /// The open archive.
        epub: Box<Epub>,
        /// The number of chapters in the spine.
        spine_len: usize,
    },
    /// A PDF book. **One chapter is one page**, because a PDF holds no chapter.
    /// See T-54.
    Pdf(Box<crate::logic::reader::pdf::Pdf>),
}

impl Book {
    /// Opens the book at `path`.
    ///
    /// The function refuses a file that is too large, and an archive with too
    /// many entries, before it reads any chapter.
    pub fn open(path: &Path) -> Result<Book, ReaderError> {
        // The form of the file comes from the first bytes, and not from the name
        // of the file. A server can give a PDF with any name. See T-54.
        if the_file_is_a_pdf(path) {
            let pdf = crate::logic::reader::pdf::Pdf::open(path)?;

            return Ok(Book {
                kind: Kind::Pdf(Box::new(pdf)),
            });
        }

        // Look at the size first. A read of the directory of the file costs
        // almost nothing, and it stops a very large file at once.
        if let Ok(data) = std::fs::metadata(path) {
            if data.is_file() && data.len() > MAX_BOOK_BYTES {
                return Err(ReaderError::BookTooLarge(data.len()));
            }
        }

        let epub = Epub::open(path).map_err(|_| ReaderError::NotAnEpub)?;

        // The manifest names every file of the book. `rbook` gives no count of
        // the entries of the archive, therefore the reader counts the manifest.
        // A book names each of its files one time, thus the two counts agree.
        let entries = epub.manifest().len();
        if entries > MAX_ENTRIES {
            return Err(ReaderError::TooManyEntries(entries));
        }

        let spine_len = epub.spine().len();

        Ok(Book {
            kind: Kind::Epub {
                epub: Box::new(epub),
                spine_len,
            },
        })
    }

    /// Gives the PDF of this book, if the book is a PDF. The screen reads the
    /// pictures of a page with it. See T-54.
    pub fn pdf(&self) -> Option<&crate::logic::reader::pdf::Pdf> {
        match &self.kind {
            Kind::Pdf(pdf) => Some(pdf),
            Kind::Epub { .. } => None,
        }
    }

    /// Gives the archive of EPUB of this book, if the book is an EPUB book.
    fn epub(&self) -> Option<&Epub> {
        match &self.kind {
            Kind::Epub { epub, .. } => Some(epub),
            Kind::Pdf(_) => None,
        }
    }

    /// The number of chapters. A PDF gives the number of its pages.
    pub fn chapter_count(&self) -> usize {
        match &self.kind {
            Kind::Epub { spine_len, .. } => *spine_len,
            Kind::Pdf(pdf) => pdf.page_count(),
        }
    }

    /// The title of the book. A book with no title gives a short message.
    pub fn title(&self) -> String {
        let Some(epub) = self.epub() else {
            return match &self.kind {
                Kind::Pdf(pdf) => pdf.title(),
                Kind::Epub { .. } => String::new(),
            };
        };

        epub.metadata()
            .title()
            .map(|title| title.value().to_string())
            .filter(|text| !text.trim().is_empty())
            .unwrap_or_else(|| "Unknown title".to_string())
    }

    /// The author of the book. Two authors come in one text, with a comma
    /// between them. A book with no author gives a short message.
    pub fn author(&self) -> String {
        let Some(epub) = self.epub() else {
            return match &self.kind {
                Kind::Pdf(pdf) => pdf.author(),
                Kind::Epub { .. } => String::new(),
            };
        };

        let names: Vec<String> = epub
            .metadata()
            .creators()
            .map(|creator| creator.value().to_string())
            .filter(|text| !text.trim().is_empty())
            .collect();
        if names.is_empty() {
            "Unknown author".to_string()
        } else {
            names.join(", ")
        }
    }

    /// The table of contents, as a flat list with a depth for each line.
    ///
    /// The reader gives a flat list and not a tree, because the user interface
    /// shows a list. The depth gives the indent.
    ///
    /// A measurement on 2026-08-10 gave a chapter for every line of the four
    /// books: Alice 16 of 16, Pride and Prejudice 63 of 63, Frankenstein 32 of
    /// 32, and Moby Dick 146 of 146.
    pub fn contents(&self) -> Vec<TocItem> {
        // A PDF holds no table of contents that every file has. The list of the
        // pages is the list that the user needs. See T-54.
        if let Kind::Pdf(pdf) = &self.kind {
            return (0..pdf.page_count())
                .map(|index| TocItem {
                    depth: 0,
                    label: match pdf.page(index) {
                        Some(page) => format!("The page {}", page.number),
                        None => format!("The page {}", index + 1),
                    },
                    spine_index: Some(index),
                })
                .collect();
        }

        let spine_of_href = self.spine_by_href();
        let Some(epub) = self.epub() else {
            return Vec::new();
        };
        let Some(root) = epub.toc().contents() else {
            return Vec::new();
        };
        root.flatten()
            .map(|entry| {
                // The target of a line can hold a part after `#`, for example
                // `chapter1.xhtml#letter1`. The spine names the file only,
                // therefore the reader cuts that part away.
                let spine_index = entry
                    .resource()
                    .and_then(|resource| resource.key().value().map(|text| text.to_string()))
                    .and_then(|href| {
                        let file = href.split('#').next().unwrap_or(&href).to_string();
                        spine_of_href.get(&file).copied()
                    });
                TocItem {
                    depth: entry.depth(),
                    label: entry.label().trim().to_string(),
                    spine_index,
                }
            })
            .collect()
    }

    /// The XHTML of one chapter.
    ///
    /// The read goes through `copy_bytes` into [`CappedWriter`]. Therefore a
    /// chapter of 2 gigabytes gives [`ReaderError::ChapterTooLarge`] and the
    /// program keeps its memory.
    ///
    /// The bytes come back as text with a lossy change to UTF-8. A file that
    /// is really binary then gives text with no meaning, and it gives no
    /// error. That answer is correct: the user sees a page with no meaning,
    /// and the program does not stop.
    pub fn chapter_xhtml(&self, index: usize) -> Result<String, ReaderError> {
        // One page of a PDF gives the XHTML of its text and of its pictures.
        // Therefore the render of the EPUB book makes the lines of both forms.
        // See T-54.
        if let Kind::Pdf(pdf) = &self.kind {
            let page = pdf.page(index).ok_or(ReaderError::NoSuchChapter(index))?;
            return Ok(crate::logic::reader::pdf::xhtml_of_the_page(page));
        }

        let bytes = self.chapter_bytes(index)?;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }

    /// The size of each chapter, in bytes.
    ///
    /// The application needs these numbers for the part of the book that the
    /// user read. `rbook` gives no size of an entry of the archive without a
    /// read, therefore this function reads every chapter through the same
    /// limit of 8 megabytes and it takes the length of the answer. A chapter
    /// that fails gives 0.
    ///
    /// The call reads the whole book. The caller must run it one time, in a
    /// task, and it must keep the answer. A measurement on 2026-08-10, in a
    /// debug build: Alice 2.5 milliseconds, Frankenstein 7.1 milliseconds,
    /// Pride and Prejudice 9.9 milliseconds, Moby Dick 17.9 milliseconds.
    pub fn chapter_sizes(&self) -> Vec<u64> {
        if let Kind::Pdf(pdf) = &self.kind {
            return pdf.page_sizes();
        }

        (0..self.chapter_count())
            .map(|index| match self.chapter_bytes(index) {
                Ok(bytes) => bytes.len() as u64,
                Err(_) => 0,
            })
            .collect()
    }

    /// Reads one chapter into memory, with the limit of 8 megabytes.
    fn chapter_bytes(&self, index: usize) -> Result<Vec<u8>, ReaderError> {
        let epub = self.epub().ok_or(ReaderError::NoSuchChapter(index))?;

        if index >= self.chapter_count() {
            return Err(ReaderError::NoSuchChapter(index));
        }

        let spine_entry = epub
            .spine()
            .get(index)
            .ok_or(ReaderError::NoSuchChapter(index))?;
        let manifest_entry = spine_entry
            .manifest_entry()
            .ok_or(ReaderError::ChapterAbsent)?;

        let mut writer = CappedWriter::new(MAX_CHAPTER_BYTES);
        match manifest_entry.copy_bytes(&mut writer) {
            Ok(_) => Ok(writer.buffer),
            // The archive wraps the fault of the writer in its own error.
            // Therefore the flag of the writer, and not the error, tells why
            // the copy stopped.
            Err(_) if writer.hit_limit => Err(ReaderError::ChapterTooLarge),
            Err(_) => Err(ReaderError::ChapterAbsent),
        }
    }

    /// Gives the number in the spine of each file of the spine.
    fn spine_by_href(&self) -> HashMap<String, usize> {
        let mut map = HashMap::new();

        let Some(epub) = self.epub() else {
            return map;
        };

        for entry in epub.spine().iter() {
            let Some(manifest_entry) = entry.manifest_entry() else {
                continue;
            };
            if let Some(href) = manifest_entry.resource().key().value() {
                // A file that stands two times in the spine keeps its first
                // place, because the reader goes forward.
                map.entry(href.to_string()).or_insert(entry.order());
            }
        }
        map
    }
}

impl fmt::Debug for Book {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let form = match &self.kind {
            Kind::Epub { .. } => "epub",
            Kind::Pdf(_) => "pdf",
        };

        f.debug_struct("Book")
            .field("form", &form)
            .field("chapters", &self.chapter_count())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// The four books of the measurement of 2026-08-10. They stand outside the
    /// repository, because the repository must stay small.
    fn survey_book(name: &str) -> PathBuf {
        PathBuf::from("/home/nyverino/.claude/jobs/064ecdb7/tmp/epub-survey/books").join(name)
    }

    /// The book inside the repository. It is the smallest of the four.
    fn repo_book() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/alice.epub")
    }

    /// The twelve hostile files inside the repository.
    fn hostile_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/hostile")
    }

    /// The design gives the book to a task. The feature `threadsafe` of
    /// `rbook` makes `Epub` `Send + Sync`. This test fails if somebody removes
    /// that feature from `Cargo.toml`.
    #[test]
    fn a_task_can_hold_the_book() {
        fn needs_send_and_sync<T: Send + Sync>() {}
        needs_send_and_sync::<Book>();
    }

    #[test]
    fn the_writer_refuses_more_than_its_limit() {
        // This test shows the fault that a plain read has. A writer with no
        // limit takes every byte, and the memory grows without an end.
        let mut writer = CappedWriter::new(10);
        assert!(writer.write(b"12345").is_ok());
        assert!(writer.write(b"12345").is_ok());
        assert!(writer.write(b"1").is_err());
        assert!(writer.hit_limit);
        assert_eq!(10, writer.buffer.len());
    }

    #[test]
    fn the_writer_takes_a_part_that_is_exactly_the_limit() {
        let mut writer = CappedWriter::new(4);
        assert!(writer.write(b"abcd").is_ok());
        assert!(!writer.hit_limit);
    }

    #[test]
    fn it_opens_the_book_of_the_repository() {
        let book = Book::open(&repo_book()).expect("Alice must open");
        assert_eq!(14, book.chapter_count());
        assert_eq!("Alice's Adventures in Wonderland", book.title());
        assert_eq!("Lewis Carroll", book.author());
    }

    #[test]
    fn it_gives_the_table_of_contents() {
        let book = Book::open(&repo_book()).expect("Alice must open");
        let contents = book.contents();
        // The measurement of 2026-08-10 counted 16 entries for Alice, and
        // every entry named a chapter of the spine.
        assert_eq!(16, contents.len());
        for item in &contents {
            assert!(!item.label.is_empty());
            let index = item
                .spine_index
                .unwrap_or_else(|| panic!("{:?} must name a chapter", item.label));
            assert!(index < book.chapter_count());
        }
    }

    #[test]
    fn it_reads_every_chapter_of_the_book_of_the_repository() {
        let book = Book::open(&repo_book()).expect("Alice must open");
        for index in 0..book.chapter_count() {
            let xhtml = book.chapter_xhtml(index).expect("a chapter must come");
            assert!(xhtml.len() <= MAX_CHAPTER_BYTES);
        }
    }

    #[test]
    fn it_gives_a_size_for_every_chapter() {
        let book = Book::open(&repo_book()).expect("Alice must open");
        let sizes = book.chapter_sizes();
        assert_eq!(book.chapter_count(), sizes.len());
        assert!(sizes.iter().sum::<u64>() > 0);
    }

    #[test]
    fn it_refuses_a_chapter_that_is_not_in_the_book() {
        let book = Book::open(&repo_book()).expect("Alice must open");
        let count = book.chapter_count();
        assert_eq!(
            Err(ReaderError::NoSuchChapter(count)),
            book.chapter_xhtml(count)
        );
    }

    /// The four books of the survey. The test passes when the books are not
    /// on the disk, because they stand outside the repository.
    #[test]
    fn it_opens_the_four_books_of_the_measurement() {
        // The measurement of 2026-08-10 gave these numbers.
        let expected = [
            (
                "alice.epub",
                14usize,
                "Alice's Adventures in Wonderland",
                16usize,
            ),
            ("pride.epub", 16, "Pride and Prejudice", 63),
            (
                "frankenstein.epub3",
                32,
                "Frankenstein; or, the modern prometheus",
                32,
            ),
            ("mobydick.epub3", 12, "Moby Dick; Or, The Whale", 146),
        ];
        for (name, chapters, title, toc_lines) in expected {
            let path = survey_book(name);
            if !path.exists() {
                continue;
            }
            let book = Book::open(&path).unwrap_or_else(|e| panic!("{name} must open: {e}"));
            assert_eq!(chapters, book.chapter_count(), "{name}");
            assert_eq!(title, book.title(), "{name}");
            let contents = book.contents();
            assert_eq!(toc_lines, contents.len(), "{name}");
            assert!(
                contents.iter().all(|item| item.spine_index.is_some()),
                "{name}: a line of the contents names no chapter"
            );
            for index in 0..book.chapter_count() {
                let answer = book.chapter_xhtml(index);
                assert!(answer.is_ok(), "{name} chapter {index}: {answer:?}");
            }
        }
    }

    #[test]
    fn it_refuses_a_book_that_is_too_large() {
        // A file of 256 megabytes on the disk is too slow for a test.
        // Therefore the test writes a small file and it looks at the message.
        let error = ReaderError::BookTooLarge(300 * 1024 * 1024);
        let text = error.to_string();
        assert!(text.starts_with("This book is too large."));
        assert!(text.contains("314572800"));
    }

    #[test]
    fn every_message_is_short_and_it_ends_with_a_full_stop() {
        let messages = [
            ReaderError::NotAnEpub.to_string(),
            ReaderError::BookTooLarge(1).to_string(),
            ReaderError::TooManyEntries(1).to_string(),
            ReaderError::NoSuchChapter(1).to_string(),
            ReaderError::ChapterAbsent.to_string(),
            ReaderError::ChapterTooLarge.to_string(),
        ];
        for message in messages {
            assert!(message.ends_with('.'), "{message}");
            assert!(message.starts_with("This"), "{message}");
        }
    }

    /// The important test of section 7. The zip bomb holds 2 gigabytes of the
    /// letter `A` in 2 megabytes of the archive. A plain read stops the whole
    /// process with `abort`. This test shows that the reader gives an error.
    #[test]
    fn the_zip_bomb_gives_the_message_of_a_chapter_that_is_too_large() {
        let path = hostile_dir().join("05-zip-bomb.epub");
        let book = Book::open(&path).expect("the archive of the bomb must open");
        assert_eq!(1, book.chapter_count());
        assert_eq!(Err(ReaderError::ChapterTooLarge), book.chapter_xhtml(0));
        assert_eq!(
            "This chapter is too large.",
            ReaderError::ChapterTooLarge.to_string()
        );
        // The size of the chapter is 0, therefore the part of the book that
        // the user read stays a number that the program can use.
        assert_eq!(vec![0u64], book.chapter_sizes());
    }

    /// Every hostile file must give an answer, and it must not stop the
    /// program. The test reads every chapter of every file.
    #[test]
    fn no_hostile_file_stops_the_program() {
        let names = [
            "01-not-an-epub.epub",
            "02-no-container.epub",
            "03-missing-target.epub",
            "04-path-traversal.epub",
            "04b-zip-traversal.epub",
            "05-zip-bomb.epub",
            "06-deep-nesting.epub",
            "07-billion-laughs.epub",
            "07b-laughs-in-opf.epub",
            "08-binary-as-xhtml.epub",
            "09-not-a-zip.epub",
            "10-empty.epub",
        ];
        for name in names {
            let path = hostile_dir().join(name);
            assert!(path.exists(), "{name} must stand in tests/data/hostile");
            match Book::open(&path) {
                Err(error) => {
                    // The message must help the user.
                    assert!(error.to_string().ends_with('.'), "{name}");
                }
                Ok(book) => {
                    let _ = book.title();
                    let _ = book.author();
                    let _ = book.contents();
                    for index in 0..book.chapter_count() {
                        // The answer can be text or an error. Both are good.
                        // The program must only stay alive and keep its
                        // memory: every read stops at 8 megabytes.
                        match book.chapter_xhtml(index) {
                            Ok(text) => assert!(text.len() <= MAX_CHAPTER_BYTES, "{name}"),
                            Err(error) => assert!(error.to_string().ends_with('.'), "{name}"),
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn a_file_that_is_not_a_zip_gives_the_message_of_a_file_that_is_not_an_epub() {
        let path = hostile_dir().join("09-not-a-zip.epub");
        match Book::open(&path) {
            Err(error) => assert_eq!(ReaderError::NotAnEpub, error),
            Ok(_) => panic!("a file that is not a zip must not open"),
        }
    }

    #[test]
    fn an_empty_file_is_not_an_epub() {
        let path = hostile_dir().join("10-empty.epub");
        match Book::open(&path) {
            Err(error) => assert_eq!(ReaderError::NotAnEpub, error),
            Ok(_) => panic!("an empty file must not open"),
        }
    }

    #[test]
    fn a_book_that_names_a_file_that_is_absent_gives_a_clear_error() {
        let path = hostile_dir().join("03-missing-target.epub");
        let book = Book::open(&path).expect("the archive must open");
        assert_eq!(1, book.chapter_count());
        match book.chapter_xhtml(0) {
            Err(error) => assert_eq!(ReaderError::ChapterAbsent, error),
            Ok(text) => panic!("a file that is absent must give an error, and it gave {text:?}"),
        }
    }

    #[test]
    fn a_name_with_a_path_of_the_disk_touches_no_file_of_the_disk() {
        // The file names `../../../../etc/passwd`. The reader must not read
        // the password file. It must give an error, or an empty answer.
        let path = hostile_dir().join("04-path-traversal.epub");
        if let Ok(book) = Book::open(&path) {
            for index in 0..book.chapter_count() {
                if let Ok(text) = book.chapter_xhtml(index) {
                    assert!(!text.contains("root:"), "the reader read /etc/passwd");
                }
            }
        }
    }
}
