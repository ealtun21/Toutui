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
use crate::utils::convert_seconds::clock;

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
                clock(chapter.start)
            )
        })
        .collect()
}

/// The mark of a boundary of a chapter, in the bar of the whole book.
pub const THE_MARK_OF_A_BOUNDARY: char = '│';

/// The smallest bar of this view. A bar of fewer cells says nothing of a place
/// at all, therefore the view then holds no bar and the list takes every row.
pub const THE_SMALLEST_BAR: u16 = 8;

/// The width under which the bar of the book holds no mark of a boundary.
///
/// **The marks then stand beside each other with no space at all** (the note of
/// `docs/mockups/mockup-7.md`), and a bar of marks alone says less than a bar of
/// no mark. **The rule reads the width of the bar and not the width of the
/// screen**: the bar is the thing that the marks make unreadable, and the
/// columns of the name and of the percent belong to no bar.
pub const THE_WIDTH_OF_THE_MARKS: u16 = 40;

/// The smallest number of cells of a chapter of the bar of the book, with the
/// mark of its boundary.
///
/// A bar whose chapters hold fewer cells than this holds no mark at all.
const THE_CELLS_OF_A_CHAPTER: usize = 3;

/// The columns of the percent of a bar: four for the number, and one for `%`.
const THE_COLUMNS_OF_THE_PERCENT: u16 = 5;

/// The columns of the percent at the right of a bar of this view.
pub fn the_columns_of_the_percent() -> u16 {
    THE_COLUMNS_OF_THE_PERCENT
}

/// A number of seconds of the engine, as a whole number of seconds.
///
/// A value that is not finite, and a value under zero, give zero: the bars of
/// this view say no place that the program does not have (T-91).
fn the_seconds(of_the_engine: f64) -> u32 {
    if !of_the_engine.is_finite() || of_the_engine <= 0.0 {
        return 0;
    }

    of_the_engine as u32
}

/// One bar of the two bars of the view of the chapters.
///
/// `the_cells_that_played` is the number of cells at the start of `the_cells`
/// that the playback passed: the render paints those cells with the accent of
/// the program, as the band of the player does (T-322).
///
/// `the_percent` is `None` for a bar of a chapter that the media does not have:
/// a media of no chapter keeps the row of the second bar, and it says no number
/// at all (T-91).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ABarOfTheView {
    /// The name at the left of the bar: `Book`, or `Ch 12`.
    pub the_name: String,
    /// The cells of the bar.
    pub the_cells: String,
    /// The number of cells at the start that the playback passed.
    pub the_cells_that_played: usize,
    /// The percent of the part that played.
    pub the_percent: Option<u32>,
}

/// The cells of the bar of the whole book, with a mark at each boundary of a
/// chapter.
///
/// **Two boundaries of the same column take one mark** (the note of
/// `docs/mockups/mockup-7.md`): a book of 70 chapters in 52 columns holds more
/// boundaries than columns, and a mark for each chapter would be a bar of marks
/// alone. The write of a mark in a cell that holds a mark already changes
/// nothing.
///
/// **The start of the first chapter is no boundary**: it stands at the column 0,
/// which is the start of the bar itself.
///
/// A media of no length gives a bar of cells that stay, as the band of the
/// player does (T-180).
pub fn the_bar_of_the_book(
    width: u16,
    position: f64,
    duration: f64,
    chapters: &[Chapter],
) -> String {
    let length = the_seconds(duration);
    let mut cells: Vec<char> =
        crate::ui::the_band_of_the_player::a_bar_of_a_part(width, the_seconds(position), length)
            .chars()
            .collect();

    if length == 0 || width < THE_WIDTH_OF_THE_MARKS {
        return cells.into_iter().collect();
    }

    let mut columns: Vec<usize> = chapters
        .iter()
        .skip(1)
        .map(|chapter| {
            (u64::from(width) * u64::from(the_seconds(chapter.start)) / u64::from(length)) as usize
        })
        // The column 0 is the start of the bar itself, and no boundary.
        .filter(|column| *column > 0 && *column < cells.len())
        .collect();

    columns.dedup();

    // **The marks go away while a chapter of the bar holds fewer than two cells
    // of its own** (the note of `docs/mockups/mockup-7.md`): the marks then
    // stand beside each other with no space at all, and the cells of the bar and
    // the marks of the boundaries are one noise. The measurement of T-330.5: a
    // bar of 150 cells of the book of 70 chapters of the sandbox gave
    // `█││█││█│█│█│██│█│██│██│█│░││░│░│░`.
    //
    // **The rule of the note reads the width of the bar alone**
    // (`THE_WIDTH_OF_THE_MARKS`), and a wide bar of many chapters passes it:
    // this rule reads the reason that the note gives.
    if columns
        .windows(2)
        .any(|two| two[1] - two[0] < THE_CELLS_OF_A_CHAPTER)
    {
        return cells.into_iter().collect();
    }

    for column in columns {
        cells[column] = THE_MARK_OF_A_BOUNDARY;
    }

    cells.into_iter().collect()
}

