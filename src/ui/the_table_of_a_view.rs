//! The table of the panel 4, and the columns of it. See T-321.
//!
//! **This is the stage 4 of the road of the panels.** The stage 2 (T-320) gave
//! the panel 4 its frame, and the stage 3 (T-316) gave it the mouse. The list
//! inside that frame still held **one text of each row**: the mark of the state
//! and the title, and no other word at all.
//!
//! **The measurement of the real program v0.8.147 inside tmux**, of the Library
//! view of the library `Large` of the sandbox, on a screen of 160 columns and
//! 45 rows:
//!
//! ```text
//! ╔4 Library [500 items of 2056] ═══════════════════════════════════════════╗
//! ║➤     Large Book 2056                                                   █║
//! ║      Large Book 2055                                                   │║
//! ```
//!
//! The panel held **74 columns** and the title of a book took **17** of them.
//! The author, the length, and the place of the user of each row stood in no
//! column of that panel: the user read them for the row of the cursor alone, in
//! the panel of the description under the list, and a user who looks for a book
//! of two hours had to move the cursor over every row of the library.
//!
//! The design of `docs/mockups/mockup-1.txt` gives that panel a row of a header
//! and four columns:
//!
//! ```text
//! ║  Title                             Author            Time Done▲║
//! ║  A Big Book Of A Scan              Ada Lovelace     11h20  90%█║
//! ║✓ A Huge Book Of A Scan             Alan Turing       6h05 100%█║
//! ```
//!
//! ## The three shapes of the table
//!
//! **A column that takes the title away is a column that costs more than it
//! gives.** The panel 4 stands at 120 columns and up (`frame::TheShape`), and
//! the width of it changes with the width of the screen: the measurement of
//! this round gave the panel **52 columns at a screen of 120** and **74 at a
//! screen of 160**. The table therefore takes its columns in the sequence of
//! their value, and it keeps [`THE_SMALLEST_TITLE`] columns for the title of
//! the media:
//!
//! - **Done** comes first: it is the one word that the list of today never
//!   said and that the user asks for the most (T-242 gave that percent to the
//!   panel of the description, and it never reached the list).
//! - **Time** comes second.
//! - **Author** comes last, because it is the widest of the three.
//!
//! **A panel that holds no `Done` column holds no table at all**
//! ([`TheColumns::the_table_stands`]), and the view then draws the list of
//! today. That is the road of a terminal of 84 to 119 columns, and of a screen
//! that the panel of the covers takes the width of.
//!
//! ## The mark of the state, and the percent
//!
//! **The mark of a line of this program holds the percent already**
//! (`crate::ui::marks::of_progress` gives `11% ` in four columns), therefore a
//! table that keeps that mark beside a column `Done` says the same number two
//! times. The rows of this table take [`crate::ui::marks::of_the_state`], which
//! is the mark of the state alone in two columns, and the percent stands in the
//! column of the header that names it.
//!
//! **Every other view of the program keeps the mark that it had**: this module
//! reaches the Home view and the Library view alone, and it reaches them at the
//! widths of the table alone.

use crate::logic::message::in_one_row;

/// The columns that the title of a media keeps at every width of the table.
///
/// A title of fewer columns says nothing: `Large Book 20…` is not the name of a
/// book. The measurement of the sandbox gave the titles of the library `Large`
/// 15 columns, and the books of the library `Books` up to 30.
pub const THE_SMALLEST_TITLE: u16 = 20;

/// The columns of the mark of the state of a row, with the space after it.
pub const THE_WIDTH_OF_THE_MARK: u16 = 2;

/// The columns of the author of a row.
pub const THE_WIDTH_OF_THE_AUTHOR: u16 = 18;

/// The columns of the length of a media: `11h20` and `41h02` of the design.
pub const THE_WIDTH_OF_THE_TIME: u16 = 6;

/// The columns of the place of the user: `100%`, `11%`, and `-` of the design.
pub const THE_WIDTH_OF_THE_DONE: u16 = 5;

