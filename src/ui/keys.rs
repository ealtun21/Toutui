//! The keys of the program, in one place. See T-49.
//!
//! The footer of a view held every key of that view. The Home view then needed
//! two lines of more than 300 characters, and a terminal of 80 columns showed
//! a part of them only. The user asked for the useful keys in the footer, and
//! every key behind one key.
//!
//! Therefore this module holds two things:
//!
//! - **The footer of each view.** It names the keys of the work of that view.
//!   The area of the footer holds two rows, and the text wraps: a footer of 94
//!   letters therefore keeps every word in a terminal of 80 columns. The old
//!   text of this line said that every footer fits in 80 columns, and no footer
//!   of more than 80 letters did. See T-90.
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
            key("m", "Put the media in a collection or in a playlist"),
            key("b", "Write a bookmark at the place of the playback"),
            key("D", "Make a copy on the disk"),
            key(
                "X",
                "Remove the copy of the disk, and the ebook of the reader",
            ),
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
            key("v", "The narrators of the library"),
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
            key("d", "The episodes that the server downloads, and the queue"),
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
            key("e", "The books of this media, when it holds more than one"),
            key("h / Esc", "Leave the book"),
        ],
    },
    Group {
        name: "The lists that take a media (the key `m`)",
        keys: &[
            key("l", "Put the media in the list of the line"),
            key("c", "Make a new collection, and put the media in it"),
            key("p", "Make a new playlist, and put the media in it"),
        ],
    },
    Group {
        name: "The collections and the playlists (the key `c`)",
        keys: &[
            key("l", "The media of the list of the line"),
            key("r", "Give the list a new name"),
            key("D", "Give the list a new description"),
            key("X", "Remove the list. The program asks one time"),
        ],
    },
    Group {
        name: "The media of a list (the key `c`, and then `l`)",
        keys: &[
            key("l", "Play the media of the line"),
            key("< / >", "Move the media one line up, and down"),
            key("X", "Take the media out of the list"),
        ],
    },
    Group {
        name: "The program",
        keys: &[key("?", "This list of every key"), key("Q / Esc", "Quit")],
    },
];

/// Gives a number with the name of the thing that it counts.
///
/// **One thing is "1 track", and not "1 tracks".** The plural of every name of
/// this program takes one letter `s`. A name that does not follow that rule
/// needs its own function.
///
/// A sweep of a library of one media on 2026-08-12 read
/// "1 items, 57 tracks, 0 authors, 0 genres" in the view of the statistics. See
/// T-106.
pub fn counted(count: usize, name: &str) -> String {
    if count == 1 {
        return format!("1 {}", name);
    }

    format!("{} {}s", count, name)
}

/// Gives the number of the lines of a view, for the title of that view.
///
/// **One line is "1 item", and not "1 items".** A measurement of 2026-08-11 read
/// "A Test Playlist [1 items]" after the key `X` took one media out of a
/// playlist of two. `ListView::line` held this rule already, and no title of a
/// view held it. See T-85.
///
/// The function is pure, therefore a test needs no screen.
pub fn items(count: usize) -> String {
    counted(count, "item")
}

/// Removes `http://` or `https://` from an address.
///
/// The header of the screen holds no scheme: the user reads the machine and the
/// port. See T-105.
pub fn without_the_scheme(url: &str) -> &str {
    url.strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
        .unwrap_or(url)
}

/// The two lines of the header that name the account and the address.
///
/// **The header names the address that the program uses now**, and not the
/// address of the login. `config.toml` takes more than one address of one
/// server, and a sweep of two addresses on 2026-08-12 read `localhost:13399` in
/// the header while every request went to `127.0.0.1:13456`. See T-105.
///
/// **A program that no address answers must not say "Connected".** The same
/// sweep stopped the server in the middle of a playback: the log said "the
/// server does not answer" every six seconds, and the header said "Connected"
/// until the user pressed `R`. See T-107.
///
/// `active` is the address of the pool, with its scheme. `stored` is the address
/// of the login, with no scheme.
pub fn the_lines_of_the_connection(
    username: &str,
    active: Option<&str>,
    stored: &str,
    is_offline: bool,
) -> String {
    if is_offline {
        return format!("📴 Offline as {}\n🔗 {} does not answer", username, stored);
    }

    match active {
        Some(url) => format!(
            "👋 Connected as {}\n🔗 {}",
            username,
            without_the_scheme(url)
        ),
        None => format!(
            "⚠ {}: the server does not answer\n🔗 {} does not answer",
            username, stored
        ),
    }
}

