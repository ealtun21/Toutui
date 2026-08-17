//! **The server ignores a filter of an author, of a series, of a narrator,
//! of a publisher, and of the position in a library of podcasts, and it
//! answers every item** (T-382). The program of v0.8.212 sent such a filter
//! and it then named it in the header: `▣ Lewis Carroll` stood over the two
//! podcasts of the library, and `▣ Finished` stood over two podcasts that
//! no one finished. A view must not say a state that the program does not
//! have (T-91), therefore the start clears such a filter for the request
//! and for the header, and the view of the sequence and of the filter of a
//! library of podcasts offers no choice of the position.
//!
//! The parts of this test stay in one function: two test functions of one
//! module fight for the slot of that module (T-144).

use toutui::logic::sort_filter::{is_a_filter_of_the_library, rows, Row};

#[test]
fn a_filter_that_the_server_ignores_stays_out_of_a_library_of_podcasts() {
    // The five kinds that the server ignores in a library of podcasts.
    for filter in [
        "authors.MzEyYzQyZmYtZTgwMC00YjI5LTk5NzQtZDJkODk5ZDBiYmE5",
        "series.YzE0YzYxYzM",
        "narrators.U29tZSBOYXJyYXRvcg==",
        "publishers.U29tZSBQdWJsaXNoZXI=",
        "progress.ZmluaXNoZWQ=",
    ] {
        assert!(
            !is_a_filter_of_the_library(filter, true),
            "a library of podcasts must not take the filter {}",
            filter
        );

        // A library of books takes every kind.
        assert!(
            is_a_filter_of_the_library(filter, false),
            "a library of books takes the filter {}",
            filter
        );
    }

    // A genre, a tag, and a language act in the two kinds of a library.
    for filter in [
        "genres.RmFpcnkgVGFsZXM=",
        "tags.YS10ZXN0LXRhZw==",
        "languages.ZW4=",
    ] {
        assert!(is_a_filter_of_the_library(filter, true));
        assert!(is_a_filter_of_the_library(filter, false));
    }

    // A filter of no value is no filter, in the two kinds of a library.
    assert!(is_a_filter_of_the_library("", true));
    assert!(is_a_filter_of_the_library("", false));

    // The view of a library of podcasts offers no choice of the position,
    // and the view of a library of books keeps the three of them.
    let of_the_podcasts = rows(true, &[], None);

    assert!(
        !of_the_podcasts.iter().any(|row| matches!(
            row,
            Row::Filter { value, .. } if value.starts_with("progress.")
        )),
        "the view of a library of podcasts must offer no choice of the position"
    );

    assert!(
        !of_the_podcasts
            .iter()
            .any(|row| *row == Row::Title("Your position".to_string())),
        "the view of a library of podcasts must not name the group of the position"
    );

    let of_the_books = rows(false, &[], None);

    assert_eq!(
        of_the_books
            .iter()
            .filter(|row| matches!(
                row,
                Row::Filter { value, .. } if value.starts_with("progress.")
            ))
            .count(),
        3,
        "the view of a library of books keeps the three choices of the position"
    );
}
