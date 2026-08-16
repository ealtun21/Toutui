//! The words of a key that refreshes the program reach the user. See T-308.
//!
//! **A `say` of a key that writes `must_refresh` reaches nobody at all.** The
//! key writes the words into the box of the message, and it then asks the loop
//! of `src/main.rs` for a new application; that road makes the new `App` and it
//! calls `crate::logic::message::forget()` before the first frame of it. The
//! words go away before the screen holds one byte of them. That is the shape of
//! the trap 234 of the key `R` and of T-264, and two keys of this program held
//! it: the key of the next library and the key `l` of the view
//! `Settings Library`.
//!
//! The measurement of 2026-08-16, of the real program v0.8.136 inside tmux
//! against the sandbox on `:13399`, with the account `toutuitest` and a
//! terminal of 40 columns and 45 rows. **The data of this fault is the program
//! itself**: it needs no proxy, no book of a harness, and no change of the
//! sandbox at all.
//!
//! The keys `S`, `j`, `l`, and `l` of the Home view took the library
//! `Podcasts`. The header of the screen changed to `Podcasts (podcas`, the
//! Library view of it stood with its two items, and **no frame of the six
//! seconds after that key held one character of** `The program shows the
//! library "Podcasts" now.` — 40 captures of `tmux capture-pane`, of a step of
//! 0.15 seconds. The key of the next library (`BTab`) of the Home view gave the
//! same measurement: the header changed to `Books (book)`, and no frame held
//! the sentence.
//!
//! **The control of the same run**: `chmod 444` of the database of the program
//! (T-206) after the first frame, and the same keys `S`, `j`, `l`, and `l`. The
//! write of the row of the library then fails, that road **returns** in the
//! place of writing `must_refresh`, and its `say` reached the screen at once:
//!
//! ```text
//! The program did not write the library of
//!    this account: the database did not
//!    answer. A different program of this
//!  account can hold it. Press Enter again.
//! ```
//!
//! It stood in 25 of 25 captures. The two roads hold one key, one view, and one
//! `say`; the road that refreshes the program is the one that says nothing.
//!
//! The gate holds two rules.
//!
//! 1. The box of the message keeps the words of such a key over a `forget`, and
//!    it gives them to the user after it.
//! 2. **No function of `src/app.rs` writes `must_refresh` and says a message
//!    together**, and `src/main.rs` gives those words after its last `forget`.
//!    The first rule alone passes for a program that writes `say` at the key
//!    again, therefore the source holds the second one.

use std::fs;
use std::path::Path;

use toutui::app::AppView;
use toutui::logic::message::{
    for_the_screen, forget, forget_the_words_of_the_refresh, say, say_after_the_refresh,
    the_words_of_the_refresh_come,
};

/// The words of the measurement.
const THE_WORDS: &str = "The program shows the library \"Podcasts\" now.";

/// The box of the message belongs to the process, therefore the parts of this
/// test stay in one function. Two test functions would fight for the slot, and
/// `cargo test` gives each test of one binary a thread (T-144 and T-157).
#[test]
fn the_box_of_the_message_keeps_the_words_of_a_key_that_refreshes() {
    forget();
    forget_the_words_of_the_refresh();

    // **The fault.** A `say` of such a key does not stand after the `forget` of
    // the refresh.
    say(THE_WORDS);
    assert_eq!(for_the_screen(AppView::Home), Some(THE_WORDS.to_string()));
    forget();
    assert_eq!(
        for_the_screen(AppView::Home),
        None,
        "the `forget` of the refresh takes away a message that a key said"
    );

    // **The correction.** The words wait outside `forget`, and the loop gives
    // them to the user when the new screen stands.
    say_after_the_refresh(THE_WORDS);
    forget();
    assert_eq!(
        for_the_screen(AppView::Home),
        None,
        "the words wait for the new screen, therefore no frame of the refresh \
         holds them"
    );
    the_words_of_the_refresh_come();
    assert_eq!(
        for_the_screen(AppView::Home),
        Some(THE_WORDS.to_string()),
        "the words of the key must reach the user after the refresh"
    );

    // One key gives one message: the slot is empty after it, therefore the
    // refresh of the key after this one says nothing of an older key.
    forget();
    the_words_of_the_refresh_come();
    assert_eq!(for_the_screen(AppView::Home), None);

    // **A screen that did not change makes those words a lie** (T-205 and
    // T-266): the road that keeps the application of the user drops them.
    say_after_the_refresh(THE_WORDS);
    forget_the_words_of_the_refresh();
    the_words_of_the_refresh_come();
    assert_eq!(for_the_screen(AppView::Home), None);

    // A text of no character is no message.
    say_after_the_refresh("   ");
    the_words_of_the_refresh_come();
    assert_eq!(for_the_screen(AppView::Home), None);

    forget();
}

/// No function of `src/app.rs` writes `must_refresh` and says a message
/// together, and `src/main.rs` gives the words after its last `forget`.
#[test]
fn the_source_keeps_the_words_of_a_refresh_away_from_say() {
    let of_the_application = fs::read_to_string(Path::new("src/app.rs")).expect("src/app.rs");
    let lines: Vec<&str> = of_the_application.lines().collect();

    // Every function of `impl App` stands at four spaces of indent.
    let the_start_of_a_function = |line: &str| {
        let words = line.trim_start();
        line.len() - words.len() == 4
            && (words.starts_with("fn ")
                || words.starts_with("pub fn ")
                || words.starts_with("pub(crate) fn ")
                || words.starts_with("async fn ")
                || words.starts_with("pub async fn "))
    };

    let mut the_marks = 0;

    for (number, line) in lines.iter().enumerate() {
        if !line.contains("must_refresh = true") {
            continue;
        }

        the_marks += 1;

        let start = (0..=number)
            .rev()
            .find(|place| the_start_of_a_function(lines[*place]))
            .unwrap_or(0);

        let end = ((number + 1)..lines.len())
            .find(|place| the_start_of_a_function(lines[*place]))
            .unwrap_or(lines.len());

        let name = lines[start].trim();

        for one in &lines[start..end] {
            assert!(
                !one.contains("message::say("),
                "src/app.rs:{}: the function `{}` writes `must_refresh` and it says a \
                 message: the loop of src/main.rs makes a new application for that mark, \
                 and its `forget` takes the message away before the first frame. Say the \
                 words with `message::say_after_the_refresh`. See T-308.",
                number + 1,
                name
            );
        }
    }

    assert!(
        the_marks >= 3,
        "src/app.rs holds no mark of a refresh: this gate then measures nothing"
    );

    let of_the_loop = fs::read_to_string(Path::new("src/main.rs")).expect("src/main.rs");
    let lines: Vec<&str> = of_the_loop.lines().collect();

    let the_last_forget = lines
        .iter()
        .rposition(|line| line.contains("message::forget();"))
        .expect("src/main.rs calls `logic::message::forget()`");

    let the_words = lines
        .iter()
        .rposition(|line| line.contains("the_words_of_the_refresh_come()"))
        .expect("src/main.rs must give the words of a key that refreshes the program. See T-308.");

    assert!(
        the_words > the_last_forget,
        "src/main.rs:{}: the loop says the words of the refresh before its `forget` of \
         src/main.rs:{}, therefore that `forget` takes them away. See T-308.",
        the_words + 1,
        the_last_forget + 1
    );
}
