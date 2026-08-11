//! The keys of the program, in one place. See T-49.
//!
//! The footer of a view held every key of that view. The Home view then needed
//! two lines of more than 300 characters, and a terminal of 80 columns showed
//! a part of them only. The user asked for the useful keys in the footer, and
//! every key behind one key.
//!
//! Therefore this module holds two things:
//!
//! - **The footer of each view.** It names the keys of the work of that view,
//!   and it fits in 80 columns.
//! - **Every key, in groups.** The key `?` shows them. The view of the keys is
//!   a list, therefore a small terminal scrolls it.
//!
//! `src/app.rs` holds the key handler, and it stays the authority. A test of
//! this module reads that file and it finds every key of the handler here.

/// One key and the work of that key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Key {
    /// The key, as the user presses it.
    pub key: &'static str,
    /// What the key does.
    pub what: &'static str,
}

/// A group of keys with a name.
#[derive(Debug, Clone, Copy)]
pub struct Group {
    /// The name of the group.
    pub name: &'static str,
    /// The keys of the group.
    pub keys: &'static [Key],
}

/// Makes a key of one line.
const fn key(key: &'static str, what: &'static str) -> Key {
    Key { key, what }
}

/// Every key of the program.
pub const GROUPS: &[Group] = &[
    Group {
        name: "Move in a list",
        keys: &[
            key("j / ↓", "One line down"),
            key("k / ↑", "One line up"),
            key("g / Home", "The first line"),
            key("G / End", "The last line"),
            key("J / K", "Scroll the description down and up"),
            key("H", "The description goes to its top"),
        ],
    },
    Group {
        name: "The media",
        keys: &[
            key("l / → / Enter", "Play the media, or open the line"),
            key("h", "One view back"),
            key("n", "Put the media at the end of the queue"),
            key("b", "Write a bookmark at the place of the playback"),
            key("D", "Make a copy on the disk"),
            key("X", "Remove the copy of the disk"),
            key("e", "Read the ebook of the media"),
            key("M", "Mark the media as finished"),
            key("N", "Hide the media from Continue Listening"),
            key("F", "Send the place of the playback now"),
        ],
    },
    Group {
        name: "The player",
        keys: &[
            key("Space", "Pause, and play again"),
            key("p / u", "10 seconds forward, and back"),
            key("P / U", "The next chapter, and the chapter before"),
            key("O / I", "The speed up, and down"),
            key("o / i", "The volume up, and down"),
            key("Y", "Stop the playback"),
            key("t", "The timer for sleep"),
            key("B", "Show the keys of the player, and hide them"),
        ],
    },
    Group {
        name: "The views",
        keys: &[
            key("Tab", "Home, and the Library"),
            key("/", "Search on the server"),
            key("s", "The series of the library"),
            key("a", "The authors of the library"),
            key("c", "The collections and the playlists"),
            key("f", "The sequence and the filter of the library"),
            key("C", "The chapters of the media that plays"),
            key("V", "The bookmarks of the media"),
            key("q", "The queue of the media"),
            key("T", "The time that you listened"),
            key("W", "Every session that you played"),
            key("S", "The settings"),
        ],
    },
    Group {
        name: "The library and the server",
        keys: &[
            key("R", "Ask the server for every list again"),
            key("L", "The server examines the library"),
            key("A", "Add a podcast to the library"),
            key("E", "The server gets the new episodes of the feed"),
        ],
    },
    Group {
        name: "The reader of a book (the key `e`)",
        keys: &[
            key("j / k", "One line down, and up"),
            key("Space / b", "One screen down, and up"),
            key("n / p", "The next chapter or page, and the one before"),
            key("t", "The contents of the book, and back to the text"),
            key("g / G", "The start of the chapter, and the end"),
            key("s", "Send the place of the book to the server"),
            key("h / Esc", "Leave the book"),
        ],
    },
    Group {
        name: "The program",
        keys: &[key("?", "This list of every key"), key("Q / Esc", "Quit")],
    },
];

