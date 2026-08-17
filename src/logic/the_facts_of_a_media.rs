//! The facts of a media of the panel 5 of the cover. See T-325.
//!
//! **The maintainer chose the mockup 1, the panels, on 2026-08-16**, therefore
//! `docs/mockups/mockup-1.txt` is the design of the program now. The stage 6 of
//! that road gave the panel 5 its frame, its picture, and its words (T-319),
//! and it left **the facts of the design** open: the mockup names the series,
//! the narrator, the time that is left, the genre, the number of the files, and
//! the state of the ebook, each on a line of its own, with a bar of the
//! progress under them.
//!
//! **The day when the user started the media takes a line too** (T-328): the
//! design names it under the line of the time, and the answer of the account is
//! the one road to it — the answer of the items of a library holds no such
//! field, therefore that line came one round after the six facts.
//!
//! ## The fault of the program before this module
//!
//! **The answer of the server holds every one of those facts already, and no
//! view of the program says one of them.** The measurement of the sandbox of
//! 2026-08-16, of `GET /api/libraries/:id/items` for `A Long Test Book`:
//!
//! ```json
//! "narratorName": "A Test Narrator",
//! "seriesName": "",
//! "genres": [ "Fiction", "Adventure" ],
//! "numAudioFiles": 1,
//! "size": 7337326,
//! "ebookFormat": "epub"
//! ```
//!
//! `crate::api::libraries::get_all_books::Metadata` reads `narratorName`,
//! `seriesName`, and `genres`, and `Media` reads `numAudioFiles` and `size`:
//! the five of them stood in the memory of the program at every frame, and the
//! panel 5 of that same frame said the author, the year, the length, and the
//! place of the user alone, over 15 rows that held no character at all.
//!
//! **The sixth fact was worse**: the field of the ebook of that struct was
//! `ebook_file_format`, and `#[serde(rename_all = "camelCase")]` reads that
//! name as `ebookFileFormat`. The server sends `ebookFormat`, therefore the
//! field was `None` for every book of every library of every server, and no
//! call site of the program read it.
//!
//! ## The rule of this module
//!
//! **A fact that the server did not give takes no line at all.** T-114 gave the
//! line of a view the value `N/A` for a text of no letter, because the line of
//! that time held every fact together and `Author:  - Year: N/A` reads like a
//! program that lost its words. A panel of one fact of one line holds the same
//! intent with no line: a row that says `Narrator  N/A` costs a row of the
//! screen and it tells the user nothing.
//!
//! **The length of the media and the place of the user always take a line**,
//! because a media of no length and a media at the start are facts of their
//! own, and the bar of the progress is the one part of the panel that a user
//! reads with no letter at all.
//!
//! Every function of this module is pure, therefore a test of it needs no
//! terminal, no server, and no `App`.

/// The column where the value of a fact starts.
///
/// The longest label of the design is `Progress`, of eight columns. Two columns
/// of space after it give the values of every line one start.
pub const THE_COLUMN_OF_A_VALUE: usize = 10;

/// The cell of the part of the media that the user heard, in the bar of the
/// progress.
pub const THE_CELL_THAT_IS_DONE: char = '█';

/// The cell of the part of the media that stays, in the bar of the progress.
pub const THE_CELL_THAT_STAYS: char = '░';

/// The narrowest bar of the progress.
///
/// A bar of fewer cells than this says no part of a whole: one cell of four is
/// 25 percent, and the user reads the percent of the line above it instead.
pub const THE_NARROWEST_BAR: u16 = 8;

/// The facts of one media that the answer of the items of a library holds, and
/// that no other list of the program keeps. See T-325.
///
/// The program holds one of these for each item of the library, beside
/// `App::titles_library` and the other lists of a row.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TheFactsOfAMedia {
    /// The name of the series of the book, with the number of the book in it:
    /// `The Test Chronicles #2`. A book of no series holds no text.
    pub series: String,

    /// The name of the person who reads the book. A book of no narrator holds
    /// no text.
    pub narrator: String,

    /// The genres of the media, with a comma between two of them. A media of no
    /// genre holds no text.
    pub genre: String,

    /// The number of the files of the audio of the media, and 0 for a media
    /// whose number the server did not give.
    pub files: i64,

    /// The size of the media on the disk of the server, in bytes, and 0 for a
    /// media whose size the server did not give.
    pub size: i64,

    /// The kind of the file of the ebook of the media: `epub`, `pdf`, and so
    /// on. A media of no ebook holds no text.
    pub the_ebook: String,
}

