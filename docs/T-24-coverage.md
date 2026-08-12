# T-24: what Audiobookshelf gives, and what Toutui takes

Date: 2026-08-11. The session of 2026-08-12 closed every row that said `Half`,
and the session of that evening closed the last row that said `No` for a function
that a user of a terminal can use: the key `@` sends an ebook to an e-reader
(T-119). **Every row of section 4 that says `No` now belongs to an administrator
of the server, or to work that the client must not do**, and section 6 holds the
reason of each.

This document compares the functions of an Audiobookshelf server with the
functions of this client. It gives the maintainer one list of the work that is
not done.

| What | Value |
|---|---|
| The server | Audiobookshelf 2.36.0 |
| The client | Toutui 0.7.62 (`Cargo.toml`) |
| The address of the server | `http://127.0.0.1:13399`, the sandbox of `docs/TEST-SERVER.md` |
| The user | `toutuitest`, of the type `root` |

`GET /status` gives
`{"app":"audiobookshelf","serverVersion":"2.36.0","isInit":true,...}`.

**Every measurement in this document comes from that sandbox.** No request went
to a server of a user. A row that says "not tested" holds no measurement, and
the reason is beside it.

The sandbox holds three libraries: `Books`, `Podcasts` with 1 podcast of 3
episodes of a feed of 57, and `Empty` with no item. The session of 2026-08-12
added four books to `Books`: a book of one chapter, a PDF of 47 megabytes of a
scan of 60 pages, a PDF that no reader reads, and their audio.

## 1. Two errors of the official reference

<https://api.audiobookshelf.org> does not agree with 2.36.0 in two places. A
measurement of both forms gives this:

| Path | Answer of 2.36.0 |
|---|---|
| `GET /api/podcasts/:id/episode-downloads` | `404` |
| `GET /api/libraries/:id/episode-downloads` | `200`, `{"queue":[]}` |
| `POST /api/session/:id/sync` | `200` |
| `POST /api/sessions/:id/sync` | `404` |
| `POST /api/me/item/:id/bookmark` | `200` |
| `POST /api/me/bookmarks` | `404` |

The third row is a correction of an older version of this document: it named
`GET /api/podcasts/:id/episode-downloads` as an endpoint that answers, and a
measurement on 2026-08-11 gives `404`. The queue belongs to the library.

The reference gives the second form of each pair. Toutui uses the first form of
the first pair, therefore Toutui is correct. Read the reference with care, and
measure the path before you write it.

## 2. What Toutui asks the server today

This list comes from `src/api/` and from `src/logic/`. It is complete.

| Request | Where |
|---|---|
| `POST /login`, `GET /ping` | `src/api/server/auth_process.rs`, `src/api/client/probe.rs` |
| `GET /api/libraries` | `src/api/libraries/get_all_libraries.rs` |
| `GET /api/libraries/:id/items?limit=500&page=N&sort=&desc=&filter=&collapseseries=1` | `src/api/libraries/get_all_books.rs`. **The program reads one page at a time** (T-70) |
| `GET /api/libraries/:id/filterdata` | `src/api/libraries/get_filter_data.rs` |
| `GET /api/libraries/:id/personalized` | `src/api/libraries/get_library_perso_view.rs` |
| `GET /api/libraries/:id/series?limit=500&page=N&sort=name` | `src/api/libraries/get_all_series.rs` |
| `GET /api/libraries/:id/authors` | `src/api/libraries/get_authors.rs` |
| `POST /api/libraries/:id/scan` | `src/app.rs`, the key `L` |
| `GET /api/libraries/:id/collections?limit=&page=` | `src/api/libraries/get_lists.rs` |
| `GET /api/libraries/:id/playlists?limit=&page=` | `src/api/libraries/get_lists.rs` |
| `GET /api/items/:id` | `src/logic/playback/mod.rs` |
| `GET /api/items/:id/cover` | `src/ui/cover.rs` |
| `GET /api/items/:id/ebook` | `src/logic/reader/book.rs` |
| `GET /api/items/:id/file/:ino/download` | `src/logic/download/fetch.rs`, `src/player/engine/http_file.rs` |
| `POST /api/items/:id/play`, `POST /api/items/:id/play/:episodeId` | `src/api/library_items/play_lib_item_or_pod.rs` |
| `GET /api/me/progress/:id` | `src/api/me/get_media_progress.rs` |
| `GET /api/me/listening-stats` | `src/api/me/listening_stats.rs` |
| `GET /api/me/listening-sessions?itemsPerPage=&page=` | `src/api/me/sessions.rs` |
| `GET /api/libraries/:id/stats`, `GET /api/stats/year/:year` | `src/api/stats/mod.rs` |
| `POST /api/me/item/:id/bookmark`, `DELETE /api/me/item/:id/bookmark/:time` | `src/api/me/bookmarks.rs` |
| `GET /api/me` | `src/api/me/permissions.rs`, `src/api/me/bookmarks.rs` |
| `PATCH /api/me/progress/:id`, `PATCH /api/me/progress/:id/:episodeId` | `src/api/me/update_media_progress.rs` |
| `POST /api/session/:id/sync`, `POST /api/session/:id/close` | `src/api/sessions/` |
| `GET /api/search/podcast?term=`, `POST /api/podcasts/feed`, `POST /api/podcasts`, `POST /api/podcasts/:id/download-episodes` | `src/api/podcasts/mod.rs` |
| `GET /api/libraries/:id/episode-downloads`, `GET /api/podcasts/:id/clear-queue` | `src/api/podcasts/the_downloads.rs` |

