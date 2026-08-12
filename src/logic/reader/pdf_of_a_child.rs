//! A child process reads a PDF book. See T-62.
//!
//! `Document::load` of `lopdf` reads the **whole** file, and it makes one value
//! of every object of that file. A measurement of 2026-08-11 of a book of 150
//! pages of a scan gave 279 megabytes of memory for the program, and
//! `MAX_BOOK_BYTES` of 512 megabytes permits a file that needs a machine of a
//! gigabyte for one moment. **That memory belongs to the program that the user
//! reads**, and the user of a small machine then loses the player too.
//!
//! `lopdf` is pure Rust and it forbids no `unsafe` code of its dependencies.
//! A file that a person made to break a reader can therefore stop the process.
//! **A program of a user must not stop because of one book.**
//!
//! The answer of the maintainer of 2026-08-12: **the program spawns itself with
//! a hidden flag**, the child reads the book, and it writes the text and the
//! pictures beside the file of the book. The peak of the memory and every fault
//! of `lopdf` stay in that child:
//!
//! - The child that stops gives an exit code, and the parent gives the user a
//!   message and no dead screen (T-52).
//! - The child gives its memory back when it stops, and the program of the user
//!   holds the text and the small pictures only.
//! - **This needs no dependency**: `std::process` spawns the child, and this
//!   module writes the form of the file. The rule of T-20 stays, and `mupdf`
//!   stays outside.
//!
//! The parsed book stands beside the PDF, in the cache of the ebooks. A second
//! visit of the book therefore spawns no child at all.

use crate::logic::reader::book::ReaderError;
use crate::logic::reader::pdf::{Page, Pdf, Picture};
use log::{info, warn};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::OnceLock;

/// The flag that makes the program read one PDF and stop.
///
/// The flag holds two dashes and a name that no user writes by accident. It
/// stands in no message of `--help`: the program gives it to itself.
pub const THE_FLAG: &str = "--the-pdf-of-a-child";

/// The first bytes of the file of the parsed book.
///
/// A file of an older form of this program gives a different mark, therefore
/// the parent reads it as no book and it spawns the child again.
const THE_MARK: &[u8; 12] = b"TOUTUIPDF001";

/// The longest time that the parent waits for the child.
///
/// A book of 500 megabytes of a scan took 9 seconds in the measurement of T-62.
/// This value holds a machine that is much slower, and it stops a child that
/// never comes back.
const THE_LONGEST_WAIT: std::time::Duration = std::time::Duration::from_secs(300);

/// The time between two looks at the child.
const LOOK_AGAIN: std::time::Duration = std::time::Duration::from_millis(50);

/// Tells if the program that runs is the program of the user.
///
/// **`current_exe` gives the binary that runs, and that binary is the binary of
/// a test inside a test.** A run of `cargo nextest` on 2026-08-12 opened a PDF
/// of the sandbox and the child was the test binary: that process knows no flag
/// of this module, therefore it gave a fault and the reader said "This PDF gives
/// no page". The same holds for every program that takes this library.
///
/// `main` writes this value, therefore the child does the work for the user and
/// the reader of a test reads the book in its own process. See T-62.
fn the_program() -> &'static AtomicBool {
    static THE_PROGRAM: OnceLock<AtomicBool> = OnceLock::new();
    THE_PROGRAM.get_or_init(|| AtomicBool::new(false))
}

/// Says that the program of the user runs. `main` calls this one time.
pub fn the_program_of_the_user_runs() {
    the_program().store(true, Ordering::SeqCst);
}

/// Tells if a child can read a PDF.
pub fn a_child_can_read() -> bool {
    the_program().load(Ordering::SeqCst)
}

/// Gives the path of the parsed book of a PDF.
///
/// The file stands beside the PDF, therefore it stands in the cache of the
/// ebooks and the rules of that cache hold it. See T-67.
pub fn the_parsed_book_of(book: &Path) -> PathBuf {
    let mut name = book.as_os_str().to_os_string();
    name.push(".pages");
    PathBuf::from(name)
}

