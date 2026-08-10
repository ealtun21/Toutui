# T-24: what Audiobookshelf gives, and what Toutui takes

Date: 2026-08-11

This document compares the functions of an Audiobookshelf server with the
functions of this client. It gives the maintainer one list of the work that is
not done.

| What | Value |
|---|---|
| The server | Audiobookshelf 2.36.0 |
| The client | Toutui 0.7.0 (`Cargo.toml`) |
| The address of the server | `http://127.0.0.1:13399`, the sandbox of `docs/TEST-SERVER.md` |
| The user | `toutuitest`, of the type `root` |

`GET /status` gives
`{"app":"audiobookshelf","serverVersion":"2.36.0","isInit":true,...}`.

**Every measurement in this document comes from that sandbox.** No request went
to a server of a user. A row that says "not tested" holds no measurement, and
the reason is beside it.

The sandbox holds three libraries: `Books` with 9 items, `Podcasts` with 1
podcast of 3 episodes, and `Empty` with no item.

## 1. Two errors of the official reference

<https://api.audiobookshelf.org> does not agree with 2.36.0 in two places. A
measurement of both forms gives this:

| Path | Answer of 2.36.0 |
|---|---|
| `POST /api/session/:id/sync` | `200` |
| `POST /api/sessions/:id/sync` | `404` |
| `POST /api/me/item/:id/bookmark` | `200` |
| `POST /api/me/bookmarks` | `404` |

The reference gives the second form of each pair. Toutui uses the first form of
the first pair, therefore Toutui is correct. Read the reference with care, and
measure the path before you write it.

## 2. What Toutui asks the server today

This list comes from `src/api/` and from `src/logic/`. It is complete.

| Request | Where |
|---|---|
| `POST /login`, `GET /ping` | `src/api/server/auth_process.rs`, `src/api/client/probe.rs` |
| `GET /api/libraries` | `src/api/libraries/get_all_libraries.rs` |
| `GET /api/libraries/:id/items?limit=500&page=N&sort=&desc=&filter=` | `src/api/libraries/get_all_books.rs` |
| `GET /api/libraries/:id/filterdata` | `src/api/libraries/get_filter_data.rs` |
| `GET /api/libraries/:id/personalized` | `src/api/libraries/get_library_perso_view.rs` |
| `GET /api/libraries/:id/series?limit=500&page=N&sort=name` | `src/api/libraries/get_all_series.rs` |
| `GET /api/libraries/:id/collections?limit=&page=` | `src/api/libraries/get_lists.rs` |
| `GET /api/libraries/:id/playlists?limit=&page=` | `src/api/libraries/get_lists.rs` |
| `GET /api/items/:id` | `src/logic/playback/mod.rs` |
| `GET /api/items/:id/cover` | `src/ui/cover.rs` |
| `GET /api/items/:id/ebook` | `src/logic/reader/book.rs` |
| `GET /api/items/:id/file/:ino/download` | `src/logic/download/fetch.rs`, `src/player/engine/http_file.rs` |
| `POST /api/items/:id/play`, `POST /api/items/:id/play/:episodeId` | `src/api/library_items/play_lib_item_or_pod.rs` |
| `GET /api/me/progress/:id` | `src/api/me/get_media_progress.rs` |
| `GET /api/me/listening-stats` | `src/api/me/listening_stats.rs` |
| `PATCH /api/me/progress/:id`, `PATCH /api/me/progress/:id/:episodeId` | `src/api/me/update_media_progress.rs` |
| `POST /api/session/:id/sync`, `POST /api/session/:id/close` | `src/api/sessions/` |

Toutui calls 17 paths. The server has more than 100.

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
| `/` | Search |
| `s` | Show the series of the library (T-22) |
| `c` | Show the collections and the playlists (T-9) |
| `e` | Open the EPUB book of the item (T-10) |
| `D`, `X` | Get a local copy, and remove a local copy |
| `R` | Ask the server again |
| `F` | Send the position now (T-32) |
| `T` | Show the time that you listened (T-24) |
| `N` | Take a media away from Continue Listening, or put it back (T-24) |
| `C` | Show the chapters of the media that plays (T-24) |
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

