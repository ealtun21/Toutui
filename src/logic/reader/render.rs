//! XHTML to lines of the screen. See T-10, design section 5.2 step 2.
//!
//! `html2text` breaks the text into lines of a given width, and it gives a
//! `RichAnnotation` for each part. This module changes each annotation into a
//! `ratatui::style::Style`, therefore the text keeps its bold, its italic, and
//! its links.
//!
//! **This function must never run on the thread that draws.** All the
//! chapters of Moby Dick need 218 milliseconds in a debug build, measured on
//! 2026-08-10. The hostile page with 10000 nested `<div>` needs 895
//! milliseconds in a debug build, because the time of `html2text` grows with
//! the square of the depth of the tags. A page with 100000 nested `<div>`
//! needs more than 60 seconds. Therefore the caller runs this function in a
//! task, and it holds a limit of 5 seconds.

use html2text::render::{RichAnnotation, TaggedLineElement};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

/// The largest XHTML that this module parses, in bytes. 8 megabytes.
///
/// `book::chapter_xhtml` already holds the same limit. This second guard
/// stands here because the function is public and a caller can give it any
/// text.
pub const MAX_XHTML_BYTES: usize = 8 * 1024 * 1024;

/// The smallest width for the layout, in columns.
///
/// A width of 0 gives no place for a word. The layout then has no answer, and
/// it can fail. The reader takes 4 columns as the smallest width.
const MIN_WIDTH: u16 = 4;

/// The colour of code and of text that keeps its form.
///
/// A different colour, and not a modifier, because a terminal shows a colour
/// well and it shows some modifiers badly.
const CODE_COLOUR: Color = Color::Cyan;

/// Changes the XHTML of one chapter into lines with a style.
///
/// The answer is empty in three cases:
///
/// 1. The text is larger than [`MAX_XHTML_BYTES`].
/// 2. The parse fails.
/// 3. The chapter holds no text.
///
/// The third case is normal. The first item of the spine is often a wrapper of
/// the cover with no text: Moby Dick gives 553 bytes and no line. The caller
/// must go to the next chapter when the answer is empty, and it must not show
/// an empty page.
pub fn to_lines(xhtml: &str, width: u16) -> Vec<Line<'static>> {
    if xhtml.len() > MAX_XHTML_BYTES {
        return Vec::new();
    }
    let width = usize::from(width.max(MIN_WIDTH));

    let Ok(rich_lines) = html2text::config::rich().lines_from_read(xhtml.as_bytes(), width) else {
        return Vec::new();
    };

    rich_lines
        .iter()
        .map(|rich_line| {
            let spans: Vec<Span<'static>> = rich_line
                .iter()
                .filter_map(|element| match element {
                    TaggedLineElement::Str(text) => {
                        Some(Span::styled(text.s.clone(), style_of(&text.tag)))
                    }
                    // A fragment start carries no text. The reader drops it.
                    _ => None,
                })
                .collect();
            Line::from(spans)
        })
        .collect()
}