/// The place of the user inside the chapter of the cursor, and the length of
/// that chapter, in seconds.
///
/// **The bar of the chapter is the chapter of the cursor and not the chapter
/// that plays** (the note of `docs/mockups/mockup-7.md`): the user reads the
/// list of a book, and they move the cursor along it.
///
/// **The place goes inside the chapter of the cursor**: a chapter that stands
/// before the place of the user is whole, and a chapter after it holds nothing.
/// A chapter of no length, and a cursor that names no chapter, give `None`.
pub fn the_place_in_the_chapter_of_the_cursor(
    chapters: &[Chapter],
    the_cursor: Option<usize>,
    position: f64,
) -> Option<(u32, u32)> {
    let chapter = chapters.get(the_cursor?)?;
    let length = chapter.end - chapter.start;

    if length <= 0.0 {
        return None;
    }

    let done = (position - chapter.start).clamp(0.0, length);

    Some((the_seconds(done), the_seconds(length)))
}

/// The two bars of the view of the chapters, for a row of this width.
///
/// It gives `None` for a media of no length, and for a row that holds no bar of
/// [`THE_SMALLEST_BAR`] cells: the view then holds no bar at all, and the list
/// takes every row of it.
///
/// **A playback that stopped gives no bar at all**: the engine keeps the length
/// and the place of the media that played last, and the two bars of that media
/// stood over the words `No media plays now.` in the measurement of T-330.5. A
/// view must say no state that the program does not have (T-91).
///
/// The cursor is the line of the list. A view with no line takes the chapter
/// that plays, because the user then reads the place of the media and no place
/// of their own.
pub fn the_bars_of_the_view(
    width: u16,
    chapters: &[Chapter],
    position: f64,
    duration: f64,
    the_cursor: Option<usize>,
    the_playback_stopped: bool,
) -> Option<[ABarOfTheView; 2]> {
    if the_playback_stopped || the_seconds(duration) == 0 {
        return None;
    }

    let of_the_cursor = the_cursor.or_else(|| chapter_at(chapters, position));
    let the_columns_of_the_name = the_columns_of_the_name(chapters.len());
    let of_the_bar = width
        .checked_sub(the_columns_of_the_name + THE_COLUMNS_OF_THE_PERCENT)
        .filter(|cells| *cells >= THE_SMALLEST_BAR)?;

    let of_the_book = ABarOfTheView {
        the_name: "Book".to_string(),
        the_cells: the_bar_of_the_book(of_the_bar, position, duration, chapters),
        the_cells_that_played: crate::ui::the_band_of_the_player::the_cells_that_played(
            of_the_bar,
            the_seconds(position),
            the_seconds(duration),
        ),
        the_percent: Some(crate::ui::the_band_of_the_player::the_percent_of_a_part(
            the_seconds(position),
            the_seconds(duration),
        )),
    };

    let the_place = the_place_in_the_chapter_of_the_cursor(chapters, of_the_cursor, position);

    let of_the_chapter = match (of_the_cursor, the_place) {
        (Some(number), Some((done, whole))) => ABarOfTheView {
            the_name: format!("Ch {}", number + 1),
            the_cells: crate::ui::the_band_of_the_player::a_bar_of_a_part(of_the_bar, done, whole),
            the_cells_that_played: crate::ui::the_band_of_the_player::the_cells_that_played(
                of_the_bar, done, whole,
            ),
            the_percent: Some(crate::ui::the_band_of_the_player::the_percent_of_a_part(
                done, whole,
            )),
        },
        // **A media of no chapter keeps the row of the second bar**, therefore
        // the shape of the view does not change with the media. It names no
        // number of a chapter, and it says no percent at all (T-91).
        _ => ABarOfTheView {
            the_name: "Ch -".to_string(),
            the_cells: crate::ui::the_band_of_the_player::a_bar_of_a_part(of_the_bar, 0, 0),
            the_cells_that_played: 0,
            the_percent: None,
        },
    };

    Some([of_the_book, of_the_chapter])
}

/// The columns of the name at the left of the two bars, with the space after
/// it.
///
/// **The two names stand in one field**, therefore the two bars start at the
/// same column. The field grows with the number of the chapters: `Ch 7` takes
/// four columns and `Ch 137` takes six.
pub fn the_columns_of_the_name(count: usize) -> u16 {
    let digits = count.max(1).to_string().len() as u16;

    // "Ch " and the digits, or "Book"; and one space after the longest of them.
    (3 + digits).max(4) + 1
}

/// What the media of the view of the chapters is now. See T-162.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TheMediaOfTheChapters {
    /// The media whose chapters the user opened still plays.
    ItStillPlays,
    /// The media whose chapters the user opened does not play now.
    ItWentAway,
    /// The view holds no media yet, therefore the program reads the media that
    /// plays.
    TheProgramReadsItAgain,
}

