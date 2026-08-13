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