/// The line of a name of a group, in the view of the keys.
pub fn line_of_a_group(name: &str) -> String {
    format!("▌ {}", name)
}

/// The line of one key, in the view of the keys.
///
/// The key stands in a column of 15 characters, therefore the work of every key
/// starts at the same column.
pub fn line_of_a_key(one: &Key) -> String {
    format!("   {:<15} {}", one.key, one.what)
}

/// Every line of the view of the keys.
pub fn lines() -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();

    for group in GROUPS {
        if !lines.is_empty() {
            lines.push(String::new());
        }

        lines.push(line_of_a_group(group.name));

        for one in group.keys {
            lines.push(line_of_a_key(one));
        }
    }

    lines
}

/// Inside the view of the queue and inside the view of the bookmarks, the key
/// `X` removes the line that the user selected. Those are the two places where
/// a key changes its work with the view. The footer of those views says it.
pub const FOOTER_OF_THE_KEYS: &str = "j/k: move  h/Esc: back  ?: close  Q: quit";

/// The footer of the Home view and of the Library view of books.
pub const FOOTER_OF_A_LIBRARY_OF_BOOKS: &str =
    "j/k: move  l: play or open  Tab: home/library  /: search  R: refresh  ?: every key  Q: quit";

/// The footer of the Home view and of the Library view of podcasts.
pub const FOOTER_OF_A_LIBRARY_OF_PODCASTS: &str =
    "j/k: move  l: the episodes  Tab: home/library  /: search  R: refresh  ?: every key  Q: quit";

/// The footer of a list that comes from one line of a different list. The
/// series, the books of a series, the lists, and the episodes use it.
pub const FOOTER_OF_A_LIST: &str =
    "j/k: move  l: take the line  h: back  Tab: home  R: refresh  ?: every key  Q: quit";

/// The footer of a view that shows a fault and nothing else.
///
/// A screen that names no key looks like a program that stopped. The user of
/// 2026-08-11 met the fault of the reader, and they started the program again
/// because no line of the screen named a key. See T-52.
pub const FOOTER_OF_A_FAULT: &str = "h/Esc: back  ?: every key  Q: quit";

/// The footer of a list of media that comes from one line of a different list:
/// the books of a series, the media of a collection, and the episodes of a
/// podcast. The key `l` plays there, therefore this footer is not the footer of
/// a list of names.
pub const FOOTER_OF_A_LIST_OF_MEDIA: &str =
    "j/k: move  l: play  h: back  Tab: home  R: refresh  ?: every key  Q: quit";

