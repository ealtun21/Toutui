//! The start asks the server for the name of a filter of an old row. See
//! T-381.
//!
//! **The parts of this test stay in one function.** The rule of the box of
//! the process holds for every test of this repository (T-144 and T-157).
//!
//! # The condition
//!
//! A row that a database of the version 10 or before wrote holds the value of
//! the filter and no name: the migration to the version 11 adds the column
//! `library_filter_name` with an empty text (T-380), and the write of the
//! name stands behind a key. The value of an author and of a series holds an
//! identity, therefore no arithmetic gives the name back. The measurement of
//! v0.8.211 inside tmux, with the filter of the author Lewis Carroll and an
//! empty name in the row of the account, gave the header
//! `⇅ The sequence of the server ▣ An author` at the start — the group, and
//! not the name — at every start, for ever.
//!
//! # The correction
//!
//! The start asks the server for the choices of the filter one time, when
//! `the_filter_needs_a_name_of_the_server` says so: the value stands, the
//! name of the row is empty, and the kind is an author or a series. The name
//! comes of `the_name_out_of_the_choices`, the box and the row of the account
//! keep it, and the next start reads the row alone.

use toutui::logic::sort_filter::{
    the_filter_needs_a_name_of_the_server, the_name_out_of_the_choices, FilterChoice,
};

#[test]
fn the_name_of_an_old_filter_comes_of_the_server() {
    // The heal fires for an author and for a series whose name is empty. The
    // build of the fault — the predicate disabled — fails here.
    assert!(the_filter_needs_a_name_of_the_server(
        "authors.MzEyYzQyZmYtZTgwMC00YjI5LTk5NzQtZDJkODk5ZDBiYmE5",
        ""
    ));
    assert!(the_filter_needs_a_name_of_the_server(
        "series.OGE1ZGNlNzgtYzgyMy00NDFlLWE5OTgtZWJhOWY5ZThkMDZi",
        ""
    ));

    // A row that holds the name already needs no request.
    assert!(!the_filter_needs_a_name_of_the_server(
        "authors.MzEyYzQyZmYtZTgwMC00YjI5LTk5NzQtZDJkODk5ZDBiYmE5",
        "Lewis Carroll"
    ));

    // The five other kinds hold the name itself in base64, and the header
    // decodes it (T-379): they need no request.
    assert!(!the_filter_needs_a_name_of_the_server(
        "genres.RmljdGlvbg==",
        ""
    ));

    // A row of no filter needs no request, and a value of a group with no
    // identity is not a filter of an author.
    assert!(!the_filter_needs_a_name_of_the_server("", ""));
    assert!(!the_filter_needs_a_name_of_the_server("authors", ""));

    // The name comes of the choice whose value is the value of the row.
    let choices = vec![
        FilterChoice {
            label: "Lewis Carroll".to_string(),
            group: "The authors",
            value: "authors.MzEyYzQyZmYtZTgwMC00YjI5LTk5NzQtZDJkODk5ZDBiYmE5".to_string(),
        },
        FilterChoice {
            label: "The Test Chronicles".to_string(),
            group: "The series",
            value: "series.OGE1ZGNlNzgtYzgyMy00NDFlLWE5OTgtZWJhOWY5ZThkMDZi".to_string(),
        },
    ];
    assert_eq!(
        the_name_out_of_the_choices(
            &choices,
            "authors.MzEyYzQyZmYtZTgwMC00YjI5LTk5NzQtZDJkODk5ZDBiYmE5"
        ),
        Some("Lewis Carroll".to_string())
    );
    assert_eq!(
        the_name_out_of_the_choices(
            &choices,
            "series.OGE1ZGNlNzgtYzgyMy00NDFlLWE5OTgtZWJhOWY5ZThkMDZi"
        ),
        Some("The Test Chronicles".to_string())
    );

    // A filter whose author the server no longer holds gives no name: the
    // header names the group, and the log says why.
    assert_eq!(
        the_name_out_of_the_choices(&choices, "authors.QW4gYXV0aG9yIHRoYXQgd2VudCBhd2F5"),
        None
    );
}