/// Writes a number of four bytes, in the sequence of the little end first.
fn write_a_number(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

/// Writes a run of bytes: the length, and then the bytes.
fn write_the_bytes(out: &mut Vec<u8>, value: &[u8]) {
    write_a_number(out, value.len() as u32);
    out.extend_from_slice(value);
}

/// Makes the bytes of the file of a parsed book.
///
/// The form is a mark, the title, the author, and then one group for each page.
/// Every number holds four bytes with the little end first, and every text and
/// every picture holds its length before its bytes. **A form of this kind needs
/// no dependency**, and the rule of T-20 stays.
pub fn the_bytes_of(pdf: &Pdf) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();

    out.extend_from_slice(THE_MARK);
    write_the_bytes(&mut out, pdf.title().as_bytes());
    write_the_bytes(&mut out, pdf.author().as_bytes());
    write_a_number(&mut out, pdf.page_count() as u32);

    for index in 0..pdf.page_count() {
        let Some(page) = pdf.page(index) else {
            continue;
        };

        write_a_number(&mut out, page.number);
        write_the_bytes(&mut out, page.text.as_bytes());
        write_a_number(&mut out, page.pictures.len() as u32);

        for picture in &page.pictures {
            write_the_bytes(&mut out, picture.name.as_bytes());
            write_a_number(&mut out, picture.width);
            write_a_number(&mut out, picture.height);
            write_the_bytes(&mut out, &picture.file);
        }
    }

    out
}

/// Reads bytes of the file, one field at a time.
struct Reader<'a> {
    bytes: &'a [u8],
    place: usize,
}

impl<'a> Reader<'a> {
    fn a_number(&mut self) -> Option<u32> {
        let end = self.place.checked_add(4)?;
        let four: [u8; 4] = self.bytes.get(self.place..end)?.try_into().ok()?;
        self.place = end;
        Some(u32::from_le_bytes(four))
    }

    fn the_bytes(&mut self) -> Option<&'a [u8]> {
        let length = self.a_number()? as usize;
        let end = self.place.checked_add(length)?;
        let value = self.bytes.get(self.place..end)?;
        self.place = end;
        Some(value)
    }

    fn a_text(&mut self) -> Option<String> {
        Some(String::from_utf8_lossy(self.the_bytes()?).into_owned())
    }
}

/// Makes a book of the bytes of a parsed book.
///
/// Gives `None` for a file that this program did not write, and for a file that
/// stops in the middle: a child that the machine stopped leaves such a file, and
/// the parent must then read the book again.
pub fn the_book_of(bytes: &[u8]) -> Option<Pdf> {
    if bytes.len() < THE_MARK.len() || &bytes[..THE_MARK.len()] != THE_MARK {
        return None;
    }

    let mut reader = Reader {
        bytes,
        place: THE_MARK.len(),
    };

    let title = reader.a_text()?;
    let author = reader.a_text()?;
    let count = reader.a_number()? as usize;

    let mut pages: Vec<Page> = Vec::with_capacity(count.min(5000));

    for _ in 0..count {
        let number = reader.a_number()?;
        let text = reader.a_text()?;
        let pictures_of_the_page = reader.a_number()? as usize;

        let mut pictures: Vec<Picture> = Vec::with_capacity(pictures_of_the_page.min(64));

        for _ in 0..pictures_of_the_page {
            let name = reader.a_text()?;
            let width = reader.a_number()?;
            let height = reader.a_number()?;
            let file = reader.the_bytes()?.to_vec();

            pictures.push(Picture {
                name,
                width,
                height,
                file: Arc::new(file),
            });
        }

        pages.push(Page {
            number,
            text,
            pictures,
        });
    }

    Some(Pdf::of_the_parts(pages, title, author))
}

/// Reads the line of command, and it does the work of the child.
///
/// Gives the code of the process for a line that holds the flag, and `None` for
/// every other line: `main` then goes on and it draws the screen of the user.
///
/// **The flag stands in no message of `--help`.** The program gives it to
/// itself, and a user who writes it by accident gets a message and the code 1.
pub fn the_child_of_the_line_of_command() -> Option<i32> {
    let words: Vec<String> = std::env::args().collect();

    if words.iter().all(|word| word != THE_FLAG) {
        return None;
    }

    Some(the_answer_of_the_line(&words))
}

