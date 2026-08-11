//! The position in the book. See T-10, design section 6.
//!
//! The reader holds two numbers: the chapter in the spine, and the first line
//! that the screen shows. The server keeps two fields:
//!
//! - `ebookLocation`, a text. A measurement on 2026-08-10 wrote `toutui:3:120`
//!   and read the same text back, therefore the server changes nothing in it.
//! - `ebookProgress`, a number from 0 to 1. Every client understands it.
//!
//! **The reader writes an EPUBCFI in `ebookLocation` from the version 0.7.8.**
//! The module `cfi` makes that text and it reads that text. The form
//! `toutui:<spine>:<line>` of this module stays for two reasons: a server holds
//! such a text from an older version, and a chapter that gives no text gives no
//! EPUBCFI.
//!
//! Every function here is pure. A test needs no file and no server.

/// The place where the user reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Position {
    /// The number of the chapter in the spine. The first chapter is 0.
    pub spine: usize,
    /// The first line of the chapter that the screen shows. The first line
    /// is 0.
    pub line: usize,
}

/// The start of every location that this application writes.
const PREFIX: &str = "toutui:";

/// Makes the text for `ebookLocation`.
pub fn to_ebook_location(p: Position) -> String {
    format!("{PREFIX}{}:{}", p.spine, p.line)
}

/// Reads the text of `ebookLocation`.
///
/// The function gives a position for a text of this application only. Every
/// other text, and an EPUBCFI of the web reader, gives `None`. The function
/// never fails and it never stops the program.
pub fn from_ebook_location(text: &str) -> Option<Position> {
    let rest = text.trim().strip_prefix(PREFIX)?;
    let (spine, line) = rest.split_once(':')?;
    // `parse` refuses a sign, a space, and a number that is too large.
    // Therefore `toutui:-1:0` gives `None` and not a number that goes round.
    Some(Position {
        spine: spine.parse().ok()?,
        line: line.parse().ok()?,
    })
}

/// The part of the book that the user read, from 0.0 to 1.0.
///
/// The value comes from the size of the chapters: the bytes of the chapters
/// before, plus the part of this chapter, divided by the bytes of all
/// chapters. That value is near the value of the web reader, and it is not the
/// same value. The two bars then agree well enough.
///
/// `lines_in_chapter` is the number of lines of the chapter that the user
/// reads now. A chapter with no line gives the start of that chapter.
pub fn fraction(sizes: &[u64], p: Position, lines_in_chapter: usize) -> f64 {
    let total: u64 = sizes.iter().sum();
    if sizes.is_empty() || total == 0 {
        return 0.0;
    }
    // A chapter after the end of the book means that the user read all of it.
    if p.spine >= sizes.len() {
        return 1.0;
    }
    let before: u64 = sizes[..p.spine].iter().sum();
    let inside = if lines_in_chapter == 0 {
        0.0
    } else {
        // The line can stand after the last line, for example after a change
        // of the width. The value then stays at the end of the chapter.
        let line = p.line.min(lines_in_chapter) as f64;
        line / lines_in_chapter as f64
    };
    let read = before as f64 + inside * sizes[p.spine] as f64;
    (read / total as f64).clamp(0.0, 1.0)
}

