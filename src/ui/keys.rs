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
        // **The panels of the frame stand in the Home view and in the Library
        // view of a terminal of 120 columns or more** (T-320), and the group
        // says so: a text must not promise a function that the program does not
        // have (T-118), and a footer must not promise a key that the view does
        // not hold (T-143).
        //
        // **The two panels of the design that no stage drew hold no digit**
        // (T-79): the panel 6 of the gallery comes with the rest of T-319, and
        // the panel 7 of the player comes with T-322.
        //
        // **The digit of the panel 5 does nothing for a view that draws no
        // cover** (T-319), and `App::a_panel_of_the_frame_stands` is that
        // rule: a terminal that is not wide, a media that the server holds
        // with no cover, and `TOUTUI_NO_COVERS` each take that panel away.
        //
        // **A stack that is not tall loses the panel 3 first and the panel 2
        // after it** (T-318), and the digit of a panel that the frame did not
        // draw does nothing at all.
        name: "The panels (a screen of 120 columns and more, Home and Library)",
        keys: &[
            key("1", "The focus goes to the panel 1 of the views"),
            key("2", "The focus goes to the panel 2 of the sequence"),
            key("3", "The focus goes to the panel 3 of the filter"),
            key("4", "The focus goes to the panel 4 of the list"),
            key("5", "The focus goes to the panel 5 of the cover"),
            key("6", "The focus goes to the panel 6 of the gallery"),
            key("Ctrl+h", "The focus goes to the panel at the left"),
            key("Ctrl+l", "The focus goes to the panel at the right"),
            key("Ctrl+j", "The focus goes to the panel below"),
            key("Ctrl+k", "The focus goes to the panel above"),
            key(
                "l / → / Enter",
                "The panel 1 opens a view, the panels 2 and 3 act",
            ),
            key("h / ←", "A panel of the stack gives the focus back"),
            key(
                "j / k",
                "The panel 5 moves the description, the panel 6 the cursor",
            ),
            // **The keys `+` and `-` are the buttons `[+ bigger]` and
            // `[- smaller]` of the design** (T-327). The key `+` of most
            // keyboards needs the modifier of the shift, and the key `=` of
            // that same place gives the same character with no modifier.
            key("+ / = / -", "The panel 6 changes the size of a cell"),
            // **The stack of the panels 1, 2, and 3 goes away with one key**
            // (T-323): the section (f) of `docs/mockups/mockup-1.md` names the
            // cost of a screen that is always full, and the panel 4 of the
            // list takes the 34 columns of the stack.
            //
            // **The words name the three panels, and they say no `1 to 3`**
            // (T-330): the maintainer read `1 to 3` as the panels 1 and 3, and
            // they then looked for a panel 2 that stood. A list of three names
            // takes two commas and the word `and`.
            key("z", "Hide the panels 1, 2, and 3, and show them again"),
        ],
    },
    Group {
        // **The mouse works in every view of the program** (T-316), and the key
        // that stops it works in every view too: a capture of the mouse takes
        // the selection of the text of the terminal away from the user, and the
        // road back must stand where the user is.
        name: "The mouse",
        keys: &[
            key(
                "Click",
                "The line of the pointer, and the focus of its panel",
            ),
            key("Wheel", "One line up or down, in the list of the pointer"),
            // **The row of the header of the table of the panel 4** (T-321).
            // The sequence of one column belongs to T-318, therefore this
            // click opens the view that holds every sequence today.
            key("Click a header", "The sequence and the filter"),
            key("Ctrl+o", "Stop the mouse, and start it again"),
            key(
                "Shift+Click",
                "Most terminals give the selection of the text",
            ),
        ],
    },
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
            key("@", "The server sends the ebook to an e-reader"),
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
            key("Shift+Tab", "The next library of the server"),
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
        name: "The accounts (the key `S`, and then the first line)",
        keys: &[
            key("a", "Add an account. The program starts again"),
            key("c", "Start with the account of the line. It asks one time"),
            key("l", "Log out. The copies of the disk of the account stay"),
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

/// Gives a number of bytes as a text in megabytes.
///
/// **A size that the user reads is a size in megabytes** (T-284): the reader
/// said "It has 269486151 bytes, and the limit is 268435456 bytes", and the
/// user of a terminal counts no digits. The bar of a download of the same
/// program says "1.2 MB" already, therefore every view of this program says a
/// size in this one form.
pub fn megabytes(bytes: u64) -> String {
    format!("{:.1} MB", bytes as f64 / 1_048_576.0)
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

/// The number of the lines of the Library view, for the title of that view.
///
/// **The program reads the library page by page** (T-70), therefore it holds
/// the items of the pages that came only. The title says the number of the
/// lines that it draws, and the number of the items of the library when the
/// program did not read every page.
///
/// A line is not an item: every book of a series gives one line (T-22).
/// Therefore the title says "8 items of 12" and never "8 of 12 items".
pub fn the_lines_of_the_library(lines: usize, loaded: usize, total: usize) -> String {
    if loaded >= total {
        return items(lines);
    }

    format!("{} of {}", items(lines), total)
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
/// **A server that answers with a fault is not a server that does not answer.**
/// A measurement of 2026-08-14 with `docs/harness/one_path_fails.py` gave the
/// status 500 to `GET /api/libraries/:id/authors` alone. The pool marks that
/// address down, because a second address of the same server can answer it
/// (T-87), and the header then said `⚠ toutuitest: the server does not answer`
/// for 10.5 seconds while `curl` got an answer of that same address in 1.4
/// milliseconds. That sentence is a reason that the program does not have
/// (T-91). See T-171.
///
/// `active` is the address of the pool, with its scheme. `stored` is the address
/// of the login, with no scheme. `the_server_reports_a_fault` says that every
/// address of the pool answered, and that the answer holds a fault.
pub fn the_lines_of_the_connection(
    username: &str,
    active: Option<&str>,
    stored: &str,
    is_offline: bool,
    the_server_reports_a_fault: bool,
    width: u16,
) -> String {
    // **A narrow terminal takes the short form.** The three parts of the header
    // stand on one row, and each of them writes its own letters only: a
    // measurement of 2026-08-12 in 60 columns read
    // "👋 Connected as toutuitestBooks (book)", because the part of the account
    // and the part of the library met. See T-115.
    let short = width < THE_WIDTH_OF_THE_LONG_HEADER;

    if is_offline {
        if short {
            return format!("📴 {}\n🔗 {} does not answer", username, stored);
        }

        return format!("📴 Offline as {}\n🔗 {} does not answer", username, stored);
    }

    match active {
        Some(url) if short => format!("👋 {}\n🔗 {}", username, without_the_scheme(url)),
        Some(url) => format!(
            "👋 Connected as {}\n🔗 {}",
            username,
            without_the_scheme(url)
        ),
        // The server answered every request, and the answers hold a fault. The
        // program must not say that the server is away. See T-171.
        None if the_server_reports_a_fault && short => {
            format!("⚠ {}: a fault\n🔗 {} reports a fault", username, stored)
        }
        None if the_server_reports_a_fault => format!(
            "⚠ {}: the server reports a fault\n🔗 {} reports a fault",
            username, stored
        ),
        None if short => format!("⚠ {}: no answer\n🔗 {} does not answer", username, stored),
        None => format!(
            "⚠ {}: the server does not answer\n🔗 {} does not answer",
            username, stored
        ),
    }
}

/// The width where the header holds its long form.
///
/// The header holds three parts on one row: the account at the left, the library
/// in the middle, and the name of the program at the right. Every part is a
/// paragraph of its own over the same area, therefore a part that is too long
/// writes on the letters of its neighbour.
///
/// The measurement of 2026-08-12, of the account "toutuitest" and the library
/// "Books (book)":
///
/// | The width | The header |
/// |---|---|
/// | 80 | `👋 Connected as toutuitest    📖 Books (book)    🦜 Toutui v0.7.58` |
/// | 70 | the same, with fewer spaces |
/// | 60 | `👋 Connected as toutuitestBooks (book)     🦜 Toutui v0.7.58` |
///
/// The long form needs 26 + 15 + 17 cells and two spaces: 60 columns hold it
/// almost, and 68 hold it with room for a longer name of an account. See T-115.
pub const THE_WIDTH_OF_THE_LONG_HEADER: u16 = 68;

/// The name of the program for the right of the header.
///
/// A narrow terminal takes the short form, in the same way as the part of the
/// account. See T-115 and `THE_WIDTH_OF_THE_LONG_HEADER`.
pub fn the_name_of_the_program(version: &str, width: u16) -> String {
    if width < THE_WIDTH_OF_THE_LONG_HEADER {
        return format!("🦜 v{}", version);
    }

    format!("🦜 Toutui v{}", version)
}

/// The name of the library at the middle of the header.
///
/// **The name of a library is a text of the server**, and an administrator of it
/// gives that name: `PATCH /api/libraries/:id` of Audiobookshelf takes a name
/// that holds an end of a line, and the answer of `GET /api/libraries` then
/// gives it back to every client. The header of the screen holds **two** rows,
/// and `render_header` of `src/ui/tui.rs` draws this text in a `Paragraph` with
/// no wrap: an end of a line in it therefore puts every character after it on
/// the second row of the header, beside the row of the connection. See T-314.
pub fn the_name_of_the_library(name: &str, media_type: &str) -> String {
    format!(
        "📖 {} ({})",
        crate::logic::message::in_one_line(name),
        crate::logic::message::in_one_line(media_type)
    )
}

/// The two lines of the panel of a media.
///
/// **A text of the server puts the place of the user off the screen** (T-315).
/// The panel of a media stands under the list, and `the_areas_of_a_list` of
/// `src/ui/tui.rs` gives it **two** rows in a terminal that is not tall. The
/// first line of it names the author, the year, and the length of the media,
/// and the second line names the place of the user. An author of a name of two
/// lines therefore gives that area three lines, and the row of the place goes
/// away.
///
/// This function holds the rule of T-311 for the two lines: a text of the
/// server keeps one line, and every end of a line of it becomes one space.
pub fn the_panel_of_a_media(of_the_media: &str, of_the_place: &str) -> String {
    format!(
        "{}\n{}",
        crate::logic::message::in_one_line(of_the_media),
        crate::logic::message::in_one_line(of_the_place)
    )
}

/// The notice at the right of the header for a server that does not answer.
///
/// The program still holds the lists of the server, therefore it is not in the
/// offline mode. The key `R` gives the media of the disk. See T-107.
pub const THE_SERVER_DOES_NOT_ANSWER: &str = "R: the media of the disk";

/// The notice at the right of the header for a server that answers with a
/// fault.
///
/// **The media of the disk are not the road of this user.** The server stands,
/// and it answers every other request: the key `R` asks it again, and it gives
/// the lists of the server. A sentence of a fault must name a key that does the
/// work of that fault (T-170). See T-171.
pub const THE_SERVER_REPORTS_A_FAULT: &str = "R: ask the server again";

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
pub const FOOTER_OF_A_LIBRARY_OF_BOOKS: &str = "j/k: move  l: play or open  \
     Tab: home/library  S-Tab: the next library  /: search  R: refresh  ?: every key  Q: quit";

/// The footer of the Home view and of the Library view of podcasts.
pub const FOOTER_OF_A_LIBRARY_OF_PODCASTS: &str = "j/k: move  l: the episodes  \
     Tab: home/library  S-Tab: the next library  /: search  R: refresh  ?: every key  Q: quit";

/// The footer of a view that the frame of the panels holds. See T-320.
///
/// **A footer must not promise a key that the view does not hold** (T-143), and
/// the frame of the panels stands at 120 columns and more: the footer of a
/// narrow terminal therefore names no panel at all, and this function gives the
/// text of the view back with no change.
///
/// **The footer of the panel that holds the focus is the footer of that panel**:
/// the keys `j`, `k`, and `l` of the panel 1 move its lines and they open a
/// view, therefore a footer that said `l: play or open` at that moment would
/// name a work that the key does not do.
///
/// **The panel 2 of the sequence and the panel 3 of the filter hold the same
/// keys as the panel 1** (T-318), and the key `l` of them takes the line and it
/// opens no view: the footer of each of the three panels therefore names the
/// work of its own key `l`.
/// **The key `z` hides the stack of the panels 1, 2, and 3** (T-323), and the
/// footer of the panel 4 then names no digit of that stack: the digit of a
/// panel that the frame did not draw does nothing (T-79), and a footer must not
/// promise it. The key `z` itself stands in the footer of the two modes,
/// because it is the road back of the mode that hides them.
pub fn the_footer_of_a_panel(
    of_the_view: &str,
    the_frame_stands: bool,
    the_stack_stands: bool,
    the_focus: crate::ui::frame::ThePanel,
) -> String {
    use crate::ui::frame::ThePanel;

    if !the_frame_stands {
        return of_the_view.to_string();
    }

    match the_focus {
        ThePanel::TheViews => "j/k: move  l: open the view  h: the list  \
                4/Ctrl+l: the list  ?: every key  Q: quit"
            .to_string(),
        ThePanel::TheSequence => "j/k: move  l: this sequence  h: the list  \
                4/Ctrl+l: the list  ?: every key  Q: quit"
            .to_string(),
        ThePanel::TheFilter => "j/k: move  l: this filter  h: the list  \
                4/Ctrl+l: the list  ?: every key  Q: quit"
            .to_string(),
        // **The footer of the panel 4 names the key `f` of the sequence and of
        // the filter** (T-318): the key stood in the panel 1 and in no footer,
        // and a user who cannot find a key has no key at all (the rule of T-143
        // in reverse).
        // **The footer of the panel 5 names the keys of that panel** (T-319):
        // the keys `j` and `k` move the description of the media, and the key
        // `l` plays it, which is the key of the list of the view.
        ThePanel::TheCover => "j/k: the description  l: play or open  h: the list  \
                4/Ctrl+h: the list  ?: every key  Q: quit"
            .to_string(),
        // **The footer of the panel 6 names the keys of the gallery** (T-327):
        // the keys `j` and `k` move the cursor of the list one row of the grid,
        // the keys `+` and `-` change the size of a cell, and the key `l` plays
        // the media of the cell of the cursor.
        ThePanel::TheGallery => "j/k: a row of the grid  +/-: the size of a cell  \
                l: play or open  h: the list  ?: every key  Q: quit"
            .to_string(),
        ThePanel::TheList if the_stack_stands => {
            format!("{of_the_view}  f: sequence  1/Ctrl+h: the panels  z: hide them")
        }
        ThePanel::TheList => format!("{of_the_view}  f: sequence  z: the panels 1, 2, and 3"),
    }
}

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

/// The footer of the reader, while it shows the text of a chapter.
///
/// **The footer of the reader is a footer of this program** (T-301). The four
/// texts of the reader stood in `src/ui/reader_tui.rs`, each of them with a
/// `\n` of its own and with no wrap at all, therefore the gate of the footers
/// of this module never read them: a terminal of 40 columns cut the first row
/// at `n/p: chapter` and the second row at `h:`, and the user then read no key
/// of the road back and no key of the quit.
pub const FOOTER_OF_THE_READER: &str = "j/k: line  Space/b: screen  n/p: chapter  \
     t: contents  g/G: start/end  s: send the position  ?: every key  \
     h: leave the book  Q: quit";

/// The footer of the reader, while it shows a page of a PDF.
///
/// One chapter of a PDF is one page, therefore the keys of that book name a
/// page and not a chapter. See T-54.
pub const FOOTER_OF_THE_READER_OF_PAGES: &str = "j/k: line  Space/b: screen  n/p: page  \
     t: the pages  g/G: start/end  s: send the position  ?: every key  \
     h: leave the book  Q: quit";

/// The footer of the contents of a book of chapters.
pub const FOOTER_OF_THE_CONTENTS: &str = "j/k: move  l/Enter: go to the chapter  \
     t: back to the text  h: leave the book";

/// The footer of the pages of a PDF.
pub const FOOTER_OF_THE_PAGES: &str =
    "j/k: move  l/Enter: go to the page  t: the pages  h: leave the book";

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
///
/// **The text said what the program does not do**, and T-118 corrected the
/// words: the program held one account then. **T-124 gives the program the
/// function**, therefore the text names the three keys of this view now. A text
/// must say what the program does, and no more (T-91 and T-118).
pub const THE_ACCOUNTS: &str = "The accounts of this program.\n\n\
    The mark ▶ is on the account that the program starts with.\n\n\
    The key a adds an account: the program starts again, and it asks you for a \
    server, a name, and a password. The key c gives the start to the account of \
    the line, and the program starts again with it. The key l logs out: the \
    program removes the account and it forgets its token. The copies of the disk \
    of that account stay, and a login with the same name and the same server \
    gives them again.\n\n\
    A playback stops when the program starts again.";

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

/// The text of the view that sends a book to an e-reader. See T-119.
///
/// **The server sends the book of `media.ebookFile`**, and an item can hold more
/// than one ebook (T-76): the endpoint of the server takes the item, and never a
/// file. The text says it, therefore a user who reads a second book of an item
/// knows which book goes.
pub const THE_DEVICES_OF_AN_EREADER: &str = "An administrator of the server makes \
    every device, with a name and an address of e-mail. The server sends the book \
    of this item to that address.\n\n\
    An item can hold more than one ebook. The server sends the book that it holds \
    for the item, and not the book that the reader of this program shows.";

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

/// The footer of the view that sends a book to an e-reader. See T-119.
pub const FOOTER_OF_THE_DEVICES_OF_AN_EREADER: &str =
    "j/k: move  l: send the book  h: back  ?: every key  Q: quit";

/// The footer of the view of the accounts.
///
/// **The text held a `\n` and it stood in `src/ui/tui.rs`** (T-302), therefore
/// the gate of this module never read it, and the row of that `\n` did the work
/// of the wrap. The words of the log out stand in the text of the view
/// (`THE_ACCOUNTS`): the footer names the keys.
pub const FOOTER_OF_THE_ACCOUNTS: &str = "h: back, a: add an account, \
     c: this account starts, l/→: log out, Tab: home, R: refresh, Q/Esc: quit.";

/// The footer of the view that chooses the library. See T-302.
pub const FOOTER_OF_THE_LIBRARY_OF_THE_USER: &str =
    "h: back, l/→: change library, Tab: home, R: refresh, Q/Esc: quit.";

/// The footer of the view of the statistics of the user.
///
/// The text stood in `render_stats` of `src/ui/tui.rs`, therefore the gate of
/// this module never read it. See T-302.
pub const FOOTER_OF_THE_STATISTICS: &str =
    "j/k: move  T: ask the server again  h: back  ?: every key  Q: quit";

/// The footer of the view of the sessions of the user. See T-302.
pub const FOOTER_OF_THE_SESSIONS: &str =
    "j/k: move  W: ask the server again  h: back  ?: every key  Q: quit";

/// The footer of the view that adds a podcast to the library. See T-302.
pub const FOOTER_OF_A_NEW_PODCAST: &str =
    "j/k: move  l: add the podcast  A: other words  h: back  ?: every key  Q: quit";

/// Every footer of a view of this program, for the gates of this module.
///
/// **A footer that stands outside this list stands outside every gate**
/// (T-302): the footers of the statistics, of the sessions, of a new podcast,
/// of the accounts, and of the library of the user stood in `src/ui/tui.rs`,
/// and two of them held a `\n` that did the work of the wrap. The footers of a
/// `footer_with` stand in the gates beside this list, because that function
/// makes its text of the view.
pub const THE_FOOTERS_OF_THE_VIEWS: &[&str] = &[
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
    FOOTER_OF_THE_DEVICES_OF_AN_EREADER,
    FOOTER_OF_THE_STATISTICS,
    FOOTER_OF_THE_SESSIONS,
    FOOTER_OF_A_NEW_PODCAST,
    FOOTER_OF_THE_ACCOUNTS,
    FOOTER_OF_THE_LIBRARY_OF_THE_USER,
    FOOTER_OF_THE_READER,
    FOOTER_OF_THE_READER_OF_PAGES,
    FOOTER_OF_THE_CONTENTS,
    FOOTER_OF_THE_PAGES,
];

/// The smallest number of rows of a footer.
///
/// Every view of this program held two rows for its footer at every width, and
/// a terminal that holds every key in one row keeps those two rows: a view that
/// grows by one row at 160 columns is a change that no fault asks for.
pub const THE_SMALLEST_FOOTER: u16 = 2;

/// Gives the number of rows that a footer needs.
///
/// **A footer stands on the rows that it needs** (T-301 for the reader, and
/// T-302 for every other view). The footer of a view held two rows at every
/// width, and a terminal of 40 columns holds 80 cells in them: the Home view of
/// the measurement of 2026-08-16 said `j/k: move  l: play or open  Tab:
/// home/library  S-Tab: the next library` and no more, therefore the user read
/// no key of the search, of the refresh, of the table of the keys, and of the
/// quit.
///
/// That is the rule of T-299 for the row of the message of a view: the text
/// takes the rows that its wrap needs, and it grows over the view. A list of a
/// view loses a line, and no line of a list goes out of the reach of the user,
/// because the key `j` moves the list.
///
/// **The footer must not take the view**: it holds no more than one half of the
/// rows of the view, and a footer that needs more than that loses its end to
/// three points (`crate::logic::message::in_the_rows`).
///
/// The function is pure, therefore a test needs no screen.
pub fn the_rows_of_a_footer(text: &str, width: u16, rows_of_the_view: u16) -> u16 {
    let room = (rows_of_the_view / 2).max(1);

    crate::logic::message::the_rows_of_a_message(text, width)
        .max(THE_SMALLEST_FOOTER)
        .min(room.max(THE_SMALLEST_FOOTER).min(rows_of_the_view))
}

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

/// The text of the Library view of the offline mode with a database that gave no
/// media of the disk. See T-203.
pub const THE_LIBRARY_WITH_NO_MEDIA_OF_THE_DISK: &str =
    "The program did not read the media of the disk in its database.\n\
     Press R to try again.";

/// The text of the Library view with a server that does not answer. See T-103.
pub const THE_LIBRARY_WITH_NO_ANSWER: &str =
    "The server gave no media: the server does not answer.\n\
     A media of the disk plays in this mode. Press R when the server answers again.";

/// The label of a media that stands on the disk of this account.
pub const THE_COPY_OF_THE_DISK: &str = " - [Downloaded]";

/// What the key `B` says when the disk did not take the row of the keys of the
/// player. See T-204.
///
/// **A key of the user that writes the disk takes a sentence** (T-199), and the
/// sentence names the key of the view that the user sees at that moment (T-183).
/// A second program of the account can hold the database of this account
/// (T-140).
pub const THE_KEYS_OF_THE_PLAYER_DID_NOT_REACH_THE_DISK: &str =
    "The program did not write the row of the keys of the player: the database \
     did not answer. A different program of this account can hold it. Press B again.";

/// What a refresh says when it did not read the accounts of the database. See
/// T-205.
///
/// **A refresh is not a start.** T-199 stops the program when the read of the
/// accounts of `main` fails, and at the start that is right: the program holds
/// no account, and it can do no work. A refresh holds the account, the token,
/// every list, and the playback of the user already, therefore a database that a
/// second program of this account writes for six seconds (T-140) must take none
/// of them away. The application of the user stays, and the sentence names the
/// key of the view that the user sees at that moment (T-183).
pub const THE_REFRESH_DID_NOT_READ_THE_DATABASE: &str =
    "The program did not read the accounts of its database, therefore the screen \
     did not change. A different program of this account can hold it. Press R again.";

/// What a refresh says when it cannot read the configuration file. See T-266.
///
/// **A refresh is not a start** (T-205). The key `R` reads `config.toml` again
/// (T-142), therefore the user who changes one colour of that file and who
/// leaves one bracket out meets this road. A measurement of 2026-08-15 of the
/// real program v0.8.94: that key took the whole program away with the status 1,
/// and the words of T-265 stood in the terminal of the shell. The application of
/// the user holds the account, every list, and the playback already, and the
/// values of the file that it read before stay good: therefore the application
/// stays, and the sentence names the file and the key of the view that the user
/// sees at that moment (T-183). The log holds the line and the column of the
/// fault, because the words of the crate `config` name them and no row of a
/// message holds that much text.
pub const THE_REFRESH_DID_NOT_READ_THE_CONFIGURATION_FILE: &str =
    "The program cannot read its configuration file, therefore the screen did \
     not change. The log names the fault of that file. Correct it, and press R \
     again.";

/// What the key of the next library says when the disk did not take it. See
/// T-205.
///
/// **A write of the disk that failed is no new library.** The old line was
/// `let _ = update_id_selected_lib(...)`, and the program then said
/// `The program shows the library "Podcasts" now.` while the row of the account
/// held the library of before: a refresh reads that row again, and the user read
/// the words of a work that the program did not do (T-91).
pub const THE_NEXT_LIBRARY_DID_NOT_REACH_THE_DISK: &str =
    "The program did not write the library of this account: the database did not \
     answer. A different program of this account can hold it. Press Shift-Tab again.";

/// What the view of the libraries of the settings says when the disk did not
/// take the library of the line. See T-205.
pub const THE_LIBRARY_DID_NOT_REACH_THE_DISK: &str =
    "The program did not write the library of this account: the database did not \
     answer. A different program of this account can hold it. Press Enter again.";

/// What the sequence and the filter of the library say when the disk did not
/// take them. See T-205.
pub const THE_SEQUENCE_DID_NOT_REACH_THE_DISK: &str =
    "The program did not write the sequence of this library: the database did not \
     answer. A different program of this account can hold it. Press Enter again.";

/// The label of a media whose copy on the disk the program did not read. See
/// T-203.
pub const THE_DISK_DID_NOT_ANSWER: &str = " - [the disk did not answer]";

/// The label of a media whose copy of the disk is not whole. See T-217.
///
/// **The label `[Downloaded]` of such a media said a copy that no playback of it
/// takes** (T-215 and T-216): the disk holds the rows of that download and not
/// every file of it, therefore the media plays from the server and the offline
/// mode of it plays nothing. The key `D` writes the files that went away again.
pub const THE_COPY_THAT_IS_NOT_WHOLE: &str = " - [the disk does not hold every file]";

/// Gives the label of the copy of the disk of one media. See T-203.
///
/// **A read of the disk that failed is not a media with no copy on the disk.**
/// The row of the detail of six views reads the table `downloads` at each frame,
/// and it held `is_some()` of that read: a database that says nothing then took
/// the label of every copy of the disk away, and the user read the line of a media
/// that the program did not measure.
///
/// `None` is a read that gave no answer. That row holds no key of the user,
/// therefore it takes no line of the log at each frame: the keys of the disk say
/// the fault (T-177 and T-185).
pub fn the_label_of_the_copy_of_the_disk(
    the_copy_of_the_disk: crate::logic::the_copies_of_the_disk::TheCopyOfTheDisk,
) -> &'static str {
    use crate::logic::the_copies_of_the_disk::TheCopyOfTheDisk;

    match the_copy_of_the_disk {
        TheCopyOfTheDisk::AWholeCopy => THE_COPY_OF_THE_DISK,
        // **A copy of the disk that is not whole is no copy of the disk**
        // (T-215, T-216, and T-217). The playback of that media takes the road
        // of the server, therefore the line says what the disk holds.
        TheCopyOfTheDisk::ACopyThatIsNotWhole => THE_COPY_THAT_IS_NOT_WHOLE,
        TheCopyOfTheDisk::NoCopy => "",
        TheCopyOfTheDisk::TheDiskDidNotAnswer => THE_DISK_DID_NOT_ANSWER,
    }
}

/// Gives the text of the Home view that holds no line. See T-103.
///
/// **The request of the shelves that came back with a fault is a condition of
/// its own** (T-170). The server answers, therefore `is_offline` holds `false`
/// (T-25), and the view said "The server gave no shelf for this library." for a
/// request that gave no answer at all. The sentence names what the server said,
/// and it names the key that asks the server again.
pub fn the_text_of_the_home_view_with_no_line(
    is_offline: bool,
    what_the_server_said: Option<&str>,
) -> String {
    if is_offline {
        return THE_HOME_VIEW_WITH_NO_ANSWER.to_string();
    }

    if let Some(fault) = what_the_server_said {
        return format!(
            "The server did not give the shelves of this library: {}\n\
             Press R to ask the server again.",
            fault
        );
    }

    THE_HOME_VIEW_WITH_NO_LINE.to_string()
}

/// Gives the text of the Library view that holds no line. See T-103.
///
/// **The sequence of the four conditions holds a rule of its own.** A server
/// that does not answer comes first, because the program then knows nothing of
/// the library. **The request that came back with a fault comes after it**: the
/// program knows nothing of the library in that condition too, and a filter
/// says nothing of a list that never came (T-170). A filter comes before the
/// library itself, because the library holds media in that condition.
pub fn the_text_of_the_library_view_with_no_line(
    is_offline: bool,
    the_disk_did_not_answer: bool,
    a_filter_is_on: bool,
    is_podcast: bool,
    what_the_server_said: Option<&str>,
) -> String {
    // **A read of the disk that failed comes before every other condition**
    // (T-203). The offline mode of T-25 holds the media of the disk alone, and
    // this view said "The server gave no media: the server does not answer." for
    // the nine downloads that stood on the disk of the measurement: the program
    // names the thing that failed (T-91 and T-172).
    if the_disk_did_not_answer {
        return THE_LIBRARY_WITH_NO_MEDIA_OF_THE_DISK.to_string();
    }

    if is_offline {
        return THE_LIBRARY_WITH_NO_ANSWER.to_string();
    }

    if let Some(fault) = what_the_server_said {
        return format!(
            "The server did not give the media of this library: {}\n\
             Press R to ask the server again.",
            fault
        );
    }

    if a_filter_is_on {
        return THE_LIBRARY_WITH_A_FILTER.to_string();
    }

    if is_podcast {
        return THE_LIBRARY_OF_PODCASTS_WITH_NO_MEDIA.to_string();
    }

    THE_LIBRARY_WITH_NO_MEDIA.to_string()
}

pub const THE_TEXTS_OF_THE_VIEWS: &[&str] = &[
    THE_HOME_VIEW_WITH_NO_LINE,
    THE_HOME_VIEW_WITH_NO_ANSWER,
    THE_LIBRARY_WITH_NO_MEDIA,
    THE_LIBRARY_OF_PODCASTS_WITH_NO_MEDIA,
    THE_LIBRARY_WITH_A_FILTER,
    THE_LIBRARY_WITH_NO_ANSWER,
    THE_LIBRARY_WITH_NO_MEDIA_OF_THE_DISK,
    THE_ACCOUNTS,
    THE_LIBRARIES,
    THE_CACHE_OF_THE_EBOOKS,
    THE_DOWNLOADS_OF_THE_SERVER,
    THE_LISTS_THAT_TAKE_A_MEDIA,
];

#[cfg(test)]
mod tests {
    use super::*;

    /// The footers that a view makes with `footer_with`, for the gates of the
    /// footers. See T-302.
    fn the_footers_that_a_view_makes() -> [String; 5] {
        [
            footer_with("play it now", Some("take it out")),
            footer_with("go to the place", Some("remove the bookmark")),
            footer_with("go to the chapter", None),
            footer_with("read this book", None),
            footer_with("write this value in config.toml", None),
        ]
    }

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

    /// The title of the Library view says what the program holds, and what the
    /// library holds. See T-70.
    #[test]
    fn the_title_of_the_library_says_the_pages_that_came() {
        // Every page came: the title says the lines only, as it did before.
        assert_eq!(the_lines_of_the_library(8, 12, 12), "8 items");
        assert_eq!(the_lines_of_the_library(1, 1, 1), "1 item");

        // The program holds the first page of a library of 2056 items.
        assert_eq!(
            the_lines_of_the_library(500, 500, 2056),
            "500 items of 2056"
        );

        // A library that the server did not give holds nothing, and the title
        // must not say a number of the server that the program cannot show.
        assert_eq!(the_lines_of_the_library(0, 0, 0), "0 items");
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
            false,
            160,
        );

        assert!(text.contains("👋 Connected as toutuitest"), "{}", text);
        assert!(text.contains("🔗 127.0.0.1:13456"), "{}", text);
        assert!(
            !text.contains("localhost:13399"),
            "the header names the address of the login: {}",
            text
        );

        // The address holds no scheme, and `https` goes away too.
        assert!(the_lines_of_the_connection(
            "u",
            Some("https://abs.example.com"),
            "x",
            false,
            false,
            160
        )
        .contains("🔗 abs.example.com"));
    }

    /// **A program that no address answers must not say "Connected".**
    ///
    /// A sweep of 2026-08-12 stopped the server in the middle of a playback.
    /// The log of the program said "the server does not answer" every six
    /// seconds, and the header said "👋 Connected" for 60 seconds, until the
    /// user pressed `R`. See T-107.
    #[test]
    fn the_header_says_that_no_address_answers() {
        let text =
            the_lines_of_the_connection("toutuitest", None, "localhost:13399", false, false, 160);

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
        let offline =
            the_lines_of_the_connection("toutuitest", None, "localhost:13399", true, false, 160);
        assert!(offline.contains("📴 Offline as toutuitest"), "{}", offline);
    }

    /// **The three parts of the header must not write on each other.**
    ///
    /// The sweep of a terminal that changes its size, 2026-08-12: 60 columns
    /// gave "👋 Connected as toutuitestBooks (book)". Every part takes its short
    /// form below `THE_WIDTH_OF_THE_LONG_HEADER`. See T-115.
    #[test]
    fn a_narrow_header_takes_the_short_form() {
        let long =
            the_lines_of_the_connection("toutuitest", Some("http://one:1"), "x", false, false, 80);
        let short =
            the_lines_of_the_connection("toutuitest", Some("http://one:1"), "x", false, false, 60);

        assert!(long.contains("👋 Connected as toutuitest"), "{}", long);
        assert!(short.contains("👋 toutuitest"), "{}", short);
        assert!(!short.contains("Connected"), "{}", short);

        // Every form names the account and the address: the short form holds
        // fewer words, and no value goes away.
        assert!(short.contains("🔗 one:1"), "{}", short);

        // The offline mode and the server that does not answer keep the rule.
        let offline = the_lines_of_the_connection("toutuitest", None, "one:1", true, false, 60);
        assert!(offline.contains("📴 toutuitest"), "{}", offline);
        assert!(offline.contains("🔗 one:1 does not answer"), "{}", offline);

        let no_answer = the_lines_of_the_connection("toutuitest", None, "one:1", false, false, 60);
        assert!(!no_answer.contains("Connected"), "{}", no_answer);
        assert!(no_answer.contains("no answer"), "{}", no_answer);

        // The name of the program takes the same rule.
        assert_eq!(the_name_of_the_program("0.7.58", 80), "🦜 Toutui v0.7.58");
        assert_eq!(the_name_of_the_program("0.7.58", 60), "🦜 v0.7.58");

        // **The three parts fit in 60 columns now.** The account holds 13 cells,
        // the library of the measurement holds 15, and the name of the program
        // holds 11.
        let of_the_account = "👋 toutuitest".chars().count();
        let of_the_program = the_name_of_the_program("0.7.58", 60).chars().count();

        assert!(of_the_account + 15 + of_the_program < 60);
    }

    /// **A text must not promise a function that the program does not have.**
    ///
    /// The sweep of two accounts of two servers of 2026-08-12 found that no key
    /// of the program added a second account, and T-118 gave the view a text
    /// that says what the program does. **T-124 gives the program the three
    /// keys**, therefore the text names each of them: a key that a view holds
    /// and that no text names is a key that no user finds (T-79).
    ///
    /// The text also names the cost of the two keys that start the program
    /// again: a playback stops with the process.
    #[test]
    fn the_text_of_the_accounts_names_every_key_of_the_view() {
        for key in ["The key a", "The key c", "The key l"] {
            assert!(
                THE_ACCOUNTS.contains(key),
                "the text names no {}: {}",
                key,
                THE_ACCOUNTS
            );
        }

        assert!(
            THE_ACCOUNTS.contains("▶"),
            "the text says nothing of the mark of the account that starts: {}",
            THE_ACCOUNTS
        );
        assert!(
            THE_ACCOUNTS.contains("A playback stops"),
            "the text says nothing of the playback that stops: {}",
            THE_ACCOUNTS
        );
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
    /// **The area of the footer holds two rows**, and the text wraps: the two
    /// rows of a terminal of 80 columns hold 160 cells, and a wrap of the words
    /// loses some of them at the end of the first row. A measurement of the real
    /// program on 2026-08-12 read every word of a footer of 116 characters in a
    /// terminal of 80 columns, on two rows. See T-90 and T-109.
    #[test]
    fn every_footer_fits_in_eighty_columns() {
        // **The gate measures the rows of the wrap, and not a count of
        // characters** (T-302). A count of 130 characters stood here before,
        // and it said nothing of the widths that the wrap of the words meets.
        let footers: Vec<String> = THE_FOOTERS_OF_THE_VIEWS
            .iter()
            .map(|one| one.to_string())
            .chain(the_footers_that_a_view_makes())
            .collect();

        for footer in footers {
            assert!(
                !footer.contains('\n'),
                "a footer holds one line, and the wrap of the view makes its rows: {}",
                footer
            );

            let rows = the_rows_of_a_footer(&footer, 80, 45);

            assert!(
                rows <= THE_SMALLEST_FOOTER,
                "the footer takes {} rows of a terminal of 80 columns: {}",
                rows,
                footer
            );
        }
    }

    /// A footer that is longer than two rows of the terminal takes the rows
    /// that it needs.
    ///
    /// **The measurement of T-302**, of the real program v0.8.130 inside tmux
    /// against the sandbox, in a terminal of 40 columns and 30 rows: the Home
    /// view said `j/k: move  l: play or open  Tab: home/library  S-Tab: the
    /// next library` on its two rows, and the keys `/: search`, `R: refresh`,
    /// `?: every key`, and `Q: quit` stood outside the screen. The view of the
    /// search lost `Q: quit`.
    #[test]
    fn a_footer_of_a_narrow_terminal_takes_the_rows_that_it_needs() {
        // 116 characters. Two rows of 40 columns hold 80 cells.
        let keys = FOOTER_OF_A_LIBRARY_OF_BOOKS;

        assert_eq!(
            the_rows_of_a_footer(keys, 40, 30),
            4,
            "the footer of the Home view needs four rows at 40 columns"
        );

        // A terminal that holds the whole footer in two rows keeps two rows: a
        // view that loses a line at 160 columns is a change that no fault asks
        // for.
        assert_eq!(the_rows_of_a_footer(keys, 80, 45), THE_SMALLEST_FOOTER);
        assert_eq!(the_rows_of_a_footer(keys, 160, 45), THE_SMALLEST_FOOTER);

        // **The footer must not take the view**: one half of the rows, and no
        // more.
        assert_eq!(the_rows_of_a_footer(keys, 10, 12), 6);

        // A view of few rows gives the floor, and a width of 0 gives it too.
        assert_eq!(the_rows_of_a_footer(keys, 10, 2), THE_SMALLEST_FOOTER);
        assert_eq!(the_rows_of_a_footer(keys, 0, 30), THE_SMALLEST_FOOTER);
    }

    /// Every footer of a view stands whole in a terminal of 40 columns.
    ///
    /// The narrowest terminal that a measurement of this fork uses is 40
    /// columns (T-300, T-301, and T-302). A view of 30 rows gives its footer no
    /// more than 15 of them, therefore a footer of no more than 600 characters
    /// stands whole there. This gate holds the class: a footer that grows past
    /// the room of the narrowest terminal loses its end to three points, and
    /// the user then reads no key of the road back.
    #[test]
    fn every_footer_of_a_view_stands_whole_in_forty_columns() {
        /// The rows of the narrowest terminal that this fork measures.
        const ROWS: u16 = 30;

        let footers: Vec<String> = THE_FOOTERS_OF_THE_VIEWS
            .iter()
            .map(|one| one.to_string())
            .chain(the_footers_that_a_view_makes())
            .collect();

        for footer in footers {
            let rows = the_rows_of_a_footer(&footer, 40, ROWS);

            assert_eq!(
                crate::logic::message::in_the_rows(&footer, 40, rows),
                footer,
                "the footer loses its end in a terminal of 40 columns: {}",
                footer
            );
        }
    }

    /// The footers of the reader hold one line, and they fit in the rows that
    /// the reader gives them.
    ///
    /// **The footer of the reader is a footer of this program** (T-301), and it
    /// stood outside the gate above: each of the four texts held a `\n` of its
    /// own, and the `Paragraph` of the reader had no wrap at all. A terminal of
    /// 40 columns therefore lost `t: contents`, `g/G: start/end`,
    /// `h: leave the book`, and `Q: quit`.
    ///
    /// **The limit of these footers is not the 130 characters above**: the
    /// footer of the reader takes the rows that its wrap needs (see
    /// `the_rows_of_the_footer` of `crate::ui::reader_tui`), therefore two rows
    /// of a terminal of 80 columns do not bind it. The narrowest terminal that
    /// a measurement of this fork used is 40 columns (T-300 and T-301), and the
    /// reader gives its footer four rows there: 160 cells.
    #[test]
    fn every_footer_of_the_reader_holds_one_line() {
        /// The largest number of characters of a footer of the reader: four
        /// rows of a terminal of 40 columns.
        const THE_WIDEST_FOOTER_OF_THE_READER: usize = 160;

        let footers = [
            FOOTER_OF_THE_READER,
            FOOTER_OF_THE_READER_OF_PAGES,
            FOOTER_OF_THE_CONTENTS,
            FOOTER_OF_THE_PAGES,
        ];

        for footer in footers {
            assert!(
                !footer.contains('\n'),
                "a footer holds one line, and the wrap of the reader makes its rows: {}",
                footer
            );

            let width = footer.chars().count();
            assert!(
                width <= THE_WIDEST_FOOTER_OF_THE_READER,
                "the footer of {} characters is too wide: {}",
                width,
                footer
            );

            // The road back of the reader stands in every one of them.
            assert!(
                footer.contains("h: leave the book"),
                "a footer of the reader names the key of the road back: {}",
                footer
            );
        }
    }

    /// The names of the things that a text of this program counts.
    ///
    /// A unit of measure (a byte, a second, a pixel) stays outside: `human_size`
    /// and `human_time` make those texts, and no view writes a number of them
    /// beside a name.
    const THE_THINGS_THAT_A_TEXT_COUNTS: &[&str] = &[
        "item",
        "track",
        "author",
        "genre",
        "session",
        "book",
        "episode",
        "chapter",
        "page",
        "picture",
        "file",
        "position",
        "bookmark",
        "narrator",
        "collection",
        "playlist",
        "line",
        "answer",
        "day",
        "shelf",
        "podcast",
        "media",
        "list",
        "name",
        "key",
        "view",
        "word",
        "address",
        "part",
    ];

    /// The macros that write in the log. A line of the log is for the
    /// maintainer, and `ui::keys::counted` is for the user.
    const THE_MACROS_OF_THE_LOG: &[&str] = &["info!", "warn!", "error!", "debug!", "trace!"];

    /// Gives the name of each thing that one line counts with a value.
    ///
    /// The function looks for a value of `format!` (`{}`, `{count}`, or
    /// `{:>3}`), then one space, then a word. The word `books` and the word
    /// `book(s)` both count books. **The rule holds every form**, and not the
    /// one form `{} items` that the test before it read. See T-108.
    fn the_things_that_a_line_counts(line: &str) -> Vec<String> {
        let letters: Vec<char> = line.chars().collect();
        let mut found: Vec<String> = Vec::new();
        let mut place = 0;

        while place < letters.len() {
            if letters[place] != '{' {
                place += 1;
                continue;
            }

            let Some(end) = (place..letters.len()).find(|&step| letters[step] == '}') else {
                break;
            };

            let mut after = end + 1;
            place = after;

            if after >= letters.len() || letters[after] != ' ' {
                continue;
            }

            after += 1;
            let start = after;

            while after < letters.len() && letters[after].is_ascii_alphabetic() {
                after += 1;
            }

            let word: String = letters[start..after].iter().collect();

            // "book(s)" holds the name and the plural together.
            let with_a_choice = letters[after..].starts_with(&['(', 's', ')']);

            let name = if with_a_choice {
                word.clone()
            } else if let Some(one) = word.strip_suffix('s') {
                one.to_string()
            } else {
                continue;
            };

            if THE_THINGS_THAT_A_TEXT_COUNTS.contains(&name.as_str()) {
                found.push(word);
            }
        }

        found
    }

    /// Gives the path and the text of every file of a directory of the source.
    fn the_files_of(directory: &str) -> Vec<(String, String)> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(directory);
        let mut wait = vec![root];
        let mut out: Vec<(String, String)> = Vec::new();

        while let Some(place) = wait.pop() {
            let Ok(entries) = std::fs::read_dir(&place) else {
                continue;
            };

            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_dir() {
                    wait.push(path);
                    continue;
                }

                if path.extension().and_then(|end| end.to_str()) != Some("rs") {
                    continue;
                }

                if let Ok(text) = std::fs::read_to_string(&path) {
                    out.push((path.display().to_string(), text));
                }
            }
        }

        assert!(!out.is_empty(), "the directory {} holds no file", directory);
        out
    }

    /// **One function holds the rule of "1 item".** A text that writes those
    /// words itself holds a second copy of the rule, and the copy of the view
    /// of the search said "1 items" for two years of releases (T-95).
    ///
    /// **The test before this one read the form `{} items` of four files**, and
    /// it did not find `{} item(s)` of the view of the lists (T-100 found that
    /// text by hand) or `{} sessions` of the view of the sessions. This test
    /// reads **every file of `src/ui` and of `src/logic`**, and it names every
    /// text that counts a thing in any form. See T-108.
    ///
    /// Two kinds of line stay outside: a line of the log, which the maintainer
    /// reads, and the tests of the file, which hold the words of an answer.
    #[test]
    fn no_text_of_a_view_counts_its_own_items() {
        let mut faults: Vec<String> = Vec::new();

        for (name, text) in the_files_of("src/ui")
            .into_iter()
            .chain(the_files_of("src/logic"))
        {
            // The tests of a file hold the words of an answer, and every test
            // of this program stands at the end of its file.
            let code = match text.find("mod tests {") {
                Some(place) => &text[..place],
                None => &text[..],
            };

            let lines: Vec<&str> = code.lines().collect();

            for (number, line) in lines.iter().enumerate() {
                // A macro of the log takes more than one line, therefore the
                // rule reads the three lines before this one too.
                let first = number.saturating_sub(3);
                let around = lines[first..=number].join("\n");

                if THE_MACROS_OF_THE_LOG
                    .iter()
                    .any(|macro_of_the_log| around.contains(macro_of_the_log))
                {
                    continue;
                }

                for thing in the_things_that_a_line_counts(line) {
                    faults.push(format!(
                        "{}:{} counts \"{}\" itself: {}",
                        name,
                        number + 1,
                        thing,
                        line.trim()
                    ));
                }
            }
        }

        assert!(
            faults.is_empty(),
            "these texts count a thing themselves. Use ui::keys::counted:\n{}",
            faults.join("\n")
        );
    }

    /// The reader of the guard must find every form of such a text.
    #[test]
    fn the_guard_finds_every_form_of_a_text_that_counts() {
        assert_eq!(
            the_things_that_a_line_counts("format!(\"[{} items]\", n)"),
            vec!["items"]
        );
        assert_eq!(
            the_things_that_a_line_counts("\"{} - {} book(s) - Duration: {}\""),
            vec!["book"]
        );
        assert_eq!(
            the_things_that_a_line_counts("\"{count} files, and {MAX} files.\""),
            vec!["files", "files"]
        );
        assert_eq!(
            the_things_that_a_line_counts("\"{} sessions of {}\""),
            vec!["sessions"]
        );

        // A word that names no thing of this program is not a fault, and
        // `counted` itself is the answer and not the fault.
        assert!(the_things_that_a_line_counts("\"🔗 {} does not answer\"").is_empty());
        assert!(the_things_that_a_line_counts("\"{} bytes of {}\"").is_empty());
        assert!(the_things_that_a_line_counts("format!(\"{} {}s\", count, name)").is_empty());
        assert!(the_things_that_a_line_counts("\"1 item\"").is_empty());
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
            the_text_of_the_home_view_with_no_line(false, None),
            THE_HOME_VIEW_WITH_NO_LINE
        );
        assert_eq!(
            the_text_of_the_home_view_with_no_line(true, None),
            THE_HOME_VIEW_WITH_NO_ANSWER
        );

        // A server that does not answer comes before every other reason, and
        // before the filter too.
        assert_eq!(
            the_text_of_the_library_view_with_no_line(true, false, true, true, None),
            THE_LIBRARY_WITH_NO_ANSWER
        );

        // The filter comes before the library, because the library holds media.
        assert_eq!(
            the_text_of_the_library_view_with_no_line(false, false, true, false, None),
            THE_LIBRARY_WITH_A_FILTER
        );

        assert_eq!(
            the_text_of_the_library_view_with_no_line(false, false, false, true, None),
            THE_LIBRARY_OF_PODCASTS_WITH_NO_MEDIA
        );
        assert_eq!(
            the_text_of_the_library_view_with_no_line(false, false, false, false, None),
            THE_LIBRARY_WITH_NO_MEDIA
        );

        // **The request that came back with a fault is a condition of its
        // own**, and it stands above the filter and the library: the program
        // knows nothing of that library (T-170).
        let text = the_text_of_the_home_view_with_no_line(false, Some("Status 500."));

        assert!(
            text.starts_with("The server did not give the shelves of this library:"),
            "{}",
            text
        );
        assert!(text.contains("Status 500."), "{}", text);

        let text = the_text_of_the_library_view_with_no_line(
            false,
            false,
            true,
            false,
            Some("Status 500."),
        );

        assert!(
            text.starts_with("The server did not give the media of this library:"),
            "{}",
            text
        );
        assert!(
            !text.contains("agrees with the filter"),
            "a filter says nothing of a list that never came: {}",
            text
        );
        assert!(
            !text.contains("Press L"),
            "a text must not promise a key that does no work of this fault \
             (T-118): {}",
            text
        );

        // The server that does not answer stands above them all: no request
        // went at all in that mode (T-25).
        assert_eq!(
            the_text_of_the_library_view_with_no_line(
                true,
                false,
                false,
                false,
                Some("Status 500.")
            ),
            THE_LIBRARY_WITH_NO_ANSWER
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
