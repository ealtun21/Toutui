pub mod search_active;

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
        /// The identity of every media that the server found.
        pub items: Vec<String>,
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