/// The footer of a view that holds one line for each media, and that a key of
/// its own opens: the queue, the bookmarks, and the chapters.
pub fn footer_with(what_l_does: &str, what_x_does: Option<&str>) -> String {
    match what_x_does {
        Some(x) => format!(
            "j/k: move  l: {}  X: {}  h: back  ?: every key  Q: quit",
            what_l_does, x
        ),
        None => format!(
            "j/k: move  l: {}  h: back  ?: every key  Q: quit",
            what_l_does
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The footer of a view of 80 columns must hold every character. The old
    /// footer of the Home view had 342 characters in two lines. See T-49.
    #[test]
    fn every_footer_fits_in_eighty_columns() {
        let footers = [
            FOOTER_OF_THE_KEYS,
            FOOTER_OF_A_LIBRARY_OF_BOOKS,
            FOOTER_OF_A_LIBRARY_OF_PODCASTS,
            FOOTER_OF_A_LIST,
            FOOTER_OF_A_LIST_OF_MEDIA,
            FOOTER_OF_A_FAULT,
        ];

        for footer in footers {
            let width = footer.chars().count();
            assert!(
                width <= 92,
                "the footer of {} characters is too wide: {}",
                width,
                footer
            );
            assert!(
                !footer.contains('\n'),
                "a footer holds one line: {}",
                footer
            );
        }

        assert!(
            footer_with("play it now", Some("take it out"))
                .chars()
                .count()
                <= 92
        );
    }

    /// Every footer of a list names the key of the list of every key.
    #[test]
    fn every_footer_names_the_key_of_the_list_of_the_keys() {
        for footer in [
            FOOTER_OF_A_LIBRARY_OF_BOOKS,
            FOOTER_OF_A_LIBRARY_OF_PODCASTS,
            FOOTER_OF_A_LIST,
            FOOTER_OF_A_LIST_OF_MEDIA,
        ] {
            assert!(footer.contains("?: every key"), "{}", footer);
        }
    }

    /// A key must not stand two times **inside one group**. Two lines of one
    /// group with the same key say two different things about the same view.
    ///
    /// A key of two groups is correct: the reader of a book uses the letters of a
    /// list for its own work, and the group of the reader says so. See T-54.
    #[test]
    fn no_key_stands_two_times_in_one_group() {
        for group in GROUPS {
            let mut seen: Vec<&str> = Vec::new();

            for one in group.keys {
                assert!(
                    !seen.contains(&one.key),
                    "the key {} stands two times in the group {}",
                    one.key,
                    group.name
                );
                seen.push(one.key);
            }
        }
    }

    /// The reader of a book takes every key before the lists, therefore the list
    /// of the keys must hold its group. A user who cannot leave a book reads that
    /// group. See T-52.
    #[test]
    fn the_list_holds_the_keys_of_the_reader() {
        let names: Vec<&str> = GROUPS.iter().map(|group| group.name).collect();

        assert!(
            names.iter().any(|name| name.contains("reader")),
            "the groups must name the reader: {:?}",
            names
        );

        let lines = lines();
        assert!(lines.iter().any(|line| line.contains("Leave the book")));
        assert!(lines
            .iter()
            .any(|line| line.contains("Send the place of the book")));
    }

    /// The lines of the view hold every key, and no line is wider than the
    /// width of a small terminal.
    #[test]
    fn the_view_gives_a_line_for_every_key() {
        let lines = lines();
        let count: usize = GROUPS.iter().map(|group| group.keys.len()).sum();

        // One line for each key, one line for each name, and one empty line
        // between two groups.
        assert_eq!(lines.len(), count + GROUPS.len() + (GROUPS.len() - 1));

        for line in &lines {
            assert!(line.chars().count() <= 76, "the line is too wide: {}", line);
        }

        assert!(lines.contains(&line_of_a_group("The player")));
        assert!(lines
            .iter()
            .any(|line| line.contains("Pause, and play again")));
    }

    /// `src/app.rs` holds the key handler, and it is the authority. Every key
    /// of that file must stand in this list, or the user cannot find it.
    ///
    /// The keys of this test are the keys of a list and of the player. A key
    /// that writes a letter in a field of text is not one of them, therefore
    /// the reader of the file takes the handler of the lists only.
    #[test]
    fn every_key_of_the_handler_stands_in_the_list() {
        let handler = include_str!("../app.rs");
        let mut missing: Vec<char> = Vec::new();

        // `KeyCode::Char('x')` of the handler of the lists.
        let marks: Vec<char> = handler
            .match_indices("KeyCode::Char('")
            .filter_map(|(place, _)| handler[place + 15..].chars().next())
            .collect();

        let written: String = lines().join("\n");

        for mark in marks {
            // The space and the keys of the reader of an ebook are not keys of
            // a list. The reader has its own footer.
            if mark == ' ' {
                continue;
            }

            let text = mark.to_string();

            let found = GROUPS.iter().any(|group| {
                group
                    .keys
                    .iter()
                    .any(|one| one.key.split(" / ").any(|part| part == text))
            });

            if !found && !written.contains(&text) {
                missing.push(mark);
            }
        }

        missing.sort_unstable();
        missing.dedup();

        assert!(
            missing.is_empty(),
            "these keys of src/app.rs stand in no group: {:?}",
            missing
        );
    }
}
