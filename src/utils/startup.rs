//! What the program does while it starts. See T-40.
//!
//! `App::new` asks the server many times before the first screen: the
//! libraries, the list "Continue Listening", the position of each book of that
//! list, the series, the items, and the lists. A slow server therefore gave a
//! black screen for a long time, and a server that answers nothing gave a
//! black screen for the whole time of the timeout.
//!
//! The program draws a screen at once now, and that screen names the step. The
//! step lives here, because `App::new` writes it and the loop of the loading
//! screen reads it.

use std::sync::{Mutex, OnceLock};

fn box_of_the_step() -> &'static Mutex<String> {
    static STEP: OnceLock<Mutex<String>> = OnceLock::new();
    STEP.get_or_init(|| Mutex::new(String::from("the program starts")))
}

/// Names the step that runs now. The loading screen shows this text.
pub fn set(step: impl Into<String>) {
    if let Ok(mut value) = box_of_the_step().lock() {
        *value = step.into();
    }
}

/// Gives the name of the step that runs now.
pub fn step() -> String {
    box_of_the_step()
        .lock()
        .map(|value| value.clone())
        .unwrap_or_else(|_| String::from("the program starts"))
}

/// Names a step that has a number of parts, for example the position of each
/// book of the list "Continue Listening".
pub fn set_part(step: &str, done: usize, total: usize) {
    if total <= 1 {
        set(step);
    } else {
        set(format!("{} ({} of {})", step, done, total));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_step_of_many_parts_gives_the_count() {
        set_part("the position of each book", 3, 12);
        assert_eq!(step(), "the position of each book (3 of 12)");
    }

    #[test]
    fn one_part_gives_no_count() {
        set_part("the libraries", 1, 1);
        assert_eq!(step(), "the libraries");
    }
}