/// Gives the number of letters of each line.
///
/// The place of the user in the form of the web reader counts the letters. See
/// `cfi` and T-10. This function gives that count for the screen, and
/// `cfi::text_places` gives it for the tree of the XHTML.
pub fn letters_of_each_line(lines: &[Line<'static>]) -> Vec<usize> {
    lines
        .iter()
        .map(|line| {
            line.spans
                .iter()
                .map(|span| crate::logic::reader::cfi::letters(span.content.as_ref()))
                .sum()
        })
        .collect()
}

/// Gives the style of one part of a line.
///
/// A part can carry more than one annotation, for example bold inside a link.
/// The function therefore adds each annotation to the same style.
fn style_of(tags: &[RichAnnotation]) -> Style {
    let mut style = Style::default();
    for tag in tags {
        style = match tag {
            RichAnnotation::Emphasis => style.add_modifier(Modifier::ITALIC),
            RichAnnotation::Strong => style.add_modifier(Modifier::BOLD),
            RichAnnotation::Link(_) => style.add_modifier(Modifier::UNDERLINED),
            RichAnnotation::Strikeout => style.add_modifier(Modifier::CROSSED_OUT),
            RichAnnotation::Code | RichAnnotation::Preformat(_) => style.fg(CODE_COLOUR),
            // A picture inside the text shows its alternative text only. The
            // design says that T-10 shows no picture.
            RichAnnotation::Image(_) => style.add_modifier(Modifier::DIM),
            // The book can name a colour. The reader follows it, because a
            // book uses a colour with a meaning, for example for a note.
            RichAnnotation::Colour(colour) => style.fg(Color::Rgb(colour.r, colour.g, colour.b)),
            RichAnnotation::BgColour(colour) => style.bg(Color::Rgb(colour.r, colour.g, colour.b)),
            _ => style,
        };
    }
    style
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn repo_book() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/alice.epub")
    }

    /// Gives the whole text of the lines, with no style.
    fn text_of(lines: &[Line<'static>]) -> String {
        lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    #[test]
    fn a_page_with_no_text_gives_no_line() {
        // The wrapper of the cover of Moby Dick has this form. The caller must
        // then go to the next chapter.
        let lines = to_lines("<html><body></body></html>", 80);
        assert!(lines.is_empty());
    }

    #[test]
    fn it_gives_the_text_of_a_simple_page() {
        let lines = to_lines("<html><body><p>Hello reader</p></body></html>", 80);
        assert_eq!("Hello reader", text_of(&lines).trim());
    }

    #[test]
    fn strong_gives_bold_and_em_gives_italic() {
        let lines = to_lines(
            "<html><body><p><strong>bold</strong> and <em>italic</em></p></body></html>",
            80,
        );
        let spans: Vec<&Span<'static>> = lines.iter().flat_map(|line| line.spans.iter()).collect();
        let bold = spans
            .iter()
            .find(|span| span.content.contains("bold"))
            .expect("the word bold must stand in the answer");
        assert!(bold.style.add_modifier.contains(Modifier::BOLD));
        let italic = spans
            .iter()
            .find(|span| span.content.contains("italic"))
            .expect("the word italic must stand in the answer");
        assert!(italic.style.add_modifier.contains(Modifier::ITALIC));
    }

    #[test]
    fn a_link_gives_an_underline() {
        let lines = to_lines(
            "<html><body><p><a href=\"c2.xhtml\">next page</a></p></body></html>",
            80,
        );
        let underlined = lines
            .iter()
            .flat_map(|line| line.spans.iter())
            .any(|span| span.style.add_modifier.contains(Modifier::UNDERLINED));
        assert!(underlined);
    }

    #[test]
    fn code_gives_a_different_colour() {
        let lines = to_lines("<html><body><p><code>fn main</code></p></body></html>", 80);
        let coloured = lines
            .iter()
            .flat_map(|line| line.spans.iter())
            .any(|span| span.style.fg == Some(CODE_COLOUR));
        assert!(coloured);
    }

    #[test]
    fn a_narrow_width_gives_more_lines_than_a_wide_width() {
        let xhtml = "<html><body><p>".to_string()
            + &"one two three four five six seven eight nine ten ".repeat(20)
            + "</p></body></html>";
        let wide = to_lines(&xhtml, 100);
        let narrow = to_lines(&xhtml, 30);
        assert!(
            narrow.len() > wide.len(),
            "narrow {} wide {}",
            narrow.len(),
            wide.len()
        );
    }

    #[test]
    fn no_line_is_wider_than_the_width() {
        let xhtml = "<html><body><p>".to_string()
            + &"alpha bravo charlie delta echo foxtrot ".repeat(20)
            + "</p></body></html>";
        for line in to_lines(&xhtml, 40) {
            let width: usize = line
                .spans
                .iter()
                .map(|span| span.content.chars().count())
                .sum();
            assert!(width <= 40, "a line has {width} columns");
        }
    }

    #[test]
    fn a_width_of_zero_gives_no_panic() {
        // The screen can be 0 columns wide for one draw after a resize.
        let _ = to_lines("<html><body><p>hello</p></body></html>", 0);
    }

    #[test]
    fn it_refuses_an_xhtml_that_is_too_large() {
        // This is the second guard. `book::chapter_xhtml` holds the first one.
        let big = "a".repeat(MAX_XHTML_BYTES + 1);
        assert!(to_lines(&big, 80).is_empty());
    }

    #[test]
    fn a_page_with_no_meaning_gives_no_panic() {
        // The hostile file `08-binary-as-xhtml.epub` has this form.
        let rubbish: String = (0..=255u8)
            .map(|byte| byte as char)
            .collect::<String>()
            .repeat(100);
        let _ = to_lines(&rubbish, 80);
    }

    /// A page with 10000 nested `<div>` must give an answer and it must not
    /// wait for ever. The measurement of 2026-08-10 gave 1 line in 895
    /// milliseconds in a debug build. That time stays below the limit of 5
    /// seconds of the caller.
    #[test]
    fn a_page_with_10000_nested_tags_gives_an_answer() {
        use crate::logic::reader::book::Book;
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/hostile/06-deep-nesting.epub");
        let book = Book::open(&path).expect("the archive must open");
        let xhtml = book.chapter_xhtml(0).expect("the chapter must come");
        assert_eq!(110_105, xhtml.len());
        let start = std::time::Instant::now();
        let lines = to_lines(&xhtml, 80);
        assert_eq!(1, lines.len());
        // A wide window, because a machine of the CI is slower.
        assert!(
            start.elapsed() < std::time::Duration::from_secs(20),
            "the render took {:?}",
            start.elapsed()
        );
    }

    #[test]
    fn the_book_of_the_repository_gives_lines() {
        use crate::logic::reader::book::Book;
        let book = Book::open(&repo_book()).expect("Alice must open");
        let mut with_text = 0;
        for index in 0..book.chapter_count() {
            let xhtml = book.chapter_xhtml(index).expect("a chapter must come");
            if !to_lines(&xhtml, 80).is_empty() {
                with_text += 1;
            }
        }
        assert!(with_text > 10, "only {with_text} chapters gave text");
    }
}