/// The notice at the right of the header for a server that does not answer.
///
/// The program still holds the lists of the server, therefore it is not in the
/// offline mode. The key `R` gives the media of the disk. See T-107.
pub const THE_SERVER_DOES_NOT_ANSWER: &str = "R: the media of the disk";

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

/// The footer of the view of the search.
///
/// **The key `h` goes back**, as it does in every other view, and the footer
/// says so: a sweep of 2026-08-11 pressed `h` in that view and the screen did
/// not move. See T-79.
pub const FOOTER_OF_THE_SEARCH: &str =
    "j/k: move  l: play or open  h: back  /: search again  R: refresh  ?: every key  Q: quit";

/// The footer of a list that comes from one line of a different list. The
/// series, the books of a series, the lists, and the episodes use it.
pub const FOOTER_OF_A_LIST: &str =
    "j/k: move  l: take the line  h: back  Tab: home  R: refresh  ?: every key  Q: quit";

/// The footer of the view of the downloads of the server.
///
/// The footer names no key that asks the server again: **the view asks by
/// itself**, at each message of the server and after three seconds. See T-81.
pub const FOOTER_OF_THE_DOWNLOADS: &str =
    "j/k: move  X: empty the queue of this podcast  h: back  ?: every key  Q: quit";

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

/// The text of the line "Accounts and log out" of the settings.
///
/// A sweep of every view of 2026-08-11 read a run of 22 spaces inside a
/// sentence of this text. `Wrap` takes a space away at the start of a line
/// that it makes, and it keeps every space that stands inside a line.
/// Therefore a text of the screen holds one space between two words.
pub const THE_ACCOUNTS: &str = "The accounts that this program holds.\n\n\
    The key l on an account logs out of it: the program forgets the token of \
    that server, and it asks for the password again at the next start.\n\n\
    A program that holds more than one account starts with the account that is \
    the default one.";

/// The text of the line "Library: choose the library" of the settings.
pub const THE_LIBRARIES: &str = "The libraries of this server.\n\n\
    The key l on a library makes it the library that the program shows.";

/// The text of the view of the cache of the ebooks. See T-77.
pub const THE_CACHE_OF_THE_EBOOKS: &str = "The program keeps the ebook of a media \
    on the disk, therefore a second visit needs no request and the reader works \
    with no server.\n\n\
    The key l writes the value that you take in config.toml, and it keeps every \
    comment of that file. The program removes the book of the oldest use first, \
    and it never removes the book that you read now.\n\n\
    The variable TOUTUI_EBOOK_CACHE_BYTES comes before this value.";

/// Every text of the program that a view draws as a paragraph.
///
/// A test holds each of them to one space between two words.
/// The text of the view of the downloads of the server. See T-81.
pub const THE_DOWNLOADS_OF_THE_SERVER: &str = "The server gets the episodes, and \
    it holds a queue of that work. The key E on a podcast puts every episode of \
    the feed that the server does not hold in this queue.\n\n\
    The key X empties the queue of the podcast of the line. The episode that \
    downloads now (▼) goes on, because the server holds it outside the queue.";

/// The text of the view that puts a media in a list. See T-84.
pub const THE_LISTS_THAT_TAKE_A_MEDIA: &str = "A collection holds books, and every \
    user of the server sees it. A playlist belongs to you, and it holds books or \
    episodes of a podcast.\n\n\
    The key X of a list takes the media of the line out of that list. The key c \
    makes a new collection of this media, and the key p makes a new playlist of \
    it.";

/// The footer of the view of the collections and of the playlists.
///
/// The keys `r` and `X` of that view give a list a new name and remove it,
/// therefore the view has a footer of its own: `FOOTER_OF_A_LIST` names neither
/// of them. See T-93.
/// The footer of the view of the media of a collection or of a playlist.
///
/// **The keys `<` and `>` write the sequence of that list**, and the key `X`
/// takes the media out of it: `FOOTER_OF_A_LIST_OF_MEDIA` names neither of them,
/// and that footer stood in this view before. See T-102.
pub const FOOTER_OF_THE_MEDIA_OF_A_LIST: &str =
    "j/k: move  l: play  </>: the sequence  X: take it out  h: back  ?: every key  Q: quit";