Toutui calls 32 paths. The server has more than 100.

## 3. The keys of Toutui

`src/app.rs` holds the key handler, and it is the authority.

| Key | What it does |
|---|---|
| `j`/`↓`, `k`/`↑` | Move the selection |
| `g`/`Home`, `G`/`End` | Go to the first line and to the last line |
| `J`, `K`, `H` | Scroll the description down, up, and to the top |
| `l`/`→`/`Enter` | Play the media, or open the list that the line names |
| `h` | Go back one view |
| `Tab` | Change between Home and Library |
| `Shift+Tab` | The next library of the server (T-66) |
| `/` | Search |
| `s` | Show the series of the library (T-22) |
| `c` | Show the collections and the playlists (T-9) |
| `e` | Open the ebook of the item: an EPUB book (T-10) or a PDF book (T-54) |
| `D`, `X` | Get a local copy, and remove a local copy |
| `R` | Ask the server again |
| `F` | Send the position now (T-32) |
| `T` | Show the time that you listened (T-24) |
| `W` | Show every session that you played, with pages (T-24) |
| `N` | Take a media away from Continue Listening, or put it back (T-24) |
| `C` | Show the chapters of the media that plays (T-24) |
| `b` | Write a bookmark at the place of the playback (T-24) |
| `V` | Show the bookmarks of a media (T-24) |
| `t` | The timer for sleep (T-24) |
| `A` | Look for a new podcast, and add it (T-24) |
| `E` | The server gets the episodes that it does not hold (T-24) |
| `d` | The episodes that the server downloads, and the queue of that work (T-81) |
| `m` | Put the media in a collection or in a playlist (T-84) |
| `<`, `>` | Move the media of a list one line up, and down (T-102) |
| `a` | Show the authors of the library (T-24) |
| `L` | The server examines the library again (T-24) |
| `f` | Choose the sequence and the filter of the library (T-24) |
| `S` | Settings |
| `B` | Show the keys, or hide them |
| `Q`/`Esc` | Close the application |
| ` ` | Play, or pause |
| `p`, `u` | Go forward, and go back |
| `P`, `U` | The next chapter, and the chapter before |
| `O`, `I` | The speed up, and the speed down |
| `o`, `i` | The volume up, and the volume down |
| `Y` | Stop the playback |

The reader of a book takes the keys first, and it uses the same letters for a
different work: `j`/`k` a line, `Space`/`b` a page, `n`/`p` a chapter, `t` the
contents, `g`/`G` the start and the end, `s` sends the place, `e` the list of
the books of that media (T-76), and `h` leaves the book.

The variable `TOUTUI_NO_COVERS` stops the cover art. The variable
`TOUTUI_COVERS_IN_TMUX` asks the terminal inside tmux. The variable
`TOUTUI_AUDIO_DEVICE` names the sound device.

## 4. The table of the functions

`Yes` means that the user can do the work. `Half` means that a part operates.
`No` means that the client has nothing.

**A row can be old. Read the code before you take one.** A session of
2026-08-11 read the code of each row that said `No` or `Half`, and it corrected
four of them: the live messages (`No`, and the work landed with T-47 and T-66),
the list of the ebooks of an item (the reader reads a PDF since T-54, and T-76
gives the list of every ebook), the narrators (T-73), and one new row for the
reader of a PDF.

