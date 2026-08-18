//! **A write of the sequence in a library where the filter stays suppressed
//! must not erase the filter of the row** (T-384). The start of v0.8.214
//! keeps a filter of another library (T-383) and a filter that a library of
//! podcasts ignores (T-382) out of the request and out of the header, and
//! the row of the disk keeps it. The write funnel wrote the filter of the
//! application, and that filter is empty in such a library: one Enter on a
//! row of the sequence then erased the filter, its name, and its library
//! together, and no library gave it back.
//!
//! The parts of this test stay in one function.

use toutui::logic::sort_filter::the_filter_of_the_row_stays;

#[test]
fn a_sequence_of_another_library_keeps_the_filter() {
    let author = "authors.MzEyYzQyZmYtZTgwMC00YjI5LTk5NzQtZDJkODk5ZDBiYmE5";
    let series = "series.YzE0YzYxYzM";
    let genre = "genres.RmFpcnkgVGFsZXM=";

    // The filter of an identity of another library stays suppressed in this
    // library (T-383): the write of the sequence keeps it in the row.
    for filter in [author, series] {
        assert!(
            the_filter_of_the_row_stays(filter, "lib-a", "lib-b", false),
            "the filter {} of another library stays in the row",
            filter
        );
    }

    // A filter that acts in this library is not suppressed: an empty filter
    // of the application then came of the key of the user, and the write
    // erases the row.
    assert!(
        !the_filter_of_the_row_stays(author, "lib-a", "lib-a", false),
        "a filter that acts here goes with the key of the user"
    );

    // A filter that a library of podcasts ignores stays suppressed there
    // (T-382): the write of the sequence keeps it in the row, and the
    // library of books gives it back.
    for filter in [author, series, "progress.ZmluaXNoZWQ="] {
        assert!(
            the_filter_of_the_row_stays(filter, "lib-a", "lib-a", true),
            "the filter {} that the library of podcasts ignores stays",
            filter
        );
    }

    // A filter of a name rides into another library with its meaning: it
    // acts there, and an empty filter of the application came of the key of
    // the user.
    assert!(
        !the_filter_of_the_row_stays(genre, "lib-a", "lib-b", false),
        "a filter of a name acts in every library of books"
    );

    // A filter of an identity of a row that holds no library acts, as it
    // did before the version 12 of the database.
    assert!(!the_filter_of_the_row_stays(author, "", "lib-b", false));

    // A row of no filter holds nothing to keep.
    assert!(!the_filter_of_the_row_stays("", "lib-a", "lib-b", false));
    assert!(!the_filter_of_the_row_stays("", "", "lib-b", true));
}