/// The name of each column of the header of the table.
pub const THE_TITLE: &str = "Title";
/// The name of the column of the author.
pub const THE_AUTHOR: &str = "Author";
/// The name of the column of the length.
pub const THE_TIME: &str = "Time";
/// The name of the column of the place of the user.
pub const THE_DONE: &str = "Done";

/// The width of each column of the table, in columns of the screen. A width of
/// 0 says that the table does not hold that column at this width.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TheColumns {
    /// The title of the media. It takes every column that the others leave.
    pub title: u16,
    /// The author of the media.
    pub author: u16,
    /// The length of the media.
    pub time: u16,
    /// The place of the user in the media.
    pub done: u16,
}

impl TheColumns {
    /// Says that the panel holds the table. A panel that holds no column of the
    /// place of the user draws the list of today.
    pub fn the_table_stands(self) -> bool {
        self.title > 0 && self.done > 0
    }

    /// The columns of the whole of a row, with the mark and the spaces between
    /// the columns. A row of a shelf and a row of a series take this width.
    pub fn the_whole_row(self) -> u16 {
        let mut width = THE_WIDTH_OF_THE_MARK.saturating_add(self.title);

        for column in [self.author, self.time, self.done] {
            if column > 0 {
                width = width.saturating_add(1).saturating_add(column);
            }
        }

        width
    }
}

/// The columns of a table of this width, in the sequence of their value.
///
/// `width` is the width of the lines of the list, which is the width inside the
/// border of the panel with the bar of the scroll and the sign of the cursor
/// taken away.
pub fn the_columns_of_the_table(width: u16) -> TheColumns {
    let of_the_title = width.saturating_sub(THE_WIDTH_OF_THE_MARK);

    if of_the_title < THE_SMALLEST_TITLE {
        return TheColumns::default();
    }

    let mut the_columns = TheColumns {
        title: of_the_title,
        ..TheColumns::default()
    };

    // **The sequence of the columns is the sequence of their value** (the head
    // of this module). Each one comes with the space that stands before it.
    for (of_the_column, width) in [
        (&mut the_columns.done, THE_WIDTH_OF_THE_DONE),
        (&mut the_columns.time, THE_WIDTH_OF_THE_TIME),
        (&mut the_columns.author, THE_WIDTH_OF_THE_AUTHOR),
    ] {
        let cost = width.saturating_add(1);

        if the_columns.title.saturating_sub(cost) < THE_SMALLEST_TITLE {
            break;
        }

        the_columns.title -= cost;
        *of_the_column = width;
    }

    the_columns
}

/// One row of the table of the panel 4.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ARowOfTheTable {
    /// The mark of the state of the media, in [`THE_WIDTH_OF_THE_MARK`]
    /// columns.
    pub the_mark: String,
    /// The title of the media, or the name of a shelf.
    pub title: String,
    /// The author of the media.
    pub author: String,
    /// The length of the media.
    pub time: String,
    /// The place of the user in the media.
    pub done: String,
    /// Says that this row takes every column of the table. The name of a shelf
    /// of the Home view and the line of a series are rows of that shape: they
    /// name no one media, therefore they hold no author, no length, and no
    /// place of the user.
    pub the_whole_width: bool,
}

/// Gives a text of exactly `width` columns of the screen: it cuts the text that
/// is longer, and it fills the text that is shorter with spaces.
fn of_the_width(text: &str, width: u16, at_the_right: bool) -> String {
    let text = in_one_row(text, width);
    let columns = crate::logic::message::the_columns_of(&text);
    let room = usize::from(width).saturating_sub(columns);
    let spaces = " ".repeat(room);

    if at_the_right {
        format!("{spaces}{text}")
    } else {
        format!("{text}{spaces}")
    }
}

