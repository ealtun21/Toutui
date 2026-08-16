//! A message of the login screen that is longer than the screen stands on more
//! than one row. See T-297, and the same rule of a view in T-278.
//!
//! **The measurement of 2026-08-16, of the real program v0.8.125 inside tmux
//! against the sandbox**: the keys `S`, `Enter`, `l`, and `l` logged out of the
//! one account, the program started again by itself, and the login screen of it
//! said, in a terminal of **160** columns:
//!
//! ```text
//! The program removed the account toutuitest. The disk keeps the copies of that account: 11 media, and 239.7 MB. Log in again with the same name and the same serv
//! ```
//!
//! The road back of that sentence stood outside the screen. The row of the
//! message of that screen held one row and no `wrap`, therefore every sentence
//! of more than the width of the terminal lost its end, and a terminal of 80
//! columns loses much more.
//!
//! The render of that screen needs a frame and no terminal at all: these tests
//! draw it into a `Buffer` and they read the characters of that buffer.

use ratatui::backend::TestBackend;
use ratatui::Terminal;
use toutui::logic::auth::auth_input::{
    draw_the_login, the_row_of_the_message, the_rows_of_the_message, TheLoginScreen,
};

/// The sentence of the measurement above, of a log out of the sandbox.
const THE_WORDS_OF_THE_LOG_OUT: &str =
    "The program removed the account toutuitest. The disk keeps the copies of that account: \
     11 media, and 239.7 MB. Log in again with the same name and the same server: the key X \
     then removes a copy.";

/// The words of a screen, with no line and no colour.
///
/// The wrap breaks a sentence at a space, therefore this function makes one
/// space of every run of spaces and of every end of a row: a sentence of three
/// rows then reads as one sentence.
fn the_words_of(width: u16, height: u16, message: &str) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();

    terminal
        .draw(|frame| {
            draw_the_login(
                frame,
                &TheLoginScreen {
                    title: "Server address",
                    text: "http://localhost:13399",
                    the_text_tells_what_to_write: false,
                    scroll: 0,
                    cursor: 0,
                    message,
                    of_the_border: (255, 255, 255),
                    of_the_background: (0, 0, 0),
                    of_the_message: (0, 0, 0),
                },
            );
        })
        .unwrap();

    let buffer = terminal.backend().buffer().clone();
    let mut words = String::new();

    for row in 0..buffer.area.height {
        for column in 0..buffer.area.width {
            words.push_str(buffer[(column, row)].symbol());
        }
        words.push(' ');
    }

    words.split_whitespace().collect::<Vec<&str>>().join(" ")
}

#[test]
fn the_message_of_the_login_screen_stands_whole() {
    // **The whole sentence stands at 80 columns.** The old render cut it at the
    // width of the screen, and the road back of the user went away.
    let at_eighty = the_words_of(80, 30, THE_WORDS_OF_THE_LOG_OUT);

    assert!(
        at_eighty.contains(THE_WORDS_OF_THE_LOG_OUT),
        "the sentence of the log out stands whole at 80 columns: {}",
        at_eighty
    );

    // The measurement above stood at 160 columns, and the sentence lost its end
    // there too.
    let at_one_sixty = the_words_of(160, 45, THE_WORDS_OF_THE_LOG_OUT);

    assert!(
        at_one_sixty.contains(THE_WORDS_OF_THE_LOG_OUT),
        "the sentence of the log out stands whole at 160 columns: {}",
        at_one_sixty
    );

    // **The field of the login keeps its place.** The rows of the message grow
    // upward, therefore the last row of it stays where one row stood.
    assert!(
        at_eighty.contains("Server address"),
        "the field of the login stays: {}",
        at_eighty
    );

    // A message of one row keeps the row that it had.
    let short = the_words_of(80, 30, "The token is not valid. Log in again.");
    assert!(short.contains("The token is not valid. Log in again."));

    // A screen with no message writes no word of a message at all.
    let none = the_words_of(80, 30, "");
    assert!(!none.contains("token"), "{}", none);
}

#[test]
fn the_rows_of_a_message_follow_the_width_of_the_screen() {
    // The function is pure, therefore no terminal stands behind these numbers.
    assert_eq!(the_rows_of_the_message("", 80), 0);
    assert_eq!(the_rows_of_the_message("   ", 80), 0);
    assert_eq!(the_rows_of_the_message("a word", 0), 0);

    assert_eq!(the_rows_of_the_message("a word", 80), 1);
    assert_eq!(the_rows_of_the_message("aaa bbb ccc", 7), 2);

    // A word that is longer than the width takes rows of its own.
    assert_eq!(the_rows_of_the_message("aaaaaaaaaa", 4), 3);

    // The sentence of the measurement holds 194 characters.
    assert_eq!(THE_WORDS_OF_THE_LOG_OUT.chars().count(), 194);
    assert_eq!(the_rows_of_the_message(THE_WORDS_OF_THE_LOG_OUT, 160), 2);
    assert_eq!(the_rows_of_the_message(THE_WORDS_OF_THE_LOG_OUT, 80), 3);

    // The last row of the message stays where one row of a message stood.
    let size = ratatui::layout::Size::new(80, 30);
    assert_eq!(the_row_of_the_message(size), 24);
}