The reader of an EPUB book takes the keys first, and it uses the same letters
for a different work: `j`/`k` a line, `Space`/`b` a page, `n`/`p` a chapter,
`t` the contents, `g`/`G` the start and the end, `s` sends the place, and `h`
leaves the book.

The variable `TOUTUI_NO_COVERS` stops the cover art. The variable
`TOUTUI_COVERS_IN_TMUX` asks the terminal inside tmux. The variable
`TOUTUI_AUDIO_DEVICE` names the sound device.

## 4. The table of the functions

`Yes` means that the user can do the work. `Half` means that a part operates.
`No` means that the client has nothing.

| Function | The server | Toutui | What is missing |
|---|---|---|---|
| **Sign in** | `POST /login` gives a token of 201 characters. `GET /status` gives `authMethods: ["local"]` and the fields of OpenID | Yes | OpenID. `GET /api/auth-settings` gives 14 fields of OpenID. A terminal cannot open a browser page of a provider with no work |
| **More than one server** | Not a function of the server | Yes | Nothing. `[[servers]]` holds the addresses, and the pool selects one |
| **List the libraries** | `GET /api/libraries` gives 3 libraries with `id`, `name`, and `mediaType` | Yes | Nothing |
| **Choose the library** | The client holds the choice | Yes | Nothing. `S` then the line "Library" |
| **The items of a library** | `GET /api/libraries/:id/items?limit=&page=` gives `results`, `total`, `limit`, `page` | Yes | Nothing. The client asks for pages of 500 (T-7) |
| **Sort the items** | `?sort=media.metadata.title&desc=1` changes the sequence. Measured: `desc=1` gives `Volume 3, Volume 2, Volume 1`, and no `desc` gives `A Long Test Book, Alice in Wonderland, Multi File Test Book` | Yes | Nothing. The key `f` gives seven fields for a library of books and three for a library of podcasts, and a line that changes the direction. The choice belongs to the account, therefore it stays after the program stops |
| **Filter the items** | `?filter=<type>.<base64>` gives `filterBy` in the answer. `GET /api/libraries/:id/filterdata` gives the values: 4 authors, 2 series, 0 genres, 0 tags, 0 narrators, 0 languages | Yes | Nothing. The key `f` gives the authors, the series, the genres, the tags, the narrators, the languages, the publishers, and the three values of the position |
| **Group a series in the list** | `?collapseseries=1` gives `collapseseries` in the answer | Half | The client makes the group itself, in `group_library` of `src/logic/library_view.rs`. The result is correct, and the server can do the same work |
| **The shelves of Home** | `GET /api/libraries/:id/personalized` gives 6 shelves for a book library: `continue-listening` (4), `recently-added` (9), `recent-series` (2), `discover` (2), `listen-again` (2), `newest-authors` (4). A podcast library gives `newest-episodes` (3), `recently-added` (1), `listen-again` (2) | Yes | The shelf `newest-authors` only. An author holds no media and no book, therefore a terminal can show nothing for that shelf. Every other shelf gives its name and its lines. A line of `recent-series` opens the books of the series |
| **Search** | `GET /api/libraries/:id/search?q=` gives six groups: `book`, `authors`, `series`, `narrators`, `tags`, `genres`. Measured: `q=Volume` gives 5 books, `q=Carroll` gives the author "Lewis Carroll" and no book | Half | The whole endpoint. `src/ui/tui.rs:884` filters the titles that the client holds already, with `to_lowercase().contains()`. The client finds no author, no narrator, no series, and no tag. It finds no word inside a description |
| **The series of a library** | `GET /api/libraries/:id/series?limit=&page=` gives `results` and `total`. `limit=0` gives an empty list, and not every series | Yes | Nothing. The key `s`, and one line for each series in the Library view |
| **One series** | `GET /api/series/:id` gives `id`, `name`, `nameIgnorePrefix`, `description` | Half | The description of the series. The client shows the name and the books only |
| **Collections** | `GET /api/libraries/:id/collections` and `GET /api/collections` give the collections | Half | The client reads and plays. It cannot make a collection, add a book, or remove a book |
| **Playlists** | `GET /api/libraries/:id/playlists` and `GET /api/playlists`. An entry holds `libraryItemId` and, for an episode, `episodeId` | Half | The same. The client reads and plays, and it changes nothing |
| **The position of a media** | `GET /api/me/progress/:id` gives `currentTime`, `progress`, `isFinished`, `hideFromContinueListening`, `ebookLocation`, `ebookProgress`, `lastUpdate` | Yes | Nothing. The client reads the position at the start and it writes the position (T-4) |
| **Mark as finished** | `PATCH /api/me/progress/:id` with `{"isFinished":true}` | Half | A key. The client sends this at the end of a playback only, in `update_media_progress2_book`. The user cannot mark a book that they will not finish |
| **Hide from Continue Listening** | The field `hideFromContinueListening` of `PATCH /api/me/progress/:id` | Yes | Nothing. The key `N`. A measurement on 2026-08-11 shows that the shelf of the server loses the media at once |
| **Open a session** | `POST /api/items/:id/play` gives `id`, `audioTracks`, `chapters`, `duration`, `playMethod` | Yes | Nothing |
| **Sync a session** | `POST /api/session/:id/sync` gives `200` | Yes | Nothing. The key `F` sends the position now (T-32) |
| **Close a session** | `POST /api/session/:id/close` gives `200` | Yes | Nothing |
| **The sessions of the user** | `GET /api/me/listening-sessions` gives `total`, `numPages`, `page`, `itemsPerPage`, `sessions`. It takes `itemsPerPage` and `page` | No | Everything. The user cannot see what they played, and when |
| **The sessions of the server** | `GET /api/sessions` gives the same shape for every user | No | Everything. This is for an administrator |
| **Bookmarks** | `POST /api/me/item/:id/bookmark` with `{"time":12,"title":"..."}` gives `200` and `{libraryItemId,time,title,createdAt}`. `DELETE /api/me/item/:id/bookmark/:time` gives `200`. `GET /api/me` gives the field `bookmarks` | No | Everything. The user cannot mark a place in a book, and the client shows no bookmark of a different client |
| **Play, pause, and stop** | The client does this work | Yes | Nothing. ` ` and `Y` |
| **Go forward and back** | The client does this work | Yes | Nothing. `p` and `u` |
| **Chapters** | `POST /api/items/:id/play` gives `chapters` with `start`, `end`, and `title` | Yes | Nothing. `P` and `U`, and the player shows the name of the chapter. `src/logic/playback/mod.rs:73` reads them |
| **The list of the chapters** | The same field | Yes | Nothing. The key `C` shows them, with a mark on the chapter that plays, and `l` goes to a chapter |
| **The speed** | The client does this work | Yes | Nothing. `O` and `I`, and the pitch does not change (T-19) |
| **The volume** | The client does this work | Yes | Nothing. `o` and `i` |
| **A timer for sleep** | Not a function of the server | No | Everything. The web player has this, and a person who listens in bed asks for it |
| **A queue of media** | Not a function of the server | No | Everything. The client plays one media, and it stops |
| **The cover art** | `GET /api/items/:id/cover` gives `200` and the bytes | Yes | Nothing. T-23. The panel stands beside the description, and a series shows its books |
| **The description** | `media.metadata.description` of the item | Yes | Nothing. `src/utils/html_text.rs` removes the HTML tags (T-13) |
| **Read an EPUB book** | `GET /api/items/:id/ebook` gives `200` and the whole file, and it takes a `Range` | Yes | The place of the web reader. The reader writes `ebookLocation` in its own form `toutui:C:L`, and it reads no EPUBCFI. The user then finds the chapter and not the line (T-10) |
| **The list of the ebooks of an item** | `media.ebookFile` of the item | Half | An item can hold one ebook only in this measurement. A PDF or a CBZ has no reader in the client |
| **Send an ebook to an e-reader** | `GET /api/emails/settings` gives `200`. The reference names the devices of an e-reader | No | Everything. This needs the settings of the email of the server |
| **List the podcasts** | The same endpoint as the books | Yes | Nothing |
| **The episodes of a podcast** | `GET /api/items/:id` gives `media.episodes` | Yes | Nothing. `l` on a podcast gives the episodes |
| **Play an episode** | `POST /api/items/:id/play/:episodeId` | Yes | Nothing |
| **Search a new podcast** | `GET /api/search/podcast?term=balzac` gives a list of 48, with `title`, `artistName`, `description`, `feedUrl`, `trackCount`, `cover` | No | Everything. The README names this as a future function |
| **Read a feed** | `POST /api/podcasts/feed` with `{"rssFeed":"..."}` gives `200` and the key `podcast` | No | Everything |
| **Make a podcast** | `POST /api/podcasts`. Not tested today: the request writes a new podcast in the library, and this measurement changed no library. `docs/TEST-SERVER.md` section 5 used it | No | Everything |
| **The server gets an episode** | `POST /api/podcasts/:id/download-episodes`. Not tested today, for the same reason. `GET /api/podcasts/:id/episode-downloads` gives `{"downloads":[]}`, and `GET /api/libraries/:id/episode-downloads` gives `{"queue":[]}` | No | Everything. The client copies an episode to its own disk, and it cannot tell the server to get an episode |
| **Look for a new episode** | `GET /api/podcasts/:id/checknew` gives `200` and the key `episodes` | No | Everything |
| **Empty the queue of the podcast** | `GET /api/podcasts/:id/clear-queue` gives `200` | No | Everything |
| **A local copy** | `GET /api/items/:id/file/:ino/download` gives the one audio file | Yes | Nothing. `D` and `X`, for a book and for one episode (T-1, T-11) |
| **Play with no server** | Not a function of the server | Yes | Nothing. The positions wait in `pending_progress`, and a task sends them (T-25) |
| **The archive of a whole item** | `GET /api/items/:id/download` gives `200` and a ZIP archive | No | The client does not use it, and it must not: T-1 says that the archive cannot play |
| **Change the metadata of an item** | `PATCH /api/items/:id/media`. `docs/TEST-SERVER.md` used it. Not tested today | No | Everything. The client reads, and it never writes |
| **Find the metadata of an item** | `POST /api/items/:id/match` gives `200`. With `{"provider":"google"}` it gives `{"warning":...}`, because the item has no title to match | No | Everything |
| **Make an M4B file** | `POST /api/items/:id/encode` of the reference. Not tested: the request starts a long job of ffmpeg on the server | No | Everything. See section 6 |
| **Write the metadata in the audio files** | `POST /api/items/:id/update-embedded-metadata` of the reference. Not tested, for the same reason | No | Everything. See section 6 |
| **Scan a library** | `POST /api/libraries/:id/scan` gives `200` | No | Everything. The user must open the web page to scan |
| **The authors of a library** | `GET /api/libraries/:id/authors` gives the key `authors`. `GET /api/authors/:id` gives `name`, `description`, `imagePath` | No | A view. The user cannot see the books of one author |
| **The narrators of a library** | `GET /api/libraries/:id/narrators` gives the key `narrators` | No | Everything |
| **The tags** | `GET /api/tags` gives the key `tags` | No | Everything |
| **The statistics of the library** | `GET /api/libraries/:id/stats` gives `totalItems`, `totalSize`, `totalDuration`, `numAudioTracks`, `largestItems`, `longestItems`, `totalAuthors`, `totalGenres` | No | Everything |
| **The statistics of the user** | `GET /api/me/listening-stats` gives `totalTime` 281, `today` 281, `days` `{"2026-08-10":281}`, `dayOfWeek` `{"Monday":281}`, `items` (a map of 2), and `recentSessions` (5) | Yes | Nothing. The key `T` shows the time of this day and the time in total, the last 14 days, the seven days of the week, the five media of the largest time, and the five last sessions |
| **The statistics of a year** | `GET /api/stats/year/2026` gives `numListeningSessions`, `totalListeningTime`, `topAuthors`, `topNarrators`, `topGenres`, and 8 more fields | No | Everything |
| **The account of the user** | `GET /api/me` gives `id`, `username`, `type` (`root`), `permissions`, `mediaProgress` (9 rows), `bookmarks`, `lastSeen` | Half | The client signs in and holds the token. It shows no permission and no type. The README says that `D` needs the permission `download`, and the client does not read that permission before it tries |
| **The permissions** | `GET /api/me` gives 9 permissions: `download`, `update`, `delete`, `upload`, `createEreader`, `accessAllLibraries`, `accessAllTags`, `accessExplicitContent`, `selectedTagsNotAccessible` | No | The client never reads them. A key that the account does not permit gives an error of the server and not a clear message |
| **The users of the server** | `GET /api/users` gives the key `users`. `GET /api/users/online` gives `usersOnline` and `openSessions`. `POST`, `PATCH`, and `DELETE` of the reference are not tested: they change the accounts of the server | No | Everything. See section 6 |
| **An RSS feed of an item** | `GET /api/feeds` gives `{"feeds":[],"minified":false}`. `POST /api/items/:id/open-feed` of the reference is not tested: it makes a public address | No | Everything. See section 6 |
| **A share of an item** | `GET /api/share/xx` gives `404` for an identity that does not exist, therefore the group answers | No | Everything. See section 6 |
| **The notifications** | `GET /api/notifications` gives `data` and `settings` | No | Everything. See section 6 |
| **The backups** | `GET /api/backups` gives `backups`, `backupLocation`, `backupPathEnvSet` | No | Everything. See section 6 |
| **The file system of the server** | `GET /api/filesystem` gives `posix` and `directories` | No | Everything. See section 6 |
| **The settings of the sign in** | `GET /api/auth-settings` gives 14 fields, and 12 of them belong to OpenID | No | Everything. See section 6 |
| **Live messages** | Audiobookshelf sends the changes over socket.io | No | Everything. A change of a different client comes to Toutui at the next `R` only |