/// The media of the panel 5, in the words that the screen holds already.
///
/// The lists of `App` hold the author, the year, and the length of a media, and
/// `crate::logic::the_panel_of_a_line` gives the place of the user of it: this
/// struct carries those values beside the facts of this module, so that
/// [`the_lines_of_the_facts`] stays one function of one argument.
#[derive(Debug, Clone, Copy)]
pub struct TheMediaOfThePanel<'a> {
    /// The facts of the answer of the items of the library.
    pub facts: &'a TheFactsOfAMedia,

    /// The name of the author, in the words of
    /// `crate::api::utils::collect_get_all_books::collect_auth_names_library`.
    pub author: &'a str,

    /// The year of the publication, or `N/A`.
    pub year: &'a str,

    /// The length of the media, in the words of
    /// `crate::utils::convert_seconds::convert_seconds`.
    pub length: &'a str,

    /// What the disk of the user holds of this media, in the words of
    /// `crate::ui::keys::the_label_of_the_copy_of_the_disk`.
    pub of_the_disk: &'a str,

    /// The percent of the media, with no sign of the percent.
    pub percent: &'a str,

    /// The time that is left of the media, in the words of
    /// `crate::utils::convert_seconds::convert_seconds_for_prg`.
    pub the_time_that_is_left: &'a str,

    /// `Finished` or `Not finished`.
    pub the_end: &'a str,

    /// The day when the user started the media, in the words of
    /// `crate::api::utils::collect_get_media_progress::the_day_of_the_start`. A
    /// media that the user never started holds no text. See T-328.
    pub the_day_of_the_start: &'a str,
}

/// Says if the server gave this fact.
///
/// **`N/A` is the word of a value that the server did not give** (T-114), and
/// this module gives no line to such a value.
fn a_value(text: &str) -> Option<&str> {
    // **The words of the time that is left end with a comma**: the line of the
    // panel of today is `Progress: {}%, {} {}`, and
    // `convert_seconds_for_prg` gives `15m left,` for the middle of it. A line
    // of one fact holds no such comma.
    let text = text.trim().trim_end_matches(',').trim_end();

    (!text.is_empty() && text != "N/A").then_some(text)
}

/// One line of the panel: the label, and the value at its column.
fn line_of(label: &str, value: &str) -> String {
    format!("{:<width$}{}", label, value, width = THE_COLUMN_OF_A_VALUE)
}

/// The value of the line of the files: the number of them, and the size of the
/// media on the disk of the server.
fn the_files_of(facts: &TheFactsOfAMedia) -> Option<String> {
    let files = (facts.files > 0).then(|| crate::ui::keys::counted(facts.files as usize, "file"));

    let size = (facts.size > 0).then(|| crate::ui::keys::megabytes(facts.size as u64));

    match (files, size) {
        (Some(files), Some(size)) => Some(format!("{}, {}", files, size)),
        (Some(files), None) => Some(files),
        (None, Some(size)) => Some(size),
        (None, None) => None,
    }
}

/// The value of the line of the length: the length of the media, and the time
/// that is left of it.
fn the_time_of(media: &TheMediaOfThePanel) -> Option<String> {
    let length = a_value(media.length);
    let left = a_value(media.the_time_that_is_left);

    match (length, left) {
        (Some(length), Some(left)) => Some(format!("{}, {}", length, left)),
        (Some(length), None) => Some(length.to_string()),
        (None, Some(left)) => Some(left.to_string()),
        (None, None) => None,
    }
}

/// The value of the line of the place of the user: the percent, and the mark of
/// the end.
fn the_place_of(media: &TheMediaOfThePanel) -> Option<String> {
    let percent = a_value(media.percent).map(|percent| format!("{}%", percent));
    let end = a_value(media.the_end);

    match (percent, end) {
        (Some(percent), Some(end)) => Some(format!("{}, {}", percent, end)),
        (Some(percent), None) => Some(percent),
        (None, Some(end)) => Some(end.to_string()),
        (None, None) => None,
    }
}

/// The bar of the progress of the media. See T-325.
///
/// **A bar of no whole is no bar**: a percent that the program does not have
/// gives `None`, and the panel then holds no row for it.
///
/// The bar takes the whole width of the panel, and a panel that is narrower
/// than [`THE_NARROWEST_BAR`] holds no bar at all.
pub fn the_bar_of_the_progress(percent: &str, width: u16) -> Option<String> {
    if width < THE_NARROWEST_BAR {
        return None;
    }

    let percent: f64 = a_value(percent)?.parse().ok()?;
    let cells = width as usize;
    let done = ((percent.clamp(0.0, 100.0) / 100.0) * cells as f64).round() as usize;

    let mut bar = String::new();
    bar.extend(std::iter::repeat_n(THE_CELL_THAT_IS_DONE, done));
    bar.extend(std::iter::repeat_n(THE_CELL_THAT_STAYS, cells - done));

    Some(bar)
}

