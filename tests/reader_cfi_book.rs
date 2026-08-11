//! The count of the letters of a real book. See T-10 and
//! `src/logic/reader/cfi.rs`.
//!
//! The EPUBCFI of the web reader names a place in the tree of the XHTML, and
//! the screen of Toutui holds lines. The two forms have one common unit: the
//! letter. This test measures that the two counts agree.
//!
//! A difference of one letter moves the user by one letter, and not by a
//! paragraph. A difference that grows with each chapter would be a fault of the
//! walk over the tree, and this test finds such a fault.
//!
//! The measurement of 2026-08-11 gave a difference of 0 letters for all 74
//! chapters of the four books of the survey.

use std::path::PathBuf;
use toutui::logic::reader::book::Book;
use toutui::logic::reader::cfi;
use toutui::logic::reader::render::{letters_of_each_line, to_lines};

/// The book inside the repository.
fn repo_book() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/alice.epub")
}

/// The three other books of the survey. They stand outside the repository,
/// because the repository must stay small. Give their directory in
/// `TOUTUI_SURVEY_BOOKS`. A test that finds no directory reads the book of the
/// repository only, therefore continuous integration also passes.
fn survey_book(name: &str) -> Option<PathBuf> {
    let directory = std::env::var_os("TOUTUI_SURVEY_BOOKS")?;
    Some(PathBuf::from(directory).join(name))
}

/// The letters of the screen and the letters of the tree, for one chapter.
fn two_counts(xhtml: &str) -> (usize, usize) {
    let screen: usize = letters_of_each_line(&to_lines(xhtml, 80)).iter().sum();
    let tree: usize = cfi::text_places(xhtml)
        .iter()
        .map(|place| cfi::letters(&place.text))
        .sum();
    (screen, tree)
}

#[test]
fn the_two_counts_of_the_letters_agree_for_every_chapter() {
    let mut books = vec![repo_book()];
    for name in ["pride.epub", "frankenstein.epub3", "mobydick.epub3"] {
        if let Some(path) = survey_book(name) {
            if path.exists() {
                books.push(path);
            }
        }
    }

    let mut chapters = 0;
    for path in books {
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        let book = Book::open(&path).unwrap_or_else(|error| panic!("{name} must open: {error}"));
        for chapter in 0..book.chapter_count() {
            let Ok(xhtml) = book.chapter_xhtml(chapter) else {
                continue;
            };
            let (screen, tree) = two_counts(&xhtml);
            assert_eq!(screen, tree, "{name}, chapter {chapter}");
            chapters += 1;
        }
    }
    assert!(chapters >= 14, "the test read {chapters} chapters only");
}

/// The place goes out as an EPUBCFI and it comes back as the same line. This is
/// the promise of the feature: a user reads on the telephone, and the terminal
/// then opens the same paragraph.
#[test]
fn a_line_of_a_real_chapter_goes_out_and_it_comes_back() {
    let book = Book::open(&repo_book()).expect("Alice must open");
    // The chapter 2 is the first chapter with much text.
    let xhtml = book.chapter_xhtml(2).expect("the chapter must come");
    let lines = to_lines(&xhtml, 80);
    let of_each_line = letters_of_each_line(&lines);
    let places = cfi::text_places(&xhtml);
    assert!(lines.len() > 50, "the chapter gave {} lines", lines.len());

    for line in 0..lines.len() {
        // A line with no letter, for example the empty line between two
        // paragraphs, holds no place of its own. The reader then comes back to
        // the next line that holds a letter.
        if of_each_line[line] == 0 {
            continue;
        }
        let before = cfi::letters_before_line(&of_each_line, line);
        let text = cfi::to_epubcfi(2, &places, before).expect("the chapter must give a place");
        let read = cfi::parse_epubcfi(&text).expect("the text must be an EPUBCFI");
        assert_eq!(2, read.spine, "line {line}");
        let back = cfi::line_of_letters(&of_each_line, cfi::letters_before(&places, &read));
        assert_eq!(line, back, "line {line}, {text}");
    }
}

/// The value of the web reader must give a line of the chapter, and never a
/// panic.
#[test]
fn a_value_of_the_web_reader_gives_a_line_of_the_chapter() {
    let book = Book::open(&repo_book()).expect("Alice must open");
    let xhtml = book.chapter_xhtml(2).expect("the chapter must come");
    let lines = to_lines(&xhtml, 80);
    let of_each_line = letters_of_each_line(&lines);
    let places = cfi::text_places(&xhtml);

    // This is the form that the web reader of Audiobookshelf writes.
    let read = cfi::parse_epubcfi("epubcfi(/6/6[id2]!/4/2/8/2/1:120)")
        .expect("the text must be an EPUBCFI");
    assert_eq!(2, read.spine);
    let line = cfi::line_of_letters(&of_each_line, cfi::letters_before(&places, &read));
    assert!(line < lines.len(), "the line {line} is outside the chapter");
}

/// No hostile file may stop the program, and no hostile file may hold the
/// walk for a long time.
#[test]
fn no_hostile_file_stops_the_walk() {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/hostile");
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
    let start = std::time::Instant::now();
    for name in names {
        let path = directory.join(name);
        let Ok(book) = Book::open(&path) else {
            continue;
        };
        for chapter in 0..book.chapter_count() {
            let Ok(xhtml) = book.chapter_xhtml(chapter) else {
                continue;
            };
            let places = cfi::text_places(&xhtml);
            assert!(places.len() <= cfi::MAX_PLACES, "{name}");
            // A place of such a file must also come back with no panic.
            let _ = cfi::to_epubcfi(chapter, &places, 500);
        }
    }
    // A wide window, because a machine of the CI is slower.
    assert!(
        start.elapsed() < std::time::Duration::from_secs(30),
        "the walk took {:?}",
        start.elapsed()
    );
}
