//! The list of the chapters of the media that plays. See T-24.
//!
//! `POST /api/items/:id/play` gives `chapters` with `start`, `end`, and
//! `title`, and the engine holds them already: the keys `P` and `U` use them.
//! The user could not see the list, and they could not go to a chapter by its
//! name.
//!
//! The functions here are pure, therefore a test needs no engine and no
//! screen.

use crate::player::engine::track::Chapter;
use crate::utils::convert_seconds::convert_seconds;

/// Gives the number of the chapter that holds a position.
///
/// A book with no chapter gives nothing. A position after the last chapter
/// gives the last chapter, because the end of the last chapter can stand
/// before the end of the audio.
pub fn chapter_at(chapters: &[Chapter], position: f64) -> Option<usize> {
    if chapters.is_empty() {
        return None;
    }

    let found = chapters
        .iter()
        .position(|chapter| position >= chapter.start && position < chapter.end);

    match found {
        Some(index) => Some(index),
        // A position before the first chapter gives the first chapter.
        None if position < chapters[0].start => Some(0),
        None => Some(chapters.len() - 1),
    }
}

/// Makes the text of each line of the list of the chapters.
///
/// A mark stands before the chapter that plays. The number of the chapter
/// stands at the start, and the time of its start stands at the end.
pub fn lines(chapters: &[Chapter], position: f64) -> Vec<String> {
    let now = chapter_at(chapters, position);

    // `convert_seconds` writes a time for a person. It takes a list, thus one
    // call gives the time of every chapter.
    let times = convert_seconds(chapters.iter().map(|chapter| chapter.start).collect());

    chapters
        .iter()
        .enumerate()
        .map(|(index, chapter)| {
            let mark = if Some(index) == now { "▶ " } else { "  " };

            format!(
                "{}{}. {}  ({})",
                mark,
                index + 1,
                chapter.title,
                times.get(index).cloned().unwrap_or_default()
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chapter(start: f64, end: f64, title: &str) -> Chapter {
        Chapter {
            start,
            end,
            title: title.to_string(),
        }
    }

    fn three() -> Vec<Chapter> {
        vec![
            chapter(0.0, 25.0, "One"),
            chapter(25.0, 45.0, "Two"),
            chapter(45.0, 60.0, "Three"),
        ]
    }

    #[test]
    fn the_position_gives_the_chapter() {
        let all = three();

        assert_eq!(chapter_at(&all, 0.0), Some(0));
        assert_eq!(chapter_at(&all, 24.9), Some(0));
        assert_eq!(chapter_at(&all, 25.0), Some(1));
        assert_eq!(chapter_at(&all, 50.0), Some(2));
    }

    /// The end of the last chapter can stand before the end of the audio. A
    /// position after it must give the last chapter, and not nothing.
    #[test]
    fn a_position_after_the_last_chapter_gives_the_last_chapter() {
        assert_eq!(chapter_at(&three(), 900.0), Some(2));
    }

    #[test]
    fn a_position_before_the_first_chapter_gives_the_first_chapter() {
        let all = vec![chapter(10.0, 20.0, "One")];
        assert_eq!(chapter_at(&all, 0.0), Some(0));
    }

    #[test]
    fn a_book_with_no_chapter_gives_nothing() {
        assert_eq!(chapter_at(&[], 12.0), None);
        assert!(lines(&[], 12.0).is_empty());
    }

    #[test]
    fn every_chapter_gives_one_line() {
        let text = lines(&three(), 30.0);

        assert_eq!(text.len(), 3);
        assert!(text[0].contains("1. One"));
        assert!(text[1].contains("2. Two"));
        assert!(text[2].contains("3. Three"));
    }

    /// The user must see which chapter plays.
    #[test]
    fn the_chapter_that_plays_has_a_mark() {
        let text = lines(&three(), 30.0);

        assert!(text[1].starts_with('▶'));
        assert!(!text[0].starts_with('▶'));
        assert!(!text[2].starts_with('▶'));
    }

    /// Every line must start at the same column, or the list looks broken.
    #[test]
    fn every_line_starts_at_the_same_column() {
        // The mark of the chapter that plays is one character, and a space
        // takes its place in the other lines. Therefore the column is a
        // number of characters, and not a number of bytes.
        let column = |line: &str| {
            line.chars()
                .position(|one| one.is_ascii_digit())
                .expect("the number must stand in the line")
        };

        let text = lines(&three(), 30.0);
        let first = column(&text[0]);

        for line in &text {
            assert_eq!(column(line), first, "the line {:?} is not in line", line);
        }
    }
}
