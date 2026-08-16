//! The mark at the start of a line of a list. See T-44.
//!
//! A list of titles tells the user nothing about the media. The user asked:
//! "is this a new book, or a book that I started?" The mark answers that
//! question with three characters at the start of the line.
//!
//! Every function here is pure, therefore a test needs no screen.

/// The width of a mark, in columns. Every mark has the same width, therefore
/// the titles of a list stand under each other.
pub const WIDTH: usize = 4;

/// The mark of the media that plays now.
const PLAYS: &str = "▶";

/// The mark of a media that the user finished.
const FINISHED: &str = "✓";

/// The mark of a media at the end of its length that the user did not finish.
/// See T-290.
///
/// **The mark holds four columns and one of them is the space that stands
/// between the mark and the title** (T-44), therefore `100%` does not fit: it
/// takes the whole of the width, the title of that one line then stands one
/// column to the right of every other title, and the list looks broken. The
/// three numbers are the whole of the value, therefore the mark of that percent
/// is the number with no sign of the percent.
const THE_WHOLE_LENGTH: &str = "100";

/// Gives the mark of a media that the list "Continue Listening" holds.
///
/// `percent` and `finished` come from the server, in the form that
/// `collect_progress_percentage_book` and `collect_is_finished_book` give:
/// a number as a text, and "Finished" or "Not finished". A media that never
/// played gives "N/A".
///
/// **A percent of 100 is not the mark of the end** (T-290). The field
/// `isFinished` of the server is the one truth of a media that the user
/// finished, and the percent of that same row can stand at 100 while the field
/// says that the user did not finish it. The old code gave the mark `✓` to
/// every percent above 99, therefore the line said that the user finished a
/// media, and the panel of that same line said `Not finished` in that same
/// frame.
///
/// **The two values of one row of the server go apart** (the measurement of
/// 2026-08-16, of the real program v0.8.118 against the sandbox). The server
/// clamps `progress` at 1 and it never takes that value down: a `PATCH` of
/// `/api/me/progress/:item/:episode` that holds `progress` alone leaves
/// `isFinished` where it stood, and a later `PATCH` of `currentTime` alone
/// leaves `progress` at 1. **This program makes that row itself**:
/// `crate::api::me::update_media_progress::update_media_progress2_pod` and
/// `..._book` send the place of the end in one request and the mark of the end
/// in a second one, therefore a program that dies between the two of them, or a
/// second request that the server refused, leaves the row of the server at 100
/// percent and not finished for ever.
///
/// The frame of that measurement, of the episode `Chapter 01` of the podcast
/// `Arthur Gordon Pym`, whose row of the server held `progress: 1`,
/// `currentTime: 300`, and `isFinished: false`:
///
/// ```text
///   ▌ Continue Listening
/// ➤ ✓   Chapter 01
///   89% Chapter 02
/// [Arthur Gordon Pym] - Author: LibriVox - Episode: 1 - Duration: 22m
/// Progress: 100%, 17m left, Not finished
/// ```
///
/// **The shelf `Continue Listening` holds the media that the user did not
/// finish**, therefore the mark `✓` of that shelf says two things that do not
/// agree, and the panel of the same line says the third one.
pub fn of_progress(percent: &str, finished: &str, plays_now: bool) -> String {
    if plays_now {
        return fill(PLAYS);
    }

    if finished.trim() == "Finished" {
        return fill(FINISHED);
    }

    match percent.trim().parse::<i64>() {
        // A book at 0 percent is a book that the user did not start.
        Ok(0) => fill(""),
        // **The percent says the place and it does not say the end** (T-290).
        Ok(value) if value >= 100 => fill(THE_WHOLE_LENGTH),
        Ok(value) if value > 0 => fill(&format!("{}%", value)),
        // A percent of less than zero, and a percent that is no number, are a
        // percent that the program does not have.
        Ok(_) | Err(_) => fill(""),
    }
}

/// Gives the mark of a line that holds more than one media.
///
/// A line of a series has no mark of a position, because a series holds more
/// than one book, therefore the mark tells the media that plays only.
///
/// **A book of the Library view takes `of_progress`** (T-242): the box of
/// `crate::logic::the_positions` holds the place of every media of the account,
/// therefore that line says the percent of the user with no request at all.
pub fn of_library(plays_now: bool) -> String {
    if plays_now {
        fill(PLAYS)
    } else {
        fill("")
    }
}

