pub mod search_active;

use crate::api::libraries::get_all_books::LibraryItem;

/// One media of the view of the search.
///
/// **The view of the search held the lists of the library**, and it read them
/// with the place of the media in those lists: a media of a page that the
/// program did not read therefore gave no line at all, and the screen said "The
/// server found nothing" for a book that the server found. Every value of a line
/// comes from the answer now. See T-113.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Found {
    pub id: String,
    pub title: String,
    /// The author of a book (`authorName`).
    pub author: String,
    /// The author of a podcast (`author`).
    pub author_of_a_podcast: String,
    pub year: String,
    pub description: String,
    pub duration: f64,
    /// The place of the media in the lists of the library, when the program
    /// holds it.
    ///
    /// The lists of the episodes of a podcast come from that place. A library of
    /// books needs it for no line. See T-113.
    pub place: Option<usize>,
}

/// Makes one line of the view of the search for each media of the server.
///
/// The rule of a text of no letter is the rule of every other view (T-114),
/// therefore a book with no author says "N/A" here too.
///
/// The function is pure, therefore a test needs no server.
pub fn the_media_that_the_server_found(media: &[LibraryItem]) -> Vec<Found> {
    media
        .iter()
        .map(|item| {
            let metadata = item
                .media
                .as_ref()
                .and_then(|media| media.metadata.as_ref());

            let description = metadata
                .and_then(|data| data.description.as_deref())
                .map(crate::utils::html_text::to_plain_text);

            Found {
                id: item.id.clone().unwrap_or_default(),
                title: crate::utils::values_of_the_server::a_text_or_nothing(
                    metadata.and_then(|data| data.title.as_deref()),
                ),
                author: crate::utils::values_of_the_server::a_text_or_nothing(
                    metadata.and_then(|data| data.author_name.as_deref()),
                ),
                author_of_a_podcast: crate::utils::values_of_the_server::a_text_or_nothing(
                    metadata.and_then(|data| data.author.as_deref()),
                ),
                year: crate::utils::values_of_the_server::a_text_or_nothing(
                    metadata.and_then(|data| data.published_year.as_deref()),
                ),
                description: crate::utils::values_of_the_server::a_text_or(
                    description.as_deref(),
                    "No description available",
                ),
                duration: item
                    .media
                    .as_ref()
                    .and_then(|media| media.duration)
                    .unwrap_or(0.0),
                place: None,
            }
        })
        .collect()
}

/// Gives the title of the view of the search.
///
/// **A view says why it holds no line.** The old title said "Search result [from
/// the server]" for an answer of nothing, and the user then read an empty screen
/// with no reason. This is the shape of the view of the queue. See T-70.
///
/// The title also names the author and the narrator of the answer. T-70 gives the
/// books of those names, therefore the name is the **reason** of a line and not a
/// note beside it.
///
/// The function is pure, therefore a test needs no server and no screen.
pub fn the_title_of_the_search(
    words: &str,
    the_server_answered: bool,
    names: &[String],
    count: usize,
) -> String {
    if count == 0 {
        if !the_server_answered {
            return "The program looks in its own titles. The answer of the server \
                    comes."
                .to_string();
        }

        return format!(
            "The server found nothing for \"{}\". Press / to write other words.",
            words
        );
    }

    let of_the_names = if names.is_empty() {
        String::new()
    } else {
        format!(", with the books of {}", names.join(", "))
    };

    if the_server_answered {
        format!(
            "Search result [{}{}]",
            crate::ui::keys::items(count),
            of_the_names
        )
    } else {
        format!(
            "Search result [{} of the titles of this program]",
            crate::ui::keys::items(count)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A view with no line must say why, and it must say what the user can do.
    #[test]
    fn the_title_says_why_the_view_holds_no_line() {
        let nothing = the_title_of_the_search("zzzznothing", true, &[], 0);

        assert!(nothing.contains("found nothing"), "{}", nothing);
        assert!(nothing.contains("zzzznothing"), "{}", nothing);
        assert!(
            nothing.contains('/'),
            "the title names the key: {}",
            nothing
        );

        // The answer of the server did not come yet. The program shows its own
        // titles, and it must not say that the server found nothing.
        let waiting = the_title_of_the_search("carroll", false, &[], 0);

        assert!(!waiting.contains("found nothing"), "{}", waiting);
        assert!(waiting.contains("comes"), "{}", waiting);
    }

    /// The name of an author is the reason of a line, because T-70 gives the
    /// books of that name.
    #[test]
    fn the_title_names_the_author_of_the_answer() {
        let title = the_title_of_the_search("carroll", true, &["Lewis Carroll".to_string()], 1);

        assert_eq!(
            title,
            "Search result [1 item, with the books of Lewis Carroll]"
        );

        let two = the_title_of_the_search(
            "test",
            true,
            &["A Test Narrator".to_string(), "Test Author".to_string()],
            5,
        );
        assert!(two.contains("A Test Narrator, Test Author"), "{}", two);

        // No name, and the answer of the server holds books.
        //
        // **One line is "1 item", and not "1 items".** The sweep of 80 columns
        // of 2026-08-11 read "Search result [1 items]" for the book of the
        // measurement: this title held its own words, and it did not use
        // `ui::keys::items`. The old form of this test held the fault too.
        // See T-95 and T-85.
        assert_eq!(
            the_title_of_the_search("alice", true, &[], 1),
            "Search result [1 item]"
        );
        assert_eq!(
            the_title_of_the_search("alice", true, &[], 2),
            "Search result [2 items]"
        );
        assert_eq!(
            the_title_of_the_search("alice", false, &[], 1),
            "Search result [1 item of the titles of this program]"
        );

        // The titles of the program, while the server answers.
        assert!(the_title_of_the_search("alice", false, &[], 2).contains("this program"));
    }
}

/// The answer of the search of the server. See T-24.
///
/// The search runs inside the render, and the render is not asynchronous.
/// Therefore a task asks the server and it puts the answer here, and the
/// render takes it at the next frame. This is the shape of the cover art of
/// T-23.
pub mod from_the_server {
    use std::sync::{Mutex, OnceLock};

    /// The answer of the server for one set of words.
    #[derive(Debug, Clone, Default)]
    pub struct Answer {
        /// The words that the user wrote.
        pub words: String,
        /// Every media that the server found, with its own values. See T-113.
        pub media: Vec<super::Found>,
        /// The names of the authors and of the narrators that the server
        /// found. The screen shows them when the library holds no media of
        /// that name.
        pub names: Vec<String>,
    }

    fn box_of_the_answer() -> &'static Mutex<Option<Answer>> {
        static ANSWER: OnceLock<Mutex<Option<Answer>>> = OnceLock::new();
        ANSWER.get_or_init(|| Mutex::new(None))
    }

    /// The task puts the answer of the server here.
    pub fn keep(answer: Answer) {
        if let Ok(mut place) = box_of_the_answer().lock() {
            *place = Some(answer);
        }
    }

    /// Gives the answer for these words, if the server answered them.
    ///
    /// The answer of an older search has no use, therefore the words must
    /// agree.
    pub fn answer_for(words: &str) -> Option<Answer> {
        let place = box_of_the_answer().lock().ok()?;
        let answer = place.as_ref()?;

        if answer.words == words {
            Some(answer.clone())
        } else {
            None
        }
    }

    /// Forgets the answer. A new search starts from nothing.
    pub fn forget() {
        if let Ok(mut place) = box_of_the_answer().lock() {
            *place = None;
        }
    }
}