pub const FOOTER_OF_THE_LISTS: &str =
    "j/k: move  l: the media  r/D: a name/description  X: remove  h: back  \
     ?: every key  Q: quit";

/// The footer of the view that puts a media in a list.
///
/// The keys `c` and `p` make a list, therefore this view has a footer of its
/// own: `footer_with` names the key `l` only. See T-88.
pub const FOOTER_OF_THE_LISTS_THAT_TAKE_A_MEDIA: &str =
    "j/k: move  l: put it here  c: a collection  p: a playlist  h: back  \
     ?: every key  Q: quit";

/// The text of the Home view that holds no shelf. See T-103.
pub const THE_HOME_VIEW_WITH_NO_LINE: &str = "The server gave no shelf for this library.\n\
     Press Tab for the Library, and R to ask the server again.";

/// The text of the Home view with a server that does not answer. See T-103.
///
/// **A view must not give a reason that the program does not have** (T-91). With
/// no answer of the server, the program knows nothing of the shelves of this
/// library.
pub const THE_HOME_VIEW_WITH_NO_ANSWER: &str =
    "The server does not answer, therefore this screen holds no shelf.\n\
     A media of the disk plays in this mode. Press R when the server answers again.";

/// The text of the Library view of a library of books with no media. See T-103.
pub const THE_LIBRARY_WITH_NO_MEDIA: &str = "This library holds no media.\n\
     Press L to tell the server to examine the library.";

/// The text of the Library view of a library of podcasts with no media. See
/// T-103.
pub const THE_LIBRARY_OF_PODCASTS_WITH_NO_MEDIA: &str = "This library holds no podcast.\n\
     Press A to add a podcast, and L to tell the server to examine the library.";

/// The text of the Library view that a filter emptied. See T-103.
///
/// **The library holds media, and the filter hides every one of them.** A screen
/// that says "This library holds no media" is false for that condition.
pub const THE_LIBRARY_WITH_A_FILTER: &str = "No media of this library agrees with the filter.\n\
     Press f for the sequence and the filter.";

/// The text of the Library view with a server that does not answer. See T-103.
pub const THE_LIBRARY_WITH_NO_ANSWER: &str =
    "The server gave no media: the server does not answer.\n\
     A media of the disk plays in this mode. Press R when the server answers again.";

/// Gives the text of the Home view that holds no line. See T-103.
pub fn the_text_of_the_home_view_with_no_line(is_offline: bool) -> &'static str {
    if is_offline {
        return THE_HOME_VIEW_WITH_NO_ANSWER;
    }

    THE_HOME_VIEW_WITH_NO_LINE
}

/// Gives the text of the Library view that holds no line. See T-103.
///
/// **The sequence of the three conditions holds a rule of its own.** A server
/// that does not answer comes first, because the program then knows nothing of
/// the library. A filter comes before the library itself, because the library
/// holds media in that condition.
pub fn the_text_of_the_library_view_with_no_line(
    is_offline: bool,
    a_filter_is_on: bool,
    is_podcast: bool,
) -> &'static str {
    if is_offline {
        return THE_LIBRARY_WITH_NO_ANSWER;
    }

    if a_filter_is_on {
        return THE_LIBRARY_WITH_A_FILTER;
    }

    if is_podcast {
        return THE_LIBRARY_OF_PODCASTS_WITH_NO_MEDIA;
    }

    THE_LIBRARY_WITH_NO_MEDIA
}

pub const THE_TEXTS_OF_THE_VIEWS: &[&str] = &[
    THE_HOME_VIEW_WITH_NO_LINE,
    THE_HOME_VIEW_WITH_NO_ANSWER,
    THE_LIBRARY_WITH_NO_MEDIA,
    THE_LIBRARY_OF_PODCASTS_WITH_NO_MEDIA,
    THE_LIBRARY_WITH_A_FILTER,
    THE_LIBRARY_WITH_NO_ANSWER,
    THE_ACCOUNTS,
    THE_LIBRARIES,
    THE_CACHE_OF_THE_EBOOKS,
    THE_DOWNLOADS_OF_THE_SERVER,
    THE_LISTS_THAT_TAKE_A_MEDIA,
];

