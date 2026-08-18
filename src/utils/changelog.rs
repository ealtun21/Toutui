//! The changelog that the settings show. The key `S` and then "About and
//! changelog" gives it to the user.
//!
//! **Every release of this fork holds one entry**, and the entry names its own
//! version. The entries of the fork stood in ten local values of one function
//! before, and the newest of them took the version of the build: the screen
//! therefore said "Changelog Toutui v0.7.46" above the words of v0.6.9, and 38
//! releases reached no user. See T-101.
//!
//! A release writes its entry in the words of a user, and not in the words of
//! the code: "The keys c and p make a collection or a playlist" and not "T-88".
//! Four tests of this module hold the rules of an entry.

/// The version of this build.
///
/// A test holds the newest entry of the changelog to this value, therefore a
/// release that writes no entry fails the gate. See T-101.
#[cfg(test)]
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// One entry of the changelog: one release of this fork.
struct Entry {
    /// The version of the release, as `Cargo.toml` holds it.
    version: &'static str,
    /// The day of the release, as DD/MM/YYYY.
    date: &'static str,
    /// The words for the user: one line for each name of a group ("Added:"),
    /// one line for each item, and an empty line between two groups.
    ///
    /// **One item of the list is one line of the text**, and the view wraps it:
    /// a paragraph of ratatui breaks a line that is too long, and it never joins
    /// two lines. A body that holds the wrap of the source therefore gives a
    /// column of 65 letters in a terminal of 200.
    body: &'static [&'static str],
}