/// Tells what happened to the media whose chapters the user opened.
///
/// **The media that plays changes while the view stands open, and no key of the
/// user does it**: the media comes to its end and the queue starts the media of
/// its front. The list of the chapters is then the list of another media, and
/// the line keeps the number of the line: the key `l` of the view seeks in a
/// media that the user did not choose. The measurement of 2026-08-14: the user
/// chose "The third part" of a book of 30 minutes, the queue started a book of
/// eight hours, and the key `l` took that book from 4:50:35 to 5:33:20. See
/// T-162, and T-160 and T-161 for the same rule of two other views.
///
/// `of_the_program` is the playback that the view opened, and `of_the_player` is
/// the playback of the engine now. A playback that stopped gives nothing.
///
/// The function is pure, therefore a test needs no engine and no screen.
pub fn what_the_media_of_the_chapters_is(
    of_the_program: Option<u64>,
    of_the_player: Option<u64>,
) -> TheMediaOfTheChapters {
    match of_the_program {
        None => TheMediaOfTheChapters::TheProgramReadsItAgain,
        Some(playback) if Some(playback) == of_the_player => TheMediaOfTheChapters::ItStillPlays,
        Some(_) => TheMediaOfTheChapters::ItWentAway,
    }
}

/// The text for the user when the media of the view of the chapters goes away.
///
/// **The program cannot know which chapter the user wants now**, therefore it
/// takes the line away and it says what happened. The key `l` then seeks in no
/// media at all, and the user chooses the next chapter.
///
/// The sentence names no cause: this program cannot tell a media that came to
/// its end from a media that a key of the player stopped (T-91). It names the
/// two keys of the view that give a line again, and it promises no other key
/// (T-118 and T-143). See T-162.
///
/// **The sentence names the episode of a podcast** (T-227): the name of a
/// playback of a podcast is the name of the podcast, and every episode of that
/// podcast holds it (T-223). The sentence said `The media "Arthur Gordon Pym"
/// does not play now.` for the episode `Chapter 01`, and the queue then started
/// `Chapter 00` of that same podcast with no key of the user.
pub fn the_text_of_the_media_that_went_away(title: &str, episode_title: Option<&str>) -> String {
    format!(
        "The media \"{}\" does not play now. \
         No line is selected: the keys j and k select one.",
        crate::logic::media_name::the_name_of_the_media(title, episode_title)
    )
}

/// The header of the view of the chapters.
///
/// The list holds no line for three reasons, and the header must name the right
/// one. A user who presses `C` with no media reads a different sentence from a
/// user whose media holds no chapter. See T-59.
///
/// **The two headers of a media name the episode of a podcast** (T-227): the
/// header said `"Arthur Gordon Pym" holds no chapter.` for the episode
/// `Chapter 01` and, after the queue started `Chapter 00` of that same podcast
/// with no key of the user, the same words again, while the row of the player of
/// that same frame said which episode plays (T-225). The two episodes of one
/// podcast gave one header, and the user could not tell which episode the view
/// holds.
///
/// A media that plays no more names no media at all, because the program then
/// holds the name of no media of a chapter.
pub fn the_header_of_the_view(
    title: &str,
    episode_title: Option<&str>,
    count: usize,
    the_playback_stopped: bool,
) -> String {
    if count > 0 {
        return format!(
            "The chapters of \"{}\" [{}]",
            crate::logic::media_name::the_name_of_the_media(title, episode_title),
            crate::ui::keys::items(count)
        );
    }

    if the_playback_stopped {
        return "No media plays now. A media that plays gives its chapters. Press h to go back."
            .to_string();
    }

    format!(
        "\"{}\" holds no chapter. Press h to go back.",
        crate::logic::media_name::the_name_of_the_media(title, episode_title)
    )
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

    /// The media of the user plays, therefore the line of the user stays. See
    /// T-162.
    #[test]
    fn the_media_that_the_user_opened_still_plays() {
        assert_eq!(
            what_the_media_of_the_chapters_is(Some(7), Some(7)),
            TheMediaOfTheChapters::ItStillPlays
        );
    }

    /// **The queue starts the media of its front with no key of the user**, and
    /// the list of the chapters is then the list of another media. See T-162.
    #[test]
    fn a_media_that_does_not_play_now_went_away() {
        assert_eq!(
            what_the_media_of_the_chapters_is(Some(7), Some(8)),
            TheMediaOfTheChapters::ItWentAway
        );

        // The media came to its end, and no media plays now.
        assert_eq!(
            what_the_media_of_the_chapters_is(Some(7), None),
            TheMediaOfTheChapters::ItWentAway
        );
    }

    /// The view opens with the media of this moment. See T-162.
    #[test]
    fn a_view_that_holds_no_media_reads_the_media_that_plays() {
        assert_eq!(
            what_the_media_of_the_chapters_is(None, Some(7)),
            TheMediaOfTheChapters::TheProgramReadsItAgain
        );

        assert_eq!(
            what_the_media_of_the_chapters_is(None, None),
            TheMediaOfTheChapters::TheProgramReadsItAgain
        );
    }

    /// The text names the media that went away, and it promises the two keys
    /// of the view only. See T-118, T-143, and T-162.
    #[test]
    fn the_text_names_the_media_and_two_keys() {
        let text = the_text_of_the_media_that_went_away("A Long Test Book", None);

        assert!(text.contains("A Long Test Book"), "{}", text);
        assert!(text.contains("keys j and k"), "{}", text);
        assert!(!text.contains("press h"), "{}", text);
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