#[cfg(test)]
mod tests {
    use super::*;

    /// One line of a view is "1 item", and not "1 items". See T-85.
    #[test]
    fn the_title_of_a_view_names_one_line_in_the_singular() {
        assert_eq!(items(0), "0 items");
        assert_eq!(items(1), "1 item");
        assert_eq!(items(2), "2 items");
    }

    /// One thing of any name is "1 track", and not "1 tracks". See T-106.
    #[test]
    fn a_text_of_a_view_names_one_thing_in_the_singular() {
        assert_eq!(counted(0, "track"), "0 tracks");
        assert_eq!(counted(1, "track"), "1 track");
        assert_eq!(counted(2, "track"), "2 tracks");
        assert_eq!(counted(1, "item"), items(1));
    }

    /// **The header names the address that the program uses now.**
    ///
    /// A sweep of two addresses on 2026-08-12 gave the pool
    /// `127.0.0.1:13456` and `localhost:13399`. Nine connections went to the
    /// first address, and the header said `localhost:13399`: it named the
    /// address of the login for ever. See T-105.
    #[test]
    fn the_header_names_the_address_that_answers() {
        let text = the_lines_of_the_connection(
            "toutuitest",
            Some("http://127.0.0.1:13456"),
            "localhost:13399",
            false,
        );

        assert!(text.contains("👋 Connected as toutuitest"), "{}", text);
        assert!(text.contains("🔗 127.0.0.1:13456"), "{}", text);
        assert!(
            !text.contains("localhost:13399"),
            "the header names the address of the login: {}",
            text
        );

        // The address holds no scheme, and `https` goes away too.
        assert!(
            the_lines_of_the_connection("u", Some("https://abs.example.com"), "x", false)
                .contains("🔗 abs.example.com")
        );
    }

    /// **A program that no address answers must not say "Connected".**
    ///
    /// A sweep of 2026-08-12 stopped the server in the middle of a playback.
    /// The log of the program said "the server does not answer" every six
    /// seconds, and the header said "👋 Connected" for 60 seconds, until the
    /// user pressed `R`. See T-107.
    #[test]
    fn the_header_says_that_no_address_answers() {
        let text = the_lines_of_the_connection("toutuitest", None, "localhost:13399", false);

        assert!(
            !text.contains("Connected"),
            "the header says \"Connected\" for a server that does not answer: {}",
            text
        );
        assert!(text.contains("the server does not answer"), "{}", text);
        assert!(
            text.contains("🔗 localhost:13399 does not answer"),
            "{}",
            text
        );

        // The offline mode keeps its own words: the lists come from the disk.
        let offline = the_lines_of_the_connection("toutuitest", None, "localhost:13399", true);
        assert!(offline.contains("📴 Offline as toutuitest"), "{}", offline);
    }

    /// A text of a view holds one space between two words.
    ///
    /// The text of the accounts held "the program" and "forgets" with 22
    /// spaces between them: an old wrap of the source stayed in the string, and
    /// the user read it on the screen. See the sweep of the views of
    /// 2026-08-11.
    #[test]
    fn a_text_of_a_view_holds_no_run_of_spaces() {
        for text in THE_TEXTS_OF_THE_VIEWS {
            for line in text.lines() {
                assert!(
                    !line.trim().contains("  "),
                    "the line \"{}\" holds a run of spaces. The screen shows it \
                     as it stands.",
                    line
                );
            }
        }
    }