/// The text of the row of the header of the table.
///
/// **The header holds the mark of no state**, therefore its first columns are
/// the spaces of the mark, and the name `Title` stands over the titles of the
/// media.
pub fn the_header_of_the_table(the_columns: TheColumns) -> String {
    the_text_of_a_row(
        &ARowOfTheTable {
            the_mark: String::new(),
            title: THE_TITLE.to_string(),
            author: THE_AUTHOR.to_string(),
            time: THE_TIME.to_string(),
            done: THE_DONE.to_string(),
            the_whole_width: false,
        },
        the_columns,
    )
}

/// The text of one row of the table.
///
/// **The title of a media stands at the left of its column and the numbers
/// stand at the right of theirs**: a column of numbers that ends at the same
/// column reads as one number, and the design of `docs/mockups/mockup-1.txt`
/// holds that shape. The author stands at the left, because it is a name.
pub fn the_text_of_a_row(row: &ARowOfTheTable, the_columns: TheColumns) -> String {
    let mark = of_the_width(&row.the_mark, THE_WIDTH_OF_THE_MARK, false);

    if row.the_whole_width {
        let of_the_row = the_columns
            .the_whole_row()
            .saturating_sub(THE_WIDTH_OF_THE_MARK);

        return format!("{mark}{}", of_the_width(&row.title, of_the_row, false));
    }

    let mut text = format!(
        "{mark}{}",
        of_the_width(&row.title, the_columns.title, false)
    );

    for (word, width, at_the_right) in [
        (the_word_of_a_column(&row.author), the_columns.author, false),
        (the_word_of_a_column(&row.time), the_columns.time, true),
        (the_word_of_a_column(&row.done), the_columns.done, true),
    ] {
        if width == 0 {
            continue;
        }

        text.push(' ');
        text.push_str(&of_the_width(word, width, at_the_right));
    }

    text
}

/// The length of a media, in the form of the column `Time` of the design:
/// `11h20` for a book of eleven hours and twenty minutes, and `44m` for a media
/// under one hour.
///
/// **A length that the server did not give is no length at all** (T-180), and
/// the row of it then says `-`, which is the word of the column `Done` for a
/// media that the user did not start.
pub fn the_time_of_a_row(seconds: Option<f64>) -> String {
    let Some(seconds) = seconds else {
        return "-".to_string();
    };

    if !seconds.is_finite() || seconds < 1.0 {
        return "-".to_string();
    }

    let minutes = (seconds / 60.0).round() as i64;
    let hours = minutes / 60;

    // **A media of one second is not a media of no length** (the measurement of
    // this round): every one of the 2056 books of the library `Large` of the
    // sandbox holds `duration: 1`, and `(1 / 60).round()` gives 0. A column
    // that says `0m` for a media that the server gave a length of says a
    // number that is not true.
    if minutes == 0 {
        return "<1m".to_string();
    }

    if hours == 0 {
        return format!("{minutes}m");
    }

    format!("{hours}h{:02}", minutes % 60)
}

/// The word of a column that names a value of the server which that server did
/// not give. See T-321.
///
/// **`N/A` is the value of this program and not the value of the server**: the
/// collectors of `src/api` write those three characters for a field that the
/// answer of the server leaves empty, and the panel of the description of a
/// media therefore says `Author: N/A`. A **column** of that word says it on
/// every row of the screen, and the measurement of this round gave 17 rows of
/// `N/A` in the column of the author of the library `Large` of the sandbox.
///
/// A cell with no word says the same thing, and it says it one time: the header
/// of the column names the value that the row does not hold.
pub fn the_word_of_a_column(word: &str) -> &str {
    if word.trim() == "N/A" {
        return "";
    }

    word
}