/// Gives the code of the process for a line of command that holds the flag.
///
/// The function is pure over its argument, therefore a test needs no process.
pub fn the_answer_of_the_line(words: &[String]) -> i32 {
    let Some(place) = words.iter().position(|word| word == THE_FLAG) else {
        return 1;
    };

    let (Some(book), Some(to)) = (words.get(place + 1), words.get(place + 2)) else {
        eprintln!(
            "toutui: {} takes the path of a PDF and the path of the pages. \
             The program gives this flag to itself.",
            THE_FLAG
        );
        return 1;
    };

    the_work_of_the_child(Path::new(book), Path::new(to))
}

/// The work of the child: it reads the book and it writes the parsed book.
///
/// The function gives the code that the process must give the parent. `main`
/// calls it before every other line of the program: the child opens no
/// terminal, it makes no database, and it plays nothing.
pub fn the_work_of_the_child(book: &Path, to: &Path) -> i32 {
    let pdf = match Pdf::of_the_file(book) {
        Ok(pdf) => pdf,
        Err(error) => {
            eprintln!("toutui: this PDF gives no page: {}", error);
            return 2;
        }
    };

    // The file goes beside the book first, and then it takes its name. A child
    // that the machine stops therefore never leaves a file that is not
    // complete.
    let beside = to.with_extension("part");

    if let Err(error) = std::fs::write(&beside, the_bytes_of(&pdf)) {
        eprintln!("toutui: the program did not write the pages: {}", error);
        return 3;
    }

    if let Err(error) = std::fs::rename(&beside, to) {
        eprintln!("toutui: the pages did not take their name: {}", error);
        return 3;
    }

    0
}

/// Opens a PDF book, and a child process does the work. See T-62.
///
/// The function reads the parsed book of a visit before this one, when that
/// file exists. Therefore a second visit of a book spawns no child.
pub fn the_book_that_a_child_reads(book: &Path) -> Result<Pdf, ReaderError> {
    let parsed = the_parsed_book_of(book);

    if let Some(pdf) = read_the_parsed_book(&parsed) {
        info!(
            "[pdf] the pages of {} come from the cache",
            book.file_name().unwrap_or_default().to_string_lossy()
        );
        return Ok(pdf);
    }

    let program = std::env::current_exe().map_err(|error| {
        warn!("[pdf] the program does not know its own path: {}", error);
        ReaderError::ThePdfGivesNoPage
    })?;

    let started = std::time::Instant::now();

    let mut child = std::process::Command::new(program)
        .arg(THE_FLAG)
        .arg(book)
        .arg(&parsed)
        // The child writes no line on the screen of the user. A message of a
        // fault goes to the log of the parent below.
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|error| {
            warn!("[pdf] the child did not start: {}", error);
            ReaderError::ThePdfGivesNoPage
        })?;

    let code = wait_for_the_child(&mut child);

    let mut words = String::new();

    if let Some(out) = child.stderr.as_mut() {
        let _ = out.read_to_string(&mut words);
    }

    match code {
        Some(0) => {}
        Some(code) => {
            warn!(
                "[pdf] the child stopped with the code {}: {}",
                code,
                words.trim()
            );
            return Err(ReaderError::ThePdfGivesNoPage);
        }
        None => {
            warn!("[pdf] the child did not come back, and the program stopped it");
            return Err(ReaderError::ThePdfGivesNoPage);
        }
    }

    let Some(pdf) = read_the_parsed_book(&parsed) else {
        warn!("[pdf] the child wrote no page of this book");
        return Err(ReaderError::ThePdfGivesNoPage);
    };

    info!(
        "[pdf] a child read {} page(s) in {} ms",
        pdf.page_count(),
        started.elapsed().as_millis()
    );

    Ok(pdf)
}

/// Waits for the child, and it stops a child that never comes back.
///
/// Gives the code of the child, and `None` for a child that this function
/// stopped.
fn wait_for_the_child(child: &mut std::process::Child) -> Option<i32> {
    let started = std::time::Instant::now();

    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Some(status.code().unwrap_or(-1)),
            Ok(None) => {}
            Err(_) => return None,
        }

        if started.elapsed() > THE_LONGEST_WAIT {
            let _ = child.kill();
            let _ = child.wait();
            return None;
        }

        std::thread::sleep(LOOK_AGAIN);
    }
}