| Function | The server | Toutui | What is missing |
|---|---|---|---|
| **Sign in** | `POST /login` gives a token of 201 characters. `GET /status` gives `authMethods: ["local"]` and the fields of OpenID | Yes | OpenID. `GET /api/auth-settings` gives 14 fields of OpenID. A terminal cannot open a browser page of a provider with no work |
| **More than one server** | Not a function of the server | Yes | Nothing. `[[servers]]` holds the addresses, and the pool selects one |
| **List the libraries** | `GET /api/libraries` gives 3 libraries with `id`, `name`, and `mediaType` | Yes | Nothing |
| **Choose the library** | The client holds the choice | Yes | Nothing. `S` then the line "Library" |
| **The items of a library** | `GET /api/libraries/:id/items?limit=&page=` gives `results`, `total`, `limit`, `page` | Yes | Nothing. The client asks for pages of 500 (T-7) |
| **Sort the items** | `?sort=media.metadata.title&desc=1` changes the sequence. Measured: `desc=1` gives `Volume 3, Volume 2, Volume 1`, and no `desc` gives `A Long Test Book, Alice in Wonderland, Multi File Test Book` | Yes | Nothing. The key `f` gives seven fields for a library of books and three for a library of podcasts, and a line that changes the direction. The choice belongs to the account, therefore it stays after the program stops |
| **Filter the items** | `?filter=<type>.<base64>` gives `filterBy` in the answer. `GET /api/libraries/:id/filterdata` gives the values: 4 authors, 2 series, 0 genres, 0 tags, 0 narrators, 0 languages | Yes | Nothing. The key `f` gives the authors, the series, the genres, the tags, the narrators, the languages, the publishers, and the three values of the position |
| **Group a series in the list** | `?collapseseries=1` gives `collapseseries` in the answer, and **`total` then counts the lines**: the library of the sandbox gives 14 items with no parameter and **10 with it** | Yes | Nothing. The program sends the parameter for a library of books since 2026-08-12. A measurement of that day compared the two answers: the same 10 lines, in the same sequence, and the same series of one book. `group_library` stays, and it gives the line of a series the place of that series in `App::series`: the view reads the books, the description, and the cover there |
| **The shelves of Home** | `GET /api/libraries/:id/personalized` gives 6 shelves for a book library: `continue-listening` (4), `recently-added` (9), `recent-series` (2), `discover` (2), `listen-again` (2), `newest-authors` (4). A podcast library gives `newest-episodes` (3), `recently-added` (1), `listen-again` (2) | Yes | The shelf `newest-authors` only. An author holds no media and no book, therefore a terminal can show nothing for that shelf. Every other shelf gives its name and its lines. A line of `recent-series` opens the books of the series |
| **Search** | `GET /api/libraries/:id/search?q=` gives six groups for a library of books: `book`, `authors`, `series`, `narrators`, `tags`, `genres`. **A library of podcasts gives `podcast`, `episodes`, `tags`, and `genres`** (measured 2026-08-12: `q=Balzac` gives 1 podcast). Measured: `q=Volume` gives 5 books, `q=Carroll` gives the author "Lewis Carroll" and no book | Yes | Nothing. The key `/` shows the titles that the client holds at once, and it adds the answer of the server when that answer comes. `ask_the_server_to_search` of `src/app.rs` sends the request, and `src/logic/search/from_the_server.rs` keeps the answer. **The answer carries the media of every hit since 2026-08-12** (T-113): the view shows a book of a page that the program did not read, and a search of a library of podcasts gives its lines. The group `episodes` stays outside: no measurement of the sandbox gave one hit of it |
| **The series of a library** | `GET /api/libraries/:id/series?limit=&page=` gives `results` and `total`. `limit=0` gives an empty list, and not every series | Yes | Nothing. The key `s`, and one line for each series in the Library view |
| **One series** | `GET /api/series/:id` gives `id`, `name`, `nameIgnorePrefix`, `description`. **`GET /api/libraries/:id/series` gives the same `description` for every series of the page**, therefore this endpoint gives nothing new | Yes | Nothing, and the client must not use this endpoint. A measurement on 2026-08-11 gave a description to one series of the sandbox with `PATCH /api/series/:id`, and the list that the client already asks for carried that text. The view of the series shows it, and a series with no description shows the description of its first book (T-43) |
| **Collections** | `GET /api/libraries/:id/collections` and `GET /api/collections` give the collections. `POST /api/collections/:id/book` adds one, and `DELETE /api/collections/:id/book/:itemId` removes one. `POST /api/collections` makes one, `DELETE /api/collections/:id` removes one, and `PATCH /api/collections/:id` takes `name`, `description`, and `books` | Yes | Nothing. The client reads, plays, adds a book (`m`), removes a book (`X`), makes a collection (`c` of the view of the key `m`, T-88), gives it a new name (`r`, T-93) and a new description (`D`, T-100), removes it (`X`, T-93), and writes the sequence of its books (`<` and `>`, T-102) |
| **Playlists** | `GET /api/libraries/:id/playlists` and `GET /api/playlists`. An entry holds `libraryItemId` and, for an episode, `episodeId`. `POST /api/playlists/:id/item` adds one, and `DELETE /api/playlists/:id/item/:itemId` removes one. `PATCH /api/playlists/:id` takes `items`, and **the body must hold every media of the playlist**: one item fewer gives `400`, "Invalid playlist items. Length mismatch" | Yes | The same as the collections: the key `p` of the view of the key `m` makes a playlist (T-88), and the keys `<` and `>` write the sequence of its media (T-102) |
| **The position of a media** | `GET /api/me/progress/:id` gives `currentTime`, `progress`, `isFinished`, `hideFromContinueListening`, `ebookLocation`, `ebookProgress`, `lastUpdate` | Yes | Nothing. The client reads the position at the start and it writes the position (T-4) |
| **Mark as finished** | `PATCH /api/me/progress/:id` with `{"isFinished":true}` | Yes | Nothing. The key `M`, and it marks a media back also |
| **Hide from Continue Listening** | The field `hideFromContinueListening` of `PATCH /api/me/progress/:id` | Yes | Nothing. The key `N`. A measurement on 2026-08-11 shows that the shelf of the server loses the media at once |
| **Open a session** | `POST /api/items/:id/play` gives `id`, `audioTracks`, `chapters`, `duration`, `playMethod` | Yes | Nothing |
| **Sync a session** | `POST /api/session/:id/sync` gives `200` | Yes | Nothing. The key `F` sends the position now (T-32) |
| **Close a session** | `POST /api/session/:id/close` gives `200` | Yes | Nothing |
| **The sessions of the user** | `GET /api/me/listening-sessions` gives `total`, `numPages`, `page`, `itemsPerPage`, `sessions`. It takes `itemsPerPage` and `page`. **The first page is the page 0**, and a page after the last page gives `200` and an empty list | Yes | Nothing. The key `W`. The view holds 25 sessions of one page, and it reads the next page when the user comes near the end |
| **The sessions of the server** | `GET /api/sessions` gives the same shape for every user | No | Everything. This is for an administrator |
| **Bookmarks** | `POST /api/me/item/:id/bookmark` with `{"time":12,"title":"..."}` gives `200` and `{libraryItemId,time,title,createdAt}`. `DELETE /api/me/item/:id/bookmark/:time` gives `200`, and `404` for a place that does not exist. `GET /api/me` gives the field `bookmarks` | Yes | Nothing. The key `b` writes a place, the key `V` shows the list, `l` goes to a place, and `X` removes one. The client reads the bookmarks of a different client, because they come from `GET /api/me` |
| **Play, pause, and stop** | The client does this work | Yes | Nothing. ` ` and `Y` |
| **Go forward and back** | The client does this work | Yes | Nothing. `p` and `u` |
| **Chapters** | `POST /api/items/:id/play` gives `chapters` with `start`, `end`, and `title` | Yes | Nothing. `P` and `U`, and the player shows the name of the chapter. `src/logic/playback/mod.rs:73` reads them |
| **The list of the chapters** | The same field | Yes | Nothing. The key `C` shows them, with a mark on the chapter that plays, and `l` goes to a chapter |
| **The speed** | The client does this work | Yes | Nothing. `O` and `I`, and the pitch does not change (T-19) |
| **The volume** | The client does this work | Yes | Nothing. `o` and `i` |
| **A timer for sleep** | Not a function of the server | Yes | Nothing. The key `t` gives 5, 10, 15, 30, 45, and 60 minutes, the end of the chapter, and then off. The volume falls in the last 30 seconds. The player shows the time that is left |
| **A queue of media** | Not a function of the server. Audiobookshelf holds its queue in the web page, and it sends it to no client | Yes | Nothing. The key `n` puts the selected media at the end of the queue, and the key `q` shows the queue. `l` starts a media now, and `X` takes one out. **The queue goes on at an end, and at nothing else.** `src/logic/queue.rs` |
| **The cover art** | `GET /api/items/:id/cover` gives `200` and the bytes | Yes | Nothing. T-23. The panel stands beside the description, and a series shows its books |
| **The description** | `media.metadata.description` of the item | Yes | Nothing. `src/utils/html_text.rs` removes the HTML tags (T-13) |
| **Read an EPUB book** | `GET /api/items/:id/ebook` gives `200` and the whole file, and it takes a `Range` | Yes | Nothing. The reader writes an EPUBCFI in `ebookLocation` and it reads one, therefore the user reads on the telephone and continues in the terminal at the same line (T-10). The path agrees with `epub.js`, the library of the web reader: a measurement with a real browser on 2026-08-11 compared 29315 texts of seven books, and every path agreed. The reader also reads the form of the specification, which the versions v0.7.8 to v0.7.11 wrote. See `src/logic/reader/cfi.rs` |
| **The list of the ebooks of an item** | `media.ebookFile` names one book, and `libraryFiles` holds every file of the item. `GET /api/items/:id/ebook/:ino` gives one of them | Yes | Nothing. The key `e` opens the book of the server, and the key `e` inside the reader gives the list of every ebook of that media (T-76). **The server holds one place for each media**, therefore the place of a book that is not the book of the server stays on this machine |
| **Read a PDF book** | The same endpoint gives the file | Yes | Nothing. T-54 gives the words of a page in the terminal, and it draws the pictures of that page beside them with the protocol of T-23. A book of a scan holds its text inside a picture, therefore such a book gives few words. `MAX_BOOK_BYTES` of 512 megabytes holds the memory of the read (T-62) |
| **Send an ebook to an e-reader** | **`POST /api/authorize` gives the devices of the account** in `ereaderDevices`, and the server filters that list itself. `POST /api/emails/send-ebook-to-device` with `{libraryItemId, deviceName}` sends the book of `media.ebookFile`. **`GET /api/emails/settings` cannot do this work**: every endpoint of `/api/emails/` holds an `adminMiddleware`, and it answers `404` for an account that is not an administrator | Yes | Nothing. The key `@` on a book gives the devices of this account, and `l` sends the book (T-119). The three answers of `404` of that endpoint say three different things, therefore the program reads the body: an item with no ebook, a device that went away, and an item that went away each hold their own sentence. **The send holds a time limit of 15 minutes**: the server took 36 seconds for a book of 479.5 megabytes, and the limit of a request is 15 seconds |
| **List the podcasts** | The same endpoint as the books | Yes | Nothing |
| **The episodes of a podcast** | `GET /api/items/:id` gives `media.episodes` | Yes | Nothing. `l` on a podcast gives the episodes |
| **Play an episode** | `POST /api/items/:id/play/:episodeId` | Yes | Nothing |
| **Search a new podcast** | `GET /api/search/podcast?term=balzac` gives a list of 48, with `title`, `artistName`, `description`, `feedUrl`, `trackCount`, `cover`. **`limit` changes nothing** | Yes | Nothing. The key `A` in a library of podcasts |
| **Read a feed** | `POST /api/podcasts/feed` with `{"rssFeed":"..."}` gives `200` and the key `podcast` | Yes | Nothing. The key `A`, after the user selects an answer |
| **Make a podcast** | `POST /api/podcasts` gives `200` and the new item. A second add of one podcast gives `400`, because the directory exists | Yes | Nothing. The key `A` asks the user before it sends, because the request writes in the library |
| **The server gets an episode** | `POST /api/podcasts/:id/download-episodes` with the episodes of the feed gives `200`, and the server holds the file a few seconds later. **`GET /api/podcasts/:id/episode-downloads` gives `404`** on 2.36.0; `GET /api/libraries/:id/episode-downloads` gives `{"queue":[]}` | Yes | Nothing. The key `E` |
| **Look for a new episode** | `GET /api/podcasts/:id/checknew` gives `200` and the key `episodes`. **It compares with the time of the last examination.** A measurement of 2026-08-12 against a podcast that holds 3 episodes of a feed of 57: the endpoint gives **0** episodes in 15 bytes, and `POST /api/podcasts/feed` gives **57** in 27598 bytes | Yes | Nothing. The key `E`. The program reads the feed and it compares with the episodes of the server itself, therefore it finds every one of the 54 episodes that are missing. **The endpoint stays outside**, and section 6 holds the reason: it is cheaper only where it is wrong |
| **Empty the queue of the podcast** | `GET /api/podcasts/:id/clear-queue` gives `200`. It does **not** stop the episode that downloads now | Yes | Nothing. The key `d` shows the queue and the key `X` empties it (T-81) |
| **A local copy** | `GET /api/items/:id/file/:ino/download` gives the one audio file | Yes | Nothing. `D` and `X`, for a book and for one episode (T-1, T-11) |
| **Play with no server** | Not a function of the server | Yes | Nothing. The positions wait in `pending_progress`, and a task sends them (T-25) |
| **The archive of a whole item** | `GET /api/items/:id/download` gives `200` and a ZIP archive | No | The client does not use it, and it must not: T-1 says that the archive cannot play |
| **Change the metadata of an item** | `PATCH /api/items/:id/media`. `docs/TEST-SERVER.md` used it. Not tested today | No | Everything. The client reads, and it never writes |
| **Find the metadata of an item** | `POST /api/items/:id/match` gives `200`. With `{"provider":"google"}` it gives `{"warning":...}`, because the item has no title to match | No | Everything |
| **Make an M4B file** | `POST /api/items/:id/encode` of the reference. Not tested: the request starts a long job of ffmpeg on the server | No | Everything. See section 6 |
| **Write the metadata in the audio files** | `POST /api/items/:id/update-embedded-metadata` of the reference. Not tested, for the same reason | No | Everything. See section 6 |
| **Scan a library** | `POST /api/libraries/:id/scan` gives `200` | Yes | Nothing. The key `L`. The examination runs on the server, therefore the program says that the work started and the user presses `R` after a moment |
| **The authors of a library** | `GET /api/libraries/:id/authors` gives the key `authors`, with `name`, `description`, and `numBooks`. `GET /api/authors/:id` gives no `numBooks`, therefore the list is the whole answer | Yes | Nothing. The key `a` shows the authors in the sequence of the alphabet, and `l` shows the books of one author |
| **The narrators of a library** | `GET /api/libraries/:id/narrators` gives the key `narrators` | Yes |Nothing. The key `v`, and one view with the authors. The filter of a narrator takes the name, and not an identity. See T-73 |
| **The tags** | `GET /api/tags` gives the key `tags` of every library | Yes | Nothing. The key `f` holds a line for each tag of the library that the user reads, and `GET /api/libraries/:id/filterdata` gives those tags. **The endpoint stays outside**, and section 6 holds the reason |
| **The statistics of the library** | `GET /api/libraries/:id/stats` gives `totalItems`, `totalSize`, `totalDuration`, `numAudioTracks`, `largestItems`, `longestItems`, `totalAuthors`, `totalGenres` | Yes | Nothing. The view of the key `T` holds the group "The library", with the five longest items and the five largest items |
| **The statistics of the user** | `GET /api/me/listening-stats` gives `totalTime` 281, `today` 281, `days` `{"2026-08-10":281}`, `dayOfWeek` `{"Monday":281}`, `items` (a map of 2), and `recentSessions` (5) | Yes | Nothing. The key `T` shows the time of this day and the time in total, the last 14 days, the seven days of the week, the five media of the largest time, and the five last sessions |
| **The statistics of a year** | `GET /api/stats/year/2026` gives `numListeningSessions`, `totalListeningTime`, `topAuthors`, `topNarrators`, `topGenres`, and 8 more fields. **`topGenres` names its value `genre`, and the two other lists name it `name`** | Yes | Nothing. The view of the key `T` holds the group "The year". The lists of the narrators and of the genres come from the copy of the metadata inside each session, therefore a session that came before the metadata gives an empty list |
| **The account of the user** | `GET /api/me` gives `id`, `username`, `type` (`root`), `permissions`, `mediaProgress` (9 rows), `bookmarks`, `lastSeen` | Yes | Nothing. The settings, and then "Accounts and log out": the screen says the type of the account and every permission that changes the work of the program, in the words of a user. See T-110 |
| **The permissions** | `GET /api/me` gives 9 permissions: `download`, `update`, `delete`, `upload`, `createEreader`, `accessAllLibraries`, `accessAllTags`, `accessExplicitContent`, `selectedTagsNotAccessible` | Yes | Nothing that a user of a terminal meets. The program reads `download`, `update`, and `delete`, and the settings say each of them before the user presses a key. `upload`, `createEreader`, and the three permissions of the libraries and of the tags belong to work that this program does not do. **An absent permission means "yes"**: a server that gives no permission must not stop the user |
| **The users of the server** | `GET /api/users` gives the key `users`. `GET /api/users/online` gives `usersOnline` and `openSessions`. `POST`, `PATCH`, and `DELETE` of the reference are not tested: they change the accounts of the server | No | Everything. See section 6 |
| **An RSS feed of an item** | `GET /api/feeds` gives `{"feeds":[],"minified":false}`. `POST /api/items/:id/open-feed` of the reference is not tested: it makes a public address | No | Everything. See section 6 |
| **A share of an item** | `GET /api/share/xx` gives `404` for an identity that does not exist, therefore the group answers | No | Everything. See section 6 |
| **The notifications** | `GET /api/notifications` gives `data` and `settings` | No | Everything. See section 6 |
| **The backups** | `GET /api/backups` gives `backups`, `backupLocation`, `backupPathEnvSet` | No | Everything. See section 6 |
| **The file system of the server** | `GET /api/filesystem` gives `posix` and `directories` | No | Everything. See section 6 |
| **The settings of the sign in** | `GET /api/auth-settings` gives 14 fields, and 12 of them belong to OpenID | No | Everything. See section 6 |
| **A stream of the server** | `POST /api/items/:id/play` with `forceTranscode` gives one address of HLS for the whole media | Yes | Nothing. The program reads the file itself, and it takes the stream for a file that no decoder of the program reads. See T-53 |
| **Live messages** | Audiobookshelf sends the changes over socket.io | Yes | Nothing. The second transport of socket.io is plain HTTP, therefore this needed no dependency. `src/api/live.rs` holds the connection and `src/logic/live.rs` holds the box between the task and the render. The mark of a line takes the position of a different client (T-44 and T-47), a media that a different client finished leaves the shelf of Continue Listening with no request (T-66), and a change of the metadata makes the header ask for the key `R`. The log holds the name of a message and never its body, because `user_updated` carries a new token |