/// Puts a mark in a space of the width of every mark.
fn fill(mark: &str) -> String {
    let width = mark.chars().count();
    let space = WIDTH.saturating_sub(width + 1);

    format!("{}{} ", mark, " ".repeat(space))
}

/// Gives the line that names a shelf of the Home view. See T-24.
///
/// The name stands at the first column, and a media of that shelf stands at
/// the column `WIDTH`. Therefore the user sees which media belong to which
/// shelf. The user cannot select this line.
pub fn shelf(label: &str) -> String {
    format!("▌ {}", label)
}

/// Puts the mark before the title.
pub fn line(mark: &str, title: &str) -> String {
    format!("{}{}", mark, title)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every line must start at the same column, or the list looks broken.
    fn width_of(text: &str) -> usize {
        text.chars().count()
    }

    #[test]
    fn every_mark_has_the_same_width() {
        let marks = [
            of_progress("50", "Not finished", false),
            of_progress("5", "Not finished", false),
            of_progress("100", "Finished", false),
            of_progress("100", "Not finished", false),
            of_progress("101", "Not finished", false),
            of_progress("-1", "Not finished", false),
            of_progress(" N/A", " N/A", false),
            of_progress("0", "Not finished", false),
            of_progress("50", "Not finished", true),
            of_library(true),
            of_library(false),
        ];

        for mark in &marks {
            assert_eq!(
                width_of(mark),
                WIDTH,
                "the mark {:?} is not {} columns wide",
                mark,
                WIDTH
            );
        }
    }

    #[test]
    fn the_media_that_plays_has_its_own_mark() {
        assert!(of_progress("50", "Not finished", true).contains(PLAYS));
        assert!(of_library(true).contains(PLAYS));
    }

    #[test]
    fn a_media_that_the_user_finished_has_a_mark() {
        assert!(of_progress("100", "Finished", false).contains(FINISHED));
        assert!(of_progress("42", "Finished", false).contains(FINISHED));
    }

    #[test]
    fn a_media_that_the_user_started_shows_the_part() {
        assert!(of_progress("47", "Not finished", false).contains("47%"));
        assert!(of_progress("5", "Not finished", false).contains("5%"));
    }

    /// **A percent of 100 is not the mark of the end** (T-290). The field
    /// `isFinished` of the server is the one truth of a media that the user
    /// finished, and the row of the server holds a percent of 100 beside
    /// `Not finished` after the program sent the place of the end and no mark
    /// of it.
    #[test]
    fn a_percent_of_the_whole_length_is_not_the_mark_of_the_end() {
        assert!(!of_progress("100", "Not finished", false).contains(FINISHED));
        assert!(!of_progress("101", "Not finished", false).contains(FINISHED));
        assert_eq!(
            of_progress("100", "Not finished", false).trim(),
            THE_WHOLE_LENGTH
        );
        assert_eq!(
            of_progress("101", "Not finished", false).trim(),
            THE_WHOLE_LENGTH
        );
    }

    /// The mark of a media at the end of its length must not read as the mark
    /// of a media of another percent, therefore it says the three numbers of
    /// the value.
    #[test]
    fn the_mark_of_the_whole_length_says_the_number_of_the_percent() {
        assert!(of_progress("100", "Not finished", false).starts_with("100"));
        assert!(of_progress("99", "Not finished", false).starts_with("99%"));
    }

    #[test]
    fn a_media_that_the_user_did_not_start_has_no_mark() {
        assert_eq!(of_progress("0", "Not finished", false).trim(), "");
        assert_eq!(of_progress(" N/A", " N/A", false).trim(), "");
        assert_eq!(of_progress("-1", "Not finished", false).trim(), "");
        assert_eq!(of_library(false).trim(), "");
    }

    /// The name of a shelf must not stand at the column of a media, or the
    /// user cannot tell a title from the name of a shelf.
    #[test]
    fn the_name_of_a_shelf_stands_before_the_column_of_a_media() {
        let name = shelf("Recently Added");
        assert!(name.contains("Recently Added"));
        assert!(!name.starts_with(' '));
        assert!(line(&of_library(false), "A Book").starts_with("   "));
    }

    #[test]
    fn the_mark_stands_before_the_title() {
        let text = line(&of_progress("47", "Not finished", false), "A Book");
        assert!(text.ends_with("A Book"));
        assert!(text.starts_with("47%"));
    }
}
