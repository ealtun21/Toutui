//! The rule of a text that the server gives, for the screen. See T-114.
//!
//! **A text of no letter is not a value.** The program read `null` as "the
//! server gave nothing", and it wrote every other text on the screen as it
//! stood. The server gives `""`.
//!
//! The measurement of 2026-08-12, of a library of 2056 books that hold no tag of
//! an author:
//!
//! ```text
//! {"title": "Large Book 2056", "authorName": "", "narratorName": "",
//!  "seriesName": "", "publishedYear": null}
//! ```
//!
//! The Library view then said `Author:  - Year: N/A`: the year of the same book
//! said "N/A", and the author said nothing at all. **A user cannot tell an empty
//! value from a fault of the program.**
//!
//! Every list of a media takes its text through this module now, therefore one
//! rule holds for every view.

/// The words for a value that the server does not have.
pub const NOT_AVAILABLE: &str = "N/A";

/// Gives the text of the server, or other words when that text is not a value.
///
/// A text of no letter is not a value: `""` and a text of spaces both give the
/// words of `absent`.
pub fn a_text_or(value: Option<&str>, absent: &str) -> String {
    match value {
        Some(text) if !text.trim().is_empty() => text.to_string(),
        _ => absent.to_string(),
    }
}

/// Gives the text of the server, or "N/A".
pub fn a_text_or_nothing(value: Option<&str>) -> String {
    a_text_or(value, NOT_AVAILABLE)
}

/// The words of a description that the server does not have. See T-249.
pub const NO_DESCRIPTION: &str = "No description available";

/// Gives the description of the server, or the words of a description that it
/// does not have. See T-249.
///
/// **"N/A" is a value of a field, and a description is a panel of its own.** The
/// words `NOT_AVAILABLE` stand beside a label: the line of the Library view says
/// `Year: N/A`, and the label tells the user which value the server does not
/// have. The panel of the description holds no label at all, therefore `N/A`
/// alone stands on a line of the screen and it says nothing to the user.
///
/// The measurement of 2026-08-15, of `A Long Test Book` of the sandbox, which
/// holds no description at all. One book, one frame each, two views:
///
/// ```text
/// Search result [1 item]        Home [35 items]
/// ➤ 50% A Long Test Book        ➤ 50% A Long Test Book
/// …                             …
/// No description available      N/A
/// ```
///
/// Every panel of a description takes these words now, therefore one rule holds
/// for every view.
pub fn a_description_or_nothing(value: Option<&str>) -> String {
    a_text_or(value, NO_DESCRIPTION)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A text of no letter is not a value. See T-114.
    #[test]
    fn a_text_of_no_letter_is_not_a_value() {
        // The measurement of 2026-08-12: the server gives `""` for a book that
        // holds no tag of an author.
        assert_eq!(a_text_or_nothing(Some("")), "N/A");
        assert_eq!(a_text_or_nothing(Some("   ")), "N/A");
        assert_eq!(a_text_or_nothing(Some("\t\n")), "N/A");
        assert_eq!(a_text_or_nothing(None), "N/A");

        // A text that holds a letter stays as it stands, with its spaces.
        assert_eq!(a_text_or_nothing(Some("Lewis Carroll")), "Lewis Carroll");
        assert_eq!(a_text_or_nothing(Some(" Balzac ")), " Balzac ");
        assert_eq!(a_text_or_nothing(Some("0")), "0");
    }

    /// A view can need its own words for a value that is absent.
    #[test]
    fn a_view_gives_its_own_words() {
        assert_eq!(
            a_text_or(Some(""), "No description available"),
            "No description available"
        );
        assert_eq!(
            a_text_or(Some("A book."), "No description available"),
            "A book."
        );
    }
}