## 5. What is worth building next

The sequence inside each group gives the value for the work.

### Small: a few hours each

1. ~~**Search on the server.**~~ **Done on 2026-08-11.**
   `GET /api/libraries/:id/search?q=`, `src/api/libraries/search_library.rs`, and
   the slot `logic::search::from_the_server`. The screen shows the titles of the
   program at once, and the answer of the server when it comes.
   **T-70 finished it:** the group of the books of that endpoint holds no name of
   an author, therefore a search of "carroll" gave one author and no line. The
   program asks the server for the books of each author and of each narrator that
   it found, with `?filter=authors.<base64>` and `?filter=narrators.<base64>`. A
   tag and a genre stay with the view of the filter of T-60.
2. ~~**A key that marks a media as finished.**~~ **Done on 2026-08-11.** The
   key of the mark, and `mark_the_media`. The book leaves the shelf of Continue
   Listening at the next frame, and the user presses no other key: T-66 reads
   `isFinished` and `hideFromContinueListening` of the live message.
3. ~~**The statistics of the user.**~~ **Done on 2026-08-11.** The key `T`,
   `src/api/me/listening_stats.rs`, `src/logic/stats.rs`, and
   `src/ui/stats_tui.rs`. The bar uses the blocks of Unicode, therefore the
   program needs no new dependency.