/// Every line of the facts of one media, for the panel 5 of the cover. See
/// T-325.
///
/// `width` is the width of the inside of the panel, it gives the bar of the
/// progress its cells, and **it holds every line of the facts in one row**
/// (T-370). The values of these lines come from the server: a genre of
/// `Science Fiction & Fantasy, Fantasy, Fairy Tales` and a name of a series of
/// `Depthless Hunger, Book #2` are each longer than the panel of a terminal of
/// 85 columns, and the `Paragraph` of the panel carries no `wrap`. ratatui then
/// cuts the end of such a line with **no mark of the cut at all**, and the user
/// reads a genre of `Science Fiction & Fant` and a series that the library does
/// not have. The three points of [`crate::logic::message::in_one_row`] say that
/// the screen cut it.
///
/// The function is pure, therefore a test needs no terminal and no server.
pub fn the_lines_of_the_facts(media: &TheMediaOfThePanel, width: u16) -> Vec<String> {
    let mut lines = Vec::new();

    for (label, value) in [
        ("Series", a_value(&media.facts.series).map(str::to_string)),
        ("Author", a_value(media.author).map(str::to_string)),
        (
            "Narrator",
            a_value(&media.facts.narrator).map(str::to_string),
        ),
        ("Year", a_value(media.year).map(str::to_string)),
        ("Time", the_time_of(media)),
        // **The day of the start stands under the length of the media**, as the
        // design gives it (T-328): the two of them say the time of the media
        // and the time of the user beside each other.
        (
            "Started",
            a_value(media.the_day_of_the_start).map(str::to_string),
        ),
        ("Genre", a_value(&media.facts.genre).map(str::to_string)),
        ("Files", the_files_of(media.facts)),
        ("Ebook", a_value(&media.facts.the_ebook).map(str::to_string)),
        ("Disk", a_value(media.of_the_disk).map(str::to_string)),
        ("Progress", the_place_of(media)),
    ] {
        if let Some(value) = value {
            lines.push(crate::logic::message::in_one_row(
                &line_of(label, &value),
                width,
            ));
        }
    }

    if let Some(bar) = the_bar_of_the_progress(media.percent, width) {
        lines.push(bar);
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The facts of `A Long Test Book` of the sandbox, of the measurement of
    /// 2026-08-16.
    fn a_long_test_book() -> TheFactsOfAMedia {
        TheFactsOfAMedia {
            series: String::new(),
            narrator: "A Test Narrator".to_string(),
            genre: "Fiction, Adventure".to_string(),
            files: 1,
            size: 7337326,
            the_ebook: "epub".to_string(),
        }
    }

    fn the_media_of(facts: &TheFactsOfAMedia) -> TheMediaOfThePanel<'_> {
        TheMediaOfThePanel {
            facts,
            author: "Long Author",
            year: "N/A",
            length: "30m",
            of_the_disk: "",
            percent: "50",
            the_time_that_is_left: "15m left,",
            the_end: "Not finished",
            the_day_of_the_start: "16 Aug 2026",
        }
    }

    /// The panel says the narrator, the genre, the files, and the ebook of the
    /// media, and each of them takes a line of its own.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn the_facts_of_the_server_each_take_a_line() {
        let facts = a_long_test_book();
        let lines = the_lines_of_the_facts(&the_media_of(&facts), 48);

        assert_eq!(
            lines,
            vec![
                "Author    Long Author".to_string(),
                "Narrator  A Test Narrator".to_string(),
                // The day of the start stands under the line of the time
                // (T-328).
                "Time      30m, 15m left".to_string(),
                "Started   16 Aug 2026".to_string(),
                "Genre     Fiction, Adventure".to_string(),
                "Files     1 file, 7.0 MB".to_string(),
                "Ebook     epub".to_string(),
                "Progress  50%, Not finished".to_string(),
                format!("{}{}", "█".repeat(24), "░".repeat(24)),
            ]
        );

        // The year of this book is `N/A`, therefore it takes no line, and the
        // book stands in no series.
        assert!(!lines.iter().any(|line| line.starts_with("Year")));
        assert!(!lines.iter().any(|line| line.starts_with("Series")));
    }

    /// A fact that the server did not give takes no line at all.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn a_fact_that_the_server_did_not_give_takes_no_line() {
        let facts = TheFactsOfAMedia::default();
        let media = TheMediaOfThePanel {
            facts: &facts,
            author: "N/A",
            year: "N/A",
            length: "0m",
            of_the_disk: "",
            percent: "0",
            the_time_that_is_left: "",
            the_end: "Not finished",
            the_day_of_the_start: "",
        };

        // The length of the media and the place of the user always stand.
        assert_eq!(
            the_lines_of_the_facts(&media, 40),
            vec![
                "Time      0m".to_string(),
                "Progress  0%, Not finished".to_string(),
                "░".repeat(40),
            ]
        );
    }

    /// The line of the series, and the line of the disk.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn the_series_and_the_disk_take_a_line_of_their_own() {
        let facts = TheFactsOfAMedia {
            series: "The Test Chronicles #2".to_string(),
            files: 3,
            size: 0,
            ..TheFactsOfAMedia::default()
        };
        let media = TheMediaOfThePanel {
            facts: &facts,
            author: "Series Author",
            year: "2024",
            length: "9h40",
            of_the_disk: "[Downloaded]",
            percent: "100",
            the_time_that_is_left: "0m left",
            the_end: "Finished",
            the_day_of_the_start: "10 Aug 2026",
        };

        let lines = the_lines_of_the_facts(&media, 40);

        assert_eq!(lines[0], "Series    The Test Chronicles #2");
        // This book holds no narrator, therefore the year stands after the
        // author with no line between them.
        assert_eq!(lines[2], "Year      2024");
        assert_eq!(lines[3], "Time      9h40, 0m left");

        // A media whose size the server did not give says the number of its
        // files alone.
        assert!(lines.contains(&"Files     3 files".to_string()));
        assert!(lines.contains(&"Disk      [Downloaded]".to_string()));
        assert_eq!(lines.last(), Some(&"█".repeat(40)));
    }

    /// A line of the facts that is wider than the panel says that the screen
    /// cut it (T-370).
    ///
    /// The measurement of 2026-08-17, of the real program at 85 columns with
    /// the book `Depthless Hunger, Book 2` of the sandbox under the cursor of
    /// the Home view: the panel 5 held 30 columns inside, and it said
    /// `Genre     Science Fiction & Fant` and `Series    Depthless Hunger, Book`
    /// with **no mark of the cut at all**. The user reads a genre and a name of
    /// a series that the media does not have.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn a_line_of_the_facts_that_is_wider_than_the_panel_says_that_the_screen_cut_it() {
        let facts = TheFactsOfAMedia {
            series: "Depthless Hunger, Book #2".to_string(),
            narrator: "Jonathan Davis".to_string(),
            genre: "Science Fiction & Fantasy, Fantasy, Fairy Tales".to_string(),
            files: 1,
            size: 8912896,
            the_ebook: String::new(),
        };
        let media = TheMediaOfThePanel {
            facts: &facts,
            author: "Sarah Lin",
            year: "2026",
            length: "10m",
            of_the_disk: "",
            percent: "1",
            the_time_that_is_left: "",
            the_end: "Not finished",
            the_day_of_the_start: "11 Aug 2026",
        };

        // The panel of the Home view of a terminal of 85 columns.
        let lines = the_lines_of_the_facts(&media, 30);

        assert_eq!(lines[0], "Series    Depthless Hunger, B…");
        assert_eq!(lines[6], "Genre     Science Fiction & F…");

        // Every line of that panel stands in its 30 columns.
        for line in &lines {
            assert!(
                crate::logic::message::the_columns_of(line) <= 30,
                "the line {line:?} takes more than the 30 columns of the panel"
            );
        }

        // A line that the panel holds whole keeps every character of it, and it
        // takes no mark of a cut.
        assert_eq!(lines[1], "Author    Sarah Lin");
        assert!(!lines[1].contains('…'));

        // The panel of 48 columns of a terminal of 160 holds the name of the
        // series whole, and it cuts the genre alone.
        let of_the_wide_panel = the_lines_of_the_facts(&media, 48);
        assert_eq!(of_the_wide_panel[0], "Series    Depthless Hunger, Book #2");
        assert_eq!(
            of_the_wide_panel[6],
            "Genre     Science Fiction & Fantasy, Fantasy, F…"
        );
    }

    /// A panel that is narrow holds no bar of the progress, and a percent that
    /// the program does not have holds none either.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn a_bar_of_no_whole_is_no_bar() {
        assert_eq!(the_bar_of_the_progress("50", THE_NARROWEST_BAR - 1), None);
        assert_eq!(the_bar_of_the_progress("", 40), None);
        assert_eq!(the_bar_of_the_progress("N/A", 40), None);

        // A percent of the server that is not a number says nothing.
        assert_eq!(the_bar_of_the_progress("half", 40), None);

        // A percent above 100 gives a bar that is full, and no cell more.
        assert_eq!(
            the_bar_of_the_progress("140", 10),
            Some("█".repeat(10)),
            "a percent above 100 fills the bar"
        );

        // The bar holds one cell for each column of the panel.
        assert_eq!(
            the_bar_of_the_progress("25", 40).map(|bar| bar.chars().count()),
            Some(40)
        );
    }
}
