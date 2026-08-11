//! The authors and the narrators of a library, between the task and the screen.
//! See T-24 and T-73.
//!
//! One view holds the two lists, because a narrator of the server has the shape
//! of an author: an identity, a name, and a number of books. The key `a` shows
//! the authors and the key `v` shows the narrators, and `Kind` says which list
//! the view holds now.

use crate::api::libraries::get_authors::Author;
use std::sync::{Mutex, OnceLock};

/// The list that the view holds. See T-73.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Kind {
    #[default]
    Authors,
    Narrators,
}

impl Kind {
    /// Gives the title of the list, with the number of its lines.
    ///
    /// The function is pure, therefore a test needs no screen.
    pub fn title(&self, count: usize) -> String {
        match self {
            Kind::Authors => format!("The authors [{}]", crate::ui::keys::items(count)),
            Kind::Narrators => format!("The narrators [{}]", crate::ui::keys::items(count)),
        }
    }

    /// Gives the title of a list that holds no line.
    pub fn title_of_nothing(&self) -> String {
        match self {
            Kind::Authors => "This library has no author.".to_string(),
            Kind::Narrators => "This library has no narrator. A narrator comes from the \
                 metadata of a file."
                .to_string(),
        }
    }

    /// Gives the sentence of a fault of the server.
    pub fn title_of_a_fault(&self, text: &str) -> String {
        match self {
            Kind::Authors => format!("The server gave no author: {}", text),
            Kind::Narrators => format!("The server gave no narrator: {}", text),
        }
    }

    /// Gives the work of the key `l` for the footer of the view.
    ///
    /// A measurement of 2026-08-11 read the footer of the list of the
    /// narrators: it said "the books of this author". See T-73.
    pub fn work_of_the_key_that_opens(&self) -> &'static str {
        match self {
            Kind::Authors => "the books of this author",
            Kind::Narrators => "the books of this narrator",
        }
    }

    /// Gives the sentence for a library of podcasts.
    pub fn message_of_a_library_of_podcasts(&self) -> String {
        match self {
            Kind::Authors => "A library of podcasts has no author.".to_string(),
            Kind::Narrators => "A library of podcasts has no narrator.".to_string(),
        }
    }

    /// Gives the filter of the library for one line of the list.
    ///
    /// **The two filters do not take the same value.** The filter of an author
    /// takes the identity of that author, and the filter of a narrator takes the
    /// **name**: the server holds a narrator inside the metadata of a file, and
    /// not as a row of its own.
    ///
    /// A measurement of 2026-08-11: the identity that
    /// `GET /api/libraries/:id/narrators` gives is already the name in base64
    /// (`QSBUZXN0IE5hcnJhdG9y` for "A Test Narrator"). The function takes the name
    /// therefore, and it does not depend on that form of the identity.
    pub fn filter_of(&self, one: &Author) -> String {
        match self {
            Kind::Authors => crate::logic::sort_filter::filter_value("authors", &one.id),
            Kind::Narrators => crate::logic::sort_filter::filter_value("narrators", &one.name),
        }
    }
}

/// What the view must draw.
#[derive(Debug, Clone, Default)]
pub enum State {
    /// The program did not ask the server.
    #[default]
    Nothing,
    /// The program asked the server, and no answer came.
    Waiting,
    /// The server answered, in the sequence of the alphabet.
    Ready(Vec<Author>),
    /// The server gave no answer, and this text says why.
    Fault(String),
}

fn box_of_the_state() -> &'static Mutex<State> {
    static STATE: OnceLock<Mutex<State>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(State::Nothing))
}

fn box_of_the_kind() -> &'static Mutex<Kind> {
    static KIND: OnceLock<Mutex<Kind>> = OnceLock::new();
    KIND.get_or_init(|| Mutex::new(Kind::Authors))
}

/// Gives the list that the view holds now. See T-73.
pub fn kind() -> Kind {
    match box_of_the_kind().lock() {
        Ok(place) => *place,
        Err(_) => Kind::Authors,
    }
}

/// Writes the list that the view holds, and it forgets the answer of the list
/// that came before it. See T-73.
pub fn keep_the_kind(new: Kind) {
    let old = kind();

    if let Ok(mut place) = box_of_the_kind().lock() {
        *place = new;
    }

    if old != new {
        forget();
    }
}

/// Writes the state. The task of the request calls this.
pub fn keep(state: State) {
    if let Ok(mut place) = box_of_the_state().lock() {
        *place = state;
    }
}

/// Gives the state. The render calls this at each frame.
pub fn state() -> State {
    match box_of_the_state().lock() {
        Ok(place) => place.clone(),
        Err(_) => State::Nothing,
    }
}

/// Gives the authors that the view holds now.
pub fn authors() -> Vec<Author> {
    match state() {
        State::Ready(all) => all,
        _ => Vec::new(),
    }
}