4. ~~**A choice of the sequence.**~~ **Done on 2026-08-11.** The key `f`, and
   `src/logic/sort_filter.rs`.
5. ~~**Hide a media from Continue Listening.**~~ **Done on 2026-08-11.** The
   key `N`, and `hide_the_media` beside `mark_the_media`. The line goes away at
   the next frame, and it comes back at the second press. See T-66.

### Medium: one or two days each

6. ~~**Bookmarks.**~~ **Done on 2026-08-11.** The keys `b` and `V`, and
   `src/api/me/bookmarks.rs`.
7. ~~**The other shelves of Home.**~~ **Done on 2026-08-11.**
   `get_the_shelves` gives every shelf, and `src/logic/home_view.rs` makes the
   lines. The request did not change.
8. ~~**Add a podcast.**~~ **Done on 2026-08-11.** The key `A`, and
   `src/api/podcasts/mod.rs`.
9. ~~**Filter the library.**~~ **Done on 2026-08-11.** The same view as
   item 4, and `src/api/libraries/get_filter_data.rs`.
10. ~~**The server gets an episode.**~~ **Done on 2026-08-11.** The key `E`.

### Large: a week or more each

11. ~~**A timer for sleep.**~~ **Done on 2026-08-11.** The key `t`, and
    `src/logic/sleep_timer.rs`. The work needed no change of the engine: the
    loop of the program sends `SetVolume` and `Pause`.