    /// The footer of a view must hold every character in a terminal of 80
    /// columns. The old footer of the Home view had 342 characters in two
    /// lines. See T-49.
    ///
    /// **The area of the footer holds two rows**, and the text wraps: 92
    /// characters therefore reach the user in 80 columns, and 342 do not. See
    /// T-90.
    #[test]
    fn every_footer_fits_in_eighty_columns() {
        let footers = [
            FOOTER_OF_THE_KEYS,
            FOOTER_OF_THE_SEARCH,
            FOOTER_OF_THE_DOWNLOADS,
            FOOTER_OF_A_LIBRARY_OF_BOOKS,
            FOOTER_OF_A_LIBRARY_OF_PODCASTS,
            FOOTER_OF_A_LIST,
            FOOTER_OF_A_LIST_OF_MEDIA,
            FOOTER_OF_A_FAULT,
            FOOTER_OF_THE_LISTS_THAT_TAKE_A_MEDIA,
            FOOTER_OF_THE_LISTS,
            FOOTER_OF_THE_MEDIA_OF_A_LIST,
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

    /// **One function holds the rule of "1 item".** A title that writes those
    /// words itself holds a second copy of the rule, and the copy of the view
    /// of the search said "1 items" for two years of releases. See T-95.
    ///
    /// The test reads the files that make a title of a list, and it finds every
    /// text that counts its own items. A new file of that kind joins this list.
    #[test]
    fn no_title_of_a_view_counts_its_own_items() {
        let sources: &[(&str, &str)] = &[
            ("src/ui/tui.rs", include_str!("tui.rs")),
            (
                "src/logic/search/mod.rs",
                include_str!("../logic/search/mod.rs"),
            ),
            ("src/logic/authors.rs", include_str!("../logic/authors.rs")),
            (
                "src/api/utils/collect_lists.rs",
                include_str!("../api/utils/collect_lists.rs"),
            ),
        ];

        for (name, text) in sources {
            for line in text.lines() {
                // The tests of a file hold the words of the answer, and the
                // answer of `items` is that text. A line of a test says
                // "1 item" or "6 items" with a number, and never with a value.
                assert!(
                    !line.contains("{} items"),
                    "{} counts its own items: {}. Use ui::keys::items.",
                    name,
                    line.trim()
                );
            }
        }
    }

    /// **A view that holds no line says why**, and the reason of the Home view
    /// and of the Library view came late: those two views drew an empty list and
    /// no word. See T-103.
    ///
    /// The sequence of the conditions is the rule of this test. A server that
    /// does not answer comes first (T-91), and a filter comes before the library
    /// itself: the library holds media in that condition.
    #[test]
    fn a_view_with_no_line_says_why() {
        assert_eq!(
            the_text_of_the_home_view_with_no_line(false),
            THE_HOME_VIEW_WITH_NO_LINE
        );
        assert_eq!(
            the_text_of_the_home_view_with_no_line(true),
            THE_HOME_VIEW_WITH_NO_ANSWER
        );

        // A server that does not answer comes before every other reason, and
        // before the filter too.
        assert_eq!(
            the_text_of_the_library_view_with_no_line(true, true, true),
            THE_LIBRARY_WITH_NO_ANSWER
        );

        // The filter comes before the library, because the library holds media.
        assert_eq!(
            the_text_of_the_library_view_with_no_line(false, true, false),
            THE_LIBRARY_WITH_A_FILTER
        );

        assert_eq!(
            the_text_of_the_library_view_with_no_line(false, false, true),
            THE_LIBRARY_OF_PODCASTS_WITH_NO_MEDIA
        );
        assert_eq!(
            the_text_of_the_library_view_with_no_line(false, false, false),
            THE_LIBRARY_WITH_NO_MEDIA
        );

        // Every one of those texts names a key of the program, and it says one
        // thing in one sentence.
        for text in [
            THE_HOME_VIEW_WITH_NO_LINE,
            THE_HOME_VIEW_WITH_NO_ANSWER,
            THE_LIBRARY_WITH_NO_MEDIA,
            THE_LIBRARY_OF_PODCASTS_WITH_NO_MEDIA,
            THE_LIBRARY_WITH_A_FILTER,
            THE_LIBRARY_WITH_NO_ANSWER,
        ] {
            assert!(
                text.contains("Press "),
                "the text \"{}\" names no key of the program",
                text
            );
            assert!(
                !text.contains("Press '"),
                "a text of a view names a key with no quotation mark: {}",
                text
            );
        }
    }

    /// A text of a view names a key as the key stands, and never inside
    /// quotation marks. See T-91.
    ///
    /// The sweep of the empty library of 2026-08-11 read "Press 'h' to go
    /// back." in the view of the series, in the view of the lists, and in the
    /// view of the episodes, and it read "Press h to go back." in the view of
    /// the chapters. One program must say one thing in one way.
    #[test]
    fn a_text_of_a_view_names_a_key_with_no_quotation_mark() {
        let views = include_str!("tui.rs");

        assert!(
            !views.contains("Press '"),
            "a text of src/ui/tui.rs holds a key inside quotation marks. \
             The program says \"Press h to go back.\""
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