## 5. What is worth building next

The sequence inside each group gives the value for the work.

### Small: a few hours each

1. **Search on the server.** `GET /api/libraries/:id/search?q=`. Change
   `src/logic/search/search_active.rs` and the filter at `src/ui/tui.rs:884`.
   The search today reads the titles that the client holds, therefore it finds
   no author and no series, and it cannot find a book that the client did not
   load. The work is small, and it removes the largest difference between the
   client and the web page.
2. **A key that marks a media as finished.** `PATCH /api/me/progress/:id` with
   `{"isFinished":true}`. `update_media_progress2_book` in
   `src/api/me/update_media_progress.rs:88` sends this body already. Add a key
   in `src/app.rs`. The user gives up on a book, and the book then leaves
   "Continue Listening".
3. ~~**The statistics of the user.**~~ **Done on 2026-08-11.** The key `T`,
   `src/api/me/listening_stats.rs`, `src/logic/stats.rs`, and
   `src/ui/stats_tui.rs`. The bar uses the blocks of Unicode, therefore the
   program needs no new dependency.
4. ~~**A choice of the sequence.**~~ **Done on 2026-08-11.** The key `f`, and
   `src/logic/sort_filter.rs`.
5. ~~**Hide a media from Continue Listening.**~~ **Done on 2026-08-11.** The
   key `N`, and `hide_the_media` beside `mark_the_media`.