12. ~~**The place of the ebook in the form of the web reader.**~~ **Done on
    2026-08-11.** `src/logic/reader/cfi.rs` walks the tree of the XHTML of one
    chapter and it makes the whole path. The reader writes an EPUBCFI, and it
    reads one. The letter is the common unit of the tree and of the screen: a
    measurement on 2026-08-11 gave a difference of 0 letters for all 74
    chapters of the four books of the survey.
13. ~~**A view of the authors.**~~ **Done on 2026-08-11.** The key `a`. The
    key `l` uses the filter of the author, and the program held that work
    already.
14. ~~**Live messages.**~~ **Done on 2026-08-11.** socket.io gives the changes
    of a different client. The two crates of socket.io both bring
    `native-tls`, therefore the rule of section 6 refuses both. **The work
    needed no crate:** the transport `polling` of socket.io is plain HTTP, and
    `reqwest` does it already. `src/api/live.rs` holds the connection, and
    `src/logic/live.rs` holds the box between the task and the render. T-47 and
    T-66 of `docs/TAKEOVER-BACKLOG.md` hold the measurements.

**The four works that stay, and no one of them is large:**

15. ~~**Empty the queue of the podcast.**~~ **Done on 2026-08-11.** The key `d`
    shows the queue of the downloads of the server, and the key `X` empties the
    queue of one podcast (T-81).