/// Forgets the answer. A refresh of the program asks the server again.
pub fn forget() {
    keep(State::Nothing);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The footer of the view names the list that it holds.
    ///
    /// A sweep of every view of 2026-08-11 read "the books of this author" in
    /// the footer of the list of the narrators. See T-73.
    #[test]
    fn the_footer_of_each_list_names_its_own_kind() {
        assert_eq!(
            Kind::Authors.work_of_the_key_that_opens(),
            "the books of this author"
        );
        assert_eq!(
            Kind::Narrators.work_of_the_key_that_opens(),
            "the books of this narrator"
        );
    }

    /// The two lists give their own title, and the title of a list that holds
    /// no line says what a narrator is. See T-73.
    #[test]
    fn each_list_gives_its_own_title() {
        assert_eq!(Kind::Authors.title(6), "The authors [6 items]");
        assert_eq!(Kind::Narrators.title(2), "The narrators [2 items]");

        // One line of a view is "1 item", and not "1 items". See T-85.
        assert_eq!(Kind::Authors.title(1), "The authors [1 item]");

        assert_eq!(
            Kind::Authors.title_of_nothing(),
            "This library has no author."
        );
        assert!(Kind::Narrators.title_of_nothing().contains("no narrator"));
        assert!(
            Kind::Narrators.title_of_nothing().contains("metadata"),
            "the title must say where a narrator comes from"
        );

        assert!(Kind::Authors.title_of_a_fault("404").contains("no author"));
        assert!(Kind::Narrators
            .title_of_a_fault("404")
            .contains("no narrator"));
        assert!(Kind::Narrators.title_of_a_fault("404").contains("404"));

        assert!(Kind::Authors
            .message_of_a_library_of_podcasts()
            .contains("no author"));
        assert!(Kind::Narrators
            .message_of_a_library_of_podcasts()
            .contains("no narrator"));
    }

    /// **The filter of an author takes the identity, and the filter of a
    /// narrator takes the name.** A narrator holds no row of its own on the
    /// server. See T-73.
    #[test]
    fn the_filter_of_a_narrator_takes_the_name() {
        let one = Author {
            id: "QSBUZXN0IE5hcnJhdG9y".to_string(),
            name: "A Test Narrator".to_string(),
            description: None,
            num_books: 2,
        };

        // The name in base64. The measurement of the sandbox gave the same value
        // as the identity of the answer, and this function does not depend on it.
        assert_eq!(
            Kind::Narrators.filter_of(&one),
            "narrators.QSBUZXN0IE5hcnJhdG9y"
        );

        let author = Author {
            id: "f49b0437-bb55-450a-8d20-38ad9b6c35ac".to_string(),
            name: "Test Author".to_string(),
            description: None,
            num_books: 1,
        };

        assert_eq!(
            Kind::Authors.filter_of(&author),
            crate::logic::sort_filter::filter_value("authors", &author.id),
            "the filter of an author takes the identity"
        );
        assert!(!Kind::Authors.filter_of(&author).contains("narrators"));
    }

    /// A new list forgets the answer of the list that came before it. The view
    /// must not show the authors under the title of the narrators.
    ///
    /// **The two boxes belong to the process, therefore this test stays in one
    /// function.** See the trap 29 of `docs/HANDOVER.md`.
    #[test]
    fn a_new_list_forgets_the_answer_of_the_list_before_it() {
        keep_the_kind(Kind::Authors);
        keep(State::Ready(vec![Author {
            id: "a".to_string(),
            name: "A Name".to_string(),
            description: None,
            num_books: 1,
        }]));
        assert_eq!(authors().len(), 1);

        keep_the_kind(Kind::Narrators);
        assert_eq!(kind(), Kind::Narrators);
        assert!(
            authors().is_empty(),
            "the answer of the authors must not stand under the narrators"
        );

        // The same list again keeps the answer, therefore the program asks the
        // server one time only.
        keep(State::Ready(vec![Author {
            id: "QSBOYW1l".to_string(),
            name: "A Name".to_string(),
            description: None,
            num_books: 2,
        }]));
        keep_the_kind(Kind::Narrators);
        assert_eq!(authors().len(), 1, "the same list keeps its answer");

        keep_the_kind(Kind::Authors);
        forget();
    }

    /// The state belongs to the process, therefore the parts of this test
    /// must stay in one function.
    #[test]
    fn the_state_goes_from_the_task_to_the_screen() {
        forget();
        assert!(matches!(state(), State::Nothing));
        assert!(authors().is_empty());

        keep(State::Waiting);
        assert!(authors().is_empty());

        keep(State::Ready(vec![Author {
            id: "a".to_string(),
            name: "A Name".to_string(),
            description: None,
            num_books: 3,
        }]));

        assert_eq!(authors().len(), 1);
        assert_eq!(authors()[0].num_books, 3);

        keep(State::Fault("no answer".to_string()));
        assert!(authors().is_empty());

        forget();
    }
}