/// Every entry of this fork, the newest first.
///
/// **A new release puts its entry at the top of this list**, and it names its
/// own version. The versions 0.6.6 and 0.7.25 have no release, and
/// `THE_VERSIONS_WITH_NO_RELEASE` of the tests holds the reason.
const THE_ENTRIES_OF_THE_FORK: &[Entry] = &[
    Entry {
        version: "0.8.217",
        date: "18/08/2026",
        body: &[
            "Fixed:",
            "- **An author of the server with no identity takes no line.** \
             A row of the answer of the authors that lost its identity kept \
             its line, and the key of that line wrote a filter of no value \
             into the account. The server then answered 0 items, the view \
             said that no media agrees with the filter, and the view of the \
             filter could not show it. The line now goes away, a narrator \
             with no name goes with it, the start does not apply a filter \
             of no value, and the log says why.",
        ],
    },
    Entry {
        version: "0.8.216",
        date: "18/08/2026",
        body: &[
            "Fixed:",
            "- **The header of no filter names no filter.** The second row \
             of the header of a terminal of 84 to 119 columns said \
             `▣ No filter` at the start of the program and after a removal \
             of the filter. The words said a filter when no filter stands. \
             The header now holds the sequence alone, and a filter that \
             stands keeps its mark and its name.",
        ],
    },
    Entry {
        version: "0.8.215",
        date: "18/08/2026",
        body: &[
            "Fixed:",
            "- **A change of the sequence does not erase the filter of \
             another library.** The program keeps a filter of another \
             library and a filter that a library of podcasts ignores out of \
             the view, and the account keeps the filter. One change of the \
             sequence in such a library then erased the filter, its name, \
             and its library together. The write of the sequence now keeps \
             the filter of the account, and the library of the filter gives \
             it back.",
        ],
    },
    Entry {
        version: "0.8.214",
        date: "18/08/2026",
        body: &[
            "Fixed:",
            "- **A filter of an author stays in its library.** The value of \
             a filter of an author and of a series holds an identity of one \
             library: the filter rode into a second library of books, the \
             server answered 0 items, and the view said that no media agrees \
             with the filter. The account now keeps the library of the \
             filter, such a filter stays out of a request of another \
             library, and the library of the filter gives it back.",
        ],
    },
    Entry {
        version: "0.8.213",
        date: "18/08/2026",
        body: &[
            "Fixed:",
            "- **A library of podcasts does not take a filter that the \
             server ignores.** The server ignores a filter of an author, of \
             a series, of a narrator, of a publisher, and of the position in \
             a library of podcasts, and it answers every item: the header \
             then named a filter that did not act. Such a filter now stays \
             out of the request and out of the header, the view of the \
             filter of such a library offers no choice of the position, and \
             a library of books gives the filter back.",
        ],
    },
    Entry {
        version: "0.8.212",
        date: "18/08/2026",
        body: &[
            "Fixed:",
            "- **The start of the program names the filter of an old \
             database.** A row that a version before 0.8.211 wrote holds the \
             value of a filter of an author or of a series and no name: the \
             header of the start named the group, for example \"An author\", \
             at every start. The start now asks the server for the name one \
             time, and the account then keeps it.",
        ],
    },
    Entry {
        version: "0.8.211",
        date: "18/08/2026",
        body: &[
            "Fixed:",
            "- **The start of the program names the filter of an author and \
             of a series.** The name of such a filter came of the moment of \
             the application alone, and it went away when the program \
             stopped: the header of the next start named the group, for \
             example \"An author\", and not \"Lewis Carroll\". The account \
             now keeps the name of the filter beside its value, and the \
             header of the start reads it back.",
        ],
    },
    Entry {
        version: "0.8.210",
        date: "18/08/2026",
        body: &[
            "Fixed:",
            "- **The header names the filter that the user took.** An \
             application of a filter refreshes the whole screen, and the \
             refresh forgot the names of the filters of the server: the \
             header then named the group, for example \"A genre\", and not \
             the name that the user just took. The header now reads the name \
             of a genre, of a tag, of a narrator, of a language, and of a \
             publisher out of the filter itself — at the start of the \
             program too — and it keeps the name of an author and of a \
             series at the moment of the application.",
        ],
    },
    Entry {
        version: "0.8.209",
        date: "18/08/2026",
        body: &[
            "Fixed:",
            "- **The name of a filter stands in one row of the header.** A \
             genre, a tag, or a narrator of the server can hold an end of a \
             line, and the header of the screen then showed the first line \
             of that name alone, with no mark that a word was gone. The name \
             now takes one space for each end of a line, as the rows of the \
             lists do.",
        ],
    },
    Entry {
        version: "0.8.208",
        date: "18/08/2026",
        body: &[
            "Fixed:",
            "- **A name of the server stands in one row of the statistics \
             view and of the sessions view.** A title of the server can hold \
             an end of a line, and the two views drew it in two bad ways: \
             the statistics view glued the words of the two lines together, \
             with no space between them, and the sessions view broke the row \
             at the end of the line — the second part then read as a session \
             of its own, with no time at all. Every name of the server of \
             the two views now takes one space for each end of a line, as \
             the rows of the lists do.",
        ],
    },
    Entry {
        version: "0.8.207",
        date: "18/08/2026",
        body: &[
            "Fixed:",
            "- **The title of a view whose name holds an end of a line stands \
             in one row.** A title of the server can hold an end of a line, \
             and the screen draws no such character: the words of the two \
             lines glued together in the title of a view, with no space \
             between them — a book of the title \"Alpha\", an end of a line, \
             and \"OMEGAEND\" gave the view `The bookmarks of \
             \"AlphaOMEGAEND\"`, while the row of the same book in the list \
             said `Alpha OMEGAEND`. Every title of a view now takes one \
             space for each end of a line, as the rows of the lists do.",
        ],
    },
    Entry {
        version: "0.8.206",
        date: "17/08/2026",
        body: &[
            "Fixed:",
            "- **The key l of a search with no hit says why, and it opens no \
             view.** The cursor of the view of the search stands at the \
             first line from the start of the program, therefore a search \
             with no hit held a cursor over a line that the view does not \
             have. The key l of such a search of a library of podcasts \
             opened the view of the episodes of no podcast at all, and that \
             view said \"The program gets the episodes of this podcast…\" \
             for a request that the program never made: the words stood for \
             ever. The same key of a library of books did nothing and said \
             nothing. The key now says \"This line holds no media.\" and it \
             stays in the view of the search, and a search with a hit keeps \
             the work of the key.",
        ],
    },
    Entry {
        version: "0.8.205",
        date: "17/08/2026",
        body: &[
            "Fixed:",
            "- **A fact of the cover panel whose value holds an end of a \
             line stands in one row, and the bar of the progress stays in \
             the panel.** The values of the facts come of the server, and a \
             server can hold a genre, a series, or a narrator of more than \
             one line. The panel gave such a value a second row with no \
             label, and the budget of the rows of the panel counts the \
             lines: the last row of the panel — the bar of the progress — \
             then fell out of it. A genre of the server of two lines gave \
             the row `Genre     Alpha`, a row of `OMEGAEND, Adventure` under \
             it, and no bar for a book of 50 percent. Every end of a line of \
             a value now takes one space, as every list of the program gives \
             it (T-311), and the bar stays.",
        ],
    },
    Entry {
        version: "0.8.204",
        date: "17/08/2026",
        body: &[
            "Fixed:",
            "- **The title of a view with no line that the screen cuts keeps \
             its start, and it says that the screen cut it.** The title of \
             such a view holds the name of its list, and that name can come \
             of the server: the bookmarks of \"A Book Of An Epub With No \
             Container\" hold a title of 65 characters. The screen draws a \
             centered title that is wider than its box in a smaller area, at \
             the right of it, therefore the title lost its start and its end \
             together with no mark: a terminal of 40 columns said `With No \
             Container\" [0 items]` and no word of it named the view. A view \
             with its lines cuts its title already, and the two roads of a \
             view with no line now cut it in the same way: the title keeps \
             its start, which names the view, and the three points say that \
             the screen cut it.",
        ],
    },
    Entry {
        version: "0.8.203",
        date: "17/08/2026",
        body: &[
            "Fixed:",
            "- **The row of the seek of the band of the player that the screen \
             cuts says that the screen cut it.** A band that is too narrow for \
             a bar says the place of the user and the length of the media \
             alone, and those two times hold 18 columns: a band of fewer than \
             20 columns therefore lost the end of them with no mark at all. A \
             terminal of 16 columns said a length of `8:0` for a book of \
             `8:00:00`, and a terminal of 12 columns said no length at all, \
             while the row of the words above it and the row of the keys under \
             it each said the three points. That row now says the three points \
             too, and it keeps the place of the user, which is the value of it.",
        ],
    },
    Entry {
        version: "0.8.202",
        date: "17/08/2026",
        body: &[
            "Fixed:",
            "- **A line of the screen of the start that the screen cuts says \
             that the screen cut it.** That screen names the program, the \
             address of the server, the step that runs, and an advice for a \
             wait that is long, and each of them took a line of its own with no \
             wrap at all. A line that was wider than the box lost its end with \
             no mark: a terminal of 40 columns said `The server is slow. The \
             program waits ` and no word after it, and a terminal of 50 columns \
             said `The server is slow. The program waits for the an`. The advice \
             of a server that does not answer holds 89 columns, and the box \
             holds at most 70 of them: the words `Press Q to stop.` of it \
             therefore reached no terminal at all. Every line of that screen now \
             goes over the rows under it, and it keeps every word. An address \
             that is longer than the box takes that same rule, because a screen \
             that cuts an address says an address that the user does not have.",
        ],
    },
    Entry {
        version: "0.8.201",
        date: "17/08/2026",
        body: &[
            "Fixed:",
            "- **A line of the facts of the panel of the cover that the screen \
             cuts says that the screen cut it.** The panel names the series, \
             the author, the narrator, the year, the time, the day of the \
             start, the genre, the files, the ebook, the disk, and the place of \
             the user, and each of them takes a line of its own. A value of the \
             server that is longer than the panel lost its end with no mark at \
             all: a terminal of 85 columns said `Genre     Science Fiction & \
             Fant` for a genre of the name `Science Fiction & Fantasy, Fantasy, \
             Fairy Tales`, and `Series    Depthless Hunger, Book` for a series \
             of the name `Depthless Hunger, Book #2`. The user therefore read a \
             genre and a name of a series that the media does not have. Every \
             line of the panel now keeps its start and it takes three points \
             for the end that the panel cannot hold, which is the rule of a \
             line of a list and of a row of the band of the player already.",
        ],
    },
    Entry {
        version: "0.8.200",
        date: "17/08/2026",
        body: &[
            "Fixed:",
            "- **A row of the band of the player that the screen cuts says that \
             the screen cut it.** The row of the media and the row of the keys \
             of the player each lost the columns of their end with no mark at \
             all: a terminal of 60 columns said `Many Hours A` for an author of \
             the name `Many Hours Author`, and a terminal of 80 columns said \
             `o/i: vol +/`, which is a key and no word of its work. The row of \
             the keys holds 99 columns, therefore every terminal under 102 \
             columns lost the keys `t: sleep` and `Y: quit` of it, and no user \
             read them in any form. The two rows now keep their start and they \
             take three points for the end that the band cannot hold, which is \
             the rule of the title of a panel and of a line of a list already.",
        ],
    },
    Entry {
        version: "0.8.199",
        date: "17/08/2026",
        body: &[
            "Fixed:",
            "- **A line of a list that the screen cuts says that the screen cut \
             it.** A line of a list stands on one row of its panel, and a line \
             that was wider than that panel lost the columns of its end with no \
             mark at all: the Collections view of a terminal of 40 columns said \
             `[Collection] A Test Collection [1 item`, therefore the user read a \
             number of the items that the collection does not have. Every line \
             of every list now keeps its start and it takes three points for the \
             end that the screen cannot hold, which is the rule of the title of \
             a panel and of the columns of the table of a media already.",
        ],
    },
    Entry {
        version: "0.8.198",
        date: "17/08/2026",
        body: &[
            "Fixed:",
            "- **The panel of the views names no view of a podcast in a library \
             of books.** The server downloads the episodes of a podcast alone, \
             and the panel 1 of a library of books still named the view of the \
             downloads of the server: the key `l` of that line, and the key \
             `d`, each said `This library holds books. The server downloads \
             the episodes of a podcast only.` and gave no view at all. The \
             panel now names the views of the library that stands in the two \
             directions, therefore a library of books names no view of a \
             podcast and a library of podcasts names no view of a book.",
        ],
    },
    Entry {
        version: "0.8.197",
        date: "17/08/2026",
        body: &[
            "Fixed:",
            "- **The panel of the views names the series of the library.** The \
             program holds the Series view, and the key `s` of the Home view \
             and of the Library view opens it, but the panel 1 of the views \
             named that view nowhere: the panel held fourteen lines, and the \
             user of it read the key of the series in the list of every key \
             alone. The panel now holds the line `Series s` at its third line, \
             which is the line of the design, and the key `l` of that line \
             opens the list of the series. A library of podcasts holds no \
             series, therefore the panel of such a library names that view \
             nowhere.",
        ],
    },
    Entry {
        version: "0.8.196",
        date: "17/08/2026",
        body: &[
            "Fixed:",
            "- **The panel of the views names no view that the library does \
             not have.** A library of podcasts holds no author and no \
             narrator, and the panel 1 of such a library still named the view \
             of the authors and the view of the narrators. The footer of that \
             panel says `l: open the view`, and the key `l` of those two lines \
             gave no view at all: it said `A library of podcasts has no \
             author.` and it took the focus back to the panel of the list. The \
             panel now names the views of the library that stands, which is \
             the rule that the panel 2 of the sequence holds already. The key \
             `G` of it takes the last line of that panel, and a click under \
             that line moves no line. A library of books keeps every view.",
        ],
    },
    Entry {
        version: "0.8.195",
        date: "17/08/2026",
        body: &[
            "Fixed:",
            "- **The footer of a panel of the stack names no list of a view \
             that holds none.** The keys `h` and `4` of the panel 1, of the \
             panel 2, and of the panel 3 take the focus back to the panel 4, \
             and that panel holds the list of the media. A library with no \
             media gives it one sentence and no list at all, and the three \
             footers still read `h: the list` and `4/Ctrl+l: the list` over a \
             panel that said `This library holds no media.` The word now says \
             what the panel holds: `the view` while the view holds no line, \
             and `the list` while it holds one. The key does the same work in \
             the two conditions.",
        ],
    },
    Entry {
        version: "0.8.194",
        date: "17/08/2026",
        body: &[
            "Fixed:",
            "- **The view of the statistics and the view of the sessions keep \
             every number in a narrow terminal.** The two views draw a text \
             that takes no wrap, therefore a line that is longer than the \
             panel lost its end with no mark of a cut. **A cut of a number \
             gives another number**: at 40 columns the time of a media read \
             `(1 ` for `(1 h 26 min)`, the size of a year read `892.6` with no \
             unit for `892.6 MB`, the time of the account read `13 h ` for `13 \
             h 33 min`, and the facts of a library read `5 g` for `5 genres`. \
             The view of the sessions lost the name of each media at the same \
             column, and six sessions of six different times then read `A \
             Second Book Of Many` together. Every line of the two views now \
             takes the rows that it needs, and a row after the first stands at \
             an indent of four. The field of the time of a session keeps its \
             column, a line that stands in the screen keeps its one row, and \
             the keys `j` and `k` reach every row. A terminal of 160 columns \
             draws the two views as it drew them before.",
        ],
    },
    Entry {
        version: "0.8.193",
        date: "17/08/2026",
        body: &[
            "Fixed:",
            "- **The view of every key of the program says what each key does in \
             a narrow terminal.** A line of a list stands on one row and it \
             takes no wrap, therefore the work of a key that is longer than \
             the panel lost its end with no mark of a cut. A terminal of 40 \
             columns gave that work 16 columns: the view said `The focus goes \
             to` for six keys, `Hide the panels 1,` for the key `z`, and `The \
             line of the po` for a click of the mouse, and **no key of the 83 \
             keys of the program said what it does**. The name of a group went \
             the same road, and the words `and more, Home and Library` of the \
             group of the panels stood outside the screen. The work of a key \
             now takes the rows that it needs: the two columns of the design \
             hold while the work has 20 columns beside the key, and a panel \
             that is narrower draws the key on a row of its own with the work \
             under it. The keys `j`, `k`, and `G` of that view move over those \
             rows. A terminal of 160 columns draws every row of the view as it \
             drew it before.",
        ],
    },
    Entry {
        version: "0.8.192",
        date: "17/08/2026",
        body: &[
            "Fixed:",
            "- **A view with no line keeps every word of its reason in a narrow \
             terminal.** The reason of eleven views stood in the name of the \
             panel, and a name holds no wrap: a terminal of 40 columns \
             therefore cut the sentence and the user lost the key of the work. \
             The bookmarks of a media that holds no bookmark said `\"A Book Of \
             An Epub With No Container\" h…` and no more, the queue said `The \
             queue is empty. Press n on a media…`, the chapters said `No media \
             plays now. A media that plays…`, and a search of no hit said `The \
             server found nothing for \"zzqqxnoth…`. The rows under each of \
             those held nothing at all. Every one of the eleven views now names \
             its list in the panel, for example `The queue [0 items]`, and it \
             says the whole reason under that name, over as many rows as the \
             sentence needs. The eleven: the bookmarks, the queue, the chapters, \
             the search, the authors, the narrators, the view that puts a media \
             in a list, the downloads of the server, the devices of an \
             e-reader, the ebooks of a media, and a new podcast. A view of one \
             line or more keeps the list and the name that it held before.",
        ],
    },
    Entry {
        version: "0.8.191",
        date: "17/08/2026",
        body: &[
            "Fixed:",
            "- **Six more views with no line name no key of a line.** The \
             bookmarks of a media that holds no bookmark said `j/k: move  l: go \
             to the place  X: remove the bookmark`, the view that puts a media \
             in a list said `j/k: move  l: put it here` for a library with no \
             collection and no playlist, and the view of the downloads of the \
             server said `j/k: move  X: empty the queue of this podcast` for an \
             empty queue. The view of the devices of an e-reader, the view of \
             the ebooks of a media, and the view of a new podcast each said the \
             same. Those six views now name the keys that they hold, for \
             example `h: back  ?: every key  Q: quit` for the bookmarks and `c: \
             a collection  p: a playlist  h: back  ?: every key  Q: quit` for \
             the view that puts a media in a list, because the keys `c` and `p` \
             make the first list of a library. A view of one line or more keeps \
             every key that it held before.",
        ],
    },
    Entry {
        version: "0.8.190",
        date: "17/08/2026",
        body: &[
            "Fixed:",
            "- **The footer of a view with no line names no key of a line.** \
             The footer of the view of the collections and the playlists of a \
             library with no collection said `j/k: move  l: the media  r/D: a \
             name/description  X: remove`, and no line of that view holds a \
             media, a name, or a description: four of the seven keys of that \
             footer did nothing at all. The view of the series, the view of the \
             authors, the view of the narrators, the view of the chapters of a \
             program that plays nothing, the view of an empty queue, and the \
             two views of the library each said the same. The footer of a view \
             with no line now names the keys of that view alone, for example \
             `h: back  ?: every key  Q: quit`, and the footer of a view of one \
             line or more keeps every key that it held before.",
        ],
    },
    Entry {
        version: "0.8.189",
        date: "17/08/2026",
        body: &[
            "Fixed:",
            "- **A view with no line says the name of the list that holds no \
             line.** The view of the collections and the playlists, the view of \
             the series, and the view of the episodes of a podcast each drew a \
             line over the whole width of the screen with no word in it while \
             they held no line, and no word of the screen then said which list \
             is empty. The same view with its lines says the name of that list \
             in that same line, for example `Collections and playlists [2 \
             items]`, therefore the two roads of one view said two different \
             things. Each of the three views now says the name of its list and \
             the number of its lines, for example `Series [0 items]`, above the \
             sentence that says why it holds no line.",
        ],
    },
    Entry {
        version: "0.8.188",
        date: "17/08/2026",
        body: &[
            "Fixed:",
            "- **Six views with no line take no column of the covers.** The \
             view of the series, the view of the books of a series, the view of \
             the collections, the view of the media of a collection, the view \
             of a search, and the view of the episodes of a podcast each held a \
             panel of the cover with no character in it while they held no \
             line: that panel took 64 columns of a screen of 160 and 40 rows, \
             and the sentence that says why the view holds no line said its \
             words in the columns that stayed. The panel of the cover holds a \
             picture of a media and the facts of it, and a view of no line \
             gives it no media at all. The two views of the frame of the \
             panels held this rule from the version 0.8.185, and the six views \
             hold it now. A media that plays keeps that panel.",
        ],
    },
    Entry {
        version: "0.8.187",
        date: "17/08/2026",
        body: &[
            "Fixed:",
            "- **A click of the panel of the list of a view with no line \
             names that panel.** A view that holds no line says why, and no \
             click of the mouse in that panel did anything at all: the \
             program wrote the place of that panel for the mouse only when it \
             drew the lines of a list in it, therefore the place stayed the \
             place of the frame before it. The key of the next library makes \
             the program read its lists again, and the place of the panel was \
             then nothing at all. A click of that panel gives it the focus \
             now, and it reads no row of a list that the view does not hold.",
        ],
    },
    Entry {
        version: "0.8.186",
        date: "17/08/2026",
        body: &[
            "Fixed:",
            "- **The panel of the list of a view with no line keeps its \
             border, its number, and its name.** A view that holds no line \
             says why, and that sentence stood under one line at the top with \
             no border around it: the panels of the stack at the left of it \
             each said their number and their name, and the panel of the list \
             said neither. A user who cannot read the number of a panel \
             cannot press the digit of it, and the key of the focus then gave \
             a panel that the screen did not show. The sentence stands inside \
             the panel of the list now, with the title of that panel and with \
             the border of its focus, and a terminal that holds no frame of \
             the panels keeps the line at the top that it had.",
        ],
    },
    Entry {
        version: "0.8.185",
        date: "17/08/2026",
        body: &[
            "Fixed:",
            "- **A view with no line takes no panel of the cover and no panel \
             of the gallery.** The two panels hold the picture of a media and \
             the cells of the media beside it, therefore a view that holds no \
             line gives them no media at all. They stood with no character in \
             them: a terminal of 160 columns and 45 rows gave the Library view \
             of a library with no media a panel of the cover of 8 rows and a \
             panel of the gallery of 32 rows, the two of them took 48 columns, \
             and the reason of the view said its two lines in the 74 columns \
             that stayed. The reason takes the whole width of the view now, and \
             a media that plays keeps the two panels.",
        ],
    },
    Entry {
        version: "0.8.184",
        date: "17/08/2026",
        body: &[
            "Fixed:",
            "- **The panel of the cover of a media with no picture leaves the \
             gallery the rows that its words cannot use.** A media that the \
             server holds with no cover keeps no row for a picture, therefore \
             the words are every row that the panel can use. The panel took \
             the share of the design and the rows after the words then said \
             nothing at all: a terminal of 160 columns and 45 rows gave that \
             panel 18 rows inside its border, the words took 4 of them, 14 \
             rows held no character, and the gallery under it showed 12 covers \
             of a library of 2056 books. The panel now holds 5 rows, and the \
             gallery shows 20 covers in 5 bands. A media with a picture keeps \
             the share of the design, because the picture takes every row that \
             the words leave, and a description of many lines keeps that share \
             too, because the description scrolls with the keys J and K.",
        ],
    },
    Entry {
        version: "0.8.183",
        date: "17/08/2026",
        body: &[
            "Fixed:",
            "- **The panel of the cover takes no column that its picture \
             cannot use.** The panel keeps its width under the width that its \
             height can hold, because a wider panel gives the picture no pixel \
             and it takes columns of the list. That limit read the rows of the \
             whole panel, and the picture stands inside the border: the panel \
             therefore opened two columns wider than the widest picture that \
             it can hold. A terminal of 160 columns gave the panel 28 columns \
             for a picture of 24 at a screen of 19 rows, 40 for a picture of \
             16 at 25 rows, and 50 for a picture of 26 at 30 rows. The panel \
             now holds 26, 38, and 48 columns at those screens, the picture \
             keeps every column that it had, and the list takes the two \
             columns that stay.",
        ],
    },
    Entry {
        version: "0.8.182",
        date: "17/08/2026",
        body: &[
            "Fixed:",
            "- **The panel of the cover goes away at a screen that cannot hold \
             its picture.** The panel stood at a screen of eight rows, and the \
             border of it took two of them: the picture then had six rows, the \
             render of the picture needs eight, and the panel therefore held no \
             character at all. A terminal of 160 columns and 13 rows gave that \
             empty panel 22 columns of the list. The panel of a media that \
             holds a picture now needs the rows of the border and the rows of \
             the picture together, and the list takes those columns while the \
             screen is too low. The panel of a media that the server holds with \
             no cover does not change.",
        ],
    },
    Entry {
        version: "0.8.181",
        date: "17/08/2026",
        body: &[
            "Fixed:",
            "- **The gallery of the covers takes no row that the facts of the \
             media need.** The panel of the gallery stands under the panel of \
             the cover, and it kept three rows for the facts of that panel. \
             The facts take a line each, and a book can give nine of them: a \
             terminal of 160 columns and 28 rows therefore lost the place of \
             the user, the bar of the progress, and two lines more, while the \
             same terminal of 27 rows said all of them. The gallery goes away \
             before the panel of the cover cuts a line now.",
            "- **The gallery stands in more terminals for a media that has no \
             cover.** The rows that the gallery kept for the panel of the \
             cover held a picture always. A media that the server holds with \
             no cover at all needs no such row: the panel of it said its \
             three facts over three rows, twenty rows of it held nothing, and \
             no gallery stood under it. The gallery reads the rows that the \
             panel needs now.",
        ],
    },
    Entry {
        version: "0.8.180",
        date: "17/08/2026",
        body: &[
            "Fixed:",
            "- **The panel of the cover of a media that has no cover stands in \
             a terminal of few rows.** That panel went away when it had fewer \
             rows than a picture needs. A media that the server holds with no \
             cover at all shows the words of the media alone there, and those \
             words need the three facts of the media and no row of a picture: \
             a terminal of 160 columns and 12 rows therefore lost the panel, \
             the list of it lost three lines, and the two lines under that \
             list said less than the panel said. The panel of such a media \
             stands while it has room for its facts now, and a panel that \
             holds a picture keeps the height that it had.",
        ],
    },
    Entry {
        version: "0.8.179",
        date: "17/08/2026",
        body: &[
            "Fixed:",
            "- **The panel of the cover of a media that has no cover says its \
             words whole in a terminal of few rows.** That panel takes the \
             columns that the picture in it can use, because a panel that is \
             wider gives the picture no more pixels. A media that the server \
             holds with no cover at all shows the words of the media alone \
             there, and the height of the panel then says nothing about the \
             columns that those words need: a terminal of 160 columns and 16 \
             rows gave that panel 22 columns, and it cut the size of the files \
             in the middle of a number. The panel of such a media keeps the \
             columns that it has in a tall terminal now, and a panel that \
             holds a picture keeps the limit that it had.",
        ],
    },
    Entry {
        version: "0.8.178",
        date: "17/08/2026",
        body: &[
            "Fixed:",
            "- **A terminal of few rows shows a line of the list again in a \
             screen of 120 columns and more.** The panel of the list of the \
             Home view and of the Library view holds a border of four sides in \
             such a screen, and the rows that the program kept for that list \
             were the rows of a border at the top alone: a terminal of 6 rows \
             and fewer therefore held the two rows of the border and no line at \
             all, and a terminal of 8 rows gave a row to the panel of the item \
             while the list had no line. The program keeps the rows of the \
             border of the panel now. A terminal of 9 rows and more stands in \
             the shape that it had.",
        ],
    },
    Entry {
        version: "0.8.177",
        date: "17/08/2026",
        body: &[
            "Fixed:",
            "- **A terminal of few rows says the messages of the program \
             again.** The row of the message read the two rows that the text of \
             the footer wants, and a terminal of 3 rows and fewer gives that \
             footer fewer rows: a terminal of 2 rows and a terminal of 1 row \
             therefore said nothing at all, and a terminal of 3 rows wrote the \
             message over the title of the list while the row above the footer \
             stayed free. The message reads the rows of the header and of the \
             footer of the same frame now. A terminal of 4 rows and of 5 rows \
             says the whole sentence of a message in the rows that its header \
             no longer holds, and a terminal of 6 rows and more stands in the \
             shape that it had.",
        ],
    },
    Entry {
        version: "0.8.176",
        date: "17/08/2026",
        body: &[
            "Fixed:",
            "- **The work of a view no longer goes away before the header, the \
             row of the message, and the footer.** A terminal of 5 rows held \
             the two rows of the header, one blank row, and the two rows of \
             the footer, and it held no title of the list and no line of it at \
             all; a terminal of 3 rows held the footer alone; and a terminal \
             of 1 row held no letter at all. The three parts that stand around \
             a view give way now, in the sequence of what they say to the \
             user: the header first, the row of the message after it, and the \
             footer last. The work of the view keeps its border and one line, \
             therefore a terminal of 2 rows shows the list of the view. A \
             screen of 7 rows and more stands in the shape that it had.",
        ],
    },
    Entry {
        version: "0.8.175",
        date: "17/08/2026",
        body: &[
            "Fixed:",
            "- **The list of seven views no longer goes away before the panel \
             under it.** The Authors view, the view of the lists that take a \
             media, the view of the devices of an e-reader, the view of the \
             downloads of the server, the view of the ebooks of a media, the \
             view of a new podcast, and the settings of the reader each gave \
             that panel four or five rows of a fixed number, therefore a \
             terminal of few rows gave the panel its rows first and the list \
             took what stayed: at 100 columns and 8 rows the Authors view of a \
             library of nine authors said `No description available` and \
             nothing else — no title, and no author at all. The panel takes no \
             row that the list needs for its border and one line now, and a \
             screen that held the whole panel keeps it.",
        ],
    },
    Entry {
        version: "0.8.174",
        date: "17/08/2026",
        body: &[
            "Fixed:",
            "- **The band of the player no longer writes over the lines of a \
             view.** Fifteen views gave the band of the player no row of their \
             layout, therefore it drew over their last six lines: the reader of \
             an ebook lost six lines of the page and the line under the band \
             went on with the text, and the same road took six lines of the \
             chapters, of the queue, of the bookmarks, of the table of the \
             keys, and of ten views beside them. The band stands under the work \
             of every view now.",
            "- **A terminal of few rows shows the list again while a media \
             plays.** The band of the player and the two bars of the chapters \
             took their rows before the list, therefore a screen of 8 rows held \
             no line of it. The work of the view goes away last: the band takes \
             the rows that stay, and it goes away when it has room for no word \
             of the media. A screen of 13 rows and more does not change.",
        ],
    },
    Entry {
        version: "0.8.173",
        date: "17/08/2026",
        body: &[
            "Fixed:",
            "- **A terminal of few rows shows the list of the view again.** The \
             row that says the author, the year, and the length of the media of \
             the cursor took its two rows before the list, therefore a screen \
             of 8 rows said `Library [500 items of 2056]` and it showed no one \
             of those items. The list is the work of the view, and it now keeps \
             its line first. A screen of 9 rows and more does not change.",
        ],
    },
    Entry {
        version: "0.8.172",
        date: "17/08/2026",
        body: &[
            "Fixed:",
            "- **The header of a narrow screen no longer writes one part over \
             another.** The account, the name of the library, and the name of \
             the program are three texts of one row, and the program measured \
             none of them: a screen of 40 columns read \
             `toutuitestPodcasts (podcas`, and the offline mode of the same \
             width said the address `localhost:133` for a server at the port \
             13399. Each part now stands whole with a gap of two columns, or \
             it does not stand at all.",
        ],
    },
    Entry {
        version: "0.8.171",
        date: "17/08/2026",
        body: &[
            "Added:",
            "- **A cover that the terminal cannot draw says the name of its \
             media.** A media that the server holds with no cover, a terminal \
             with no protocol of pictures, and `TOUTUI_NO_COVERS` each gave a \
             cell of a border and nothing at all: a Home view of twelve such \
             cells said no name of a media at all. The cell keeps its border \
             and its place, and the title of the media stands in the rows of \
             the picture.",
            "Changed:",
            "- **The panel 6 of the gallery shows the shelf of the cursor of \
             the Home view.** It held the media of every shelf of that view \
             together, therefore the grid said no group of the server at all. \
             The Library view keeps every row of its list.",
        ],
    },
    Entry {
        version: "0.8.170",
        date: "17/08/2026",
        body: &[
            "Changed:",
            "- **One frame asks the server for eight new covers, and no \
             more.** The bands of covers of the Home view drew about 20 \
             cells, and each new cell of a frame was one request: the first \
             frame of a library asked the server for 15 covers inside one \
             millisecond, and the key `R` asked for 15 more. The frame that \
             meets more new covers than its limit leaves them for the frame \
             after it, therefore every cover comes and the program is kind to \
             the server.",
        ],
    },
    Entry {
        version: "0.8.169",
        date: "17/08/2026",
        body: &[
            "Added:",
            "- **The mouse reaches the bands of covers of the Home view.** A \
             click of a cover takes that media. Two clicks of one cover play \
             it, or they open the series or the podcast of it, which is the \
             work of the key `Enter`. A click of the name of a band takes the \
             first cover of that band.",
            "- **The wheel of the mouse over a band moves that band.** One \
             step gives one cover, at the left or at the right, and a band \
             that no cover of the cursor holds moves too. The cursor stays on \
             the screen when the band that holds it moves.",
        ],
    },
    Entry {
        version: "0.8.168",
        date: "17/08/2026",
        body: &[
            "Added:",
            "- **The Home view shows a band of covers for each shelf.** The \
             view showed one table of the title, the author, the length, and \
             the mark of the end. It now shows the cover of each media, in a \
             band under the name of its shelf. The name of the band says how \
             many covers the view shows, and how many the shelf holds: \
             `Recently Added ── 6 of 10 ›`. The arrow says that the band \
             holds more covers at that side.",
            "",
            "Changed:",
            "- **The keys of the Home view.** The keys `j` and `k` give the \
             shelf under and the shelf above. The keys `h` and `l` give the \
             cover at the left and the cover at the right of the same shelf. \
             The keys `g` and `G` give the first cover and the last cover of \
             that shelf. **The key `Enter` plays the media, or it opens the \
             series or the podcast.** The key `l` did that work before, and a \
             band needs it for the cover at the right.",
            "- **A screen that is too small for one whole band keeps the \
             table.** The keys of that table do not change, and the footer of \
             the view says which keys it holds.",
        ],
    },
    Entry {
        version: "0.8.167",
        date: "17/08/2026",
        body: &[
            "Added:",
            "- **A click of the bar of the book of the Chapters view moves the \
             media to that place.** That bar is the bar of the seek of the \
             view: a click of one of its cells says the same words as a click \
             of the bar of the band of the player, and the wheel over it moves \
             the media by ten seconds.",
        ],
    },
    Entry {
        version: "0.8.166",
        date: "17/08/2026",
        body: &[
            "Added:",
            "- **A click of a row of the Chapters view plays that chapter.** \
             The key l of that row does the same work, and the two say the \
             same words.",
            "Fixed:",
            "- A click of a row of the Chapters view named the wrong chapter \
             while the list stood away from its first line. The key G of a \
             book of 70 chapters gave the rows 35 to 70, and a click of the \
             second row of them moved the cursor to the chapter 2.",
        ],
    },
    Entry {
        version: "0.8.165",
        date: "17/08/2026",
        body: &[
            "Added:",
            "- **The list of the chapters is a table of the times.** Each row \
             says the number of the chapter, its title, the moment where it \
             starts in the book, and how long it is. A row of a header names \
             the four columns, and it stays over the list while you move.",
            "- The columns of the times take the width of the widest value \
             that they hold, therefore the times of two rows stand under each \
             other and you read them together.",
            "- A screen that has no room for the columns keeps the list of \
             before, which says the number, the title, and the start of each \
             chapter.",
        ],
    },
    Entry {
        version: "0.8.164",
        date: "17/08/2026",
        body: &[
            "Added:",
            "- **The view of the chapters holds two bars over its list.** The \
             first bar is the whole book, with a mark `│` at each boundary of \
             a chapter, therefore you see the place of every chapter in the \
             book. The second bar is the chapter of the cursor, therefore the \
             keys `j` and `k` say how long each chapter is and where you \
             stand in it. A chapter that the playback passed says 100%, and a \
             chapter that it did not reach says 0%.",
            "- The bar of the book holds no mark for a book of many short \
             chapters, and none under 40 columns: the marks then stand beside \
             each other, and they say less than a bar with no mark at all.",
            "- A media that holds no chapter keeps the two bars, and the \
             second of them says no number of a chapter. A view with no media \
             holds no bar.",
        ],
    },
    Entry {
        version: "0.8.163",
        date: "17/08/2026",
        body: &[
            "Changed:",
            "- **A cell of the panel `6 Gallery` holds the cover and its \
             border alone.** The cell said the percentage of your position \
             inside the border and a short title under the box, and the panel \
             `5 Cover` says the two of them for the media of the cursor \
             already. The two rows that went away go to the pictures: a \
             column of 41 rows held eight covers, and it holds twelve of them \
             now. The border of the cell of the cursor is heavy and bright, \
             and the border of every other cell is thin and dim.",
        ],
    },
    Entry {
        version: "0.8.162",
        date: "17/08/2026",
        body: &[
            "Fixed:",
            "- **The cover of the panel `5 Cover` takes every row that the \
             facts and the description leave.** The picture took a share of \
             the height of that panel, therefore a tall panel held a small \
             picture over rows of nothing at all: a screen of 60 rows held a \
             picture of 14 rows and four empty rows under the description. \
             The picture of that same screen now takes 18 rows. A description \
             of many lines keeps the picture on the screen, and the keys `J` \
             and `K` move that description.",
        ],
    },
    Entry {
        version: "0.8.161",
        date: "17/08/2026",
        body: &[
            "Fixed:",
            "- **The three filters of your position say what they filter, and \
             no word more.** The panel `3 Filter` cut the third of them at \
             `not…`, and a header of 84 columns held none of the words at all. \
             The filters now say `Finished`, `Started, not finished`, and `Not \
             started`.",
        ],
    },
    Entry {
        version: "0.8.160",
        date: "17/08/2026",
        body: &[
            "Fixed:",
            "- **The words of the key `z` name the three panels.** The footer, \
             the message of the key, and the view of the key `?` said `1 to \
             3`, which reads as `1 and 3`: a user looked for a panel 2 that \
             stood. The words now say `the panels 1, 2, and 3`.",
        ],
    },
    Entry {
        version: "0.8.159",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **The words of the sequence and of the filter no longer write on \
             the address of your server.** A terminal of 84 columns read \
             `localhost:13399title, the largest first`, with no gap and no \
             first word. The words now stand beside the address, and they keep \
             the middle of the row while the middle is free.",
            "Note:",
            "- A row that has no room for the whole of those words holds none \
             of them. The key `f` gives the sequence and the filter at every \
             width of your screen.",
        ],
    },
    Entry {
        version: "0.8.158",
        date: "16/08/2026",
        body: &[
            "Added:",
            "- **The panel of the cover says the day when you started the \
             media.** The line `Started` stands under the line of the time, in \
             the Home view and in the Library view. Your server holds that day \
             for every media that you played, and no view of this program said \
             it before.",
            "Note:",
            "- A media that you never started takes no such line, as every \
             other fact of that panel: a line that says nothing costs a row of \
             your screen.",
        ],
    },
    Entry {
        version: "0.8.157",
        date: "16/08/2026",
        body: &[
            "Added:",
            "- **The panel 6 of the gallery of the covers.** The column at the \
             right of the list holds a grid of the covers of the media around \
             your cursor, with your place and a short title under each of \
             them. The cell of the cursor takes the border of the focus, \
             therefore the grid and the panel of the cover say one media.",
            "- **The keys of that panel.** The digit `6` gives it the focus, \
             the keys `j` and `k` then move the cursor one row of the grid, \
             and the keys `+` and `-` make the cells larger and smaller. A \
             click of a cell takes the cursor to that media, and the wheel \
             over the panel moves the grid.",
            "Note:",
            "- The gallery stands in the Home view and in the Library view of \
             a screen of 120 columns and more, which are the views of the \
             frame of the panels. A column that has no room for the two \
             panels gives every row to the panel of the cover.",
        ],
    },
    Entry {
        version: "0.8.156",
        date: "16/08/2026",
        body: &[
            "Added:",
            "- **The Home view says the facts of the media too.** The panel of \
             the cover gave those facts in the Library view alone: the same \
             book of the Home view said the author, the year, the length, and \
             your place over two rows, and 15 rows of that panel held nothing \
             at all. The series, the narrator, the genre, the number of the \
             files, the size, the kind of the ebook, and the bar of the \
             progress now stand there too.",
            "Fixed:",
            "- **The program reads the kind of the ebook of a book of the Home \
             view.** The answer of the server holds it, and the program had no \
             place at all to keep it.",
            "Note:",
            "- A fact that the server did not give takes no line, therefore a \
             book of no series and no narrator says nothing of either. A \
             library of podcasts and a line of a series keep the words that \
             they held before.",
        ],
    },
    Entry {
        version: "0.8.155",
        date: "16/08/2026",
        body: &[
            "Added:",
            "- **The panel of the cover says the facts of the media.** The \
             answer of the server holds the series, the narrator, the genre, \
             the number of the files, the size, and the kind of the ebook of \
             every book, and no view of the program said one of them: the \
             panel gave three rows to the author, the year, the length, and \
             your place, and 15 rows of it held nothing at all. Each fact now \
             takes a line of its own, in the Library view of a library of \
             books.",
            "- **A bar of the progress stands under those lines.** It says \
             your place in the media with no letter at all, over the whole \
             width of the panel.",
            "Fixed:",
            "- **The program reads the kind of the ebook of a book.** It \
             asked the server for the name `ebookFileFormat`, and the server \
             sends `ebookFormat`: the value was therefore absent for every \
             book of every library.",
            "Note:",
            "- A fact that the server did not give takes no line. A book with \
             no narrator and no genre says the length and your place alone.",
            "- The length of the media and your place always take a line, \
             and a panel that is narrower than eight columns holds no bar.",
        ],
    },
    Entry {
        version: "0.8.154",
        date: "16/08/2026",
        body: &[
            "Added:",
            "- **The Library view can show every book of every series.** The \
             program asked the server for the library with the books of a \
             series in one group, therefore a book of a series stood in no \
             row of the list at all: the library `Books` of the test server \
             holds 22 books, and the list said `Library [18 items]`. The row \
             `Every book of a series` of the panel 2 of the sequence, and of \
             the view of the key `f`, gives every book a line of its own, \
             with its author, its length, and your place in it.",
            "- **A refresh keeps this mode.** The key `R` asks the server \
             again, and the list stays as you made it.",
            "Note:",
            "- A library of podcasts holds no series, therefore it holds no \
             such row.",
        ],
    },
    Entry {
        version: "0.8.153",
        date: "16/08/2026",
        body: &[
            "Added:",
            "- **The key `z` hides the panels 1, 2, and 3, and it gives them \
             back.** The screen of the design is always full: the stack of \
             the views, of the sequence, and of the filter takes 34 columns \
             at the left, and a user who wants a small and quiet screen \
             found no key for it. The key `z` gives those 34 columns to the \
             list. On a screen of 160 columns the list goes from 73 columns \
             to 93, and the panel of the cover goes from 48 to 62.",
            "- **The words of the sequence and of the filter stay on the \
             screen.** The second row of the header says them while the two \
             panels are away, therefore you keep the two facts of the list \
             that you see.",
            "- **A refresh keeps this mode.** The key `R` asks the server \
             again, and the screen stays quiet.",
            "Fixed:",
            "- **The footer names no key of a panel that went away.** With \
             the panels hidden, the footer says `z: the panels 1, 2, and 3` \
             and \
             no digit of them, and the digit `1` does nothing: a key that \
             moves the focus to a panel that you cannot see is a key of no \
             work. The digits `4` and `5` of the list and of the cover keep \
             their work in the two modes.",
        ],
    },
    Entry {
        version: "0.8.152",
        date: "16/08/2026",
        body: &[
            "Added:",
            "- **The player has a frame, and a bar of the position now.** The \
             three rows of the player stood in the air at the foot of the \
             screen, with no border and no title, and a screen of 160 columns \
             said your place in the book in two digits. The band says the \
             media, the bar of the position with the time at each end, a bar \
             of the book beside a bar of the chapter, and the keys of the \
             player.",
            "- **A click of the bar moves the playback to that place.** The \
             keys `p` and `u` move by ten seconds, and a book of eight hours \
             needs 2880 of them: this is the first control of Toutui that \
             names a place of a media directly. One step of the wheel over \
             the band moves the playback by ten seconds.",
            "- **The band says the number of the chapter**, for example \
             `Chapter 2 of 3: The hours of the middle`.",
            "- **The key `B` gives the list one line more.** The band stands \
             on the rows that it needs: it holds five rows with no row of the \
             keys, and six rows with it.",
            "Fixed:",
            "- **The rows of the player stand where the view puts them.** The \
             player counted nine rows back from the end of the screen, and a \
             view of a footer of three rows therefore drew its player over \
             its own last line.",
        ],
    },
    Entry {
        version: "0.8.151",
        date: "16/08/2026",
        body: &[
            "Added:",
            "- **The panel of the cover has a frame, and it holds the words \
             of the media now.** The picture stood in the air, with no \
             border, no title, and no number, and the rows under it held \
             nothing at all. It is the panel 5 of the design now: the picture \
             stands at the top of it, and the author, the year, the length, \
             your place, and the description of the media stand under it.",
            "- **A media that the server holds with no cover fills that panel \
             with words.** Such a media gave a column of 50 columns and 41 \
             rows with no character in it at all.",
            "- **The list holds four lines more**, because the facts and the \
             description of the media left the column of the list.",
            "- **The key `5` gives the focus to the panel of the cover**, and \
             a click of that panel does the same. The keys `j` and `k` of \
             that focus move the description, the key `l` plays the media, \
             and the key `h` gives the focus back to the list.",
        ],
    },
    Entry {
        version: "0.8.150",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **A filter of one series shows the books of that series now.** \
             The key `f` gives a list of the series of the library, and a \
             series of that list showed one line, \"The Test Chronicles \
             [3 books]\", over the three books that the server gave. The \
             library then said \"1 item\" for three books. Each book takes a \
             line of its own now, with its author, its length, and your \
             place in it, and the header says \"3 items\".",
            "- **A list of no such filter keeps one line for each series.** A \
             series of twelve books does not fill the screen.",
        ],
    },
    Entry {
        version: "0.8.149",
        date: "16/08/2026",
        body: &[
            "Added:",
            "- **The panel of the sequence and the panel of the filter stand \
             beside the list now.** A terminal of 120 columns or more shows \
             them under the panel of the views, at the left of the screen: \
             the panel 2 holds the sequences of the library and the direction \
             of them, and the panel 3 holds the places of your media. A mark \
             says the choice that stands, therefore you read the sequence and \
             the filter of the list while you read the list.",
            "- **The keys `2` and `3` give the focus to those two panels**, \
             and the keys `Ctrl+j` and `Ctrl+k` move the focus down and up in \
             the stack. The keys `j`, `k`, `g`, and `G` move the line of the \
             panel, the key `l` takes that line, and the key `h` gives the \
             focus back to the list. A click of a row of a panel takes that \
             row too.",
            "- **The key `l` of a sequence that stands turns the direction of \
             it**, therefore \"the newest first\" is one key.",
            "- **A terminal that is not wide keeps the words of the sequence \
             and of the filter in its header**, under the name of the \
             library.",
            "Changed:",
            "- **The footer of the Home view and of the Library view names \
             the key `f` of the sequence and of the filter.** That key stood \
             in the panel of the views and in no footer.",
            "- **A change of the sequence keeps the focus of the panel and \
             the line of it.** The program asks the server again for a new \
             sequence, and the focus went back to the list at each of those \
             requests.",
        ],
    },
    Entry {
        version: "0.8.148",
        date: "16/08/2026",
        body: &[
            "Added:",
            "- **The list of the Home view and of the Library view is a table \
             now.** A row of a header names the columns `Title`, `Author`, \
             `Time`, and `Done`, and each row of the list says the author of \
             the media, the length of it, and your place in it. The table \
             stands on a terminal of 120 columns or more, which is the \
             terminal that holds the panels; a terminal that is not so wide \
             keeps the list of before, and the mark at the start of each line \
             then says your place.",
            "- **A click of the row of the header opens the view of the \
             sequence and of the filter**, which is the work of the key `f`. \
             The view of the key `?` names this click.",
            "Changed:",
            "- **The mark of a line of the table says the media that plays and \
             the media that you finished alone.** The percent of your place \
             stands in the column `Done`, therefore the title of the media \
             holds more columns of the screen.",
        ],
    },
    Entry {
        version: "0.8.147",
        date: "16/08/2026",
        body: &[
            "Added:",
            "- **The mouse works in Toutui.** A click of the button at the \
             left moves your position to the line under the pointer, and it \
             moves it to the panel of that line. One step of the wheel moves \
             your position one line up or one line down, in the list under \
             the pointer, and it needs no click first. The mouse works in \
             every view that holds a list, and a click of a row that holds no \
             line moves nothing.",
            "- **The key `Ctrl+o` stops the mouse, and it starts it again.** \
             A program that reads the mouse takes the selection of the text \
             away from your terminal: press `Ctrl+o` to select the text \
             again, and press it a second time for the mouse. Most terminals \
             give the selection of the text with `Shift` and a click while \
             the mouse works. The view of the key `?` names these keys.",
        ],
    },
    Entry {
        version: "0.8.146",
        date: "16/08/2026",
        body: &[
            "Added:",
            "- **A panel of the views at the left of the Home view and of the \
             Library view.** A terminal of 120 columns or more now shows a \
             panel with the name of every view of the program and the key of \
             each one. The key `1` moves your position to that panel, the key \
             `4` moves it back to the list, and the keys `Ctrl+h` and `Ctrl+l` \
             move it to the panel at the left and at the right. The panel that \
             holds your position has a heavy border, therefore a terminal of a \
             theme of a low contrast still shows where you are. The keys at \
             the foot of the screen change with the panel. A terminal that is \
             not as wide keeps the screen that it had.",
        ],
    },
    Entry {
        version: "0.8.145",
        date: "16/08/2026",
        body: &[
            "Changed:",
            "- **Toutui now uses the colors of your terminal.** The program \
             painted a dark grey over your terminal before this version, \
             therefore a theme of a light color did not work and a change of \
             the theme of your terminal changed nothing. The program now \
             paints no color of its own: the background and the letters of \
             your terminal are the background and the letters of Toutui, and \
             the row of the cursor alone holds a color, which is the cyan of \
             your terminal. Every key of the block `colors` of your \
             configuration file keeps its work, therefore a color that you \
             give stays.",
        ],
    },
    Entry {
        version: "0.8.144",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **The panel of a media keeps the line of your place in it.** A \
             user of the server gives the name of an author, and it can hold \
             an end of a line. That panel has two rows in a terminal that is \
             not tall, therefore such a name took a row more, and the line of \
             your place went off the screen. The name of the author now stands \
             on one row, and the two parts of it hold one space between them. \
             The six panels of a media take the same rule.",
        ],
    },
    Entry {
        version: "0.8.143",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **The name of the library at the top of the screen keeps one \
             line.** An administrator of the server gives that name, and it \
             can hold an end of a line. The area at the top of the screen has \
             two rows, therefore that end of a line put the second part of the \
             name on the row of the address of the server, and no row of the \
             screen said the whole name of the library. The name now stands on \
             one row, and the two parts of it hold one space between them.",
        ],
    },
    Entry {
        version: "0.8.142",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **The header and the contents of the reader keep one line \
             each.** The name of a book and the name of a chapter come from \
             the file of the book, and a maker of an EPUB can write an end of \
             a line in one of them. The line at the top of the reader has one \
             row, therefore that end of a line took the number of the chapter \
             and the percent of the place of the user outside the screen; and \
             a name of a chapter of two lines took two rows of the table of \
             contents, and the second row of it named a chapter that the book \
             does not hold.",
        ],
    },
    Entry {
        version: "0.8.141",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **The panel of the player keeps the row of its keys.** The \
             title, the author, and the chapter of a media come from the \
             server, and a text of one of them can hold an end of a line. The \
             panel of the player has four rows, therefore that end of a line \
             took a row of its own and the row of the keys of the player then \
             stood outside the panel: no user saw it.",
        ],
    },
    Entry {
        version: "0.8.140",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **The lines of a list stand on one row each.** A title of the \
             server can hold an end of a line, and the program gave that title \
             to the list with no change: the line then took two rows of the \
             panel, the second row of it held no mark and it read as a media \
             that the library does not have, and the last line of the list had \
             no row and no bar of the scroll at all.",
        ],
    },
    Entry {
        version: "0.8.139",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **A message that holds an end of a line stands whole.** The \
             program counted the rows of a message as one line, therefore a \
             message that names a text of the server with an end of a line in \
             it stood on one row: the rows after the first one had no road at \
             all, and no three points said that the program cut the sentence.",
        ],
    },
    Entry {
        version: "0.8.138",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **The panel of a description reaches the last line of its \
             text.** The program counted one space between two words, and a \
             description of the server can hold more of them: the panel then \
             said that it holds the whole of its text, no bar of the scroll \
             came, and the key J moved nothing. The last lines of such a \
             description had no road at all.",
        ],
    },
    Entry {
        version: "0.8.137",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **The two keys that take another library say so.** The key of \
             the next library and the key l of the settings each changed the \
             library and each said nothing at all: the program made its new \
             screen for that key, and the message of the key went away before \
             the first frame of it. The user now reads \"The program shows the \
             library ... now.\" after the screen changed.",
        ],
    },
    Entry {
        version: "0.8.136",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **The box of a message counts the columns of its text.** The \
             program said that a message of the Han script, of Hiragana, or of \
             Katakana needs one row more than the screen draws it on: the box \
             of that message then stood one row above its text, the last row \
             of the box held no character, and one row of the view went away \
             for nothing. The count of a message and the count of a panel read \
             one rule of the wrap now, and a message stands on the rows that \
             it needs.",
        ],
    },
    Entry {
        version: "0.8.135",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **The panel of a description counts the columns of its text.** \
             The program measured the length of the text of a panel with a \
             number of characters, and a description of the Han script, of \
             Hiragana, or of Katakana therefore took twice the rows that the \
             program counted: a description of 30 rows in a panel of 18 rows \
             showed no bar of the scroll at all, the key J moved nothing, and \
             the last 12 rows of that text had no road. The panel counts the \
             columns of its text now, and the bar and the two keys give the \
             whole of it.",
        ],
    },
    Entry {
        version: "0.8.134",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **A character is not a column.** The program measured every text \
             of the screen with a number of characters, and a character of the \
             Han script, of Hiragana, or of Katakana takes two columns of the \
             terminal: a title, a message, and the line at the top of the \
             reader were therefore wider than the room that they had, and the \
             screen cut them a second time. A terminal of 40 columns said \
             \"und nothing for \u{65e5}\u{672c}\u{8a9e}\u{65e5}\u{672c}\u{8a9e}\u{65e5}\u{672c}\u{8a9e}\u{2026}\" for a search of \
             eighteen characters of Japanese, and the start of that title went \
             away. The program measures the columns of a text now, with the \
             same rule that the screen has.",
        ],
    },
    Entry {
        version: "0.8.133",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **The title of a view keeps its start.** The title stands in the \
             middle of the header of a list, and a title that was longer than \
             the screen lost its start and its end together: a terminal of 40 \
             columns said \"he books of Many Hours Author]\" for the title \
             \"Search result [2 items, with the books of Many Hours Author]\", \
             therefore the user read no name of the view and no number of its \
             items. A title that does not stand now keeps its start, and three \
             points say that the screen cut the end of it.",
        ],
    },
    Entry {
        version: "0.8.132",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **The box of the search no longer leaves a cell of the screen \
             behind it.** That box makes a screen of its own, and it writes on \
             the cells of the view below it. The program then drew the view \
             after it, and it sent the cells that changed only: a cell of the \
             box that held the same letter as the view after it got no byte at \
             all, and the space that the box left stayed on the screen. A \
             terminal of 40 columns therefore said \"l: play or op n\" at the \
             foot of the view of the search. The program now writes every cell \
             of the screen again after that box, in the same way as the box \
             that asks for a text.",
        ],
    },
    Entry {
        version: "0.8.131",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **The keys of every view now stay on the screen.** The footer of \
             a view stood on two rows with no room for more: a terminal of 40 \
             columns holds 80 cells in them, therefore the Home view showed \
             \"j/k: move  l: play or open  Tab: home/library  S-Tab: the next \
             library\" and no more, and the user read no key of the search, of \
             the refresh, of the table of the keys, and of the quit. The footer \
             now takes the rows that its wrap needs, and it grows over the work \
             of the view: no more than one half of the rows, and no fewer than \
             the two that a view held before.",
            "- The footers of the statistics, of the sessions, of a new \
             podcast, of the accounts, and of the library of the user stood \
             outside the gate of the footers of the program, and two of them \
             held a line break of their own. Every footer now stands in one \
             place, and the gate measures the rows of its wrap.",
        ],
    },
    Entry {
        version: "0.8.130",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **The keys of the reader now stay on the screen.** The keys at \
             the foot of the reader stood on two rows with no wrap: a terminal \
             of 40 columns cut the first row at \"n/p: chapter\" and the second \
             row at \"h:\", therefore the user read no key of the road back and \
             no key of the quit, and the contents of a book lost the whole of \
             \"h: leave the book\". The footer now takes the rows that it needs, \
             and it wraps as the footer of every other view of the program does.",
        ],
    },
    Entry {
        version: "0.8.129",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **The reader now keeps the number of your chapter and your \
             percent.** The line at the top of the reader said the title of the \
             book first, and a long title took the number of the chapter, the \
             count of the chapters, and the percent outside the screen: a book \
             of the title of Robinson Crusoe of Project Gutenberg gave the same \
             line at every chapter, at 80 columns and at 160. The place of the \
             user now keeps its room, and the title loses its end to three \
             points.",
        ],
    },
    Entry {
        version: "0.8.128",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **A message that is longer than your screen now stands on more \
             than one row.** The row of the message of a view held one row, and \
             it cut every longer sentence with three points: the words of a log \
             out lost the road back to the copies of the disk. The message now \
             takes the rows that it needs, above the footer, and the header of \
             the screen keeps its rows.",
        ],
    },
    Entry {
        version: "0.8.127",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **The words of a log out now reach you when a second account \
             takes the start.** The key l on the account that starts the \
             program gives that work to the first account that stays, and the \
             program starts again with it: the message of that log out went \
             away with the process, and the new screen said nothing at all. \
             Those words now stand on the first screen of the new account.",
        ],
    },
    Entry {
        version: "0.8.126",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **A log out now says the copies of the disk that stay on your \
             computer.** The key l removes the account and it keeps every \
             download and every book of the cache of that account: the disk \
             held 239.7 MB of them in the measurement, and no word of the \
             program said it. The message now says how many media stay, how \
             many megabytes they use, and the road back: a login with the same \
             name and the same server gives them again, and the key X then \
             removes a copy.",
            "- **The words of a log out now reach you.** The program starts \
             again by itself after a log out of your one account, and the \
             message of that log out went away with it: the login screen said \
             nothing at all.",
            "- **A message of the login screen now stands whole.** That row \
             held one line and it cut every longer sentence at the edge of the \
             screen. It stands on the lines that it needs now.",
        ],
    },
    Entry {
        version: "0.8.125",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **A log out now takes your places that wait for the server with \
             the account.** Those places stayed on your computer: if you logged \
             in again with the same name and the same server, the program gave \
             them to the server at its start, and they stood over the place \
             that you made on another computer while the account was away. The \
             message of the log out says how many of them went away with the \
             account.",
        ],
    },
    Entry {
        version: "0.8.124",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **Your place in a book now goes to the server as soon as the \
             server answers again.** The program sent a place that the server \
             refused at its next start alone: if you left that book and opened \
             a different one, your other computers held the old place while \
             the program stood. The program now looks every 30 seconds, and it \
             sends that place with the positions of your books that play.",
        ],
    },
    Entry {
        version: "0.8.123",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **Your place in a book now stays on your computer when the \
             server does not take it.** The program kept that place in its \
             memory alone: a program that the machine stopped, and a computer \
             that went off, took every line that you read with it. The program \
             now writes that place on the disk, and it sends it to the server \
             at the next start.",
        ],
    },
    Entry {
        version: "0.8.122",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **The place of a book that you left no longer goes away when \
             you open a second book.** The program kept one place for the whole \
             of it: if the server did not take the place of a book at the \
             moment that you left it, and you then opened a different book, the \
             place of the first book went away before the program stopped. The \
             program now keeps the place of each book, and it sends every one \
             of them to the server when it stops.",
        ],
    },
    Entry {
        version: "0.8.121",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **Your place in a book now goes to the server when you stop the \
             program with the key \"Q\".** The footer of the reader names that \
             key, and the program sent the place of the audio alone: every line \
             that you read went away with the program, and the server kept the \
             place of the chapter before it on all of your machines. The place \
             of the book now goes to the server before the program stops. A \
             terminal that goes away takes the same road.",
        ],
    },
    Entry {
        version: "0.8.120",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **The reader now sends your place in a book again when the \
             server did not take it.** The program said that your place was on \
             the server before the server answered. A request that failed \
             therefore took that place away: the program said the fault one \
             time, and it sent that place never again — not after 30 seconds, \
             and not when you left the book with the key \"h\". Your place now \
             goes to the server again on both roads, and the program says that \
             the server holds it only when the server answers.",
        ],
    },
    Entry {
        version: "0.8.119",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **A media of 100 percent that you did not finish now keeps the \
             mark of its place.** The line of a list gave the mark of a media \
             that you finished to every percent above 99, therefore the shelf \
             \"Continue Listening\" held a line with the mark \"✓\", and the \
             panel of that same line said \"Not finished\". The mark of the end \
             now comes of the server alone, and a media at the whole of its \
             length says \"100\".",
        ],
    },
    Entry {
        version: "0.8.118",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **A media now never says a time that is left of less than \
             zero.** The panel of a line took the length of the media away \
             from the place of the user, and it wrote the difference: a media \
             of a length that the server does not give said \"-1m left\" \
             beside a length of \"N/A\", and a media whose place stands past \
             its length said \"-1h-1m left\". A media of a length that the \
             program does not have now says no time at all, and a media whose \
             place stands at its length or past it says \"0m left\".",
        ],
    },
    Entry {
        version: "0.8.117",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **The view of the episodes of a podcast now says the length of \
             each episode.** The program made one length for each episode that \
             holds an audio file, and not one length for each episode: an \
             episode with no audio file therefore took the length of the \
             episode after it, and the last line of the view got no length at \
             all. The panel of that last line said \"Error: Episode data \
             unavailable or index out of bounds.\", and the program wrote one \
             line of the file of the log at every frame of the screen. An \
             episode of a length that the server does not give now says \
             \"N/A\", and the panel of every line says the values of that \
             line.",
        ],
    },
    Entry {
        version: "0.8.116",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **The file of the log now says the number of the chapter that \
             you read.** Two lines of the log of the reader said the number \
             of the list of the program, and that number is one less than the \
             number of the screen: a fault of the chapter 2 of a book said \
             \"no chapter 1 of the book\", and a book that names no chapter \
             said \"the chapter 0\". No book holds a chapter 0. The two lines \
             now say the number that you read at the top of the reader.",
        ],
    },
    Entry {
        version: "0.8.115",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **A chapter that the book did not give now names the keys of \
             the reader.** The sentence of that fault named the key `n` \
             alone. The other chapters of such a book can be good: the \
             sentence now names the key `p` for the chapter before this one \
             and the key `h` that leaves the book, and each of the three \
             keys does its work.",
        ],
    },
    Entry {
        version: "0.8.114",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **A file that no reader opens now says the road back.** The \
             reader said \"This file is not an EPUB.\" and no more. Toutui \
             keeps a copy of each book that you open on the disk, therefore \
             that copy gave the same sentence at every press of the key `e`, \
             and Toutui asked the server for the book no more. The sentence \
             now names the key `h` of the screen that you see, and the key \
             `X` that removes the copy of the disk: the open after that key \
             asks the server for the file again.",
            "- **That sentence now names the file of the log.** The reason of \
             the book — a damaged archive, or a file of another form — stands \
             in the log alone.",
        ],
    },
    Entry {
        version: "0.8.113",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **A book that the reader refuses now says the road back.** A \
             book that is larger than the limit of the reader, and a book that \
             holds more files than that limit, each gave one sentence of \
             numbers alone: the sentence named no key, and the fault took no \
             line of the file of the log. Toutui now names the key `h` of the \
             screen that you see, and it writes the name of the file of that \
             book in the log.",
            "- **Toutui now says a size in megabytes.** The reader said \"It \
             has 269486151 bytes, and the limit is 268435456 bytes\". It now \
             says 257.0 MB and 256.0 MB, in the form that the bar of a \
             download uses already. The sentence of a chapter that is too \
             large says its limit in the same form.",
            "- **The sentence of a book that the reader refuses now names the \
             limit of the reader that measured that book.** The reader of a \
             PDF holds a limit of 512 megabytes and a limit of 5000 pages, and \
             the sentence of each fault named the limits of the reader of an \
             EPUB book.",
        ],
    },
    Entry {
        version: "0.8.112",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **The reader says that a book holds no chapter, and the line at \
             the top of it says the number that the program measured.** A book \
             can name no chapter at all. The line at the top of the reader said \
             \"chapter 1 of 1\" for such a book, and the reader gave the \
             sentence \"This book has no chapter 0.\": the line named a chapter \
             that the book does not hold, the number 0 stands in no view of \
             this program, the sentence named no key, and the fault took no \
             line of the file of the log. Toutui now says that the book holds \
             no chapter, it names the key `h` of the view of the reader, and \
             the fault writes a line in the log. A sentence of a chapter that \
             the book does not hold now says the number that you read in the \
             line at the top.",
            "- **A PDF that holds no page says that it holds no page.** Such a \
             file gave the sentence \"This book has no chapter 0.\" Toutui now \
             says that the PDF gives no page, as it does already for a PDF that \
             the other part of the program reads.",
        ],
    },
    Entry {
        version: "0.8.111",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **The reader says which chapter a book holds no file of, and it \
             gives you a road.** A book can name a chapter and hold no file of \
             it. The reader gave the sentence \"This chapter is absent.\" for \
             that book: it named no chapter, it named no key of the view of \
             the reader, and the fault took no line of the file of the log. \
             Toutui now names the chapter, it says that the book holds no file \
             of it, and it says that the other chapters can be good. The \
             sentence names the keys `n`, `p`, and `h` of the view of the \
             reader, and the fault now writes a line in the log.",
        ],
    },
    Entry {
        version: "0.8.110",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **The reader says what it measured for a chapter that is too \
             large, and it gives you a road.** A chapter that is larger than \
             the limit of 8 megabytes gave the sentence \"This chapter is too \
             large.\" That sentence named no size and no limit, it named no \
             key of the view of the reader, and the fault took no line of the \
             file of the log. Toutui now says that the chapter has more than \
             the limit of bytes, because it stops the read at that limit and \
             it does not measure the whole chapter. The sentence names the \
             keys `n`, `p`, and `h` of the view of the reader, and the fault \
             now writes a line in the log.",
        ],
    },
    Entry {
        version: "0.8.109",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **The reader says what it measured for a chapter that did not \
             come, and it gives you a road.** A chapter of very many tags \
             held the reader for more than five seconds, and Toutui then said \
             \"This chapter is too complex.\" Toutui measured a time, and it \
             did not measure the chapter: a machine that is busy gives that \
             same five seconds. That sentence also named no key, and the \
             fault took no line of the file of the log. Toutui now says the \
             limit of time that went by, it names the two conditions that can \
             give it, it names the keys `n`, `p`, and `h` of the view of the \
             reader, and it writes the fault in the log. A chapter whose \
             thread of the render died now says what the machine said, and it \
             no longer drops that reason.",
        ],
    },
    Entry {
        version: "0.8.108",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **The reader says what the server said, and it does not say \
             that a media holds no ebook.** When the key `e` gets no book, \
             Toutui asks the server for the data of that media, to tell you \
             what the media holds. If that second request came back with a \
             fault, Toutui said \"The server has no ebook for this media.\" \
             for every reason: a server that reported a fault, a server that \
             did not answer in time, and a token that is not valid each said \
             that your book is not there. Toutui now says what the server \
             said, it names the key that asks again, and it writes the fault \
             in the file of the log. A server that does not hold the media at \
             all keeps a sentence of its own.",
        ],
    },
    Entry {
        version: "0.8.107",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **The message of a podcast with no episode stands on more than \
             one line.** When Toutui gets no episode of a podcast, it says why \
             on the screen of that podcast. That message holds the words of \
             the server, and it was longer than a terminal of 80 columns: \
             Toutui drew one line of it and it cut the rest away, therefore \
             you read \"The server reported a faul\" and you did not read the \
             status of the server. The message stands now on as many lines as \
             it needs, at every width of the terminal.",
        ],
    },
    Entry {
        version: "0.8.106",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **The reader says why a chapter gives no text, and it does not \
             say \"Reading…\" for ever.** A chapter of a book that Toutui \
             cannot read gives no line. The reader read that condition as \
             \"the text is not ready\", therefore it started the read again at \
             every frame and the message of the fault went away before you \
             read it: the screen said \"Reading…\" and it did not change. \
             Toutui says now what the machine said of that chapter, it says \
             that the other chapters can be good, and it names the key that \
             goes to the next chapter. The message stays on the screen, and \
             the file of the log holds the reason. A message that is longer \
             than one line stands now on more than one line.",
        ],
    },
    Entry {
        version: "0.8.105",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **The reader says that your disk did not give a book, and it \
             does not say that your book is not an EPUB.** Toutui keeps the \
             ebook of a media in a file of its own. A file with no permission \
             of a read, a file that went away, and a disk that answers with a \
             fault each stop the read of that file: Toutui said there \"This \
             file is not an EPUB.\" for a good book. Toutui says now that the \
             disk did not give the book, it says that the book can be good, \
             and it gives the reason of your machine. A book that no reader \
             opens keeps its words, and Toutui writes the reason of that book \
             in the file of the log.",
        ],
    },
    Entry {
        version: "0.8.104",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **Toutui says why the login screen did not reach your \
             terminal.** Toutui writes that screen to its standard output. A \
             pipe whose reader went away, a disk that is full, and a file that \
             Toutui cannot write each stop that screen: Toutui dropped the \
             fault there, it waited one second, it made a terminal again, and \
             it then said that it found no terminal while you stood in one. \
             Toutui says now that the login screen did not reach the terminal, \
             it gives the reason of your machine, and it names the road back. \
             Toutui also gives your terminal back with words of its own, and \
             not with the words of a part of another program.",
        ],
    },
    Entry {
        version: "0.8.103",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **The reader says that your disk did not take the pages of a \
             PDF, and it does not say that your book can be damaged.** Toutui \
             reads a PDF in a second process, and that process writes the \
             pages beside the book. A disk that is full, a file system that is \
             read-only, and a directory with no permission of a write each \
             stop that write. Toutui said there \"This PDF gives no page. The \
             file can be damaged.\" for a book that it read a moment before, \
             and it said the same words for a part of the program that did not \
             start and for a book that took too long. Toutui says now which of \
             them it met, it gives the reason of your machine, and the words \
             of a damaged book stay for a damaged book.",
        ],
    },
    Entry {
        version: "0.8.102",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **Toutui says why it does not start when it finds no \
             terminal.** Toutui draws its screen in a terminal, and it reads \
             your keys from that terminal. A unit of systemd, a task of cron, \
             and a program of the background give no terminal: Toutui stopped \
             there with the words of an internal fault, and those words named \
             a line of the source of a library. Toutui says now that it found \
             no terminal, it gives the reason of your machine, and it tells \
             you to start it in a terminal.",
        ],
    },
    Entry {
        version: "0.8.101",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **The login screen stops when its terminal goes away.** Toutui \
             v0.8.100 looks at its terminal each second, and that look starts \
             after you log in. A program that stood on the login screen when \
             its terminal went away therefore stayed for ever, it kept a whole \
             processor, and it gave no screen and it took no key. Toutui looks \
             at the terminal of the login screen now too. That screen holds no \
             account, therefore Toutui closes no session of the server there: \
             it stops, and the log says why.",
        ],
    },
    Entry {
        version: "0.8.100",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **Toutui stops when its terminal goes away.** A terminal that \
             closes sends a signal to the program of that terminal, and a \
             program that you put in the background, a program of `nohup`, and \
             a program of a service get no signal at all. Toutui did not see \
             it: the program stayed for ever, it kept a whole processor, it \
             gave no screen and it took no key, and its listening session \
             stayed open on the server. Three such programs on one machine \
             stood for three hours each. Toutui looks at its terminal each \
             second now: a terminal that went away stops the program, and \
             Toutui closes the session of the server and sends your place \
             first. The log says why.",
        ],
    },
    Entry {
        version: "0.8.99",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **The login screen says why the login failed, also when the \
             database of Toutui takes no write.** Every message of that screen \
             — a password that the server refused, an address with no \
             `http://`, a field with no character, and the token that the \
             server no longer accepts — made a road through the database. A \
             database that took no write of that one column therefore gave you \
             a login screen with no word at all: you wrote a wrong password, \
             the server refused it, and the screen said nothing. Toutui keeps \
             the message of the login screen now, and the screen says it. The \
             log names the database when it refuses that write.",
        ],
    },
    Entry {
        version: "0.8.98",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **Toutui says why it did not remove an account of its \
             database.** A server that does not accept the token of an \
             account sends you to the login screen, and Toutui first removes \
             the row of that account: a row that stays sends you to the same \
             answer of the server again. A database that refuses that removal \
             stopped the program with the words of a library: those words \
             named a line of a file of Rust, they gave no road back, and the \
             log kept no word of that stop. Toutui names the account, the \
             database, the reason, and the server now, it tells you to \
             correct the database and to start Toutui again, and it says that \
             it changed nothing. The log keeps the whole fault.",
        ],
    },
    Entry {
        version: "0.8.97",
        date: "16/08/2026",
        body: &[
            "Fixed:",
            "- **Toutui says why it cannot read the accounts of its \
             database.** Toutui reads its accounts at the start, before the \
             login screen. A second Toutui of that database, or a disk that \
             says nothing, stopped the program with the words of a library: \
             those words named a line of the source of Toutui, they gave no \
             road back, and the log kept no word of that stop. Toutui names \
             the database now, it gives the reason, it tells you to stop a \
             second Toutui that uses that database and to start this one \
             again, and it says that it changed nothing. The log keeps the \
             whole fault.",
        ],
    },
    Entry {
        version: "0.8.96",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **The login screen says why Toutui cannot read the configuration \
             file.** Toutui reads that file before it shows the login screen. A \
             file of a shape that is not correct, for example an array with no \
             `]`, gave the words of a library there, and those words named a \
             line of the source of Toutui and no road back. The log kept no \
             word of that fault. Toutui names the file now, it gives the \
             reason with the line and the column, and it tells you to correct \
             that file or to give it a different name. The log keeps the whole \
             fault.",
        ],
    },
    Entry {
        version: "0.8.95",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **The key `R` keeps your program when Toutui cannot read the \
             configuration file.** You can change that file while Toutui runs, \
             and the key `R` reads it again. A file of a shape that is not \
             correct, for example an array with no `]`, then stopped the whole \
             program: the playback, the queue, and every list went away for one \
             character. Toutui keeps the application now, and it says that the \
             screen did not change. The log names the line and the column of \
             the fault. Correct that file, and press `R` again.",
        ],
    },
    Entry {
        version: "0.8.94",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **A configuration file that Toutui cannot read says why, and \
             those words stay on the screen.** A file of a shape that is not \
             correct, for example an array with no `]`, gives no value of the \
             user at all, and Toutui stops. The words of that fault came from \
             a library, they named a line of the source of Toutui, and they \
             named no file and no road back. The clear of the screen of the \
             start then came after them at the exit of the program, therefore \
             the terminal of the user kept no word at all. Toutui names the \
             file now, it gives the reason of the fault with the line and the \
             column, and it says that you can correct that file or give it a \
             different name.",
        ],
    },
    Entry {
        version: "0.8.93",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **The user reads the values of the configuration file that the \
             program does not use.** The program takes a value of that file \
             away for two reasons: it cannot read that value, or a rule of the \
             program refuses it. Each of the two roads wrote a line of the log \
             alone: a user with a colour of two numbers, with a server of a \
             name of no character, and with a limit of the cache of the ebooks \
             that is no number, lost the three of them and the screen said \
             nothing at all. The row of the message says the number of those \
             values now, at the start of the program and at the key `R`, and \
             the log keeps the name and the reason of each of them.",
        ],
    },
    Entry {
        version: "0.8.92",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **An address that more than one server of the configuration \
             file holds does not take the queue and the downloads of a \
             different server.** The name of the server that holds an address \
             is the identity of the place of the user, and the program used \
             the first server of the file that held that address. Two blocks \
             `[[servers]]` with one address therefore held one identity: the \
             account of one server showed the queue of the other one, the \
             program asked the address of a different server, and it said no \
             word at all. Such an address goes away from each of those servers \
             now, the address of the login screen gives the identity, and the \
             log names the address and the servers.",
        ],
    },
    Entry {
        version: "0.8.91",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **A server of the configuration file that has the name of an \
             address does not take the queue and the downloads of a different \
             server.** The address of the login screen is the identity of the \
             place of the user when no server of the file holds that address. \
             A block `[[servers]]` with the name `http://localhost:13399` \
             therefore held the identity of the server at that address: the \
             account of a different server showed the queue of it, and the \
             program said no word at all. A server with a name that starts \
             with `http://` or `https://` goes away now, the address of the \
             login screen gives the identity of that server, and the log names \
             it.",
        ],
    },
    Entry {
        version: "0.8.90",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **Two servers of the configuration file that hold one name do \
             not share the queue and the downloads of the user.** The name of \
             a server is the identity of the place of that user on the disk. \
             Two blocks `[[servers]]` with the name `home` gave one identity \
             to two servers: the account of the second server showed the queue \
             of the first one, and the program said no word at all. The server \
             of the first block keeps the name now, a server after it that \
             repeats that name goes away, the address of the login screen \
             gives the identity of that server, and the log names it.",
        ],
    },
    Entry {
        version: "0.8.89",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **A server of the configuration file with a name of no \
             character does not take the queue and the downloads of the user \
             away.** The name of a server is the identity of the place of that \
             user on the disk. A line `name = \"\"` gave an identity of no \
             character: the queue and the downloads of the user went away, and \
             the program said no word at all. Such a server goes away now, the \
             address of the login screen gives the identity again, and the log \
             names the server.",
            "- **Two servers of the configuration file never hold the same \
             identity.** Two servers with a name of no character each held the \
             identity of no character, therefore the place of the user of one \
             server went to the other server.",
        ],
    },
    Entry {
        version: "0.8.88",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **One server of the configuration file that the program cannot \
             read does not take the other servers of the user away.** A line \
             such as `priority = 300` in one address of one server gave a \
             program with no `[[servers]]` block at all: the queue and the \
             downloads of the user then went away, because the name of a \
             server is the identity of their place on the disk. The program \
             reads each server of the file apart now, and the log names the \
             server of the fault.",
            "- **An address of a server that the program cannot read does not \
             take the other addresses of that server away.** A server has more \
             than one address, and one of them answers.",
            "- **A value of the block `[reader]` that the program cannot read \
             takes the value of the program alone, and the log names that \
             value.** A line such as `ebook_cache_mb = -1` gave the limit of \
             the program, of one gigabyte, with no word at all.",
        ],
    },
    Entry {
        version: "0.8.87",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **One colour of the configuration file that the program cannot \
             read does not take the other colours of the user away.** A number \
             above 255, such as `list_selected_background_color = [80, 80, \
             300]`, gave every colour of the program and no word at all. The \
             program reads each colour of the file apart now: that colour \
             alone takes the colour of the program, and the log names the key.",
            "- **A colour that holds no three numbers takes the colour of the \
             program.** A line such as `list_background_color = [50, 50]` gave \
             a colour that the user did not ask for, in silence. The log names \
             that key now.",
        ],
    },
    Entry {
        version: "0.8.86",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **A colour of the configuration file that holds no three numbers \
             does not stop the program.** A line such as \
             `list_background_color = [50, 50]` stopped the program before its \
             first frame, and the terminal of the user said nothing at all. \
             The program takes the last number of such a line now, and it \
             starts.",
            "- **The lists of the views draw more quickly.** The program read \
             the configuration file one time for each line of each frame. It \
             reads the colours that it holds now, and the key `R` gives the \
             colours of the file again.",
        ],
    },
    Entry {
        version: "0.8.85",
        date: "15/08/2026",
        body: &[
            "Changed:",
            "- **The bar of the scroll of a list holds the line of the cursor.** \
             The bar said which part of the list the view draws, and it did not \
             move while the cursor went through the lines of the view. It stands \
             at the top of its track at the first line of the list now, and at \
             the foot of it at the last line.",
        ],
    },
    Entry {
        version: "0.8.84",
        date: "15/08/2026",
        body: &[
            "Added:",
            "- **The list of a view says where the cursor of the user stands.** \
             A bar at the right of the list comes when the list holds more \
             lines than the rows of the view, and the place of the bar says \
             which part of the list the view draws. A list that holds every \
             line of it takes no bar.",
        ],
    },
    Entry {
        version: "0.8.83",
        date: "15/08/2026",
        body: &[
            "Added:",
            "- **The bar of the scroll of a panel names the two keys that move \
             that panel.** The letter of the key that moves the panel up \
             stands at the top of the bar, and the letter of the key that \
             moves it down stands at the foot of it. A bar of few rows keeps \
             the whole of its track, and it takes no letter.",
        ],
    },
    Entry {
        version: "0.8.82",
        date: "15/08/2026",
        body: &[
            "Added:",
            "- **The panel of a description says that it holds more text.** A \
             bar at the right of the panel comes when the text is longer than \
             the rows of it, and the place of the bar says where in that text \
             the panel stands. A panel that holds the whole of its text takes \
             no bar.",
        ],
    },
    Entry {
        version: "0.8.81",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **The key that moves the panel of a description down went past \
             the last line of the text, and the panel then held no line at \
             all.** One press of that key took a description of one line away, \
             and the user cannot tell such a panel from a media whose \
             description the server did not give. The program measures the \
             length of the text now: the key stops at the last line of the \
             panel, and the panel of the changelog stops at the oldest entry \
             of it.",
        ],
    },
    Entry {
        version: "0.8.80",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **The panel of the view of the episodes of a podcast said \
             \"No description available\" while the server gave a \
             description.** The panel read the subtitle of the episode alone, \
             and the server gives no subtitle for an episode of a podcast of a \
             feed. The description of the episode reached no box of the \
             program, and the description of the podcast reached no view of \
             the screen. The panel says the subtitle of the episode first now, \
             the description of that episode after it, the description of the \
             podcast after that, and the words of a panel that holds no text \
             when the server gives none of the three.",
        ],
    },
    Entry {
        version: "0.8.79",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **The panel of a podcast of the Home view said \"No description \
             available\" while the server gave a description.** The server \
             gives no subtitle for an episode of a podcast of a feed, and the \
             panel read that subtitle alone. The program asked the server for \
             the description of the podcast at the same moment, and no view of \
             the screen read it. The panel says the subtitle of the episode \
             first now, the description of the podcast after it, and the words \
             of a panel that holds no text when the server gives neither.",
        ],
    },
    Entry {
        version: "0.8.78",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **The panel of the description said \"N/A\" in three views, and \
             in one view it said nothing at all.** The words \"N/A\" belong to \
             a value that stands beside a label: the line of the view says \
             `Year: N/A`, and that label tells the user which value the server \
             does not have. The panel of the description holds no label, \
             therefore those two letters said nothing to the user. The Home \
             view, the view of the episodes of a podcast, and the panel of a \
             book of a series say `No description available` now, which is \
             what the view of the search and the Library view said already. \
             The Home view of a library of books held a second fault: a \
             description of no character reached the panel as it stood, and \
             the panel then held nothing at all.",
        ],
    },
    Entry {
        version: "0.8.77",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **The key `A` that adds a podcast to the library did nothing \
             in the view of the episodes of a podcast, and it said nothing.** \
             The table of the key `?` promises that key in the group \"The \
             library and the server\", beside the key `L` of the scan, the \
             key `E` of the new episodes, and the key `d` of the downloads of \
             the server, which each do their work in that view. The key `A` \
             works now in the view of the episodes: that view belongs to a \
             library of podcasts, and the key needs no line of a list. In \
             every other view the key says which two views add a podcast, and \
             it names the key that goes back to them.",
        ],
    },
    Entry {
        version: "0.8.76",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **Four keys of the table of the keys did nothing, and they \
             said nothing.** The table of the key `?` promises the key `s` \
             for the series of the library, the key `a` for the authors, the \
             key `v` for the narrators, and the key `c` for the collections \
             and the playlists, and it promises them in every view. The four \
             keys did their work in the Home view, in the Library view, and \
             in the view of the search alone: a user who pressed one of them \
             in the view of the episodes of a podcast, or in the view of the \
             queue, got no view, no message, and no line of the log at all. \
             Each of the four keys says now which two views hold that list, \
             and it names the key that goes back to them.",
        ],
    },
    Entry {
        version: "0.8.75",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **The key of the playback of the view of the episodes of a \
             podcast of a search did nothing, and it said nothing.** The \
             program reads the episodes of a podcast when the user opens it, \
             therefore the answer of that podcast comes after the view of the \
             search goes away, and the key of the playback read a list that \
             the render of the view of the search writes. A user who looked \
             for a podcast, who opened it, and who pressed the key of the \
             playback on a line of it got no playback, no message, and no \
             line of the log at all, while the key `D` of that same line took \
             the file of that same episode from the server. The keys of that \
             view read the list of that view now, and a playback that does \
             not start says why.",
        ],
    },
    Entry {
        version: "0.8.74",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **The view of the episodes of a podcast held the places of a \
             different podcast.** The program reads the episodes of a podcast \
             one time, therefore a second visit of that podcast makes no \
             request, and the places of the user came of the request alone: a \
             user who opened one podcast, went back, and opened a second \
             podcast saw the percent of the episodes of the first one on the \
             lines of the second one — `22% Letter 1`, `74% Letter 2`, and \
             `89% Letter 3` for three episodes that they never played, with \
             the panel `Progress: 22%, 28m left, Not finished`. The lists of \
             the library hold the places of each podcast now, and the view of \
             the episodes and the view of the episodes of a search each hold \
             the places of the podcast of their line.",
        ],
    },
    Entry {
        version: "0.8.73",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **The panel of an episode of a podcast said no time that is \
             left.** The panel of a book says the percent, the time that is \
             left, and the mark of the end together, and the panel of an \
             episode of the Home view, of the view of the episodes of a \
             podcast, and of the view of the episodes of a search said the \
             percent and the mark of the end alone: one program said \
             `Progress: 22%, Not finished` for an episode at 66 seconds of \
             306, while the view of a playlist that named that same episode \
             said `Progress: 100%, 0m left, Finished`. The three panels say \
             the time that is left now. An episode that the user did not \
             begin names none, an episode that plays takes the place of the \
             player of this program, and a message of the server comes at the \
             next frame.",
        ],
    },
    Entry {
        version: "0.8.72",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **The view of the books of a series and the view of the media \
             of a collection or of a playlist said no place of the user.** The \
             line of such a media held no percent and no mark of the end, and \
             the panel of it named the author and the length alone, while the \
             line of that same media of the Home view of the same screen said \
             `41% The Test Chronicles Volume 2`. The line and the panel of the \
             two views say the percent, the time that is left, and the mark of \
             the end now. A media that plays takes the place of the player of \
             this program, and a message of the server comes at the next \
             frame. The place of every media of the account comes with the \
             answer that the start reads already, therefore this costs no \
             request of the server. A media of a collection or of a playlist \
             can be an episode of a podcast, and the line of it says the place \
             of that episode and not of the podcast.",
        ],
    },
    Entry {
        version: "0.8.71",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **The line of a book of the Library view and of the view of \
             the search said no percent of the user.** The line held the mark \
             of the media that plays alone, therefore a list of 18 books said \
             no number of any of them, and a book that the user finished had \
             no mark. The line of that same book of the Home view of the same \
             screen said `84% A Book Of Many Hours`. The line of the two \
             views says the percent and the mark of the end now, and it takes \
             a message of the server at the next frame. The place of every \
             media of the account comes with the answer that the start reads \
             already, therefore this costs no request of the server.",
        ],
    },
    Entry {
        version: "0.8.70",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **The panel of a book of the Library view and of the view of \
             the search said no place of the user.** The two panels named the \
             author and the year, while the panel of that same book of the \
             Home view of the same screen said `Progress: 38%, 5h left, Not \
             finished`. The two panels say the percent, the time that is \
             left, and the mark of the end now. The place of every media of \
             the account comes with the answer that the start reads already, \
             therefore this costs no request of the server.",
        ],
    },
    Entry {
        version: "0.8.69",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **The panel of a line kept the place of the request while the \
             line of it showed a newer place.** A different client of the same \
             account moved in a book, the line said `75%` at the next frame, \
             and the panel of that same line said `Progress: 52%, 5h left`. \
             The panel of the Home view, and the panel of the two views of the \
             episodes of a podcast, read the message of the server now.",
        ],
    },
    Entry {
        version: "0.8.68",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **The panel of a line of a media that plays kept the place of \
             the moment of the request of the view.** The panel said \
             `Progress: 37%, 5h left` while the row of the player of the same \
             screen said `4:13:12 / 8:00:00 | Left: 3:46:48 (53%)`. The panel \
             of the Home view, and the panel of the two views of the episodes \
             of a podcast, read the place of the playback now.",
        ],
    },
    Entry {
        version: "0.8.67",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **The line of the media that plays in the view of the queue kept \
             the time of the moment of the key `q`.** That line said \
             `7h58m left` while the row of the player of the same screen said \
             `Left: 6:55:37`, because the place of the view came from a request \
             of that key and the playback of this same program moved the media \
             away from it. The line reads the place of the playback now.",
        ],
    },
    Entry {
        version: "0.8.66",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **A media that came into the queue while the view of the queue \
             stood open said the length of the whole media.** That line held no \
             mark of a place, because the request of the places ran at the key \
             `q` and it named the media of that moment alone. The view asks the \
             server for the place of such a media now, one time.",
        ],
    },
    Entry {
        version: "0.8.65",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **The line of an episode of a podcast in the view of the queue \
             said no time at all.** The two views that hold an episode kept \
             the length of it as a text only, therefore the key `n` put that \
             episode in the queue with no length. The line says the time that \
             is left of the episode now.",
            "- **A media that the user finished said `0m left` in that same \
             view.** The server writes the place of such a media below the \
             length of it. The line says the length of the media now, and the \
             mark of that line says the end.",
        ],
    },
    Entry {
        version: "0.8.64",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **A message of the server took the time that is left away from \
             every line of the view of the queue.** A different client of the \
             account moved in one media, and each line of that view then said \
             the length of its media again. The lines keep the time that is \
             left now, and they follow the place of the account.",
        ],
    },
    Entry {
        version: "0.8.63",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **The line of the view of the queue said the length of its \
             media.** A user who chooses the media that comes next needs the \
             time that is left, and a line of a media at 90 percent said the \
             length of the whole media. The line says the time that is left \
             now, and a media that the user did not begin keeps its length.",
        ],
    },
    Entry {
        version: "0.8.62",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **The key X of the view of the queue said no place of the media \
             that it took.** The sentence named the title alone, and a media \
             that goes out of the queue changes the number of every media \
             after it: the user could not tell which number went away. The \
             sentence names that number now.",
        ],
    },
    Entry {
        version: "0.8.61",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **The key n said one sentence for three conditions of the \
             queue.** A media that came in, a media that moved to the end, and \
             a media that stands at the end already each gave \"… is number N \
             of the queue\": a user who did not press the key q read a queue \
             that grew for each of the three. The sentence names the two \
             places of a media that moves now, and it says that a media of the \
             last line waits there already.",
        ],
    },
    Entry {
        version: "0.8.60",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **The key n on a media that waits already said a number that no \
             line of the queue held.** The message said that the media is \
             number 3 of the queue, and the view of that same key then showed \
             two lines: the disk holds one row for one media, therefore the \
             row of the second place went away. The queue holds that media one \
             time now, it moves to the end, and the number of the message is \
             the number of its line.",
        ],
    },
    Entry {
        version: "0.8.59",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **The view of the queue said nothing of the place of the \
             user.** Each line of that view is one media, and each of them \
             held the title, the author, and the length alone: no percent, no \
             mark of the media that the user finished, and no mark of the \
             media that plays. A book that the user finished and the book that \
             played at that same moment each looked like a book that never \
             played. Each line holds the place of its own media now, an \
             episode of a podcast holds the place of that episode, and a \
             message of the server moves the line.",
        ],
    },
    Entry {
        version: "0.8.58",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **The view of the episodes of a podcast said nothing of the \
             place of the user.** Each line of that view is one episode, and \
             each of them held the title alone: no percent, no mark of the \
             episode that the user finished, and no mark of the episode that \
             plays. Each line holds the place of its own episode now, a \
             message of the server moves it, and the panel of that line says \
             the percent.",
        ],
    },
    Entry {
        version: "0.8.57",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **The Home view of a library of podcasts said no place of the \
             user.** A line of that view is one episode, and the program read \
             the identity of the podcast alone: no line held a percent, the \
             mark of the media that plays stood on every line of that podcast, \
             and a message of the server reached none of them. Each line holds \
             the place of its own episode now, and the panel of that line says \
             it.",
        ],
    },
    Entry {
        version: "0.8.56",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **The view of the chapters said the name of the podcast and no \
             name of the episode.** The header of that view and the sentence of \
             a media that plays no more each named the podcast, therefore two \
             episodes of one podcast gave one set of words while the row of the \
             player of that same frame said which episode plays. The three \
             sentences name the podcast and the episode now.",
        ],
    },
    Entry {
        version: "0.8.55",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **An episode that left the shelf Continue Listening stayed on \
             the Home view.** The key `N` said that the media is away from that \
             shelf, and the line of it stayed. Every episode of one podcast \
             holds the identity of that podcast, therefore the program read the \
             identity of the episode now: the line of that episode goes away, \
             and every other episode of the podcast keeps its line.",
        ],
    },
    Entry {
        version: "0.8.54",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **The row of the player said the name of the podcast and no \
             name of the episode.** Two episodes of one podcast gave one row, \
             and the queue changes the episode with no key of the user: the \
             length of the media was the one value that moved, and a length \
             names no episode. The row says the podcast and the episode now.",
        ],
    },
    Entry {
        version: "0.8.53",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **The key `b` of the bookmarks of a podcast wrote a place of a \
             different episode.** The queue starts the media of its front with \
             no key of the user, and every episode of one podcast holds the \
             identity of that podcast: the guard of the view let a second \
             episode pass, and the key wrote a place of it with no word at \
             all. The view keeps the episode now. The key says that a \
             different episode plays, and the key `V` gives the bookmarks with \
             the episode that plays.",
        ],
    },
    Entry {
        version: "0.8.52",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **The key `V` of an episode of a podcast said that no media is \
             selected.** Audiobookshelf holds the bookmarks of a podcast, and \
             it names no episode in them: the view of the episodes gave no \
             bookmark at all, and the Home view of a library of podcasts named \
             the episode of the line above the places of the whole podcast. \
             The view names the podcast now, and the key `V` of an episode \
             opens it.",
            "- **The key that goes to the place of a bookmark moved the \
             playback of one episode to a place of a different episode.** The \
             key does the work of the user, and it says that a bookmark of a \
             podcast names no episode.",
        ],
    },
    Entry {
        version: "0.8.51",
        date: "15/08/2026",
        body: &[
            "Fixed:",
            "- **The key `e` and the key `V` of a line of a series did the work \
             of the first book of that series.** A line of the Library view \
             holds every book of one series, and the reader of the ebook and \
             the view of the bookmarks took one book of it with no word at all. \
             A line of a library of podcasts holds the episodes of one podcast, \
             and the two keys asked the server for the ebook and for the \
             bookmarks of the podcast. The two keys say what the line holds \
             now, and they name the key `l` that opens it.",
            "- **The line of a series of the Home view said that it holds no \
             media.** The Home view of a library of books holds a shelf of the \
             series, and the same line of the Library view said that it holds \
             more than one book. The keys `D`, `X`, `n`, `m`, `@`, `M`, `N`, \
             `e`, and `V` of that line name its books now.",
        ],
    },
    Entry {
        version: "0.8.50",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **The key `D` and the key `X` did nothing for a line that holds \
             more than one media.** A podcast of the Library view and of the \
             view of the search holds its episodes, and a line of a series of \
             the Library view holds the books of that series: the two keys need \
             one media, and they gave no word of the screen and no line of the \
             log. The two keys say what the line holds now, and they name the \
             key `l` that opens it. The keys `n`, `m`, and `@` of those lines \
             say the same.",
            "- **The keys `M` and `N` of a line of a series said that a podcast \
             holds no place.** A series of a library of books is no podcast, and \
             the key `l` of that line gives its books. The words of the line \
             name the books now.",
        ],
    },
    Entry {
        version: "0.8.49",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **The key `M` and the key `N` did nothing for an episode of a \
             podcast.** An episode of a podcast holds its own place on the \
             server, and the address of that place names the episode after the \
             podcast: Toutui asked for the address of the podcast alone. In the \
             Home view of a library of podcasts the server answered the read \
             with the place of **another** episode, and it refused the write \
             with the status 400; in the view of the episodes of a podcast the \
             two keys said \"No media is selected.\" for a line that holds an \
             episode. The two keys name the episode of the line now. The line \
             of a podcast holds no place of its own, therefore it names the key \
             `l` of its episodes.",
        ],
    },
    Entry {
        version: "0.8.48",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **The keys `D`, `X`, `n`, and `m` did nothing for a media of the \
             view of the search that stands in no line of the list of the \
             library.** Toutui reads one page of 500 items of a library and it \
             groups the books of a series in one line, and the server searches the \
             whole library: a book of a page that Toutui did not read, and a book \
             of a series of more than one book, are such a media. A measurement of a library of \
             2056 items and of the line \"Large Book 1200\" of the search: the key \
             `D` and the key `X` wrote no word at all, the key `n` said \"This line \
             holds no media.\", and the key `m` said \"This line holds no book and \
             no episode.\" The key `l` of that same line played the book. Every \
             list of the view of the search holds one value for each line of it, \
             therefore those four keys take the media of the line now, and they \
             need no page of the library.",
        ],
    },
    Entry {
        version: "0.8.47",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **The label `[Downloaded]` stayed on a media whose copy on the disk \
             is not whole.** Toutui reads the rows of its database to find the \
             copies of the disk, and a row is no file: a book that lost one of its \
             files, or one whose file lost some of its bytes, kept that label while \
             every playback of it took the road of the server. The line of such a \
             media says \"the disk does not hold every file\" now. Toutui asks the \
             file system at the start, at the key `R`, at the end of a download, \
             and at the key `X`, therefore the screen reads no disk.",
        ],
    },
    Entry {
        version: "0.8.46",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **A file of the disk that lost some of its bytes played less than \
             the book, and Toutui told the server that you finished it.** Toutui \
             asks the disk whether the file of a download stands there, and a file \
             that lost bytes stands. A measurement of a book of three files of 20 \
             seconds, with the second file at half of its bytes, played 50 seconds \
             of the 60 of the book with no word at all, and the server then held \
             that book as finished. The row of a file holds the size of the file of \
             the server, therefore a file of another number of bytes is no file of \
             that row now: the media takes the road of the server, the offline mode \
             says \"The disk does not hold every file of this media.\", and the log \
             names the file and the two numbers.",
        ],
    },
    Entry {
        version: "0.8.45",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **A book of the disk that lost one file stopped at that file.** \
             Toutui reads the rows of its database to find the copy of a media on \
             the disk, and a row is no file: a file that goes away outside Toutui \
             — you remove it, or a directory of the machine goes away — left the \
             row of it. A measurement of a book of three files of 20 seconds, with \
             the second file away, played 20 seconds of 60 and it said nothing at \
             all, and the whole book stood on the server. Toutui asks the disk at \
             the moment of the playback now: a copy that is not whole takes the \
             road of the server, and the log names the file that went away.",
            "- **The offline mode said that a media plays from the disk and it \
             played nothing.** The check of that mode compared the files of the \
             book with the rows of the same table, therefore a file that went away \
             passed it every time: the engine then stopped at that file with no \
             word for you. The offline mode says \"The disk does not hold every \
             file of this media.\" now.",
        ],
    },
    Entry {
        version: "0.8.44",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **The key `X` removed one half of a download when the database \
             refused the other half.** A download stands in two tables: the row \
             of the media, and one row for each file of it. Toutui removed the \
             two of them with two statements, therefore a database that refused \
             the second one kept the first one. The media then stood in the rows \
             of its files alone: the key `X` again said that the media holds no \
             local copy, and every playback of that media took the road of the \
             disk for files that went away. The two removals stand in one \
             transaction now, therefore the rows of a download go away together \
             or they stay together, and the key `X` again does the work.",
            "- **Toutui says when its database keeps a part of a download.** A \
             download whose rows the database refused goes away again with a \
             removal, and Toutui read no answer of that removal: a database that \
             refused it too left a media that holds no file of the disk, with no \
             word for you and no line of the log. That media stood in the \
             offline mode with the label of a download, and the playback of it \
             then said that the disk holds no audio file. Toutui says it on the \
             screen now, and the key `D` writes every row of that download \
             again.",
        ],
    },
    Entry {
        version: "0.8.43",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **Toutui forgot that you finished a book when its database \
             refused one write.** The mark of the end of a media stands in the \
             row of its playback, and that row is the one copy of that mark for \
             a Toutui that dies. Toutui read no answer of that write: a \
             database that refused it therefore gave the next Toutui of your \
             account a book that you finished and that the row says is not \
             finished, and the next Toutui then sent your place to the server \
             with no mark at all. The server keeps the mark of a media that \
             ends in its last ten seconds by its own arithmetic, and it loses \
             the mark of every media that ends earlier: a measurement lost the \
             end of a book of 30 minutes in that way. Toutui puts the place and \
             the mark in the table of the places that wait now, and the mark \
             reaches the server with that place.",
            "- **Toutui says when the mark of the end of a media stands on no \
             machine.** A database and a server that each refuse that mark take \
             it away, and Toutui said nothing at all: no word of the screen, and \
             no line of the log. Toutui says it on the screen now, and it keeps \
             the row of that playback, because the place of the user in that row \
             is worth more than the mark that went away with it.",
        ],
    },
    Entry {
        version: "0.8.42",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **Toutui removed the last copy of your place in a book when the \
             server and the disk each refused it.** The program closes the \
             session of a playback, it sends your place to the server, and it \
             then removes the row of that place from its database — the rule is \
             that the row goes away after the server holds that place or the \
             table of the places that wait holds it. Toutui read no answer of \
             that second write: a server that reported a fault and a database \
             that took no row therefore left your place on no machine at all, \
             and the removal took the one row that held it. A measurement lost \
             757 seconds of a book of eight hours in that way. The row stays \
             now, and the next Toutui of your account sends that place.",
            "- **Toutui said that the server holds a place that the server \
             refused.** A removal of such a row that the disk refuses puts that \
             session in a box, and Toutui sends the place of a session of that \
             box to the server no second time: the words of the box named the \
             server for every row, therefore a place that no machine held went \
             away with the next key. The box names the machine that holds the \
             place now.",
        ],
    },
    Entry {
        version: "0.8.41",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **A place of a playback that the server took stayed on your disk, \
             and Toutui sent it to the server again every 30 seconds.** The task \
             that sends the places of an offline playback removes the row of each \
             place that the server takes, and a disk that is full or a database \
             with no permission of a write keeps that row: the same place of the \
             same media then went to the server for the whole life of the \
             program, and the header of the offline mode said that a place of \
             yours waits for a server that holds it already. Toutui names that \
             disk in the log now, and it stops the attempt: every row of one \
             attempt stands on one disk.",
        ],
    },
    Entry {
        version: "0.8.40",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **A disk that took no write took the place of your playback away, \
             and nothing on the screen said it.** Toutui writes the place of a \
             media to its database at each second, and a disk that is full or a \
             database with no permission of a write takes each of those seconds \
             away: a book of eight hours from your disk then started at its first \
             minute again. The row of the player says now that the disk keeps no \
             place of that media, and that word stands while the condition \
             stands.",
            "- **The log took one line for each second of such a playback.** A \
             book of eight hours gave 28800 lines. Toutui says the fault of the \
             disk one time now, and it says it again after a write that the disk \
             took.",
        ],
    },
    Entry {
        version: "0.8.39",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **A media of yours played at the speed 1.00x while your account \
             held another speed, and nothing said why.** Toutui reads the speed \
             of your account from its database at the start of each playback, \
             and a database that did not answer gave the speed 1.00x: that is a \
             speed which you did not choose. Toutui says now which speed the \
             media plays and that its database did not answer.",
            "- **The sequence and the filter of your library went away with no \
             word.** The same database gave Toutui a library of no sequence, and \
             the header of the Library view then said `Library [17 items]` for a \
             library that you put in the sequence of the title. Toutui stops with \
             words that name its database now, and the key `R` keeps the \
             application that stands.",
            "- **The keys `O` and `I` gave your media the speed 1.00x after a \
             write that your disk took.** Toutui writes the new speed and it \
             reads that row again: the read that failed gave the speed 1.00x to \
             the player, therefore the key of a faster media made a slower one. \
             The media keeps its speed now, and the key says why.",
        ],
    },
    Entry {
        version: "0.8.38",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **A key of yours took 35 seconds while a second Toutui of your \
             account wrote the database, and the screen said nothing for 20 of \
             them.** Toutui waits five seconds for a database that another \
             program holds, and one key of yours makes more than one call of \
             that database: the key `l` of a media made seven calls, and it paid \
             those five seconds seven times. Toutui pays that wait one time for \
             each key now, and the same key says why in six seconds.",
        ],
    },
    Entry {
        version: "0.8.37",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **Toutui sent an old place of a book to your server, and the \
             place of that book went backward.** A disk that is full, a database \
             with no permission of a write, and a file system that is read-only \
             each give the condition: Toutui gave the place of your book to the \
             server, it could not remove the row of that playback from its own \
             database, and it therefore sent that same place again. A book of \
             eight hours went from 1h40m back to 10m, and every client of your \
             account then held that place. Toutui gives a place to your server \
             one time now.",
            "- **A media of yours started 30 seconds after the key `l`, and the \
             screen said `Syncing your last listening session. Please wait...` \
             for those 30 seconds.** Toutui waits for the playback before the \
             new one, and the mark of the end of that playback stood in the \
             database alone: a disk that took no write left that mark, and the \
             wait then held the whole limit of time. Toutui holds the mark of \
             its own playback now, and the media starts at once.",
            "- **A write of the database that failed said nothing at all.** The \
             place of your playback of each second, the pause of the player, the \
             chapter, and the marks of the wait each wrote the disk with no \
             reader of the answer. The log names each of them now.",
        ],
    },
    Entry {
        version: "0.8.36",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **The key `n` said that your media is in the queue while the \
             disk of your account took no media at all.** A disk that is full, \
             a database with no permission of a write, and a file system that \
             is read-only each give that condition: the key said `\"…\" is \
             number 1 of the queue. Press q to see the queue.`, and the key `q` \
             of that same sentence then said that the queue is empty. A change \
             that the disk did not take is no change now: the queue of the \
             program goes back to the queue of the disk, and the key says why.",
            "- **The key `X` of the view of the queue took a media out of the \
             screen while the disk kept it.** The media came back with the next \
             read of the disk. The key now says why, and the media of your \
             queue stays on the screen.",
            "- **The keys `O` and `I` of the speed said nothing at all when the \
             disk did not take the new speed.** The key read the row of the \
             account after the write, therefore the engine took the speed of \
             before, the screen said nothing, and the log held no line. Each of \
             the two keys says why now.",
        ],
    },
    Entry {
        version: "0.8.35",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **The key `R` took the whole program away while a second Toutui \
             of your account wrote the database.** The refresh read the \
             accounts of the disk, that read met the database of the other \
             program, and Toutui stopped with the words of a start that has no \
             account: your lists, your account, and your playback went away \
             with it. The key of the next library did the same. A refresh is \
             not a start, therefore the program keeps the screen that stands \
             and it says why that screen did not change.",
            "- **The key of the next library said that it shows the other \
             library while the database took nothing.** The three keys that \
             write the library and the sequence of a library now read the \
             answer of the disk: each of them says why, and the sequence of \
             the screen goes back to the sequence of your row.",
            "- **Two functions of the database said that they did their work \
             for a database that they did not open**: the sequence of a \
             library and the queue of the media. Each of them gives a fault \
             now.",
        ],
    },
    Entry {
        version: "0.8.34",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **The program stopped for 15 seconds at a time while a second \
             Toutui of your account wrote the database.** The row of the \
             player, the timer for sleep, and the cursor of every list stood \
             still, and the keys of the user came together at the end: five \
             presses of the key `j` moved no line for 30 seconds. The work of \
             the database of a task now stands on a thread of its own, \
             therefore the screen draws each second and the keys of the user \
             do their work while the database waits.",
            "- **The row of the keys of the player went away while you turned \
             nothing off**, and the mark of a copy on your disk went away from \
             every media: a database that did not answer became a fact of the \
             user. The program keeps those two values, and the screen asks the \
             database at no frame at all.",
            "- **The key `B` did nothing when the database did not answer.** \
             That key read the value of the disk before it wrote it, and a \
             read that failed matched neither value: the key now writes the \
             value that the program holds, and a write that failed says why.",
        ],
    },
    Entry {
        version: "0.8.33",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **The key `X` removed the files of a book that a second Toutui of \
             your account played from the disk.** That key asks the database \
             which media of your account plays now, and a second Toutui that \
             held the database took the answer away: the program then read \
             \"no program plays it\", and the three files of the book went \
             away under the ear of the user. The rows of that download stayed, \
             and no word of the screen named the fault. The key removes no file \
             now while the database says nothing, and it says so.",
            "- **The offline mode said that the server gave no media while your \
             downloads stood on the disk.** A database that did not answer the \
             read of the downloads gave a list with no media, and the Library \
             view named the server. The view names the database now, the log \
             holds the fault, and the line of a media says \"the disk did not \
             answer\" in place of nothing at all. The offline playback, the \
             engine of the audio, the places that wait for the server, and the \
             number of them in the header each hold the same rule.",
        ],
    },
    Entry {
        version: "0.8.32",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **The view of the queue said that your queue is empty while your \
             media stood on the disk.** A second Toutui of your account that \
             held the database took the answer of that read away, and the \
             program then read a queue with no media: the view said \"The queue \
             is empty.\", and a key of the queue after it wrote that emptiness \
             on the disk — the media of every Toutui of your account then went \
             away. The queue of the program keeps its media now, a key that \
             cannot read the disk changes nothing and it says so, and the \
             program no longer says that your account is gone when it cannot \
             read that account.",
        ],
    },
    Entry {
        version: "0.8.31",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **A playback that kept no place played with no row of the player \
             at all.** The row of the disk holds the place of your playback for \
             a program that stops without an exit, and the row of the player of \
             the screen reads that row: a second Toutui of your account that \
             held the database took that row away, therefore the audio played, \
             the row of the player said \"N/A\", and the place of the whole \
             playback reached no disk. The playback does not start now, and the \
             program says that it did not keep the session on its disk.",
        ],
    },
    Entry {
        version: "0.8.30",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **A download that reached the disk held no row of the database, \
             and the program said that the media is available offline.** 21 \
             functions of the database of the program said that their work was \
             successful when they got no connection at all, and the words of \
             that fault were \"Error connecting to the database.\" in the row of \
             the message of every view. A measurement with a second Toutui of \
             one account on the database: the file of an episode stood on the \
             disk, the line of that media held no mark of a download, and the \
             offline mode did not find it. The program now says that its \
             database did not take the download, and it names the key `D`; a log \
             out that removed no row of an account says so too; and no function \
             of the database writes a word for you, because a word of a fault \
             belongs to the view that you see.",
        ],
    },
    Entry {
        version: "0.8.29",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **A fault of the database of the program became a program with \
             no account.** A second Toutui of your account writes the same \
             database, and a write of that program holds the file for a moment: \
             the program then read a database of no account. A login wrote no \
             row and it said that it was successful, therefore the login screen \
             came back with no reason and it did that for ever; a key of the \
             program said that your account is gone and it started the program \
             again, and your account stood on the disk all the time. The login \
             says now that it did not write the account, a key keeps your \
             account and it writes the fault in the file of the log, and a \
             program that cannot read its accounts stops and says so.",
        ],
    },
    Entry {
        version: "0.8.28",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **A cover that came in part became an item with no cover.** The \
             program held the end of the connection as the end of the picture, \
             therefore the bytes of a part of a cover stood in the memory of \
             the program: no reader of a picture read them, the item showed no \
             cover, and the log said that the cover came. The program says the \
             fault now, and it names the key `R` that asks your server again.",
        ],
    },
    Entry {
        version: "0.8.27",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **A part of the program that had an internal fault left you with \
             a terminal that takes no key.** The program gave the terminal back \
             to your shell and it then continued: the screen wrote over the \
             words of the fault, the keys did nothing, and the key `Q` did not \
             stop the program. A measurement with a fault in the loop of a \
             playback: the sound played on, and your place stayed at the start \
             of the book on your server for the whole book. The program stops \
             now, it says that it stopped, and the file of the log holds the \
             words of the fault.",
        ],
    },
    Entry {
        version: "0.8.26",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **A book of the reader that came in part became a book that no \
             reader opens.** A machine between you and your server can end the \
             answer of a book in the middle, and that answer can look whole: \
             the program then kept the first bytes of the book on the disk \
             with the name of the whole book, it said \"This file is not an \
             EPUB.\", and it asked your server for nothing at every visit \
             after it. A measurement gave 20000 bytes of a book of 136761 \
             bytes. The program counts the bytes of the answer against the \
             size that your server gives now, a book that came in part leaves \
             no file on the disk, and you read the two numbers.",
        ],
    },
    Entry {
        version: "0.8.25",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **A book that your server transcodes and that gave a part with no \
             sound became a book that you listened to.** Such a book comes in \
             parts, and a part of your server can hold no sound at all: the \
             program went to the part after it with no word for you, it then \
             wrote the end of the whole book to your server, and it said that \
             you finished the book. A measurement gave a book of ten minutes \
             that held six seconds of sound. The program writes the place that \
             it really reached now, the book stays in Continue Listening, and \
             you read why the book stopped.",
            "- **A book whose first part of such a stream holds no sound says \
             so.** The playback of that book gave no sound and no word before, \
             and the program then said that you finished the book. It does not \
             start now, and it names the part of your server.",
        ],
    },
    Entry {
        version: "0.8.24",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **A book that your server transcodes and that stopped in the \
             middle became a book that you listened to.** Such a book comes in \
             parts, and a part that did not come stopped the playback: the \
             program then wrote the end of the whole book to your server, it \
             said that you finished the book, and the screen said nothing at \
             all. A measurement gave a book of ten minutes that stopped after \
             six of them. The list of the parts is the truth of the length now: \
             the program writes the place that it really reached, the book \
             stays in Continue Listening, and you read why the book stopped.",
            "- **A part of such a book that came in the middle takes a second \
             attempt.** A part holds packets of one size, and a part of a \
             different length is a part that stopped. The program asked for \
             such a part one time only, and every other fault of a part takes \
             twenty attempts. It now takes the same road, therefore a body that \
             stopped makes no hole in the sound.",
        ],
    },
    Entry {
        version: "0.8.23",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **A book whose audio stopped in the middle became a book that you \
             listened to.** The program reads the file of a book from your \
             server, and a connection that closes before the end of that file \
             looked the same as the end of the file: the playback of a book of \
             30 minutes stopped after five seconds, the program told your server \
             that you finished the book, and it said nothing at all. The number \
             of the bytes of the file is the truth of the length now. A body \
             that stops before it is a connection that stopped, and the program \
             asks your server again from the byte that it holds.",
            "- **The same for a book that your server transcodes.** The list of \
             the parts of such a book names its own end, and a list that stopped \
             in the middle named fewer parts with no fault of its own. The \
             program refuses such a list now, and it says so.",
        ],
    },
    Entry {
        version: "0.8.22",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **A collection or a playlist that your server gave with no identity \
             made every key of it say a fault of your media.** The program \
             cannot ask your server for such a list — it has no address — and \
             the key that puts a media in it said \"The server did not take the \
             media: The server does not have this item\", of a media that your \
             server holds. Such a list belongs to no line now, and the log says \
             which list went away.",
            "- **The same for one media of a collection or of a playlist.** A \
             media of a list with no identity took a line, and the key of the \
             playback of that line said that your server does not have the \
             item. That media belongs to no line of the list now.",
        ],
    },
    Entry {
        version: "0.8.21",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **One shelf of your Home view took every shelf away.** Your server \
             gives the shelves of the Home view in one answer, and a shelf that \
             held no name made the whole answer not valid: the view then said \
             that your server gave no shelf, and Continue Listening and every \
             other shelf went away with it. A shelf with no name keeps its media \
             now, and the line of it holds the identity of that shelf.",
            "- **One library of your server stopped the whole program.** Your \
             server gives its libraries in one answer, and a library that held \
             no name made the whole answer not valid: the program stopped, and \
             your other libraries were correct. The program reads each library \
             apart now. A library with no name keeps its line, and that line \
             holds the identity of the library; a library with no identity and a \
             library with no media type belong to no line, because the program \
             cannot ask your server for the media of them.",
        ],
    },
    Entry {
        version: "0.8.20",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **A place that your server did not take went away for ever.** The \
             program keeps such a place on your disk, and it sends it when your \
             server answers again — but it did that work for a server that does \
             not answer alone: a server that reported a fault took the place of \
             your last playback away with the row of it, and the program said \
             that it closed that media at that place. A place of your disk waits \
             for your server now, and the program tries again every 30 seconds. \
             Two answers of your server keep their road: the media that your \
             server does not hold, and the request that your server refused.",
        ],
    },
    Entry {
        version: "0.8.19",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **A place that you listened to with no server went over the newer \
             place of your server.** The program asks your server for the place \
             of a media before it sends the place of your disk, and a fault of \
             that question was the answer \"this media never played\": your \
             server held 5000 seconds of a book of eight hours, and the program \
             wrote 100 seconds over it. The place of your disk waits for an \
             answer that came back now, and the program tries again every 30 \
             seconds.",
            "- **A place of one episode of a podcast that you listened to with no \
             server went away.** The program asked for the place of the podcast, \
             and your server answers that question with the place of **one** \
             episode of it: the moment of another episode then took the place of \
             this one away. The program asks for the place of that episode now.",
        ],
    },
    Entry {
        version: "0.8.18",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **A book that loses a file on your server kept that file on your \
             disk.** The offline playback then played a part that the book does \
             not hold, and it sent the place of that part to your server: a book \
             of three parts that became a book of one part played for 60 seconds, \
             and the server holds 20 seconds of it. The key D makes the copy of \
             your disk the book of the server now, and it removes the file that \
             left that book.",
        ],
    },
    Entry {
        version: "0.8.17",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **A book that did not come whole stayed a book of your disk, and \
             the program said that it is not an EPUB.** A connection that went \
             away in the middle of the download left a part of the book with the \
             name of the whole book, and the reader of every program of your \
             account after it opened that part: it asked the server for nothing, \
             and the key X of the list was the one road out. The program gives \
             the file the name of the whole book at the end of the download now, \
             and a download that fails leaves no part of a book behind it.",
            "- **A part of a book of a program that stopped in the middle of a \
             download is a file of the key X now.** That key removes the copy of \
             a book of your disk, and it did not see such a file.",
        ],
    },
    Entry {
        version: "0.8.16",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **A cover that did not come one time stayed away for the whole \
             life of the program.** A request of a cover that came back with a \
             fault took the condition of a book with no cover, and the key R \
             asked the server for every list again and for no cover at all: the \
             book of your screen then held no picture, and no key could correct \
             it. The key R asks the server for every cover again now. A cover \
             that a different client of your account changes comes at that key \
             too.",
            "- **The log said that a book has no cover for a book whose cover \
             the server holds.** A request that came back with a fault says the \
             fault now, and the status 404 is the answer of a book with no \
             cover.",
        ],
    },
    Entry {
        version: "0.8.15",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **A media whose position went away on the server kept its old \
             percent on the screen, and the key R could not correct it.** A \
             different client of your account can take the position of a media \
             away, and Toutui then held the value of the live message of the \
             server for ever: that value stands above the value of every request. \
             A live message carries the position of every media of your account, \
             therefore that list takes the place of the list before it now, and \
             the key R asks the server for every position again.",
        ],
    },
    Entry {
        version: "0.8.14",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **One device of an e-reader with no name took every device of your \
             account away.** A server that does not give the name of one device gave \
             Toutui a fault of the whole answer: the view of the key @ said that the \
             server gave no device, and the device that you can use stood in no line. \
             Toutui reads each device apart now, a device with no name belongs to no \
             line, and every other device of the server stays.",
            "- **A device that the server no longer holds named no key.** Toutui said \
             \"Press the key again for the new list\", and the view of the devices is \
             away at that moment. Toutui names the key @ of that work now.",
        ],
    },
    Entry {
        version: "0.8.13",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **A book started at its first second and it lost your place, for a \
             server that does not give your place with the session of the playback.** \
             Toutui read that absence as the place 0: the book played from the start, \
             the loop of the playback sent that start to the server, and your place \
             went away on the server too. Toutui asks the server for the place of the \
             media now, and a media that never played is the one media that starts at \
             0.",
            "- **A playback of a session that the server did not name sent your \
             listening time to nobody.** Toutui names the session in each request of \
             the position and in the request of the close, therefore every one of them \
             came back with a fault, the session of the server stayed open, and no word \
             said why. Such a playback does not start now, and Toutui says that the \
             session of the server has no identity.",
        ],
    },
    Entry {
        version: "0.8.12",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **A download left a file of the book on the server, and it said that \
             the book is available offline.** A server that does not give the identity \
             of one audio file gave Toutui no address of that file: Toutui took the \
             other files alone, it wrote a book with a hole in it to the disk, and it \
             said nothing. Toutui makes no plan of such a book now, and it names the \
             file that it cannot ask for.",
            "- **A book of many files came to the disk in the wrong sequence, for a \
             server that does not give the number of one file.** Two files then held \
             the number 1: the last file of the book stood in the middle of it, two \
             files took one name on the disk, and one of them left the list of the \
             files of the download. A book whose files hold no number takes the \
             sequence of the answer of the server now.",
        ],
    },
    Entry {
        version: "0.8.11",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **A book started at the first second, and it lost your place, for a \
             server that does not give the length of an audio file.** Toutui looks for \
             the file that holds your place, and a file with no length has no end that \
             Toutui can find: it took the last file of the book and it made no movement \
             to your place. A book of one file therefore played from the start, the row \
             of the player said `0:0` for the length of the book, and no word said why. \
             The session of the playback holds the length of the media now, and a book \
             of one file takes it: your place comes back. A book of many files keeps \
             your place in the first file of no length, and the row of the player says \
             `N/A` for a length that Toutui does not have.",
        ],
    },
    Entry {
        version: "0.8.10",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **A download failed after Toutui wrote every byte of the book, for a \
             server that does not give the size of a file.** Toutui compared the bytes \
             that came with a size of zero, therefore it said \"the server sent 20554 \
             bytes for alice.mp3, but the file has 0 bytes\" and it kept no file. The \
             next press of the key `D` removed that work and asked the server for the \
             whole book again. A size that the server does not give changes nothing \
             now: the end of the answer is the end of the file, a file that is on the \
             disk needs no second download, and the bar of the download shows the bytes \
             that came.",
        ],
    },
    Entry {
        version: "0.8.9",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **The reader wrote the first page of a book to the server when it could \
             not read your place.** Toutui asks the server where you stopped reading, and \
             it sends your place back while you read. A request that came back with a \
             fault gave the reader the first page of the book, and the send after it took \
             your place away on every machine of your account. Toutui writes no place \
             now, and the reader says what the server said.",
            "- **Every position of your media went away for a server of another \
             version.** The answer of the server holds the position of each of your media, \
             and a server of an older version holds two fields fewer in it. Toutui then \
             read no position at all: the Home view held no percent, no mark of a book \
             that you finished, and no word said why. Toutui reads the media of a \
             position now, and every other field of that answer changes nothing.",
        ],
    },
    Entry {
        version: "0.8.8",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **The keys `M` and `N` wrote the wrong state when the server did not \
             answer their first request.** Each key reads the state of the media and it \
             then writes the opposite of it. A read that came back with a fault gave the \
             key the value \"not finished\" and \"not away from the shelf\" at every \
             press: a book that you finished stayed finished, a book that you took away \
             from Continue Listening stayed away, and Toutui told you that it made the \
             change. Toutui writes nothing now, and it says what the server said.",
            "- **One field of a library that Toutui does not read stopped Toutui.** The \
             list of the libraries of a server of another version can hold one field \
             fewer, and Toutui then said \"it cannot read the lists of the server\" and \
             stopped. Toutui reads the id, the name, and the type of a library now, and \
             every other field of that answer changes nothing. The words of an answer \
             that Toutui cannot read name the field too.",
        ],
    },
    Entry {
        version: "0.8.7",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **A login of an account that reaches no library took Toutui away.** The \
             screen went black and no key did anything: a new server before its first \
             library, and an account that an administrator gave no library, each gave that \
             answer. The login says that the server gave no library now, and it asks you to \
             speak to an administrator of the server.",
            "- **A fault of one thread stopped Toutui with a screen of no character.** The \
             screen of the login, the screen of the search, and the box that asks you for a \
             name each held the standard output, therefore Toutui could not tell you what \
             happened. Those three screens hold it no more.",
        ],
    },
    Entry {
        version: "0.8.6",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **A server that reports a fault stopped Toutui with a line of its own \
             source.** The terminal held \"Error: The server reported a fault. Status \
             500.\" with \"Location: src/app.rs\", and that text names no road. Toutui \
             says what the server said now, it names your account and your server, and it \
             tells you that it changed nothing.",
        ],
    },
    Entry {
        version: "0.8.5",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **The header said that your server does not answer, for a server that \
             answers.** One request that came back with a fault took the address away, and \
             the two lines at the top then said \"the server does not answer\" and they \
             offered you the media of the disk. The server holds every list, and the header \
             says \"the server reports a fault\" now: the key R asks the server again.",
        ],
    },
    Entry {
        version: "0.8.4",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **Three more views said that your library holds nothing.** The server \
             answered, and it gave a fault for the shelves, for the series, or for the \
             media of the library: the Library view then said \"This library holds no \
             media.\" for a library of 17 books, and the view of the series said \"This \
             library has no series.\" for a library of three. The three views say what the \
             server said now, and the key R asks the server again.",
        ],
    },
    Entry {
        version: "0.8.3",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **The view of the collections and of the playlists said that your library \
             holds none.** The server answered, and it gave a fault for those two lists \
             alone: the key c then said \"This library has no collection and no playlist.\" \
             for a library that holds both, and the key m asked you to make a list of a \
             server that the program did not read. The two views say what the server said \
             now.",
        ],
    },
    Entry {
        version: "0.8.2",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **A request of the server that did not come back said nothing at all.** The \
             server went away while the program stood, and two keys then held their \
             silence. The view of the episodes of a podcast said \"The program gets the \
             episodes of this podcast…\" for ever, and the program had stopped that work at \
             the first second: the view says what the server said now. The key G of the \
             library waited for the end of a library of 2056 items, 500 of them stood on \
             the screen, and no word came: the program says one sentence now, and it stops \
             the work of that key.",
        ],
    },
    Entry {
        version: "0.8.1",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **A playback that did not start said nothing at all.** You pressed l, you \
             read \"Loading the media...\", and then you read an empty row: the media did \
             not play, and the program never said why. A second program took one episode \
             out of a podcast, and the key l of the view of the episodes and the key l of \
             the Home view both gave that silence. The program says one sentence for each \
             of the three faults now — the server did not start the playback, the server \
             did not give the media, and this media has no audio file — and that sentence \
             names what the server said.",
        ],
    },
    Entry {
        version: "0.8.0",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **The key X emptied the queue of a podcast that you did not choose.** The \
             view of the downloads of the server moves with no key of any user: the server \
             takes an episode out of the queue when it downloaded it. The line of that view \
             held a number of a line, therefore an episode of another podcast moved under \
             your line with no word at all — a user who stood on \"Chapter 10\" of \
             \"Narrative of Arthur Gordon Pym\" pressed X two times, and the key emptied the \
             queue of \"Letters of Two Brides\" and took eight episodes away. **That queue \
             holds the work of the server for every user of it. Your line holds an episode \
             now**, and it follows that episode. An episode that leaves the queue takes the \
             line to nobody with a message that names it, the keys j and k give a line \
             again, and the key X on a line of nobody says one sentence and it changes no \
             queue at all.",
        ],
    },
    Entry {
        version: "0.7.99",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **A key of yours reached a collection or a playlist that you did not open.** \
             The line of the view of the lists held a number of a line, therefore a list that \
             a second window of your account removed moved the list below it under your line \
             with no word at all: a user who read the media of \"A Test Playlist\" pressed X \
             one time, and the key took a media out of \"Z Second Playlist\". A list that went \
             away left that view with no title, no line, and a footer of five keys that did \
             nothing. **Your line holds a list now**, and it follows that list. A list that \
             goes away takes the line to nobody with a message that names it, the keys j and k \
             give a line again, and the media of a list that went away shows you the \
             collections and the playlists again.",
        ],
    },
    Entry {
        version: "0.7.98",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **You read the message of a view that you were not in.** Three parts of the \
             program write a message with no key of you: a media that leaves the shelf \
             Continue Listening, a media that leaves your queue, and a media of the view of \
             the chapters that stops. All three wrote to one row of the screen, and the last \
             one won. A user who looked at their queue while the media came to its end read \
             \"the media is not on the shelf Continue Listening now\" for the whole six \
             seconds, and the sentence of their own view never came. **A message of a view \
             waits for that view now**, and its six seconds start when you read it: you get \
             the sentence of the view that you look at, and the sentence of the other view \
             when you go there.",
        ],
    },
    Entry {
        version: "0.7.97",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **The key `b` of the view of the bookmarks wrote a place of a book that you did \
             not choose.** The media that plays changes while you look at that view and you \
             press no key: the media comes to its end, and the queue then starts the media of \
             its front. The list kept the bookmarks of your media and the key wrote a place of \
             the media that plays, therefore a bookmark of a book of eight hours went to the \
             server at 5:25:30 while your view showed the one bookmark of a book of 30 minutes. \
             The key `b` of that view writes a place of the media of that view now, and it says \
             the reason when that media does not play.",
            "- **The title of the view of the bookmarks named no media.** It says `The bookmarks \
             of \"…\"` now, therefore you can tell whose places you read.",
        ],
    },
    Entry {
        version: "0.7.96",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **The key `l` of the view of the chapters moved the place of a book that you did \
             not choose.** The media that plays changes while you look at that view and you \
             press no key: the media comes to its end, and the queue then starts the media of \
             its front. The list became the list of chapters of that other media and your line \
             kept the number of the line, therefore the key `l` moved the place of a book of \
             eight hours by 43 minutes, and the server took that place. Your line goes away now \
             when the media of the view stops, the message names that media, and the keys `j` \
             and `k` give you a line again.",
        ],
    },
    Entry {
        version: "0.7.95",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **A key of the view of the queue changed a media that you did not choose.** The \
             queue moves while you look at it and you press no key: the media that plays comes \
             to its end, and the queue then takes the media of the front away. The lines kept \
             the number of the line, therefore the media below moved under your cursor with no \
             word at all — the key `X` took that media out of your queue, and the key `l` \
             played it and stopped the media that the queue had started. Your cursor goes with \
             the media that you chose now. A media that leaves the queue takes your line to \
             nobody, the message names that media, and the keys `j` and `k` give you a line \
             again.",
        ],
    },
    Entry {
        version: "0.7.94",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **A key of the Home view changed a media that you did not choose.** A media that \
             you mark as finished goes away from the shelf Continue Listening, and the media \
             below it took your line with no word at all: the key `M` of three presses therefore \
             marked three books, and one window of your account did the same to the line of a \
             second window. Your line goes away now when its media leaves that shelf, and the \
             message names that media: no key can then change a media that you did not select, \
             and the keys `j` and `k` give you a line again.",
        ],
    },
    Entry {
        version: "0.7.93",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **A log out reaches every window of that account.** A second window of your \
             account stayed open after the log out: the key `R` gave it a program with no name \
             at all — the header said \"Connected as\" and nothing more — every setting that you \
             changed in it went away with no word, the key `S` said that it kept your library \
             and it kept nothing, and the program went on with the token of the account that \
             logged out. Such a window starts again now: it sends the place of the media that \
             it plays to the server first, and its login screen says which account went away.",
        ],
    },
    Entry {
        version: "0.7.92",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **A media plays when a second window of your account logs out.** The program \
             waited for a listening session of an account that stands on the disk no more: the \
             message \"Syncing your last listening session. Please wait...\" then stayed for \
             ever, the media never played, and every key of the player after it took a part of \
             the program — the program answered no key at all, and the key `Q` did not stop it. \
             A playback of such an account starts at once now, and no wait of a playback stands \
             longer than 30 seconds.",
        ],
    },
    Entry {
        version: "0.7.91",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **The key `X` keeps the media that plays from the disk.** A window of your \
             account played a book of the disk while the server was away, a second window \
             removed the local copy of that book with the key `X`, and the program then had \
             no copy of the book that you listened to: no key gave it back while the server \
             stayed away. The key removes no file of such a media now, and it says why.",
        ],
    },
    Entry {
        version: "0.7.90",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **The program keeps your account when a second window of it removes a different \
             account.** The view of the accounts held the accounts of the moment of its start: \
             the key `c` on a line of an account that the other window removed took the start \
             of the program from every account, and the login screen then came at every start \
             with your account and its token on the disk. The view reads the accounts of the \
             disk now, the two keys say when a different program removed the account of your \
             line, and a program that finds no account of a start gives that work to your \
             first account.",
        ],
    },
    Entry {
        version: "0.7.89",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **The bar of a download stays on the screen when you press D two times.** The \
             second press stopped the bar of the download that runs, and the program then \
             gave you no sign of it for the whole of that download. The program said that a \
             different program of your account downloads that media, and no different \
             program did: it says \"This program downloads …\" now.",
        ],
    },
    Entry {
        version: "0.7.88",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **A second window of your account keeps the ebook that you read.** The cache of \
             the ebooks was full, and the window that got a new book took the book of the \
             other window of the disk while you read it: a book of a scan of 502 megabytes \
             and its pages went away in one key, and that window then needed the server and \
             two minutes to give the book back. Each window says on the disk which book it \
             reads now.",
        ],
    },
    Entry {
        version: "0.7.87",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **A book that you hear offline keeps your place when the program stops by \
             force.** The program sends your place to the server when it starts again. Before \
             this work, an offline playback kept that place at its end only: a terminal that \
             goes away, or a computer that stops, then took the whole playback away — the \
             server kept the place of the day before, and your next playback wrote that old \
             place over the place on your disk.",
        ],
    },
    Entry {
        version: "0.7.86",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **The key `X` of the queue says what happened to the media of your line.** A \
             second window of your account can take that media out of the queue first, and \
             the key then said nothing at all: the line went away from your list all the \
             same, and you could not tell the key that worked from the key that did nothing.",
        ],
    },
    Entry {
        version: "0.7.85",
        date: "14/08/2026",
        body: &[
            "Fixed:",
            "- **The key `X` removes a download that stopped.** A download that does not come \
             to its end holds no line in the list of your offline media, and the key `X` said \
             that the book holds no local copy: some hundred megabytes of that book then \
             stayed on your disk for ever, and no key of this program removed them. The key \
             takes them now, and it says how many megabytes went away.",
            "- **The key `X` takes no file of a download that runs.** The key removed the \
             files under the window that writes them before this work, and that window then \
             said that the download failed. The key says which window downloads the book now \
             — this window, or a different window of your account — and you press it again \
             when that download ends.",
        ],
    },
    Entry {
        version: "0.7.84",
        date: "13/08/2026",
        body: &[
            "Fixed:",
            "- **A download goes to the address of your server that answers.** Every other \
             part of this program takes the address that answers, and the key `D` took the \
             address that you gave at the login: a user away from home sent the download to \
             the address of their house, and the program said nothing at all — no message, \
             no line in the log, and no bar. A download that meets an address that does not \
             answer says so now: 3 seconds for the connection, and 30 seconds with no byte \
             of the answer. A download of a book of some gigabytes has no limit of its whole \
             time.",
        ],
    },
    Entry {
        version: "0.7.83",
        date: "13/08/2026",
        body: &[
            "Fixed:",
            "- **A book that two windows of this program download at one time is the book of \
             the server now.** The two windows wrote the same file at the same time: the file \
             on your disk held more bytes than the file of the server, the decoder found \
             audio that it cannot read at its end, and one screen said that the book is \
             available offline while the other one said that the download failed. One window \
             gets the book now, and the other one says that a different program of your \
             account downloads it.",
        ],
    },
    Entry {
        version: "0.7.82",
        date: "13/08/2026",
        body: &[
            "Fixed:",
            "- **A second window of this program no longer takes the books out of your \
             queue.** Each window held a queue of its own, and every change wrote the whole \
             list: two windows each put one book in the queue, each screen said \"1 item\" \
             with its own book, and the disk kept one of the two. Every window reads the \
             queue of the disk now — before it changes the queue, and when you open the \
             view with the key `q`.",
        ],
    },
    Entry {
        version: "0.7.81",
        date: "13/08/2026",
        body: &[
            "Fixed:",
            "- **A book of your queue no longer goes away when the server stops answering.** \
             The queue took the book out before it played it: a server that went away in the \
             middle of a queue therefore took one book of your list with it, and no key gave \
             it back. A book that did not play stays at the front of your queue now, and the \
             queue waits there for you. A book that this program holds on your disk still \
             plays, and the queue goes on to it.",
        ],
    },
    Entry {
        version: "0.7.80",
        date: "13/08/2026",
        body: &[
            "Fixed:",
            "- **A terminal that goes away no longer takes the place of your book with it.** \
             This program writes the second of your playback to its own disk, and the next \
             program removed that line before it sent it: your place then stayed at the last \
             message to the server, and the minutes after it were gone. The program sends \
             every line that waits now, and it removes a line after the server holds it.",
        ],
    },
    Entry {
        version: "0.7.79",
        date: "13/08/2026",
        body: &[
            "Fixed:",
            "- **A second window of this program no longer removes the ebooks that you keep.** \
             The limit of the cache of the ebooks came of the moment that the window started: \
             a window that you gave four gigabytes in the settings said \"4096 MB now\" on its \
             own screen and it removed your books at the old value. Every window reads \
             `config.toml` again now — when it shows the limit, and before it removes a book.",
            "- **The key `h` goes back from the view of the cache of the ebooks.** The footer \
             of that view named the key, and the key did nothing.",
        ],
    },
    Entry {
        version: "0.7.78",
        date: "13/08/2026",
        body: &[
            "Fixed:",
            "- **A book that came to its end no longer takes back a place that you made \
             later.** This program kept the end of that book, and the key `Q` sent it to the \
             server again — hours later, and over the place that another client of yours \
             wrote in the meantime. The end of a media goes away now when the server holds \
             it, and a server that did not answer keeps it until this program can send it.",
        ],
    },
    Entry {
        version: "0.7.77",
        date: "13/08/2026",
        body: &[
            "Fixed:",
            "- **Two windows of this program with one account keep their two places now.** A \
             media that played in the first window lost every sync when the second window \
             started a media: the second window closed the session of the first one on the \
             server. The key `Q` of one window then sent the place of the book of the other \
             window, and the window that stayed sent **nothing at all** — its book stood at \
             0:00 on the server after two minutes of listening. Each window holds its own \
             session now, and a session that a program left behind still reaches the server \
             at the next start.",
        ],
    },
    Entry {
        version: "0.7.76",
        date: "13/08/2026",
        body: &[
            "Fixed:",
            "- **Your place in a book stays when you change your account.** The key `a` and the \
             key `c` of the view of the accounts start this program again, and the place of a \
             media that played did not reach the server: a book at the minute 13:31 stood at \
             13:23 on the server. This program sends that place first now.",
            "- **The place of one account never goes to the server of another account.** This \
             program held one listening session for every account. The next media of a second \
             account therefore sent the place of the first account to its own server, the server \
             refused it, and **your place went away**. Each account keeps its own session now, \
             and a place that waits reaches its own server when that account plays again.",
        ],
    },
    Entry {
        version: "0.7.75",
        date: "13/08/2026",
        body: &[
            "Fixed:",
            "- **A library that your account may not read no longer stops the program.** An \
             administrator can take a library away from your account while this program holds \
             it. Every view then held no line, the header held no name, and no key gave you the \
             library that you may read — a new start gave the same screen. This program takes a \
             library of your account now, and it says so.",
        ],
    },
    Entry {
        version: "0.7.74",
        date: "13/08/2026",
        body: &[
            "Fixed:",
            "- **Your timer for sleep stays after the key `R`.** The key `R`, the key that takes \
             the next library, and a change of the sequence of the library made a new screen, and \
             your timer stayed with the screen that went away: the media that you set to stop \
             played on, and the row of the player held no timer.",
        ],
    },
    Entry {
        version: "0.7.73",
        date: "13/08/2026",
        body: &[
            "Fixed:",
            "- **The cursor of the login screen stands in the field that you write in.** A \
             message of the login moved the cursor of your terminal to the end of its own row, \
             six rows below the field: your letters went to the field, and the cursor blinked far \
             from them. The message of the login also goes away by itself now.",
        ],
    },
    Entry {
        version: "0.7.72",
        date: "13/08/2026",
        body: &[
            "Fixed:",
            "- **A program that you build yourself keeps your account.** The login took your \
             address, your name, and your password, and then the screen stayed empty for ever: \
             the start after it asked for all three again. This program needs a secret key for \
             your token, only `install.sh` made that key, and a build with `cargo`, with `nix`, \
             or with a package of your system gave you none. This program makes the key itself \
             now, at the first start.",
        ],
    },
    Entry {
        version: "0.7.71",
        date: "12/08/2026",
        body: &[
            "Fixed:",
            "- **The keys of the player keep your playback after the key `R`.** The key `R`, the \
             key that takes the next library, and a change of the sequence of the library made a \
             new engine of the sound: the row of the player went away while your media played, \
             and the keys `Space` and `Y` acted on nothing. The book played to its end and no key \
             reached it. The engine of your playback stays now.",
        ],
    },
    Entry {
        version: "0.7.70",
        date: "12/08/2026",
        body: &[
            "Fixed:",
            "- **A book that plays keeps its percentage on the screen.** This program sent the \
             position of a playback to the server as a text, and the server keeps what it takes: \
             the answer of the server then held a value that this program could not read, and the \
             line of that book showed no percentage while it played. The program sends a number \
             now, and it reads a position that came as a text: the books of your server that hold \
             such a value show their place again.",
        ],
    },
    Entry {
        version: "0.7.69",
        date: "12/08/2026",
        body: &[
            "Fixed:",
            "- **The first screen comes faster with a server that answers slowly.** The program \
             asked the server for the series, the collections, the playlists, and the items of \
             the library after the answer of the shelves of the Home view, and those requests \
             need that answer for nothing. They go beside it now: a start of 2.03 seconds with \
             a server of 500 milliseconds takes 1.56 seconds.",
        ],
    },
    Entry {
        version: "0.7.68",
        date: "12/08/2026",
        body: &[
            "Fixed:",
            "- **A key of yours works at the moment that the server answers again.** After a \
             server that went away for some seconds, the program said \"No server address \
             answered\" for up to 60 seconds while the server answered: it kept the answer of \
             the attempt before, and it waited for its own examination of the address. Every \
             request tries the address now, and the reason that you read is the answer of that \
             attempt.",
            "- **The log says the moment that the program stops to use an address of your \
             server**, and it says why. The log held no line of it.",
        ],
    },
    Entry {
        version: "0.7.67",
        date: "12/08/2026",
        body: &[
            "Fixed:",
            "- **The start is much faster with a server that answers slowly.** The program asked \
             the server for the position of each media of the Home view, one request for each of \
             them. One answer holds every position now: a start of 3.8 seconds with a server of \
             500 milliseconds takes 1.7 seconds.",
            "- **A book that you read keeps its position on the screen.** The place of the \
             reader inside a book is a fraction, and this program read a whole number: the \
             answer of the server for such a book did not read at all, and the line said \
             \"N/A\" for a book of 92 percent.",
        ],
    },
    Entry {
        version: "0.7.66",
        date: "12/08/2026",
        body: &[
            "Fixed:",
            "- **A library of podcasts of more than 500 podcasts works now.** The key that \
             opens a podcast of a page after the first stopped the program, and that podcast \
             showed no episode. The program reads the episodes of a podcast when you open it.",
            "- **The start of a library of podcasts is fast.** The program read the episodes of \
             every podcast of the library before its first screen: a library of 520 podcasts \
             took 11.9 seconds with a slow server, and it takes 0.4 seconds now.",
            "- **The search of a podcast that this program did not read gives its line.** The \
             screen said \"The server found nothing\" for a podcast that the server found. The \
             program reads the pages of the library now, and the line comes.",
        ],
    },
    Entry {
        version: "0.7.65",
        date: "12/08/2026",
        body: &[
            "Added:",
            "- **This program holds more than one account now.** The view \"Accounts and log \
             out\" of the settings shows every account, and the mark ▶ is on the account that \
             the program starts with. The key `a` adds an account: the program starts again, and \
             it asks you for a server, a name, and a password. The key `c` gives the start to \
             the account of the line. A log out of the account that starts gives the start to \
             the account that stays, and a log out of the one account brings the login screen. \
             The program starts again for each of those keys, therefore a playback stops.",
        ],
    },
    Entry {
        version: "0.7.64",
        date: "12/08/2026",
        body: &[
            "Fixed:",
            "- **The program makes `config.toml` for you.** A start with no configuration file \
             stopped the program, and it said a line of its own source. The program writes the \
             example file now, with every comment of it, and a file that exists stays as it \
             stands. A key of a color that your file does not hold takes the value of the \
             program, therefore a file of an older version starts too.",
            "- **A token that your server refuses now opens the login screen.** The program \
             stopped with \"The token is not valid. Log in again.\", and it gave you no way to \
             log in. The login screen comes now, it says the reason, and it holds the address \
             of your server already: your name and your password give a new token. The key `R` \
             does the same when the server takes the token away while the program runs.",
        ],
    },
    Entry {
        version: "0.7.63",
        date: "12/08/2026",
        body: &[
            "Fixed:",
            "- **A book that holds one file that this program cannot read now plays from your \
             place.** A book of two files, where the second file is a form that no decoder of \
             this program reads, started again from the beginning: the program left a playback \
             that played and it asked the server for a stream. The book now plays its own \
             files from your place, and it ends at the file that it cannot read.",
            "- **An account without the permission to download now plays its books.** The \
             engine asked the server for a download of each file, and the server refuses that \
             for such an account: no book played from its file, and every media went to a \
             stream of the server. The engine asks for the file of the track now.",
        ],
    },
    Entry {
        version: "0.7.62",
        date: "12/08/2026",
        body: &[
            "Added:",
            "- **The key @ sends the ebook of the line to an e-reader.** The server holds the \
             devices, and this program shows the devices that your account may use: press @ on \
             a book, and l on the device. The server then sends the book of that item to the \
             address of e-mail of the device.",
            "- A screen that says why it holds no device. A server with no device says that an \
             administrator of the server adds one, and a server with no settings of the e-mail \
             says that the administrator gives those settings.",
            "Fixed:",
            "- **A big book now reaches the e-reader.** The send of a book of 480 megabytes \
             takes about 36 seconds of the server, and every request of this program stopped \
             after 15 seconds: the user then read a fault of a send that succeeded. The send \
             holds a time limit of fifteen minutes of its own.",
        ],
    },
    Entry {
        version: "0.7.61",
        date: "12/08/2026",
        body: &[
            "Fixed:",
            "- **The screen of the accounts says what this program does.** It said that the \
             program holds more than one account and that it starts with the account that is \
             the default one, and no key does that work. It now says how a second account \
             works today: give the variable XDG_CONFIG_HOME a directory of its own.",
        ],
    },
    Entry {
        version: "0.7.60",
        date: "12/08/2026",
        body: &[
            "Fixed:",
            "- **A book of a scan no longer takes the memory of this program.** The program \
             held the whole file while it wrote it to the disk: a book of 502 megabytes gave \
             the program a peak of 1007 megabytes, and it gives 44 now. Every part of the \
             answer goes to the disk.",
            "- **The reader names your book when you open it from the search.** It said the \
             identity of the media for a PDF, because a PDF holds no title of its own.",
        ],
    },
    Entry {
        version: "0.7.59",
        date: "12/08/2026",
        body: &[
            "Fixed:",
            "- **A box that asks you for a text follows the size of your terminal.** The box of \
             the search and the box of a name stood at the place of the screen that came \
             before: a terminal that became smaller while the box stood gave you an empty \
             screen, and you could not see the letters that you wrote.",
            "- **The header of a narrow terminal keeps every value.** The name of your account \
             and the name of your library wrote on each other below 68 columns. The header \
             holds the short form there: the account, the library, the address, and the \
             version all stay.",
        ],
    },
    Entry {
        version: "0.7.58",
        date: "12/08/2026",
        body: &[
            "Fixed:",
            "- **The search shows every media that the server finds.** This program reads your \
             library page by page, and the view of the search showed the media of the pages \
             that it holds: a search for a book of a later page said \"The server found \
             nothing\", and the server had found that book.",
            "- **The search works in a library of podcasts.** The server answers with a group \
             of its own for such a library, and this program read no line of it: every search \
             said that the server found nothing.",
        ],
    },
    Entry {
        version: "0.7.57",
        date: "12/08/2026",
        body: &[
            "Fixed:",
            "- **A media that holds no author, no year, or no description says so.** The \
             server gives an empty text for a value that a media does not hold, and the \
             screen wrote that empty text: the line of a book said \"Author:  - Year: N/A\". \
             Every view says \"N/A\" now.",
        ],
    },
    Entry {
        version: "0.7.56",
        date: "12/08/2026",
        body: &[
            "Fixed:",
            "- **The key G goes to the last media of your library.** This program reads your \
             library page by page, and that key went to the end of the page: a user of a \
             library of 2056 media had to press it six times. One press is enough now, and \
             the key g gives the first line back.",
        ],
    },
    Entry {
        version: "0.7.55",
        date: "12/08/2026",
        body: &[
            "Fixed:",
            "- A PDF book opens in a process of its own for the program only. A test and every \
             other program that takes this code read the book in their own process, and the \
             reader said \"This PDF gives no page\" for a book that it reads.",
        ],
    },
    Entry {
        version: "0.7.54",
        date: "12/08/2026",
        body: &[
            "Added:",
            "- **The settings say what your account may do.** Press S and take \"Accounts and \
             log out\": the screen names the type of your account and every permission that \
             changes the work of this program.",
            "",
            "Changed:",
            "- **The server groups the books of a series now.** The screen holds the same \
             lines, and the program asks the server for fewer items: the title of the Library \
             view says how many lines the library holds.",
        ],
    },
    Entry {
        version: "0.7.53",
        date: "12/08/2026",
        body: &[
            "Added:",
            "- **Shift+Tab takes the next library of the server.** The Home view shows the \
             shelves of one library, and this key goes round every library of your account. \
             Tab keeps the Home view and the Library view.",
            "",
            "Changed:",
            "- **The program starts with one page of the library.** It read every page before, \
             therefore a large library made you wait at each start. It asks the server for the \
             next page when you come near the end of the list, and the title of the view says \
             how many items the library holds.",
            "- **A book of the form PDF opens in a process of its own.** A large book of a scan \
             took the memory of the program that you read, and a book that no reader can read \
             stopped it. That memory and that fault stay outside now, and a PDF that gives no \
             page gives you one sentence and the key h.",
            "- A second visit of a PDF opens at once: the program keeps the pages of that book \
             beside the file, and the cache of the ebooks holds them.",
        ],
    },
    Entry {
        version: "0.7.52",
        date: "12/08/2026",
        body: &[
            "Fixed:",
            "- **Every text of the program that counts one thing says it in the singular.** \
             The view of the sessions said \"1 sessions of 1\", a series of one book said \
             \"1 book(s)\", the answer of one podcast said \"1 answers\", and the reader said \
             \"1 files\" and \"removed 1 book(s)\". A test now reads every file of the views \
             and of the work, therefore a new text of that kind cannot come back.",
        ],
    },
    Entry {
        version: "0.7.51",
        date: "12/08/2026",
        body: &[
            "Fixed:",
            "- **The header names the address of the server that answers now.** A server can \
             have more than one address in config.toml, and the program moved between them \
             while the header kept the address that you gave at the login.",
            "- **The header says when no address of the server answers.** The screen said \
             \"Connected\" until you pressed R, and the program knew already. It says \"the \
             server does not answer\" now, and it names the key R for the media of the disk.",
            "- The statistics of a library of one media said \"1 items\". Every number of that \
             screen names one thing in the singular now.",
        ],
    },
    Entry {
        version: "0.7.50",
        date: "12/08/2026",
        body: &[
            "Changed:",
            "- **A small terminal shows more lines.** The 6 rows of the player stood empty \
             while nothing played, and every view holds them now: a terminal of 18 rows gives \
             the Home view 10 lines, and it gave 4. The lines move down when a playback \
             starts, and they move back when it stops.",
        ],
    },
    Entry {
        version: "0.7.49",
        date: "12/08/2026",
        body: &[
            "Fixed:",
            "- **The Home view and the Library view of a library with no media said nothing.** \
             The screen held an empty list, and no line said why. The two views name the \
             reason now: a server that does not answer, a filter that hides every media, or a \
             library that holds none.",
        ],
    },
    Entry {
        version: "0.7.48",
        date: "12/08/2026",
        body: &[
            "Added:",
            "- **The keys < and > move a media inside a collection or a playlist.** Open the \
             list with c and then l, and move the line of the media. The sequence goes to the \
             server, therefore every other client shows it too.",
            "- The footer of that view names its keys: the sequence, and the key X that takes \
             the media out of the list.",
        ],
    },
    Entry {
        version: "0.7.47",
        date: "12/08/2026",
        body: &[
            "Added:",
            "- **This screen holds every release of this fork.** It stopped at v0.6.8 while \
             the program was at v0.7.46, therefore 38 releases reached no user. The first line \
             took the version of your build, and that hid the fault.",
            "",
            "Changed:",
            "- A line of this screen fills the width of your terminal. It held the wrap of \
             the source before, therefore a wide terminal showed a narrow column.",
        ],
    },
    Entry {
        version: "0.7.46",
        date: "12/08/2026",
        body: &[
            "Added:",
            "- The key D gives a collection or a playlist a new description.",
        ],
    },
    Entry {
        version: "0.7.45",
        date: "12/08/2026",
        body: &[
            "Fixed:",
            "- A terminal of 18 rows showed one line of the list. Every row of a small screen \
             goes to the list now.",
        ],
    },
    Entry {
        version: "0.7.44",
        date: "11/08/2026",
        body: &[
            "Fixed:",
            "- **One request that stopped at its time limit took the server away.** Every \
             request after it said \"No server address answered\" for up to 60 seconds.",
        ],
    },
    Entry {
        version: "0.7.43",
        date: "11/08/2026",
        body: &["Fixed:", "- The view of the search said \"1 items\"."],
    },
    Entry {
        version: "0.7.42",
        date: "11/08/2026",
        body: &[
            "Fixed:",
            "- The row of an item lost its end in a terminal of 80 columns.",
        ],
    },
    Entry {
        version: "0.7.41",
        date: "11/08/2026",
        body: &[
            "Added:",
            "- The key r gives a collection or a playlist a new name, and the key X removes \
             it. The program asks one time first, and the question names the kind: every user \
             of the server sees a collection.",
        ],
    },
    Entry {
        version: "0.7.40",
        date: "11/08/2026",
        body: &[
            "Fixed:",
            "- **The login says why the server refused it.** Every fault gave \"ERROR: Login \
             failed\" before. The rate limit of the login has its own message, with the time \
             to wait.",
        ],
    },
    Entry {
        version: "0.7.39",
        date: "11/08/2026",
        body: &[
            "Fixed:",
            "- **Every footer lost its end in a terminal of 80 columns.** The keys ? and Q \
             stand at the end of a footer, therefore no user of a narrow terminal read them.",
            "- A view says that the server does not answer, and it no longer says that your \
             library holds nothing.",
        ],
    },
    Entry {
        version: "0.7.38",
        date: "11/08/2026",
        body: &[
            "Added:",
            "- The keys c and p make a new collection or a new playlist of the media that you \
             selected.",
            "",
            "Fixed:",
            "- A box that asks for a text left two columns of the view on the screen.",
        ],
    },
    Entry {
        version: "0.7.37",
        date: "11/08/2026",
        body: &[
            "Added:",
            "- The key m puts a media in a collection or in a playlist, and the key X of that \
             view takes it out again.",
            "",
            "Fixed:",
            "- The key s of a library of podcasts says why it does nothing: a podcast holds no \
             series.",
        ],
    },
    Entry {
        version: "0.7.36",
        date: "11/08/2026",
        body: &[
            "Added:",
            "- The key d shows the episodes that the server downloads, and the queue of that \
             work. The key X empties the queue of one podcast.",
            "",
            "Changed:",
            "- The keys of the volume say what they did. The row of the player names the \
             volume.",
        ],
    },
    Entry {
        version: "0.7.35",
        date: "11/08/2026",
        body: &[
            "Added:",
            "- **The settings write config.toml**, and they keep every comment of that file.",
            "",
            "Fixed:",
            "- A message of the program no longer writes on the letters of the view below it.",
            "- The key h of the view of the search goes back.",
        ],
    },
    Entry {
        version: "0.7.34",
        date: "11/08/2026",
        body: &[
            "Added:",
            "- The key e inside the reader gives every book of a media that holds more than \
             one.",
            "",
            "Fixed:",
            "- Two texts of the screen: the footer of the narrators said \"author\", and a \
             text of the settings held a run of spaces.",
            "- The run of the tests takes 2.2 seconds, and it took 18.7.",
        ],
    },
    Entry {
        version: "0.7.33",
        date: "11/08/2026",
        body: &["Added:", "- The key v shows the narrators of the library."],
    },
    Entry {
        version: "0.7.32",
        date: "11/08/2026",
        body: &[
            "Added:",
            "- config.toml holds the limit of the cache of the ebooks, in a block [reader]. \
             The settings write that value.",
        ],
    },
    Entry {
        version: "0.7.31",
        date: "11/08/2026",
        body: &[
            "Added:",
            "- The program says which book the cache of the ebooks removed.",
        ],
    },
    Entry {
        version: "0.7.30",
        date: "11/08/2026",
        body: &[
            "Added:",
            "- A search of the name of an author gives the books of that author.",
        ],
    },
    Entry {
        version: "0.7.29",
        date: "11/08/2026",
        body: &[
            "Fixed:",
            "- **A book of xHE-AAC plays.** The program starts the stream at a place beside \
             the one that stops the ffmpeg of the server.",
        ],
    },
    Entry {
        version: "0.7.28",
        date: "11/08/2026",
        body: &[
            "Fixed:",
            "- The message of a stream that the server cannot make says the truth. An answer \
             of 404 of one part of a stream is not a media that the server does not hold.",
        ],
    },
    Entry {
        version: "0.7.27",
        date: "11/08/2026",
        body: &[
            "Added:",
            "- **A media that a different client finished leaves the list Continue \
             Listening**, with no key and with no request.",
            "- The program keeps the ebooks on the disk, with a limit of one gigabyte. The \
             book that you read now never goes away.",
        ],
    },
    Entry {
        version: "0.7.26",
        date: "11/08/2026",
        body: &[
            "Added:",
            "- The key X removes the ebook of the reader too.",
            "",
            "Changed:",
            "- The build of the development holds the lines of the debug only. It is faster, \
             and it writes less on the disk.",
        ],
    },
    Entry {
        version: "0.7.24",
        date: "11/08/2026",
        body: &[
            "Fixed:",
            "- The position and the movement of a playback that comes from a stream of the \
             server.",
        ],
    },
    Entry {
        version: "0.7.23",
        date: "11/08/2026",
        body: &[
            "Fixed:",
            "- The task of the live messages waits longer after each fault, and a scan of the \
             library holds less memory.",
        ],
    },
    Entry {
        version: "0.7.22",
        date: "11/08/2026",
        body: &[
            "Added:",
            "- The filter of a library holds the tags of the server.",
        ],
    },
    Entry {
        version: "0.7.21",
        date: "11/08/2026",
        body: &[
            "Fixed:",
            "- Every message of the program stands inside the frame of the screen.",
        ],
    },
    Entry {
        version: "0.7.20",
        date: "11/08/2026",
        body: &[
            "Fixed:",
            "- The view of the chapters says why it holds no line.",
        ],
    },
    Entry {
        version: "0.7.19",
        date: "11/08/2026",
        body: &[
            "Changed:",
            "- The reader says \"page\" for a PDF, and \"chapter\" for an EPUB.",
            "",
            "Fixed:",
            "- The key ? works inside the reader.",
        ],
    },
    Entry {
        version: "0.7.18",
        date: "11/08/2026",
        body: &[
            "Fixed:",
            "- A picture of 16 bits of a PDF reaches the screen.",
        ],
    },
    Entry {
        version: "0.7.17",
        date: "11/08/2026",
        body: &[
            "Added:",
            "- The queue of the media stands on the disk, therefore it survives a stop of the \
             program.",
            "",
            "Fixed:",
            "- A book that ends before its length keeps the position of that end. It went back \
             to the start before.",
        ],
    },
    Entry {
        version: "0.7.16",
        date: "11/08/2026",
        body: &[
            "Fixed:",
            "- The loop of a playback read the fault of the playback that came before it, and \
             it stopped the new playback.",
        ],
    },
    Entry {
        version: "0.7.15",
        date: "11/08/2026",
        body: &[
            "Added:",
            "- **The reader shows a PDF book**, with its pictures.",
        ],
    },
    Entry {
        version: "0.7.14",
        date: "11/08/2026",
        body: &[
            "Added:",
            "- **Every codec of the server plays.** The program asks the server for a stream \
             when it cannot read the file itself.",
            "",
            "Fixed:",
            "- A fault of the reader left the user in that view. The screen of a fault names \
             the keys now.",
        ],
    },
    Entry {
        version: "0.7.13",
        date: "11/08/2026",
        body: &[
            "Added:",
            "- **The program reads the live messages of the server.** A change that a \
             different client makes therefore reaches the screen with no key.",
            "- The key ? shows every key of the program, in groups.",
            "- The cover art fills its panel.",
            "",
            "Fixed:",
            "- A book of more than one file plays. It needed a player of the system before.",
        ],
    },
    Entry {
        version: "0.7.12",
        date: "11/08/2026",
        body: &[
            "Fixed:",
            "- The place of an ebook agrees with the web reader of the server. A place that \
             this program wrote gave a different chapter there.",
        ],
    },
    Entry {
        version: "0.7.11",
        date: "11/08/2026",
        body: &[
            "Added:",
            "- **A queue of media.** The key n puts a media at the end of the queue, and the \
             key q shows the queue. The next media starts when the one before it ends.",
        ],
    },
    Entry {
        version: "0.7.10",
        date: "11/08/2026",
        body: &[
            "Added:",
            "- The key W shows every session that you played, with pages.",
        ],
    },
    Entry {
        version: "0.7.9",
        date: "11/08/2026",
        body: &[
            "Added:",
            "- The view of the key T holds the statistics of the library and of the year too.",
        ],
    },
    Entry {
        version: "0.7.8",
        date: "11/08/2026",
        body: &[
            "Added:",
            "- The reader writes the place of a book as an EPUBCFI, and it reads that form \
             too. The web reader of the server and this program therefore give the same \
             chapter.",
        ],
    },
    Entry {
        version: "0.7.7",
        date: "11/08/2026",
        body: &[
            "Added:",
            "- The key L tells the server to examine the library for new files.",
        ],
    },
    Entry {
        version: "0.7.6",
        date: "11/08/2026",
        body: &["Added:", "- The key a shows the authors of the library."],
    },
    Entry {
        version: "0.7.5",
        date: "11/08/2026",
        body: &[
            "Added:",
            "- The key E tells the server to get the new episodes of a feed.",
        ],
    },
    Entry {
        version: "0.7.4",
        date: "11/08/2026",
        body: &[
            "Added:",
            "- The key A adds a podcast to a library, with the address of its feed.",
        ],
    },
    Entry {
        version: "0.7.3",
        date: "11/08/2026",
        body: &[
            "Added:",
            "- The key t gives a timer for sleep: 5, 10, 15, 30, 45, or 60 minutes, or the end \
             of the chapter. The playback stops at that time.",
        ],
    },
    Entry {
        version: "0.7.2",
        date: "11/08/2026",
        body: &[
            "Added:",
            "- The key b writes a bookmark at the place of the playback, and the key V shows \
             the bookmarks of a media.",
        ],
    },
    Entry {
        version: "0.7.1",
        date: "11/08/2026",
        body: &[
            "Added:",
            "- The key N hides a media from the list Continue Listening.",
            "- The key C shows the chapters of the media that plays.",
        ],
    },
    Entry {
        version: "0.7.0",
        date: "11/08/2026",
        body: &[
            "Added:",
            "- **The Home view shows every shelf of the server**, and not the list Continue \
             Listening only.",
            "- The key T shows the time that you listened.",
            "- The key f gives the sequence and the filter of a library.",
        ],
    },
    Entry {
        version: "0.6.9",
        date: "11/08/2026",
        body: &[
            "Changed:",
            "- The program reads the permissions of your account. The key D on an account that \
             may not download now says so, and it no longer shows the error of the protocol of \
             the server.",
            "- The bar of the search starts with no text.",
        ],
    },
    Entry {
        version: "0.6.8",
        date: "11/08/2026",
        body: &[
            "Added:",
            "- **The key M marks a media as finished, or as not finished.** The program sent \
             that mark at the end of a playback only, therefore a user who left a book in the \
             middle could not take it out of the list Continue Listening. The key asks the \
             server for the condition first, and it sends the opposite.",
            "- A media that goes to \"not finished\" loses its position: the server puts it \
             back to the start. The message says so.",
        ],
    },
    Entry {
        version: "0.6.7",
        date: "11/08/2026",
        body: &[
            "Added:",
            "- **The search asks the server.** The program looked in the titles that it holds, \
             therefore the name of an author found nothing. The server also finds an author, a \
             series, a narrator, a tag, and a genre. The screen shows the titles at once, and \
             the answer of the server when it comes. The title of the list says where the \
             answer comes from.",
            "- The reader follows the web reader of Audiobookshelf. It reads the chapter out \
             of an EPUBCFI, therefore you find the correct chapter when you read in the web \
             page and then in the terminal.",
            "- A log out asks one time. Press l a second time to log out, and any other key \
             stops the question.",
            "- docs/T-24-coverage.md compares this program with the server, function by \
             function.",
            "",
            "Fixed:",
            "- The Home view matched its shelf on a name for a person. A server that gives \
             that name in a different language gave an empty Home view, with no error. The \
             program matches the identity now.",
            "- The list of the accounts no longer moves past its last line.",
        ],
    },
    Entry {
        version: "0.6.5",
        date: "11/08/2026",
        body: &[
            "Fixed:",
            "- **A playback that does not start no longer loses your place.** rodio gives the \
             position 0 until the seek finishes, and a playback that never starts gives 0 for \
             the whole wait. The program wrote that 0 on the disk every second, and it gave \
             that 0 to the server when the session closed. The book then started at the \
             beginning. The program now writes nothing until the engine reaches the place \
             where the playback starts.",
        ],
    },
    Entry {
        version: "0.6.4",
        date: "11/08/2026",
        body: &[
            "Added:",
            "- The reader opens a book where you stopped. The place comes from the server, \
             therefore a different machine gives the same place.",
            "- The reader sends the place by itself: when the place changed and 30 seconds \
             went by, and when you leave the book with h.",
        ],
    },
    Entry {
        version: "0.6.3",
        date: "11/08/2026",
        body: &[
            "Added:",
            "- **Read an EPUB book in the terminal.** The key e on an item that holds an ebook \
             opens the reader. The keys: j/k a line, Space/b a page, n/p a chapter, t the \
             table of contents, g/G the start and the end, s sends the place to the server, \
             and h leaves the book.",
            "- The program keeps the file of the book, therefore the reader also works with no \
             server.",
            "- The place of the reader goes to the field of the ebook of the server. It \
             changes no position of the audio.",
        ],
    },
    Entry {
        version: "0.6.2",
        date: "10/08/2026",
        body: &[
            "Fixed:",
            "- The start cannot wait for ever now. The sound device gets five seconds, and the \
             program then goes on with no sound. An answer of the server that the program \
             cannot read no longer stops the program.",
            "- The start is faster with a large library. The pages of the items go to the \
             server together. A library of 2056 items needs five pages, and the program asked \
             for them one after the other before.",
            "",
            "Added:",
            "- The reader of EPUB reads a book, and no screen shows it yet. The part that \
             reads the file refuses a book that is too large, a chapter that is too large, and \
             every one of twelve files that attack the program.",
            "- macOS has a way to remove the program with no binary: macos/uninstall.sh. It \
             deletes nothing, and it writes the paths and the commands.",
        ],
    },
    Entry {
        version: "0.6.1",
        date: "10/08/2026",
        body: &[
            "This release answers a report of a user on v0.5.0 and v0.6.0.",
            "",
            "Fixed:",
            "- The program drew nothing while it started. A slow server gave a black screen, \
             and the user could not tell a slow server from a program that stopped. The \
             program now draws at once, it names the step that it waits for, and the key Q \
             stops it.",
            "- The start is faster: the position of each book of the list Continue Listening \
             goes in one group of requests, and not one after the other.",
            "- A machine with no sound device could not open the program at all. Every \
             function that needs no sound works now.",
            "- No index of a vector can stop the screen. The render read a vector with the \
             number of the selected line in 39 places, and a list of the screen can be shorter \
             than that number.",
            "- The login examines the address of the server before it asks for the password, \
             and it says what is wrong.",
            "",
            "Added:",
            "- Every line of the Home view and of the Library view has a mark: the media that \
             plays, a media that is finished, or the part that the user heard.",
            "- The settings say \"Accounts and log out\", and each entry tells what it does.",
            "- TOUTUI_NO_COVERS turns the cover art off. Inside tmux the program asks the \
             terminal nothing.",
        ],
    },
    Entry {
        version: "0.6.0",
        date: "10/08/2026",
        body: &[
            "Added:",
            "- The cover art. The cover stands beside the description and it is always \
             visible. A series shows the cover of each of its books. The cover of the media \
             that plays is the largest one. A narrow terminal gives the whole width to the \
             text.",
            "- A series takes one line of the Library view. The key l opens its books, in the \
             sequence of the series.",
            "- The key F sends the position to the server at once. It does not close the \
             listening session.",
            "",
            "Changed:",
            "- The program uses ratatui 0.30 and crossterm 0.29. tui-textarea is gone, and \
             tui-input takes its place. The login screen and the bar of the search work as \
             before.",
            "- The build still needs no C toolchain for a library of the system, and it needs \
             no OpenSSL.",
        ],
    },
    Entry {
        version: "0.5.0",
        date: "10/08/2026",
        body: &[
            "Added:",
            "- The program plays a local copy when the server does not answer, and it sends \
             the positions when the server answers again.",
            "- The program updates itself with `toutui --update`. The program compares the sum \
             of the archive before it moves the new binary.",
            "- The releases come from this repository, and the archives have a sum SHA-256 \
             that the machine writes.",
            "",
            "Fixed:",
            "- `--update` installed the archived original project.",
            "- `Mark as finished` did not always operate.",
            "",
            "Changed:",
            "- The script of installation has 100 lines and not 1080. It installs no VLC and \
             no netcat, because the player in the program needs neither.",
        ],
    },
];