16. ~~**The list of the ebooks of an item.**~~ **Done on 2026-08-11.** The key
    `e` inside the reader gives the list, and `l` opens one book of it (T-76).
17. ~~**A view of the settings that writes `config.toml`.**~~ **Done on
    2026-08-11.** The line "The reader: the cache of the ebooks" of the settings
    writes `ebook_cache_mb` of the block `[reader]`, and the write keeps every
    comment of the file (T-77).
18. ~~**`GET /api/podcasts/:id/checknew` for the new episodes.**~~ **Closed on
    2026-08-12 with a measurement, and no code.** A podcast that holds 3
    episodes of a feed of 57 gives **0** episodes with that endpoint. The work
    of the program finds all 54 that are missing. See section 6.

## 6. What Toutui should not do

### The rule of the dependencies

`docs/TAKEOVER-BACKLOG.md`, T-20, gives the rule of 2026-08-10:

> A dependency that compiles C when a person builds the program is acceptable,
> if the binary that the release gives needs no library of the system. Pure
> Rust stays the better answer, and the work must prefer it. A dependency that
> makes the binary ask the system for a library is not acceptable.

`Cargo.toml` follows that rule in every line: `ratatui-image` takes no default
feature, because `chafa-dyn` and `chafa-static` need the C library chafa;
`rbook` came in place of `epub-parser`, because that crate brings `bzip2-sys`,
`lzma-sys`, and `zstd-sys`; and reqwest stays on 0.12, because 0.13 brings
`aws-lc-sys` and that crate needs cmake. Therefore no function of this list
may bring a library of the system.

