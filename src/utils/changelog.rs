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