/// The chapter that holds a given part of the book.
///
/// The application uses this function when `ebookLocation` came from a
/// different client, for example an EPUBCFI of the web reader. The user then
/// loses the line, and not the chapter.
pub fn from_fraction(sizes: &[u64], fraction: f64) -> Position {
    let start = Position { spine: 0, line: 0 };
    let total: u64 = sizes.iter().sum();
    if sizes.is_empty() || total == 0 {
        return start;
    }
    // A value that is not a number, and a value outside 0 to 1, must give a
    // chapter of the book and not a panic. `clamp` refuses a NaN, therefore
    // the function looks at the NaN first.
    if fraction.is_nan() {
        return start;
    }
    let target = fraction.clamp(0.0, 1.0) * total as f64;

    let mut read = 0u64;
    for (spine, size) in sizes.iter().enumerate() {
        // `read + size` cannot go round: the sum of all sizes is `total`.
        let end = read + size;
        if target < end as f64 {
            return Position { spine, line: 0 };
        }
        read = end;
    }
    // The value is 1.0, or almost 1.0. The last chapter holds it.
    Position {
        spine: sizes.len() - 1,
        line: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_text_goes_out_and_it_comes_back() {
        for p in [
            Position { spine: 0, line: 0 },
            Position {
                spine: 3,
                line: 120,
            },
            Position {
                spine: 999,
                line: 1_000_000,
            },
        ] {
            let text = to_ebook_location(p);
            assert_eq!(Some(p), from_ebook_location(&text));
        }
    }

    #[test]
    fn the_text_has_the_form_of_the_design() {
        assert_eq!(
            "toutui:3:120",
            to_ebook_location(Position {
                spine: 3,
                line: 120
            })
        );
    }

    #[test]
    fn an_epubcfi_of_the_web_reader_gives_no_position() {
        // This is the value that the web reader of Audiobookshelf writes.
        let cfi = "epubcfi(/6/14[id6]!/4/2/2/2[c1]/2/1:0)";
        assert_eq!(None, from_ebook_location(cfi));
    }

    #[test]
    fn junk_text_gives_no_position_and_no_panic() {
        let junk = [
            "",
            " ",
            "toutui:",
            "toutui:3",
            "toutui::",
            "toutui:a:b",
            "toutui:-1:0",
            "toutui:0:-1",
            "toutui:3:120:9",
            "toutui:99999999999999999999999999:0",
            "TOUTUI:3:120",
            "xtoutui:3:120",
            "3:120",
            "epubcfi(/6/4!/4/1:0)",
            "\u{0}\u{1}\u{2}",
            "toutui:3:120\u{0}",
        ];
        for text in junk {
            assert_eq!(None, from_ebook_location(text), "{text:?}");
        }
    }

    /// The sizes of the four chapters of a small book, in bytes.
    const SIZES: [u64; 4] = [100, 300, 400, 200];

    #[test]
    fn the_part_of_the_book_is_zero_at_the_start() {
        let p = Position { spine: 0, line: 0 };
        assert_eq!(0.0, fraction(&SIZES, p, 50));
    }

    #[test]
    fn the_part_of_the_book_is_one_at_the_end() {
        let p = Position { spine: 3, line: 50 };
        assert_eq!(1.0, fraction(&SIZES, p, 50));
    }

    #[test]
    fn the_part_of_the_book_counts_the_chapters_before() {
        // The chapters before hold 400 bytes of 1000, and the user stands in
        // the middle of a chapter of 400 bytes: 400 + 200 of 1000.
        let p = Position { spine: 2, line: 10 };
        assert!((fraction(&SIZES, p, 20) - 0.6).abs() < 1e-9);
    }

    #[test]
    fn a_chapter_with_no_line_gives_the_start_of_that_chapter() {
        let p = Position { spine: 1, line: 7 };
        assert!((fraction(&SIZES, p, 0) - 0.1).abs() < 1e-9);
    }

    #[test]
    fn a_line_after_the_end_of_the_chapter_gives_the_end_of_the_chapter() {
        let p = Position {
            spine: 0,
            line: 5000,
        };
        assert!((fraction(&SIZES, p, 10) - 0.1).abs() < 1e-9);
    }

    #[test]
    fn a_chapter_after_the_end_of_the_book_gives_one() {
        let p = Position {
            spine: 400,
            line: 0,
        };
        assert_eq!(1.0, fraction(&SIZES, p, 10));
    }

    #[test]
    fn an_empty_book_gives_zero() {
        let p = Position { spine: 0, line: 0 };
        assert_eq!(0.0, fraction(&[], p, 10));
        assert_eq!(0.0, fraction(&[0, 0], p, 10));
        assert_eq!(Position::default(), from_fraction(&[], 0.5));
        assert_eq!(Position::default(), from_fraction(&[0, 0], 0.5));
    }

    #[test]
    fn a_part_of_the_book_gives_the_chapter_that_holds_it() {
        assert_eq!(0, from_fraction(&SIZES, 0.0).spine);
        assert_eq!(0, from_fraction(&SIZES, 0.05).spine);
        assert_eq!(1, from_fraction(&SIZES, 0.15).spine);
        assert_eq!(2, from_fraction(&SIZES, 0.5).spine);
        assert_eq!(3, from_fraction(&SIZES, 0.9).spine);
        assert_eq!(3, from_fraction(&SIZES, 1.0).spine);
        // Every answer starts at the first line of the chapter.
        assert_eq!(0, from_fraction(&SIZES, 0.5).line);
    }

    #[test]
    fn a_part_outside_zero_to_one_gives_a_chapter_of_the_book() {
        assert_eq!(0, from_fraction(&SIZES, -3.0).spine);
        assert_eq!(0, from_fraction(&SIZES, f64::NEG_INFINITY).spine);
        assert_eq!(3, from_fraction(&SIZES, 7.5).spine);
        assert_eq!(3, from_fraction(&SIZES, f64::INFINITY).spine);
        assert_eq!(0, from_fraction(&SIZES, f64::NAN).spine);
    }

    #[test]
    fn the_two_functions_agree() {
        // The part of the book of the start of a chapter must give that same
        // chapter back.
        for spine in 0..SIZES.len() {
            let p = Position { spine, line: 0 };
            let part = fraction(&SIZES, p, 100);
            assert_eq!(spine, from_fraction(&SIZES, part).spine, "chapter {spine}");
        }
    }

    #[test]
    fn a_chapter_with_no_bytes_takes_no_place() {
        // A chapter of the cover gives 0 lines and it can give 0 bytes.
        let sizes = [0u64, 100, 0, 100];
        assert_eq!(1, from_fraction(&sizes, 0.0).spine);
        assert_eq!(3, from_fraction(&sizes, 1.0).spine);
        let p = Position { spine: 0, line: 0 };
        assert_eq!(0.0, fraction(&sizes, p, 10));
    }
}
