//! The words of a log out reach the program that starts after it. See T-298.
//!
//! **The measurement of 2026-08-16, of the real program v0.8.126 inside tmux
//! against the sandbox.** The database of the program held `toutuitest` and
//! `toutuilimited`, and `toutuilimited` was the account of the start. The keys
//! `S`, `Enter`, `j`, `l`, and `l` logged out of it:
//!
//! - the log said `[the accounts] the log out of toutuilimited took 1 row(s) of
//!   the account and 0 place(s) of the user.`, and after it `[the accounts] the
//!   account toutuitest starts the program. The program starts again.`;
//! - the program started again with `exec`, and the Home view of `toutuitest`
//!   came. **Six polls of the whole screen, one every 0.8 seconds, found no word
//!   of that log out at all**: the row of the message went away with the process.
//!
//! T-297 gave those words two roads of the three: the login screen takes them
//! over the disk, and the view that stays takes them in the row of the message.
//! **The third road holds an account, therefore it draws no login screen and the
//! disk says nothing to it.** The environment of the new process carries them,
//! and the loop of `src/main.rs` says them before the first frame.
//!
//! The corrected program, of the same keys, drew
//! `The program removed the account toutuilimited.` on the row 43 of the Home
//! view of `toutuitest`, at every one of those six polls. The build of the fault
//! — one edit, `start_the_program_with_this_account(&name, "")` — said nothing at
//! any of them.

use toutui::db::crud::the_words_of_a_log_out;
use toutui::logic::download::TheCopiesThatStay;
use toutui::logic::message::{the_words_of_the_start, THE_WORDS_OF_THE_START};
use toutui::logic::the_accounts::{
    the_variables_of_a_start, the_work_of_a_log_out, AfterALogOut, TheWorkOfALogOut,
};
use toutui::utils::exit_app::the_environment_of_a_start;

/// Every road of a log out carries the words of that log out.
#[test]
fn the_three_roads_of_a_log_out_carry_the_words() {
    let words = "The program removed the account toutuilimited.";

    // The road of the measurement: the account of the start went away, and
    // another account takes that work.
    assert_eq!(
        the_work_of_a_log_out(
            AfterALogOut::ThisAccountStarts("toutuitest".to_string()),
            words
        ),
        TheWorkOfALogOut::ThisAccountStarts {
            name: "toutuitest".to_string(),
            the_words: words.to_string(),
        }
    );

    // The two roads of T-297 keep the words that they had.
    assert_eq!(
        the_work_of_a_log_out(AfterALogOut::TheLoginScreen, words),
        TheWorkOfALogOut::TheLoginScreen {
            the_words: words.to_string(),
        }
    );

    assert_eq!(
        the_work_of_a_log_out(AfterALogOut::TheViewOnly, words),
        TheWorkOfALogOut::TheViewStays {
            the_words: words.to_string(),
        }
    );
}

/// The variable of the environment carries the words over the `exec`, and the
/// program after it reads them again.
#[test]
fn the_words_go_over_the_start_of_the_program_again() {
    let words = "The program removed the account toutuilimited.";

    let variables = the_variables_of_a_start(words);

    assert_eq!(
        variables,
        vec![(THE_WORDS_OF_THE_START.to_string(), words.to_string())]
    );

    // The program after the `exec` reads that value.
    assert_eq!(
        the_words_of_the_start(Some(variables[0].1.as_str())),
        Some(words)
    );

    // The whole road, from the words of the log out to the screen of the new
    // program.
    let of_the_log_out = the_words_of_a_log_out(
        "toutuilimited",
        1,
        TheCopiesThatStay {
            media: 11,
            bytes: 251_382_273,
        },
    );

    let of_the_start = the_variables_of_a_start(&of_the_log_out);

    assert_eq!(
        the_words_of_the_start(Some(of_the_start[0].1.as_str())),
        Some(of_the_log_out.as_str())
    );

    // A start of the key `c` says nothing: the header of the new program names
    // the account already.
    assert!(the_variables_of_a_start("").is_empty());
    assert!(the_variables_of_a_start("   ").is_empty());
}

/// A start that carries no words writes the variable empty, therefore no
/// sentence of an older process stays.
#[test]
fn a_start_of_no_words_takes_the_words_of_the_process_before_it_away() {
    // A start with no variable at all still writes this one.
    assert_eq!(
        the_environment_of_a_start(&[]),
        vec![(THE_WORDS_OF_THE_START, "")]
    );

    // The address of the login screen of T-123 stands beside it, and it does not
    // take its place.
    let of_the_login = the_environment_of_a_start(&[(
        "TOUTUI_THE_ADDRESS_OF_THE_LOGIN",
        "http://localhost:13399",
    )]);

    assert_eq!(
        of_the_login,
        vec![
            (THE_WORDS_OF_THE_START, ""),
            ("TOUTUI_THE_ADDRESS_OF_THE_LOGIN", "http://localhost:13399"),
        ]
    );

    // A caller that names the words wins, and the variable stands one time.
    let of_a_log_out = the_environment_of_a_start(&[(THE_WORDS_OF_THE_START, "The words.")]);

    assert_eq!(of_a_log_out, vec![(THE_WORDS_OF_THE_START, "The words.")]);

    // A value of no letter is no sentence: the program after the `exec` reads
    // nothing.
    assert_eq!(the_words_of_the_start(Some("")), None);
    assert_eq!(the_words_of_the_start(Some("   ")), None);
    assert_eq!(the_words_of_the_start(None), None);
}