### Medium: one or two days each

6. **Bookmarks.** `POST /api/me/item/:id/bookmark`,
   `DELETE /api/me/item/:id/bookmark/:time`, and the field `bookmarks` of
   `GET /api/me`. A new file `src/api/me/bookmarks.rs`, a key in `src/app.rs`
   during the playback, and a list in `src/ui/player_tui.rs`. A user of a long
   book needs a place to come back to.
7. ~~**The other shelves of Home.**~~ **Done on 2026-08-11.**
   `get_the_shelves` gives every shelf, and `src/logic/home_view.rs` makes the
   lines. The request did not change.
8. **Add a podcast.** `GET /api/search/podcast?term=`, then
   `POST /api/podcasts/feed`, then `POST /api/podcasts`. A new directory
   `src/api/podcasts/`, and a view for the results. The README names this
   function. The measurement gives 48 results for `term=balzac`.
9. ~~**Filter the library.**~~ **Done on 2026-08-11.** The same view as
   item 4, and `src/api/libraries/get_filter_data.rs`.
10. **The server gets an episode.**
    `POST /api/podcasts/:id/download-episodes`, and
    `GET /api/podcasts/:id/episode-downloads` for the state.
    `GET /api/podcasts/:id/checknew` finds a new episode. This is not the same
    work as the key `D`: `D` copies to the disk of the user, and this puts the
    file in the library of the server for every client.