/// The text of one entry, with the credit and the words that close it.
fn the_text_of_an_entry(entry: &Entry) -> String {
    let mut text = String::new();

    text.push_str("Changelog Toutui v");
    text.push_str(entry.version);
    text.push_str(" (");
    text.push_str(entry.date);
    text.push_str(")\n\n");

    for line in entry.body {
        text.push_str(line);
        text.push('\n');
    }

    text.push_str(
        "\nContributors:\n\
         \n\
         - AlbanDAVID (the original project), ealtun21\n\
         \n\
         Enjoy and be toutui!\n\
         ####\n",
    );

    text
}

pub fn changelog() -> String {
    let mut changelog = String::new();

    // The screen of settings shows this text. The credit is at the top,
    // because AlbanDAVID wrote the original program.
    changelog.push_str(
        "AlbanDAVID wrote Toutui and archived it. This repository continues\n\
         that work. https://github.com/AlbanDAVID/Toutui\n\
         \n\
         The entries of this fork come first, and the entries of the original\n\
         project come after them. Some of those name a script or a package\n\
         that does not exist now. The README of this repository gives the ways\n\
         to install, to update, and to remove the fork.\n\
         \n\
         ####\n",
    );

    for entry in THE_ENTRIES_OF_THE_FORK {
        changelog.push_str(&the_text_of_an_entry(entry));
    }

    let changelog_01 = "Changelog Toutui v0.1.0-beta (02/21/2025) \n\
         Fixed:\n\
         \n\
         First release.
         \n\
         Changed:\n\
         \n\
         First release.
         \n\
         Enjoy!\n
         ####\n"
        .to_string();
    let changelog_02 = "Changelog Toutui v0.1.1-beta (02/24/2025) \n\
         Fixed:\n\
         \n\
         - App crash (out of bounds) when API send empty values.
         - Close listening session not always working (bug_id: fixed_dd9a64)
         \n\
         Changed:\n\
         \n\
         No change.
         \n\
         Enjoy and be toutui!\n
         ####\n"
        .to_string();
    let changelog_03 = "Changelog Toutui v0.1.2-beta (02/24/2025) \n\
         Fixed:\n\
         \n\
         - Partially fixed, becsause not optimal: bug_id: 9bacac Sync: If you open VLC to listen X, close VLC and quickly open VLC again to listen Y: X will still be sync — according to Y (normally, only Y has to be sync in this case).

         \n\
         Changed:\n\
         \n\
         No change.
         \n\
         Enjoy and be toutui!\n
         ####\n".to_string();
    let changelog_04 = "Changelog Toutui v0.1.3-beta (02/03/2025) \n\
         Fixed:\n\
         \n\
         - Fix bug_id: 3f729c Loading time not optimized for library with a lot of items (long start loading and refresh time)
         \n\
         Changed:\n\
         \n\
         - Script `hello_toutui` to make installation easier.
         \n\
         Contributors:\n\
         \n\
         - dougy147, dhonus
         \n\
         Enjoy and be toutui!\n
         ####\n".to_string();
    let changelog_05 = "Changelog Toutui v0.2.0-beta (07/03/2025) \n\
CAUTION: This version is not compatible with the previous one.  
You need to remove the database in ~/.config/toutui before proceeding. 
         Fixed:\n\
         \n\
         - From known_bugs.md, fixed:

    Find a robust solution for bug_id: 9bacac
    Fix bug_id: 86384e
    Fix bug_id: 6ac5d8
    Fix bug_id: 06e548
    Fix bug_id: e0b61c
    Fix bug_id: fc695f
    Fix bug_id: 40f48d
    Fix bug_id: bf10cd

         \n\
         Changed:\n\
         \n\
         - 
         \n\
         Contributors:\n\
         \n\
         - AlbanDAVID
         \n\
         Enjoy and be toutui!\n
         ####\n"
        .to_string();
    let changelog_06 = "Changelog Toutui v0.3.0-beta (24/03/2025) \n\
CAUTION: This version is not compatible with the previous one.  
To make it work properly, perform a fresh reinstall.
\n\
         Added:\n\
         - Integrated player. Keep calm and stay in your terminal! :)
         \n\
         Fixed:\n\
         \n\
         - Fixed: issue where pressing R twice was required to refresh the app.
         - Fixed: issue causing the cursor to disappear when the application is closed.
         - Fixed: issue if app is quitted for the first time and that listening session is empty.
         \n\
         Changed:\n\
         \n\
         - Faster loading time to play an item.
         - Improved synchronization accurary.
         - Removed warning during compilation time.
         \n\
         Contributors:\n\
         \n\
         - AlbanDAVID, dougy147
         \n\
         Enjoy and be toutui!\n
         ####\n"
        .to_string();
    let changelog_07 = "Changelog Toutui v0.3.1-beta (25/03/2025) \n\
CAUTION: This version is not compatible with v0.2.0-beta and bellow.  
To make it work properly, perform a fresh reinstall.
\n\
         Fixed:\n\
         \n\
         - Fixed: incorrect merge
         \n\
         Contributors:\n\
         \n\
         - AlbanDAVID
         \n\
         Enjoy and be toutui!\n
         ####\n"
        .to_string();
    let changelog_08 = "Changelog Toutui v0.3.2-beta (26/03/2025) \n\
         Added:\n\
         \n\
         - macOS compatibility.
         \n\
         Fixed:\n\
         \n\
         - Issue with VLC buffer (if a chapter is manually changed or during jump/backward).
         - Display issue on small monitors.
         \n\
         Changed:\n\
         \n\
         - hello_toutui script improved
         \n\
         Contributors:\n\
         \n\
         - AlbanDAVID, dougy147
         \n\
         Enjoy and be toutui!\n
         ####\n"
        .to_string();
    let changelog_09 = "Changelog Toutui v0.3.3-beta (02/04/2025) \n\
         \n\
         Changed:\n\
         \n\
         - Adding a login placeholder to specify the use of http:// or https:// for the server address.
         - Display error login message without time limit.
         \n\
         Contributors:\n\
         \n\
         - AlbanDAVID
         \n\
         Enjoy and be toutui!\n
         ####\n".to_string();
    let changelog_10 = "Changelog Toutui v0.3.4-beta (23/04/2025) \n\
         \n\
         Fix:\n\
         \n\
         Handle empty podcast episode lists gracefully. Prevent panic and show 'No episodes' message. by @denispol in https://github.com/AlbanDAVID/Toutui/pull/22\n\
         Contributors:\n\
         \n\
         - AlbanDAVID, denispol
         \n\
         Enjoy and be toutui!\n
         ####\n".to_string();
    let changelog_11 = "Changelog Toutui v0.3.5-beta (27/04/2025) \n\
         \n\
         Added:\n\
         - Display number of total items for continue listening, library and library settings (for books and podcasts)
         - Clap crate and a function to display the version in the CLI (e.g. `toutui --version`)
         \n\
         Fixed:\n\
         \n\
         - [macos] vlc version not displayed in listening sessions (from ABS web browser)
         - Out of bounds in Library Settings
         \n\
         Contributors:\n\
         \n\
         - AlbanDAVID
         \n\
         Enjoy and be toutui!\n
         ####\n".to_string();
    let changelog_12 = "Changelog Toutui v0.4.0-beta (10/05/2025) \n\
         \n\
         Warning:\n\
         - If you're already using the app, please follow the upgrade instructions here: => 
         https://github.com/AlbanDAVID/Toutui/wiki/Major-upgrade-instruction#v--035-beta-to-v040-beta

         Added:\n\
         - Simplified installation and updates by: 
            - Downloading the binary.
            - Compiling it from source (no local clone needed).

         -  New commands available:
            - toutui --update and toutui --uninstall cmd added.

         - Notify if an update is available directly in the app.

         - [Linux only] The app can now be launched via an app launcher.
         \n\
         Contributors:\n\
         \n\
         - AlbanDAVID, dougy147
         \n\
         Enjoy and be toutui!\n
         ####\n".to_string();
    let changelog_13 ="Changelog Toutui v0.4.1-beta (14/05/2025) \n\
         \n\
         Warning:\n\
         - If you're already using the app v0.3.5 or bellow, please follow the upgrade instructions here: => 
         https://github.com/AlbanDAVID/Toutui/wiki/Major-upgrade-instruction#v--035-beta-to-v040-beta

         Added:\n\
         - Archlinux users: the app is now available in the AUR (yay -S toutui)

         Changed:\n\
         - Minor changes in the installation process.

         \n\
         Contributors:\n\
         \n\
         - AlbanDAVID
         \n\
         Enjoy and be toutui!\n
         ####\n".to_string();
    let changelog_14 =
    "Changelog Toutui v0.4.2-beta (15/05/2025) \n\
         \n\
         Warning:\n\
         - If you're already using the app v0.3.5 or bellow, please follow the upgrade instructions here: => 
         https://github.com/AlbanDAVID/Toutui/wiki/Major-upgrade-instruction#v--035-beta-to-v040-beta

         Added:\n\
         - Verifying file integrity using SHA-256 before installation via curl script

         Changed:\n\
         - Clarification of update/uninstall instructions

         \n\
         Contributors:\n\
         \n\
         - AlbanDAVID
         \n\
         Enjoy and be toutui!\n
         ####\n".to_string();

    changelog.push_str(&changelog_14);
    changelog.push_str(&changelog_13);
    changelog.push_str(&changelog_12);
    changelog.push_str(&changelog_11);
    changelog.push_str(&changelog_10);
    changelog.push_str(&changelog_09);
    changelog.push_str(&changelog_08);
    changelog.push_str(&changelog_07);
    changelog.push_str(&changelog_06);
    changelog.push_str(&changelog_05);
    changelog.push_str(&changelog_04);
    changelog.push_str(&changelog_03);
    changelog.push_str(&changelog_02);
    changelog.push_str(&changelog_01);

    changelog
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The two versions of this fork that have no release. The tag `v0.6.6` came
    /// before the version of `Cargo.toml`, therefore the workflow refused it and
    /// the work of that tag went to v0.6.7. No commit ever gave the version
    /// 0.7.25.
    const THE_VERSIONS_WITH_NO_RELEASE: &[&str] = &["0.6.6", "0.7.25"];

    /// The first release of this fork. Every version before it belongs to the
    /// original project.
    const THE_FIRST_VERSION_OF_THE_FORK: &str = "0.5.0";

    /// Gives the three numbers of a version.
    fn the_numbers_of_a_version(version: &str) -> (u32, u32, u32) {
        let mut numbers = version.split('.').map(|number| {
            number
                .parse::<u32>()
                .unwrap_or_else(|_| panic!("\"{}\" is not a version", version))
        });

        (
            numbers.next().expect("a version holds three numbers"),
            numbers.next().expect("a version holds three numbers"),
            numbers.next().expect("a version holds three numbers"),
        )
    }

    /// Gives the version of every entry of this fork, the oldest first.
    fn the_versions_of_the_fork() -> Vec<String> {
        let text = changelog();
        let mut versions: Vec<String> = Vec::new();

        for line in text.lines() {
            let line = line.trim();

            let Some(rest) = line.strip_prefix("Changelog Toutui v") else {
                continue;
            };

            let version = rest
                .split_whitespace()
                .next()
                .expect("the line of an entry names a version");

            // The original project ends at v0.4.2-beta.
            if version.contains('-') {
                continue;
            }

            if the_numbers_of_a_version(version)
                < the_numbers_of_a_version(THE_FIRST_VERSION_OF_THE_FORK)
            {
                continue;
            }

            versions.push(version.to_string());
        }

        versions.reverse();
        versions
    }

    /// **The screen of the changelog must name the version of the program.** The
    /// entries stopped at v0.6.8 while the program was at v0.7.46, therefore 38
    /// releases of this fork reached no user. See T-101.
    #[test]
    fn the_changelog_holds_an_entry_for_the_version_of_the_program() {
        let versions = the_versions_of_the_fork();

        assert_eq!(
            versions.last().map(|version| version.as_str()),
            Some(VERSION),
            "the newest entry of the changelog does not name v{}. A release \
             writes its entry in the words of a user.",
            VERSION
        );
    }

    /// **Every release of this fork holds an entry.** A user who reads the screen
    /// of the changelog must find the work of each version. See T-101.
    #[test]
    fn the_changelog_holds_an_entry_for_every_release_of_the_fork() {
        let versions = the_versions_of_the_fork();

        assert_eq!(
            versions.first().map(|version| version.as_str()),
            Some(THE_FIRST_VERSION_OF_THE_FORK),
            "the oldest entry of this fork must be v{}",
            THE_FIRST_VERSION_OF_THE_FORK
        );

        for two in versions.windows(2) {
            let before = the_numbers_of_a_version(&two[0]);
            let after = the_numbers_of_a_version(&two[1]);

            // A new minor version starts at the patch 0.
            if after == (before.0, before.1 + 1, 0) {
                continue;
            }

            assert!(
                after.0 == before.0 && after.1 == before.1 && after.2 > before.2,
                "v{} does not come after v{}",
                two[1],
                two[0]
            );

            // Every version between the two must have no release.
            for patch in (before.2 + 1)..after.2 {
                let missing = format!("{}.{}.{}", before.0, before.1, patch);

                assert!(
                    THE_VERSIONS_WITH_NO_RELEASE.contains(&missing.as_str()),
                    "the changelog holds no entry for v{}. It stands between \
                     v{} and v{}.",
                    missing,
                    two[0],
                    two[1]
                );
            }
        }
    }

    /// **An entry names its own version, and never the version of the build.**
    /// The entry of v0.6.9 took `CARGO_PKG_VERSION`, therefore the screen said
    /// "Changelog Toutui v0.7.46" above the words of v0.6.9 and the fault stayed
    /// hidden for 38 releases. See T-101.
    #[test]
    fn no_entry_of_the_changelog_takes_the_version_of_the_build() {
        let source = include_str!("changelog.rs");

        let start = source
            .find("const THE_ENTRIES_OF_THE_FORK")
            .expect("the module holds the entries of the fork");
        let end = source[start..]
            .find("\n];")
            .expect("the list of the entries ends")
            + start;

        for line in source[start..end].lines() {
            assert!(
                !line.contains("VERSION"),
                "an entry takes the version of the build: {}. Write the version \
                 of that release.",
                line.trim()
            );
        }
    }

    /// Every entry names a version one time, and two entries never name one
    /// version.
    #[test]
    fn every_entry_of_the_changelog_names_its_own_version() {
        let mut versions: Vec<&str> = THE_ENTRIES_OF_THE_FORK
            .iter()
            .map(|entry| entry.version)
            .collect();

        let count = versions.len();
        versions.sort_unstable();
        versions.dedup();

        assert_eq!(count, versions.len(), "two entries name one version");

        for entry in THE_ENTRIES_OF_THE_FORK {
            assert!(
                !entry.body.is_empty(),
                "the entry of v{} holds no line",
                entry.version
            );

            for line in entry.body {
                assert!(
                    !line.contains('\n'),
                    "one line of the entry of v{} holds a new line: {}. The view \
                     wraps a line, therefore one item is one line.",
                    entry.version,
                    line
                );
            }

            let text = the_text_of_an_entry(entry);

            assert!(
                text.starts_with(&format!("Changelog Toutui v{} (", entry.version)),
                "the entry of v{} does not name that version",
                entry.version
            );
        }
    }
}
