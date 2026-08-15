//! T-264. **The user sees no word of a value of the configuration file that the
//! program does not use.**
//!
//! The read of the file takes a value of the user away for two reasons: the
//! program cannot read it (T-258 and T-259), or a rule of the program refuses
//! it (T-260 to T-263). Each of the two took a line of the log alone. **The log
//! is the one word of a fault that no view of the user holds** (T-177), and this
//! fault holds a view: the user wrote that file, the user can correct it, and
//! the screen stands in front of them at the start of the program and at the key
//! `R`.
//!
//! The measurement, of the real program v0.8.92 inside tmux, on a screen of 160
//! columns and 45 rows. A configuration file of the sandbox with
//! `background_color = [40, 40]`, with a block `[[servers]]` of `name = ""`, and
//! with `ebook_cache_mb = "not a number"`, gave a screen with no word of the
//! configuration at all: `grep -icE "config|colour|value|file"` of the whole
//! capture of tmux gave **0**, and the three warnings stood in the log alone.
//!
//! **The parts of this test stay in one function.** The box of the message is a
//! slot of a module, therefore two test functions of one binary fight for it
//! (the shape of T-144 and of T-157).

use toutui::app::AppView;
use toutui::config::{load_config_from, say_the_values_that_the_program_does_not_use};
use toutui::logic::message;

#[test]
fn the_row_of_the_message_names_the_values_of_the_file_that_the_program_does_not_use() {
    let place = tempfile::tempdir().expect("a directory of a test");

    // A file whose every value the program uses. **A message of no fault hides
    // the answer of a key for six seconds**, therefore the screen must hold no
    // row of a message at all.
    let of_no_fault = place.path().join("of_no_fault.toml");
    std::fs::write(
        &of_no_fault,
        "[colors]\nbackground_color = [40, 40, 40]\n[reader]\nebook_cache_mb = 512\n",
    )
    .expect("the file of the test");

    message::forget();
    let config = load_config_from(&of_no_fault).expect("the program must start");
    say_the_values_that_the_program_does_not_use(&config);
    assert_eq!(
        message::for_the_screen(AppView::Home),
        None,
        "a file of no fault must say nothing at all"
    );

    // The file of the measurement: a colour of two numbers, a server of a name
    // of no character, and a value of the reader that the program cannot read.
    let of_three_faults = place.path().join("of_three_faults.toml");
    std::fs::write(
        &of_three_faults,
        "[colors]\nbackground_color = [40, 40]\n\
         [reader]\nebook_cache_mb = \"not a number\"\n\
         [[servers]]\nname = \"\"\n\
         endpoints = [ { url = \"http://one.example.com\", priority = 0 } ]\n",
    )
    .expect("the file of the test");

    message::forget();
    let config = load_config_from(&of_three_faults).expect("the program must start");
    assert_eq!(
        config.the_values_that_the_program_does_not_use.len(),
        3,
        "the three values of the user must each name themselves"
    );

    say_the_values_that_the_program_does_not_use(&config);
    assert_eq!(
        message::for_the_screen(AppView::Home),
        Some(
            "The program does not use 3 values of the configuration file. \
             The log names each of them."
                .to_string()
        ),
        "the user must read the number of the values on the screen"
    );

    // The message belongs to no view: the user can stand anywhere at the start
    // of the program, and the answer of the key `R` comes at once. See T-164.
    assert!(
        message::for_the_screen(AppView::Library).is_some(),
        "the message must stand above every view"
    );

    // A file of one fault takes no plural: the shape `1 value(s)` is no sentence
    // of a person.
    let of_one_fault = place.path().join("of_one_fault.toml");
    std::fs::write(&of_one_fault, "[colors]\nbackground_color = [40, 40]\n")
        .expect("the file of the test");

    message::forget();
    let config = load_config_from(&of_one_fault).expect("the program must start");
    say_the_values_that_the_program_does_not_use(&config);
    assert_eq!(
        message::for_the_screen(AppView::Home),
        Some(
            "The program does not use 1 value of the configuration file. The log names it."
                .to_string()
        ),
        "one value must take no plural"
    );

    message::forget();
}
