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
                // **The panel of a description says why it holds no text**
                // (T-249). Every panel of the program calls this one function,
                // therefore one rule holds for every view.
                description: crate::utils::values_of_the_server::a_description_or_nothing(
                    description.as_deref(),
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

/// Gives the reason of the view of the search that holds no line.
///
/// **A view says why it holds no line.** The old title said "Search result [from
/// the server]" for an answer of nothing, and the user then read an empty screen
/// with no reason. This is the shape of the view of the queue. See T-70.
///
/// `the_podcasts_that_come` is the number of the podcasts of the answer that
/// the program did not read. **The sentence must not say "The server found
/// nothing" for them**: the server found them, and the program reads the pages
/// of the library that hold them. See T-125 and T-91.
///
/// **This sentence stands in the body of the panel and never in the title of
/// it** (T-361). A title takes no wrap, therefore the words
/// `The server found nothing for "zzqqxnothingatall". Press / to write other
/// words.` read `The server found nothing for "zzqqxnoth…` at 40 columns, and
/// the user lost the key of the work.
///
/// The function is pure, therefore a test needs no server and no screen.
pub fn the_reason_of_no_hit(
    words: &str,
    the_server_answered: bool,
    the_podcasts_that_come: usize,
) -> String {
    if !the_server_answered {
        return "The program looks in its own titles. The answer of the server \
                comes."
            .to_string();
    }

    if the_podcasts_that_come > 0 {
        return format!(
            "The server found {}. The program reads the pages of the library, \
             and the line comes.",
            crate::ui::keys::counted(the_podcasts_that_come, "podcast")
        );
    }

    format!(
        "The server found nothing for \"{}\". Press / to write other words.",
        words
    )
}

/// Gives the title of the view of the search.
///
/// The title names the author and the narrator of the answer. T-70 gives the
/// books of those names, therefore the name is the **reason** of a line and not a
/// note beside it.
///
/// **This function gives the name of the list at every count** (T-361), and the
/// reason of a view with no hit comes of `the_reason_of_no_hit`.
///
/// The function is pure, therefore a test needs no server and no screen.
pub fn the_title_of_the_search(
    the_server_answered: bool,
    names: &[String],
    count: usize,
    the_podcasts_that_come: usize,
) -> String {
    let of_the_names = if names.is_empty() {
        String::new()
    } else {
        format!(", with the books of {}", names.join(", "))
    };

    let of_the_pages = if the_podcasts_that_come > 0 {
        format!(
            " — the program reads the pages of the library for {} more",
            crate::ui::keys::counted(the_podcasts_that_come, "podcast")
        )
    } else {
        String::new()
    };

    if the_server_answered {
        format!(
            "Search result [{}{}]{}",
            crate::ui::keys::items(count),
            of_the_names,
            of_the_pages
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
    ///
    /// **The sentence comes of `the_reason_of_no_hit` and it stands in the body
    /// of the panel** (T-361): the title of a block takes no wrap.
    #[test]
    fn the_title_says_why_the_view_holds_no_line() {
        let nothing = the_reason_of_no_hit("zzzznothing", true, 0);

        assert!(nothing.contains("found nothing"), "{}", nothing);
        assert!(nothing.contains("zzzznothing"), "{}", nothing);
        assert!(
            nothing.contains('/'),
            "the sentence names the key: {}",
            nothing
        );

        // The answer of the server did not come yet. The program shows its own
        // titles, and it must not say that the server found nothing.
        let waiting = the_reason_of_no_hit("carroll", false, 0);

        assert!(!waiting.contains("found nothing"), "{}", waiting);
        assert!(waiting.contains("comes"), "{}", waiting);

        // **The title of that same view names the list and no reason at all**
        // (T-361).
        let title = the_title_of_the_search(true, &[], 0, 0);

        assert_eq!(title, "Search result [0 items]");
    }

    /// The name of an author is the reason of a line, because T-70 gives the
    /// books of that name.
    #[test]
    fn the_title_names_the_author_of_the_answer() {
        let title = the_title_of_the_search(true, &["Lewis Carroll".to_string()], 1, 0);

        assert_eq!(
            title,
            "Search result [1 item, with the books of Lewis Carroll]"
        );

        let two = the_title_of_the_search(
            true,
            &["A Test Narrator".to_string(), "Test Author".to_string()],
            5,
            0,
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
            the_title_of_the_search(true, &[], 1, 0),
            "Search result [1 item]"
        );
        assert_eq!(
            the_title_of_the_search(true, &[], 2, 0),
            "Search result [2 items]"
        );
        assert_eq!(
            the_title_of_the_search(false, &[], 1, 0),
            "Search result [1 item of the titles of this program]"
        );

        // The titles of the program, while the server answers.
        assert!(the_title_of_the_search(false, &[], 2, 0).contains("this program"));
    }

    /// **The server found a podcast of a page that the program did not read,
    /// and the old title said "The server found nothing".**
    ///
    /// The sweep of a library of 520 podcasts of 2026-08-12 met that condition:
    /// the log said "the program did not read 1 podcast(s) of the answer" while
    /// the screen said that the server found nothing. A view must not say a
    /// reason that the program does not have. See T-125 and T-91.
    #[test]
    fn the_title_says_that_the_pages_of_the_library_come() {
        let one = the_reason_of_no_hit("Many Podcast 001", true, 1);

        assert!(
            !one.contains("found nothing"),
            "the server found that podcast: {}",
            one
        );
        assert!(one.contains("1 podcast"), "{}", one);
        assert!(
            one.contains("reads the pages"),
            "the title says what the program does now: {}",
            one
        );

        // A line came, and a podcast of a page that the program did not read
        // waits for its page too.
        let some = the_title_of_the_search(true, &[], 3, 2);

        assert!(some.contains("3 items"), "{}", some);
        assert!(some.contains("2 podcasts"), "{}", some);
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