/// Reads the parsed book of a visit before this one.
fn read_the_parsed_book(parsed: &Path) -> Option<Pdf> {
    let bytes = std::fs::read(parsed).ok()?;

    match the_book_of(&bytes) {
        Some(pdf) => Some(pdf),
        None => {
            // A file of an older form of the program, or a file that is not
            // complete. The child writes it again.
            warn!("[pdf] the pages of the cache are not the pages of this program");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a_book() -> Pdf {
        Pdf::of_the_parts(
            vec![
                Page {
                    number: 1,
                    text: "The first page of the book.".to_string(),
                    pictures: vec![Picture {
                        name: "Im0".to_string(),
                        width: 640,
                        height: 480,
                        file: Arc::new(vec![1, 2, 3, 4, 5]),
                    }],
                },
                Page {
                    number: 2,
                    text: "Le deuxième — with a letter of more than one byte.".to_string(),
                    pictures: Vec::new(),
                },
            ],
            "A Book of the Test".to_string(),
            "Test Author".to_string(),
        )
    }

    /// The parent must read every value that the child wrote. See T-62.
    #[test]
    fn the_parent_reads_the_book_of_the_child() {
        let bytes = the_bytes_of(&a_book());
        let book = the_book_of(&bytes).expect("the bytes must give a book");

        assert_eq!(book.title(), "A Book of the Test");
        assert_eq!(book.author(), "Test Author");
        assert_eq!(book.page_count(), 2);

        let first = book.page(0).expect("the first page");
        assert_eq!(first.number, 1);
        assert_eq!(first.text, "The first page of the book.");
        assert_eq!(first.pictures.len(), 1);
        assert_eq!(first.pictures[0].name, "Im0");
        assert_eq!(first.pictures[0].width, 640);
        assert_eq!(first.pictures[0].height, 480);
        assert_eq!(first.pictures[0].file.as_slice(), &[1, 2, 3, 4, 5]);

        let second = book.page(1).expect("the second page");
        assert!(second.text.contains("deuxième"));
        assert!(second.pictures.is_empty());
    }

    /// **A file that is not complete must give no book.** A child that the
    /// machine stopped leaves such a file, and a reader that takes it gives the
    /// user a book of no page.
    #[test]
    fn a_file_that_stops_in_the_middle_gives_no_book() {
        let bytes = the_bytes_of(&a_book());

        for end in [0, 5, THE_MARK.len(), THE_MARK.len() + 6, bytes.len() - 1] {
            assert!(
                the_book_of(&bytes[..end]).is_none(),
                "the first {} bytes must give no book",
                end
            );
        }

        // A file of a different program gives no book.
        assert!(the_book_of(b"a text that is not a book of this program").is_none());
    }

    /// **A child reads a PDF for the program of the user only.** A run of
    /// `cargo nextest` of 2026-08-12 opened a PDF of the sandbox, and
    /// `current_exe` gave the binary of that test: the child knew no flag of
    /// this module, and the reader said "This PDF gives no page". See T-62.
    #[test]
    fn a_test_reads_the_book_in_its_own_process() {
        // `main` never runs inside a test, therefore no child may start.
        assert!(!a_child_can_read());
    }

    /// The parsed book stands beside the book, therefore the cache of the
    /// ebooks holds it. See T-67.
    #[test]
    fn the_parsed_book_stands_beside_the_book() {
        let path = the_parsed_book_of(Path::new("/tmp/books/a book.pdf"));
        assert_eq!(path, PathBuf::from("/tmp/books/a book.pdf.pages"));
    }

    /// A book of no page and a book of no picture both go through the form.
    #[test]
    fn a_book_of_no_page_goes_through_the_form() {
        let empty = Pdf::of_the_parts(Vec::new(), String::new(), String::new());
        let book = the_book_of(&the_bytes_of(&empty)).expect("a book of no page");

        assert_eq!(book.page_count(), 0);
        assert_eq!(book.title(), "");
    }
}