Two dependencies compile C today: `libsqlite3-sys` and `ring`. Both put the
code in the binary. `cargo tree -i openssl-sys` finds nothing.

### The work of the server

These functions run a long job on the server. They need a bar of progress that
comes over socket.io, and the terminal gives no value above the web page.

| Function | Endpoint | Why not |
|---|---|---|
| Make an M4B file | `POST /api/items/:id/encode` | A job of ffmpeg of many minutes. The client would show a number only |
| Write the metadata in the files | `POST /api/items/:id/update-embedded-metadata` | The same, and it changes the files of the user |
| Change the metadata of an item | `PATCH /api/items/:id/media` | A form of 20 fields. A terminal form of that size is hard to use, and an error writes wrong data in the library |
| Find the metadata | `POST /api/items/:id/match` | It needs the same form to accept or refuse each field |

### The administration of the server

A user opens the web page one time to set these, and never again. A second
place to change them adds a risk and no value.

| Function | Endpoint |
|---|---|
| The users and their permissions | `POST`, `PATCH`, `DELETE /api/users` |
| The backups | `GET /api/backups`, and the requests that make one |
| The settings of the sign in, and OpenID | `GET /api/auth-settings` |
| The settings of the email, and the devices of an e-reader that an administrator makes | `GET`/`PATCH /api/emails/settings`, `POST /api/emails/ereader-devices`. **A user sends a book with no one of them** (T-119) |
| The notifications | `GET /api/notifications` |
| The file system of the server | `GET /api/filesystem` |
| Upload a file | The requests of upload |

### The endpoints that give less than the work of the program

A measurement of 2026-08-12 compared each of these endpoints with the work that
the program does today. **The endpoint is cheaper only where it is wrong**,
therefore the program keeps its own work.

| Function | Endpoint | Why not |
|---|---|---|
| Look for a new episode of a podcast | `GET /api/podcasts/:id/checknew` | **It compares with the time of the last examination**, and not with the episodes that the server holds. A podcast of the sandbox holds 3 episodes of a feed of 57: the endpoint gives **0** episodes in 15 bytes, and the feed gives 57 in 27598 bytes. The program compares the feed with the episodes of the server, therefore the key `E` finds all 54 that are missing. An answer that loses 54 episodes is not a cheaper answer |
| The tags of the server | `GET /api/tags` | It gives the tags of **every** library. The key `f` filters the library that the user reads, and `GET /api/libraries/:id/filterdata` gives the tags of that library with the authors, the series, the genres, the narrators, the languages, and the publishers: one request gives every line of that view. A tag of a different library is a line that gives the user no media |

### The functions that make a public address

| Function | Endpoint | Why not |
|---|---|---|
| Open an RSS feed | `POST /api/items/:id/open-feed` | The address is public, and a key that is pressed by mistake would open a library to the network |
| Share a media | The group `/api/share` | The same risk |

A user who wants these must use the web page, where the page asks a question
before it opens the address.

### The functions that a terminal cannot show

| Function | Why not |
|---|---|
| Read a CBZ | A page of such a book is a picture of a drawing, and the text of that page stands inside the picture. A terminal of 160 columns gives 160 cells for a page of 2000 pixels |
| The picture of an author | The same. It gives no information |
| A player of video | Audiobookshelf holds no video |

**A measurement changed the answer for a PDF.** This row said "Read a PDF and a CBZ"
before 2026-08-11, and the reason was that a page of a PDF is a picture. That is
true of a book of a scan only. **A PDF of text holds the text**, therefore T-54 gives
the words of the page in the terminal, and it draws the pictures of that page beside
them with the protocol of T-23. T-62 holds the memory of a book of a scan to 9.5
megabytes, and T-57 gives a picture of 16 bits. A book of a scan of 500 megabytes
stays outside `MAX_BOOK_BYTES`.

## 7. What this document did not measure

| Item | Why |
|---|---|
| `POST /api/items/:id/encode` | It starts a long job of ffmpeg |
| `POST /api/items/:id/update-embedded-metadata` | It changes the audio files |
| `PATCH /api/items/:id/media` | It changes the library. `docs/TEST-SERVER.md` section 6b used it before |
| `POST`, `PATCH`, `DELETE /api/users` | They change the accounts of the server |
| `POST /api/items/:id/open-feed` | It makes a public address |
| The requests that make a backup | They write a large file |

A `GET` on a path that takes a `POST` only gives `404`. Therefore a `404` in
this document proves nothing about a `POST` on the same path. Every row above
names the method that the measurement used.