### Large: a week or more each

11. **A timer for sleep.** No endpoint. The work is in
    `src/player/engine/worker.rs` and in `src/ui/player_tui.rs`: a time, a
    slow fall of the volume, and a stop. Every other client has this.
12. **The place of the ebook in the form of the web reader.** The reader
    writes `toutui:C:L` today. EPUBCFI needs a map from the position in the
    text to a path in the XHTML, in `src/logic/reader/position.rs`. Then the
    user reads on the telephone and continues in the terminal at the same
    line.
13. **A view of the authors.** `GET /api/libraries/:id/authors`,
    `GET /api/authors/:id`, and the filter of the items by the author. A new
    view, and the keys to move inside it.
14. **Live messages.** socket.io gives the changes of a different client. The
    tree holds no client of socket.io in pure Rust, therefore this needs a new
    dependency and an examination against the rule of section 6.

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
| The settings of the email | `GET /api/emails/settings` |
| The notifications | `GET /api/notifications` |
| The file system of the server | `GET /api/filesystem` |
| Upload a file | The requests of upload |

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
| Read a PDF and a CBZ | A page of a PDF is a picture with a layout. The cover art of T-23 shows one picture in a panel; a page of text as a picture is not readable in a terminal |
| The picture of an author | The same. It gives no information |
| A player of video | Audiobookshelf holds no video |

## 7. What this document did not measure

| Item | Why |
|---|---|
| `POST /api/podcasts` | It writes a new podcast in the library of the sandbox |
| `POST /api/podcasts/:id/download-episodes` | It gets files from the network and it writes them |
| `POST /api/items/:id/encode` | It starts a long job of ffmpeg |
| `POST /api/items/:id/update-embedded-metadata` | It changes the audio files |
| `PATCH /api/items/:id/media` | It changes the library. `docs/TEST-SERVER.md` section 6b used it before |
| `POST`, `PATCH`, `DELETE /api/users` | They change the accounts of the server |
| `POST /api/items/:id/open-feed` | It makes a public address |
| The requests that make a backup | They write a large file |

A `GET` on a path that takes a `POST` only gives `404`. Therefore a `404` in
this document proves nothing about a `POST` on the same path. Every row above
names the method that the measurement used.
