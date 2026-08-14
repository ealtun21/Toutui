//! The requests of the start that came back with a fault. See T-170.
//!
//! **The start of a library makes four requests**: the shelves of the Home
//! view, the series, the collections and the playlists, and the first page of
//! the items. Each of them held an `unwrap_or_else` with a `warn!` and no word
//! for the user, therefore a fault of one of them gave the user an empty view
//! with a reason of its own: "This library has no series." for a library of
//! series, and "This library holds no media." for a library of 17 books.
//!
//! **`is_offline` of `App` does not hold this condition** (T-25): it holds the
//! offline mode of the **start**, and the server of these faults answers. This
//! is the cause of T-168 and of T-169 too.
//!
//! `logic::the_lists` holds the box of the collections and of the playlists
//! (T-169), because the two views of the lists read it. This module holds the
//! other three.
//!
//! The shape is the shape of every box of this program: a task writes, and the
//! render reads at the next frame.

use std::sync::{Mutex, OnceLock};

/// The request of the start that this fault belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheRequest {
    /// The shelves of the Home view.
    Shelves,
    /// The series of the library.
    Series,
    /// The first page of the items of the library.
    Items,
}

/// What the server said of the requests of the start of one library.
#[derive(Debug, Clone, Default)]
struct TheFaults {
    library: String,
    shelves: Option<String>,
    series: Option<String>,
    items: Option<String>,
}

fn the_faults_that_wait() -> &'static Mutex<Option<TheFaults>> {
    static FAULTS: OnceLock<Mutex<Option<TheFaults>>> = OnceLock::new();
    FAULTS.get_or_init(|| Mutex::new(None))
}

/// Writes that a request of the start of one library came back with a fault.
///
/// **The box holds one library.** The key `S` gives the program a new library,
/// and the fault of the library before it is not the truth of this one.
pub fn keep_the_fault(library: &str, which: TheRequest, what_the_server_said: &str) {
    let Ok(mut slot) = the_faults_that_wait().lock() else {
        return;
    };

    let faults = match slot.as_mut() {
        Some(faults) if faults.library == library => faults,
        _ => slot.insert(TheFaults {
            library: library.to_string(),
            ..TheFaults::default()
        }),
    };

    let text = Some(what_the_server_said.to_string());

    match which {
        TheRequest::Shelves => faults.shelves = text,
        TheRequest::Series => faults.series = text,
        TheRequest::Items => faults.items = text,
    }
}

/// Gives what the server said of one request of the start of this library, and
/// `None` for a request that holds no fault.
pub fn the_fault_of(library: &str, which: TheRequest) -> Option<String> {
    let slot = the_faults_that_wait().lock().ok()?;
    let faults = slot.as_ref().filter(|faults| faults.library == library)?;

    match which {
        TheRequest::Shelves => faults.shelves.clone(),
        TheRequest::Series => faults.series.clone(),
        TheRequest::Items => faults.items.clone(),
    }
}

/// Takes the faults of one library away. The task of the start calls this at
/// its first line: **a new request takes the fault of the request before it
/// away**.
pub fn forget_the_faults_of(library: &str) {
    if let Ok(mut slot) = the_faults_that_wait().lock() {
        if slot
            .as_ref()
            .is_some_and(|faults| faults.library == library)
        {
            *slot = None;
        }
    }
}

/// Forgets every fault. A test starts from a known condition.
pub fn forget() {
    if let Ok(mut slot) = the_faults_that_wait().lock() {
        *slot = None;
    }
}

/// The text of the view of the series, while that view holds no line.
///
/// **A view must not give a reason that the program does not have** (T-91). The
/// view said "This library has no series." for a library of series, because the
/// request of the series came back with the status 500 and the program had no
/// word for that. See T-170.
///
/// The function is pure, therefore a test needs no server.
pub fn the_reason_of_no_series(is_offline: bool, what_the_server_said: Option<&str>) -> String {
    if let Some(fault) = what_the_server_said {
        return format!(
            "The server did not give the series of this library: {}\n\
             Press h to go back.",
            fault
        );
    }

    if is_offline {
        return "The server gave no series: the server does not answer.\nPress h to go back."
            .to_string();
    }

    "This library has no series.\nPress h to go back.".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The box holds the faults of one library, and one request of it.
    ///
    /// **The parts of this test stay in one function**: two test functions of
    /// one module fight for the boxes of the process (T-144 and T-157).
    #[test]
    fn the_box_holds_the_faults_of_one_library() {
        forget();

        assert_eq!(the_fault_of("books", TheRequest::Series), None);

        keep_the_fault("books", TheRequest::Series, "Status 500.");
        keep_the_fault("books", TheRequest::Items, "Status 503.");

        assert_eq!(
            the_fault_of("books", TheRequest::Series).as_deref(),
            Some("Status 500.")
        );
        assert_eq!(
            the_fault_of("books", TheRequest::Items).as_deref(),
            Some("Status 503.")
        );

        // The request of the shelves came back with no fault.
        assert_eq!(the_fault_of("books", TheRequest::Shelves), None);

        // **A user who takes the key `S` to another library must not read the
        // fault of the library before it.**
        assert_eq!(the_fault_of("empty", TheRequest::Series), None);

        // A library of its own takes the box, and the faults of the library
        // before it go away with it.
        keep_the_fault("empty", TheRequest::Shelves, "Status 500.");
        assert_eq!(the_fault_of("books", TheRequest::Series), None);
        assert_eq!(
            the_fault_of("empty", TheRequest::Shelves).as_deref(),
            Some("Status 500.")
        );

        // A new request of that library takes its faults away.
        forget_the_faults_of("books");
        assert_eq!(
            the_fault_of("empty", TheRequest::Shelves).as_deref(),
            Some("Status 500."),
            "the faults of another library stay"
        );
        forget_the_faults_of("empty");
        assert_eq!(the_fault_of("empty", TheRequest::Shelves), None);
    }

    /// The view of the series says why it holds no line, and it says a reason
    /// that the program has. See T-170 and T-91.
    #[test]
    fn the_view_of_the_series_says_why_it_holds_no_line() {
        assert_eq!(
            the_reason_of_no_series(false, None),
            "This library has no series.\nPress h to go back."
        );

        assert!(the_reason_of_no_series(true, None).contains("does not answer"));

        let text = the_reason_of_no_series(false, Some("The server reported a fault. Status 500."));

        assert!(
            text.starts_with("The server did not give the series of this library:"),
            "{}",
            text
        );
        assert!(text.contains("Status 500."), "{}", text);
        assert!(
            !text.contains("has no series"),
            "the view must not say a reason that the program does not have: {}",
            text
        );

        // The fault of the request stands above the words of the offline mode
        // of the start: the program made that request, therefore it knows more
        // than the state of its start.
        assert!(the_reason_of_no_series(true, Some("a fault")).contains("a fault"));
    }
}
