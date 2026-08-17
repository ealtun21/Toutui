//! The header names the filter that the user took (T-379).
//!
//! An application of a filter rebuilds the application in the way of the key
//! `R`, and that road calls `from_the_server::forget`: the choices of the
//! server are then gone, and the header of v0.8.209 named the group
//! (`▣ A genre`) and not the name that the user just took. The start of the
//! program holds the same condition, because the choices come at the key `f`
//! alone.
//!
//! The measurement of the real program v0.8.209 inside tmux at 100 columns
//! against the sandbox: the view of the key `f` applied the genre `Fiction`,
//! the Library view held `[1 item]`, and the second row of the header read
//! `⇅ The sequence of the server ▣ A genre`. The control of the same run: a
//! second `f` filled the choices again, and the header then read `▣ Fiction`.
//!
//! The value of a genre, of a tag, of a narrator, of a language, and of a
//! publisher holds the name itself in base64, therefore `decode_base64` gives
//! the name back with no list at all. The value of an author and of a series
//! holds an identity, and `the_name_that_stands` keeps the name of the last
//! application.
//!
//! The functions are pure, therefore this test needs no screen and no server.
//! **The parts of this test stay in one function.**

use toutui::logic::sort_filter::{decode_base64, filter_value, the_name_that_stands};
use toutui::ui::the_panels_of_the_stack::the_name_of_a_filter;

#[test]
fn the_header_names_the_filter_that_the_user_took() {
    // The rule of base64 itself: the road there and back, and the texts that
    // do not obey it.
    assert_eq!(decode_base64("RmljdGlvbg=="), Some("Fiction".to_string()));
    assert_eq!(decode_base64(""), None, "an empty text is no base64");
    assert_eq!(decode_base64("Rmlj dGlvbg=="), None, "a space is no letter");
    assert_eq!(
        decode_base64("Rm=j"),
        None,
        "no letter stands after the full stop"
    );
    assert_eq!(
        decode_base64("Rml"),
        None,
        "a group of base64 holds four places"
    );

    // The name of a genre comes of the value, with no choice of the server at
    // all: this is the condition after an application and at the start.
    assert_eq!(
        the_name_of_a_filter(&filter_value("genres", "Fiction"), &[]),
        "Fiction",
        "the header names the genre that the user took"
    );

    // A name of two lines stands in one line (the rule of T-378).
    assert_eq!(
        the_name_of_a_filter(&filter_value("genres", "Alpha\nOMEGAEND"), &[]),
        "Alpha OMEGAEND",
        "the name of the value stands in one line"
    );

    // The value of an author holds an identity, therefore no arithmetic gives
    // the name back: with no application, the group stands.
    let of_an_author = filter_value("authors", "cc5891d3-f0a5-42b0-ac39-6c33df199efd");

    assert_eq!(
        the_name_of_a_filter(&of_an_author, &[]),
        "An author",
        "an author with no name of an application names the group"
    );

    // The application keeps the name, and the header takes it after the
    // forget of the choices.
    the_name_that_stands::keep(&of_an_author, "Lewis\nCarroll");

    assert_eq!(
        the_name_of_a_filter(&of_an_author, &[]),
        "Lewis Carroll",
        "the header names the author of the last application, in one line"
    );

    // The name of the box belongs to its value alone: another filter of an
    // identity names its group.
    assert_eq!(
        the_name_of_a_filter(&filter_value("series", "an-identity"), &[]),
        "A series",
        "the name of the last application reaches no other value"
    );
}