/// The place of the user in a media, in the form of the column `Done` of the
/// design.
///
/// `percent` and `finished` come from the server in the form that
/// `crate::ui::marks::of_progress` reads: a number as a text, and `Finished` or
/// `Not finished`.
///
/// **A percent of 100 is not the mark of the end** (T-290), therefore the word
/// `done` comes of the field `isFinished` of the server alone, and a media of
/// 100 percent that the user did not finish says `100%`.
pub fn the_done_of_a_row(percent: &str, finished: &str) -> String {
    if finished.trim() == "Finished" {
        return "done".to_string();
    }

    match percent.trim().parse::<i64>() {
        Ok(value) if value > 0 => format!("{}%", value.min(100)),
        _ => "-".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The table takes its columns in the sequence of their value, and it keeps
    /// the title of the media. See T-321.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn the_columns_of_the_table_keep_the_title_of_the_media() {
        // The measurement of this round: the panel 4 of a screen of 160
        // columns holds 69 columns of lines, and it therefore holds every
        // column of the design.
        let of_a_wide_screen = the_columns_of_the_table(69);
        assert!(of_a_wide_screen.the_table_stands());
        assert_eq!(of_a_wide_screen.author, THE_WIDTH_OF_THE_AUTHOR);
        assert_eq!(of_a_wide_screen.time, THE_WIDTH_OF_THE_TIME);
        assert_eq!(of_a_wide_screen.done, THE_WIDTH_OF_THE_DONE);
        assert!(of_a_wide_screen.title >= THE_SMALLEST_TITLE);
        assert_eq!(of_a_wide_screen.the_whole_row(), 69);

        // The panel 4 of a screen of 120 columns holds 47 columns of lines: the
        // author would leave the title 9 columns, therefore it does not come,
        // and the place of the user and the length stay.
        let of_the_narrow_panel = the_columns_of_the_table(47);
        assert!(of_the_narrow_panel.the_table_stands());
        assert_eq!(of_the_narrow_panel.author, 0);
        assert_eq!(of_the_narrow_panel.time, THE_WIDTH_OF_THE_TIME);
        assert_eq!(of_the_narrow_panel.done, THE_WIDTH_OF_THE_DONE);
        assert_eq!(of_the_narrow_panel.the_whole_row(), 47);

        // **A panel that holds no column of the place of the user holds no
        // table at all**, and the view then draws the list of today.
        for width in [0u16, 1, 10, 21, 25] {
            let the_columns = the_columns_of_the_table(width);
            assert!(
                !the_columns.the_table_stands(),
                "a panel of {width} columns drew a table of {the_columns:?}"
            );
        }

        // The first width that holds the table: the mark of two columns, the
        // title of 20, the space, and the place of the user of five.
        assert!(the_columns_of_the_table(28).the_table_stands());
        assert!(!the_columns_of_the_table(27).the_table_stands());
    }

    /// A row of the table stands in the columns of its header. See T-321.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn a_row_of_the_table_stands_in_the_columns_of_its_header() {
        let the_columns = the_columns_of_the_table(69);

        let header = the_header_of_the_table(the_columns);
        let row = the_text_of_a_row(
            &ARowOfTheTable {
                the_mark: "✓".to_string(),
                title: "A Huge Book Of A Scan".to_string(),
                author: "Alan Turing".to_string(),
                time: "6h05".to_string(),
                done: "done".to_string(),
                the_whole_width: false,
            },
            the_columns,
        );

        // Every row of the table holds the columns of the panel, therefore the
        // words of the header stand over the words of the rows.
        assert_eq!(crate::logic::message::the_columns_of(&header), 69);
        assert_eq!(crate::logic::message::the_columns_of(&row), 69);
        assert!(header.starts_with("  Title "));
        assert!(row.starts_with("✓ A Huge Book Of A Scan"));

        // The numbers stand at the right of their column, therefore the column
        // of the header ends where the column of the row ends.
        assert!(header.ends_with(" Done"));
        assert!(row.ends_with(" done"));
        // **The column of the screen is not the index of the byte**: the mark
        // `✓` of the row takes three bytes and one column, therefore a `find`
        // of the two texts says that the columns do not agree while they do.
        let the_column_of = |text: &str, word: &str| -> Option<usize> {
            text.find(word)
                .map(|at| crate::logic::message::the_columns_of(&text[..at]))
        };
        assert_eq!(
            the_column_of(&header, THE_AUTHOR),
            the_column_of(&row, "Alan Turing")
        );

        // **A row that names no one media takes every column of the table**: a
        // shelf of the Home view and a line of a series hold no author, no
        // length, and no place of the user.
        let of_a_shelf = the_text_of_a_row(
            &ARowOfTheTable {
                the_mark: "▌".to_string(),
                title: "Continue Listening".to_string(),
                the_whole_width: true,
                ..ARowOfTheTable::default()
            },
            the_columns,
        );
        assert_eq!(crate::logic::message::the_columns_of(&of_a_shelf), 69);
        assert!(of_a_shelf.starts_with("▌ Continue Listening "));
        assert!(of_a_shelf.trim_end().ends_with("Listening"));

        // A title that is longer than its column says that the screen cut it,
        // and the row keeps the width of the table.
        let of_a_long_title = the_text_of_a_row(
            &ARowOfTheTable {
                the_mark: String::new(),
                title: "A".repeat(200),
                author: "B".repeat(200),
                time: "11h20".to_string(),
                done: "90%".to_string(),
                the_whole_width: false,
            },
            the_columns,
        );
        assert_eq!(crate::logic::message::the_columns_of(&of_a_long_title), 69);
        assert!(of_a_long_title.contains('…'));
    }

    /// The words of the columns `Time` and `Done` of a row. See T-321.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn the_time_and_the_done_of_a_row_say_the_media_of_the_server() {
        // The lengths of the design: eleven hours and twenty minutes, six hours
        // and five minutes, and a media under one hour.
        assert_eq!(
            the_time_of_a_row(Some(11.0 * 3600.0 + 20.0 * 60.0)),
            "11h20"
        );
        assert_eq!(the_time_of_a_row(Some(6.0 * 3600.0 + 5.0 * 60.0)), "6h05");
        assert_eq!(the_time_of_a_row(Some(44.0 * 60.0)), "44m");

        // **A media of one second is not a media of no length**: the 2056
        // books of the library `Large` of the sandbox each hold `duration: 1`,
        // and a column of `0m` says a number that is not true.
        assert_eq!(the_time_of_a_row(Some(1.0)), "<1m");
        assert_eq!(the_time_of_a_row(Some(20.0)), "<1m");

        // **`N/A` is the value of this program and not the value of the
        // server**, and a column of it says it on every row of the screen.
        assert_eq!(the_word_of_a_column("N/A"), "");
        assert_eq!(the_word_of_a_column("Ada Lovelace"), "Ada Lovelace");
        assert_eq!(the_word_of_a_column(""), "");

        // **A length that the server did not give is no length at all**
        // (T-180): the row of it says `-`, and it does not say `0m`.
        assert_eq!(the_time_of_a_row(None), "-");
        assert_eq!(the_time_of_a_row(Some(0.0)), "-");
        assert_eq!(the_time_of_a_row(Some(f64::NAN)), "-");

        // **A percent of 100 is not the mark of the end** (T-290): the word
        // `done` comes of the field `isFinished` of the server alone.
        assert_eq!(the_done_of_a_row("100", "Not finished"), "100%");
        assert_eq!(the_done_of_a_row("100", "Finished"), "done");
        assert_eq!(the_done_of_a_row("11", "Not finished"), "11%");
        assert_eq!(the_done_of_a_row("", "Finished"), "done");

        // A media that the user did not start, and a percent that the server
        // did not give.
        assert_eq!(the_done_of_a_row("0", "Not finished"), "-");
        assert_eq!(the_done_of_a_row("", ""), "-");
        assert_eq!(the_done_of_a_row("N/A", "N/A"), "-");

        // Every word of these two columns stands in the width of them.
        for word in [
            the_time_of_a_row(Some(41.0 * 3600.0 + 2.0 * 60.0)),
            the_done_of_a_row("100", "Not finished"),
            the_done_of_a_row("7", "Not finished"),
        ] {
            assert!(
                crate::logic::message::the_columns_of(&word) <= usize::from(THE_WIDTH_OF_THE_TIME),
                "the word {word} is wider than its column"
            );
        }
    }
}
