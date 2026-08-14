# The backlog of the fork

Date: 2026-08-09

The original project is archived. This document collects all the work that the
fork must do. It has three sources:

1. The open issues of the original repository.
2. The advice of an Audiobookshelf contributor in issue #35.
3. The tests of this fork against a real server on 2026-08-09.

Each item has a number for this fork. The column "Upstream" gives the number of
the original issue, if there is one.

## The work that is complete

| Id | Title | Commit |
|---|---|---|
| T-1 | The offline mode does not play a book | `6961659` |
| T-2 | A book that has many audio files: the length is correct now | `e84821d` |
| T-3 | The application sends `/progress` and `/sync` at the same time | `5088b55` |
| T-13 | The description shows the HTML tags | `14567c1` |
| T-2 | A book that has many audio files plays completely | sub-project 2 |
| T-5 | The token is not in the list of processes | sub-project 2 |
| T-6 | The token is not in the address of the file | sub-project 2 |
| T-8 | A change of the speed operates during the playback | sub-project 2 |
| T-19 | The pitch does not change with the speed | sub-project 2 |
| T-7 | The application asks for one page at a time | sub-project 3 |
| T-4 | The application gets the position from the server | sub-project 3 |
| T-16 | The application marks a media as finished at the end | sub-project 3 |
| T-12 | The repository has a Nix flake | sub-project 0 |
| T-11 | The application downloads a podcast episode | sub-project 5 |
| T-22 | The application shows the series of a library | sub-project 5 |
| T-9 | The application shows the playlists and the collections | sub-project 5 |
| T-26 | The key `G` in an empty list stopped the application | `597ca2d` |
| T-25 | The application does not start without the server | `bc9ceb0` |
| T-21 | The fork installs, updates, and removes its own program | v0.5.0 |
| T-28 | A tag with a capital letter gave "update available" for ever | `4221f2a` |
| T-30 | An answer with no length has a limit of size now | `ffe6a13` |
| T-29 | The update tests the proof of the origin of an archive | `f123f8b` |
| T-27 | Continuous integration builds the flake of Nix | `7f55e43` |
| T-14 | The program does not lose the configuration (examined) | this document |
| T-31 | The fork gives no bundle for macOS | this document |
| T-15 | The authentication does not fail at the first attempt (examined) | `6796d91` |
| `9bacac`, `86384e`, `dd9a649` | A playback loop reports its own playback only | `c82c9d8` |
| T-17 | The application plays an Opus file | `c342f50` |
| T-34 | A colour of the configuration file does not stop the program | `21aac71` |
| T-35 | Every playback releases the wait of the next playback | `e4b51c9` |
| T-33 | The application uses ratatui 0.30, crossterm 0.29, and `tui-input` | `8f5c938` |
| T-23 | The application shows the cover art | `35a7703` |
| T-22 | A series gives one line of the Library view | `ee36692` |
| T-32 | The key `F` sends the position at once | `a4904e0` |
| T-10 | The application reads an EPUB book | this release |
| T-31 | macOS has a way to remove the program | this release |
| T-36 to T-46 | The report of the user of 2026-08-10 | this release |

Sub-project 2 removed VLC. The application decodes the audio in the process
now. Therefore a book with many audio files plays completely, the token stays
in the memory of the process, and a change of the speed operates during the
playback.

## Priority 1: the application gives incorrect results

| Id | Upstream | Title | Sub-project |
|---|---|---|---|
| T-1 | — | The offline mode does not play a book | 1b |
| T-2 | #33 | A book that has many audio files does not play | 1b |
| T-3 | #35 | The application sends `/progress` and `/sync` at the same time | 1c |
| T-21 | — | `--update` installed the archived original project | 3 |
| T-4 | #37 | The application does not get the position from the server | 1c |

### T-1: the offline mode does not play a book

The application uses `GET /api/items/:id/download`. That endpoint gives a ZIP
archive for every book. The application gives the archive to VLC. VLC cannot
play an archive.

See `docs/superpowers/specs/2026-08-09-offline-download-design.md`. The
correction gets each audio file with its own request.

### T-2: a book that has many audio files does not play

Two users report this fault. One user says that the largest part of their
library has this format.

The application uses the length of the first audio file as the length of the
book. The user sees `14:56:04 / 14:01` and `6393%` remaining. The book does not
play after the first file.

The correction needs the same calculation as T-1: add the lengths of the files,
and calculate the start time of each file. `src/logic/download/plan.rs` has
this calculation now. The player must use it.

In a test library of 2056 books, 297 books have more than one audio file. The
largest book has 209 audio files.

A measurement of all 2056 books on 2026-08-10 gives the codecs. The library has
`mp3` in 6660 files and `aac` in 3408 files. It has no other codec. 1938 books
have chapters, thus 118 books have no chapter.

### T-3: the application sends `/progress` and `/sync` at the same time

An Audiobookshelf contributor gives this advice in issue #35:

- `/sync` changes the progress. It is the correct endpoint during playback.
- `/progress` is only for a command of the user. An example is "mark as
  finished".
- Two requests at the same time can make a race condition. Then an item stays
  in "Continue listening".
- `/progress` does not start a websocket message. Therefore other clients do
  not see the change.

The fault is in three places:

- `src/logic/handle_input/handle_l_book.rs:167`
- `src/logic/handle_input/handle_l_pod.rs:172`
- `src/logic/handle_input/handle_l_pod_home.rs:172`

This fault can be the cause of T-4 and of `known_bugs.md` `dd9a649`.

### T-4: the application does not get the position from the server

The user listens on a telephone. Then the user starts this application. The
application does not show the new position.

**The cause.** The table `listening_session` kept its row after the
application closed the session and sent the position. At the next start, the
application sent that old position again, before it opened the new session.
Therefore the position of the telephone was lost.

**The correction.** The application removes the row after it sends the
position. A row that stays means that the application stopped without a
correct exit, and in that condition the application must still send the
position one time.

A test on 2026-08-10 confirms the correction. The application played to 24303
seconds and stopped correctly. A different client then wrote 1234 seconds. The
application started again at 1234 seconds.

### T-21: `--update` installed the archived original project

`src/utils/clap.rs` ran the install script of the original repository, and
every address in that script names `AlbanDAVID/Toutui`. Therefore the command
built and installed the original program, and the user lost every correction
of the fork.

**The correction.** The fork makes its own releases from a tag on `main`.
`--update` receives the archive of its target, it compares the sum with
`SHA256SUMS`, and it moves the new binary. The program runs no file that it
receives. `--uninstall` writes the paths and deletes nothing.

The fork publishes no package to a registry. The three ways to install are
the script, `cargo install --git`, and the archives of the releases.

## Priority 2: security

| Id | Upstream | Title | Sub-project |
|---|---|---|---|
| T-5 | — | The token is visible in the list of processes | 2 |
| T-6 | #35 | The application sends the token in the address of the file | 2 |

### T-5: the token is visible in the list of processes

`src/player/vlc/start_vlc.rs:29` puts the token in a command line argument:

```rust
.arg(format!("{}{}?token={}", server_address, content_url, token.unwrap()))
```

Every user of the same computer can read this token with the command
`ps aux`. The token gives full access to the account.

The removal of VLC in sub-project 2 corrects this fault, because the
application then reads the file itself.

### T-6: the application sends the token in the address of the file

Newer versions of Audiobookshelf have a method that does not need the token in
the address. Issue #35 gives this advice. Sub-project 2 does this work.

## Priority 3: performance and correctness

| Id | Upstream | Title | Sub-project |
|---|---|---|---|
| T-7 | #35 | The application gets all items in one request | complete, `get_all_books.rs` |
| T-8 | #36 | A change of the speed needs a new start of the playback | complete, T-19 |
| T-17 | — | Play an Opus file | complete, `c342f50` |
| T-18 | — | Play a WMA file and an AWB file (the last two of 19) | complete, T-53 |
| T-20 | — | Remove the two dependencies that compile C | later |

### T-7: the application gets all items in one request — fixed

`?limit=0` told the server to send every item in one answer. A library with 10000
books then makes a very large answer, and that uses time, memory, and the
resources of the server. Upstream issue 35 gives this advice.

Measurement on 2026-08-09: a library of 2056 books gave 3.7 megabytes in 0.48
seconds. That is acceptable, and a larger library is not.

**`get_all_books.rs` asks for pages of `PAGE_SIZE` items now**, and `PAGE_SIZE` is
500. `wants_more_pages` stops at a page that is not full and at the number of items
that the server reports, and `MAX_PAGES` of 500 stops an endless loop for a server
that always gives a full page. `tests/pagination.rs` holds the rules.

### T-8: a change of the speed needs a new start of the playback — fixed

The user changed the speed, the indicator changed, and the sound did not change.
VLC was the cause: that program took the speed at its start only.

**The engine of sub-project 2 corrects it.** `PlayerCommand::SetSpeed` writes
`SharedSpeed`, an `AtomicU32` that `SpeedSource` reads **on the next sample**. The
playback does not start again, and WSOLA holds the pitch. See T-19.

**The measurement of 2026-08-11.** A run in tmux pressed `O` two times during a
playback. The panel gave `Speed: 1.00x`, then `Speed: 1.10x`, then `Speed: 1.20x`,
and the position went on between the three frames: 12:13, 13:27, and 14:40. No key
of the user started the playback again.

`tests/engine.rs` holds the sound of the change: `a_double_speed_gives_half_the_length`
and `a_double_speed_keeps_the_pitch` measure the frequency of a tone.

### T-18: a WMA file and an AWB file play now — the server gives the stream

Audiobookshelf accepts 19 forms of audio. symphonia reads 17 of them: it has no
decoder for AMR-WB (`awb`) and no reader for the container ASF (`wma`).

**T-53 closes this item, and it needed no decoder.** The engine gives the fault of
a file to the loop of the playback, and that loop asks the server for a stream of
the whole media. ffmpeg of the server reads every form.

**The measurement of 2026-08-11.** A book of the sandbox of `01 - Part 1.mp3` of
30 minutes and `02 - Part 2.wma` of 30 seconds:

```
[worker] the engine cannot open the track 2 of 2: ... It does not play wma and awb.
[play] no decoder of the program reads 02 - Part 2.wma. The program asks the server for a stream of the whole media.
[HlsFile] the stream holds 305 part(s). ... The audio is Mp3.
```

The player then showed `▶ 8:15 / 30:29` and the notice "The server makes the
stream of this media". The whole media played, and the WMA file with it.

**What a decoder of the program would add.** A playback of a WMA file with no work
of the server. `symphonia` has no reader of ASF today, and no crate of pure Rust
gives one. Therefore this item stays closed with the answer of the server.

### T-20: remove the two dependencies that compile C

**The rule of 2026-08-10.** The maintainer gave the rule its present form. A
dependency that compiles C when a person builds the program is acceptable, if
the binary that the release gives needs no library of the system. Pure Rust
stays the better answer, and the work must prefer it. A dependency that makes
the binary ask the system for a library is not acceptable: such a binary does
not run on a system that has no such library, and the candidates of v0.5.0
showed that fault with `libasound`.

Therefore T-20 is no longer a rule that stops work. It is an improvement, and
it stays open.

Two dependencies compile C today:

- `libsqlite3-sys`, from `rusqlite` with the feature `bundled`. It links the
  code into the binary, therefore the binary asks the system for nothing.
- `ring`, from `rustls` 0.23 through `reqwest` 0.12. The same is true.

`backtrace` names `cc` as a build dependency, but it compiles no C on this
target. The audio crates of sub-project 2 compile no C.

**Why the fork does not take reqwest 0.13.** 0.13 gives the feature of rustls
the name `rustls`, and that feature brings `aws-lc-rs`. That crate needs
`aws-lc-sys`, and that one needs cmake and a C compiler. Therefore a user who
builds with `cargo install --git` would need cmake, and the flake of Nix would
need it too. reqwest 0.12 with `rustls-tls` keeps `ring` and needs no cmake,
and it gives the same rustls 0.23. The fork stays on 0.12 until a provider of
pure Rust is ready.

Both answers need a crate that is not ready today. `turso` is a pre-release,
and `rustls-rustcrypto` is an alpha version. Issue 20 holds the details.

**The decision of the maintainer, 2026-08-11. This item waits, and it does not
stop any work.**

> **Both crates stay.** A later session looks again when `turso` is a release and
> `rustls-rustcrypto` is beta or better.

The reason: the binary of the release needs no library of the system, therefore
the rule above holds today. Each of the two answers touches a part of the program
where a fault costs the user a secret: `ring` carries the TLS of every request,
and `libsqlite3-sys` reads and writes the encrypted token of the user. **An alpha
version and a pre-release are not acceptable in those two places.** The gate of
every session keeps the measurement:

```
cargo tree -i openssl-sys        # must find nothing
cargo tree -i cc                 # must find libsqlite3-sys and ring only
```

## Priority 4: new functions

| Id | Upstream | Title | Sub-project |
|---|---|---|---|
| T-9 | #32 | Show the playlists and the collections | 5 |
| T-10 | — | Read an EPUB book in the application | 5 |
| T-11 | — | Download a podcast episode | 5 |
| T-12 | — | Add a Nix flake | 0 |
| T-22 | — | Show the series of a library | 5 |
| T-23 | — | Show the cover art with the Kitty protocol or Sixel | 5 |
| T-24 | — | Cover every function of Audiobookshelf | 5 |

### T-10: read an EPUB book in the application — complete

**The work, 2026-08-10.** The key `e` on an item that holds an ebook opens the
reader. `src/logic/reader/` holds the four parts: `book.rs` opens the file,
`render.rs` makes the lines, `position.rs` holds the place, and `session.rs`
holds the book while the user reads it. `src/ui/reader_tui.rs` draws it.

The keys: `j`/`k` a line, `Space`/`b` a page, `n`/`p` a chapter, `t` the table
of contents, `g`/`G` the start and the end, `s` sends the place, and `h` leaves
the book.

**A run against the sandbox, 2026-08-10.** The key `e` on "Alice in Wonderland"
gave the text of the book in the terminal, with the headings and the table of
the contents of the book. The reader went past the wrapper of the cover by
itself and started at chapter 2, as the design says. `Space` gave the next
page, `n` gave "CHAPTER I. Down the Rabbit-Hole", and `t` gave the fourteen
entries of the contents. The key `s` wrote `ebookLocation` `toutui:1:0` and
`ebookProgress` 0.0022 on the server, and it changed no position of the audio.

The text stands in the middle and it never becomes wider than 100 columns,
because a line of 200 columns is hard to read.

**The place of the user, 2026-08-11.** The reader opens a book where the user
stopped, and it sends the place by itself.

- The key `e` asks `GET /api/me/progress/:id` and it goes to that place.
- The reader sends the place when it changed and 30 seconds went by, and when
  the user leaves the book with `h`. It sends nothing while the user reads the
  same line.

A run against the sandbox: the place of the server was empty. The user read to
chapter 5 and line 5 and pressed `s`, and the server then held
`ebookLocation` `toutui:4:5` and `ebookProgress` 0.1579. The key `e` a second
time gave chapter 5 and the same first line of the screen.

**What is not there yet.** The reader reads no EPUBCFI of the web reader, and
it uses `ebookProgress` in that condition, as section 6.1 says. The user then
finds the correct chapter and not the correct line.

**The design, 2026-08-10.**
`docs/superpowers/specs/2026-08-10-epub-reader-design.md`. No code yet.

The measurements that the design rests on:

- **The crates.** `rbook` 0.7.10 for the container, and `html2text` 0.17.1 for
  the XHTML. Both are pure Rust: `cargo tree -i cc` and `cargo tree -i
  openssl-sys` find nothing. Two crates are not acceptable: `epub-parser` 0.3.4
  brings `bzip2-sys`, `lzma-sys`, and `zstd-sys`, and `iepub` 1.3.7 brings
  `zstd-sys`. `--no-default-features` does not remove them.
- **Why not the crate `epub`.** A zip archive of 2 megabytes that opens to 2
  gigabytes stops the whole process with both crates: the allocation fails,
  Rust calls `abort`, and `catch_unwind` cannot help. `rbook` has
  `ManifestEntry::copy_bytes`, which writes into a writer of the caller. With a
  limit of 8 megabytes, the same file gives an error and the program uses 5
  megabytes, and not 4102 megabytes. The crate `epub` has no such function.
- **The other eleven hostile files did nothing.** A path with `../..` inside
  the archive touches no file of the disk, and a billion laughs attack is
  inert, because html5ever, xml5ever, and quick-xml take no entity of a DTD.
- **The time.** One chapter of Moby Dick needs 3 milliseconds in a release
  build and 18 milliseconds in a debug build. 10000 nested `<div>` need 1.85
  seconds in a debug build, and the time grows with the square of the depth.
  Therefore the render runs on a task and it has a limit of time.
- **The server.** `GET /api/items/:id/ebook` gives the whole file and it takes
  a `Range`. `PATCH /api/me/progress/:id` holds `ebookLocation` as a text of
  the client and `ebookProgress` as a number, and it changes no position of the
  audio. The sandbox holds an EPUB now. See `docs/TEST-SERVER.md` section 6g.

### T-11: download a podcast episode

The application downloads a book with `src/logic/download/`. A podcast holds
many episodes, and each episode is a separate download. Therefore the download
has a key: the key of a book is the identity of the item, and the key of an
episode is the identity of the episode.

The key is the identity in the table `downloads`, in the table
`download_files`, in the map of the progress, and in the name of the
directory. The address of the file on the server still holds the identity of
the podcast.

A test on 2026-08-10 with the sandbox server downloaded one episode of three.
The file on the disk agrees with the file on the server, byte for byte. A
playback of that episode used the file on the disk, and a playback of a
different episode of the same podcast used the server.

### T-25: the offline mode

The first test of the download stopped the server. The application then showed
an empty screen, and the copy on the disk gave no help. Three parts correct
this:

1. **The start.** The first request of `App::new` tells the condition. An
   address that does not answer gives `ApiError::Unreachable` or
   `ApiError::Timeout`, and the application then makes its lists from the table
   `downloads`. A fault of the request, for example a token that is not valid,
   does not start the offline mode: the local copy gives no help there.
2. **The playback.** `play` tries the server first. If the server does not
   answer, `play_offline` reads the files, the length, and the position from
   the database. It needs no session on the server.
3. **The sync.** A position that the server did not take waits in the table
   `pending_progress`. A background task sends it every 30 seconds. The task
   examines the addresses of the server itself, because the probe task waits 60
   seconds and every address has the state `Down` after the offline mode.

**The rule of the merge.** A different client can write a newer position during
the offline period. The application asks for `lastUpdate` of the server and
compares it with the local time. The newer position wins. This is the same rule
as T-4.

**More than one server.** A user can have an account on more than one server,
and one server can have many addresses. The identity of a server is therefore
the name in the `[[servers]]` block, and not one address. The tables
`downloads` and `pending_progress` hold that identity, thus a position of one
server never goes to a different server. The pool still selects the address
that has the most importance.

**A trap of SQLite.** The first name of the column of the position was
`current_time`. That name is a keyword of SQLite, and a query gave the time of
the day. The row then did not agree with the type, and the application sent no
position at all. The name is `position_s` now, and a test guards it.

### T-9: show the playlists and the collections

The key `c` shows the collections and the playlists together. The collections
come first, because every user of the server sees them. A playlist belongs to
one user.

A playlist can hold a book and an episode of a podcast in the same list.
Therefore one entry holds the identity of the item and, for an episode, the
identity of the episode. The keys `l`, `D`, and `X` then use the correct kind.

A podcast library has no collection. It can have a playlist, thus the key `c`
operates in a podcast library also.

An episode gives its own title and its own length. The library item gives the
author, and a podcast holds that name in the field `author`, and not in the
field `authorName`.

### T-12: add a Nix flake

Pull request #38 of the original repository adds a flake for Nix users. The
original author closed it without a merge. Examine this work again.

### T-17: play an Opus file — complete, `c342f50`

The engine plays 17 of the 19 formats now. Symphonia reads the container, and
`opuscule` decodes each packet. The engine plays Opus in an OGG container, in a
Matroska container, in a WebM container, and in an MP4 container. WMA and AWB
stay outside; see T-18.

**A correction of the first measurement.** Issue 17 named `opus-decoder` 0.1.1,
and it said that the crate agrees with libopus. That measurement ran a release
build. Rust examines an arithmetic operation in a debug build only, therefore
the release build hid a fault.

A measurement on 2026-08-10 ran a debug build with 47 files:

| Crate | Result |
|---|---|
| `opus-decoder` 0.1.1 | stops the program on 13 of 47 files |
| `mousiki` 0.2.1 | stops the program on 15 of 47 files |
| `moosicbox_opus_native` 0.4.0 | `decode_float` holds `todo!()` |
| `opuscule` 0.2.0 | plays all 47 files |

The fault of `opus-decoder` is in `src/celt/vq.rs`, line 118: "attempt to shift
left with overflow". One file of `tests/fixtures/audio` gives that fault.

**The risk of `opuscule`, and the answer to that risk.** The crate is young: it
came on 2026-07-20, and it had 84 downloads on the day of this work. Its own
document says that an agent of artificial intelligence made most of the code
from the reference in C, and that the correctness rests on the test vectors of
RFC 8251.

Two properties make the risk acceptable. The crate holds `forbid(unsafe)`,
therefore a fault gives a wrong sample or a panic, and never damage of the
memory. And the source catches a panic of the decoder: the one track stops, and
the application continues. `ExpectedPanic` in `src/utils/exit_app.rs` tells the
hook of the panic to keep the terminal and the screen for such a panic.

The licence of `opuscule` is MPL-2.0. That licence agrees with GPL-3.0-or-later,
and symphonia already gives MPL-2.0 to the tree.

**The agreement with libopus.** The measurement compared the samples of the
whole path with libopus over 50 files: one channel and two channels, 6 to 128
kilobits each second, frames of 2.5, 10, 20, and 60 milliseconds, and four
containers. Every file agrees. The largest difference of one sample is 0.00002
of a full scale of 1.0, and no file has an offset of its start.

**Two faults of this work that the measurement found.**

1. The head of an Opus stream gives the number of samples that the stream skips
   at its start. The box `dOps` of MP4 writes the two bytes of that number with
   the byte of the largest value first, and an OGG container writes them with
   the byte of the smallest value first. The first code always read the byte of
   the smallest value first. The value 312 then became 14337, and the source
   removed 0.3 seconds of the audio of an MP4 file.
2. `codec_params.delay` of symphonia 0.5.5 is not that number. It gave 648 for a
   file whose head says 312, and 648 is the padding at the end of that file. The
   head has the highest importance now, and `delay` is the answer only for a
   stream that gives no head.

**One difference that stays.** Symphonia 0.5.5 does not remove the padding at
the end of an Opus stream. Therefore the source gives up to one frame of padding
after the audio: 20 milliseconds for the usual file, and 60 milliseconds for the
largest frame. The padding is the tail of the encoder, and it is not a click.

`tests/opus.rs` and `tests/formats.rs` hold the rules. The tests need no sound
card and no server.

### T-22: show the series of a library — complete, `ee36692`

The key `s` shows the series. The key `l` on a series shows its books, with
the number of each book first. The books come in the sequence of the series,
because a sort of the text gives `#10` before `#2`.

The endpoint `GET /api/libraries/:id/series` has an important difference from
the endpoint of the items: `limit=0` gives an empty list, and not every
series. Therefore the application always asks for a page of 500.

**The last part, 2026-08-10.** The Library view gives one line to a series now.
`src/logic/library_view.rs` holds the pure function `group_library`, and eight
tests examine it.

A run against the sandbox in a pseudo terminal of 150 by 40 shows the result.
The library holds eight books: two series of three books, one book that stands
alone, and one book of thirty minutes.

```
─────────────────────────────Library [4 items]──────────────────────────
➤ A Long Test Book
  The Test Chronicles [3 books]
  Second Series [3 books]
  Multi File Test Book
```

The key `l` on the line of the series gives:

```
───────────────────────The Test Chronicles [3 items]────────────────────
➤ #1 - The Test Chronicles Volume 1
  #2 - The Test Chronicles Volume 2
  #3 - The Test Chronicles Volume 3
```

The key `h` then gives the Library back, and not the list of the series. The
key `G` goes to the last of the four lines, and the key `j` after it goes to
the first.

**A trap of the test harness, measured 2026-08-10.** `ratatui-image` asks the
terminal what it can do. The last question of that group is a Device Status
Report, "implemented by all terminals", and it makes sure that the reader of
the crate does not wait for ever. A pseudo terminal with no model of a
terminal answers nothing. The reader then stays inside `read`, and it takes the
first key of the user. A test then loses its first key press and the result
looks wrong. A harness must answer `ESC [ 5 n` with `ESC [ 0 n`. A comparison
with the build of before this session showed the difference, therefore this is
a fault of the harness and not of the application.

### T-23: the cover art — complete, `35a7703`

**The work that landed, 2026-08-10.** The panel of the covers stands at the
right of the list and of the description in every view of media. `T-33` landed
first, therefore the project takes `ratatui-image` 11.0.6 and not 9.0.0.

```toml
ratatui-image = { version = "11.0.6", default-features = false, features = ["crossterm"] }
image = { version = "0.25", default-features = false, features = ["jpeg", "png", "webp"] }
```

`cargo tree -i cc` finds `libsqlite3-sys` and `ring` only, and `cargo tree -i
openssl-sys` finds nothing.

**The parts.** `src/ui/cover.rs`:

1. A task reads `GET /api/items/:id/cover`, with a limit of 8 megabytes, and it
   writes the bytes in a store of the process. The store keeps an item with no
   cover as `Missing`, therefore the application asks one time only. The store
   is not part of `App`, therefore the key `R` starts no request again.
2. The render makes a protocol of `ratatui-image` from the bytes one time.
3. `plan_covers` gives the rectangle of each cover. That function is pure.

**The measured result in a real terminal, 2026-08-10.** A debug build ran in a
pseudo terminal of 150 by 40 against the sandbox. A model of the terminal read
the colour of every cell, therefore the proof is a colour and not a picture of
a screen. The covers of the sandbox are one colour each, and
`docs/TEST-SERVER.md` tells how to make them.

| What | What the screen showed |
|---|---|
| One book, nothing plays | One cover of 40 by 21 cells, `rgb(255, 255, 255)`, the colour of the cover of that book |
| A series of three books | Three covers in a grid, cyan, magenta, and yellow, in the sequence of the series, 22 by 11 cells each |
| A book plays, the selection is a different book | Two covers: the cover of the book that plays, red, 42 by 19 cells, above the cover of the selection, green, 24 by 12 cells |
| A terminal of 80 columns | No colour of a cover at all. The text takes the whole width |

**Two faults that only a real process showed.** Both passed every test of a
pure function, and both stopped the whole application.

1. **A deadlock of the store.** The first form of `picture` held the read lock
   during a `match`, because a guard in the expression of a `match` lives to
   the end of that `match`. The arm for an unknown item called `request`, and
   `request` asked for the write lock on the same thread. The application drew
   one frame and then stopped for ever. The test
   `the_first_ask_for_an_unknown_cover_does_not_stop_the_thread` runs the call
   on its own thread and fails after two seconds.
2. **The picker gave the terminal away.** `Picker::from_query_stdio` reads the
   answer of the terminal on its own thread. That thread makes the terminal
   raw, and it gives the old condition back when it stops. A terminal that
   never answers stops that thread two seconds later. With the question before
   `ratatui::init`, the thread gave the condition of the shell back after the
   application made the terminal raw. `ICANON` and `ECHO` then stayed on for
   ever, and the application read no key at all. The question now comes after
   `ratatui::init`. `src/main.rs` holds the reason.

**T-24** holds the comparison with Audiobookshelf, and it names the functions
that the application does not have yet.

The text below is the examination and the design of 2026-08-10.

`ratatui-image` finds the protocol of the terminal
itself: the Kitty protocol, Sixel for `foot`, iTerm2, or blocks of Unicode.

**The version, 2026-08-10.** `ratatui-image` 11.0.6 asks for `ratatui ^0.30.1`,
and this project holds ratatui 0.29 because of T-33. **`ratatui-image` 9.0.0 asks
for `ratatui ^0.29`, and it holds every protocol.** Therefore T-23 needs no
upgrade of ratatui, and T-23 does not wait for T-33.

**The features, 2026-08-10.** The features `chafa-dyn` and `chafa-static` need
the C library chafa. The project must not use the default features. This gives a
build with no new C:

```toml
ratatui-image = { version = "9.0.0", default-features = false, features = ["crossterm"] }
image = { version = "0.25", default-features = false, features = ["jpeg", "png", "webp"] }
```

A measurement on 2026-08-10 added those two lines and read the tree: `cargo tree
-i cc` still finds `libsqlite3-sys` and `ring` only, and `cargo tree -i
openssl-sys` finds nothing. The crate `image` needs its own line, because
`ratatui-image` asks for `image` with the feature `png` only. A cover of
Audiobookshelf is a JPEG file or a WebP file. The feature `image-defaults` also
gives those two, and it adds decoders that a cover never needs.

**The design that the user chose on 2026-08-10.**

1. The cover stands beside the description, and it is always visible. It does not
   wait for a key.
2. The cover is generous, and it is not small.
3. A series shows more than one cover, so that the view of the series looks
   better than a view of one book.
4. The cover of the media that plays is larger than the cover of the selection.

**The work that this design needs.**

- `GET /api/items/:id/cover` gives the bytes. The request needs the token.
- The render of the application is not async. Therefore a task must read the
  cover, and the render must take the answer from a channel. The application must
  hold a cover that it read before, so that a move of the selection reads no
  cover a second time. It must also hold the items with no cover, so that it asks
  for such an item one time only.
- `Picker::from_query_stdio` asks the terminal for the protocol and for the size
  of the font. That call needs a real terminal, and `Picker::halfblocks` is the
  answer for a terminal that does not answer.
- `StatefulImage` keeps the form of the image inside the area that it gets.
  Therefore the layout gives a generous area, and the widget does the rest.
- A narrow terminal has no width for a cover and a text. The cover must go away
  below a width that the code names.

**T-24 is complete: `docs/T-24-coverage.md`.** That document compares this
program with an Audiobookshelf 2.36.0, function by function. Toutui calls 15
paths, and the server gives more than 100.

The five that the document names as the next work:

1. The search of the server, `GET /api/libraries/:id/search?q=`. The program
   looks in the titles of the page that it holds, with `contains`. The server
   also finds an author, a series, a narrator, a tag, and a genre. A search for
   "Carroll" gives the author on the server, and nothing in this program.
2. ~~A key that marks a media as finished.~~ **Complete.** The key `M` marks
   the selected media as finished, or as not finished. The task asks the server
   for the condition first and it sends the opposite, therefore one key does
   the right work in every view. A measurement on 2026-08-11 shows that
   `isFinished: false` also puts `currentTime` and `progress` back to 0, and
   the message of the program says so.
   `tests/the_mark_of_finished_against_the_sandbox.rs` marks a book and marks
   it back against a real server.
3. The statistics of the user, `GET /api/me/listening-stats`.
4. The other five shelves of the personalized view. The program asks for that
   view and it keeps one shelf of six.
5. The sort and the filter of a library.

**A fault that the comparison found, and this release corrects.** The program
matched the shelf of "Continue Listening" on its name. A name is a text for a
person, therefore a server that gives it in a different language would give an
empty Home view, with no error at all. The program matches the identity
`continue-listening` now, and it uses the name only when the server gives no
identity.

## Priority 5: small faults

| Id | Upstream | Title |
|---|---|---|
| T-13 | — | The description shows the HTML tags |
| T-14 | — | The application loses the configuration after an update (`255b86`). Examined on 2026-08-10: this fault does not occur in the fork. |
| T-15 | — | The authentication fails at the first attempt (`4b3045`). Examined on 2026-08-10: this fault does not occur in the fork. |
| T-16 | — | "Mark as finished" does not always work (`2d358c53`) |

## Faults that did not occur again

| Id | Title | Result |
|---|---|---|
| `3f729c` | Slow start with a large library | The first screen came after 0.4 seconds with 2056 books. |

## The sequence of the work

| Sub-project | Contents | Items |
|---|---|---|
| 0 | Tools, continuous integration, style | done, T-12 |
| 1 | The API client and more than one address | — |
| 1b | The offline download | T-1, T-2 |
| 1c | The correct use of the sync endpoints | T-3, T-4 |
| 2 | The audio engine, and the removal of VLC | T-5, T-6, T-8 |
| 3 | Robustness, pagination, tests | T-7 |
| 5 | New functions | T-9, T-10, T-11 |

## The work that T-21 left

T-21 gave the fork its own way to install, to update, and to remove. The
examination of that work found these items. No item stops a release, and each
one is small.

| Id | Title | Where |
|---|---|---|
| T-27 | Continuous integration does not build the flake of Nix | `.github/workflows/ci.yml` |
| T-28 | A tag with a capital letter gives "update available" for ever | `src/utils/check_update.rs` |
| T-29 | Nothing looks at the proof of the origin of an archive | `install.sh`, `src/update/install.rs` |
| T-30 | An answer with no length has no limit of size | `src/update/install.rs`, `install.sh` |
| T-31 | macOS has no way to remove the program | `macos/`, `src/utils/clap.rs` |

### T-27: continuous integration does not build the flake of Nix

`flake.nix` and `flake.lock` are in the repository, and the README tells the
user to run `nix build`. No job builds them. Therefore the flake can stop to
operate and every test stays green.

A build on the machine of the development is not possible now: the command
`nix` is present, the directory `/nix/store` is absent, and the service
`nix-daemon` does not run.

**The work.** Add a job that runs `nix flake check` to `ci.yml`.

**The result of 2026-08-10.** The job runs `nix flake check --all-systems` and
`nix build`. Run 31395406233 gives the evidence of the first form of the job:
`nix build` compiled the program and ran 224 tests of the library and every
test of integration inside the sandbox of Nix.

The job then found two faults of the flake.

1. `apps.default` came from `flake-utils.lib.mkApp`, and that function gives no
   `meta`. `nix flake check` gives a warning for an app that has none. The app
   takes the `meta` of the package now.
2. The flake gave `pkgs.darwin.apple_sdk.frameworks` to `buildInputs` on macOS.
   nixpkgs removed that attribute, and the pinned revision throws: "darwin.
   apple_sdk has been removed as it was a legacy compatibility stub". Therefore
   `nix build` on macOS did not operate at all. No test on Linux can find this
   fault, because `optionals` reads its list only when the condition is true.
   `--all-systems` reads the outputs of macOS as well, and that is the answer.
   macOS needs no input for the audio now: `stdenv` gives the SDK of Apple, and
   that SDK holds AudioUnit and CoreAudio.

3. `flake-utils.lib.eachDefaultSystem` names `x86_64-darwin`, and nixpkgs 26.11
   dropped support for that system: it throws "Nixpkgs 26.11 has dropped
   support for x86_64-darwin". Therefore the flake could not evaluate for a Mac
   with a processor of Intel. The flake names its three systems now:
   `x86_64-linux`, `aarch64-linux`, and `aarch64-darwin`. A user of such a Mac
   can still use `install.sh` or `cargo install --git`, because the archive of
   macOS holds a universal binary. `release.yml` does not use Nix, therefore
   the releases do not change.

The faults 2 and 3 are the exact condition that T-27 names: the flake stopped
to operate for a system, and every test stayed green. Both faults concern
macOS, and no test on Linux could find either one.

### T-28: a tag with a capital letter gives "update available" for ever

`src/utils/check_update.rs` removes the letter `v` from the front of the tag
with `trim_start_matches('v')`, and that function looks at the case of the
letter. A tag `V0.5.0-beta` therefore keeps its first letter, the comparison
with the version of the build never agrees, and the message stays on the
screen after the user updates.

**The work.** Remove the letter without the case, and add a test.

### T-29: nothing looks at the proof of the origin of an archive

The workflow of release runs `actions/attest-build-provenance`, and thus each
archive has a proof. Neither `install.sh` nor `--update` looks at that proof.
The two compare the sum SHA-256 only, and that sum comes from the same
release. Therefore the comparison finds a download that stops, and it does not
find a release that a different person made.

**The work.** Use `gh attestation verify` when the command `gh` is present.

### T-30: an answer with no length has no limit of size

`receive` in `src/update/install.rs` refuses an answer whose header
`Content-Length` is more than 200 MB. An answer with no such header goes into
the memory with no limit, and only the limit of 120 seconds stops it.
`install.sh` has the same fault.

**The work.** Count the bytes as they arrive, and stop at the limit.

### T-31: the fork gives no bundle for macOS

`macos/Info.plist` and `macos/launch.command` described a bundle of an
application. `install.sh` did not install that bundle, therefore `--uninstall`
could not name it. A user of macOS who made the bundle by hand got an
incomplete list.

**The decision of 2026-08-10: the fork gives no bundle.** The maintainer chose
this answer. The reasons:

1. No part of the project used the two files. `install.sh` did not write the
   bundle, `release.yml` did not put it in an archive, and `--uninstall` could
   not name it. Only `Info.plist` named `launch.command`.
2. The two files did not agree with the installation. `launch.command` opened
   `$HOME/.cargo/bin/toutui`, and `install.sh` writes `/usr/local/bin/toutui`.
3. A bundle of an application gives one thing to a program of the terminal: an
   icon that opens Terminal. `install.sh` and the binary in `/usr/local/bin`
   give macOS every other function.
4. No machine of the development runs macOS. A bundle that nobody runs is the
   fault that the candidates of v0.5.0 showed three times: every test was
   green, and the installation still did not operate.

**The work that is complete.** The fork removed the two files. `--uninstall`
takes its list from `uninstall_paths` in `src/utils/clap.rs` now, and that
function is pure. Therefore a test gives the paths of macOS and the paths of
Linux, and it confirms that each list is complete and that no list names a
bundle.

**What no test on Linux can show.** These tests give the paths of macOS as
text. They do not run on macOS, and this machine cannot run macOS. A user of
macOS must confirm the list of `--uninstall` on that system.

**The removal itself, 2026-08-10.** The decision above closed the bundle. The
title of T-31 names the removal, and that part stayed open. The examination
found three gaps, and macOS had all three.

1. `--uninstall` wrote the paths and no command. The user composed `rm`
   himself, and he guessed when the path needs `sudo`. `/usr/local/bin/toutui`
   needs it, and `~/Library/Preferences/toutui` and `~/.local/share/toutui` do
   not.
2. `--uninstall` wrote two directories and no contents. The database
   `db.sqlite3`, the secret key `.env`, the log `toutui.log`, and the downloads
   are inside those two directories, and the user did not see them.
3. A user of macOS who cannot run the binary got nothing. Two conditions give
   that state: the binary is already absent and the configuration is not, and a
   browser received the archive of the release. macOS then puts the attribute
   `com.apple.quarantine` on the files, and Gatekeeper stops the program.

**The work that is complete.** `uninstall_paths` is `uninstall_plan` now. That
function gives the name, the path, the contents, and the command of each thing
that an installation makes. The function stays pure, and it deletes nothing.
`macos/uninstall.sh` gives the same list with no binary, and it deletes nothing
as well. Eleven tests in `src/utils/clap.rs` guard the two: the paths of each
system, the rule of `sudo`, the quotes of a path with a space or with a single
quote, the contents of the message, and the two lists that must agree. One test
reads `macos/uninstall.sh` and confirms that no line of that script removes a
path.

**What no test on Linux can show.** The tests give the paths of macOS as text.
This machine cannot run macOS. A user of macOS must run `toutui --uninstall`
and the script, and they must confirm that the two lists agree.

### T-14: the program does not lose the configuration

T-14 says that the program loses the configuration after an update. The first
plan of T-21 changed the name of the program, and it made the program copy the
old directory of configuration. That copy closed T-14 as a result, and not as
an examination. The maintainer then kept the name, therefore no copy existed
and T-14 needed its own examination. That examination came on 2026-08-10.

**The cause in the original project.** `hello_toutui.sh` merged
`config.example.toml` into the configuration of the user at every
installation. The second loop of that merge reads `$pseudo_escape_line` at
line 471, and no line of the script gives that name a value. The test is
therefore `grep -E "^"`, that pattern agrees with every line, and the loop
added no line of the user that `config.example.toml` does not name. The merge
also wrote the file again from the text of the example. Thus the comments and
the sequence of the example replaced those of the user, and every option that
the example does not name went away.

**The fork.** `install.sh` writes `config.toml` only when that file is absent,
and it merges nothing. It writes `.env` only when that file is absent, thus the
secret key stays and every stored token stays readable. `--update` moves one
file: the binary. `src/config.rs` reads the configuration and never writes it.

**The measurement of 2026-08-10.** A test installed the program with a local
host, changed the colour to `magenta`, added an option that the example does
not name, and installed a newer release whose `config.example.toml` had
different contents. The file of the user did not change, byte for byte, and the
secret key did not change. The test
`the_update_does_not_touch_the_configuration` in `tests/update.rs` guards the
same rule for `--update`.

Therefore T-14 does not occur in the fork.

## The release 0.6.0

**The candidate `v0.6.0-rc.1`, 2026-08-10.** The user asked for a candidate
before the release. The tag is `v0.6.0-rc.1`, plain semver, because a name that
semver cannot read gives "an update is available" for ever. `Cargo.toml` holds
the same text, because the workflow of the release refuses a tag that does not
agree with it.

The workflow marks a tag that holds a hyphen as a pre-release. `/releases/latest`
gives no pre-release, therefore a user of v0.5.0 sees no message. A measurement
after the release confirms it: that address still gives `v0.5.0`.

The proof of the archives:

| What | Result |
|---|---|
| `sha256sum -c SHA256SUMS` | Every one of the five files: OK |
| `gh attestation verify` on the three archives | Every one: the workflow `release.yml` of `ealtun21/Toutui`, at the commit `e6413343` |
| The same command on `config.example.toml` | `HTTP 404`, therefore the test is real and not empty |
| `./toutui --version` of the published archive | `toutui 0.6.0-rc.1` |
| The published binary against the sandbox | The library came, and the cover of "Alice in Wonderland" drew with its own colours |

**The release `v0.6.0`, 2026-08-10.** The candidate is a pre-release, and
`/releases/latest` gives no pre-release. Therefore `toutui --update` could not
reach it, and the user asked for the release itself.

| What | Result |
|---|---|
| `/releases/latest` | `v0.6.0` |
| `sha256sum -c SHA256SUMS` | Every one of the five files: OK |
| `gh attestation verify` on the three archives | Every one: verified |
| **`--update` from a build of 0.5.1** | "The proof of the origin is correct. The workflow of ealtun21/Toutui made this archive." Then: "Version 0.6.0 is now installed. The version before it was 0.5.1." The binary then answers `toutui 0.6.0` |

That last line is the whole path: the program found the release, it received the
archive, it examined the proof of the origin and the sum, and it moved the new
binary over itself.

### Two small items of T-24 — complete, 2026-08-11

**The program reads the permissions of the account.** `GET /api/me` gives nine
permissions. The program read none of them, therefore the key `D` on an account
that may not download gave the error of the protocol of the server. The key now
says "Your account cannot download a media. Ask the person who holds the
server." An absent permission means "yes", therefore a server that names fewer
permissions does not stop a user.

**The words of the search start empty.** The program wrote two spaces in
`search_query` at the start. That value stood for "no words", and only the
order of the views kept it from reaching the screen. The value is an empty text
now.

### The statistics of the user — complete, 2026-08-11

The README named "Add stats" as a future function, and `docs/T-24-coverage.md`
gave it the largest value of the small items.

`GET /api/me/listening-stats` gives every number in one answer, therefore the
program sends one request and it adds no dependency. A measurement against the
sandbox on 2026-08-11 gives the shape:

| Field | What |
|---|---|
| `totalTime` | 281 |
| `today` | 281 |
| `days` | `{"2026-08-10": 281}` |
| `dayOfWeek` | `{"Monday": 281}` |
| `items` | **a map**, and not a list. The key is the identity of the media |
| `recentSessions` | 5 rows with `displayTitle`, `date`, and `timeListening` |

The key `T` opens the view. The program asks the server at each press, because
the numbers change while the user listens.

**Three points of the design.**

1. **`items` is a map, and a map has no sequence.** `top_items` sorts by the
   time and then by the title. Two media of the same time therefore keep one
   place, and the list does not move from one frame to the next one.
2. **The bar needs no dependency.** `bar` makes the mark with the blocks of
   Unicode, and it uses a part of a block for the remainder. A value above
   zero always gives one mark: a bar of nothing would say that the user played
   nothing on that day.
3. **`days` is a `BTreeMap`.** A date of the form `2026-08-10` goes in the
   sequence of the time when it goes in the sequence of the letters, therefore
   the program needs no calendar and no new crate.

**A fault that only the real answer showed.** The first build gave the name of
a line ten columns. A date takes ten columns, therefore the date and the bar
stood together with no space:

```
2026-08-10██████████████████████████████  4 min 41 s
```

The name now takes eleven columns. The measurement against the sandbox found
this, and no test of the tests before it could: every name of that test was a
day of the week, and the longest one, `Wednesday`, takes nine columns.

`tests/the_statistics_against_the_sandbox.rs` holds the measurement. It carries
`#[ignore]`, because it needs a server.

### The other shelves of the Home view — complete, 2026-08-11

The program asked `GET /api/libraries/:id/personalized`, and it kept the shelf
`continue-listening` only. The other five shelves came in the same answer, and
the program threw them away. The request did not change with this work.

A measurement against the sandbox on 2026-08-11:

| Library | The shelves of the server |
|---|---|
| Books | `continue-listening` (4 media), `recently-added` (9), `recent-series` (2 series), `discover` (2), `listen-again` (2), `newest-authors` (4 authors) |
| Podcasts | `newest-episodes` (3 episodes), `recently-added` (1 podcast), `listen-again` (2 episodes) |

**A fault that the measurement showed: the Home view of a library of podcasts
was empty.** That library gives no shelf `continue-listening`. The program
kept that shelf only, therefore the view held nothing and it said nothing. The
view now shows "Newest Episodes" and "Listen Again".

**Two shelves give no line, and that is correct.** `newest-authors` holds an
author, and an author has no media and no book. `recently-added` of a library
of podcasts holds a podcast and not an episode, and the Home view plays an
episode. A shelf that gives no line gives no name, therefore the user never
reads a name with nothing below it.

**A fault of the lists that no measurement showed yet.** The Home view holds
six lists of the same length, and the screen reads them by one number. The
functions that made those lists did not agree on what makes a value:
`collect_ids_cnt_list` pushed for every entity, and the five other functions
pushed for an entity that holds a media and a metadata only. With the shelf
`continue-listening` alone, every entity held a media and the lists agreed. A
shelf of series holds an entity with no media. That entity would have moved
one list against the others, and the screen would have shown the title of one
book beside the author of a different book. Every function of
`collect_personalized_view.rs` and of `collect_personalized_view_pod.rs` now
walks one iterator, and a test holds each one to the same length.

**The lines.** `src/logic/home_view.rs` makes them, and it is pure:

- `HomeRow::Shelf` is the name of a shelf. The user cannot select it, and the
  keys `j`, `k`, `g`, and `G` go over it.
- `HomeRow::Media` holds the place of the media in the lists of the view. That
  place counts the media, and it does not count a name and does not count a
  series.
- `HomeRow::Series` holds the place of the series in `App::series`. The key
  `l` opens the books of that series, in the same way as the Library view.
  A series that the program does not hold gives no line, because the view of
  the books reads that list.

`App::series_from_library` was a yes or a no. The Home view opens a series
too, therefore that field is now `App::series_from`, a view. The key `h` goes
back to the view that opened the series.

### The sequence and the filter of a library — complete, 2026-08-11

`GET /api/libraries/:id/items` takes `sort`, `desc`, and `filter`, and the
program sent none of them. A library of 2056 items came in one sequence only,
and the user could not show the books of one author.

The key `f` opens the view. A choice writes the account in the database and
the program then makes the application again, in the same way as the key `R`:
every list of the library comes from one request, therefore a new sequence
needs a new request. The user comes back to the Library view, and not to the
Home view.

**Measurements against the sandbox on 2026-08-11.**

| Request | Answer |
|---|---|
| `?sort=media.metadata.title` | `A Long Test Book, Alice in Wonderland, Multi File Test Book` |
| `?sort=media.metadata.title&desc=1` | The other direction |
| `?sort=addedAt` | `Multi File Test Book` first, therefore the oldest first |
| `?filter=authors.<base64 of the identity>` | 1 book of 9 |
| `?filter=progress.ZmluaXNoZWQ=` | 2 books |
| `GET /api/libraries/:id/filterdata` | 4 authors, 2 series, and no genre, tag, narrator, language, or publisher |

**A trap of the server: `?sort=bogus.field` gives `200`.** The answer holds
`sortBy: "bogus.field"`, and the sequence is then not specified. Therefore the
program offers the seven fields that it measured, and no other field. A value
of the database that this build does not know goes away before the request.
`src/logic/sort_filter.rs` holds that list and the test of it.

**base64 with no crate.** The filter is `<type>.<base64 of the value>`. The
rule of base64 is short, and T-20 asks for pure Rust. `encode_base64` is
twenty lines, and a test holds it to the value that the server gave.

**A trap of the harness of the pseudo terminal, and not of the program.** The
key `R` looked as if it stopped the program: the screen never changed again.
The measurement showed that `terminal.clear()` of ratatui asks the terminal
`ESC [ 6 n`, the report of the place of the cursor. The harness answered
`ESC [ 5 n` only, and the program then stopped with "The cursor position could
not be read within a normal duration". A harness of a pseudo terminal must
answer **both** questions:

| Question | Answer |
|---|---|
| `ESC [ 5 n` | `ESC [ 0 n` |
| `ESC [ 6 n` | `ESC [ 1 ; 1 R` |

This is the same family as the trap of `ratatui-image` of the handover of
2026-08-11. A real terminal answers both, therefore no user saw this.

### Two small items of T-24 — complete, 2026-08-11 (v0.7.1)

**The key `N` takes a media away from Continue Listening.** The field
`hideFromContinueListening` of `PATCH /api/me/progress/:id` does the work. A
measurement against the sandbox on 2026-08-11 shows that the shelf
`continue-listening` of `/personalized` loses the media at once, and that it
holds the media again after the second press. `hide_the_media` reads the state
first, therefore the key is a change of the state and not one direction only.
`tests/the_shelf_against_the_sandbox.rs` holds that measurement.

**The key `C` shows the chapters.** The engine held the chapters already: the
keys `P` and `U` use them. The state of the engine now carries the list, and
`src/logic/chapters.rs` makes the lines. `l` sends `SeekTo(start)`.

The state takes the chapters when the identity of the playback changes, and
not at each tick. A copy at each tick would copy the name of every chapter
twenty times each second.

`chapter_at` gives the **last** chapter for a position after the end of the
last chapter. The end of the last chapter can stand before the end of the
audio, and a mark that goes away at the end of a book would look wrong.

**A trap of the measurement: the device `null` played a book of 30 minutes in
two seconds.** The view of the chapters needs a media that plays, therefore
the harness must press the key inside that time. The answer is one write of
two keys: `l` starts the playback and the space pauses it at once. The
position then stops, and the view stays.

### Bookmarks — complete, 2026-08-11 (v0.7.2)

A user of a long book needs a place to come back to. Audiobookshelf holds that
place for each user, therefore a bookmark of the telephone stands in the
terminal.

Measurements against the sandbox on 2026-08-11:

| Request | Answer |
|---|---|
| `POST /api/me/item/:id/bookmark` with `{"time":42,"title":"..."}` | `200`, and `{libraryItemId,time,title,createdAt}` |
| The same request, with the same time | `200`. The name changes, and `createdAt` does not. **No second line comes.** |
| `DELETE /api/me/item/:id/bookmark/:time` | `200` |
| The same delete again | `404` |
| `GET /api/me` | the field `bookmarks`, of every media together |

**The time is the key of a bookmark.** The address of the delete names the
second, therefore the program writes a whole number and it keeps that number.
`whole_seconds` does that work, and a test holds it.

The keys: `b` writes a place while a media plays, and it asks for a name; `V`
shows the list; `l` goes to a place; `X` removes one. Inside that view `X`
does not remove a local copy, and that is the one place where a key changes
its work with the view.

**`convert_seconds` was not correct for a place.** That function rounds to the
minute. Two bookmarks at 12 minutes 30 seconds and at 12 minutes 45 seconds
would then show one text, and the user could not tell them apart. `clock` of
`src/utils/convert_seconds.rs` writes `MM:SS` and `H:MM:SS`, and the view of
the chapters uses it also.

**A question that takes a text.** `src/logic/prompt.rs` holds it. The search of
the key `/` keeps its own file, because it does more than take a text: it
changes the view and it asks the server.

### A timer for sleep — complete, 2026-08-11 (v0.7.3)

The server holds no timer, therefore the work is in the client. Every other
client of Audiobookshelf has it, and a person who listens in bed asks for it.

The key `t` moves through the choices: 5, 10, 15, 30, 45, and 60 minutes, the
end of the chapter, and then off. One key stops the timer, therefore the user
needs no second key.

**The engine did not change.** The loop of the program runs at each frame and
it holds the handle, therefore it sends `SetVolume` during the fall and
`Pause` at the end. A timer inside the worker would have needed a new command
and a new state.

**The timer measures the time of the clock.** A user who says "in 30 minutes"
means the clock, and a speed of 2.0 must not change that. The choice "the end
of the chapter" reads the book, therefore `clock_time_of` divides by the
speed.

**A pause is not a stop.** A user who pauses a book and comes back keeps the
timer. A playback that stopped, and a different media, stop it: the user asked
for sleep during that book.

**The decision is pure.** `action_for` takes the timer, the state of the
engine, and the time, and it gives one of four answers. The method of the
application is then six lines of glue. That shape was necessary here: the
device `null` plays a book of 30 minutes in three seconds, therefore no
harness can wait five minutes for a real fall of the volume. The measurement
in a real process shows the message and the time in the player (`💤 4:59`),
and the tests hold the whole life of a timer.

### Add a podcast — complete, 2026-08-11 (v0.7.4)

The README named this function. Three requests do the work, and every one is
measured against the sandbox on 2026-08-11:

| Request | Answer |
|---|---|
| `GET /api/search/podcast?term=balzac` | 48 answers. **`limit=3` gave 48 also**, therefore the program cuts the list itself |
| `POST /api/podcasts/feed` with `{"rssFeed":"..."}` | `200`, and `podcast` with `metadata` and `episodes` |
| `POST /api/podcasts` | `200`, and the new item. A second add of one podcast gives `400` |

**The server asks iTunes for the search.** The search therefore needs the
network of the server, and not the network of the user. The sandbox in podman
has that network.

**The client gives the path of the directory, and the title comes from the
network.** A title can hold `/`, and a title can be `..`; the server would
then write outside the folder of the library. `directory_of` keeps a letter,
a number, a space, a dash, and an underscore, and it removes every other
character. A test gives it `../../etc/passwd`, `..`, `~/.ssh/id_rsa`,
`$(rm -rf /)`, and a name with a byte of zero, and it holds the answer to a
name of one part. A letter of a different writing is a letter, therefore a
podcast in Greek or in Japanese keeps its name.

**The request writes in the library of the server, therefore the program asks
a question.** The user writes `yes` and presses Enter. Every other answer adds
nothing. This is the rule of section 6 of `docs/T-24-coverage.md` for a
request that changes the server: the program does not send it by one key.

**`autoDownloadEpisodes` is `false`.** A new podcast must not start a download
of every episode of a feed of 111 episodes by itself.

A measurement in a real process added "Another Study of Woman by Honoré de
Balzac (1799 - 1850)" to the sandbox, and the second run of the same test gave
`400`. The program now says "The library can hold that podcast already" for
that answer.

### The server gets an episode — complete, 2026-08-11 (v0.7.5)

The key `D` copies a media to the disk of the user. The key `E` is a different
work: the server gets the file and it puts it in the library, therefore every
client of that server can play it.

**`GET /api/podcasts/:id/checknew` does not do this work.** A measurement on
2026-08-11 gives `{"episodes":[]}` for a podcast that the program added one
second before, and whose feed holds three episodes. That endpoint compares
with the time of the last examination, therefore a new podcast has nothing
"new". The program reads the feed with `POST /api/podcasts/feed` and it
compares with `media.episodes` of the item itself. It then finds every episode
that is missing, and not the new ones only.

`missing` names an episode by its `guid`, then by the address of its file, and
then by its title. An episode with no name at all gives no request: a second
copy of one episode is worse than no copy.

**A second correction of `docs/T-24-coverage.md`.**
`GET /api/podcasts/:id/episode-downloads` gives `404` on 2.36.0. An older
version of that document named it as an endpoint that answers. The queue
belongs to the library: `GET /api/libraries/:id/episode-downloads` gives
`{"queue":[]}`.

`tests/the_new_podcast_against_the_sandbox.rs` holds the measurement. It adds
a podcast of three episodes, it asks for them, it waits for the file, and it
removes the podcast.

### A view of the authors — complete, 2026-08-11 (v0.7.6)

`GET /api/libraries/:id/authors` gives every author with `name`,
`description`, and `numBooks`. `GET /api/authors/:id` gives one author and it
gives **no** `numBooks`, therefore the list is the whole answer that the view
needs and the view sends one request.

The key `a` shows the list. The server gives its own sequence; the view puts
the names in the sequence of the alphabet, because a person reads a list of
names in that sequence.

`l` shows the books of one author. That work is the filter of the library, and
the program held it already since the key `f`: the view writes
`authors.<base64 of the identity>` and it asks for the application again. The
key `f` then takes the filter away.

The program asks the server one time. The authors of a library do not change
while the program runs, therefore the key `a` a second time draws the answer
that it holds. The key `R` forgets it.

## The report of the user of 2026-08-10, on v0.5.0

The user tested v0.5.0 and named ten items. This section holds each one, the
examination against the code of today, and the state.

| Id | Title | State |
|---|---|---|
| T-36 | No way to leave a server, and no way to find that command | Fixed |
| T-37 | `Cargo.toml` names VLC | Not present |
| T-38 | "Buffer overrun", and the book starts at the beginning again | Fixed |
| T-39 | A key that repeats gives a slow list, and the list moves after the key | Fixed |
| T-40 | The start takes a long time, and nothing tells the user | Fixed |
| T-41 | An index of a vector stops the program | Fixed |
| T-42 | The key `R` leaves a broken screen | Fixed |
| T-43 | A series with no description shows nothing | Fixed |
| T-44 | The screen does not say what a line is | Fixed |
| T-45 | The address of the server is examined after the password | Fixed |
| T-46 | A machine with no sound card cannot open the program | Fixed |

Every one of the ten items is complete.

### T-36: no way to leave a server

The command is present, and the user did not find it: `S`, then "Account",
then `l` removes the account of the list. The footer says "l/→: remove saved
user". The words do not say "log out", the view has no title of its own, and
the settings screen names the entry "Account" only.

**The correction.** The entry says "Accounts and log out". The title of the
view says "Accounts — l: log out of the account". The footer says "l/→: log out
of this account (the program forgets its token)". The panel of the description
tells what each entry of the settings does, and it said nothing for the first
two entries before.

A question before the program forgets a token stays open. That needs a screen
of confirmation, and the program has none yet.

### T-37: `Cargo.toml` names VLC

`grep -in vlc Cargo.toml` on 2026-08-10 finds nothing. The word stays in
`src/db/migrate.rs` only, and every use there is correct: the migration of
version 4 renames the column `is_vlc_launched_first_time` and removes the
column `is_vlc_running` of an old database. Those names must stay, because a
database of v0.4 holds them.

### T-38: "Buffer overrun", and the book starts at the beginning — fixed

`grep -rin "overrun" src` finds nothing, therefore that message comes from
outside the program: ALSA writes "buffer overrun" when the sound card takes the
samples too slowly. The serious part is the second one, and an examination of
the code on 2026-08-11 found the mechanism.

**The mechanism.** `rodio` gives the position inside the source, and `get_pos`
gives 0 until the seek finishes. A book of one file that starts at 1227 seconds
therefore reports 0 for a short time. A playback that does not start reports 0
for the whole wait, and that is the condition of the report: the sound card did
not take the samples.

The loop of the playback took every value that the engine gave. It wrote that 0
in the table of the downloads and in the row of the session **every second**,
and it gave that 0 to the server when the session closed. The user then lost
their place, on the disk and on the server, and the book started at the
beginning.

Both loops had the fault: `follow_playback` and `follow_playback_offline`.

**The correction.** The loop writes nothing until the engine reports a position
at the place where the playback starts. `position_is_at_the_start` holds that
rule, with a tolerance of two seconds for a decoder that lands a little early.
A book that starts at 0 gives `true` at once, therefore the rule changes
nothing for a book that the user never opened. After the engine reaches the
place, the loop follows every value, and a move backwards of the user also
works.

**The proof.** `tests/the_position_survives_a_playback_that_does_not_start.rs`
gives the loop an engine that says 0 and never moves, for a book that starts at
1227 seconds. The test then reads the row of the download.

- With the correction: 1227 seconds. The test passes.
- Without it: **0 seconds**, and the message of the test is the report of the
  user. A measurement on 2026-08-11 turned the rule off and saw that.

The second part of the test moves the engine to 1237 seconds and then to 927
seconds, and the row follows both.

### T-39: a key that repeats gives a slow list — fixed

The loop of the events took one key for each turn. The turn then drew the
screen and waited 50 milliseconds. A key that repeats gives about 30 keys each
second, therefore the keys made a queue.

A measurement on 2026-08-10 sent 40 keys in 1.3 seconds to a pseudo terminal
and then watched the line of the selection every 250 milliseconds. The build of
v0.5.0 moved the line for 0.78 seconds after the last key. The build with the
correction had finished before the first look, at 0.26 seconds.

The loop now takes every key that waits, up to 64 for one frame, and it draws
one time for the whole group. It also drops an event of the release of a key,
because a terminal that reports a release sent two events for one press.

### T-40: the start takes a long time — fixed

**The measurement.** A server that accepts a connection and answers nothing
showed the fault at once: the screen stayed black for 15 seconds, the whole
timeout of one request. `App::new` asks the server many times before the first
frame, and the old code drew nothing until all of it finished. A slow server
with many books therefore gives a black screen for minutes.

**The correction.** `ratatui::init` comes before `App::new`. The program draws
while it waits, and that screen names the server and the step that runs. It
names the time after five seconds, it gives advice after ten seconds, and the
key `Q` stops the program. `src/utils/startup.rs` holds the name of the step,
and `src/ui/loading.rs` draws the screen.

The same server that answers nothing now gives this in the first second:

```
┌──────────────────────────────────────────────┐
│                  🦜 Toutui                   │
│                                              │
│           🔗 http://127.0.0.1:13401          │
│                                              │
│        ⠹ the libraries of the server — 11 s  │
│                                              │
│  The server is slow. The program waits for   │
│  the answer.                                 │
│                                              │
│              Q: stop the program             │
└──────────────────────────────────────────────┘
```

**The start is also faster.** The position of each book of the list Continue
Listening needs its own request, and the old code sent them one after the
other. A server with a delay of 300 milliseconds and a list of ten books needed
three seconds for that step alone. The requests go together now, eight at a
time. A run against the sandbox shows that the answers keep the sequence of the
list.

### T-41: an index of a vector stops the program — in part

`self.all_usernames[index]` and `self.libraries_ids[index]` stopped the program
when the list was shorter than the selection. The user removes an account, the
list keeps its old length until the next refresh, and the next `l` reads a
position that is not there. The two places use `get` now, and the list of the
accounts follows the change at once.

**The sweep, complete.** The render read a vector with an index in 37 more
places. Every one uses `at`, `at_number`, `at_part`, or `at_number_part` now,
and a value that is not there gives "N/A".

`tests/the_screen_survives_a_short_list.rs` makes a real `App` with no server,
gives it lists of three lines and lists of no line, puts every selection at the
line 99, and draws all eleven views for a book library, for a podcast library,
and for a search. **The test found two more places that a search of the text
did not find:** `vec![self.titles_pod[0].clone(); n]` stops the program when
the list is empty, and two views hold that line.

### T-42: the key `R` leaves a broken screen — fixed

The refresh writes "Refreshing app…" with `pop_message`. That function writes
to the terminal itself, outside the buffer of ratatui. ratatui writes the cells
that changed only, therefore those bytes stayed on the screen. The loop now
clears the terminal after a refresh, and the next draw writes every cell.

### T-43: a series with no description shows nothing — fixed

Audiobookshelf holds no description for most series. `description_for_the_screen`
gives the description of the first book of the series that has one.

### T-44: the screen does not say what a line is

The list of the library gives a title and nothing else. The user cannot see a
book that they started, a book that they finished, and a book that they never
opened.

**The correction.** Every line of the Home view and of the Library view starts
with a mark of four columns: `▶` for the media that plays, `✓` for a media that
the user finished, `47%` for a media that the user started, and nothing for a
media that the user did not start. `src/ui/marks.rs` holds the rule, and every
function there is pure.

The library holds no position for each book, therefore a line of the Library
view shows the mark of the playback only. A mark of the position there needs
one request for each book of the library, and that would make the start slow
again. See T-40.

### T-45: the address of the server is examined after the password

The login asks for the address, for the name, and for the password, and it
sends the three together. An address with no `http://` therefore fails after
the user wrote everything.

**The correction.** `src/api/server/address.rs` examines the address when the
user leaves the first field. The form comes first, and it needs no network:
"The address must start with http:// or https://. Write
http://192.168.1.10:13378". The function also refuses a port that is not a
number and a name with a space, and it takes an address of IPv6 and an address
behind a path. Then one request goes to `/ping`, which every Audiobookshelf
server answers with no token. An address that answers nothing gives
"http://127.0.0.1:19999 does not answer. Is the server running?" after four
seconds.

### T-46: a machine with no sound card cannot open the program — fixed

`App::new` stopped the whole program when the audio engine did not start. A
user on a machine with no sound device, or with a configuration of ALSA that
does not work, could then not read their library, not download a book, and not
see their progress.

The program keeps every function that needs no sound now, and the header says
"🔇 No sound device: no media can play". A thread takes the commands of the
player and drops them, so that a key of the playback does not fill the memory.

### T-47: live messages need no new dependency

Audiobookshelf sends the changes of a different client over socket.io. The
handover of 2026-08-11 said that this needs a new dependency, and that a new
dependency needs an examination against T-20. **The measurement of 2026-08-11
shows that the program needs no new dependency at all.**

**The examination of the two crates.** Both fail the rule of T-20:

| Crate | Version | `cargo tree -i openssl-sys` |
|---|---|---|
| `rust_socketio` | 0.6.0 | `openssl` 0.10.81, through `native-tls` |
| `tf-rust-socketio` | 0.8.1 | the same |

Both crates ask for `native-tls` with no feature that removes it. `native-tls`
asks the binary to find `libssl` of the system, therefore a release binary would
not run on a machine that has no such library. That is the exact fault of the
candidates of v0.5.0. Both crates also hold `tokio-tungstenite` 0.21, which is
old, and both add more than 180 crates to the tree. **Therefore the fork takes
neither crate.**

**Why no crate is necessary.** socket.io has two transports, and the client
chooses. The transport `websocket` needs a library. The transport `polling` is
plain HTTP: a `GET` that the server answers when it has a message, and a `POST`
that carries a message to the server. `reqwest` does both already.

A measurement on 2026-08-11 against the sandbox made the whole flow with `curl`
and with plain HTTP requests:

| Step | The request | The answer |
|---|---|---|
| 1 | `GET /socket.io/?EIO=4&transport=polling` | `0{"sid":"...","upgrades":["websocket"],"pingInterval":25000,"pingTimeout":20000,"maxPayload":1000000}` |
| 2 | `POST` the body `40`, with `&sid=` | `ok` |
| 3 | `GET` | `40{"sid":"..."}`, the identity of the socket |
| 4 | `POST` the body `42["auth","<the token>"]` | `ok` |
| 5 | `GET` | `42["user_online",{...}]` and `42["init",{...}]` |

Then a second program changed the data, and the `GET` of the step 5 gave the
message of each change inside 3 seconds:

| The change of the second program | The message |
|---|---|
| `PATCH /api/me/progress/:id` | `user_updated` |
| `PATCH /api/items/:id/media` | `item_updated` |
| `POST /api/items/:id/play` | `user_stream_update`, `stream_progress`, `stream_open` |

**The three rules of the transport.** The measurement found all three:

1. **The server sends `2`, and the client must answer `3`.** `2` is a ping and
   `3` is a pong. The period is `pingInterval`, therefore 25 seconds. A client
   that does not answer gets `1` (close) after `pingTimeout`, and every later
   request of that identity gives `400`.
2. **One `GET` at a time for one identity.** A second `GET` in parallel gives an
   error. A `POST` beside the `GET` is correct.
3. **One answer can hold more than one packet.** The separator is the byte
   `0x1e`, and not a comma. A reader that takes the whole body as one packet
   loses every message after the first.

**A message can hold a secret.** `user_updated` carries the whole account of the
user, and that object holds a **new token**. The log must never write the body
of a message.

### T-48: a book of two files plays with no player and with no position

A user reported this on 2026-08-11: one book of their library played the sound,
**and the program showed no player and sent no position.** The book started at
minute 0 every time. Every other book of that library was correct.

**The cause.** The queue of the engine holds two tracks, therefore the engine
opens the track that plays now **and the track after it**. That book held the
same audio two times: one file of AAC-LC, and one file of xHE-AAC. symphonia
reads AAC-LC only.

`fill_queue` gave the fault of the second file to `start`, and `start` then
stopped the whole playback: the status became `Stopped` and `current` became
`None`. **The first file was in the queue of the player already, and the queue
of the player plays a track as soon as the engine appends it.** Therefore:

| What the user saw | Why |
|---|---|
| The sound plays | The queue of the player holds the first track |
| No player on the screen | The state says `Stopped`, and the screen draws the player for a state that is not `Stopped` |
| No position on the server | `follow_playback` reads the state of that playback, and the state holds no playback |
| The book starts at minute 0 | No position ever went to the server |
| The keys of the player do nothing | `current` is `None` |

**The correction.** `the_fault_stops_the_playback` gives the rule: the fault of
the track that plays now stops the playback, because no sound can come. The
fault of a track after it does not. `Current` holds `tracks_that_play` now, and
that value becomes the number of the track with the fault. Therefore:

- The playback starts, the screen shows the player, and the position goes to the
  server.
- The engine asks the server for that file no more. The old code would ask five
  times each second, because `advance` calls `fill_queue` at each tick.
- The book ends at the track before the file with the fault. `is_complete` reads
  `tracks_that_play`, and not the number of files of the book.
- The panel of the player says "The program cannot read <the file>", therefore
  the user knows why the book stops early.
- A start that ends with a fault calls `player.stop()` now. The sound of a track
  that the engine appended already must not go on.

**The measurement.** A book of the sandbox held `01 - Part 1.mp3` of 30 minutes
and `02 - Part 2.wma` of 30 seconds. Toutui plays no WMA file (T-18), therefore
this book has the exact shape of the book of the user.

| The binary | The log | The screen |
|---|---|---|
| Before | `[ERROR] the engine cannot start the book: ... It does not play wma and awb.` | No player, and the line has no mark |
| After | `[WARN] the engine cannot open the track 2 of 2: ... The tracks before it play.` | `▶` on the line, and the player says `⏸ 28:52 / 30:29 ... (95%)` and `The program cannot read 02 - Part 2.wma` |

**What stays open.** The position of a book that ends early goes on to the end of
the whole book, because the queue of the player is empty and the position of the
engine goes on. The book of the sandbox gave `currentTime` 60 for a book of 60
seconds after 30 seconds of sound. A book with this shape is a fault of the
library of the user: the same audio must not stand two times in one item.

### T-49: the footer holds more keys than the screen shows

The user reported this on 2026-08-11: "we have too many keybinds on the area,
and I can't even see them, they go off-screen".

The footer of the Home view held every key of that view in two lines of 342
characters. A terminal of 160 columns showed the first 320, and a terminal of 80
columns showed a quarter of them. The keys of the work of the view stood beside
the keys that a user needs one time in a month.

**The correction.** `src/ui/keys.rs` holds the keys now, and it holds them one
time. The footer of a view names the keys of the work of that view, and it fits
in 92 columns. The key `?` opens a list of **every** key, in six groups.

- The list is a view of its own (`AppView::Keys`), therefore a small terminal
  scrolls it with `j` and `k`.
- The key `?` a second time gives the view of the user back, and so do `h` and
  `Esc`. `Esc` alone stops the program in every other view; inside this list it
  closes the list, because a user who opened a list to read must not lose their
  work with the key that closes it.
- A test reads `src/app.rs` and it finds every `KeyCode::Char` of the handler in
  a group. Therefore a new key of a later session cannot stay hidden.

**The measurement.** A run in tmux at 160 by 45 gave the footer "j/k: move  l:
play or open  Tab: home/library  /: search  R: refresh  ?: every key  Q: quit",
and the key `?` gave the whole list of 46 keys.

### T-50: the cover is much smaller than the panel of the cover

The user reported this on 2026-08-11: "the images is too small for the area, we
can use more of the area for the image, if it's a series we can than put them
small as they are now".

Two rules made the picture small:

1. **The panel took 30 per cent of the width, and never more than 46 columns.**
   A cell of a terminal is two times higher than it is wide, therefore a picture
   of 46 columns is 23 rows high. The panel had 34 rows, and 11 of them stayed
   empty.
2. **`square_box` fitted every picture in a square.** A cover of a book is
   higher than it is wide, and such a cover then used two thirds of the height.

**The correction.**

- The panel takes 40 per cent of the width now, and the height gives the second
  limit: `width_that_the_height_can_use` gives the columns that a picture of the
  full height needs. A panel that is wider gives the picture no pixel, and it
  takes columns of the text for nothing.
- `box_of_the_picture` takes the form of the real picture. `CoverArt` keeps the
  width divided by the height of each picture that it reads, therefore a cover
  that is higher than it is wide takes every row of the panel.
- **A shelf of a series does not change.** One cover of the selection takes the
  whole area, and two to four covers stay in the grid of squares.

**The measurement.** The real program at 160 by 45, with the same book selected:

| The binary | The list | The picture |
|---|---|---|
| Before | 113 columns | 46 columns by 23 rows, from the column 114 |
| After | 95 columns | 57 columns by 28 rows, from the column 103 |

The picture holds about 50 per cent more cells, and the text keeps 95 columns.

### T-51: read a PDF book, and what bookokrat can give

The user named `https://github.com/bugzmanov/bookokrat` on 2026-08-11, a reader
of EPUB and of PDF for the terminal, and they said: "We are both GPLv3 so we can
copy code where we need it."

**The measurement of the license says no.** `bookokrat/LICENSE` holds the **GNU
Affero General Public License, Version 3**, and `Cargo.toml` of that project
says `license = "AGPL-3.0-or-later"`. Toutui is `GPL-3.0-or-later`.

The AGPL adds the rule of the section 13: a user who reaches the program over a
network must get the source. GPLv3 permits a **combination** with AGPL code, and
the AGPL part keeps its own rule inside that combination. Therefore:

- **No line of bookokrat may go into this repository** while `Cargo.toml` says
  `GPL-3.0-or-later`. The file would then say a license that the work does not
  have, and every user of the fork would get a wrong answer.
- A change of the license of Toutui to AGPL is a decision of the maintainer, and
  it touches AlbanDAVID as the first author. **Do not make that change without
  the maintainer.**
- **A person may read that project and write their own code.** A rule of a
  layout and the name of a crate are facts, and a fact has no license.

**The second fault: the way that bookokrat draws a PDF does not pass T-20.**
`src/pdf/converter.rs` asks `mupdf` 0.6 with the features `svg`,
`system-fonts`, and `img`, and it draws the pages of the pixmap with the
protocol of Kitty. MuPDF is a library of C, and MuPDF itself is AGPL. The
feature `system-fonts` asks the machine for the fonts. Therefore that path
breaks the rule of the dependencies **and** the rule of the license.

**What Toutui can do instead: read the text, and not the page.** A measurement
on 2026-08-11 of three crates of pure Rust:

| Crate | Version | License | `cargo tree -i cc` | `cargo tree -i openssl-sys` |
|---|---|---|---|---|
| `pdf-extract` | 0.12.0 | MIT | nothing | nothing |
| `lopdf` | 0.44.0 | MIT | nothing | nothing |
| `pdf` | 0.10.0 | MIT | nothing | nothing |

`pdf_extract::extract_text` read a real PDF of 11445 bytes and it gave 6751
letters in 157 lines, and the first lines are the lines of the page. Therefore
one dependency of pure Rust gives the text of a PDF, and the reader of T-10 can
show that text with the same widget as an EPUB book.

**What the program must not try.** No crate of pure Rust draws a page of a PDF
today. `pdfium-render` needs a library of the machine, and `mupdf` needs C and
AGPL. Therefore a PDF in Toutui is text, and a user who needs the page of a
figure opens the web page of the server.

**The ideas of bookokrat that a person may use, after they write their own
code.** These are facts of a design, and not code:

1. **HTML to a middle form, and then to lines.** That project makes markdown of
   the XHTML of a chapter, and it draws the markdown. Toutui walks the XHTML
   itself in `src/logic/reader/cfi.rs`. A middle form makes the search and the
   selection of a text easy, because the place of a letter stays the same.
2. **A picture of a chapter gets a line of its own in the flow.** The reader
   keeps the place of the picture, and it draws the picture when the terminal
   can. A terminal that cannot draw shows a line that names the picture.
3. **A task loads the pictures behind the screen.** The same shape as the covers
   of T-23: a store of the process, and the render takes what is ready.

**The state of this item.** The license makes it a decision of the maintainer,
therefore no code goes in before that decision. The PDF of the text needs no
such decision: `pdf-extract` is MIT and pure Rust.

**The decision of the maintainer, 2026-08-11. This item is closed.**

> **Toutui stays `GPL-3.0-or-later`.** A person may read bookokrat for the idea
> of a function, and they must then write their own code.

The rules that come with that decision:

1. **No line, and no near copy.** A name of a function, a sequence of the steps
   of a function, and the shape of a data type are the work of that project. A
   person who reads it must write the code from the **rule**, and not from the
   text of the code.
2. **The commit must say it.** A commit that comes from an idea of bookokrat
   names that project and the idea, in the way that T-51 names the three ideas
   above. The record must show what came from where.
3. **The rule of T-20 holds too.** `mupdf` and every library of the machine stay
   outside, therefore the way that bookokrat draws a PDF does not come in at all.
   T-54 gives the text and the pictures of a PDF with `lopdf` of pure Rust, and
   that work took no idea of bookokrat.
4. **The licence of the fork does not change.** AlbanDAVID is the first author of
   this work under GPL, and this decision keeps his choice.

### T-52: a fault of the reader looked like a program that stopped

The user reported this on 2026-08-11: "the player gave an error saying it had no
book or something, than I couldn't even go back, it locked in me that error, had
to relaunch the whole app".

A run in tmux gave the fault at once. The key `e` on a media with no EPUB book
gave a screen of one line: "The program did not get the book: The server does not
have this item." **That screen named no key**, and three rules made it a trap:

| The fault | Why |
|---|---|
| The screen named no key | The view of the reader draws no footer, and a view with no book drew a line of text and nothing else |
| `Q` did nothing | Every key of the view of the reader went to the reader, and the reader uses `Q` for no work. Therefore the program could not stop |
| `Esc` did nothing | The same rule. `Esc` closed the contents of the book only |
| `h` went to the Library | The user came from the Home view, and they lost their line |
| The text says nothing about the media | The endpoint of the ebook answers 404 for a media with no ebook **and** for an item that does not exist |

**The correction.**

- A view of the reader with no book keeps two lines for the footer, and that
  footer says "h/Esc: back  ?: every key  Q: quit". The message stands under a
  title, therefore the screen looks like a view and not like a program that died.
- `Q` reaches the handler of the program in every view now. The reader takes
  every other key.
- `Esc` leaves the reader when the contents are closed, and it closes the
  contents when they are open.
- `h` and `Esc` give the view that the user came from back. `App` holds
  `the_view_before_the_reader`.
- `why_the_book_did_not_come` asks for the item after a 404, and
  `the_message_of_the_format` gives the sentence: "This media has no ebook.",
  "The ebook of this media is a PDF file, and the reader shows EPUB books only.",
  or the fault of the request. That function is pure, and a test holds it.

### T-53: every codec of the server, with no new dependency

The user gave the rule on 2026-08-11: "we want to support as many codecs as
possible so a part of a book failing is a big issue". T-48 stopped the fault of
one file from stopping the whole playback, and the book of that user then played
26 hours of 51. **That is a side-step, and not a correction.**

**The measurement.** No decoder of pure Rust reads xHE-AAC. symphonia reads
AAC-LC. `libfdk-aac` reads xHE-AAC, and the license of that library does not
agree with the GPL. `faad2` reads no xHE-AAC. Therefore **the program cannot
decode that file itself, today or soon.**

**The answer of the server.** Audiobookshelf gives every media in two ways, and
the second way was not used:

| The way | The request | What comes |
|---|---|---|
| The file | `GET /api/items/:id/file/:ino` | The bytes of the file. The program reads 17 forms |
| The stream | `POST /api/items/:id/play` with `forceTranscode` | `playMethod: 2` and **one** address of HLS for the **whole** media |

ffmpeg of the server makes that stream. **Therefore every codec that ffmpeg reads
becomes a codec that Toutui plays**, and the answer needs no new dependency of
this program.

**What the work needed.** Three readers, and every one of them is pure:

1. **The playlist.** A list of lines. `parse_playlist` of
   `src/player/engine/hls.rs`.
2. **The container.** A part of the stream is an MPEG transport stream of packets
   of 188 bytes. The program reads the table of the programs, it takes the
   identity of the audio, and it gives the payload of those packets with no
   header of PES. That payload is an elementary stream of MP3 or of ADTS AAC, and
   symphonia reads both.
3. **The form.** The table of the programs names the form. `0x11` is AAC inside
   LATM, and symphonia reads no LATM. The program refuses that form **before** the
   playback starts, therefore it never gives silence to the user.

`src/player/engine/hls_file.rs` holds the reader of the stream. It has the shape
of `HttpFile`: a thread fills a buffer, and `read` copies bytes of that buffer.
**The stream moves forward only**, therefore a movement of the playback starts a
new playback of the engine.

**When the program uses the stream.** The engine opens the file of the playback
and the file after it. A fault of a decoder comes in some milliseconds, therefore
`the_file_that_no_decoder_reads` gives the name to the loop of the playback inside
2.5 seconds. That loop then closes the session of the file, it asks for a session
of a stream, and it starts the playback again with **one** track of the whole
media. The direct playback of a file stays the first choice: it needs no work of
the server.

**The measurements of 2026-08-11.**

| What | The answer of an Audiobookshelf 2.36.0 |
|---|---|
| `POST /api/items/:id/play` with `forceTranscode` | `playMethod: 2`, one track, `/hls/<session>/output.m3u8`, `duration` of the whole media |
| The playlist of a book of 30 minutes | 305 parts of 6 seconds |
| `output-0.ts` | 26884 bytes, 143 packets, the identity 256 gives 24033 bytes that start with `ff f3` |
| A part that ffmpeg did not write yet | `404`. The reader waits, and it tries again |

The whole way in the real program, with the book of `docs/TEST-SERVER.md` of one
MP3 file and one WMA file:

```
[worker] the engine cannot open the track 2 of 2: ... It does not play wma and awb.
[play] no decoder of the program reads 02 - Part 2.wma. The program asks the server for a stream of the whole media.
[HlsFile] the stream holds 305 part(s). The reader starts at the part 0 and at 0.0 seconds inside it. The audio is Mp3.
```

The player then showed `⏸ 30:00 / 30:29`, and the server held `currentTime` 1800
of 1830. **The book of two files played as one media, and the file that no
decoder of the program reads gave no silence.**

**A fault of the loop, and the correction.** The first form of this work read
`file_with_no_decoder` of the state with no thought of the playback that met the
fault. The engine clears that value when it starts a playback, and the command of
the start is not immediate. Therefore the loop of the **stream** read the fault of
the **file** and it said that the stream holds a form that no decoder reads, while
the stream played. The state holds `playback_of_the_fault` now, and a loop reads
the fault of its own playback only.

**The file of the newest form of AAC.** ffmpeg of the server copies the codec of
the file when that codec fits a transport stream, and it asks no question about
the form of AAC: `codecsToForceAAC` of the server holds `alac`, `ac3`, `eac3`, and
`opus` only. xHE-AAC names itself `aac`, therefore the server copies it. A
transport stream holds that form as LATM, and symphonia has no reader of LATM.

Two answers exist, and the program holds both:

1. **ffmpeg gives a fault for such a copy.** The server then sets `forceAAC` and
   it starts the transcode again, and the stream holds AAC of the old form. The
   reader of the parts waits for a part that does not exist yet, therefore it
   waits for that second try as well.
2. **ffmpeg gives LATM.** The program refuses that form **before** the playback,
   and the screen says: "The stream of the server holds a form that the program
   cannot read." A user who meets that message must change the file of their
   library.

**The measurement of the answer 1 needs a file of xHE-AAC**, and no program of
this machine writes one: an encoder of that form needs `libfdk-aac`. The next
session must measure it against the book of the user.

**What stays open.** The stream of the server needs ffmpeg on that server. A
server with no ffmpeg gives no stream, and the program then plays the files that
it reads. The panel says "The server makes the stream of this media" while the
stream plays, therefore the user knows why the start takes longer.

### T-54: the reader shows a PDF book, with its pictures

The user asked for it on 2026-08-11: "we do want pdf's with image supports", and
"let's just read and write our own code, but write it very nicely". T-51 holds the
reason why no line of bookokrat comes in: that project is AGPL, and Toutui is
GPL-3.0-or-later.

**The dependency.** `lopdf` 0.44, MIT, pure Rust, with no default feature. A
measurement on 2026-08-11: `cargo tree -i cc` finds `libsqlite3-sys` and `ring`
only, and `cargo tree -i openssl-sys` finds nothing. The tree grew from 452
crates to 471.

**The shape of the work.** A PDF holds no chapter and no flow of text: it holds
pages, and each page holds a program that draws letters at places. Therefore:

| The part | What it does |
|---|---|
| `src/logic/reader/pdf.rs` | Opens the file, and gives the text and the pictures of each page |
| `xhtml_of_the_page` | Makes XHTML of one page. **The render of the EPUB book then makes the lines**, therefore the program holds one render only |
| `Book` | Holds two forms now: `Kind::Epub` and `Kind::Pdf`. Every function of that type answers for both, therefore the reader of T-10, the position, and the list of the contents needed no change |
| `the_file_is_a_pdf` | The first five bytes decide, and not the name of the file. The server gives the ebook of every form at one address |
| `render_the_picture_of_the_page` | Draws the picture at the right of the text, with the form of the real picture. See T-50 |

**One page is one chapter.** The keys `n` and `p` of the reader move between the
pages, and the list of the contents of the key `t` names them.

**The pictures.** A picture of the filter `DCTDecode` **is** a JPEG file, therefore
the program copies the bytes and reads nothing. A picture of raw samples of 8 bits
becomes a PNG file of `image`, which the tree holds already for the cover art of
T-23. A picture of a different form gives no picture, and the line of the text
still says that the picture exists. Therefore a terminal that draws no picture
loses nothing.

The program keeps the form of `ratatui-image` of eight pictures. A form at each
frame would read the file twenty times in one second.

**The limits of the memory.** The file 512 megabytes, the pages 5000, one picture
32 megabytes, and the pixels of one picture 50 million. A stream that names fewer
samples than the size of the picture needs gives no picture: a reader that trusts
the size of the dictionary reads memory that is not its own.

**The title.** A PDF holds no title in most files, and the name of the file on the
disk is the identity of the item. Therefore the reader takes the title of the
media of the server. **That value must come before the view changes**, because the
title comes from the line that the user selected.

**The measurement of 2026-08-11.** A PDF of three pages: one page of text and two
pages of a picture of JPEG. The program opened it in tmux at 160 by 45:

| The page | The screen |
|---|---|
| 1 | "One File With No Decoder — chapter 1 of 3 — 0%", and the text of the page |
| 2 | "[ the picture Im0: 400 by 300 pixels ]", and **the picture of 40 columns by 15 rows at the column 96** |

400 by 300 pixels in a cell of 10 by 20 pixels gives 40 columns by 15 rows.
Therefore the picture holds its form.

**What a PDF does not give.** No crate of pure Rust draws a **page** of a PDF, and
`mupdf` needs a library of C and it is AGPL. Therefore the reader shows the text
of the page and the pictures of the page, and not the page. A user who needs the
form of a page of a figure opens the web page of the server. See T-51.

### T-55: the position of a book that ends early goes on to the end

T-48 gave the rule that a file with no decoder ends the book at the file before
it. The position of the playback did not follow that rule: the queue of the player
goes on counting when it is empty, therefore the position went to the end of the
**whole** book. The book of the sandbox of 30 seconds of MP3 and 30 seconds of WMA
gave `currentTime` 60 of 60 after 30 seconds of sound.

**The correction.** `end_of_the_first` of `TrackList` gives the end of the tracks
that play, and `position_now` gives that value when every such track played. The
old rule of T-2 and T-16 counted the tracks of the book; the new rule counts the
tracks that play, and the two agree for a book with no such file.

The media does not become finished: `reached_the_end` compares the position with
the length of the **whole** book. A book that ends early is not a book that the
user heard.

**This rule matters when the server gives no stream.** T-53 asks the server for a
stream of the whole media, therefore a server with ffmpeg plays every file. A
server with no ffmpeg keeps the direct playback, and this rule then holds the
position.

### T-56: the queue of the media stands on the disk

The queue of T-24 lived in the memory of the process. A user who stopped the
program lost every media that waited, and the handover named that as an open item.

**The table.** The version 7 of the schema holds `queue`: the account, the server,
the place, the item, the episode, the title, the author, and the length. The
account and the server hold the queue apart, therefore a user with an account on
two servers keeps one queue for each of them.

**When the program writes.** Every change of the queue writes every row again. A
queue holds some media, therefore that write costs almost nothing and it needs no
rule for a row that changed. `write_the_queue` writes nothing while no account has
a name: the queue then belongs to a test, and a test must not touch the database of
a user.

**When the program reads.** `read_the_queue_of_the_account` runs one time, before
the first frame. It names the account of the queue, and it puts every row of the
disk in the queue of the process.

**The rule of a media that the server does not hold now.** The row holds the
identity of the item, and the server answers the playback. A media that went away
gives the fault of that playback, and the queue then goes on to the media after it.
Therefore the program needs no examination of the queue at its start, and it asks
the server nothing before the user plays a media.

**The measurements of 2026-08-11, in tmux.**

| What | The screen |
|---|---|
| The key `n` two times, and then `q` | "The queue [2 items]", "1. 📕 One File With No Decoder", "2. 📕 Alice in Wonderland" |
| A new start of the program, and `q` | The same two media, in the same sequence |
| The key `X` in the view of the queue | "The queue [1 items]", "1. 📕 Alice in Wonderland" |
| A new start after that key | The same one media |

`tests/the_queue_on_the_disk.rs` holds the rules with no server: the sequence, the
account, the server, and the write that takes a row away. **That test sets
`XDG_CONFIG_HOME`, therefore it stands alone in its binary.**

### T-57: a picture of 16 bits, and the predictor of PNG

The first form of T-54 read a picture of a PDF of 8 bits of one component only. A
measurement on 2026-08-11 of a PDF that `img2pdf` made of a PNG file shows that
this is not enough:

```
BitsPerComponent = 16
ColorSpace = /DeviceGray
DecodeParms = <</BitsPerComponent 16/Colors 1/Columns 1200/Predictor 15>>
Filter = /FlateDecode
```

**Every PDF of a PNG file holds 16 bits**, therefore the reader gave the line of
the text and no picture.

**Two facts of the measurement.**

1. `decompressed_content` of `lopdf` **undoes the predictor of PNG**. The stream
   of 1200 by 1600 gave 3840000 bytes, and that is 1200 × 1600 × 2 with no byte of
   a row of the predictor. Therefore this module needs no reader of a predictor.
2. A sample of 16 bits holds the byte of the largest value first. A terminal shows
   some hundred cells of one picture, therefore the high byte holds every
   difference that a user can see. `eight_bits_of` takes that byte.

**The measurement of the memory and of the time.** `Pdf::open` reads every page one
time, therefore a book of many pictures must not stop the program:

| The book | The pages | The time | The pictures in the memory |
|---|---|---|---|
| 120 pages of JPEG of 1200 by 1600 | 120 | 3.5 milliseconds | 1006 kilobytes |
| 60 pages of Flate of 16 bits of 1200 by 1600 | 60 | 473 milliseconds | 784 kilobytes |

A picture of JPEG costs almost nothing: the bytes of the stream **are** the file.
A picture of raw samples needs one write of a PNG file, and the answer is small.
`Pdf::open` runs in a task, therefore the screen of the reader shows "The program
gets the book…" while it works.

The real program drew the picture of 1200 by 1600 as 63 columns by 42 rows: the
form of the picture is 0.75, and a cell of 10 by 20 pixels gives 630 by 840 pixels.

### T-58: the reader says "page" for a PDF, and `?` works inside it

Three small faults of T-49 and of T-54, and every one of them costs the user a
question:

1. **The reader named a chapter of a PDF a "chapter".** One chapter of such a book
   is one page, and the word says nothing about that file. The line at the top says
   "page 1 of 60" now, and the footer says "n/p: page" and "t: the pages".
2. **The key `?` did nothing inside the reader.** The reader takes every key
   before the lists, and it uses `?` for no work of its own. Therefore that key
   holds the same meaning in every view now, and `h` gives the book back.
3. **The list of every key held no key of the reader.** A user who could not leave
   a book found no line about it. The group "The reader of a book (the key `e`)"
   holds those keys now.

The rule "no key stands two times" became "no key stands two times **inside one
group**". A key of two groups is correct: the reader uses the letters of a list for
its own work, and the group of the reader says so.

The measurement in tmux: the line at the top gave "One File With No Decoder — page
1 of 60 — 0%", the key `?` gave the list of every key, and the key `h` gave the
book back.

### T-59: a message of `pop_message` can go away before the user reads it

A sweep of every view on 2026-08-11 found this. The key `C` with no media that
plays writes "No media plays now." with `pop_message`, and the screen of the user
showed **nothing**: the Home view drew that row again, and the message went away.

**The cause.** `pop_message` writes to the terminal at a row, **outside the buffer
of ratatui**. ratatui writes the cells that changed at each frame. A message
therefore stays while no view draws that row, and it goes away when a view draws
it. The row 3 of the screen holds the title of a list in most views, therefore the
message of that row is the message that goes away.

T-42 met the other half of the same fault: a message that **stays** after the view
changed, because ratatui writes no cell that did not change.

**The correction of this item.** The view of the chapters opens for every answer
now, and its title names the reason: "No media plays now. A media that plays gives
its chapters." or "\"<the title>\" holds no chapter." The view of the bookmarks
held that shape already, and it is the right shape: **a view says why it holds no
line, and a message of one row says nothing.**

**What stays open, and its measurement.** The program holds **93** calls of
`pop_message`, in `src/app.rs`, `src/logic/playback/mod.rs`,
`src/logic/download/mod.rs`, `src/db/crud.rs`, `src/main.rs`, and
`src/logic/sync_session/`. Every one of them can go away in the same way.

The answer is the shape of every other value of this program: **a slot between the
work and the screen.**

1. `src/logic/message.rs` holds the newest message and the time of it, as
   `src/logic/live.rs` holds the live messages. A task and a key both write it, and
   neither needs `&mut App`.
2. The render of `App` draws that message inside the frame, above the footer, and
   it takes the message away after some seconds.
3. Every call of `pop_message` outside the screen of the login becomes a call of
   that slot. The login draws no frame of ratatui, therefore it keeps
   `pop_message`.

**The change of the 93 call sites, in one commit.** `src/logic/message.rs` holds
the slot, and the render of `App` draws the message inside the frame:

| The part | What it does |
|---|---|
| `say(text)` | The work writes the newest message. A key and a task both call it, and neither needs `&mut App` |
| `for_the_screen()` | The render takes the message, and it gives nothing for a message that is older than `LIFE` of 6 seconds |
| `one_line(text, width)` | Cuts a message that is wider than the screen, and it names the cut with three points. The row of the message holds one row |
| `forget()` | A work that ended takes its message away |
| `render_the_message` | Draws the row above the footer, with the colours of the header. It stands after the view and **before the bar of the downloads**: a download is the work that the user waits for, therefore that bar keeps its rows |

Every call of `pop_message` and of `clear_message` went away, and the local values
of `stdout` with them: `src/app.rs` (53 and 38), `src/db/crud.rs` (22),
`src/logic/playback/mod.rs` (9), `src/logic/download/mod.rs` (6),
`src/logic/sync_session/wait_prev_session_finished.rs` (1), and `src/main.rs` (1).
`play_media` and `play_the_stream_of_the_server` take no `stdout` any more.

**The two exceptions, and the reason of each.**

1. **The screen of the login** keeps `pop_message`. That screen holds its own loop
   and it draws no frame of the application. The module of `pop_message` says so
   now, therefore no later work brings the pattern back.
2. **The key `R`** says its message and then draws **one** frame before the work.
   The loop draws no frame while `App::new` asks the server, therefore a message of
   the slot alone would come after the work and not during it.

**The measurements of 2026-08-11, in tmux at 160 by 45.**

| The key | The row above the footer |
|---|---|
| `n` | `"One File With No Decoder" is number 3 of the queue. Press q to see the queue.` |
| The same, 7 seconds later | Empty. The message went away with no work of the user |
| `F` with no media | `Sync: nothing plays now.` |
| `M` | `The media is not finished now, and its position went back to the start. Press R to see the change.` |
| `F` while a media plays | `Sync: the server has the position 23m.`, **and the panel of the player above it** |

**A trap of the tests.** The slot belongs to the process. The first form of this
work held two test functions that wrote it, and the two ran at the same time: one
run of `cargo test` of three gave a fault. The parts of such a test stay in one
function, as the tests of `live`, of `stats`, and of `queue` do.

### T-60: the filter of the library holds the tags now

The handover named "the narrators and the tags" as an open item, and it said that
the filter of the key `f` shows both already. **A measurement on 2026-08-11 shows
that the tags were absent.**

| The request | The answer |
|---|---|
| `PATCH /api/items/:id/media` with `{"tags":["a-test-tag"]}` | `200`, and `GET /api/items/:id` then holds that tag |
| `GET /api/tags` | `{"tags":["a-test-tag"]}` |
| `GET /api/libraries/:id/filterdata` | `tags: []` — **and a scan of the library changes nothing** |
| `GET /api/libraries/:id/items?filter=tags.YS10ZXN0LXRhZw==` | The one media of that tag |

Therefore the filter of a tag works, and the **list** of the tags was missing: a
user could not reach a tag from the program.

**The correction.** `get_the_tags` asks `GET /api/tags`, and `with_the_tags` puts
those tags in the data of the filter. The view of the key `f` then holds the group
"The tags", and the choice of the user goes to the server as
`filter=tags.<base64>`, in the same way as a narrator and a genre.

- A tag that `filterdata` gave already does not stand two times.
- A server that gives an error for that endpoint gives no tag, and every other
  group of the view stays. A tag is one group of eight.
- `GET /api/tags` holds the tags of the **whole server**. A tag of a different
  library therefore gives no media, and the list of the library is then empty.

**The measurement in the real program.** The key `f` after this work:

```
  ▌ The tags
    a-test-tag
  ▌ The narrators
    A Test Narrator
```

**The narrators need no work.** `filterdata` gives them, and the measurement of the
filter of `narrators.<base64>` gave the media of that narrator. A view of its own
for the narrators would hold the same list as the group of the view of the key `f`.
Therefore the item of the handover is complete.

### T-61: the task of the live messages asked for ever, every ten seconds

T-47 gave the program the live messages of the server. The task of that work waited
ten seconds after a connection that ended, and it then tried again — **for ever, and
with the same wait**.

**The measurement of 2026-08-11.** A server that answers `404` for every request of
socket.io, which is an Audiobookshelf behind a proxy that does not pass
`/socket.io/` and an older version of the server:

| The rule | The requests of 65 seconds | The requests of one day |
|---|---|---|
| Before | 6 | 8640 |
| After | 3, at the second 0, 10, and 30 | fewer than 200 |

**The correction.** `wait_after_the_faults` doubles the wait after each fault, one
after the other, and it stops at ten minutes. A connection that **opened** gives the
count of the faults the value 0, therefore a server that answers again gives the
live messages back at once. `Attempt` holds that answer: `opened` and the fault.

The log holds the first fault of a server and it holds no fault after it, and it
says one time that the program waits longer after each fault. A program that runs
for days must not fill the log of the user with one line every ten seconds.

### T-62: a book of a scan held every picture of every page in the memory

T-54 gave the reader the pictures of a PDF, and `Pdf::open` kept the file of every
picture of every page.

**The measurement of 2026-08-11**, with a book of 150 pages of a scan of 1400 by
1900 pixels (137 megabytes):

| What | Before | After |
|---|---|---|
| The pictures in the memory | 137 megabytes | 9.5 megabytes |
| The time of the open | 0.15 seconds | 3.85 seconds |
| The largest memory of the program | 279 megabytes | 277 megabytes |

**The correction.** The panel of the picture of a terminal of 160 by 45 holds about
64 columns and 42 rows, and a cell of 10 by 20 pixels makes that 640 by 840 pixels.
A picture of more pixels therefore gives the user **nothing**, and it takes their
memory while they read. `smaller_if_it_is_large` reads such a picture, it makes it
smaller with `thumbnail`, and it writes a JPEG file of quality 82.

- **The two numbers of the picture stay the numbers of the page.** The user asks
  how large a picture is, and the answer of the page is the answer that they want.
  `thumbnail` keeps the form, therefore the panel draws the same rectangle.
- A picture that the screen shows already keeps the bytes of its file: the program
  then reads no picture and it writes no picture.
- A file that `image` cannot read keeps its bytes. The screen shows what it can.
- `MAX_PICTURES_OF_A_BOOK` of 48 megabytes is the backstop. A page after that limit
  holds no picture, and the line of the text still says that a picture exists. The
  log says how many pages lost their picture.

**The time of the open is the cost.** The program reads and writes 150 pictures,
therefore the open of that book takes 3.85 seconds. `Pdf::open` runs in a task, and
the screen of the reader says "The program gets the book…" while it works.

**The largest memory did not change, and that is `lopdf`.** `Document::load` reads
the whole file, therefore the program holds the bytes of a book of 137 megabytes for
a moment. That memory goes away after the open, and the 9.5 megabytes of the
pictures stay. A book of 500 megabytes therefore needs a machine of a gigabyte for
one moment. `MAX_BOOK_BYTES` of 512 megabytes holds that limit.

**The measurement in the real program.** The heavy book of 150 pages: the line at
the top said "page 1 of 150", the line of the text said "[ the picture Im0: 1400 by
1900 pixels ]", and the picture drew 48 columns by 32 rows.

### T-63: the position and the movement of a playback of the stream

T-53 gave the program the stream of the server for a file that no decoder reads.
Two faults of that work came out of a measurement on 2026-08-11, and **both need a
media that the user did not start at its beginning**. The measurements of T-53 all
began at the second 0, therefore neither fault showed itself.

**The first fault: the position was the position of the stream.** The stream begins
at the place of the user, and `position_now` gave the position of the **decoder**.
A book that the user left at 26 hours therefore reported 0 and then 1, 2, 3… , and
the loop of the playback would write those numbers on the server: **the user would
lose 26 hours of their place.**

The loop also holds the rule of T-38: it writes nothing before the engine reaches
the place where the playback starts. A stream that reports 0 for a place of 600
seconds therefore wrote **nothing at all** for the first 600 seconds of the
playback.

**The correction.** `Opened` gives the place of the media where the bytes of a
decoder begin: a file gives 0, and `HlsFile` gives the second of the part of the
playlist where it opened. `Current` holds that value, and `position_now` adds it.
The measurement: a book that the server holds at 600 seconds gave `⏸ 29:59 / 30:29`
after the stream played to its end, and not `⏸ 19:59`.

**The second fault: a movement of the playback did nothing.** The keys `p` and `u`
of a stream gave a `try_seek` of a source that moves forward only. The engine wrote
"the engine cannot move inside the track", and the playback stayed where it was.

**The correction.** `seek_to` asks the server for the stream **again** at the new
place: the value `seconds` of `TrackSource::Stream` takes the new position, and the
queue of the player comes again. `HlsFile::open` then takes the part of the playlist
that holds that second.

**A movement back asks for a part that the server did not write.** The transcode of
the server began at the place of the user, therefore a part before it does not
exist. The server answers 404 and it starts the transcode again: its log says
"Segment #N Request is before starting segment number #M - Reset Transcode". That
work takes a second or two.

`HlsFile::open` runs on the thread of the engine, and that thread reads the commands
of the user. Therefore the open waits little: `ATTEMPTS_OF_THE_OPEN` of 8 with a
delay of 2 seconds at the most. The thread that fills the buffer keeps the 20
attempts and the delay of 10 seconds, because a wait there costs the user nothing.

**The measurement of the movement.** The stream of a book at `⏸ 26:46 / 30:29`, and
then the key `u`: `▶ 26:36 / 30:29`, and the log says "the stream of the server
starts again at 1597 seconds".

### T-64: the build of the development filled the disk of the maintainer

The maintainer reported this on 2026-08-11: "we need to cleanup toutui's disk usage,
it's using over 200GB of space". The measurement:

| The place | The size |
|---|---|
| `target/debug` | **221 gigabytes** |
| `target/release` | 2 gigabytes |
| The files of `target/debug/deps` | 11963 |
| The free space of the disk | 356 megabytes of 1.8 terabytes |

**The cause is not a fault of the program.** A test binary of this project holds
every dependency and the whole debug information, and that gives about 300
megabytes. `cargo test` makes 36 such binaries. **cargo keeps the binary of every
build that came before**: a change of one line of the source gives a new name of the
file, and the old file stays for ever. A session of one day of work therefore left
hundreds of them.

`cargo clean --profile dev` gave 410 gigabytes back, and it kept the binary of the
release that the maintainer tests.

**The correction of the growth.** `[profile.dev]` holds `debug = "line-tables-only"`
now. That form keeps the file and the line of every frame, therefore a panic of a
test still names its place. It holds no type and no variable: a debugger of a step
gives less, and no test of this project needs one.

The measurement of the same build of every test binary:

| The debug information | `target/debug` |
|---|---|
| The whole (`debug = true`, the value of cargo) | 5.7 gigabytes |
| `line-tables-only` | 2.4 gigabytes |

**The rule for every session.** The growth of `target/debug` is about 300 megabytes
for each rebuild of the tests, and cargo removes nothing. Therefore:

- Look at `du -sh target` in a session of many builds.
- Run `cargo clean --profile dev` at the end of such a session. The release stays.
- Every cargo command runs under `nice -n 19 ionice -c 3` with `-j 16`: the machine
  has 32 cores, and the maintainer tests the program while the tests build.

**One run of the tests of ten gave a fault, and this session did not find it.** That
run came after a clean of the build, and nine runs after it gave 803 of 803 — with
`-j 16`, with `-j 4` beside a build of the release, and with eight threads of the
tests. The name of the test did not reach the log of that run. **A next session that
meets a fault of one run must keep the whole output of `cargo test`**, because the
name of the test is the whole answer.

### T-65: the reader kept every ebook for ever, and no key removed it

The disk of the maintainer became full on 2026-08-11, and T-64 holds the large part
of that: the build of the development. **A part of it belongs to the program**, and
this item holds that part.

The reader of T-10 writes the ebook of a media in the directory of the downloads,
and it keeps that file: a second visit of the book then needs no request, and the
reader works with no server. **Nothing removed such a file.**

- The key `X` removes the files of the tables `downloads` and `download_files`, and
  those tables hold the **audio** of a download only. The ebook stayed.
- A user who read an ebook and downloaded no audio had **no way at all** to remove
  it: `remove_download` gives nothing for a media with no row of a download, and it
  then did no work.
- T-54 makes this larger: an ebook of a PDF of a scan holds some hundred megabytes.
  The measurement of that item used a file of 137 megabytes.

**The correction.** `remove_the_ebook_of_the_item` removes the EPUB book and the PDF
of one item, and the key `X` calls it beside `remove_download`. The key therefore
does what its name says: it removes the local copy of that media, of every form.

`text_of_the_removal` gives the sentence, and it names what went away:

| The audio | The ebook | The sentence |
|---|---|---|
| Removed | Removed | `Removed the local copy of "X", and its ebook of 5 MB.` |
| Removed | None | `Removed the local copy of "X".` |
| None | Removed | `Removed the ebook of "X" of 137 MB. It held no local copy of the audio.` |
| None | None | `"X" holds no local copy and no ebook.` |

Both functions are pure, therefore a test holds every sentence with no file.

**The measurement in the real program.** A media whose ebook of 137 megabytes stood
on the disk. The key `X` gave: `Removed the ebook of "One File With No Decoder" of
137 MB. It held no local copy of the audio.`, and the file went away.

**What stays open.** The program removes no ebook of its own. A user who reads
twenty books of a scan therefore holds twenty files, and the key `X` of each media
is the answer. A limit of the whole cache would need a rule for the book that the
user reads now, and the value of that work is small while the key exists.

### T-66: a media that a different client finished stayed on Continue Listening

T-47 gave the program the live messages of the server, and the mark of a line takes
the new position at once. **The line itself stayed.** A different client finished a
book, and the shelf of Continue Listening of the server then held that book no more,
but the Home view of the program held its line until the user pressed `R`.

**The fault, measured on 2026-08-11.** The program ran in tmux against the sandbox,
and `curl` of a second client wrote `{"isFinished":true}` on
`PATCH /api/me/progress/:id` of "A Long Test Book":

```
────────────────────Home [22 items]────────────────────      the log of the program:
  ▌ Continue Listening                                       [live] user_updated: the position
  87% One File With No Decoder                                     of 10 media.
➤ ✓   A Long Test Book        <- the mark is new, the line stays
  ▌ Recently Added
```

The server gave one media of that shelf at the same moment:
`server CL: ['One File With No Decoder']`.

**The correction.** The program holds every line of the shelf already, therefore it
needs no request. `mediaProgress` of `user_updated` carries the whole account, and
two of its values keep a media away from that shelf: `isFinished` and
`hideFromContinueListening`. `the_media_away_from_continue_listening` reads them,
the box of `logic::live` holds the list, and the render makes the lines of the Home
view again when that list changed.

| The part | What it does |
|---|---|
| `api::live::the_media_away_from_continue_listening` | The identity of each media of the message that must leave the shelf |
| `logic::live::note_the_media_away_from_continue_listening` | Writes that list. The new list **takes the place** of the old one, because the message holds the whole account: a media that becomes unfinished comes back |
| `home_view::the_media_of_continue_listening` | `true` for each media of the Home view that stands on that shelf |
| `home_view::without_the_media_that_left` | The lines with the media that left away. A shelf with no line gives no name |
| `App::take_the_media_that_left_away` | The render calls it. It does nothing when the list did not change |

**The program asks the server for nothing.** The sync of its own playback makes one
`user_updated` every ten seconds, and each of those messages costs a comparison of
two small lists. The key `R` clears the list, because the shelf of the new request
holds none of those media already.

**A fault of the first form of this work, and the measurement that showed it.** The
list held the **identity** of a media, and `without_the_media_that_left` then took
away every line of that identity. One media stands on two shelves: the book stood on
Continue Listening and on Recently Added together, and the count of the view went
from 22 lines to 20. The server gives the line of Recently Added, and a book that
the user finished belongs there. **The list holds the number of the line now**, and
each shelf gives its own number.

**The measurement of the correction, 2026-08-11.** The program ran in tmux, and
**no key was pressed at all** for any of the three frames:

| What a second client did | The Home view |
|---|---|
| `{"isFinished":true}` | 22 lines to 21. The line left Continue Listening, and `✓ A Long Test Book` stays on Recently Added |
| `{"isFinished":false}` and then `{"progress":0.5}` | 21 lines to 22. The line came back |
| The key `N` of the program | 22 lines to 21. The line left at once |

The selection stays on the line of the user. A line that went away gives the line
that took its place, and never the top of the view.

The key `N` and the key of the mark said "Press R to see the change". That sentence
is wrong now, and both messages say what happened only.

**A trap of the server, measured 2026-08-11.** `PATCH /api/me/progress/:id` with
`{"isFinished":false,"progress":0.5,"currentTime":900}` gives `200`, and the record
then holds `progress` 0 and `currentTime` 0. **`isFinished` false writes the position
back to the start, and it ignores every other field of the same request.** A media of
progress 0 stands on no shelf of Continue Listening. Two requests do the work.

### T-67: the cache of the ebooks holds a limit now

T-65 gave the user the key `X`, and it left this open: **the program removed no ebook
of its own.** A user who reads twenty books of a scan holds twenty files, and the
measurement of T-62 used a PDF of 137 megabytes.

`src/logic/reader/cache.rs` holds the rule, and `LIMIT_OF_THE_CACHE` is one
gigabyte. That value holds some hundred EPUB books, or two of the largest books that
the reader opens: `MAX_BOOK_BYTES` is 512 megabytes for one book.

| The rule | Why |
|---|---|
| The book of the **oldest use** goes first | A book that the user reads every day must stay |
| **The book that the user reads now never goes away** | A cache that removes it asks the server for it again at once. One book of 500 megabytes is a correct cache of one book |
| The program stops **at** the limit | It must not empty the cache |
| The program looks at the limit after a **new** book came | That is the one moment when the cache grows. No frame and no key costs a read of the directory |
| A file that is not an `epub` and not a `pdf` stays | The audio of a download stands in a directory of the same parent |

**The time of the file is the time of the last use.** `the_book_is_in_use` writes it
with `std::fs::FileTimes`, and `get_the_ebook` calls that function for a book that
the disk holds already. This needed no new dependency.

`the_ebooks_that_must_go` is pure, and six tests hold it.
`tests/the_cache_of_the_ebooks.rs` measures the work on the disk with real files:
three ebooks of 1000 bytes with three times of use, one audio file in a directory,
and one file of text. That test writes `XDG_DATA_HOME`, therefore it stays alone in
its binary and it holds every part in one function. See the trap 25.

**The measurement in the real program, 2026-08-11.** The key `e` on "Alice in
Wonderland" wrote `8fda6e43….epub` of 136761 bytes at 17:02:43. A second run of the
program pressed `e` on the same book: the reader gave chapter 3 with no request, and
the time of the file became **17:03:55**. Therefore the program knows which book the
user read last.

**A limit that a user must know about.** The reader works with no server for a book
of the cache. A user who wants a book of that form on a journey must read it one
time, and a cache above one gigabyte then removes the book of the oldest use. The
program says the name of every file that it removes in the log.

**What the reader does not do.** The reader keeps the book of the session while the
user reads it. Therefore the key `h` and a second `e` give the book again with no
call of `get_the_ebook`, and the time of the file then does not change. The time
changes at the next run of the program, and that is the moment that matters for the
limit.

### T-68: the book of xHE-AAC of the user, and three faults that it showed

**T-53 and T-63 waited for this measurement, and it is complete now.** The user gave
the real file on 2026-08-11:

```
02_Depthless Hunger 2_[B0GGDKX4GP]_AAC-LC.m4b     1.5 GB
02_Depthless Hunger 2_[B0GGDKX4GP]_xHE-AAC.m4b    1.3 GB
```

```
ffprobe: codec_name=aac  profile=xHE-AAC  codec_tag_string=mp4a  duration=93278 s
```

A piece of 10 minutes with `-c copy` keeps `profile=xHE-AAC`, therefore the
measurement needs no file of 1.3 gigabytes. That piece stands in the sandbox as the
book "Depthless Hunger, Book 2".

#### The answer: ffmpeg reads most of that form, and it drops the rest

**A correction of the first form of this item.** The first measurement read the
sentence "Not yet implemented in FFmpeg, patches welcome" and it said that no program
plays xHE-AAC. **That is wrong**, and the user said so: mpv plays the file, "with a
lot of errors". The sentence names the frames that ffmpeg cannot read, and not the
file.

The measurement of 2026-08-11, of the same piece of 10 minutes:

| The program | The answer |
|---|---|
| mpv 0.41.0, 20 seconds | 196 lines "Error decoding audio", and the sound plays |
| ffmpeg 9.0, 20 seconds | 195 lines of the same fault. The same frames |
| ffmpeg 9.0, 60 seconds to a WAV file | **46.2 seconds of audio of 60 seconds** |

**Therefore ffmpeg reads about 77 percent of the frames**, and it drops the other 23
percent. The sound of mpv holds a hole at each frame that it drops, and that is the
whole cause of the sound that "is not smooth". T-69 gives the same sound to Toutui,
through the stream of the server, because ffmpeg of the server is the same program.

#### The steps of the stream of the server

| The step | The answer |
|---|---|
| symphonia of the program | "The format of the data has not been recognized." It **refuses** the file, and it gives no wrong sound |
| `codecsToForceAAC` of the server | `alac`, `ac3`, `eac3`, `opus`. xHE-AAC names itself `aac`, therefore the server **copies** it |
| ffmpeg of the server, first try, `-c:a copy` | "Could not write header (incorrect codec parameters ?)", and it stops with the code 183. **It writes no part** |
| The server, 10 seconds later | "Transcode never closed...", and it starts ffmpeg again with `-c:a aac` |
| ffmpeg of the server, second try, `-c:a aac` | "[dec:aac] Error submitting packet to decoder: **Not yet implemented in FFmpeg, patches welcome**" |
| ffmpeg 9.0 of this machine, on the whole file | The same sentence, 195 times in 20 seconds. The AAC-LC file of the same book decodes with no fault |

**T-53 expected LATM here, and no LATM exists.** ffmpeg stops at the header of the
transport stream, therefore no byte of that form ever reaches the client. The arm of
`HlsFile::open` for such a form stays, for a server that gives one.

**What the second try gives depends on the place of the media**, and both answers are
bad:

- **From the second 0:** the parts come after **10.5 seconds**, and they hold the
  full number of bytes (77 kB for 6 seconds). ffmpeg decodes some frames and it
  drops the others. **The sound plays, and it is not smooth**: the user heard that,
  and this is the whole cause of it.
- **From the middle (322 s):** the samples of the decoder hold NaN, therefore the
  **encoder** stops too: "Input contains (near) NaN/+-Inf", "Error encoding a frame",
  the code 234. The server then writes "Closing Stream" and "Deleted session data",
  and **every part of that stream answers 404 for ever**.

**T-69 makes this file play.** The place of the user decides where ffmpeg starts, and
a place beside a bad one works. The AAC-LC file of the same book stays the better
answer, because it holds every frame.

#### The three faults of the program that this measurement found

**1. The program said something false.** The old message was "The stream of the
server holds a form that the program cannot read", and the log said "The server does
not have this item". Both are wrong: the item stands in the library, and the program
never received a form at all. `classify_status` gives that sentence for a 404, and a
404 of a **part** is not a media that is absent. `the_sentence_of_no_part` and
`the_sentence_of_a_stream_that_ended` say the truth now, and
`the_message_of_a_stream_that_did_not_play` adds what the user can do. **The place
that meets the fault writes the sentence**, therefore no other place reads a text and
guesses the cause: `PlaybackState::why_the_start_did_not_work` carries it.

**2. The open gave up at the edge of the ten seconds of the server.**
`ATTEMPTS_OF_THE_OPEN` was 8, and that is 13.5 seconds. The server needs 10.5
seconds for the first part of its second try. **Therefore the same book played one
time and failed the next time**, and the measurements of this session showed both.
The value is 14 attempts now, and that is about 25 seconds.

**A long wait of the open must not cost the user their keys.** That open runs on the
thread of the engine. `PlayerCommand::the_user_does_not_want_the_open` names the
commands that stop it — `Start`, `Stop`, `Pause`, `SeekTo`, and `SeekBy` — and
`a_command_waits` of the engine gives the answer at each attempt. `SetVolume` and
`SetSpeed` are **not** in that group, because the sleep timer of T-24 sends them
while a playback runs. The log says "a command of the user stops the wait of the
open", and a measurement of 2026-08-11 showed it at the second 23 of an open.

**3. A stream that the server ended made the program wait for nothing.** The server
deletes the session, therefore every part answers 404 and the answer never changes.
The reader asks for the **playlist** after the second attempt of a part: a playlist
that answers 404 says that the stream is gone. **The measurement: 25 seconds of
waiting became 11.6 seconds and a true sentence.**

**4. The user read nothing at all.** `WAIT_FOR_A_FAULT` is 2500 milliseconds, and
its comment said that "the engine opens the decoders inside the command `Start`,
therefore the fault comes in some milliseconds". **That is true of a file and false
of a stream:** the open of a stream waits for ffmpeg of the server. The engine wrote
the fault at the second 11.6, and no loop read it any more. `WAIT_FOR_THE_STREAM` of
35 seconds holds the stream now, and a playback that starts costs nothing: the loop
stops at the first frame of the engine.

#### The measurement of the correction, 2026-08-11

The program in tmux, against the sandbox, with the media at 322 seconds of 600:

```
[HlsFile] the part of the stream is not ready. The reader waits.
[HlsFile] the server ended the stream of this media.                    <- 11.6 s
[worker] the engine cannot start the book: The server ended the stream of this
         media. Its ffmpeg cannot read the form of this audio.
```

And the row of the message of the screen, inside the frame:

```
The server ended the stream of this media. Its ffmpeg cannot read the form of this
audio. A file of a different form is the answer.
```

The keys `j` and `k` work after it, and the position of the media stays: the server
held `currentTime` 322 for a playback that began at 300 seconds, therefore the rule
of T-63 is correct for this file too.

**A trap of the message of the screen.** That row holds one line. The first form of
this message held 200 letters, and a terminal of 160 columns lost the end of it. A
test holds the message at 150 letters or fewer.

**A trap of the sandbox.** A transcode that dies leaves the server in a state where
it answers "No Segments" for every new session of that media, and it writes "Failed
checking files" every two seconds for ever. `podman restart abs-test` is the answer,
and a measurement of such a media must start from a server that came up now.

### T-69: the book of xHE-AAC plays now, from a place beside the bad one

The user said it: "MPV can play xHE-acc but it does it with a lot of errors so we
should be able as well." **They are right**, and T-68 gives the reason: ffmpeg reads
77 percent of the frames of that form. The program plays every codec of ffmpeg through
the stream of the server, therefore it must play this one too.

T-68 left one fault of the server between the user and the sound: **the place of the
media where ffmpeg starts decides if ffmpeg lives.** A seek to some places gives a
frame of NaN to the encoder, and ffmpeg then stops with the code 234 and the server
deletes the whole session.

**The measurement of the places, 2026-08-11**, of the book of the user in the sandbox:

| The place of the user | The part | The answer of the server |
|---|---|---|
| 0 s | 0 | 77268 bytes after 10.5 s |
| 60 s | 10 | 82156 bytes after 11.0 s |
| 180 s | 30 | 78960 bytes after 11.0 s |
| **310 s** | **51** | **it ended the stream** |
| **322 s** | **53** | **it ended the stream** |
| 316 s | 52 | 74448 bytes after 11.0 s |
| 328 s | 54 | 79148 bytes after 11.0 s |
| 480 s | 80 | 72004 bytes after 11.0 s |

**Two places of eight fail, and a place beside each of them works.** Therefore the
program tries more than one place: `the_places_to_try` gives the place of the user
first, and then the part before it. **A user hears a few seconds again more easily
than they lose a few seconds.** Three places are enough, and each one costs about 11
seconds.

**The trap that made the first form of this work useless.** The program moved the part
that it asked for, and nothing changed: the server answered with the same ffmpeg
command every time.

```
[STREAM] Starting Stream at startTime 4:52 (User startTime 5:22) and Segment #48
ffmpeg ... -ss 292s -noaccurate_seek ... -start_number 48 ...
```

**The place of ffmpeg comes from the position of the user, and not from the part that
the client asks for**, and the server takes 30 seconds of that position as a
pre-roll. Therefore `write_the_place` writes the position of the user before each
attempt after the first one. That position is true at once, because the program plays
that place. **A run that gives no stream at all writes the position of the user
again**, therefore a failure leaves nothing behind.

**The measurement of the correction, 2026-08-11.** The server held the position 322
seconds, and the program ran in tmux:

```
[play] the stream of the item bb9c73c7… starts at 322 seconds
[HlsFile] the server ended the stream of this media.                     <- 11.5 s
[play] the stream of the place 322 s did not play
[play] the place 322 s gave no stream. The program tries 316 s.
[play] the stream of the item bb9c73c7… starts at 316 seconds
[HlsFile] the stream holds 101 part(s). The reader starts at the part 52 and at
          4.0 seconds inside it. The audio is AdtsAac.
```

The screen said "The server gave no stream of that place. The program tries 05:16
now.", and the panel then held `▶ Depthless Hunger, Book 2`. The whole work took 23
seconds. A run of a place that works needs one attempt and 11 seconds.

The position stays correct: the panel gave `▶ 6:00 / 10:00` for a stream that began at
5:28, therefore the offset of T-63 holds for this file too.

**The session of each attempt closes.** The server would hold one session for each
place, and a session that stays open is the report `dd9a649`.

**What stays true.** The sound holds a hole at each frame that ffmpeg drops, and no
client can do better today: mpv gives the same holes. **The AAC-LC file of the same
book is the better answer for a user**, and this work makes the other file playable.

### T-70: a search of the name of an author gave no line

The search of the server came in an earlier session, and it left one hole. **The
group of the books of `GET /api/libraries/:id/search?q=` does not hold the name of an
author.** A measurement of 2026-08-11 against the sandbox:

```
q=carroll     -> book: 0   authors: 1 ("Lewis Carroll")   series: 0   narrators: 0
q=chronicles  -> book: 2   series: 1 (with its three books)
```

The screen of the program then held one line of the header and **no line of a book**:

```
─────────────Search result [the server also found: Lewis Carroll]─────────────
                          (and nothing below it)
```

The book of that author stands in the library, and the user cannot reach it.

**The correction.** The filter of the library gives the books of a name, and the
program measured both forms on 2026-08-11:

| The request | The answer |
|---|---|
| `?filter=authors.MzEyYzQyZmY…` (the identity of the author in base64) | Alice in Wonderland |
| `?filter=narrators.QSBUZXN0IE5hcnJhdG9y` (the name of the narrator in base64) | A Long Test Book, Alice in Wonderland |

`the_names_to_ask` gives the names of the answer with their filter, and the task of
the search then asks `get_all_books` for each of them. **An author comes before a
narrator**, because a user who writes a name looks for the writer more often, and
`NAMES_TO_ASK` of 3 holds the number of requests.

**A tag and a genre do not come.** The view of the filter of T-60 holds them, and a
genre of a large library gives some hundred books.

**A view says why it holds no line.** The old title said "Search result [from the
server]" for an answer of nothing. `the_title_of_the_search` is pure and it holds
every case:

| The condition | The title |
|---|---|
| The server found nothing | `The server found nothing for "zzzznothing". Press / to write other words.` |
| The answer did not come yet | `The program looks in its own titles. The answer of the server comes.` |
| The answer holds a name | `Search result [1 items, with the books of Lewis Carroll]` |
| The answer holds no name | `Search result [1 items]` |

The name is the **reason** of a line now, and not a note beside it.

**The measurement in the real program, 2026-08-11**, of the three conditions:

```
"carroll"       -> Search result [1 items, with the books of Lewis Carroll]
                   ➤ Alice in Wonderland
"test narrator" -> Search result [2 items, with the books of A Test Narrator]
                   ➤ A Long Test Book / Alice in Wonderland
"zzzznothing"   -> The server found nothing for "zzzznothing". Press / to write other words.
```

**What the search still does not do.** The lines of that view come from the lists of
the library, therefore a book that the program did not load gives no line.
`get_all_books` reads every page at the start, and `MAX_PAGES` of 500 with
`PAGE_SIZE` of 500 holds 250000 items. A library that is larger than that needs a
view that holds the title of the answer of the server itself.

**A trap of the harness.** The key `Escape` of the view of the search closes the
program. A sweep of the views must use `/` again, or `Tab`.

### T-71: the program removed a book of the user with no word

T-67 gave the cache of the ebooks its limit, and it left this open: **the program
removed a file of the user and it wrote that in the log only.** A user who keeps a
book of a scan for a journey then loses it with no word on the screen.

`the_sentence_of_the_cache` gives the sentence, and the render draws it in the row of
the message of **every** view. The reader is the view of that moment, therefore the
user reads it there:

```
The cache of the ebooks was full. The program removed 1 book(s) of the disk, and it
gave 133 kB back. Press e to get one again.
```

The sentence names the key that gets the book again, because the answer of the user to
that message is `e`.

**A measurement of the real program needs a small limit.** A cache of one gigabyte
needs some hundred books, and no session makes them. `TOUTUI_EBOOK_CACHE_BYTES` holds
a number of bytes, and it has the shape of `TOUTUI_NO_COVERS` and of
`TOUTUI_AUDIO_DEVICE`. A value that is not a number, and the value 0, give the limit
of the program: **a cache of 0 bytes would remove every book of the disk.**

**The measurement of 2026-08-11.** The sandbox holds two books with an EPUB book of
136761 bytes each, and the program ran with `TOUTUI_EBOOK_CACHE_BYTES=200000`:

| The step | The answer |
|---|---|
| The key `e` on "A Long Test Book" | The reader opens. The cache holds one file |
| The key `e` on "Alice in Wonderland" | The reader opens, and the file of the book before it goes away |
| The row of the message, 2 to 6 seconds | The sentence above |
| The log | `[reader] the cache of the ebooks gave 136761 bytes of 1 book(s) back.` |
| The disk | One file of the cache |

**A trap of the measurement.** A message lives **six** seconds (`message::LIFE`). A
capture of the screen 9 seconds after the key gives no message, and the program is
correct. Capture inside that time.

### T-72: the limit of the cache of the ebooks belongs to the user

T-71 gave `TOUTUI_EBOOK_CACHE_BYTES` for a measurement, and the user of a small disk
had no way to change the limit. **A limit of the disk of a user belongs in
`config.toml`.**

The block is new, and every value of it is optional:

```toml
[reader]
ebook_cache_mb = 2048
```

`load_config` reads that block with `unwrap_or_default`, therefore a file of an older
version keeps working. This is the shape of the block `[[servers]]` of an earlier
session.

**The sequence of the three sources, and the reason for it:**

| The source | Why it stands there |
|---|---|
| `TOUTUI_EBOOK_CACHE_BYTES` | A measurement of the real program must not change the file of the user |
| `ebook_cache_mb` of `config.toml` | The value of the user |
| `LIMIT_OF_THE_CACHE`, of one gigabyte | A file with no block, and the value 0 |

**The value 0 cannot mean itself**, because a cache of 0 bytes would remove every
book of the disk. It gives the source that comes after it.

**The cache runs inside a task, and that task holds no `App`.** Therefore the start
of the program writes the value in a slot of `logic::reader::cache`, one time. This
is the shape of `logic::live` and of `logic::message`.

**The measurement of the real program, 2026-08-11.** The program writes the limit
that it uses in the log, therefore a user can see which source gave it:

| The condition | The log |
|---|---|
| `ebook_cache_mb = 1` | `the cache of the ebooks holds 1048576 byte(s) at the most.` |
| The same, with `TOUTUI_EBOOK_CACHE_BYTES=200000` | `… holds 200000 byte(s) at the most.` |
| A file with no block `[reader]` | `… holds 1073741824 byte(s) at the most.` |

**Why the log and not the screen.** Two books of the sandbox hold 133 kB each, and a
limit of 1 megabyte removes neither of them. A measurement of the **resolution** of
the limit therefore cannot come from the files of the disk: it needs the number that
the program holds. One line of the log gives it, and it costs the user nothing.

### T-73: the narrators of a library, with the key `v`

Section 4 of `docs/T-24-coverage.md` said `No` for "The narrators of a library", and
section 6 does not forbid it: the work needs no library of the system, it makes no
public address, and a terminal shows a list of names well.

**A narrator of the server holds the shape of an author.** A measurement against an
Audiobookshelf 2.36.0 on 2026-08-11:

```json
GET /api/libraries/:id/narrators
{ "narrators": [ { "id": "QSBUZXN0IE5hcnJhdG9y", "name": "A Test Narrator",
                   "numBooks": 2 } ] }
```

Therefore **one view holds the two lists**, and `logic::authors::Kind` says which list
the view holds. The key `a` gives the authors and the key `v` gives the narrators.
`Kind` holds every sentence of the two lists, and it is pure.

**The one difference between the two lists, and it matters.** The filter of an author
takes the **identity** of that author, and the filter of a narrator takes the
**name**: the server holds a narrator inside the metadata of a file, and not as a row
of its own. The identity of the answer is already the name in base64
(`QSBUZXN0IE5hcnJhdG9y` for "A Test Narrator"), and `filter_of` does not depend on
that form.

**A new list forgets the answer of the list before it.** The view would else show the
authors under the title of the narrators, because the two lists share one slot.

**The measurement in the real program, 2026-08-11:**

| The key | The screen |
|---|---|
| `a` | `The authors [6 items]`, with "Decoder Test [1 book(s)]", "Lewis Carroll [1 book(s)]", … |
| `v` | `The narrators [2 items]`, with "A Test Narrator [2 book(s)]" and "Jonathan Davis [1 book(s)]" |
| `l` on "A Test Narrator" | `Library [2 items] — a filter is on (f)`, with "A Long Test Book" and "Alice in Wonderland" |

The database then held `narrators.QSBUZXN0IE5hcnJhdG9y`, and that value agrees with
the measurement of `curl` of T-70.

**The key `v`.** Eight letters had no key: `d`, `m`, `r`, `v`, `w`, `x`, `y`, and `z`.
`v` says "voice", and `src/ui/keys.rs` holds the line of the key `?`. The test
`every_key_of_the_handler_stands_in_the_list` holds that rule.

### T-74: the run of the tests took 18.7 seconds, and it takes 2.2

The maintainer said that the tests take much time. The session of 2026-08-11
measured every part, and `cargo test` was not the largest part: a build after
one edit takes 6 seconds and the run took 18.7. **Two tests held 12.7 seconds of
that run**, and `cargo test` runs the 37 test binaries one after the other.

**The trap of such a wait, and it is real.** Some of those sleeps give a
**fault** the time to appear. `the_position_survives_a_playback_that_does_not_start`
asks: does the loop of the playback write the **wrong** position of an engine
that did not start? A poll of "the position is the position of the user" answers
`true` before the loop ever ran, because the row of the database holds that value
already. **That poll is a false pass.**

Two answers, and each fits its test:

- **A clock of the test.** `#[tokio::test(start_paused = true)]` gives the test
  its own clock: each `sleep` of the code still gives the loop its steps, and it
  takes no real time. The test took 8.01 seconds, and it takes 0.01 now. **The
  fault still appears:** a build with the correction of T-38 removed fails with
  "the position went to 0 seconds". `test-util` is a feature of tokio, and the
  resolver of the edition 2021 keeps a feature of a dev-dependency away from the
  binary of the release, therefore the rule of T-20 stays.
- **A poll of the evidence that the loop acted.** `playback_ownership` waits for
  a server (wiremock), therefore it must not take a clock of its own: the clock
  would move to the timeout of a request while that request is still on its way.
  The key `F` of the forced sync asks the loop to send its position at its next
  step, therefore `POST /api/session/:id/sync` is the evidence that the loop read
  the state of **its own** playback. The two tests took 7.7 seconds, and they
  take 3.0. A build with the correction removed fails after the limit of the
  poll.

**The flag of the forced sync holds one identity for the whole process.** The two
tests of that file run at the same time, therefore each of them holds its own
identity of a playback, and the poll asks again at each step.

`cargo nextest` runs every test of every binary in one pool of processes.
`.config/nextest.toml` holds the tests of the sandbox in a group of one thread,
for the limit of the rate of the login of the server.

| The work | Before | After |
|---|---|---|
| `cargo test` | 18.7 s | **8.7 s** |
| `cargo nextest run` | 8.6 s | **2.2 s** |
| The tests of the sandbox, with `--run-ignored all` | — | 14.2 s, 18 of 18 |

Ten runs of the whole suite gave 838 of 838 each time.

### T-75: two texts of the screen that a sweep of the views found

The sweep of the thirteen views with `docs/harness/drive.sh` took 9.7 seconds. A
sweep with a `sleep` takes about two minutes, and the session before this one
made about 30 such measurements.

- **The footer of the list of the narrators said "the books of this author".**
  One view holds the authors and the narrators (T-73), and the footer held the
  word of one list. `Kind::work_of_the_key_that_opens` gives the word of each
  list now.
- **The text of the accounts of the settings held a run of 22 spaces**: "the
  program                      forgets the token". An old wrap of the source
  stayed inside the string, and `Wrap` takes a space away at the start of a line
  that it makes and keeps every space that stands inside a line. The two texts of
  the settings stand as constants of `src/ui/keys.rs` now, and
  `a_text_of_a_view_holds_no_run_of_spaces` holds every such text to one space.

**A view key works in the Home view, in the Library view, and in the view of the
search only.** The first sweep pressed `a`, `v`, `c`, and `f` inside the view of
the series, and nothing came. That is the rule of `show_the_names`, of
`show_the_sequence_and_the_filter`, and of the keys `s` and `c`, and it is not a
fault. A sweep must come back to the Library between two views.

### T-76: an item can hold more than one ebook

Section 4 of `docs/T-24-coverage.md` said `Half` for "The list of the ebooks of an
item": the program takes `media.ebookFile`, and that field names one book.

**The measurement of 2026-08-11.** An EPUB book stood beside the PDF book of one
item of the sandbox:

```text
ebookFile: A Book Of The Test.pdf
 file: A Book Of The Test.pdf  ebook  ino 6121534
 file: A Second Book.epub      ebook  ino 94488

GET /api/items/:id/ebook           200  53688 bytes  application/pdf
GET /api/items/:id/ebook/6121534   200  53688 bytes  application/pdf
GET /api/items/:id/ebook/94488     200 136761 bytes  application/epub+zip
```

Therefore the address of one ebook is the address of the ebook of the item with
the `ino` of the file after it, and `libraryFiles` names every file.

**The key `e` inside the reader gives the list.** The key `e` of a list opens the
book of the server, as before. The reader takes every key of the program,
therefore `e` was free there, and no key of the program changed its work.

**The trap, and the rule that comes from it.** The server holds one place for
each **media** (`ebookLocation`), and not one place for each file. A reader of a
second book of the same item would write the place of that book over the place of
the book of the server, and the user would lose their line. Therefore a book that
is not the book of the server **keeps its place on this machine**: the reader
neither reads the place of the server nor writes it, the key `s` says why, and
the view says the rule.

Each book takes its own name on the disk: the book of the server keeps the name
of the item (a user of a version before this work gets no file a second time),
and every other book takes `<the item>-<the ino>`. The key `X` removes every
ebook of the item, and it reads the directory for that work.

**The measurement in the real program, 2026-08-11:**

| The key | The screen |
|---|---|
| `e` on "One File With No Decoder" | `One File With No Decoder — page 1 of 3 — 0%`, the PDF of the server |
| `e` inside the reader | `The books of this media [2 items]`, with "A Book Of The Test.pdf (the book of the server)" and "A Second Book.epub" |
| `l` on the second line | `Alice's Adventures in Wonderland — chapter 1 of 14 — 0%` |
| `s` in that book | "This is not the book of the server. The place of this book stays on this machine." |

The disk then held `<the item>.pdf` and `<the item>-94488.epub`.
`tests/the_ebooks_of_an_item_against_the_sandbox.rs` holds the whole path, and it
carries `#[ignore]`.

### T-77: the settings write `config.toml`, and they keep every comment

T-72 gave `config.toml` the block `[reader]` with `ebook_cache_mb`, and the user
had to open the file with an editor. The view of the settings showed the values
of the program, and it changed none of them.

**The file belongs to the user.** `config.example.toml` is 58 lines, and almost
every one of them is a comment that says what a value does. A writer that makes
the file again from the values of the program would remove every one of them, and
the file of the user holds their own comments too.

Therefore `with_the_value` changes **one line** and it keeps every other line:

- The block holds the key: that line takes the new value, and it keeps the spaces
  at its start.
- The block exists and holds no such key: the key comes after the last value of
  the block.
- The block does not exist: the block and the key come at the end of the text.
  **A block that stands inside a comment is not a block**, therefore the
  `# [reader]` of the example file stays a comment and a real block comes at the
  end.

The function is pure, and a test gives it `config.example.toml` and finds every
line of that file in the answer. A second test parses the answer and reads the
value back, and a third holds a key whose name is longer (`ebook_cache_mb_old`)
away from the key.

**The write is atomic.** The bytes go to `config.toml.new` beside the file, and a
rename puts that file in place: a program that stops in the middle of a write must
not leave the user with half a configuration file.

**The view.** The line "The reader: the cache of the ebooks" of the settings opens
a list of six values, and the key `l` writes the value that the user takes. The
program uses that value **at once**: `keep_the_limit_of_the_configuration` holds
the slot that the task of the cache reads, therefore the user starts the program
no second time. The view offers values only, therefore the program examines no
line of text of the user, and the file still takes any value by hand.

**The measurement in the real program, 2026-08-11:**

| The key | What happened |
|---|---|
| `S`, then `G`, then `l` | `The cache of the ebooks — 1024 MB now`, with the mark `✓` on "1024 MB (the value of the program)" |
| `j`, then `l` | "The cache of the ebooks holds 2048 MB now. config.toml has the value." |
| The file | Three lines came at the end (a blank line, `[reader]`, and `ebook_cache_mb = 2048`), and the 58 lines of the user did not change |
| `g`, `j`, then `l` | The same line took the value 512. The file holds 61 lines and one block `[reader]` |

### T-78: the message of the program kept the letters of the view below it

The sweep of the keys of the reader of 2026-08-11 read this row of the screen,
after the key `s`:

```text
CHAPTER IV.       │The Rabbit SeThe server has the place of the book.
```

**A `Paragraph` of ratatui gives its style to every cell of its area, and it
writes its own text only.** The background of the row therefore changed, and
every letter that the view wrote on that row stayed. The message stands one row
above the footer (T-59), and in every list view that row holds no letter:
therefore no session saw this before the reader.

`Clear` takes the row away before the message comes.
`draw_the_row_of_the_message` holds that work, and a test writes a line of a view
in a buffer, draws the message over it, and reads the row back. A build with the
`Clear` removed gives:

```text
left: "CHAPTER The server has the place.n a Bil"
```

**The sweep found this, and no test did.** A message of the program lives six
seconds, and every measurement of a message before this one read a list view.

### T-79: the key `h` of the view of the search did nothing

The sweep of 2026-08-11 searched "zzzqqq", read "The server found nothing for
\"zzzqqq\". Press / to write other words.", and pressed `h`. **The screen did not
move.** Every other view of the program goes back one step with that key, and
the handler held no line for `AppView::SearchBook`.

The footer of that view was the footer of the Library, and it named `Tab:
home/library` therefore the user was not held there. **A key that does nothing
in one view of fifteen is a fault of its own**: the user learns the key in every
other view.

`FOOTER_OF_THE_SEARCH` names the keys of that view now (`h: back` and `/: search
again`), and `the_view_before_the_search` holds the view that opened the search:
the search of the Home view goes back to Home, and the search of the Library goes
back to the Library. A second search inside the view of the search keeps the
first view.

**The measurement in the real program, 2026-08-11:**

| The keys | The screen |
|---|---|
| `/`, "alice", Enter, `h` from Home | `Home [24 items]` |
| `/`, "zzzqqq", Enter, `h` from the Library | `Library [7 items]` |
| `/`, "alice", Enter, `/`, "wonder", Enter, `h` | `Library [7 items]` |

### T-80: the keys of the volume answered with nothing

The sweep of the keys of the player of 2026-08-11 pressed `o` and `i` and read
the screen after each of them. **No row moved.** The row of the player names the
speed and it named no volume, and the keys wrote no message: a user who presses
`i` some times therefore hears less and reads nothing, and a media that plays and
gives no sound looks like a fault of the program.

- **The message says the new value.** 0% says "the media plays and you hear
  nothing", 100% says "the volume of the file", and a value above 100% says "more
  than the file".
- **The row of the player names the volume when it is not 100%.** That value is
  the value of almost every playback, and a row of 80 columns holds little.

**A second fault stood behind the first one.** `handle_key_player` read
`state.volume` of the engine, and the engine writes that value at its next tick
only: two keys inside one tick both read the old value, and the second key gave
no step. `PlayerHandle::change_the_volume` writes the new value in the state at
once. A measurement pressed `i` ten times with 80 milliseconds between them, and
the volume went from 100% to 0% with every step.

The engine held the value between 0 and 2 before this work, therefore no volume
of a negative number ever reached the sound card.

**The measurement in the real program, 2026-08-11:**

| The keys | The row of the player | The message |
|---|---|---|
| `i` | `… | Speed: 1.30x | Vol: 90%` | "The volume is 90%." |
| `i` `i` | `… | Vol: 70%` | "The volume is 70%." |
| ten times `i` | `… | Vol: 0%` | "The volume is 0%: the media plays and you hear nothing." |
| `o` | `… | Vol: 10%` | "The volume is 10%." |

### T-81: the queue of the episodes that the server downloads, with the key `d`

The key `E` tells the server to get every episode of a feed that it does not
hold, and **the server does that work alone**. The program showed nothing of it:
a user who pressed `E` on the feed of the sandbox (57 episodes) read one message,
and no view said what happened after it. Section 5 of `docs/T-24-coverage.md`
named this work, and section 6 forbids it not.

**The measurement of 2026-08-11, against an Audiobookshelf 2.36.0:**

```json
GET /api/libraries/:id/episode-downloads
{ "currentDownload": { "episodeDisplayTitle": "Letter 4", "libraryItemId": "…",
                       "podcastTitle": "Letters of Two Brides", … },
  "queue": [ { "episodeDisplayTitle": "Letter 5", … }, … ] }
```

Three properties of the server, and each of them changed the work:

1. **The episode that downloads now is not in `queue`.** The view holds it at the
   top with the mark `▼`.
2. **`GET /api/podcasts/:id/clear-queue` gives `200`, and it does not stop the
   episode that downloads now.** The log of the server wrote "Successfully
   downloaded podcast episode \"Letter 12\"" after the clear. The text of the view
   says it, and the message of the key `X` says it again.
3. **The queue does not fill at once.** `POST /api/podcasts/:id/download-episodes`
   answers `200`, and a read two seconds later gave an **empty** queue. The clear
   three seconds after that removed nine episodes. A measurement of this endpoint
   must poll, and a sleep of two seconds says "the server queued nothing".

**The view asks by itself.** `grep -rho "episode_download[a-z_]*"` of the server
gives four messages of socket.io: `episode_download_queued`,
`episode_download_started`, `episode_download_finished`, and
`episode_download_queue_cleared`. The task of the live messages (T-47) writes a
mark, and the render asks the server again at the next frame. A connection of the
live messages that is not open gives no message, therefore the view asks again
after three seconds too. **The user presses no key**, and the list moved from 9
lines to 7, to 6, and to 4 in the measurement while the screen stood open.

**The key `X` asks one time**, as the key of the log out does (T-36): the queue
holds the work of the server, and a key at a wrong moment costs the user every
episode of it.

**The measurement in the real program, 2026-08-11:**

| The keys | The screen |
|---|---|
| `d` in a library of books | "This library holds books. The server downloads the episodes of a podcast only." |
| `d` in the library of the podcasts | `The downloads of the server [9 items]`, with `▼ Letter 31 — Letters of Two Brides…` |
| nothing, four seconds later | `[7 items]`, then `[6 items]`, then `[4 items]` |
| `X` | "Press X again to empty the queue of \"Letters of Two Brides…\". Any other key stops this." |
| `X` again | "The queue of \"Letters of Two Brides…\" is empty now. The episode that downloads goes on." |
| the view after it | "The server downloads no episode. Press E on a podcast to get its new episodes." |

### T-82: the choice of the library changed nothing on the screen

The measurement of T-81 needed the library of the podcasts. `S`, then "Library:
choose the library", then `l` on "Podcasts": **the header still said
"📖 Books (book)"**, every list of the screen stayed, and the program wrote no
message. The log held "[update_id_selected_lib] The library has been updated".

The key wrote the row of the database and nothing else. The user then reads the
library of the books until they press `R` or start the program again, and the
text of that view says "The key l on a library makes it the library that the
program shows."

`must_refresh` does that work already for the sequence and for the filter of the
library (T-24), therefore the key takes it now, and the program says "The program
shows the library \"Podcasts\" now." The header of the next frame holds
"📖 Podcasts (podcast)".

### T-83: the key `s` of a library of podcasts said nothing

The sweep of the views of a library of podcasts of 2026-08-11 pressed `s`, `a`,
and `v`. The keys `a` and `v` said "A library of podcasts has no author" and
"…no narrator". **The key `s` said nothing at all**, and no line of the screen
moved.

This is the fault of T-79 in a different place: the user learns the key in a
library of books, and one library of two answers with nothing. The key says "A
library of podcasts has no series." now.

The same sweep read every other key of that library, and each of them was
correct: `c` gives the collections and the playlists of the podcasts, `f` gives
the three fields of the sequence of a library of podcasts, `A` opens the line of
the name of a new podcast, `E` reads the feed and gives the episodes to the
server, `T` gives the time that the user listened, and `l` gives the episodes of
the podcast.

### T-84: the media of a collection and of a playlist, with the keys `m` and `X`

Section 4 of `docs/T-24-coverage.md` said `Half` for the collections and for the
playlists: "The client reads and plays. It cannot make a collection, add a book,
or remove a book." A user who wanted a book in a playlist opened the web page of
the server.

**The measurement of 2026-08-11, against an Audiobookshelf 2.36.0:**

| Request | Answer |
|---|---|
| `POST /api/collections/:id/book` with `{"id":"<the item>"}` | `200`, and the whole collection |
| the same request a second time | **`400`, "Book already in collection"** |
| `DELETE /api/collections/:id/book/:itemId` | `200` |
| `POST /api/playlists/:id/item` with `{"libraryItemId":"…"}` | `200`, and the whole playlist |
| the same request a second time | **`400`, "Item already in playlist"** |
| `DELETE /api/playlists/:id/item/:itemId` | `200` |

- **The key `m`** of a media opens the list of the collections and of the
  playlists of the library, and the key `l` puts the media in the line that the
  user took.
- **The key `X`** of the view of the media of a list takes the media of the line
  out of that list.
- **A collection holds books.** The server refuses an episode of a podcast,
  therefore the program says it before the request.
- **The 400 of the server is not a fault of the program.** `put_in_the_list`
  gives `Ok(false)` for it, and the message says "stands in the playlist
  already".

**The lines of the screen come after the write.** The first form of this work
asked the server for the lists **beside** the write, and the view then showed the
state of the moment before the key: the title said "[2 items]" after the key `X`
took one media out. The question stands inside the task of the write now, after
that write. `logic::the_lists` holds the answer, and the render takes it.

`tests/the_lists_against_the_sandbox.rs` holds the whole path: it puts a book in
every list of the library, it reads the 400 of the second request, and it gives
the lists back.

### T-85: "1 items"

The measurement of T-84 read `A Test Playlist [1 items]` after the key `X` took
one media out of a playlist of two. `ListView::line` held the rule of the
singular already, and no title of a view held it.

`crate::ui::keys::items` gives "1 item" and "2 items", and every title of every
view uses it now: Home, the Library, the series, the episodes, the queue, the
bookmarks, the chapters, the lists, the media of a list, the books of a media,
the downloads of the server, the authors, the narrators, and the settings of the
library.

### T-86: the test of the requests of the start measured the machine

`the_four_requests_of_the_start_go_together` gave the mock server a delay of 700
milliseconds for each answer, and it failed when the whole start took more than
2 seconds. **That is a measurement of the machine as much as of the program.**
This session ran the suite with a build and a program of tmux beside it, and one
run of twelve failed: the start took 4.2 seconds, and no line of the program had
changed.

The test holds the **time of each request** now, with a rule of `wiremock` that
writes `Instant::now()`. Four requests that go together arrive inside some
milliseconds of each other; four requests that wait for each other arrive 700
milliseconds apart. A measurement of 2026-08-11 gave
`[0 ms, 705, 705.5, 705.6, 705.6, 705.7]`: the examination of the address of the
pool first, and then the list of the libraries with the four requests of the
start, all together. **The time between the first request of the start and the
last one is 0.5 milliseconds, and the load of the machine does not change it.**

The first request of that list is the examination of the address, and it stands
outside the rule.

### T-87: one answer of 400 took the address of the server away

The test of T-84 failed with "No server address answered", and the real program
failed in the same way: a second `m` on the same media gave the message "stands
in the playlist already", and **every request after it said that the program had
no server**.

`ApiError::is_endpoint_fault` held every status of `Server` as a fault of the
address. `send` therefore called `mark_down` for a **400**, the pool of one
address then had no address with the state `Up`, and `active()` gave `None`. The
program came back when the examination of the address ran again.

**The server of Audiobookshelf answers 400 for work that a user does every day:**
a book that stands in a collection already, an episode that stands in a playlist
already, and a podcast whose directory exists (T-24 holds that measurement).

A fault of the endpoint is now a transport fault or a status of **500 or more**:
the server answered and it understood the request, therefore a different address
of the same server gives the same answer. The measurement in the real program of
2026-08-11: the second `m` gave the message of the 400, and the key `T` after it
gave the statistics of the user.

### T-88: a view that makes a collection or a playlist, with the keys `c` and `p`

T-84 gave the media of a list, and the program made **no** list: a user who
wanted a new playlist opened the web page of the server. A library that held no
list showed a message of one row, "The web page of the server makes one", and the
view did not open at all.

**The measurement of 2026-08-11 decided the shape of this work.** The two
requests do not behave in the same way:

| Request | Answer |
|---|---|
| `POST /api/collections` with `books` of one item | `200`, and the whole collection |
| `POST /api/collections` with **no** `books` | **`400`, "Invalid collection data. No books"** |
| `POST /api/playlists` with **no** `items` | `200`, and an empty playlist |
| Either, with a name of no letter | **`400`, "Invalid … data"** |
| Either, with a name that a list of the library holds | `200`, **and a second list of that name** |

**A new collection therefore needs a media**, and a view of its own with a name
and a library would fail for every collection. The key `m` holds a media
already, therefore the work stands in that view: the key `c` makes a collection
of the media of the line, and the key `p` makes a playlist of it.

The rules that the measurement gave:

- **The view opens for a library that holds no list.** The title says the
  condition and the two keys: an empty box says nothing.
- **A collection holds books.** The program says it before it asks for a name,
  and it makes no request for an episode of a podcast.
- **A name of no letter needs no request.** The program says the reason.
- **The program refuses a name that a list of that kind holds already.** The
  server takes that name and it gives a second list its own identity: the
  measurement made two lines "Measure Collection [1 item]" that no key of the
  user tells apart. The comparison ignores the case and the spaces of the two
  ends. A collection and a playlist of one name stay apart, because the line of
  the screen names the kind.
- **The lines come after the write**, as they do in T-84.

The measurement in the real program, in tmux against the sandbox: the key `c`
with the name "A Test Collection" gave "A collection of the name … exists
already", the name "The Books I Want" gave "The collection … exists now, and it
holds "A Long Test Book"", and the view held eight lines after it. `curl` of the
server read the new collection with that one book. The key `p` on an episode of
a podcast made a playlist that holds the **episode**, and the key `c` on that
same episode said "A collection holds books only".

### T-89: the box that takes a text left two columns of the view on the screen

The measurement of T-88 read a letter `T` beside the left border of the box that
asks for a name, and **that letter stayed after the box went away**. It is the
first letter of the text of the view below the box.

`ask_for_a_text` draws its box at the column 1, with a width of two columns fewer
than the screen. The wipe at the end wrote the same rectangle, therefore the
column 0 and the last column kept the letters of the view. **The rest of those
rows never came back either**: the box makes a `Terminal` of its own, and the
terminal of the program writes the cells that changed only — it knows nothing of
the letters that a different terminal wrote over.

Two answers together:

1. The box writes `Clear` and its background over the **whole rows**, before it
   draws and after it.
2. `App::the_screen_must_be_drawn_again` says that a box took the cells. The loop
   of the program then calls `terminal.clear()`, and the next draw writes every
   cell. T-42 holds the same answer for the refresh of the key `R`.

The fault belonged to every box of the program: the name of a bookmark (`b`), the
name of a podcast (`A`), and the name of a new list (`c` and `p`). The
measurement after the correction: the two lines of the text of the view come
back, and no letter of the box stays.

### T-90: every footer lost its end in a terminal of 80 columns

**The sweep of 80 columns is one of the sweeps that no session had made.** The
first screen of that sweep, a Home view of a library of podcasts in a terminal
of 80 by 24, ended with:

```
j/k: move  l: the episodes  Tab: home/library  /: search  R: refresh  ?: every k
```

The keys `?` and `Q` went away. **Those are the two keys that a lost user
needs**, and T-52 holds that rule: "A screen that names no key looks like a
program that stopped."

The footer of a library of podcasts holds 94 letters, and the footer of a list
holds 83. **The area of the footer holds two rows**, and `render_footer` wrote
on one of them: a `Paragraph` with no `Wrap` writes one row for each line of its
text, and it cuts every letter after the width. The module `ui::keys` said that
every footer fits in 80 columns, and no footer of more than 80 letters did.

The footer wraps now, therefore a wide terminal draws one row as it did before,
and a terminal of 80 columns draws two. A sweep of thirteen views after the
correction found no other text that goes away at that width.

`the_screen_survives_a_short_list` draws every view in 120 by 40 **and in 80 by
24** now, and it holds the last word of the footer on the screen. A build with
the wrap removed fails with "the view Home lost the end of its footer in 80
columns".

### T-91: the program said that the library holds no list, and the server was down

**The sweep of the offline mode is the second sweep that no session had made.**
`podman stop abs-test`, and then every key of the Library view. The program
answers well almost everywhere: the header says "Offline", the row of the media
says "This media plays from the disk", and the views of the authors and of the
narrators say "The server gave no author: the server does not answer".

**Three views said something that is not true.** With no answer from the server:

| The key | What the view said | What is true |
|---|---|---|
| `c` | "This library has no collection and no playlist." | No request gave an answer |
| `s` | "This library has no series." | The same |
| `m` | "This library holds no collection and no playlist. Press c or p to make one." | A new list needs the server |

The third one is the worst of the three: it names two keys that cannot work,
because a collection and a playlist stand on the server. The key `c` of that
view now says the reason and it asks for no name.

The rule: **a view that holds no line must say the reason of that condition, and
the program must not give a reason that it does not know.** The view of the
authors held the rule already, and `App::is_offline` gives it to the three views
of this item.

The sweep of the empty library of the same day found one text of a different
kind: "Press 'h' to go back." in three views, and "Press h to go back." in the
view of the chapters. One program says one thing in one way, therefore a test
reads `src/ui/tui.rs` and holds every text to the second form. The view of the
episodes said "No episodes found for this podcast." too, and it says "This
podcast has no episode." now.

### T-92: the login said "ERROR: Login failed" for every fault

**The sweep of the view of the login is the third sweep that no session had
made.** A fresh `XDG_CONFIG_HOME` with a `config.toml` and a `.env` gives that
view (the trap 7 of the harness). Two of its three messages are good work
already:

```
The address must start with http:// or https://. Write http://localhost:13399
http://127.0.0.1:1 does not answer. Is the server running?
```

**A wrong password gave "ERROR: Login failed".** Four items came out of that
screen:

1. **`auth_process` made one error for every status.** The status of the answer
   holds the reason, and `the_sentence_of_a_login_that_failed` gives it now. The
   status that costs the most time is **429**: the server permits 40 requests of
   the login in 600 seconds (the trap 22), and a user who writes their password
   again and again reaches it. The old four words sent them to look for a fault
   that does not exist. That sentence now says "Wait 10 minutes."
2. **"ERROR: " stood before the text**, and no other message of the program
   holds that word.
3. **An empty username and an empty password went forward with no word.** The
   field of the address refused an empty value already, and the two other fields
   did not. They say "Write your username." and "Write your password." now, and
   they make no request.
4. **A login that failed emptied the field of the address.** The user wrote the
   whole address of their server again after each wrong password, and that
   address answered `/ping` some seconds before. The program keeps it in the
   process and it writes it in the field: the user presses Enter one time.

The measurement after the corrections, against the sandbox: a wrong password
gives "The server refused the username or the password.", the address stays in
its field, and the login of `toutuitest` after it gives the Home view.

**One run of every test failed while this item was open, and the cause was the
rate limit.** `the_filter_of_the_user_reaches_the_server` said "the answer must
hold a token", and `podman logs abs-test` held "[RateLimiter] Rate limit
exceeded - Endpoint: POST /login". The sweep of the login had used the 40
requests. `podman restart abs-test` gives the limiter back, and the run then
gave 902 of 902.

### T-93: the keys that remove a list and that give it a new name

T-88 makes a collection and a playlist, and it removed neither and renamed
neither. The view of the lists (`c`) holds the line of each list, and its two
free keys do that work now: **`r` gives the list a new name, and `X` removes
it.**

**The measurement of 2026-08-11 found the asymmetry that decides the rules of the
program:**

| Request | Answer |
|---|---|
| `PATCH /api/collections/:id` with a new name | `200`, and the media of the list do not change |
| `PATCH /api/collections/:id` with a name of no letter | **`200`, and the collection then has no name** |
| `PATCH /api/playlists/:id` with a name of no letter | `200`, and the playlist **keeps** its old name |
| Either, with a name that a different list holds | `200` |
| `DELETE` of either | `200` |
| `DELETE` of either, a second time | `404` |

**The server examines the name when it makes a list (T-88), and it does not
examine it here.** A `PATCH` of a collection with no name gives a line
"[Collection]  [1 item]" that names nothing. Therefore the rules of the name
belong to the program, and they are the rules of T-88: a name of no letter and a
name that a different list of that kind holds both give a message and no request.

**A list keeps its own name.** `a_different_list_holds_that_name` takes the
identity of the list of the line, therefore a user who writes the name that the
list holds already gets no refusal. The program makes no request in that
condition either: the name did not change.

**The program asks one time before it removes a list**, as it does for the queue
of the downloads (T-81). The question names the kind, because the two kinds are
not the same thing for a user:

```
Press X again to remove the collection "A List With A New Name" (1 item). Every user of the server loses it.
Press X again to remove your playlist "A Test Playlist" (1 item). Any other key stops this.
```

**Any other key stops the removal.** The measurement pressed `X`, then `j`, then
`X`: the second `X` asked the question of the **second** list, and it removed
nothing.

The line of the list goes away after the removal, therefore the selection moves
to the line before it when the last line goes. `take_the_lists` held that rule
for the media of a list already (T-41).

The measurement in the real program: a collection "A List To Remove" took the
name "A List With A New Name" and the line of the screen changed with no key of
the user, "A Test Collection" gave "A collection of that name exists already", an
empty name gave "A collection and a playlist need a name", and the two presses of
`X` took the collection away. `curl` of the server read the three lists of
`docs/TEST-SERVER.md` after the measurement, and nothing else.

### T-94: the row of the item lost its end in a terminal of 80 columns

**This is the fault of T-90 a second time, in a different row of the screen.**
The sweep of 80 columns read this in the Home view of a library of podcasts:

```
[Letters of Two Brides by Honoré de Balzac (1799 - 1850)] - Author: LibriVox - E
```

The line stops at "- E", and **the two rows below it are empty**. The words
"Episode: 57 - Duration: 12m" reached no user.

The row of the item holds **three** rows (`Constraint::Length(3)`), and the
`Paragraph` of that row wrote on one of them: a paragraph with no `Wrap` writes
one row for each line of its text, and it cuts every letter after the width.
**Twenty-one paragraphs of `src/ui/tui.rs` held that fault**, one for each view
that shows the author, the year, the length, or the number of an episode. The
description below them wrapped already.

The measurement after the correction, in the same terminal:

```
[Letters of Two Brides by Honoré de Balzac (1799 - 1850)] - Author: LibriVox -
Episode: 57 - Duration: 12m
```

`the_screen_survives_a_short_list` holds the rule now: it gives the author a name
of 84 letters, it draws the Library view in 80 columns, and it asks for the year
that stands after that name. **The first form of that test passed with the fault
in the program**, because the name of the author held 53 letters and the whole
line then fit in 80 columns. A test of a limit must hold data that goes past the
limit: the name has 84 letters now, and a build with the wrap removed fails with
"the row of the item lost its end in 80 columns".

### T-95: "1 items", a second time, in the view of the search

The sweep of the reader of 80 columns began with a search for "alice", and the
title of the answer said:

```
Search result [1 items]
```

**T-85 made `ui::keys::items` for this rule**, and it corrected every title that
it found. The title of the search held its own words (`format!("Search result
[{} items{}]", …)`), therefore no correction of T-85 reached it. **The test of
that title held the fault too**: it asked for "Search result [1 items]" and it
passed.

`ListView::line` held a second copy of the rule as well — an `if count == 1` of
its own. It gives the same answer, and a rule in two places is a rule that goes
apart.

Both call `ui::keys::items` now, and **a test finds the next copy**:
`no_title_of_a_view_counts_its_own_items` reads the four files that make a title
of a list with `include_str!`, and it fails for a line that holds `{} items`. A
build with the old title gives:

```
src/logic/search/mod.rs counts its own items: "Search result [{} items{}]",. Use ui::keys::items.
```

**The lesson of this item is about the test, and not about the words.** A test
that holds the answer of the code, and not the rule of the user, keeps a fault
for as long as the code lives. The rule is "one line is 1 item", and the test of
T-85 wrote it for the titles that T-85 read.

### T-96: one login for every test of the sandbox, and a feed that is not a fault

**The rate limit of the login stopped a run of the tests three times in one
session.** Sixteen files of `tests/` held the same `token()`, and each of them
asked the server for a token of its own: one run of every test of the sandbox
made about twenty logins, and the server permits 40 of 600 seconds. Two runs
inside ten minutes therefore gave `429`, and the test said "the answer must hold
a token" — a sentence that names the token and not the cause.

`tests/common/mod.rs` holds that work one time now. It keeps the token in a file
of `CARGO_TARGET_TMPDIR`, and a run that finds a token there **examines it with a
request that is not a login** (`GET /api/libraries`). One run of every test makes
one login, and the run after it makes none. **Three runs one after the other gave
no line of the rate limiter**, and the same three runs before this work gave two
faults.

The message of the login also names the cause now: "A run of the tests that comes
after many logins meets the rate limit of 40 requests of 600 seconds: read
`podman logs abs-test`, and give the container a restart."

**A second item of the same measurement: the feed of the internet.**
`the_program_reads_the_queue_of_the_downloads_and_it_empties_it` failed in four
runs of eight with "the server must read the feed". `POST /api/podcasts/feed`
makes the server read librivox.org, and a slow answer of that web site gives the
client its time limit of 15 seconds. **That measures the network, and not this
program.** The test tries three times now, and it writes a line and gives no
fault when the feed never answers.

### T-97: one request that stopped at its time limit took the server away

**This is T-87 a second time, for a transport fault.** The measurement of T-96
gave the sequence that shows it:

```
the attempt 1 of the feed gave: The server did not answer in time.
the attempt 2 of the feed gave: No server address answered.
the attempt 3 of the feed gave: No server address answered.
```

The first request stopped at its time limit. `is_endpoint_fault` holds
`ApiError::Timeout` as a fault of the address, therefore the client marked the
one address `Down`, and **every request after it found no address at all** until
the probe task ran again — up to **60 seconds** later.

**A request that stops at its time limit is not evidence that the address is
down.** The server does slow work for requests that a user makes every day:
`POST /api/podcasts/feed` reads a web site, and a scan of a library reads every
file. A connection that no machine takes is a different condition, and it still
gives the state `Down` at once.

The pool counts the requests of each address that stopped, one after the other.
**Two of them give the state `Down`**, and one answer of that address forgets
them. `TIMEOUTS_OF_ONE_ADDRESS` holds that number.

**A second fault of the same path, and the user reads it:** a pool of one address
had no second address for the attempt after the fault, and `send` then gave
`ApiError::Unreachable` in place of the fault that came. The user therefore read
"No server address answered" for a server that answered slowly. The fault of the
first address is the answer of the request now, and the user reads "The server
did not answer in time."

`one_request_that_stops_at_its_time_limit_keeps_the_address` of
`tests/api_client.rs` holds the rule: a mock server waits 16 seconds for one
path, and the request after that one must reach the same address. It carries
`#[ignore]`, because 15 seconds is longer than every other test of the program
together. The count of the pool needs no wait, and a test of
`src/api/client/endpoint.rs` holds it.

### T-98: `CARGO_TARGET_DIR` on a `tmpfs` gives nothing. Do not do that work

**Three sessions carried this item, and no session measured it.** The handover
said "The lag of the machine comes from the disk", and the answer of the
measurement of 2026-08-11 is **no**.

The machine of the maintainer:

| What | The value |
|---|---|
| The file system of `target` | ZFS, `zpcachyos/ROOT/cos/home`, two NVMe devices |
| The pool | 2.72 T, **88 percent full**, 62 percent fragmentation |
| `compression` / `recordsize` / `atime` | on / 16 K / off |
| The memory | 60 G, 26 G free |

**The measurements**, of data that no algorithm makes smaller (a file of
`/dev/urandom` in the memory, and then a copy of it):

| The work | ZFS (`/home`) | `tmpfs` (`/dev/shm`) |
|---|---|---|
| One write of 1 GiB | **1.4 GB/s** | 4.6 GB/s |
| 3000 files of 16 KiB | **1140 ms** | 953 ms |

**A build writes about 11 gigabytes**, therefore the write of the whole build
takes about **8 seconds** of the disk at 1.4 GB/s. The 3000 files of the second
row hold the shape of `target`, and the answer of the memory is 16 percent
faster: the loop of the shell holds most of those 953 milliseconds.

**A `tmpfs` therefore gives some seconds of a build of 21 seconds**, and it takes
11 gigabytes of the 26 that stay. The machine feels slow while a build runs
because **16 processes of rustc use the processor**, and `nice -n 19 ionice -c 3`
is the answer that this project has already.

Two notes that stay:

- **The pool is 88 percent full.** ZFS writes more slowly above about 80 percent,
  and `cargo clean --profile dev` gives 15 gigabytes back. That is a reason of
  **space**, and not of speed.
- A first measurement of this item wrote zeros with `dd`, and it gave 7.4 GB/s on
  ZFS: `compression=on` makes a file of zeros almost nothing. **A measurement of
  a disk must write data that no algorithm makes smaller.**

### T-99: a terminal of 18 rows showed one line of the list

The sweep of a small terminal, in 100 columns by 18 rows: the Home view of a
library of 24 media held **one line**.

The three parts of a view of a list are the lines, the row of the item, and the
description. They held 7 rows of that screen, the row of the item took 3 of them
with `Constraint::Length(3)`, and the list and the description then took 2 rows
each. **The description held "N/A", and 10 rows of the screen were empty.**

`the_areas_of_a_list` holds that rule for the nine views that have those three
parts. A terminal of 13 rows or more in that area keeps the split that it had:
the row of the item takes 3 rows, and the list and the description take a half
each of what stays. **Fewer rows give every row to the list**: the row of the
item takes 2 (its text wraps in 80 columns, see T-94), the description takes
none, and the lines take the rest.

The measurement after the correction, in the same terminal: **four lines in place
of one**. A terminal of 24 rows and one of 45 rows draw what they drew before,
and `the_screen_survives_a_short_list` draws every view in 100 by 18 now.

**The rows of the player are the next question, and this session did not answer
it.** The layout of a view holds 6 rows for the player and 1 for the refresh,
above the footer. Those 7 rows are empty while no media plays, and they are 7 of
the 18 rows of a small terminal. **A view that gives them to the list while
nothing plays would move every line when a playback starts**, therefore this is
a choice of the maintainer and not a fault.

### The sweep of two accounts found no fault

The program holds more than one account in its database, and no session had
measured that view. A second user of the sandbox, and then the view of the
accounts (`S`, then the first line):

- The view names the account, and the key `l` gives the question "Press l again
  to log out of "toutuitest". Any other key stops this." **The key works at the
  first press.**
- A measurement of this session said the opposite for some minutes, and **the
  measurement was wrong**: it read the top of the screen after the key, and then
  it read the row of the message 10 seconds later. **A message of the program
  lives six seconds** (the trap 8 of the program). Read the row of the message
  inside that time, and read the row that holds it.

### T-100: the description of a collection and of a playlist, with the key `D`

T-93 gives a list a new name, and the same request takes the description:
`PATCH` with `description` only gives `200`, and the name does not change. The
key `D` of the view of the lists (`c`) writes it.

**The view showed the description of a list already**, and no key of the program
made one. A description of no letter takes the description away, and the server
takes that value: the program needs no rule of its own here, and the sentence for
the user says which of the two happened.

The measurement in the real program: the key `D` of "A Test Collection" wrote "A
collection of the measurement of T-100.", the row of the description of the view
held it at the next frame with no key of the user, and `curl` of the server read
the same words.

**One text of that view held the fault of T-85 in a third place.** The row of the
item said `{} item(s)`, and the rule of the program is "1 item" and "2 items".
The guard test of T-95 reads `{} items` only, therefore it did not find this one.
The row uses `ui::keys::items` now.

### T-101: the changelog of the fork stopped at v0.6.8, and the program was at v0.7.46

The key `S` and then "About and changelog" shows `src/utils/changelog.rs` to the
user. That file held 24 entries, and the newest of them named v0.6.8: **38
releases of this fork reached no user.** T-27 to T-100 all stood outside that
screen.

**The fault hid itself.** The newest entry took `CARGO_PKG_VERSION` with
`format!`, therefore the first line of the screen said "Changelog Toutui
v0.7.46" above the words of v0.6.9. A reader of the screen saw the version of
their build at the top, and no line said that 38 releases were absent. **A text
that names the version of the build is not a text that names the release.**

The measurement of 2026-08-12: the program in tmux, the key `S`, the line "About
and changelog", and the key `J` nine times. The screen held "Changelog Toutui
v0.7.46 (11/08/2026)" and then "The program reads the permissions of your
account", which is the work of v0.6.9.

**The shape now.** `THE_ENTRIES_OF_THE_FORK` is a list of `Entry`, the newest
first, and each entry holds its version, its date, and its lines. A release puts
one entry at the top of that list. The entries of the original project stay in
the local values of `changelog()`, because they belong to a program that no
commit of this repository changes.

**The words come from `git log` and from this file**, and they are the words of a
user: "The key D gives a collection or a playlist a new description." and not
"T-100". Every version from 0.6.9 to 0.7.46 has one entry now, and the whole
list holds 56 entries of the fork.

**One item of a body is one line.** A `Paragraph` of ratatui breaks a line that
is too long, and it never joins two lines: the old entries held the wrap of the
source, therefore a terminal of 200 columns showed a column of 65 letters. The
measurement after the change: the line "- A terminal of 18 rows showed one line
of the list. Every row of a small screen goes to the list now." filled the width
of the panel.

**Four tests hold the rules.** Two of them fail with the old file, and the
measurement of that run stands here: "v0.7.46 does not come after v0.6.8", and
"an entry takes the version of the build".

- `the_changelog_holds_an_entry_for_the_version_of_the_program` reads the newest
  entry and it compares with `CARGO_PKG_VERSION`. **A release that writes no
  entry fails the gate.** This test passed with the old file, because the
  `format!` gave the version of the build: the entry of a release must therefore
  name its own version, and
  `no_entry_of_the_changelog_takes_the_version_of_the_build` reads the source of
  the list and it refuses the word `VERSION` inside it.
- `the_changelog_holds_an_entry_for_every_release_of_the_fork` walks the versions
  of the entries from 0.5.0 up. A step of more than one patch is a fault, and
  `THE_VERSIONS_WITH_NO_RELEASE` names the two exceptions: **v0.6.6** came before
  the version of `Cargo.toml` and the workflow refused that tag (the work went to
  v0.6.7), and no commit ever gave the version **0.7.25**.
- `every_entry_of_the_changelog_names_its_own_version` holds the shape of an
  entry: no two entries name one version, no entry is empty, and no line of a
  body holds a new line.

### T-102: the sequence of the media inside a collection and inside a playlist

A user could not move a book of a collection or an episode of a playlist. The
keys `<` and `>` of the view of the media of a list (`c`, and then `l`) do that
work now.

**The measurement of the request came before the code**, and it decided the
shape. Against Audiobookshelf 2.36.0 on 2026-08-12:

| Request | Answer |
|---|---|
| `PATCH /api/playlists/:id` with `items` of the same media, in a new sequence | `200`, and the new sequence |
| The same, with one item fewer | **`400`, "Invalid playlist items. Length mismatch"** |
| The same, with an `episodeId` that no episode holds | **`400`**, the same words |
| The same, with one item two times | `200`, **and no change of the sequence** |
| `PATCH /api/collections/:id` with `books` of the same books | `200`, and the new sequence |
| The same, with one book fewer | `200`, **and the book that the body does not name goes to the first line** |

**The two lists therefore do not behave in the same way, and the program must
always send every media.** A playlist refuses a body that is not complete, and a
collection takes it and moves the book that the body forgot to the front. The
name and the description of the list do not change in either case.

**The screen holds the new sequence before the answer of the server.** A user
presses the key more than one time to move a media some lines: a screen that
waits for the answer between two keys shows the old sequence, and the second key
then moves the wrong line. `move_the_media_of_the_list` writes the new sequence
in `self.lists`, it moves the selection with the media, and the task asks the
server for the lists after the write (the trap 40). An answer that differs takes
the place of the sequence of the screen.

**The keys.** `<` and `>` are free in the whole program, and they need no
modifier: `handle_key` reads `key.code` only, therefore a key of Ctrl would reach
the handler as the letter itself. The two keys work in the view of the media of a
list, and no other view of the program holds a sequence that a user writes. The
group "The media of a list" of `src/ui/keys.rs` names them, and the view has a
footer of its own now: `FOOTER_OF_A_LIST_OF_MEDIA` named neither `<`, `>`, nor
the key `X` of T-84.

**The measurement in the real program**, with the sandbox and an isolated
`XDG_CONFIG_HOME`:

- The collection "A Test Collection" of 5 books. The key `>` moved "The Test
  Chronicles Volume 1" to the line 2, the message said "\"The Test Chronicles
  Volume 1\" is the line 2 of the collection \"A Test Collection\" now.", and
  `curl` of `GET /api/collections/:id` gave the same sequence.
- Three more presses put that book at the last line, and the fourth said "This
  media is the last line of the list." Four presses of `<` brought it back to the
  line 1, and the fifth said "This media is the first line of the list."
- The playlist "A Podcast Playlist" of 2 episodes. The key `>` moved "Letter 2"
  down, and `curl` gave `['Letter 1', 'Letter 2']`: **an episode needs
  `episodeId` beside `libraryItemId`**, and the body of the program holds it.

**The fault before the correction.** `give_the_list_a_new_sequence` with the
request removed gives a test that fails: "the collection \"A Test Collection\"
must hold the new sequence", with the old sequence beside the new one.

Three tests hold the pure part with no server (`the_sequence_that_moved` at the
two ends and outside the list, the body of each kind of list, and the sentences),
and `the_program_writes_the_sequence_of_a_list` of
`tests/the_lists_against_the_sandbox.rs` moves one line of every list of two
media or more and it gives the sequence back.

### T-103: the Home view and the Library view of a library with no media said nothing

**The sweep of a library of no item found this**, and no session had made that
sweep. The sandbox holds a library "Empty" of 0 items. The screen of 2026-08-12,
before the correction:

```
👋 Connected as toutuitest          📖 Empty (book)          🦜 Toutui v0.7.48
🔗 localhost:13399
──────────────────────────────Library [0 items]──────────────────────────────
                            (nothing, 10 rows)
```

The title said "[0 items]" and no line of the screen said why. **Six views of the
program held the rule of T-91 already** — the series, the authors, the narrators,
the lists, the chapters, and the queue — and the two views that a user sees first
did not.

**The program starts in the Library view when the Home view holds no line**
(`src/app.rs`, near the line 939), therefore a user of such a library reads the
text of the Library view first.

**Three conditions, and their sequence is the rule.** `the_text_of_the_library_view_with_no_line`
of `src/ui/keys.rs` holds it:

1. **The server does not answer.** The program then knows nothing of the library,
   and it must not say that the library holds no media (T-91).
2. **A filter is on.** The library holds media, and the filter hides every one of
   them: "This library holds no media" is false for that condition.
3. **The library itself.** A library of podcasts names the key `A` and a library
   of books names the key `L`, because those keys give the user the next step.

The Home view holds two of those conditions: the server that does not answer, and
a library with no shelf.

**The measurement after the correction**, with the sandbox and an isolated
`XDG_CONFIG_HOME`:

- The library "Empty": the Library view says "This library holds no media. Press L
  to tell the server to examine the library.", and the Home view says "The server
  gave no shelf for this library. Press Tab for the Library, and R to ask the
  server again."
- With `podman stop abs-test`: the Home view says "The server does not answer,
  therefore this screen holds no shelf. A media of the disk plays in this mode.
  Press R when the server answers again." **The Library view of that mode holds
  the four media of the disk**, therefore its own sentence stays away, and that is
  correct.

`a_view_with_no_line_says_why` of `src/ui/keys.rs` holds the sequence of the
conditions, and every text of the two views joined `THE_TEXTS_OF_THE_VIEWS`:
those texts now hold the rule of one space between two words and the rule of a
key with no quotation mark.

**`App::render_the_reason` holds the shape of such a screen in one place.** Five
views wrote that paragraph themselves before, and a sixth view of the future needs
one line now.

### T-104: the 6 rows of the player go to the view while nothing plays

**The decision of the maintainer of 2026-08-12: "let the rows reflow when nothing
plays".** T-99 left this question, because a view that takes those rows moves
every line when a playback starts.

`main.rs` draws the panel of the player for a playback only, and the eleven views
of a list reserved its 6 rows at every moment. A terminal of 18 rows therefore
gave the work of the view 7 rows and it held 6 empty ones.

`the_areas_of_a_view` of `src/ui/tui.rs` holds the layout of every one of those
eleven views now, in one place: the header, the work of the view, and the footer.
The rows of the player stand in that layout while a media plays, and the work of
the view takes them while nothing plays. **A playback that a pause holds is a
playback**: the panel stays, and the user reads the place of that media.

**The row above the footer stays at every moment.** `render_the_message` writes
the message of the program there. A view that takes that row loses its last line
for the six seconds of a message (the trap 39), therefore the reflow gives the
view 6 rows and not 7.

**The first form of this work took a line of the list away, and a test found
it.** `the_areas_of_a_list` compared its own area with 12 rows, and that area
grew by 6: a screen of 20 rows then gave the list **6** lines while nothing
played and **7** while a media played, because the larger area crossed the
threshold of T-99 and a half of it went to the description. **The split is a rule
of the screen, and not of the area**: the function takes the rows that the player
left away before it compares, therefore one screen holds one split and a playback
moves no line into the description.

The measurement in the real program, a terminal of 100 by 18 with nothing
playing:

| The moment | The lines of the Home view |
|---|---|
| Before T-99 | 1 of 24 |
| After T-99, before this item | 4 of 24 |
| Now | **10 of 24**, with the row of the item and the footer |

Three tests hold the rules: `the_view_takes_the_rows_of_the_player_while_nothing_plays`
(the work of the view grows by 6 rows, and the header and the footer stay where
they stand), `one_screen_holds_one_split_of_the_work_of_a_view` (60 heights of a
screen, and the list of a screen with no playback never holds fewer lines than
the list of the same screen with one), and `a_terminal_of_five_rows_gives_the_areas_of_a_view`.

### T-105: the header named the address of the login, and not the address that answers

**The sweep of a pool of two addresses, 2026-08-12.** `config.toml` takes more
than one address of one server, and no session had ever driven the program with
two. The measurement gave the isolated `XDG_CONFIG_HOME` this block:

```toml
[[servers]]
name = "sandbox"
endpoints = [
  { url = "http://127.0.0.1:13456", priority = 0 },
  { url = "http://localhost:13399", priority = 1 },
]
```

`socat` held the first address, and it sent every byte to the sandbox:

```bash
socat -d -d TCP-LISTEN:13456,fork,reuseaddr,bind=127.0.0.1 TCP:127.0.0.1:13399
```

The second program of the measurement is the log of `socat`: it writes one line
"starting data transfer" for each connection. **The start of the program made 9
connections to `127.0.0.1:13456`, and the header said `🔗 localhost:13399`.**

```
👋 Connected as toutuitest        📖 Books (book)        🦜 Toutui v0.7.50
🔗 localhost:13399
```

`server_address_pretty` of `src/app.rs` holds the address of the **login**, and
`App::new` reads it one time from the database. The pool moves between the
addresses at every request, and no line of the screen followed it. A user of two
addresses therefore has no way to know which address carries their data, and a
user who wants the fast local address cannot see that the program uses the slow
public one.

**Every view of the program works with a pool of two addresses.** The sweep drove
the Library view, the series, the authors, the narrators, the lists, the sequence
and the filter, the statistics, the sessions, and the settings: each of them drew
its lines, and 11 connections went to the first address.

**The change of the address needs no key.** The measurement stopped the first
address in the middle of the work, pressed `R`, and every list came back from the
second address. The probe task gave the first address back 70 seconds later (one
connection of `GET /ping`), and the 8 connections of the next `R` went to it
again.

`ui::keys::the_lines_of_the_connection` holds the two lines of the header now,
and `render_header` gives it `self.api.pool().active()`. The measurement with the
correction:

| The moment | The header |
|---|---|
| The two addresses answer | `🔗 127.0.0.1:13456` |
| The first address goes away | `🔗 localhost:13399`, after 20 seconds and no key |
| No address answers | `⚠ toutuitest: the server does not answer` |

The 20 seconds are the rule of T-97: a connection of `socat` that no process
takes stops at its time limit, and **two** such requests give the address the
state `Down`.

### T-106: the statistics of a library of one media said "1 items"

**The sweep of every "1 item" that stays, 2026-08-12.** T-85, T-95, and T-100
each found one of those texts in a different place, and `ui::keys::items` holds
the rule for the title of a view. **The view of the statistics holds numbers, and
not a title**, therefore no title carried that rule there.

The sandbox holds a library `Podcasts` of one podcast. The measurement chose that
library in the settings and it pressed `T`:

```
The library Podcasts
1 items,  57 tracks,  0 authors,  0 genres
```

Three lines of `src/ui/stats_tui.rs` counted their own things: the line of the
library, "{} of listening in {} sessions", and "{} books came, and {} authors".
An account with one session and one book of the year reads three wrong words.

`ui::keys::counted(count, name)` gives "1 track" and "2 tracks" now, and
`items(count)` is `counted(count, "item")`. The same screen after the correction:

```
The library Podcasts
1 item,  57 tracks,  0 authors,  0 genres
```

`the_statistics_name_one_thing_in_the_singular` holds the rule of a library of one
media and of a library of many. A build with the correction removed fails with
"1 items,  1 tracks,  1 authors,  1 genres".

**The book of one chapter found no fault.** The sandbox holds a new book
`One Chapter Book`: one MP3 file with one chapter of `ffmetadata`. The view of
the chapters said `The chapters of "One Chapter Book" [1 item]`, and the view of
the queue of one media said `The queue [1 item]`.

### T-107: the program said "Connected" while it knew that the server does not answer

**The sweep of a server that goes away in the middle of a playback, 2026-08-12.**
The measurement opened no sound device: `TOUTUI_AUDIO_DEVICE=null` gives the null
device of ALSA, and the log of the program says
"[worker] the application uses the sound device alsa:null". **`ALSA_CONFIG_PATH`
does not silence the real program** (the trap 11 of the harness), and this
variable does.

The measurement played `A Long Test Book`, a media that the server holds and the
disk does not, and it then ran `podman stop abs-test`. The playback went on from
the bytes that the program held, and it came to its end at 1800 seconds. Every
part of the offline mode did its work:

```
[offline] the position 1800s of 9a671047-... waits for the server
[play] the server does not answer: No server address answered.. The offline mode starts.
[offline] 1 position(s) wait for the server                 # every six seconds
[offline] the server does not answer: No server address answered.
```

`pending_progress` held the row, therefore no second of the user went away.

**The header said "👋 Connected as toutuitest" for 60 seconds**, while the log of
the same program said "the server does not answer" every six seconds. The screen
became true at the key `R` only, and a user who watches the screen presses no
key: the program that they read said that everything is well.

`App::is_offline` comes from `App::new`, therefore the start of the program and
the key `R` are the two moments that give it a value. **The pool knows more**, and
it knows it earlier: `EndpointPool::active()` gives `None` when no address
answers.

Two corrections hold the screen to what the program knows:

1. `render_header` reads `self.api.pool().active()`. No address gives
   "⚠ <the user>: the server does not answer", and the notice at the right says
   "R: the media of the disk". The offline mode keeps its own words, because the
   lists come from the disk there.
2. **The live task tells the pool.** No request of the program failed while the
   user pressed no key, therefore the pool learned nothing. The task of the live
   messages tries a connection every few seconds, and it marks the address down
   for one fault only: a connection that no machine takes.
   `the_address_is_down` holds that rule, and a connection that opened and a
   request that stopped at its time limit are not evidence (T-97).

The measurement with the correction, with no key of the user:

| The time after `podman stop abs-test` | The header |
|---|---|
| 10 s | `👋 Connected as toutuitest` |
| 30 s | `⚠ toutuitest: the server does not answer` |

**One screen of this sweep stayed without a name.** One run of the measurement
gave a screen with no header, no title, and two texts of two views on the same
row, with the word "ink" alone on the last row. Two runs of the same sequence
after it drew the correct screen, therefore this document holds the observation
and no item: a fault that a session cannot make again is not a fault that a
session can correct.

### T-108: the guard of "1 item" read one form of one text, in four files

**The fault of the guard, and not of a view.**
`no_title_of_a_view_counts_its_own_items` read the form `{} items` in four
files. T-100 found `{} item(s)` of the view of the lists by hand, and this
session found `{} sessions` of the view of the sessions and three other forms
that no rule held: a guard that reads one form of one text finds one fault of
many.

`no_text_of_a_view_counts_its_own_items` reads **every file of `src/ui` and of
`src/logic`** now. It looks for a value of `format!` (`{}`, `{count}`, or
`{:>3}`), then one space, then a word that names a thing of this program:
`{} items`, `{} item(s)`, and `{count} files` all break the rule.

Two kinds of line stay outside, and each has its reason:

- **A line of the log.** The maintainer reads it, and `[offline] 1 position(s)
  wait for the server` is correct there. The rule reads the three lines before a
  line too, because a macro of the log takes more than one line.
- **The tests of a file.** They hold the words of an answer, and a test says
  "1 item" with a number. Every test of this program stands at the end of its
  file, therefore the rule stops at `mod tests {`.

**A unit of measure stays outside too**: `human_size` and `human_time` make
those texts, and no view writes a number of bytes beside a name.

The guard found eight texts, and every one of them reaches a user:

| The file | The text | What a user of one thing read |
|---|---|---|
| `src/ui/sessions_tui.rs` | `{} sessions of {}` | "1 sessions of 1" |
| `src/ui/tui.rs` | `{} - {} book(s) - Duration: {}`, three views | "Depthless Hunger, Book - 1 book(s)" |
| `src/ui/tui.rs` | `A new podcast [{} answers]` | "A new podcast [1 answers]" |
| `src/ui/tui.rs` | ` - {} positions wait` | the line of one position held its own rule already |
| `src/ui/stats_tui.rs` | `The last {} days that you played` | 14 days, therefore no user met it |
| `src/logic/reader/book.rs` | `It has {count} files, and the limit is {MAX_ENTRIES} files.` | "It has 1 files" |
| `src/logic/reader/cache.rs` | `The program removed {} book(s) of the disk` | "removed 1 book(s)" |
| `src/ui/cover.rs` | `CoverArt({} pictures)` | the maintainer reads it, in the debug output |

Each of them takes `ui::keys::counted(count, name)` now. That function gives
"1 track" and "2 tracks", and `items(count)` is `counted(count, "item")`.

`the_guard_finds_every_form_of_a_text_that_counts` holds the reader of the guard
to each of the four forms, and to the texts that are not a fault: `🔗 {} does
not answer` names no thing, `{} bytes of {}` names a unit of measure, and
`format!("{} {}s", count, name)` is the answer itself.

A build with one of those texts back gives the name of the file, the number of
the line, and the whole line.

### T-70 (closed): the program reads the library page by page

**The decision of the maintainer of 2026-08-12.** `get_all_books` read **every**
page of the library before the first frame. A page holds 500 items (T-7),
therefore a library of 2056 items made five requests and a library of 250000
items made 500 of them: the cost of the start grew with the library of the user.

`get_one_page_of_books` reads one page now, and `App::new` takes the page 0 only.
`crate::logic::library_pages` holds the state between the task and the render,
and it is the shape of `logic::sessions_view`:

- `ask_for_the_next_page_of_the_library` runs at the end of every key. It reads
  the position of the item of the line that the user selected, and
  `wants_the_next_page` says if the program must ask. The rule: the server holds
  more items, no task asks already, and the user stands 50 lines or fewer from
  the end of the items that the program holds.
- The **task** makes the seven lists of the screen (`collect_titles_library` and
  the six functions beside it are asynchronous, and the render is not), and it
  puts them in the box.
- `take_the_next_page_of_the_library` runs at each frame. It takes the page one
  time, it puts the lists after the lists that came, and it makes the lines of
  the view again with `group_library`.

**The lines of the pages before a new page do not move.** `group_library` reads
the items in their sequence, and every book of a series gives one line: the lines
of the first page are therefore the same lines after the second page came.

**The title of the view says what the program knows** (T-91).
`ui::keys::the_lines_of_the_library` gives "500 items of 2056" while a page of the
library stays outside, and "8 items" when the program holds every page. A line is
not an item, therefore the title says "8 items of 12" and never "8 of 12 items".

**The search of the server stays the authority** for a title of a page that the
program did not read. The key `/` shows the titles of the program at once, and it
adds the answer of the server when that answer comes.

**The measurement**, `tests/the_library_comes_page_by_page.rs` with a mock server
of a library of 2056 items:

| | The requests of `/items` of the start |
|---|---|
| Before | **5** |
| Now | **1** |

A build with the correction removed fails with "the start must read one page of
the library, and it read 5". The same test moves the line of the user to the item
499 and it holds every list of the library to 1000 items after the page came.

The real program against the sandbox: the library of 12 items gives one request,
8 lines (two series of three books), and the title "Library [8 items]".

### T-62 (closed): a child process reads a PDF book

**The decision of the maintainer of 2026-08-12.** `Document::load` of `lopdf`
reads the whole file, and `MAX_BOOK_BYTES` of 512 megabytes permits a file that
needs a machine of a gigabyte for one moment. **That memory belonged to the
program that the user reads**, and a fault of `lopdf` stopped that program.

`Pdf::open` spawns the program itself now, with the flag
`--the-pdf-of-a-child`, the path of the book, and the path of the pages. The
child reads the book with `Pdf::of_the_file`, it writes the text and the
pictures, and it stops. **This needs no dependency**: `std::process` spawns the
child and `pdf_of_a_child` writes the form of the file. The rule of T-20 stays,
and `mupdf` stays outside.

`the_child_of_the_line_of_command` stands at the **first** line of `main`: the
child opens no terminal, it makes no database, and it plays nothing.

**The form of the file** is a mark of 12 bytes, the title, the author, and one
group for each page. Every number holds four bytes with the little end first, and
every text and every picture holds its length before its bytes. A file of a
different mark and a file that stops in the middle both give no book, therefore a
child that the machine stopped costs one read of the book again.

**The pages stand beside the PDF, in the cache of the ebooks** (T-67). A second
visit of the book therefore spawns no child at all, and `the_cache_of` counts the
bytes of the pages with the bytes of the book: the removal of a book removes its
pages too.

**The measurement of 2026-08-12.** A PDF of 47 megabytes: 60 pages of a scan of
1200 by 1600 pixels, and no algorithm makes those samples smaller. The reader of
the real program in tmux, and `/proc/<the program>/status`:

| The moment | The memory of the program, with the parse inside it | With a child |
|---|---|---|
| Before the key `e` | 37 MB | 39 MB |
| While the parse runs | **101 MB, and it grows to 113** | **39 MB, and it does not move** |
| The book stands on the screen | 113 MB | **53 MB** |

The child itself took **106 megabytes** at its peak and 46 seconds (a build of
the development), and that memory went away with the process. The pages of that
book hold 9.2 megabytes, and the program of the user keeps those.

**A book that the child cannot read gives a message and no dead screen** (T-52).
The first 120 kilobytes of that PDF make such a book: the child writes
"toutui: this PDF gives no page" and it gives the code 2, and the screen of the
user says

```
                    The reader of the ebook
   This PDF gives no page. The file can be damaged. Press h to go back.
                 h/Esc: back  ?: every key  Q: quit
```

`ReaderError::ThePdfGivesNoPage` holds that message. The old value said "This
file is not an EPUB." for a file that starts as a PDF, and that sentence is false.

**The parent stops a child that never comes back** after 300 seconds, and it
reads the message of the child from its `stderr` into the log. A child that gives
no page, a child that stopped, and a child that never came back all give the same
message to the user and the same screen.

### T-66 (closed): Shift+Tab takes the next library of the server

**The decision of the maintainer of 2026-08-12.** The Home view shows the shelves
of one library, therefore a user of two libraries read the shelf of Continue
Listening of one of them only. The settings hold that work behind three keys
(`S`, the line "Library", and `l`).

crossterm gives Shift+Tab as its own code, `KeyCode::BackTab`, therefore
`handle_key` needed no work for a modifier (the trap 58). `Tab` keeps the Home
view and the Library view, and the new key reads as its pair.

`the_next_library` of `crate::logic::library_pages` gives the place of the next
library, and the list goes round: the last library gives the first one. A server
of one library says "This server holds one library.", and the offline mode says
that the program holds one library.

**The key works in the Home view and in the Library view.** The decision named
the Home view, and **the two views share one footer**: a key that a footer names
and that does nothing in that view is a fault of its own (T-79). This is the
nearest answer that keeps the rules of the handover, and "The decisions that this
session made" of `docs/HANDOVER.md` holds it.

The program holds one library at every moment, and no request of the start
changes: the key writes the choice in the database and it asks for a refresh, as
the settings do (T-82).

**The footer holds 116 characters now**, and the old rule of the test said 92.
The area of the footer holds **two rows**, and a terminal of 80 columns therefore
holds 160 cells. A measurement of the real program on 2026-08-12 in a terminal of
80 columns read every word:

```
   j/k: move  l: play or open  Tab: home/library  S-Tab: the next library  /:
                    search  R: refresh  ?: every key  Q: quit
```

The test holds 130 characters now, and that value leaves room for the words that
the wrap moves to the second row.

The real program with the three libraries of the sandbox: `Books` → `Empty` →
`Podcasts` → `Books`. The header names the library at each press, and the message
of the program says the name.

### T-110: the four rows of `docs/T-24-coverage.md` that said Half

**No row of the table says `Half` now.** The four rows are these, and two of them
closed with a measurement and no code.

**1. The account of the user, and its permissions.** `GET /api/me` gives the
type of the account and nine permissions, and no screen of the program showed one
of them: a user whose account may not download read the message of the key `D`
and nothing else. The settings, and then "Accounts and log out", hold the account
now:

```
Accounts — l: log out of the account
➤ toutuitest
The account toutuitest, of the type root.
You may make a copy of a media on the disk (the key D).
You may give a collection or a playlist a new name, a new description, and a new sequence.
You may remove a collection or a playlist.
```

`the_lines_of_the_account` is pure, and it names the permissions that change the
work of the program only: `download`, `update`, and `delete`. `upload`,
`createEreader`, and the three permissions of the libraries and of the tags
belong to work that this program does not do, and a line of them would tell the
user nothing. **An absent permission means "yes"**, therefore a server that names
none gives every line as "You may". The offline mode says that the program knows
nothing of the account (T-91).

**A key that a permission refuses keeps its own message**: the key `D` says "Your
account cannot download a media. Ask the person who holds the server."

**2. `?collapseseries=1`, and it came after the paging of T-70.** The measurement
of 2026-08-12 against the sandbox, with the same library:

| The request | `total` | The results |
|---|---|---|
| `?limit=500&page=0` | 14 | 14 items |
| `?limit=500&page=0&collapseseries=1` | **10** | 10 lines, and three of them hold `collapsedSeries` |

**The grouping of the server changes what one page holds, and it changes
`total`**: `total` counts the lines that the user reads. That is why the
measurement had to come after T-70, and the title of the Library view is exact
now.

**The two screens are the same screen.** The server gives the same 10 lines in
the same sequence as `group_library`, and it collapses a series of one book
("Depthless Hunger, Book") in the same way. The program sends the parameter for a
library of books now, and a library of podcasts holds no series.

**`group_library` stays**, and the maintainer said that it goes away only when
the screen of the server is the same screen. It is: the function gives the line
of a series the place of that series in `App::series`, and the view reads the
books, the description, and the cover there. The server gives no such place. The
work of that function is small now — one line of the answer gives one line of the
view — and the mapping is the work that stays.

The real program: "Library [10 items]", with "The Test Chronicles [3 books]", and
the key `l` gives the three books in the sequence of the series.

**3. `GET /api/podcasts/:id/checknew`: the measurement says no.** The maintainer
asked for that endpoint **where it is cheaper than the work of the program**, and
with the rule that **no episode that is missing may go away**. The measurement of
2026-08-12 shows that the two conditions cannot both hold.

The podcast of the sandbox holds **3 episodes of a feed of 57**:

| The request | The answer | The bytes | The time |
|---|---|---|---|
| `GET /api/podcasts/:id/checknew` | **0 episodes** | 15 | 3.5 s |
| `POST /api/podcasts/feed` | **57 episodes** | 27598 | 5.7 s |

The endpoint compares with the **time of the last examination**, and not with the
episodes that the server holds. It therefore says that nothing is new for a
podcast that is missing **54** episodes. The key `E` reads the feed and it
compares with the episodes of the server, and it finds all 54.

**The endpoint is cheaper only where it is wrong**, therefore the program does
not use it. The row of the table says `Yes` for the function, because the program
does that work and it does it better, and section 6 of
`docs/T-24-coverage.md` holds the endpoint with this measurement. "The decisions
that this session made" of `docs/HANDOVER.md` holds the change of the answer of
the maintainer.

**4. `GET /api/tags` stays outside**, as the maintainer decided.
`GET /api/libraries/:id/filterdata` gives the tags of the library that the user
reads, with the authors, the series, the genres, the narrators, the languages,
and the publishers: one request gives every line of the view of the key `f`.
`GET /api/tags` gives the tags of **every** library, and a tag of a different
library is a line that gives the user no media. Section 6 holds that reason, and
the row says `Yes`: the user filters by a tag today.

### T-111: a child read a PDF inside a test, and that child was the test

**The run of every test with the sandbox found this**, and no test of the fast
suite did. `Pdf::open` of T-62 spawns `std::env::current_exe()`, and **that
binary is the binary of a test inside a test**: that process knows no flag of
`pdf_of_a_child`, therefore it gave a fault and
`the_place_of_the_ebook_against_the_sandbox` said "the book must open:
ThePdfGivesNoPage". Every program that takes this library as a crate meets the
same condition.

`main` writes `the_program_of_the_user_runs()` now, and `Pdf::open` reads
`a_child_can_read()`: the child does the work for the user, and a test reads the
book in its own process. `a_test_reads_the_book_in_its_own_process` holds that
rule, and it is a test that can never pass by accident: `main` never runs inside
a test.

**The same run found a fault of the data of the sandbox**, and not of the
program. `item_with_an_ebook` of that test took the first item of any form of
ebook, and the PDF of 47 megabytes of T-62 stands first in the alphabet: the test
of an EPUBCFI then measured a PDF, which writes no such place. The rule takes an
item of the form `epub` now.

**A run of every test with the sandbox belongs at the end of a session**, and not
at the end of one item: it took 16 seconds, and it found two faults that 913
tests of the fast suite did not.

### The sweep of a library of the size that a user has, 2026-08-12

**The road of 2026-08-12 named this sweep, and it found four faults.** The
sandbox held 14 books, and the paging of T-70 came from a mock server only: a
library of 2056 items had no measurement with the real program at all.

**The data.** A library `Large` of **2056 items** stands in the sandbox now
(`/largebooks` inside the container, 2056 directories of one MP3 file of 4940
bytes). `docs/TEST-SERVER.md` holds the commands. Every item of that library
holds **no author, no narrator, and no year**, because the file holds no such
tag: that is the shape of a book that a user takes from a disk of their own, and
it found the fault T-114.

**What the sweep measured, and what was correct.**

| The work | The measurement |
|---|---|
| The first frame | **609 ms**, and the library holds 2056 items |
| The start | **one** request of `/items`, and the title says "500 items of 2056" |
| Shift+Tab to that library | **610 ms** |
| 500 presses of `j`, over the end of the page | the page came, and **no line of the user moved** |
| The sequence and the filter (`f`) | the page goes back to 0, and the title says "500 items of 2056" |
| The Home view of that library | 20 lines of Recently Added, and no page of the library |
| A page of the server, with `curl` | **2 ms** and 470 kilobytes for 500 items |

**The four faults: T-112 (the key `G`), T-113 (the search), and T-114 (a text of
no letter).** T-113 holds two faults of one shape.

### T-112: the key `G` went to the end of the page, and not to the end of the library

**The user pressed `G` six times for a library of 2056 items.** The key means "go
to the end", and `select_last` took the last line of the lines that the program
holds. The program holds one page of 500 items at the start (T-70), therefore the
measurement of 2026-08-12 gave this:

| The press | The title of the view | The line of the user |
|---|---|---|
| The first | 1000 items of 2056 | Large Book 1557 (the item 500) |
| The second | 1500 items of 2056 | Large Book 1057 |
| The third | 2000 items of 2056 | Large Book 0557 |
| The fourth | 2056 items | Large Book 0057 |
| The fifth | 2056 items | **Large Book 0001** |

**Every press asked for one page**, because the line of the user then stood near
the end of the lines (`wants_the_next_page`), and the page that came moved no
line: the key had to come again for each of the four pages.

**The answer: the key waits for the pages that are left.**
`reads_every_page_of_the_library` of `App` holds that wait. The key `G` sets it
when the program holds fewer items than the library, it asks for the page at
once, and `take_the_next_page_of_the_library` takes the last line again and asks
for the page after it. The wait ends with the last page, and **a move of the user
(`j`, `k`, or `g`) ends it too**: a user who does not want the end must not fight
the key.

**The measurement of the answer, with the real program:** one press of `G`, and
the last item of 2056 came after **2026 ms** (four pages, and the title said
"2056 items"). The key `g` gives the first line back with no request.

`tests/the_key_g_goes_to_the_end_of_the_library.rs` holds the rule against a mock
server of 2056 items, and it fails with the old code: "the program holds 500
items of 2056, and the key G asked for the end".

**The offline mode asks for nothing.** The wait needs a server, therefore the key
takes the last line that the program holds and it stops there.

### T-113: the search showed the media that the program holds, and not the media that the server found

**Two faults of one shape, and the docs of T-70 said the opposite of both.** That
item wrote "the search of the server stays the authority for a title that the
program did not load". The view of the search held the **lists of the library**,
and it read them with the place of the media in those lists:

```rust
Some(answer) => answer.items.iter()
    .filter_map(|id| self.ids_library.iter().position(|one| one == id))
```

**A media that the program did not read therefore gave no line at all.**

**1. A book of a page that the program did not read.** The measurement of
2026-08-12, of the library of 2056 items:

| Who asks | The words | The answer |
|---|---|---|
| `curl` of `/api/libraries/:id/search` | `q=Large Book 0100` | **1 book**, "Large Book 0100" |
| The program, in the same library | `Large Book 0100` | **"The server found nothing for "Large Book 0100". Press / to write other words."** |

The book stands on the page 4 of 5. The program held the page 0, therefore the
identity of that book stood in no list of the library.

**2. Every search of a library of podcasts.** The answer of the server for such a
library holds the group `podcast`, and `SearchRoot` read `book`, `series`,
`authors`, and `narrators` only:

```
q=Balzac  -> {"podcast": 1, "tags": 0, "genres": 0, "episodes": 0}
q=Letters -> {"podcast": 1, "tags": 0, "genres": 0, "episodes": 0}
```

The program said "The server found nothing for "Letters"" for the one podcast of
the sandbox, whose name is "Letters of Two Brides by Honoré de Balzac". **The
answer of the server also stopped the work that the program does itself**: the
titles of the program hold that name, and a view that has an answer does not look
in them.

**The answer: the answer of the server carries the media, and not the identity
alone.** `libraryItem` of every hit holds the title, the author, the year, the
description, and the length already, therefore this needs **no request more**.

- `search_library::media_of` gives every media of the groups `book`, `podcast`,
  and `series`, with no repetition. `BookMatch` and `SeriesMatch` hold a
  `LibraryItem` of `get_all_books` now, and not an identity alone.
- `logic::search::Found` is one line of the view, and
  `the_media_that_the_server_found` makes those lines. It is pure, and it holds
  the rule of a text of no letter (T-114).
- The view builds its seven lists from those lines, therefore **every value of a
  line comes from the answer**.
- `Found::place` holds the place of the media in the lists of the library, when
  the program holds it. **The lists of the episodes of a podcast come from that
  place**, and they come in the sequence of the lines of the view: the old code
  filtered the list of the library, therefore an answer in a different sequence
  gave the episodes of a different podcast. That is a fault that no user
  reported, and the measurement of a podcast library found it.

**A library of podcasts drops a line whose media the program did not read**, and
the log says how many. One page holds 500 podcasts, therefore no user of the
measurement meets that condition, and a view that opens a podcast with no episode
would say a reason that it does not have (T-91).

**The measurement of the answer, with the real program:**

| The library | The words | The screen |
|---|---|---|
| Large (2056 books, one page read) | `Large Book 0100` | "Search result [1 item]", and the book |
| Podcasts | `Balzac` | "Search result [1 item]", and the key `l` gives its 57 episodes |
| Books | `carroll` | "Search result [1 item, with the books of Lewis Carroll]" |
| Books | `chronicles` | "Search result [3 items]", the three books of the series |

`tests/the_search_shows_a_media_of_the_server.rs` draws the view with a real
`App` and a media that the lists of the library do not hold. Two tests of
`search_library` hold the group `podcast` and the values of a media.

**The group `episodes` stays outside.** No measurement of the sandbox gave one
hit of that group (`q=Chapter` of a podcast of 57 episodes gives 0), therefore
the program has no evidence of its shape. **Do not add what you cannot measure.**

### T-114: a text of no letter is not a value

**The line of the Library view said `Author:  - Year: N/A`.** The two values of
that line are absent in the same way, and the screen said one of them with words
and the other with nothing at all: **a user cannot tell an empty value from a
fault of the program.**

The measurement of 2026-08-12, of one item of the library of 2056 books:

```json
{"title": "Large Book 2056", "authorName": "", "narratorName": "",
 "seriesName": "", "publishedYear": null}
```

**The server gives `""`, and the program read `null` only.** Every collector held
the same shape: `if let Some(value) = … { push(value) } else { push("N/A") }`. A
book that holds no tag of an author therefore gave an empty text to the screen,
and a book with no tag at all is the shape of a book that a user takes from a
disk of their own.

`src/utils/values_of_the_server.rs` holds the rule now: **a text that holds no
letter is not a value.** `a_text_or_nothing` gives "N/A", and `a_text_or` gives
the words of a view ("No description available"). Six collectors take their text
through it: the library, the episodes of a podcast, the two shelves of the Home
view, the collections and the playlists, and the series. **One rule therefore
holds for every view.**

A description holds a web page in some libraries, and `to_plain_text` of a page
of no text gives an empty text: such a page is no description, and the same rule
answers it.

The tests of `collect_get_all_books` hold the measurement of the sandbox, and
they fail with the old code: `assert_eq!(collect_auth_names_library(&answer)
.await, vec!["N/A"])` gave `[""]`.

**The identity of an item keeps its own shape.** An identity of no letter is a
fault of the server, and no view shows it: the rule of this item is a rule of the
text that a user reads.

### The sweep of a terminal that changes its size, 2026-08-12

**The road of 2026-08-12 named this sweep, and it found two faults.** Every
measurement before it started the program at one size: `tmux new-session -x 160
-y 45`. A user changes the size of their terminal while the program runs, and
`tmux resize-window -t check -x 80 -y 24` does that work in the harness.

**What the sweep measured, and what was correct.**

| The work | The measurement |
|---|---|
| Every size from 200x50 to **10x3** | the program stands, and it draws its header |
| A resize inside 6 views (`T`, `S`, `a`, `c`, `d`, `?`) | every view draws again, and no log holds a panic |
| A resize inside the reader | **the text reflows and the place stays**: chapter 5 of 14, 16%, the same paragraph |
| 500 lines of a list, then a resize | the line of the user stays |
| The footer of every size | it wraps, and it keeps the keys `?` and `Q` (T-90) |

**The two faults: T-115 holds both**, because both come from a size that the
program read one time.

### T-115: a box that takes a text, and the header of a narrow screen

**1. The box of a text did not answer a resize, and the screen then held
nothing.** `search_active` and `ask_for_a_text` read `term.size()` **before** the
loop of the events, and they made the rectangle of the box one time. The
measurement of 2026-08-12:

```
The terminal is 160 by 45. The user presses `/`, and the box stands at the row 41.
The terminal becomes 80 by 24.
The screen holds 24 rows, and every one of them is empty.
The user writes "alice" and they see nothing at all. Enter gives
"Search result [1 item]", therefore the program read every letter.
```

**A box that draws outside the screen draws nothing**, and ratatui writes no cell
of an area that stands outside its buffer. The user therefore lost the box, the
letters, and the cursor.

`logic::prompt::the_areas_of_the_box` gives the two rectangles of the box, and
the loop of each box asks for them **at each turn**. The pure function
`the_areas_of_a_box_of_this_size` holds the rule, and a test holds the two sizes
of the measurement: a screen of 160 by 45 gives the row 40, and a screen of 80 by
24 gives the row 19.

**2. The three parts of the header wrote on each other below 68 columns.** The
header holds three paragraphs over **one** area: the account at the left, the
library in the middle, and the name of the program at the right. Every paragraph
writes its own letters only (the trap 32), therefore a long part meets its
neighbour and no letter of the two goes away:

| The width | The header |
|---|---|
| 80 | `👋 Connected as toutuitest       📖 Books (book)       🦜 Toutui v0.7.58` |
| 70 | the same, with fewer spaces |
| **60** | **`👋 Connected as toutuitestBooks (book)     🦜 Toutui v0.7.58`** |

The book of the library ate the mark `📖`, and the name of the account ran into
the name of the library.

**Every part takes a short form below `THE_WIDTH_OF_THE_LONG_HEADER` (68
columns)**: `👋 toutuitest` and `🦜 v0.7.58`. **No value goes away**: the account,
the library, the address, and the version all stay, and the words around them go.
The measurement after the answer:

| The width | The header |
|---|---|
| 60 | `👋 toutuitest          📖 Books (book)            🦜 v0.7.58` |
| 45 | `👋 toutuitest  📖 Books (book)     🦜 v0.7.58` |
| 40 | `👋 toutuitest📖 Books (book)  🦜 v0.7.58` |

**41 columns is the honest limit** of the short form: the three parts hold 13, 16,
and 10 cells. At 40 columns they touch, and no letter goes away. A terminal of
fewer than 41 columns holds one word of a title, therefore no work of the header
can help that user.

`a_narrow_header_takes_the_short_form` of `ui::keys` holds the rule, and it counts
the cells of the three parts.

### The sweep of a book of a scan of 502 megabytes, 2026-08-12

**The road of 2026-08-12 named this sweep**, and it named the value to measure:
"the child of T-62 holds the memory, and the time of the parse is the value to
measure". The sweep found **two faults**, and the memory of the child was not one
of them.

**The data.** A PDF of **502745447 bytes** (479 megabytes, and 502 in the words of
a maker of disks) stands in the sandbox as "A Huge Book Of A Scan": 150 pages of a
picture of JPEG of 1200 by 1600 pixels of bytes that no algorithm makes smaller.
`MAX_BOOK_BYTES` is 512 megabytes, therefore this book stands almost at that
limit. `docs/TEST-SERVER.md` holds the commands.

**The measurement of the first run:**

| The work | The measurement |
|---|---|
| The download of the book, from the sandbox | under **800 ms** |
| The parse of the 150 pages, in the child | **123809 ms** (2 minutes 4 seconds) |
| The peak of the child | 974 megabytes |
| **The peak of the program of the user** | **1007 megabytes** |
| The screen while the user waits | "The program gets the book…" |
| The cache of the ebooks | it removed 4 books of 56 megabytes, and it said so |
| The reader, when the book came | `27c55369-b048-4d68-9e70-17653b4d618f — page 1 of 150 — 0%` |

**The child of T-62 does its work**: 974 megabytes of the parse stand outside the
program of the user, and that memory goes away with the process. **The program of
the user held 1007 megabytes of its own**, and that is T-116.

### T-116: the program held the whole book in its memory

`ApiClient::download_to_file` read `response.bytes()`. **The whole answer therefore
stood in the memory of the program that the user reads**, and the buffer of
`reqwest` grows by a copy of itself: a file of 502 megabytes gave a peak of
**1007**.

T-62 moved the parse of a PDF into a child for exactly this reason, and **the
download of the same book stayed in the program of the user**.

`logic::download::fetch` of a media of the disk held the right shape already: it
writes each part of the answer with `response.chunk()`. `download_to_file` holds
that shape now.

**The measurement of the answer, with the real program and the same book of 502
megabytes** (a new `XDG_DATA_HOME` gives a cache that is empty, the trap 24 of the
harness):

```
the peak before the key e:            41960 kB
the child stands after 782 ms:        the book came to the disk
the peak of the program of the user:  45708 kB (44 MB)
the file of the book:                 502745447 bytes
```

**1007 megabytes became 44**, and the download of the book costs 3.7 megabytes of
the program of the user.

**A test of that memory needs a server outside the process of the test.** A
measurement of 2026-08-12 read `VmHWM` around a download of 96 megabytes of a mock
server of `wiremock`: the mock makes its answer inside the process, and the two
forms of the code both gave 192 megabytes. **The memory of the answer of the
server hides the memory of the client.**
`tests/the_download_of_a_book_holds_no_book.rs` therefore holds two rules that a
test can hold: a book of 96 megabytes comes to the disk complete, and
**`download_to_file` holds no `response.bytes()`** — a rule that reads the source,
in the same way as `every_key_of_the_handler_stands_in_the_list`. That second rule
fails with the old code.

### The words while a book of a scan opens: the answer of the maintainer

**The question of T-116 is closed, and the text does not change.** The parse of a
book of 502 megabytes takes 2 minutes, and the screen says "The program gets the
book…" for every second of it. The child process knows the pages that it wrote, and
no slot carries that number to the screen.

The maintainer decided on 2026-08-12: **leave the text as it is.** A book of that
size is rare, and the work would add a slot, a message of the child, and a rule of
the render for one condition that few users meet. This row is a decision now, and
not work that waits.

### T-117: the reader said the identity of the item as the title of the book

The reader of a book that the user opened from the **view of the search** said
`27c55369-b048-4d68-9e70-17653b4d618f — page 1 of 150 — 0%`.

**A PDF holds no title in most files**, therefore the reader takes the title of
the server (T-54). `selected_item_title` gave `None` for the view of the search,
and the comment of that line said "The view of the search holds no list of the
titles. The reader then takes the title of the file, and that is not a fault." The
name of the file on the disk is the identity of the item, therefore the user read
a UUID.

**T-113 made that comment wrong**: the answer of the server carries the title of
every media now, and the view holds `App::titles_search_book`. The reader takes
that title, and the screen says
`A Big Book Of A Scan — page 1 of 60 — 0%`.

`tests/the_search_shows_a_media_of_the_server.rs` holds the rule.

### What this sweep leaves for a next session

**The parse of 150 pages of a scan takes 2 minutes, and the screen says "The
program gets the book…" for every second of it.** The words are wrong after the
first second: the book stands on the disk, and the program reads its pages. The
child knows the number of the pages that it wrote, and no message of the program
carries it.

**The value of the work**: a user of a book of a scan waits minutes with no sign
of progress, and a user who waits with no sign presses the key again. A message
that names the phase and the pages ("the program reads the page 84 of 150") needs
a slot between the child and the render, and `logic::reader::opened_book` holds
the shape of such a slot already.

### The sweep of two accounts of two servers, 2026-08-12

**The road of 2026-08-12 named this sweep**, and it found that the condition
cannot exist: **no key of the program makes a second account.**

**The data.** A second Audiobookshelf stands in a container of its own on the port
**13400** (`abs-test-2`), with the account `secondtest`, a library "Second Books",
and one book of 30 minutes. Each server holds a position of a media of its own:

| The server | The media | The position |
|---|---|---|
| `127.0.0.1:13399` | A Long Test Book | 900 s of 1800 (50%) |
| `127.0.0.1:13400` | A Book Of The Second Server | 600 s of 1800 (33%) |

**Every value of an account is correct, for the account that the program starts
with.** The measurement of 2026-08-12, of a login of each server in an isolated
`XDG_CONFIG_HOME`:

```
👋 Connected as toutuitest    🔗 127.0.0.1:13399   Continue Listening: 50% A Long Test Book
👋 Connected as secondtest    🔗 127.0.0.1:13400   Continue Listening: 33% A Book Of The Second Server
```

The token, the address, the library, and the shelves of each account come from the
row of that account. **The program holds every value that a second account
needs.**

**The four measurements of the fault:**

1. **The view of the login comes only when the database holds no default
   account** (`main.rs`, the loop of `_database.default_usr.is_empty()`). A user
   who holds one account can therefore never reach that view, and **no key of the
   program adds an account**: the key `l` of the accounts view removes one.
2. **The view of the accounts lists the account of the start only.** `App::new`
   makes `all_usernames` from `database.default_usr`, and that is one row. The
   database of the measurement held two accounts, and the view held one line.
3. **Every login writes `is_default_usr = true`** (`auth_process.rs`), and
   `get_default_usr` reads `WHERE is_default_usr = 1 LIMIT 1`. With two such rows
   the **rowid** decides: the measurement started the program with `toutuitest`,
   the account of the first login, and the user had no key to reach `secondtest`.
   **No key chooses the account of the start**, and the SQL of that work stands in
   `db/crud.rs` as a comment.
4. **The way to make the condition at all**: give `is_default_usr` of the first
   row the value 0 with an editor of SQLite, and the view of the login then comes.
   No user does that work.

### T-118: the text of the accounts promised a function that the program does not have

The text of "Accounts and log out" said:

> The accounts that this program holds. … A program that holds more than one
> account starts with the account that is the default one.

**Both sentences are false**, and the second one names a function that no key of
the program reaches. This is the rule of T-91 for a view: **a view must not say a
thing that the program cannot do.**

The text says this now:

> The account of this program. The key l on the account logs out: the program
> removes it, and it asks you for a server, a name, and a password at the next
> start. A second account needs a second configuration: give the variable
> XDG_CONFIG_HOME a directory of its own, and this program then holds its own
> database there.

**The last sentence is the answer that works today**, and the tests of the sandbox
use it. `delete_user` said "User 'x' deleted. Please restart the app to apply the
changes"; it says "The program removed the account x. Start the program again."

`the_text_of_the_accounts_says_what_the_program_does` of `ui::keys` holds the
rule, and it fails with the old text.

### The question for the maintainer, of the function itself

**Must this program hold more than one account?** The rows of the database, the
column `is_default_usr`, and the view of the accounts all come from a session that
wanted it, and no session finished it. The work needs three keys and no new data:

1. A key that opens the view of the login while an account stands.
2. A key of the accounts view that gives an account the value `is_default_usr = 1`
   and takes it from every other row (`db/crud.rs` holds that SQL as a comment).
3. A start of the program that takes the new account with no restart, or a message
   that names the restart.

**The cost of the work stands in the render**: every list of the program comes from
one account, therefore the change of an account is the work of `App::new`. The key
`R` does almost that work already.

### The answer of the maintainer, of 2026-08-12

**The question stays open.** The maintainer said "yes, and a later session does the
work". The text of the view says what the program does today (above), therefore no
user reads a promise that no key keeps. This row waits for that session.

### T-119: the program sends a book to an e-reader

This is the last row of section 4 of `docs/T-24-coverage.md` that said `No` for a
function that a user of a terminal can use, and the issue #24 stayed open for it.

**The measurement came before the screen** (T-88). Every number below comes from
the sandbox of `docs/TEST-SERVER.md`, an Audiobookshelf 2.36.0, on 2026-08-12, with
an SMTP server of the measurement on the machine of the maintainer.

#### 1. The list of the devices does not come from the settings of the e-mail

The road of the handover said that this work needs `GET /api/emails/settings`.
**That endpoint cannot give the list to a user**, and the measurement of two
accounts shows it:

| The request | The account `root` | The account `user` |
|---|---|---|
| `GET /api/emails/settings` | `200`, and every device | **`404`** |
| `POST /api/emails/ereader-devices` | `200` | `404` |
| `POST /api/emails/send-ebook-to-device` | `200` | `200` |
| `GET /api/me` | no device at all | no device at all |
| The `init` of socket.io | `userId` and `username` only | the same |

`ApiRouter.js` gives `/emails/*` an `adminMiddleware`, and every request of a user
that is not an administrator therefore gives `404`. **The user can send a book, and
no endpoint of the e-mail names the device that they may use.**

**`POST /api/authorize` is the answer.** `Auth.js` gives one payload for the login
and for that endpoint, and `getUserLoginResponsePayload` holds
`ereaderDevices: Database.emailSettings.getEReaderDevices(user)`: **the server
filters that list for the account itself.** The measurement:

| The account | `ereaderDevices` of `POST /api/authorize` |
|---|---|
| `root` | the device of `adminOrUp`, and the device of `guestOrUp` |
| `user` | its own device of `specificUsers`, and the device of `guestOrUp` |

One request, a bearer token, and no permission of an administrator. The program
therefore asks `POST /api/authorize` when the user presses the key, and it never
reads the settings of the e-mail.

**The four values of `availabilityOption`** are `adminOrUp`, `userOrUp`,
`guestOrUp`, and `specificUsers`. The program reads none of them: the server gave
the list of that account already, and a rule of the program would be a second
authority that can disagree with the first one.

#### 2. The answers of `POST /api/emails/send-ebook-to-device`

Every body is plain text, and not JSON.

| The condition | The status | The body |
|---|---|---|
| The server sent the book | `200` | `OK` |
| The server has no settings of the e-mail | **`400`** | `Failed to verify SMTP connection configuration` |
| The e-mail did not go | `400` | the words of nodemailer |
| No device holds that name | `404` | `Ereader device not found` |
| The account may not use that device | `403` | `Forbidden` |
| The server does not hold that item | `404` | `Library item not found` |
| The account may not read that item | `403` | `Forbidden` |
| The item holds no ebook | `404` | `Ebook file not found` |

**The three conditions of `404` say three different things**, therefore the status
alone cannot make the sentence for the user. The program reads the body of the
answer, and `the_sentence_of_the_send` of `api::ereaders` gives one sentence for
each of them.

#### 3. The fault of the program: the time limit of 15 seconds

**`REQUEST_TIMEOUT` of `api::client` is 15 seconds, and the server needs more.**
The measurement, with three books of the sandbox:

| The book | The size | The time of `POST …/send-ebook-to-device` |
|---|---|---|
| A Book That No Reader Reads | 0.1 MB | **0.007 s** |
| A Big Book Of A Scan | 45.2 MB | **3.6 s** |
| A Huge Book Of A Scan | **479.5 MB** | **36.2 s** |

That is about 13 megabytes each second, and the whole work stands on the server:
it reads the file, it makes the e-mail, and it gives the bytes to the SMTP server.
**A book of more than about 200 megabytes therefore stops at the time limit of the
program while the server sends it**, and the user reads a fault of a work that
succeeded. The rule of T-97 makes the second such request mark the address down.

`MAX_BOOK_BYTES` of this program is 502 megabytes, therefore the condition is not
a condition of an imagined book: the library of the sandbox holds one.

**The send holds its own time limit of ten minutes.** `download_to_file` holds the
same shape for the same reason, and its comment says it: a request that carries a
file needs no limit of 15 seconds. Ten minutes carries 480 megabytes at 0.8
megabytes each second, and that is sixteen times slower than the measurement above.
The connect timeout of 3 seconds does not change, therefore an address that no
machine takes still fails at once.

#### 4. What the user sees

The key `@` opens the view. It works where the key `m` works, and on a book only:
an episode of a podcast holds no `ebookFile`, and the program says that before it
makes a request.

- The program asks the server at the key, and the view says "The program asks the
  server for the devices…" while it waits. The answer comes in one request.
- **A server with no device gives an empty list**, and the view then says why:
  "The server holds no device for an e-reader. An administrator of the server adds
  one." A view must not give a reason that the program does not have (T-91), and
  the program does have this one: the server answered, and the list is empty.
- A server that does not answer says the sentence of the offline mode.
- The key `l` sends. The message says "The server sends "<the title>" to <the
  device>. A big book takes some minutes." — the send stands on the server, and
  the program cannot measure its progress.

**The program sends the book of `media.ebookFile`, and not the book that the reader
holds open.** An item can hold more than one ebook (T-76, and the trap 30), and the
endpoint of the server takes the item and never a file. The text of the view says
it.

### T-120: a later file that no decoder reads ended a playback that played

**The user reported this on 2026-08-12, and it is the fault of the highest value
of that day.** "Depthless Hunger, Book 2" always started from the minute 0. The
web page of the server and the client of Android both play it from the place of
the user, and this program did not.

**The book of the measurement holds the same 26 hours two times.** The server
gives two tracks:

| The track | The file | The place in the media | The length |
|---|---|---|---|
| 1 | `02_Depthless Hunger 2_[B0GGDKX4GP]_AAC-LC.m4b` | 0 s | 93285.2 s |
| 2 | `02_Depthless Hunger 2_[B0GGDKX4GP]_xHE-AAC.m4b` | 93285.2 s | 93278.8 s |

The media is therefore 186564 seconds (51 hours 49 minutes), and **the place of
the user (2 percent, 3731 seconds) stands inside the first track**, which is
AAC-LC and which this program plays itself. The second track is the form of
xHE-AAC of T-68, and no decoder of this program reads it.

#### The measurement, with the old code

```text
[play]   the item baedee53-… starts at 3731 seconds with 2 tracks
[worker] stream is seekable with len=1481967364 bytes.
[worker] the engine cannot open the track 2 of 2: … xHE-AAC.m4b: The format of
         the data has not been recognized… The tracks before it play.
[worker] the playback starts at 3731 seconds          <- the book plays
[play]   no decoder of the program reads … xHE-AAC.m4b. The program asks the
         server for a stream of the whole media.      <- and this ends it
[worker] the playback starts at 0 seconds
```

**The engine did the right work, and `play_media` threw it away.** The engine
opened the track of the place of the user, it started the playback at 3731
seconds, and it said "The tracks before it play" for the track that it cannot
read: that is the rule of T-48 and of T-55, and the book ends at the track before
that file. `play_media` then read the flag of the fault and it went to a stream
of the server, therefore the user lost a playback that worked. The screen said
"One file needs the server" for a file that stands **26 hours** after their place.

#### The cause: one flag for two conditions

The engine writes `file_with_no_decoder` in two places of `worker.rs`, and the
two conditions need two answers:

1. **`fill_queue` failed.** The track that the playback needs **now** does not
   open. The engine stops the player, and it never writes `playback_id` for this
   playback. The playback is dead, therefore the stream of the server is the
   answer. This is T-53.
2. **The engine skipped a later track.** It plays the tracks before it. The
   playback works, therefore the stream of the server is the wrong answer.

`the_file_that_no_decoder_reads` read the flag of the fault **before** it read
"the engine plays this playback", therefore the condition 2 gave the name of the
file and the caller left a playback that played.

**The state of the engine tells the two conditions apart.** `worker.rs` writes
`value.playback_id` in the loop that follows a playback that plays (the line
580), and a start that failed never reaches that loop: the state then holds the
identity of the playback before it and the status `Stopped`.

**The correction** puts the rule in a pure function,
`the_stream_must_take_the_playback`, and that function reads "the playback plays"
first. `tests/a_later_file_with_no_decoder_keeps_the_playback.rs` holds the six
conditions, and **two of its tests fail with the old order of the two checks.**

#### The measurement after the correction

The local sessions of the program were removed and the server was set to 3731
seconds:

```text
the server holds 3731 s
[play]   the item baedee53-… starts at 3731 seconds with 2 tracks
[worker] the engine cannot open the track 2 of 2: … The tracks before it play.
[worker] the playback starts at 3731 seconds
```

**No stream comes, and no line of the log says 0 seconds.** The row of the player
says "The program cannot read 02_…_xHE-AAC.m4b", which is true and which names
the file.

**T-53 does not change.** The book of the sandbox whose **only** file is xHE-AAC
still goes to the stream of the server, and it resumed at the part 70 (423
seconds) of that stream in the measurement of the same day.

### T-121: the engine asked the server for a download, and an account without that permission played nothing

This came out of the measurement of T-120, with an account of the type `user`.

**`HttpFile::open` asked for `/api/items/:id/file/:ino/download`.** The server
holds every route of a download behind the permission `download`, and it answers
`403` for an account that does not have it. The engine then gave a fault at
once, and the program said that no decoder reads the file:

```text
[worker] the engine cannot start the book: The server did not give the file:
         Your account does not have this permission.
[play]   no decoder of the program reads 01_…_AAC-LC.m4b. The program asks the
         server for a stream of the whole media.
```

**No book of such an account played from its file at all.** Every media went to a
stream of the server: the server transcoded 51 hours of audio for a file that the
program reads itself, and the user read a sentence about a decoder for a fault of
a permission.

**The address of a track is the value of `contentUrl` that the server gives**, and
it holds no `/download`. The measurement of 2026-08-12 against an Audiobookshelf
2.36.0, with an account of the type `user` whose permission `download` is false:

| The address | The answer |
|---|---|
| `GET /api/items/:id/file/:ino` with a `Range` | **`206`** |
| `GET /api/items/:id/file/:ino/download` with a `Range` | **`403`** |

The same two addresses both give `206` for an account of the type `root`,
therefore no session before this one met the fault: every measurement used the
account `toutuitest` of the sandbox, and that account is `root`.

`the_engine_reads_the_file_and_it_does_not_ask_for_a_download` of
`tests/http_file.rs` holds the rule: the mock server answers the address of a
download with `403`, as the real server does.

**`src/logic/download/fetch.rs` keeps the address of a download**, and that is
right: the key `D` makes a copy on the disk, that work **is** a download, and the
permission belongs to it.

### T-122: a program with no configuration file stopped, and it said a line of its own source

The user moved from the program before this fork to this fork, and the first
start gave this:

```text
Error: configuration file "/home/…/.config/toutui/config.toml" not found

Location:
    src/config.rs:70:22
```

**`install.sh` copies `config.example.toml`, and no other way of installation
copies it.** `cargo install`, a package of a distribution, a build of the
repository, and a move from a different program give a user who has no file. The
program held the sentence of the `config` crate and a line of its own source, and
it drew no view.

**The program holds the text of the example**, therefore it writes the file
itself:

- `THE_EXAMPLE_OF_THE_CONFIGURATION` is `config.example.toml`, and `include_str!`
  puts it in the binary. The user receives every comment of that file, therefore
  they find the line of each setting.
- `make_the_configuration_if_it_is_absent` makes the directory and the file. A
  disk that permits no write gives `false`, and `load_config_from` then gives the
  values of the program: the program starts on a read-only system too.
- A file that exists stays as it stands. No start writes over the file of the
  user.

**One key of a color that is absent lost every color of the file.** The block
`colors` is one value for `serde`, therefore a file of an older version, which
holds no `player_background_color`, stopped the program in the same way. The
block takes `#[serde(default)]` now, and `Colors::default()` holds the values of
`config.example.toml`: the key that is absent takes the value of the program, and
every color of the file stays.

Seven tests of `src/config.rs` hold the rules, and two of them compare the
example of the repository with the values of the program: a color of the program
that the example does not name would take its value in silence, and the user
would find no line to change.

**The measurement of 2026-08-12.** A directory of configuration with no file, and
the real binary in tmux: the program made `config.toml`, the file agreed with
`config.example.toml` byte for byte, and the login screen came.

### T-123: a token that the server refused stopped the program, and no login screen came

The same user met this after T-122, at the same start:

```text
Error: The token is not valid. Log in again.

Location:
    src/app.rs:494:44
```

**The sentence tells the user to log in again, and the program gave them no way
to do it.** `App::new` asks the server for the libraries first, the server
answers `401`, and the report left `main`. A user who moves from the program
before this fork meets this at the first start, because the database of that
program holds a token that this server no longer holds.

**The program opens the login screen now.**
`the_program_needs_a_new_token` of `src/logic/auth/auth_input.rs` does three
things:

1. the row of the account goes away. A row that stays would give the same fault
   at the next start. **The rows of the downloads, of the queue, and of the
   positions that wait hold the name of the account only**, and no key of the
   database removes them with this row: a login with the same name finds all of
   them again;
2. the login screen takes the sentence "The token is not valid. Log in again."
   through `update_login_err`, in the same way as a wrong password;
3. the program starts again, and the login screen of a first start comes.

**Why the program starts again, and does not make the login screen inside the
same process.** A measurement of 2026-08-12 in tmux made the login screen after
`restore_terminal` and a second `ratatui::init` of the same process: the screen
drew the box of the address one time, the box then went away, and no key gave a
character. The program that starts again meets the login screen of a first start,
and that screen works. The new process also takes away the task of the live
messages, the task of the probe, and the task of the positions that wait: each of
them holds the token that the server refused, and two live tasks write one state
of the program.

`start_the_program_again` of `src/utils/exit_app.rs` uses `exec` of the system,
therefore no process stays. The address of the server goes to the new program in
the variable `TOUTUI_THE_ADDRESS_OF_THE_LOGIN`, because a value of the process
before it goes away: the user reads their address in the first field, and one
press of Enter takes it. A system that has no `exec` gives an answer, and `main`
then makes the login screen inside the process: the loop `'the_session` of
`src/main.rs` holds that way, and it aborts every task of the account first.

**Both places that make an `App` read the category now**, therefore a server that
takes the token away while the program runs sends the user to the login screen
too: the startup, and the key `R`.
`api::client::error::the_token_is_not_valid` reads the chain of the report, thus a
sentence of a caller does not hide the category. **The words of a message are not
the category**: a report of a string that holds the same words gives `false`, and
three tests hold that rule.

**The measurement of 2026-08-12, with the sandbox.** A real login wrote the row
of `toutuitest`, and the token of that row then became a token that the server
refuses. The next start: the row went away, the login screen came with the
sentence and with `http://127.0.0.1:13399` in its field, and Enter, the username,
and the password gave the Home view of the account again.

### T-124: the program holds more than one account

**The sweep of two accounts of two servers of 2026-08-12 found that the condition
cannot exist**, and T-118 gave the view a text that says so. The maintainer
answered "yes, and a later session does the work". This is that work.

The three faults of the program, and the key of each:

| The fault | The key now |
|---|---|
| The view of the login came only when the database held no account | **`a`** adds an account |
| The view of the accounts listed the account of the start alone | every account holds a line, and `▶` marks the account that starts |
| No key gave the start to a different account | **`c`** gives it, and the program asks one time |

**The database needed no new column.** The table `users` holds a row for each
account already, with `is_default_usr`. Two functions of `db/crud.rs` do the
work: `select_every_usr` gives every row to the view, and
`make_this_account_the_default` gives one row the start **in a transaction**
(`UPDATE users SET is_default_usr = 0`, and then the row of the name). The SQL of
the second function stood in that file as a comment for the whole life of the
fork.

**Every key that changes the account starts the program again**, and that is the
shape of T-123: a login screen inside the same process draws a box that goes
away, and every task of the old account holds the old token. `exec` gives the new
program the terminal of the old one, and no value of the process crosses it.
`start_the_program_again_with` takes the variables of the environment that the
caller needs:

| The key | The variables | What the user sees |
|---|---|---|
| `a` | `TOUTUI_ADD_AN_ACCOUNT=1`, and the address of the login | the login screen, with the address of the account of now in its field |
| `c` | none: the database holds the account of the start | the Home view of the account of the line |
| `l` of the account that starts, with a second account | none | the Home view of the account that stays |
| `l` of the one account | the address of the login | the login screen |

**A second login writes a second row, and the newest login starts the program.**
`auth_process` wrote `is_default_usr = true` and it changed no other row: two
such rows let the **rowid** decide (T-118). It calls
`make_this_account_the_default` after the insert now.

**The rule of the log out is a pure function.** `the_account_after_a_log_out` of
`src/logic/the_accounts.rs` gives one of three answers: the view only, this
account starts, or the login screen. `tests/two_accounts_of_one_program.rs` holds
the rules of the database, and it stands alone in its binary (the trap 8).

**The measurement of 2026-08-12, with the two containers and tmux.** An isolated
`XDG_CONFIG_HOME`, a login of `toutuitest` of `127.0.0.1:13399`, and then:

```
--- the key a, and the login of secondtest of 127.0.0.1:13400
👋 Connected as secondtest   🔗 127.0.0.1:13400   Home [2 items]

--- the view of the accounts
Accounts — a: add, c: this account starts, l: log out
➤   toutuitest — http://127.0.0.1:13399
  ▶ secondtest — http://127.0.0.1:13400

--- the key c on the account that starts already
The program starts with the account secondtest already.

--- the key c on the line of toutuitest
Press c again to start with the account "toutuitest". The program starts again, and a playback stops.
👋 Connected as toutuitest   🔗 127.0.0.1:13399   Home [13 items]

--- the key l on the account that starts, with a second account
👋 Connected as secondtest   🔗 127.0.0.1:13400

--- the key l on the one account that stays
┌Server address────────────┐
│http://127.0.0.1:13400    │
```

**Each of the two servers gave its own values**: 13 items of the library of
`toutuitest`, and 2 items and 33% of A Book Of The Second Server of `secondtest`.
That is the measurement of T-118, and the user reaches it with two keys now.

**The words of the view say what the program does** (T-118 and T-91). The text of
"Accounts and log out" names the three keys, it names the mark `▶`, and it says
that a playback stops when the program starts again. The footer holds two rows of
80 columns (T-90), therefore it names the keys and the text of the view holds the
reason of each.

### The sweep of a media that plays while the program does other work, 2026-08-12

**The road of 2026-08-12 named this sweep, and it found no fault.** That is the
first sweep of a new condition of seven sessions that found none.

The measurement, with `TOUTUI_AUDIO_DEVICE=null` and a book of 30 minutes:

| The work, while the media plays | The measurement |
|---|---|
| The key `/` and a search of the server | the view came, and the player row went on: 2:30 → 3:11 |
| A resize to 80x24, and back to 160x45 | every view drew again, and the player row kept its place |
| The reader of a book (`e`), 5 seconds of it | **2:28 → 9:34**, and `user_updated` of the live messages came at each sync |
| Four sizes, each with the reader and a search | the program stands, and the log holds no line of a fault |

**The reader holds every key of the user** (T-10 and T-52), therefore the space
of a playback is the space of a page there. The footer of the reader says it, and
the key `h` gives the keys of the player back.

### The sweep of a library of podcasts of more than 500 items, 2026-08-12

**The road named this sweep for the paging of T-70, and it found three faults.**
One of them stops the program.

The data: a library `ManyPods` of **520 podcasts** of one episode each, in the
sandbox. `docs/TEST-SERVER.md` holds the commands.

| Item | What | The measurement |
|---|---|---|
| T-125 | The search said "The server found nothing" for a podcast that the server found | the log said "the program did not read 1 podcast(s) of the answer" |
| T-126 | **The key `l` on a podcast of a later page stopped the program** | tmux said "can't find pane" after that key |
| T-126 | The podcasts of the second page held no episode, and the view said "This podcast has no episode" | the server said `numEpisodes: 1` |
| T-126 | The start read the episodes of every podcast, one request after the other | **11.9 s** for the first frame with a server of 20 ms, and 0.409 s now |

### T-125: the search said that the server found nothing, and the server found a podcast

**T-113 gave the view of the search the media of the server**, and it left one
condition open for a library of podcasts: the lists of the episodes of a podcast
come from the place of that media in the lists of the library, therefore a
podcast of a page that the program did not read gave **no line at all**. The
decision of that day said "one page holds 500 podcasts, therefore no user of the
measurement meets that condition". **A library of 520 podcasts meets it.**

The screen of the measurement, with the words "Many Podcast 001":

```
The server found nothing for "Many Podcast 001". Press / to write other words.
```

and the log of the same moment:

```
[search] the program did not read 1 podcast(s) of the answer
```

**The program reads the pages of the library now.** The user asked for that
media, therefore the cost of the requests is theirs: that is the decision of the
key `G` of T-112. The line came after **705 ms** in the measurement, and the key
`l` then opened its episodes.

The title of the view says what the program does while those pages come, and it
never says that the server found nothing for a media that the server found
(T-91). `the_title_of_the_search` holds the rule, and two tests of that function
fail with the old words.

**The line of the log went to the render**, therefore it came at every frame:
four lines of one search in one second. The program says it one time now, in
`the_search_reads_the_pages_that_are_left`.

### T-126: the key `l` of a podcast of a later page stopped the program

**`App::new` read the episodes of every podcast of the first page**, one request
after the other, and it wrote nine lists of 500 rows. Three faults came out of
that shape, and the sweep of 520 podcasts met all three.

**1. The program stopped.** `self.all_ids_pod_ep[index]` with the line 519 of a
list of 500 rows: an index of a vector that does not exist stops the program, and
`get` does not (T-41). The measurement pressed `G` for the end of the library and
then `l`:

```
=== after the key l on that line:
can't find pane: pods
=== the program stands: NO
```

**The panic never reached the log of the program**, therefore a user who meets
this reads a shell and no line of a file.

**2. The podcasts of a later page held no episode.**
`take_the_next_page_of_the_library` extended seven lists of the library, and it
extended none of the nine lists of the episodes. The view then said "This podcast
has no episode" for a podcast whose `numEpisodes` is 1: a reason that the program
does not have (T-91).

**3. The start made one request for each podcast.** The measurement, with a proxy
of Python that gives every request of the sandbox a delay of 20 milliseconds:

| The library | The first frame, before | The first frame, now |
|---|---|---|
| `ManyPods`, 520 podcasts | **11.9 s** | **0.409 s** |
| `Podcasts`, 1 podcast | 0.409 s | 0.409 s |

500 requests, one after the other. A user of a server of the internet meets 30 or
50 milliseconds of each request, therefore that start took half a minute.

**The program reads the episodes of one podcast now, and it reads them when the
user opens that podcast.** This is the shape of `logic::library_pages` (T-70): a
task asks, `logic::the_episodes` holds the answer, and the render takes it at the
next frame. The lists of the library hold one empty row for each item, and a new
page adds its own empty rows. `the_episodes_that_came` tells a podcast of no
episode apart from a podcast that the program did not read, therefore the view
says one of three sentences: the podcast holds no episode, the program gets the
episodes, or the server does not answer.

**The view of the search opens a podcast too**, and it asks for the same
episodes: the line of that view names the podcast, and the answer goes to the
lists of that view.

**The measurement after the work, with the 520 podcasts:**

| The work | The measurement |
|---|---|
| The first frame | **409 ms**, and it was 1624 ms |
| The key `l` on a podcast of the first page | `Episodes [1 item]` |
| The key `G`, and then `l` on the line 520 | `Episodes [1 item]`, and the program stands |
| The search of `Many Podcast 001`, and then `l` | `Episodes [1 item]` |
| The podcast of 57 episodes of the sandbox | `Episodes [57 items]`, the info of the line, and the mark `[Downloaded]` |
| A second visit of one podcast | no request, and the list comes at once |

`tests/a_podcast_of_a_later_page.rs` holds the fault: it makes a library of 520
podcasts whose lists of the episodes hold 500 rows, and the key `l` of the line
519 stopped the program before this work.

### The sweep of a book of an EPUB of 100 megabytes, and of an EPUB that is not valid, 2026-08-12

**The road named this sweep, and it found no fault.** Four books stand in the
sandbox now: a valid EPUB of **100.5 megabytes** (100 chapters of text and 18
pictures of 4 megabytes), a file of random bytes with the name `.epub`, a zip
with no `container.xml`, and a zip whose container names a file that is absent.

| The book | The measurement |
|---|---|
| The EPUB of 100.5 MB, with no copy on the disk | the reader came after **2 seconds**, "chapter 1 of 100" |
| The same book, with the copy on the disk | the reader came at once |
| The memory of the program with that book open | **55 megabytes** (T-116 holds that shape) |
| The three files that are not an EPUB | "This file is not an EPUB.", and the program stands |

**One measurement of that sweep did not repeat.** The first attempt of the book
of 100 megabytes said "The program did not get the book: No server address
answered", and the server sends that book in **0.13 seconds** with `curl`. Four
attempts after it gave the book. The measurement of that moment came after a scan
of the library of the server, therefore the socket of the live messages can have
marked the address down (T-107). **A later session must look again**, with the
log of the program at that moment.

### The sweep of a server that answers slowly, 2026-08-12

**The session before this one began this sweep and the fault of the user stopped
it.** This session finished it, and it found the sequence of the start.

A proxy of Python holds a port and it gives every request of the sandbox a delay
of 500 milliseconds. The steps of the start come from the box of the start
itself, and a poll of the screen every 50 milliseconds reads them:

| The step | Before T-127 | After T-127 |
|---|---|---|
| the libraries of the server | 165 ms | 165 ms |
| the shelves of the Home view | 649 ms | 649 ms |
| **the position of each book of that list (29 media)** | **1134 → 3228 ms** | — |
| the series, the lists, and every item | 3228 ms | 1134 ms |
| the sound device | 3713 ms | 1618 ms |
| **THE FIRST FRAME** | **3767 ms** | **1725 ms** |

**The positions held 2.1 seconds of a start of 3.8**, and one endpoint holds all
of them.

### T-127: the start asked one request for the position of each media

`GET /api/me/progress/:id` gives the position of one media, and the Home view of
the measurement holds **29** media: the start therefore made 29 requests, eight
at a time (T-40). **`GET /api/me` holds `mediaProgress` for every media of the
account**, and the program asks that endpoint for the permissions of the account
already (T-110): one answer holds every position, and the start of a library of
every size costs the same.

The rows of the two endpoints are the same rows. A measurement of 2026-08-12,
of the book "A Long Test Book":

```
GET /api/me/progress/9a671047-…   {"id":"ed7bcef3-…","progress":1,"currentTime":1800,"isFinished":true, …}
the row of GET /api/me            {"id":"ed7bcef3-…","progress":1,"currentTime":1800,"isFinished":true, …}
```

**The request of the account goes at the first moment of the start now**, beside
the examination of the address, and no request of the start waits for it. The
test `the_four_requests_of_the_start_go_together` of T-86 holds that rule: the
account has its own mock, and the four requests still arrive together.

**A media that the answer does not name played never.** `GET /api/me/progress/:id`
answers 404 for it, and the line of the view says "N/A" either way: the program
therefore asks for **no** media of a library of books now. It asks as it did
before when the answer of the account did not come at all, because a program that
knows nothing must not say that every book is at the start.

**A row of a podcast names the episode beside the media**, and the position of
one episode is not the position of the line of the Home view. Such a media keeps
its own request, and `the_position_of_a_media` of `src/logic/the_positions.rs`
holds that rule with three tests.

**Two faults of the program came out of this work.**

1. **`ebookProgress` is a fraction, and the program read a whole number.** The
   field held `i64`. A book that the user read gives
   `"ebookProgress": 0.8277488992014371`, therefore the answer of
   `GET /api/me/progress/:id` **did not read at all** and the line of that book
   lost its position: the user read "N/A" for a book of 92 percent. The field is
   `f64` now, and a test holds the answer of the real server.
2. **A row that the program cannot read took every other row away.** One answer
   holds 20 rows, and `serde` gives an error for the whole list when one row does
   not read. Each row reads by itself now (T-41).

**The measurement of the sandbox after the work**: the first frame of the program
takes **207 milliseconds**, and every percentage of the Home view is the
percentage of the server (92%, 42%, and the mark of a book that is finished).

### T-128: the program said "No server address answered" for a server that answers

**This is the measurement that did not repeat**, and the road of the session of
T-124 to T-127 named it as the first work. The first attempt of the book of an
EPUB of 100 megabytes said "The program did not get the book: **No server address
answered**", four attempts after it gave the book, and `curl` sent that book in
0.13 seconds. The session made a guess: a scan of the library, and the socket of
the live messages (T-107).

**The guess was right, and the condition repeats every time now.** The
measurement of 2026-08-12, with the sandbox and tmux: the server goes away for 25
seconds, and it then answers again.

| The moment | What happened |
|---|---|
| 21:24:46.36 | the live connection ended: "the request failed" |
| **21:24:56.36** | the live task tried again, no machine took the connection, and **the one address of the pool took the state `Down`** |
| 21:25:12.05 | `curl` read the server again: **200 in 1.5 milliseconds** |
| 21:25:13.55 → 21:25:43.69 | **16 presses of the key `e`, and each of them said "The program did not get the book: No server address answered"** |
| 21:25:43.64 | the probe task gave the address the state `Up` again |
| 21:25:45.70 | the next press of `e` gave the book |

**31.6 seconds of a false reason, and the limit is 60 seconds**: the probe task
sleeps `PROBE_INTERVAL` first, therefore the wait of a user is the rest of that
minute. The address of the fault needs no scan of a library: **one connection
that no machine takes** is enough (T-107), and two requests that stop at their
time limit, one after the other, give the same state (T-97).

**A request must try an address before the program says that no address
answered.** The state `Down` is the answer of an attempt that came before, and a
key of the user is a new question. `EndpointPool::an_address` gives the address
that has the most importance and the state `Up`, and it gives the address that
has the most importance when no address holds that state. Three places take it:
`send`, `download_to_file`, and `post_and_read_the_answer`. `ApiError::Unreachable`
now belongs to a pool that holds no address at all, and to an attempt that
failed.

**An address that answered holds the state `Up`.** `the_address_answered` forgot
the requests of the time limit (T-97), and it left the state alone: the request
of the user is the newest measurement of that address, therefore that function
writes the state too. The header of the program reads `pool.active()` (T-105),
and it says "Connected" again at the same moment.

The same measurement with the correction:

| The moment | What happened |
|---|---|
| 21:30:31.52 | the address took the state `Down`, and **the log says it now** |
| 21:30:47.54 | `curl` read the server again |
| 21:30:47.66 | **the first press of `e` gave the book**, and the log says "The address http://localhost:13399 answers again" |

**The log holds the moment that the program stops to use an address.**
`mark_down` wrote no line of the log at all, and the measurement above had to read
the fault of the live task and make a guess. The function takes the reason now,
and it writes one line for the first fault of an address:

```
[api] The program does not use the address http://localhost:13399 now: the
program cannot connect to the server. It examines that address every 60 seconds,
and a request of the user tries it.
```

**A server that is truly away says the truth.** With `podman stop abs-test` the
header says "the server does not answer", and the key `e` says "No server address
answered." after the program tried the address: the reason of the user is the
answer of an attempt, and not the memory of one.

Five tests hold the rules, and every one of them fails with the correction
removed: three of `tests/api_client.rs` (a request, a request of a pool of two
addresses that are both down, and a download) and two of
`src/api/client/endpoint.rs`.

### T-129: the four requests of the start waited for the shelves of the Home view

**T-127 took 29 requests of the start away, and it left the rounds.** The road of
that session asked which requests of the start are in sequence, and it said that
no measurement gives the answer. **A proxy of 70 lines of Python answers it with
no line of code of the program**: the proxy holds a port, it gives every request a
delay of 500 milliseconds, and it writes the path and the time of each request.
The pool takes that address with a block `[[servers]]` of `config.toml` (the trap
68).

The three rounds of the start, of the measurement of 2026-08-12:

| The time | The requests |
|---|---|
| 0.000 | `GET /api/libraries` |
| 0.511 | `/personalized` (the shelves of the Home view), and `GET /api/me` |
| 1.059 | the series, the collections, the playlists, and the items |
| **2.03 s** | **the first frame** |

**The four requests need the answer of the shelves for nothing.** They need the
identity of the library, and that identity comes with the first round. The four
went together already (T-40), and they waited for a request that says nothing to
them: `App::new` asked for the shelves with `await`, and it made the four
afterwards.

**The four stand in a task now, and that task starts before the request of the
shelves.** The start holds two rounds:

| The time | The requests |
|---|---|
| 0.000 | `GET /api/libraries` |
| 0.505 | the shelves, `GET /api/me`, the series, the collections, the playlists, and the items |
| **1.56 s** | **the first frame** |

**One round of the start is gone, and the first frame of a server of 500
milliseconds takes 1.56 seconds of the 2.03 that it took.** The sandbox gives the
first frame in 566 milliseconds. The first round stays: every request of the
second round needs the identity of the library, and `GET /api/libraries` gives it.

`the_four_requests_of_the_start_go_together` of T-86 holds the new rule.
The mock of that test notes the **path** of each request now, and the measurement
of the shelves against the items fails with the old code: 706 milliseconds apart,
of a server of 700 milliseconds for each answer.

### T-130: the program wrote a position that it cannot read

**The sweep of a media that plays with a slow server found this line in the log
of the program:**

```
[app] a position of the account does not read: invalid type: string "714", expected f64
```

**The program writes that value itself.** `POST /api/session/:id/sync` goes every
ten seconds while a media plays, and `sync_open_session` sent
`{"currentTime": "714", "timeListened": "10"}`: **two numbers as a text.** The
server keeps the form that a client gives it, and the row of `GET /api/me` then
holds a text. The measurement of 2026-08-12 against Audiobookshelf 2.36.0:

| The request | The row of `GET /api/me` after it |
|---|---|
| `{"currentTime":"714","timeListened":"10"}` | `{"currentTime": "714", "progress": 0.3966…}` |
| `{"currentTime":714,"timeListened":10}` | `{"currentTime": 714, "progress": 0.3966…}` |

**The row of a text does not read, and the media loses its position on the
screen** (T-41 keeps every other row). The line of the Home view of a book at the
minute 11 of 30, of the same server and the same account:

| The program | The line | The log |
|---|---|---|
| with the correction | `➤ 40% A Long Test Book` | the answer holds the position of **21** media of 29 |
| the correction removed | `➤     A Long Test Book` | **19** media, and one warning for each row of a text |

**The rule of T-127 hides it.** A media that the answer of the account does not
name played never, therefore the program asks no request for it: the position of
that media is gone, and no view says a reason. The position comes back at the
**close** of the session, because `update_media_progress_book` sends a number.
The user therefore reads a line with no percentage while the media plays, and a
line with a percentage after the playback ends — and a session that never closed
(a program that stopped, or the offline mode) keeps the text for ever.

**Two corrections, and each of them holds a test.**

1. `the_body_of_a_sync` is a pure function, and the two values are numbers. Two
   tests of `src/api/sessions/sync_open_session.rs` hold that rule, and two tests
   of `tests/playback_ownership.rs` asked for the text `"100"` before this work:
   the measurement of the fault stood in the tests of the program.
2. **A number of the answer that comes as a text reads.** `a_number` of
   `src/api/me/get_media_progress.rs` takes a number or a text for `duration`,
   `progress`, `currentTime`, and `ebookProgress`. **The rows of a text stand in
   the database of every server that this program wrote to**, therefore the
   correction of the request alone gives those users nothing. A text of no number
   gives 0, and the row still reads.

### T-131: the key `R` took the playback of the user away from every key

**The sweep of a library of more than 500 items with a media that plays found
this**, and the key that found it is the key of T-66: the row of the player went
away when the program took the next library of the server, and the media played
on.

**Every key that refreshes the screen makes a new application**: the key `R`, the
key that takes the next library (T-66), and the keys of the sequence of the
library. **`App::new` starts a new engine of the sound**, therefore the old engine
kept the playback and the new application knew nothing of it:

| What the user holds | Before | After |
|---|---|---|
| the row of the player | **it goes away**, and the media plays | it stays |
| the key `Space`, `Y`, and every key of the player | they go to the engine of no playback | they go to the playback |
| the lines "the application uses the sound device" of the log | **5** in 15 seconds | 1 |

The measurement of 2026-08-12, with the sandbox and tmux: the key `R` at the
minute 2 of a book of 30 minutes. Before the correction the row of the player
went away, the key `Space` stopped nothing, and the log said "the playback
stopped at 1800 seconds, finished=true" — **the book played to its end while the
user held no key of it.** With the correction the row stays, and the same
sequence gives "the playback stopped at 310 seconds, finished=false".

**The engine of a program that plays already stays.**
`App::new_with_the_engine` takes the handle of the engine and the fault of the
sound device, and the loop of `src/main.rs` gives them at each refresh.
`App::new` is that function with no engine, therefore the first start of the
program does not change.

Two tests of `tests/the_refresh_keeps_the_engine.rs` hold the rule, and both fail
with the correction removed. The first needs **no sound device and no server**:
`PlayerHandle::without_engine` gives a handle whose commands go to a channel of
the test, and the test reads the command of a key. The second reads
`src/main.rs`, because no unit test reaches the loop of the program.

**What a refresh still loses**: the timer for sleep of T-24 and the queue of the
media stand in the application, and a new application holds neither. No
measurement of a user names them, therefore this item does not move them.

### T-132: the test of the live messages passed for the data of the sandbox, and not for the program

**The run of every test at the end of the session found it**, as T-111 was found:
`a_change_of_a_different_client_comes_to_the_screen` failed after this session
wrote the position of one book with `curl`. **The fault was in the test**, and it
held three weaknesses of one shape — the test measured the data of the sandbox and
not the work of the program.

1. **It took the first item of the library** (`?limit=1`). The sandbox holds four
   books of **one second** before that item now (the books of an EPUB of T-127),
   and the test sends the second 756: the server marks such a media as finished,
   therefore the box said "Finished" and 100 percent while the test waited for
   "42".
2. **It waited for a value that the row of the sandbox held already.** The message
   `init` of the connection carries the position of every media of the account,
   therefore a row of 42 percent let the test pass **with no live message at
   all**.
3. **It trusted the arithmetic of the server.** `PATCH /api/me/progress/:id`
   keeps the `progress` of the body, and a body that holds `isFinished` beside it
   gives the fraction of the position that stood there before: the measurement of
   2026-08-12 sent `progress: 0.10` and read `0.1722` back. A body of `progress`
   alone keeps the value.

The test takes a media of more than 1000 seconds now, it writes **two** positions
and it waits for each of them, and it reads the value of the second one **from the
server**: the box must hold a value that the message `init` never carried, and one
`assert_ne!` holds that rule. The mark goes away in a request of its own. Three
runs one after the other give the test each time.

### T-133: a program that a user builds keeps no token, and it stops with a screen of no character

**The maintainer met this**: the login took the address, the name, and the
password of a server that answers, and then the program stood with an empty
screen for ever. The kill and the start after it asked for the address, the name,
and the password again.

**The program had no secret key.** `install.sh` makes 32 bytes and it writes
`TOUTUI_SECRET_KEY=…` in `.env`, and **no other way to the program does that
work**: `cargo build`, `cargo install`, `nix`, and a package of a system all give
a program with no key. The measurement of 2026-08-13, with a configuration
directory of no file:

```
[INFO] - [auth_input] Login
[ERROR] - No secret key is present. Do this: …
```

and no line after it. The table `users` held no row.

Two faults stand behind that screen:

1. **The program waited for itself.** `encrypt_token` gave its fault, and
   `auth_process` wrote that fault with `println!`. That function runs on the
   thread of the login, and the login screen holds the lock of the standard
   output while it waits for that thread with `join`: the two threads then waited
   for each other for ever. **A `println!` of a thread of the login stops the
   program.** The user saw a screen of no character, and no key answered.
2. **A login that keeps no token gave no word.** The old code kept `""` for the
   token and it went on, therefore the row of the account would hold no token and
   the next start would show the login screen with no reason.

The three corrections:

- **The program makes the key itself**, at the start, when the machine has none:
  32 bytes of `/dev/urandom` in the form of the hexadecimal, in
  `<config>/.env`, and the file belongs to the user alone (mode 600). The
  function reads `.env` first, therefore **it never makes a second key**: a new
  key makes every token of every account unreadable.
- **No line of `auth_process` writes to the terminal.** The fault of the cipher
  goes to the log, and the login screen says "The program has no secret key,
  therefore it keeps no token. See the log." A login that keeps no token is a
  login that failed, and it says so.
- `App::new` wrote the fault of the decipher with `println!` too, on the cells of
  a frame of ratatui. That line goes to the log now.

The measurement after the correction, with a configuration directory of no file:
the login gives the Home view in 5 seconds, `.env` holds the key with the mode
600, and the start after it gives the Home view with no question.

### T-134: the cursor of the terminal stood six rows below the field of the user

**The maintainer met this**: "when I am typing something and an error or some
prompt comes, the blinking cursor gets moved to the error area". The user writes
in the field, and the cursor of the terminal blinks at the end of a message far
from it.

**The login screen wrote its message outside the buffer of ratatui.**
`pop_message` moved the cursor of the terminal with `MoveTo(0, rows - 6)` and it
wrote the message with `println!`, **after** the frame of ratatui set the cursor
in the field. The last word wins: the terminal keeps the cursor at the end of the
message. `clear_message` did the same work at each turn of the loop, therefore
**the cursor was wrong at every moment of the login screen**, and not only when a
message stood. A measurement of 2026-08-13 in a terminal of 100 by 30: the cursor
stood at the column 69 of the row 24, and the field of the user stood at the row
14.

The message stands **inside the frame** now, as `crate::logic::message` does for
every view of the application (T-59), and the cursor comes last: it stands in the
field, always. The module `pop_up_message` had one user, and it has none:
therefore it goes away, and no function of the program can move the cursor
outside a frame again.

Two rules of the screen come with it:

- **The size comes at each frame.** The old code took it one time before the
  loop, therefore a terminal that became small while the login screen stood held
  a field outside the screen. The box of `crate::logic::prompt` learned that rule
  in T-115.
- **A frame with no message writes the cells of that row again**, therefore no
  old message stays and `clear_message` is not needed. The old code needed that
  function, and a message that was shorter than the message before it kept the
  end of the old one.

The measurement after the correction, in a terminal of 100 by 30: the cursor
stands at the column 26 of the row 14 with no message, at the column 33 after
seven letters and a message, and at the column 36 after three more letters. The
message goes away with the frame that no longer holds it.

### T-135: a refresh of the screen took the timer for sleep of the user away

**The road of the session before this one named it**, and the measurement of
2026-08-13 gave it: the key `t` gave the row of the player `💤 4:58`, the key `R`
took that text away, and the book played on. **The user set a media to stop after
five minutes, and it would play for ever.**

**Every key that refreshes the screen makes a new application** (T-131), and
every field of a new application starts at its first value: `sleep` and
`sleep_choice` are `None`. T-131 gave the engine of the playback to the new
application, therefore the media plays on with the keys of the user — and with no
timer.

The new application takes the timer of the application before it now.
`the_state_that_a_refresh_keeps` gives that state, and
`keep_the_state_of_the_application_before` takes it: **one name holds the rule**,
and a field that a later session adds stands in one place.

**The identity of the playback needs no correction.** The engine belongs to the
playback, therefore `playback_id` of the timer is the identity of the media that
plays, and `sleep_timer::action_for` measures the same media after the refresh.

**The queue of the media needs no such work.** The handover of the session before
this one named it beside the timer, and a measurement of the source says
otherwise: `crate::logic::queue` holds the queue in a slot of the module, and no
line of `App::new` touches it. The line of the queue that the user selected
starts again, and the view of the queue takes the first line of a list that
changed anyway.

The measurement after the correction: `💤 4:58` before the key `R`, and `💤 4:50`
after it. The timer measures the same playback, therefore the time of it goes on.

### The sweep of a library that the account may not read, 2026-08-13

**The road named this sweep in three sessions, and no session had made it.** The
account `toutuilimited` of the sandbox is of the type `user`, its permission
`download` is false, and it may read one library of the five. The sweep found
**one fault, and it locks the account out of the program for ever** (T-136).

What the measurement gave, before that fault:

- **An account of one library of five works.** The Home view, the Library view,
  and the key Shift+Tab all hold that one library, and the key says "This server
  holds one library".
- **`librariesAccessible` of an account with no library is every library.**
  Audiobookshelf 2.36.0 reads an empty list as "every library", therefore an
  account of no library at all does not exist and `library_ids[0]` of
  `auth_process` finds a library for every account that can log in.
- **A `PATCH /api/users/:id` takes `librariesAccessible` inside `permissions`
  only.** The same name beside `permissions` gives `200` and it changes nothing.
  A session that measures a permission must read the account again after the
  request.

### T-136: an account that loses a library cannot use the program again

**A library of an account can go away while the program of that account holds
it**: an administrator changes `librariesAccessible`, and the server then answers
`403` for every request of that library. The database of the program holds
`id_selected_lib`, and no line of the program looked at that answer.

The measurement of 2026-08-13, with the account `toutuilimited` and the library
`Books` taken away while the program ran:

| The key | What the user saw |
|---|---|
| — (the key `R`) | `📖  ()` in the header, and "This library holds no media. Press L to tell the server to examine the library." |
| `Tab` | "The server gave no shelf for this library." |
| `S-Tab` | "This server holds one library", and the view did not change |
| a new start | the same screen, for ever |

Three faults of one cause:

1. **The program said a reason that it does not have.** The library holds 15
   media, and the account may not read them. The program named the library empty,
   and it asked the user to press `L` for a scan of the server — a request that
   the account may not make either. The rule of T-91 says that a view must never
   say a reason of its own making.
2. **The header lost the name and the kind of the library**, because the library
   of the database stands in no list of the account.
3. **No key gave the user the library that they may read.** `the_next_library`
   gives nothing for a library that the list does not hold, and that rule is
   right: the program must not guess. **The start is the place of the answer.**

`the_library_that_the_program_must_take` gives the first library of the account
when the library of the database is not one of them, and `App::new` writes it in
the database before every request of the start. The user reads "Your account
cannot read the library of this program. It shows "Books" now.", and the log
holds the two identities.

**The offline mode holds no library**, therefore a list of no library changes
nothing: a program that wrote a library there would forget the library of the
user.

The measurement after the correction: the start gives the Home view of the
library that the account may read, with the message, in one start and with no key
of the user.

### T-137: the covers of the graphics protocol, and the harness that reads them

**The maintainer said on 2026-08-13: "when I press `R` and an image was
rendering, it gets stuck; after three refreshes the new images render under it".**
The measurement of that day did not repeat the fault, and the maintainer could
not repeat it either. **The tool of the measurement is the answer of this item**,
because no session could see a cover at all before it.

**A screen of tmux holds no byte of a cover.** The program inside tmux asks the
terminal nothing, therefore `drive.sh` measures the covers of blocks of Unicode
only (`asks_the_terminal` of `src/ui/cover.rs`). The terminal of the maintainer
answers `Kitty with a font of 9 by 20 pixels`, and that protocol draws a picture
with **unicode placeholders**: every cell of the picture holds the character
U+10EEEE, and **the colour of the letter of that cell holds the identity of the
picture**.

Therefore the letters of a screen of kitty say where every picture stands, and
which picture it is. `docs/harness/kitty.sh` opens a window of kitty with the
remote control of kitty and it runs the program under `script`;
`docs/harness/covers.py` reads the screen and the bytes of the program together,
and it names:

- the row, the column, and the number of cells of each picture of the screen;
- the identity of each picture of the screen;
- **a placeholder of a picture that the program did not send**, and that is the
  fault of a picture that the terminal cannot draw.

The measurements of 2026-08-13, each of them in a real window of kitty:

| The measurement | The answer |
|---|---|
| The start, and the cover of the first line | one picture, and the program sent one |
| A key that moves the line, with a cover of the new line | one picture, and the picture of the line before it goes away |
| The key `R` one time | one picture, of a new identity |
| **The key `R` three times, one after the other** | **one picture**, and no placeholder of the three pictures before it |
| The key `R` 150 milliseconds after a key that asks for a new cover | one picture |
| A media that plays, with the cover of the media that plays | one picture of 72 by 32 cells |
| **A line of a series while a media plays: four pictures at one time** | **four pictures**, each of them in the rectangle of `plan_covers` |
| The key `R` in that state | one picture, and no placeholder of the four before it |
| 27 pictures of one window | **the memory of kitty did not move** (94452 kB before and after) |

**Every measurement gave the same answer**: the pictures of the screen are the
pictures that the program sent, and a picture that no line of the screen holds
goes away. **kitty takes a picture away with its placeholders**, therefore the
program needs no request that deletes a picture: the comment of `transmit_virtual`
of `ratatui-image` says so, and the memory of the terminal proves it.

**What a next session must ask the maintainer, when the fault comes again:** the
view, the state of the playback, and the size of the terminal at that moment, and
then the report of `the_covers`. One line of that report tells the two faults
apart: a placeholder of a picture that the program did not send is a fault of the
program, and a picture of an old moment with its own identity is a fault of the
terminal.

### The sweep of two accounts of two servers while a media plays, 2026-08-13

**The road named this sweep in four sessions, and no session had made it.** It is
the last sweep of the road, and it found **two faults of one shape**: both of
them take the place of a media of the user away.

The measurement: the account `toutuitest` of the server of the port 13399 played
"A Long Test Book" (30 minutes, the speed 1.30), and the account `secondtest` of
the port 13400 held one book of its own. `docs/harness/drive.sh` drove the
program, and `curl` read `GET /api/me` of each server.

| The measurement | The answer |
|---|---|
| The key `a` at the minute 13:31 of the book | **one fault** (T-139): the server held **13:23**, and the program held 13:31 |
| The key `c` at the minute 10:25 of the book of the second account | the same fault: the server held 10:20 |
| The next media of the second account, with a session of the first account in the database | **one fault** (T-138), and it destroys the place: `The server does not have this item`, and the row went away |
| The login of a second account on a second server | **no fault.** The header holds the address and the library of that server, and every view holds its data |
| The key `c` back to the first account | **no fault.** Every value of the screen belongs to the account that starts |

### T-138: the place of one account went to the server of another account

**One row of `listening_session` stood for the whole program, and that row held
no account at all.** `insert_listening_session` began with
`DELETE FROM listening_session`, therefore the program held the session of the
account that played last.

The sequence of the fault, with the two accounts of the sweep:

1. `toutuitest` played a book of the server of the port 13399, and the row held
   the item and the second 810.
2. The key `c` gave the start to `secondtest`, and the program started again. The
   row stayed: no line of the program closed the session (T-139).
3. `secondtest` played its own book. The program closes the session of the
   database before a new playback (`wait_prev_session_finished`), therefore it
   sent the position of a book of **the other server**.
4. The server of the port 13400 answered **"The server does not have this
   item"**, for the close and for the position both.
5. `delete_listening_session` removed the row, because that answer is not the
   answer of a server that does not answer: `pending_progress` of T-25 keeps a
   position of the offline mode only.

**The place of the user went away**, and no line of the screen said it. The
server of the first account kept the position of its last sync, and the minutes
after it are lost.

The row holds the account and the server now (the version 8 of the schema), and
that is the rule of the queue of the version 7 already: **a user with an account
on two servers keeps one session for each of them.** `get_listening_session` and
`delete_listening_session` take the account, therefore:

- The program of one account never reads the session of another account.
- A session that waits **stays** while another account plays, and it reaches its
  own server when its own account plays again. The measurement after the
  correction: the row of 833 seconds stayed while `secondtest` played, and the
  log of the next start of `toutuitest` says
  `Item 9a671047… closed at 833s (not finished)` with no refusal.

**A row that an older program wrote holds no account.** The two columns are empty
for such a row, and the account that asks takes it: a database of an older
version holds the row of the one account that program had.

### T-139: the place of a playback did not reach the server before the program started again

**Every key of the view of the accounts starts the program again with `exec`**
(T-123 and T-124), and `exec` takes every task of this process away. No line of
the program sent the position of the playback that it stops, therefore the server
kept the position of the last sync of the loop.

The measurement of 2026-08-13: the key `a` at the minute **13:31** of a book, and
`GET /api/me` of that server then said **803 seconds (13:23)**. The loop of the
playback writes the position of every second in the database, therefore the
place of the user stood in the database and it went nowhere.

**A key handler cannot wait for the server**: `handle_key` is not asynchronous,
and a task that it starts dies with `exec`. Therefore the handler writes the
request in `the_program_starts_again`, and **the loop of `src/main.rs` does the
work**: it says a word to the user, it stops the engine, it closes the session
with `sync_session_from_database`, and it then starts the program again. That is
the shape of the key `Q` already.

The measurement after the correction: the screen said **13:43** at the moment of
the key `a`, and the server holds **823 seconds** — the same second. The three
keys that start the program again (`a`, `c`, and a log out of the one account)
take that path, and `tests/the_session_belongs_to_one_account.rs` holds the rule
of the loop with a test that reads the source, as T-131 and T-135 do.

### T-140: two programs of one account destroyed the place of both users

**The sweep of 2026-08-13: two programs of one account, on one database, while a
media plays.** The table of the road held no new condition after T-138 and
T-139, therefore this session named its own: a user starts the program in two
terminals, with the same account and the same server. Every measurement of every
session before this one ran **one** program.

**One row of `listening_session` stood for one account** (the version 8 of the
schema, T-138), therefore the two programs of that account shared it. The
measurement gave three effects of that one cause, and each of them takes the
place of a user away.

The two books of the measurement hold eight hours each, because the device
`null` plays a book of 30 minutes in about 40 seconds (the trap 14 of the
harness). `docs/TEST-SERVER.md` holds the command that makes them.

| The moment | Before | After |
|---|---|---|
| The program B plays its own book, while A plays | B closes the **live session of A** on the server, and it removes the row of A | B leaves the row of A, and the log says "The database holds no session to close" |
| The key `Q` of A, while B plays | A sends the position of **the book of B**, and it removes the row of B | A closes its own session, and the row of B stays |
| The key `Q` of B | "The database holds no session to close": **the place of B reaches no server at all** | B closes its own session |
| The server, after the two keys | the book of A: **73 s** of 114, the book of B: **0 s** of 116 | the book of A: 107 s, the book of B: 108 s |

**The first effect comes of the rule of T-4.** `play_media` closes the session
that the database holds before it opens its own, because a program that stopped
without a correct exit leaves a row. That rule cannot tell the row of a program
that **died** from the row of a program that **lives**, and the answer is
therefore the identity of the program:

- `owner` holds the program of the row (the version 9 of the schema), and the
  identity of the process is enough: two programs of one machine never hold one
  number at one moment. **This needs no dependency**, and the rule of T-20 holds.
- `heartbeat` holds the moment of the last second of that playback. The loop
  writes the position of every second already, therefore `update_current_time`
  writes that moment too.
- A program takes a row of its own, **or** a row that stood still for
  `THE_LIMIT_OF_THE_HEARTBEAT` (30 seconds): that is the row of a program that
  stopped without a correct exit, and the rule of T-4 keeps it. The limit is
  longer than one second, because the loop writes nothing while the engine seeks
  to the place of the user (T-38).

**A row of an older program holds no owner and the moment 0**, therefore it is
old and the program that asks takes it. That is the same answer that the version
8 gives to a row with no account.

**The key `Space` shared that row too.** `handle_key_player` reads the session of
the account to write the mark of the pause, and the key of one program therefore
wrote the mark of the other one. The rule of the owner corrects that key with no
line of its own.

`tests/the_session_belongs_to_one_program.rs` holds the six rules, and the sweep
after the correction gave the table above.

### T-141: a media that came to its end left its row, and a later key destroyed a newer place

**The sweep of T-140 found this one beside it.** A book of eight hours played to
its end, and the program stayed open: the row of `listening_session` then held
`t=28800` and `finished=1`, and the server held the same values. **The position
was safe on the server, and the row stayed.**

The measurement of 2026-08-13, with the program open after the end of that book:

| The moment | Before | After |
|---|---|---|
| The end of the media | the row stays, `t=28800 finished=1` | no row |
| A different client marks the book "not finished" | the server holds 0 s | the same |
| The key `Q` | the program sends **28800 s and "finished"** again, and the place of the other client goes away | "The database holds no session to close", and the place of the other client stays |

**That is the fault of T-4 in a new place.** T-4 gave the answer for the start of
the program: the row goes away when the server holds the position. The loop of
the playback closes its own session and sends the position with
`close_and_report`, and no line removed the row after it.

`close_and_report` says now if the server took the position, and the loop removes
the row of **that playback** (`delete_the_session_of_a_playback`, by the identity
of the session, therefore the row of another program stays: T-140). **A server
that refused it keeps the row**, because the position of the user then lives in
that row only (T-25).

`tests/the_end_of_a_media_takes_its_row.rs` holds the two rules with the mock
server of `tests/playback_ownership.rs`.

### T-142: a second program removed the books of the user, and its own screen said the value that keeps them

**The sweep of 2026-08-13: two programs of one account, and one of them changes a
setting with the key `S`.** The road of the session of T-140 named that condition
and no session had measured it. **The key `S` writes one value of `config.toml`**:
the limit of the cache of the ebooks (T-77).

`write_the_value` reads the file and it changes one line (T-77), therefore **no
program loses the line of another program**: that is the fault that the road
expected, and the measurement did not find it. It found a worse one. **The limit
stands in three places of one program:**

| The place | Who writes it | What it does |
|---|---|---|
| `config.toml` | the key `l` of the view, and every program of the account | the value of the user |
| `self.config.reader.ebook_cache_mb` | `App::new`, therefore the start **and every refresh** (T-131) | the title and the mark `✓` of the view |
| a slot of `logic::reader::cache` | `src/main.rs`, one time at the start (T-72) | **the removal of a book of the disk** |

The task that removes a book holds no `App`, therefore that slot exists. **It
stood still for the life of the program**, and the three places then held two
different values.

The measurement, with two programs of one account in tmux, the sandbox, and a
cache of 447 megabytes of the disk:

| The moment | Before | After |
|---|---|---|
| The window A takes 4096 MB with `S` | `config.toml` holds 4096 | the same |
| The view of the settings of B, with no key of a refresh | **"The cache of the ebooks — 512 MB now"**, and the mark `✓` on 512 | "4096 MB now", and the mark on 4096 |
| The same view of B, after the key `R` | "4096 MB now": `App::new` reads the file again | the same |
| **B gets one book with the key `e`** | B **removed two books of 105386785 bytes**, and the log said "the cache of the ebooks holds 536870912 byte(s) at the most" | no book goes away |
| The message of B | "The cache of the ebooks was full. The program removed 2 books…" | no message |

**The screen of B promised 4096 MB at that moment, and B removed the books at 512
MB.** A user who gives a window of their program a cache of four gigabytes for a
journey loses the book that they downloaded for it.

The rule: **the file is the truth of that limit, and the program reads it at
every moment that it needs it.**

- `App::new` writes the slot with the value that it just read, therefore the
  start and every refresh give one value to the screen and to the task.
- `show_the_settings_of_the_reader` reads the file again, therefore the title and
  the mark say what the file holds and not what the program read hours before.
- **The removal reads the file again too** (`read_the_limit_of_the_configuration_again`),
  because the removal takes a book of the disk with no key of that window: a
  program of 2 kilobytes of reading holds the books of the user.

The value of the environment `TOUTUI_EBOOK_CACHE_BYTES` comes before the file, as
it did (T-71), and the text of the view says so.

`tests/the_limit_of_the_cache_follows_the_file.rs` holds the rule with no server
and no sound device: the test writes the file as the other program does, it makes
a new application, and it fails with `left: 536870912, right: 4294967296` when the
correction goes away.

### T-143: the key `h` did nothing in the view of the cache of the ebooks

**The measurement of T-142 found it with the first key that it pressed.** The
footer of that view says `h: back`, and three presses of `h` moved no line of the
screen. The next key of the measurement was `Esc`, and **that key stops the
program** (the trap 69): the measurement lost the program of the window B, and it
began again.

`AppView::SettingsReader` came with T-77 and it took an arm of `toggle_view` (the
key `Tab`) and no arm of the handler of the key `h`: the match ends with `_ => {}`,
therefore the key said nothing and did nothing. **A key that does nothing in one
view is a fault of its own** (T-79), and a footer must not promise a key that the
view does not have (T-118).

The key goes back to the settings now, as the four other views of the settings
do. **A view of a later session can forget that arm again**, therefore
`tests/the_key_h_leaves_every_view.rs` reads the source of the program: it names
every view of `AppView`, and it holds each of them to an arm of the handler of the
key `h`. The Home view, the Library view, and the reader of an ebook stand outside
that rule, because `h` is not a key that goes back in a list of media or in a
book. The test named `SettingsReader` and no other view.

### T-144: the gate of the machine passed and the gate of CI failed, because nextest gives each test a process

**The workflow of CI runs `cargo test --verbose`, and every session of this fork
ran `cargo nextest run`.** The two tools do not run a test in the same way:
**nextest gives each test a process of its own**, and `cargo test` runs the tests
of one binary in threads of one process. Three binaries of the tests hold a
database and a variable of the environment, and they therefore passed on the
machine of the session and failed on CI.

The run 31708670046 of CI, of the handover of the session of T-140 and T-141:
`a_server_that_refuses_the_position_keeps_the_row` failed in `build-and-test` and
in `nix`, and **the same test passes with nextest every time**. `cargo test`
stops at the first binary that fails, therefore that run named one test of the
six that this item corrects.

| The binary | The state that the tests share | The fault |
|---|---|---|
| `tests/the_end_of_a_media_takes_its_row.rs` | one database, and **one account** | `insert_listening_session` removes the row of the account of this program before it writes its own (T-138 and T-140): the second test took the row of the first one |
| `tests/the_session_belongs_to_one_account.rs` | `XDG_CONFIG_HOME`, and the accounts of the rows | a test held the account "second" to no row, and another test wrote that row. **A row with no account belongs to every account** (T-138), therefore that row crosses every test too |
| `tests/the_session_belongs_to_one_program.rs` | `XDG_CONFIG_HOME`, and the count of every row of the table | four tests of six failed: `the_rows_of_the_table` counted the rows of another test |

**`XDG_CONFIG_HOME` belongs to the process and not to the test**, and that is the
trap 29 of `docs/HANDOVER.md` in a new place: a test that writes it must stand
alone in its binary, **or the tests of that binary must run one at a time**.

The two answers of this item, and the shape of the test decides which one:

- **A test that reads one row takes an account of its own.** The two tests of
  `the_end_of_a_media_takes_its_row.rs` hold one database and two accounts now,
  and they run together.
- **A test that reads every row of a table takes the turn of the binary.**
  `a_database_of_the_test` gives a `MutexGuard` of the binary with the directory,
  therefore one test of that binary runs at one time. The guard takes the fault of
  a test that stopped inside its turn (`into_inner`), because a lock that is
  poisoned would stop every test after it.

**The gate of a session holds both commands now**: `cargo nextest run` for the
2.3 seconds of the work, and **`cargo test` before the last commit**, because that
is the command of CI. `cargo test --no-fail-fast` says every binary that fails,
and `cargo test` alone says the first one.

### T-145: the terminal of the user went away, and the next program threw the place of that playback away

**The sweep of 2026-08-13: a media that plays while the terminal goes away
(`SIGHUP`), and while the machine sleeps.** The road of the session of T-142
named that condition, and no session had measured it. **The sleep of the machine
gives no fault, and the death of the terminal takes the place of the user away.**

`tmux kill-session` gives the program the `SIGHUP` of a terminal that goes away.
The program dies at once: no line of the exit runs, therefore no request carries
the position. **The row of `listening_session` holds it**, and the loop of the
playback writes that row every second for exactly this condition ("Write the
position for each second. A crash must not lose it.").

The measurement, with a book of eight hours and the device `null`:

| The moment | The row of the disk | The server |
|---|---|---|
| The terminal goes away | **1026 s** | 872 s (the sync of ten seconds, T-3) |
| The user starts the program again **at once**, and plays a second book | the row stays, and the log says "The database holds no session to close" | 872 s |
| The key `Q` of that program | **the table holds no row at all** | **872 s**, for ever |

**The two rules of `sync_session_from_database` did not agree.** It closes
**one** session (`get_listening_session` gives one row, `LIMIT 1`), and it
removed **every** row that this program may take
(`delete_listening_session`). A user who starts the program again inside
`THE_LIMIT_OF_THE_HEARTBEAT` seconds meets both:

1. At the moment of the key `l`, the row of the program that died is **younger**
   than 30 seconds, therefore the rule of T-140 hides it: the play does not close
   it, and `insert_listening_session` leaves it beside the new row.
2. At the moment of the key `Q`, that row is **older** than 30 seconds, therefore
   the removal takes it. The close took the row of this program, and the removal
   took both.

**The place of the user of the first book was on the disk, and no request ever
carried it.** 154 seconds of that measurement, and a user of a real device loses
the listening of the ten seconds of the sync (T-3) — **and the whole playback
when the server did not answer at all**, because the row is then the one copy.

**The correction.** `get_the_sessions_to_close` gives **every** row that this
program may take, and `sync_session_from_database` closes each of them: one
`POST /api/session/:id/close`, one position, and then
`delete_the_session_of_a_playback` of **that** session alone. **No row goes away
without a request now.** `delete_listening_session` has no caller left, and it
went away with this item: a removal of every row of an account is the fault
itself.

**The rows of a program that died go first, and the row of this program goes
last** (`ORDER BY (owner = ?) ASC, heartbeat ASC`). Two rows of one media then
leave the newest position on the server: the same book gave 1024 s and then
3000 s, and the server holds 3000.

The same measurement after the correction: the key `Q` says
`Item 6ba57b9a… closed at 1026s` and `Item e2b76945… closed at 1771s`, and the
server holds **1026** for the book of the program that died.

`tests/a_program_that_died_keeps_the_place_of_the_user.rs` holds the two rules
with a mock server. **The sequence of the two rows is the condition** (the trap
94): the test writes the row of this program first, because
`insert_listening_session` removes a row that stands still already.

**What stays, and it is a decision of this item.** The row of a program that
died stays hidden for 30 seconds (T-140), therefore a user who starts the
program again at once and plays **the same book** hears the ten seconds of the
sync a second time. The position is safe — the next close sends it, and the
newest position wins — and the answer of that last ten seconds needs the program
to know that the process of `owner` does not live. **That needs a call of the
system for each program**, and the decision of T-140 keeps it outside.

### The sleep of the machine, and it gives no fault

The freezer of the cgroup holds every thread of the program, as a suspend of the
machine does (`SIGSTOP` does not reach a program of tmux from a session of this
harness). A book of eight hours played, and the program stood still for **120
seconds**:

| The measurement | The answer |
|---|---|
| The position of the row and of the server, while the program sleeps | 536 s, and it did not move: **no clock of the wall stands in the loop of the playback** |
| The playback after the wake | it goes on, and the sync reaches the server (1292 s after 20 seconds) |
| The connection of the live messages | "The connection ended: the server did not answer in time", and it is open again **10 seconds** later, with no key of the user |
| The row of the database | the same row, the same owner: a program that sleeps keeps its own row |
| The key `Q` after the wake | the session closes, and the server holds 1331 s |

**The limit of that measurement:** the device `null` is not a real sound device,
therefore it says nothing of an ALSA device that a suspend takes away. A
measurement of that needs the sound of the machine of a user, and the rule of the
handover keeps a run of a session silent.

### T-146: a queue of media, and a server that goes away in the middle of it

**The sweep of 2026-08-13: a queue of media while the server goes away.** The
road of the session of T-145 named that condition, and no session had measured
it: the queue of T-56 lives in the database, and the sweep of an offline server
(T-91) held no queue. **The condition found one fault, and it takes a media of
the user away for ever.**

**The queue took the media out before the playback of that media started.** A
playback that then did not start left nothing: the entry stood in no list, and
no key gave it back.

The measurement, with the sandbox and tmux. The queue held "One Chapter Book"
and "Alice in Wonderland", the book of 30 minutes played, and
`podman stop -t 0 abs-test` took the server away in the middle of it:

| The moment | The queue |
|---|---|
| Before the playback | **2 items**: One Chapter Book, Alice in Wonderland |
| The book comes to its end | the log says `the queue starts "One Chapter Book", and 1 media wait` |
| The server does not answer, and the disk holds no copy of that media | the log says `the disk has no copy of d8f33299…`, and the message says so |
| The view of the queue, and the queue of the disk after it | **1 item**: Alice in Wonderland. **One Chapter Book is gone** |

**The queue stopped there too.** `the_queue_goes_on` gives `false` for
`Outcome::Fault`, therefore the media after the fault never played and the user
pressed a key for it. The head of `crate::logic::queue` promised the opposite
("the queue then goes on to the media after it"), and no code said it.

**Two keys met the same rule.** The loop of the playback takes the next media
with `queue::take_next` at the end of a media, and the key `l` of the view of
the queue takes the media with `queue::take_at` before it plays it. Each of them
removed the media, and neither of them gave it back.

**The correction: a playback that did not start gives its media back to the
front of the queue.** `the_media_goes_back_to_the_queue(outcome)` is that rule,
and it reads `Outcome::Fault` — the outcome that says that no audio played at
all: the server did not answer, or it gave no stream, or the item holds no audio
file, or the disk holds no copy. An end and a stop keep the queue as it is,
because the user heard that media.

**The queue must not go on after a fault, and that stays.** A server that does
not answer gives the same fault to every media of the queue, therefore a queue
that goes on empties itself in one second. The queue stops at the media that it
cannot play, and it keeps every media that it holds.

`play_the_media_of_the_queue` is the second door: the key `l` of the view of the
queue gives the whole entry now, and `play` gives the target alone.
`the_loop_of_the_playback` holds both, and it carries the entry of the media that
came of the queue.

The same measurements after the correction:

| The measurement | The answer |
|---|---|
| The book of 30 minutes ends while the server is down | the log says "the playback of \"One Chapter Book\" did not start. The media goes back to the front of the queue, and the queue stops", and **the queue holds 2 items in the sequence of the user** |
| The program starts again, and the server answers | the disk gives 2 media, the key `l` plays them both, and the queue is empty at their end |
| **A media of the queue that the disk holds, while the server is down** | **the queue goes on**: "the offline mode plays Multi File Test Book at 60 seconds with 3 track(s)", and it stops at the media after it that the disk does not hold |
| The key `n` while the server does not answer | the media goes in the queue, and the disk holds it |
| The key `l` of the view of the queue while the server does not answer | the media stays at the front of the queue, and the queue holds both media |

`tests/the_queue_keeps_a_media_that_did_not_play.rs` holds the rule. The test
needs no server and no sound device: nothing listens on the port of `NO_SERVER`,
therefore the playback goes to the offline mode (T-25), the disk holds no copy,
and the outcome is `Fault`. A build with the correction removed says
`the playback did not start, therefore the queue must hold both media. It holds
["1. 📕 The Second Book — An Author  (1m)"]`.

**The words for the user stay as they are, and it is a decision.** The screen
says the reason ("The server does not answer, and the disk has no copy of this
media."), and the message before it named the media ("The queue starts \"One
Chapter Book\"."). The row of the message holds one message and one row,
therefore a second message about the queue would take the reason away. **The view
of the queue is the answer**: the media stands at its front, and the user reads
it with the key `q`.

### T-147: a second program of the account took the media out of the queue

**The sweep of 2026-08-13: a state of the program that a second program cannot
see.** The road of the session of T-142 named that condition, and it named the
queue of the media as one of the three states. **The queue holds that fault, and
it takes the media of the user away.**

`write_the_queue` holds **every** row of the queue (T-56), and the queue of the
process stood beside it: a program that changed its own memory therefore wrote
that memory over the media of every other program of the account.

The measurement, with the sandbox and two sessions of tmux of one
`XDG_CONFIG_HOME` (the trap 89):

| The moment | The window A | The window B | The table `queue` of the disk |
|---|---|---|---|
| A presses `n` on "One Chapter Book" | `The queue [1 item]`: One Chapter Book | — | One Chapter Book |
| B presses `n` on "Multi File Test Book" | `The queue [1 item]`: One Chapter Book | `The queue [1 item]`: Multi File Test Book | **Multi File Test Book alone** |

**The book of A is gone, and the screen of A still names it.** A key of A that
changes the queue then takes the book of B away in the same manner, and a start
of the program gives the user the media of the last write.

**The correction is the rule of T-142 for the disk: the file is the truth, and
the program reads it at the moment that it uses it.** `read_the_disk` takes the
rows of the queue into the queue of the process, and every function that changes
the queue calls it first: `add`, `take_next`, `put_at_the_front`, and
`take_the_media`. `clear` needs no read, because it takes every media of the
account away.

**The view of the queue reads the disk when it opens** (the key `q`). The render
reads the queue of the process at every frame, therefore a read of the database
at each frame would pay for a change that comes some times in a day. A view that
stands open while a second program changes the queue is older than the disk until
the user leaves it and opens it again, and that is a decision of this item.

**A key of a view that is older than the disk takes the media of its own line.**
`the_place_of_the_media(entries, index, key)` gives the place when the media of
that place is the media of the line, and it gives the first media of that
identity otherwise; a media that stands in the queue no more gives nothing, and
the key then does nothing. The keys `l` and `X` of the view of the queue read
that rule. `queue::take_at` has no caller left, and it went away.

The same measurement after the correction:

| The moment | The window A | The window B | The disk |
|---|---|---|---|
| A presses `n`, and B presses `n` after it | `The queue [2 items]` | `The queue [2 items]` | **both media, in the sequence of the user** |
| B presses `X` on the first line, and A opens the view again | `The queue [1 item]`: Multi File Test Book | the same | the same |

`tests/the_queue_belongs_to_the_disk.rs` holds the rule. The test needs no
server: it writes the queue of the second program with `save_the_queue`, which is
the one function that every program of the account calls. A build with the
correction removed says
`left: ["The Book Of A", "The Second Book Of A"]` against
`right: ["The Book Of B", "The Second Book Of A"]`.

**The two states of the road that stay.** The cache of the ebooks holds the rule
already (T-142), and **the downloads of the server** are the third state that no
session has measured.

### T-148: two programs of the account wrote one file of a download

**The sweep of 2026-08-13: the downloads, with two programs of one account.**
This is the third state of the shape of T-142 and of T-147, and the road named
it in two sessions. **The two programs write the same file at the same time, and
the file that stays is not the file of the server.**

The key `D` spawns a task, and no line of the program asks if that media comes
already: **the map of the progress is a map of the process** (`OnceLock` of
`logic::download`), therefore it says nothing of a second program. The two
writers then meet in the directory of the download:

1. The first program finds no `.part` file. It opens that file with `truncate`,
   and it writes the answer of the server from the byte 0.
2. The second program finds a `.part` file of some megabytes. `resume_from`
   gives that number, the request holds `Range: bytes=<the number>-`, and the
   answer goes to the **end** of the same file with `append`.
3. Each writer counts its own bytes, therefore the guard `written != file.size`
   of `fetch_one` passes for both of them. The first `rename` gives the file its
   name, and the second one says `cannot rename …: No such file or directory`.

**The measurement, with the sandbox and two sessions of tmux of one
`XDG_CONFIG_HOME` (the trap 89)**, on the book "A Book Of Many Hours" of eight
hours and one file of 115200330 bytes:

| The measurement | The answer |
|---|---|
| The bytes on the disk, four runs of the key `D` in two windows | **116576586, 115200330, 117316426, 117123402** |
| The bytes of the server | 115200330 |
| The first 115200330 bytes of the file of 116576586 | **the file of the server**, `ef133993…` |
| The audio of that file, with `ffmpeg -f null -` | **8:07:24**, and `Header missing` with `Invalid data found when processing input` |
| The audio of the file of the server | 8:00:00, and no line of a fault |
| The screen of the two windows | **`[Downloaded]`**, and the message of one of them says `"A Book Of Many Hours" is now available offline.` |
| The screen of the other window | `Download failed for "A Book Of Many Hours": cannot rename …` |

**The user holds seven minutes of audio that no decoder reads, and the program
says that the media is available offline.** The offline mode plays that file when
the server is away, therefore this is the copy that the user has at the moment
that they have nothing else.

**Two presses of the key `D` of one program give the same shape.** The second
press of the same window took the same road, and the log of that measurement
holds the same `cannot rename` for a download that worked: **a message of a fault
for the user, and no fault at all.**

**The correction: one program writes the files of one download, and the disk
says which one.** `logic::download::lock` makes the file
`.the-program-of-the-download` inside the directory of that download with
`create_new`, before the first byte. `fetch_item` takes that lock, and a second
program gives `TheFaultOfTheDownload::ADifferentProgramWritesTheFiles` and writes
nothing. The lock goes away with its value, therefore every return of the
function and every task that stops remove it.

**A program that died leaves its lock, and a lock is therefore not for ever.**
The rule is the heartbeat of T-140: the time of the lock **and the time of every
`.part` file of that directory** say when the program of that lock last worked,
and the newest of them decides. A download that stood still for 30 seconds
belongs to a program that is gone, and the next program takes the lock. **This
needs no call of the system and no dependency**, and a download of an hour holds
its lock for that hour because its file grows at each block.

The same measurement after the correction, four runs:

| The measurement | The answer |
|---|---|
| The bytes on the disk | **115200330**, four times of four |
| The sum of the file, against the file of the server | **the same**, `ef13399303c5150cb61c5bea50403299` |
| The window that came second | `A different program of this account downloads "A Book Of Many Hours" now.` |
| The log of that window | `[download_item] a different program downloads "A Book Of Many Hours" now` |

**The sentence of the second window must not say "failed".** The download of the
user is on its way already, and the user did nothing wrong.

`logic::download::fetch::tests::two_programs_of_one_account_do_not_write_one_file`
holds the rule with no server of the sandbox: a `.part` file of 40 bytes stands
on the disk, the answer of the mock server waits 300 milliseconds, and the second
program starts 100 milliseconds after the first one. A build with the correction
removed gives `left: Ok([…])` against `right: Err(ADifferentProgramWritesTheFiles)`.

**What this item does not measure.** The key `X` of one window while the other
window downloads: the two keys took their sequence in every run of the
measurement, because the download of 115 megabytes over the loopback ends in less
than one second. A removal **inside** a download needs a server that sends the
body slowly, and the address of the download does not go through the proxy of the
harness (T-149).

### T-149: the download goes to the address of the login, and it waits for ever

**The measurement of T-148 found it.** A proxy of the harness held the address of
the login, and the header of the program said a different address: **`pool` holds
every request of the program (T-105, T-107, and T-128), and the key `D` holds
`self.server_address`**, which is the address of the row of the database.

The measurement, with the block `[[servers]]` of `config.toml` of two addresses
and the address of the login second:

| The measurement | The answer |
|---|---|
| The header of the program | `🔗 localhost:13399`, the address that the pool holds |
| The two requests of the key `D` | **`127.0.0.1:13500`**, the address of the login: `GET /api/items/:id` and `GET /api/items/:id/file/:ino/download` |
| The same measurement with an address of the login that **does not answer** | **no message, no line of the log, and no bar of the progress**: the program said `Downloading "A Book Of Many Hours" for offline listening...` and then nothing at all |

**The second row is the harm of this item, and it is the shape of a user away
from home**: the address of the login is the address of the house, the public
address answers every view, and the key `D` then gives the user a silence with no
end. `reqwest::Client::new()` of `logic::download` holds **no limit of time at
all**, therefore a connection that no machine answers waits for ever.

**The correction.** The key `D` takes `api.pool().an_address()`, as the playback
does (T-138), and the client of the download holds two limits: **3 seconds for
the connection** (`api::client::CONNECT_TIMEOUT`) and **30 seconds with no byte**
of the answer. The request of the list of the audio files takes
`REQUEST_TIMEOUT`, the 15 seconds of every other request of the program, because
that answer is small. **A limit of the whole download must not exist** — the send
of a book of 479 megabytes took 36 seconds in the measurement of T-119, and a
book of some gigabytes takes much more.

The same measurements after the correction:

| The measurement | The answer |
|---|---|
| The requests of the key `D`, with the proxy on the address of the login | **no request at all**: the download went to the address of the pool, and the file holds the 115200330 bytes of the server |
| The key `D` while the address of the pool answers nothing | **`Download failed for "A Book Of Many Hours": the request failed: error sending request for url (http://127.0.0.1:13500/api/items/…)`**, at the second 15 |

The second measurement takes a port that accepts the connection and that answers
nothing at all (`blackhole.py` of the harness of the session), because a port
that no program holds **refuses** a connection at once and it says nothing of a
limit of time.

**No unit test reaches a key handler of `src/app.rs`, and no test reads the
limits of a client of `reqwest`.**
`tests/the_download_takes_the_address_that_answers.rs` therefore reads the
source, as the test of T-131 and the test of T-143 do: the handler of the key `D`
must name `pool().an_address()`, the client of a download must hold
`connect_timeout` and `read_timeout` and **no `timeout`**, and no line of
`logic::download` may make a `reqwest::Client::new()`.

### T-150: the key `X` read the database, and the disk held what the database did not

**The sweep of 2026-08-14: the key `X` of one window while the other window
downloads.** The road named that condition in the session of T-148, and that
session could not measure it: a download of 115 megabytes over the loopback ends
in less than one second (the trap 111). `docs/harness/slow_body.py` gives the
delay of the **body** of the answer, and the key `D` takes the address of the
pool since T-149: a proxy of 0.05 seconds for each block of 64 kilobytes in the
block `[[servers]]` of `config.toml` makes the download of that book take about
90 seconds, and the two keys then meet.

**`remove_download` read the database, and the database holds a row after the
last byte of the last file.** A download that runs and a download that stopped
therefore stand in no row at all: the key `X` found nothing, it removed nothing,
and it said that the media holds no local copy.

The measurement, on "A Second Book Of Many Hours" of eight hours and one file of
115200330 bytes, with two sessions of tmux of one `XDG_CONFIG_HOME` (the trap
89):

| The measurement | The answer |
|---|---|
| The window B presses `X` while the window A downloads | **`"A Second Book Of Many Hours" holds no local copy and no ebook.`** |
| The disk, one second after that key | the `.part` file of 14941771 bytes, and it grows |
| The disk, 60 seconds after that key | **115200330 bytes, and the row of the database**: the copy came, and the key of the user did nothing at all |
| The program dies at the second 6 of a download (the trap 103) | the disk holds **7713867 bytes** of a `.part` file and the lock of T-148 |
| The key `X` of the next program, on that media | **`"A Second Book Of Many Hours" holds no local copy and no ebook.`**, and the 7713867 bytes stay |
| Every other key of the program, on that media | **no key removes those bytes**, and no view names them |

**The second row of that table is the harm.** A book of some gigabytes that
stopped at its half leaves that half on the disk of the user for ever: the
program says that the media holds no local copy, the offline mode plays no part
of a download, and the user must find the directory of the downloads themselves.

**The correction is the rule of T-142, of T-147, and of T-148: the disk is the
truth.** `remove_the_directory_of_the_download` removes every file of the
directory of that download — the audio, the `.part` file, and the lock — and then
the directory, whatever the database holds. `remove_download` gives
`TheAudioOfTheRemoval` now, and the three values of it give three sentences: the
whole copy, the bytes of a download that did not come to its end, and nothing.
**The sentence of a part must not say "the local copy"**: the offline mode plays
no such file, therefore the user never had a copy.

**A download that runs holds its files, and the key must take none of them.**
That is the shape of T-148 from the other side: a removal under a writer unlinks
the file that the writer holds open, the writer then writes its bytes to an inode
that no name holds, and the `rename` of `fetch_one` says `No such file or
directory` — **a message of a fault for a download that the user wants**. The
lock of T-148 is the answer, and it needs no new state:
`lock::a_program_writes_the_files` says that a lock stands and that it did not
stand still for 30 seconds. The map of the progress of the process says which
program that is (T-148), therefore the two sentences name the program of this
window and the program of the other window.

The same measurements after the correction:

| The measurement | The answer |
|---|---|
| The window B presses `X` while the window A downloads | **`A different program of this account downloads "A Second Book Of Many Hours" now. The key X removes it when that download ends.`**, and the `.part` file of 8388171 bytes stays |
| The window A presses `X` while the window A downloads | **`This program downloads "A Second Book Of Many Hours" now. …`** |
| The key `X` on the 7713867 bytes of a program that died | **`Removed 7 MB of a download of "A Second Book Of Many Hours" that did not come to its end.`**, and the directory is gone |
| The key `X` after that download came to its end | the directory is gone, and the table `downloads` holds no row of that media |

**No key of this program stops a download that runs, and that is a decision.**
The key `D` spawns a task, and a key that stops it needs a map of the handles of
the tasks of this process — and it reaches the download of this program alone,
because the download of the other window belongs to a different process. The
sentence therefore says the truth and it promises no key that the program does
not hold (T-118 and T-143): the user presses `X` again when the download ends.

`tests/the_key_x_takes_the_disk_of_the_download.rs` holds the rule with no
server. It writes `XDG_DATA_HOME`, therefore it stands alone in its binary and
every part of it stands in one function (the trap 8 of the harness). A build with
the correction removed gives `left: Nothing` against
`right: ThePartOfADownload(1000)`. **No unit test reaches a key handler of
`src/app.rs`**, therefore the last part of that test reads the source, as the
tests of T-131, T-143, and T-149 do: the handler of the key `X` must name
`the_work_of_the_key_that_removes` **before** it names `remove_download`.

### T-151: a queue of media that a second program plays, and the key that said nothing

**The sweep of 2026-08-14: a queue of media that a second program plays.** The
road named that condition in the session of T-148: the window A plays the queue,
and the window B takes a media out of it with the key `X`.

**The queue itself holds, and that is the first measurement of a condition of the
road that found no fault of the data.** The rule of T-147 does the work: every
function that changes the queue reads the disk before it writes.

| The measurement | The answer |
|---|---|
| The window B takes the next media of the queue out with `X`, while A plays | **no fault**: the log of A says `the media came to its end. The queue starts "A Book Of Many Hours", and 0 media wait`, and that is the media that B left |
| The queue of the disk after it | empty, and the media that B removed came back never |
| The key `X` of A on a line whose media B took out | **no fault of the data**: A took nothing, and the media of the other line stayed |
| The view of A after that key | the disk, with 1 item |

**One fault stands beside it, and it is a fault of the words.** The key `X` of A
on a line whose media a different program took out **said nothing at all**:
`remove_from_the_queue` returned on `None` of `take_the_media` with no message.
The list of that view then lost the line all the same, because the render reads
the queue that `take_the_media` read of the disk — **the user therefore reads a
list of one media less and no word, and they cannot tell the key that worked from
the key that did nothing**. That is the rule of T-79: a key that does nothing is a
fault of its own.

The correction: `queue::text_of_the_key_that_takes` is a pure function of the
title of the line and of the media that went out, and the two roads give one
sentence — `"<the title>" is not in the queue now.` **The sentence names no
program**: this program cannot say which program took that media out, and a text
must not say a reason that the program does not have (T-91).

The measurement after the correction, with two windows of one `XDG_CONFIG_HOME`:
the window B takes "A Second Book Of Many Hours" out, the key `X` of A on its
line of that media says `"A Second Book Of Many Hours" is not in the queue now.`,
and the disk keeps "Depthless Hunger, Book 2".

`tests/the_queue_belongs_to_the_disk.rs` holds the rule: the line that said "the
key then does nothing at all" now holds the sentence of both roads, and the last
part of it reads `src/app.rs`, because no unit test reaches a key handler. A
build with the correction removed fails at that part.

### T-152: a program that dies while the server does not answer, and the playback that went away

**The sweep of 2026-08-14: the sharp form of T-145.** The road named this
condition in the session of T-150: the session of T-145 measured a program that
dies while the server **answers**, and the row of `listening_session` then holds
the position for the next program of the account. **An offline playback has no
such row at all.** `play_offline` opens no session on the server, and no request
of that playback ever reaches the server: **the row of the disk is the one copy
of the whole playback.**

`follow_playback_offline` wrote the place of the user to the row of the download
at each second, and it kept that place for the server — the table
`pending_progress` — **at the end of the loop only**. A program that dies reaches
no end.

The measurement, with `podman stop -t 0 abs-test`, the book of eight hours on the
disk, and `tmux kill-session` for the terminal that goes away (the trap 103):

| The moment | The row of the download | `pending_progress` | The server |
|---|---|---|---|
| Before the playback | 100 s | no row | 100 s |
| The offline playback runs | 1731 s | **no row** | 100 s |
| The terminal goes away | 1731 s | **no row** | 100 s |
| The server answers again, and the program starts again | 1731 s | no row | **100 s** |

**27 minutes of the book went away, and the one copy went away with them.** The
user then played that book with the server up: the program took the 100 seconds
of the server, and the loop of that playback wrote 100 over the 1731 of the row
of the download. The `Home` view held `37%` of a day before, the playback started
at the second 100, and the row of the disk held 366 seconds ten seconds later.

**The correction is the rule of the loop of the online playback**: "Write the
position for each second. A crash must not lose it." The loop of the offline
playback keeps the place of the user for the server at each second, in the same
way that it writes that place to the row of the download at each second.
`INSERT OR REPLACE` gives one row of a media whatever the number of the calls,
therefore the cost is one row of the disk for each second — the cost of the loop
of the online playback.

**`remember_progress` says one line of the log, and the loop must say none**: one
line for each second gives 28800 lines for a book of eight hours.
`logic::offline::keep_progress` writes the row and it says nothing, and
`remember_progress` calls it and says the line. The two callers of the end of the
loop keep `remember_progress`, because they carry the value of `finished`.

The same measurement after the correction:

| The moment | The row of the download | `pending_progress` | The server |
|---|---|---|---|
| The offline playback runs | 1154 s | **1154 s** | 100 s |
| The terminal goes away | 1154 s | **1154 s** | 100 s |
| The server answers again, and the program starts again | 1154 s | no row | **1154 s** |

The log of that start says `[offline] 1 position(s) wait for the server` and
`[offline] the server took the position 1154s of 6ba57b9a…`, and `curl` of the
endpoint `GET /api/me/progress/:id` gives `"currentTime": 1154`. The flush of
`app.rs` runs **before** the program asks the server for anything more,
therefore the place of the user is on the server before the user can press a key.

`tests/the_position_of_an_offline_playback_survives_a_program_that_dies.rs` holds
the rule: the test drives `follow_playback_offline` with the engine of
`PlayerHandle::without_engine`, it takes the loop away with `abort` in the middle
of the playback — that is the death of the program — and it asks for the position
that waits. A build with the correction removed gives `left: 0, right: 1`. The
test holds a clock of its own (`start_paused = true`), and it makes no request.

### T-153: two programs of one account that read one ebook, and the book that went away under the reader

**The sweep of 2026-08-14: two programs of one account and the cache of the
ebooks.** The road named this condition in the session of T-150. The measurement
took two forms, and the second one holds the fault.

**The first form: the two windows open the same book.** The window A pressed `e`
on "A Huge Book Of A Scan" of 502745447 bytes, and the window B pressed `e` on
the same book 4 seconds later.

| The measurement | The answer |
|---|---|
| The PDF of the disk after the two downloads | **502745447 bytes**, the size of the file of the server |
| The two children of T-62 | `a child read 150 page(s) in 131214 ms` and `… in 131872 ms` |
| The pages of the disk | one file of 43016313 bytes, and no `.part` file stayed |
| The two screens | **page 1 of 150**, and the picture of the page on each of them |
| The log | no line of a fault |

**No fault of the data, and a cost of two.** The two children each hold about a
gigabyte at their peak and each take 131 seconds, for one file that one child
gives. `std::fs::write` of the child writes the whole file in one call after the
parse, therefore the two writes of one `.part` path did not meet: the second
child renamed its file over the file of the first one, and the bytes of the two
are the bytes of one book. **A machine of less memory would meet the two peaks
at one time**, and that is a cost and not a fault of the data.

**The second form: the window B gets a book of its own, and it holds the fault.**
The window A read the book of 502 megabytes, and the window B pressed `e` on a
book of 105386785 bytes with a cache of 512 megabytes:

| The measurement | Before | After |
|---|---|---|
| The log of the removal of B | `the cache of the ebooks gave 545898521 bytes of 2 book(s) back` | **no line: the removal took nothing** |
| The PDF of 502745447 bytes that A reads | **gone** | stays |
| The 43016313 bytes of its pages of T-62 | **gone** | stays |
| The screen of A | page 1 of 150, and the book of the disk is gone under it | page 1 of 150 |

**`keep` is a fact of the process.** `the_ebooks_that_must_go` keeps the book
that **this** program reads, and one account holds more than one program
(T-140): the removal of B named its own book in `keep`, and it knew nothing of
the book of A. That is the shape of T-148 (`the map of the progress is a map of
the process`) and of T-150, and **the module of the cache says the rule that it
breaks**: "The book that the user reads now never goes away."

**The user loses no line and no place** — the reader of A holds the book in the
memory of its process, and the key `h` and the key `e` open it again from that
memory. **The user loses the bytes of the disk**: the next start of A asks the
server for 502 megabytes again and it waits 131 seconds for the child of T-62,
and a user with no server has no book at all.

**The correction is the rule of the fork: the disk is the truth.** The reader
writes the time of its file every 15 seconds
(`Reader::say_that_a_program_reads_this_book`, from the loop of `main.rs`), and
`the_ebooks_that_must_go` keeps every book of a time inside `THE_LIMIT_OF_THE_USE`
of 30 seconds. **The time of the file is the one word that two programs of one
account share here**, and `the_book_is_in_use` wrote that word at the open of a
book since T-67 already: the correction gives it a heartbeat, and that is the
rule of T-140 and of the lock of T-148. **It needs no new file, no call of the
system, and no dependency.**

**A mark of a reader is not for ever.** A window that goes away writes no more
marks, and its book is old 30 seconds later: the next removal takes it. Two
measurements hold that half of the rule, one of the disk and one of the pure
function.

**The cache can stand above its limit while the user reads**, by one book for
each program that holds a reader open. The module said the same of `keep`
already — one book of 500 megabytes is a correct cache of one book — and this is
that rule for an account of two windows. That is a decision of this session, and
the head of the module says it.

The measurement of the real program after the correction: the time of the PDF of
A moved at 01:40:37, 01:40:52, and 01:41:07 — **every 15 seconds** — the window B
took Alice in Wonderland from the server, the log holds no line of a removal, and
the 545 megabytes of A stayed.

Two tests hold the rule.
`tests/the_cache_keeps_the_book_of_a_second_window.rs` makes three books of three
times on the disk and it calls `the_removal`: a build with the correction removed
fails with "the removal took the book that the window A reads".
`tests/the_cache_of_the_ebooks.rs` held the rule of T-67 that this item changes —
"it goes although its time of use is the newest of the two" — and it holds the
new rule and the end of the mark now.

### T-154: a download of an episode while the server downloads that podcast, and the key `D` that lost its own bar

**The sweep of 2026-08-14: the key `D` of the program and the queue of the
downloads of the server (T-81), in one library at one time.** The road named
this condition in the session of T-148, and it was the last condition of the
road that a session named and no session measured.

**The condition needs work on both sides.** The server holds the 57 episodes of
the feed of `docs/TEST-SERVER.md` already, therefore
`POST /api/podcasts/:id/download-episodes` gives it nothing to do:
`DELETE /api/podcasts/:id/episode/:episode?hard=1` on ten of them makes the
work of the server exist again. **The body of that request is the bare array of
the episodes of the feed**, and an object of one field gives `400`. A download
of the loopback ends in less than one second (the trap 116), therefore
`docs/harness/slow_body.py` with 0.4 seconds for each block of 64 kilobytes
gives the download of the program about 60 seconds, and the two sides then meet.

**The named condition holds no fault of the data.**

| The measurement | The answer |
|---|---|
| The program downloads "Letter 13" of 10050287 bytes while the server downloads Letters 51 to 57 | **the file of the disk and the file of the server give one sum of MD5** |
| The same, with "Letter 15" of 10041092 bytes and Letters 40 to 49 | **one sum of MD5 again**, and every episode of the server holds its audio file |
| The list of the episodes of the program, while the server adds eight of them | 49 lines, and the header says `R: the server has newer data` |
| The key `D` on a line of that list | **the line of the program is the media of the program**: a new episode of the server stands at the end of `media.episodes`, therefore no line moves |
| The view of the downloads of the server (the key `d`) while the program downloads | `The downloads of the server [8 items]`, with `▼ Letter 42` |

**The two sides write two files.** The server writes the audio of a new episode
in the directory of the library, and the program writes
`downloads/<user>/<episode>/`: the one word that they share is the item of the
podcast, and a new episode of the server changes no line of it.

**A sweep of the keys inside that condition found a fault of one key**, and it
is the fault of this item.

#### The key `D` two times on one media

The user pressed `D` on "Letter 15", and they pressed `D` again 5 seconds later.

| The moment | The screen | The disk |
|---|---|---|
| the first press | `⬇ Letter 15  0.0 MB / 9.6 MB` | the `.part` file grows |
| **the second press** | **no bar at all**, and `A different program of this account downloads "Letter 15" now.` | the `.part` file grows |
| 58 seconds later | no bar, and no line of the screen names that work | `001 - Letter 15.mp3` of 10041092 bytes, and its sum is the sum of the server |

**The map of the progress is global and its key is the media** (`downloads()`),
because a refresh with the key `R` makes a new `App` and a map inside `App`
would lose a download that runs (T-131). The second task of the key therefore
writes on the row of the first task: `fetch_item` writes
`bytes_done = 0` and `Running` at its head, it then finds the lock of T-148 in
the hand of the first task, and it writes `Failed` on that row.
`render_downloads` draws a bar for each row of the state `Running` alone.

**The user reads nothing of a download that runs.** 58 seconds for an episode of
10 megabytes, and a book of 700 megabytes gives an hour of it. The bytes are
safe — the lock of T-148 keeps one writer, and the file of the disk is the file
of the server — **and the user cannot see them**.

**The words were wrong beside it.** The key `X` holds two sentences since T-150,
one of this window and one of the other window, and the key `D` held the
sentence of the other window alone: the program named a different program of the
account, and no such program existed.

**The correction is the rule of T-148 and of T-150: the map of the progress of
this process says which download this program runs.** `claim_the_download` reads
and writes that map under one lock, therefore two presses of one moment give one
claim:

- a row of the state `Running` gives `ThisProgramDownloadsIt`, and the key
  **changes no field of that row**. The program says
  `This program downloads "…" now.` and it asks the server nothing.
- every other condition gives `ThePlaceIsTaken`, and the row of the media starts
  at no byte.

The claim stands **before** the request of the item, therefore the bar comes
with the key and not with the first byte. Every road out of the download gives
the place back with `release_the_download`: a claim that stays `Running` for
ever would hold the key `D` of that media for ever.

The same measurement after the correction:

| The moment | The screen |
|---|---|
| the first press | `⬇ Letter 54  0.1 MB / 11.7 MB` |
| **the second press** | `This program downloads "Letter 54" now.`, and `⬇ Letter 54  0.9 MB / 11.7 MB` |
| 12 seconds later | `⬇ Letter 54  1.9 MB / 11.7 MB`, and the bar grows |
| the end | the file of 12278636 bytes, and its sum is the sum of the server |

`tests/the_key_d_of_a_download_that_runs.rs` holds the rule with no server and
no file. A build with the correction removed gives `left: ThePlaceIsTaken`
against `right: ThisProgramDownloadsIt`, and the row of the download that runs
then loses its bar and its bytes. **No unit test reaches a key handler of
`src/app.rs`**, therefore the last part of that test reads the source: the claim
must stand before `get_item`, and three roads out of the download must give the
place back.

**A note of the sandbox.** A hard delete of an episode and a new download of it
can leave the file `Letter 49 (<uuid>).mp3` beside `Letter 49.mp3`, and the
library then holds 58 episodes of a feed of 57. That is the work of
Audiobookshelf and not of the program. The session took the second row away, and
the library holds 57 episodes and 57 files again.

### T-155: the view of the accounts of two windows, and the mark of the start that stood on nobody

**The sweep of 2026-08-14: two windows of one account, and the view of the
accounts (T-124).** The road named that part of the program, and no session had
measured it. **The condition holds a fault, and that fault locks the user out of
their account.**

**The list of the accounts comes of `App::new` alone.** `self.the_accounts` holds
the rows of `users` of the moment of the start, and the view of the accounts
draws that list at every frame after it: a second program of the account adds a
row with the key `a` and removes a row with the key `l`, and no line of the first
window follows. That is the shape of T-142, of T-147, of T-148, and of T-153.

The measurement, with the sandbox and two sessions of tmux of one
`XDG_CONFIG_HOME`. The database held `toutuitest` (the account of the start) and
`toutuilimited`, and the window A held the view of the accounts open:

| The moment | The window A | The disk |
|---|---|---|
| Before | `▶ toutuitest`, `  toutuilimited` | both, and `toutuitest` holds the mark |
| The window B logs out of `toutuilimited` with `l` | **the two lines stay** | `toutuitest` alone |
| The user of A presses `c` on the line of `toutuilimited` | `Press c again to start with the account "toutuilimited"` | — |
| The second press of `c` | **the login screen**, and it asks for a server, a name, and a password | **`toutuitest`, and its `is_default_usr` is `0`** |
| A new window of the program after it | **the login screen** | the same |

**The cause is one write of two lines.** `make_this_account_the_default` takes
the mark from every account and it then gives that mark to the account of the
name; a name that no row holds gives **0 rows** of the second write, and the
transaction commits all the same. `select_default_usr` reads
`WHERE is_default_usr = 1 LIMIT 1`, therefore `Database::new` gives no account,
and `src/main.rs` draws the login screen. The log of that moment says
`[the accounts] the account toutuilimited starts the program`, and no such
account exists.

**The user loses the account, and the token of that account stands on the disk.**
No key of the program gives the mark back, in any view and after every start:
that is the shape of T-136. The rows of the queue, of the downloads, and of the
positions that wait hold the name of the account, therefore a login with the same
name finds them again — **and the user needs the password of that server for it.**

**The same list gave two faults beside it.** The key `l` on a line of an account
that a second program removed called `delete_user`, which removes 0 rows and
**says nothing** (T-79); and the view hid an account that a second program
**added** with the key `a`.

**The correction is the rule of T-142: the disk is the truth, and the program
reads it at the moment of the use.**

- `App::the_accounts_come_from_the_disk` reads `select_every_usr` again. The view
  calls it when it opens, and the keys `c` and `l` call it before they act.
- **A key acts on the name of its own line** (T-147), and not on the place of
  that line: the key takes the name of the list that the user reads, and the
  disk then says if that account stays.
- `logic::the_accounts::the_account_of_the_line` gives that answer, and
  `the_text_of_an_account_that_is_gone` gives the sentence
  `A different program of this account removed the account "…".` The sentence
  promises no key (T-118 and T-143): this program holds no key that gives such
  an account back.
- `make_this_account_the_default` gives the database back as it was when its
  name holds no row (`rollback`), and it gives `0` to its caller.
  `start_the_program_with_this_account` starts the program again no more on that
  road.
- **A database that met the fault already must find its account again**, and no
  key of the program can do that work: `an_account_takes_the_start_when_none_holds_it`
  gives the start to the first account of the table, and `src/main.rs` calls it
  before `Database::new`. The start is the place of that answer (T-136).

The same measurement after the correction: the key `c` says
`A different program of this account removed the account "toutuilimited".`, the
view of A holds one line, `toutuitest` keeps the mark, and the program stays. The
key `l` says the same sentence and it removes no row. A window that opens the
view after the key `a` of a second window reads **both** accounts, and the mark
stands on the account of the newest login. The program of the database of the
fault says
`[an_account_takes_the_start] no account held the start. The account toutuitest takes it.`
and it draws the Home view.

`tests/the_accounts_belong_to_the_disk.rs` holds the whole rule. It writes
`XDG_CONFIG_HOME`, therefore it stands alone in its binary (T-144), and its last
part reads `src/app.rs`: **no unit test reaches a key handler**. Without the
rollback of the write, the test gives `left: None` for the account of the start.

### T-156: the key `X` of a media that plays from the disk in the other window

**The second sweep of 2026-08-14: the key `X` of a media that plays in the other
window.** The prompt of the session named that part of the program as a part
that no measurement had reached. **It holds a fault, and that fault takes the
book of the user while they listen to it.**

**The condition needs the server away.** `play` takes the stream of the server
when the server answers, and it reads the files of the download only when no
address answers (T-152). `podman stop -t 0 abs-test` makes that condition, and a
book of eight hours gives the two keys a window of some minutes (the trap 90).

The measurement, with two sessions of tmux of one `XDG_CONFIG_HOME` and the book
of 115200330 bytes:

| The moment | The window A | The window B | The disk |
|---|---|---|---|
| A plays the book of the disk | `▶ 34:01 / 8:00:00` | — | the file and its row |
| **B presses `X` on that line** | the playback goes on | `Removed the local copy of "A Book Of Many Hours".` | **no file, and no row** |
| A presses `l` on the same media | **`The server does not answer, and the disk has no copy of this media.`** | — | the same |

**The playback of A went on**, because the engine holds the file open: the user
hears the book, and the copy of that book is gone. **The user reads no word of
it** until they stop that playback, and no key of the program gives the book
back while the server is away.

**The cause is the rule of T-150 with one condition too few.** The key asks two
questions — does this program download the media, does a program of the account
write its files — and **it asks nothing of a playback**. An offline playback
opens no session on the server (T-152), therefore `listening_session` holds no
row of it and a second program of the account can see nothing of that work: that
is the shape of T-142, of T-147, of T-148, and of T-153.

**The correction is the rule of the cache of the ebooks, for the audio**: "the
book that the user reads now never goes away" (T-65 and T-153). The loop of the
offline playback keeps the place of the user in `pending_progress` **at each
second** since T-152, and that moment is the heartbeat that two programs of one
account share. `a_program_keeps_the_place_of_this_media` reads it, and a media
whose place moved inside `THE_LIMIT_OF_THE_HEARTBEAT` (30 seconds) belongs to a
playback that runs. **It needs no new column, no call of the system, and no
dependency**, and it is the rule of T-140 and of the lock of T-148.

**A mark of a playback is not for ever** (T-153): a window that goes away writes
no more places, and the key takes the disk 30 seconds later.

**The sentence names no program**, and that is a decision: no column of
`pending_progress` holds a process, therefore a sentence of "a different
program" would name a program that this program does not know — the fault of
T-154 — and this program cannot say which window plays that media (T-91). It
promises no key (T-118 and T-143).

The same measurement after the correction:

| The measurement | The answer |
|---|---|
| The key `X` of B while A plays that book of the disk | `A program of this account plays "A Book Of Many Hours" from the disk now.`, and **the 115200330 bytes stay** |
| The key `X` of B, 35 seconds after the window A went away | `Removed the local copy of "A Book Of Many Hours".`, and the directory goes away |

`tests/the_key_x_keeps_the_media_that_plays.rs` holds the rule of the database
and the rule of the key, and its last part reads `src/app.rs`: **no unit test
reaches a key handler.** Without the condition of the playback, the two tests of
this item fail.

### T-157: two tests of one binary shared the boxes of the authors, and the gate of CI failed

**The gate of CI of the session of T-156 failed one run of six**, and the gate
of the machine passed every time: `cargo nextest run` gives each test a process
of its own, and `cargo test` gives the tests of one binary a **thread** of its
own. That is the shape of T-144, and the state here is a box of the process and
not a database.

`logic::authors` holds two boxes — the kind of the list and its answer — and the
head of each of its two tests said the rule already: "the state belongs to the
process, therefore the parts of this test must stay in one function". **The two
functions broke that rule between them**: `the_state_goes_from_the_task_to_the_screen`
called `forget()` while `a_new_list_forgets_the_answer_of_the_list_before_it`
counted the answer of the list, and that test then read `authors().len() == 0`.

```
test logic::authors::tests::a_new_list_forgets_the_answer_of_the_list_before_it ... FAILED
test result: FAILED. 860 passed; 1 failed
```

**The two tests are one test now**, and the measurement of the fault is the
measurement of the correction: `cargo test --lib` failed one run of six before
it, and it passed eight runs of eight after it.

### T-158: the playback of an account that a second window logged out waited for ever

**The condition of this session, and no session had measured it.** One account
holds more than one program (T-140), and the key `l` of the view of the accounts
removes the row of an account (T-124). **A program of that account stands open
while its row goes away**: that is the shape of T-142, of T-147, of T-148, of
T-153, and of T-155 — a state of one process that a second program cannot see —
and the road named the write of a state that names a row of the database as a
sweep of its own.

**The window A then plays nothing at all, and no key of it works.**

| The moment | The window A |
|---|---|
| Before | the Home view of `toutuitest`, and the account holds its row |
| The window B logs out with `l` | the screen of A does not change, and `users` holds **0 rows** |
| The key `l` of A on a media | `Syncing your last listening session. Please wait...` |
| **78 seconds later** | **the same message, and no media plays** |
| 40 presses of `l` after it | **the key `j` moves no line**, and **the key `Q` does not stop the program** |

**The two reads of the wait gave a text of a fault for a row that does not
exist.** `get_has_played_before` and `get_is_loop_break` gave `No db found`, and
`wait_prev_session_finished` waits while `is_loop_break` is not `1`:

```
[wait_prev_session_finished][has_played_before] No db found
[wait_prev_session_finished][is_loop_break] No db found
```

**No loop of a playback of a row that does not exist can ever write that
value**, therefore the wait had no end. That is the fault of T-35 again, and it
comes of a different cause: the caller could not tell a value of the database
from a row that no account holds.

**The program froze after that.** The key `l` gives its work to `tokio::spawn`,
and the wait holds `std::thread::sleep`: each press takes one worker of the
runtime for ever. The machine of the measurement holds 32 processors, and 40
presses took every worker. The program held 33 threads before and after, every
one of them in the state `S`, and **the key `Q` left it standing**: the user of
that terminal must take the program away with a signal.

**The correction says the two rules of the fork.** The disk is the truth, and a
read of the disk must say when it found no row: the two functions give
`Option<String>` now, and `None` says that the account stands in no row. **A row
that no account holds means that no loop stands before this playback**,
therefore the playback starts at once, and the program says
`A different program of this account removed the account "…".` — the sentence of
T-155, and it promises no key (T-118 and T-143).

**No wait of a playback stands longer than 30 seconds.** A program that dies
inside the loop of its playback writes `is_loop_break` never, therefore a wait
with no limit can come back with a cause that no session has met. 30 seconds is
the time of this fork for a program that stood still: the row of a session
(T-140), the lock of a download (T-148), and the book of a reader (T-153).

| The measurement | Before | After |
|---|---|---|
| The key `l` of A, after the log out of B | **no media, and the message for ever** | the podcast plays at the second **0.07** |
| The log of that key | `is_loop_break No db found` | `the account toutuitest stands in no row of the disk` |
| The playback 18 seconds later | — | 21:51 of 23:39 (the device `null` of the trap 72) |
| The wait of a playback whose loop wrote no end | for ever | the limit of 30 seconds |

`tests/a_playback_waits_for_no_account_that_is_gone.rs` holds the three rules.
**A test must not call the wait itself**: the wait of the old code never comes
back, and a test of that shape holds the gate for ever. The test gives the wait
a thread of its own, and it reads the end of that thread with a limit of time:
the two tests then **fail** with the correction removed, and they do not hang.

### T-159: a log out that left a program of that account, and that program named nobody

**The second half of the condition of T-158**, and it is the same shape: the
window B logs out of the account, and the window A of that account stands open.
The key `l` of the view of the accounts removes the row of `users` (T-124), and
**every key that refreshes the screen makes a new application** (T-131). That
application read the disk of an account that the disk does not hold:

| The measurement | The window A after the log out of B |
|---|---|
| The header | **`👋 Connected as `** — the program named nobody |
| The log of the token | `Failed to decrypt the token`, and the program went on |
| The requests of the server | **they all answered**: the client of the start holds the token, and `App::new` takes that client (T-131) |
| The key `S` on the line `Books` | `The library has been updated. Please refresh the app to apply the changes.`, and the header said `📖 Podcasts` after it |
| The rows of `users` | **0**, before and after that key |

**A logout that leaves a program of that account is no logout.** The user of the
window B took the account of the program away, and the window A held its token
and it wrote and read the disk of a name of no row: the write of the library
changed 0 rows and it said that it kept the choice of the user (T-79 and
T-118), and the writes of the sequence, of the speed, and of the key bindings
went away with no word at all.

**The correction is the rule of T-142 and of T-155: the disk is the truth, and
the program reads it at the moment of the use.** A key of the user is that
moment: the loop of `src/main.rs` reads `select_every_usr` after every key, and
a program whose account stands in no row of that list starts again.
**`the_account_of_this_program_is_gone` writes the request of T-139**, therefore
the sequence of that restart is the sequence of the key `a` of the accounts: the
engine stops, the place of the playback reaches the server, and `exec` gives the
new program the terminal of this one. **The login screen of that program says
which account went away**, with the sentence of T-155 that promises no key.

**A read of the database for each key is the cost, and that is a decision.** A
key is an event of a person, and a view of this program reads `get_download` of
the database at **every frame** already (T-148). A read of a state that a second
program can change belongs to the moment of the use, and no other moment can
hold it: the account can go away at any second of the program.

| The measurement | Before | After |
|---|---|---|
| The key `R` after the log out of B | a program of no name, and it holds the token | **the login screen**, with the address of the server and the reason |
| **The key `j` at the minute 14:44 of an episode** | the key moves a line, and the place stays in this program | `Item 9fa45bd1… closed at 884s`, and `GET /api/me` of `curl` holds **884** |
| The key `S` on a line of a library | `The library has been updated.` for 0 rows | the sentence of the account that a second program removed |
| The rows of the queue and of the downloads | they hold the name of the account | the same: a login of that name finds them again (T-123) |

`tests/the_account_of_this_program_belongs_to_the_disk.rs` holds the three
rules. **No unit test reaches the loop of `src/main.rs`**, therefore the rule of
that loop stands as a rule of the source, as the rule of T-131 does: the read of
the accounts stands after the key of the user and before the block that starts
the program again.

## The upgrade of the dependencies, 2026-08-10

Every crate went to the newest version that the fork can take. The gate passed
after each step, and each step is its own commit.

| Crate | Before | After | Note |
|---|---|---|---|
| every crate that semver allows | — | newest | `cargo update`, no range changed |
| `rusqlite` | 0.33 | 0.40 | seven major versions, no change of the code |
| `reqwest` | 0.11 | 0.12 | brings `rustls` 0.21 to 0.23 |
| `magic-crypt` | 4.0.1 | 5.0.1 | the form of the cipher did not change |
| `sha2` | 0.10 | 0.11 | one version in the tree now, and not two |
| `dotenv` | 0.15 | `dotenvy` 0.15.7 | `dotenv` has no maintainer since 2019 |
| `serde_derive` | 1.0 | removed | `serde` with `derive` gives that macro |

Three measurements support the upgrades that touch a form of data or a
protocol:

1. **The token.** magic-crypt 4.0.1 wrote a cipher for a known token and a
   known key. magic-crypt 5.0.1 read that text and gave the token back.
   Therefore no user must give their password again.
   `src/utils/encrypt_token.rs` holds that text and guards the rule.
2. **The sum.** sha2 0.11 gives the same sum as `sha256sum` of coreutils for
   the archive of v0.5.0, of 5960719 bytes.
3. **TLS.** `--update` reached api.github.com through rustls 0.23 and read the
   last release. The tests use a mock server with no TLS, therefore only a real
   request can show this.

### T-32: a command that forces the sync — complete, `a4904e0`

Issue #37 of the original repository asks for a way to force the sync. The
backlog did not hold this request before. The other four open issues of that
repository are complete: #36 is T-8, #35 is T-3, T-6 and T-7, #33 is T-2, and
#32 is T-9.

**The key is `F`, and not `S`.** The design named `S`, and a measurement on
2026-08-10 shows that `S` is not free: `src/app.rs` gives `S` to the settings,
and every footer of the application says "S: Settings". `F` reads as "force the
sync", and no view uses it.

**Why the key writes a flag only.** The endpoint `POST /api/session/:id/sync`
takes the time that the user listened since the last sync. A second sender
would give that time to the server a second time, and the server would then
hold too much listened time. Therefore the key writes a flag, and the loop of
the playback does the work at its next second. That loop holds the position and
the listened time.

The flag carries the identity of the playback. A loop takes the flag only when
the identity is its own, because two playbacks can run at the same time. That
rule comes from `9bacac`.

**The proof, 2026-08-10.**

1. `tests/force_sync_against_the_sandbox.rs` opens a real session, gives the
   engine the position 37 seconds, and asks for the sync. Two seconds later
   `GET /api/me/progress/:id` holds 37 seconds, and `GET /api/sessions/open`
   still holds the session. The test needs the sandbox, thus it carries
   `#[ignore]`.
2. A debug build in a pseudo terminal played the book of thirty minutes. The
   server held 0 seconds while the player showed 3:17. The key `F` gave the
   message "Sync: the server has the position 4m." on the screen, and the
   server then held 268 seconds. The session `452859eb` was open before the key
   and after it.

**A trap of the server.** `GET /api/me/progress/:id` gives `currentTime` as a
text, and not as a number. A test that reads a number only finds nothing.

### T-33: ratatui 0.30 — complete, `8f5c938`

**The work that landed, 2026-08-10.** The project holds `ratatui` 0.30.2,
`crossterm` 0.29.0, and `tui-input` 0.15.4. `tui-textarea` is gone. The change
touched two files of the application and added one:

- `src/logic/auth/auth_input.rs` — the three fields of the login screen.
- `src/logic/search/search_active.rs` — the bar of the search.
- `src/ui/text_field.rs` — new. It makes the three values that a `Paragraph`
  needs: the text to show, the number of hidden columns at the left, and the
  column of the cursor.

**The feature of `tui-input`.** The crate has two ways to read a key. The
default feature `ratatui-crossterm` reads the events through the copy of
crossterm that ratatui holds. This project names crossterm itself, therefore it
takes `default-features = false` and the feature `crossterm`. `cargo tree -i
crossterm` then shows one crossterm 0.29.0 for the application, for ratatui, and
for `tui-input`.

**No new C.** `cargo tree -i cc` finds `libsqlite3-sys` and `ring` only, and
`cargo tree -i openssl-sys` finds nothing.

**The proof.** A test of the screen cannot reach the login screen or the search
bar. Therefore the work has two proofs.

1. **The pure part.** `src/ui/text_field.rs` holds 12 tests. They examine the
   column of the cursor, the horizontal scroll of a text that is longer than the
   field, a character that takes two columns, and the mask of the password. One
   test walks 200 positions of the cursor and 39 widths, and it shows that the
   cursor always stays inside the field.
2. **The real program.** A debug build ran in a pseudo terminal against the
   sandbox, with `XDG_CONFIG_HOME` at a new directory. The login screen showed
   the text "http:// or https:// required" while the first field was empty. The
   arrow keys moved the cursor. The password field showed 13 mask characters for
   a password of 13 characters, and two Backspace keys left 11. That wrong
   password gave "ERROR: Login failed", and the screen of the login came again.
   The correct password wrote the user in the database with a token of 280
   characters, and the library came. In the search bar, the text "the" with the
   cursor two columns to the left took an "X" in the middle and gave "tXhe", and
   Backspace gave "the" again. Enter then found the three books of "The Test
   Chronicles". `tests/login_against_the_sandbox.rs` passes with `--ignored`.

**A trap of the test, measured 2026-08-10.** `ALSA_CONFIG_PATH=/dev/null` is
correct for `cargo test`, because no test opens a sound device. The real program
stops for ever with that value: it reaches "The pool has 1 address(es)" and it
draws nothing. Give the program a real file instead:

```
</usr/share/alsa/alsa.conf>
pcm.!default { type null }
ctl.!default { type null }
```

The text below is the examination of 2026-08-10 before this work.

**`tui-textarea` 0.7.0** of 2024-10-22 is still the newest version of
that crate, and it still asks for `ratatui ^0.29`. Therefore the blocker did not
go away by itself.

**The answer: `tui-input` 0.15.4.** That crate came on 2026-08-10, it has
1.78 million downloads, and it asks for `ratatui ^0.30.2` and `crossterm ^0.29`.
It needs `unicode-segmentation` and `unicode-width` only, and both are pure Rust.
`ratatui` and `crossterm` are features of that crate, therefore the project names
the two features that it uses.

`tui-input` holds one line of text. Every place of this project holds one line:
the address of the server, the name of the user, the password, and the words of
the search. Therefore the crate is enough.

**The size of the work, measured on 2026-08-10.** A test removed
`tui-textarea`, and it added `ratatui 0.30`, `crossterm 0.29`, and `tui-input
0.15`. The compiler then gave **two errors, and both errors are the line
`use tui_textarea` of the two files that hold a text input**:

- `src/logic/auth/auth_input.rs`, 208 lines, three fields.
- `src/logic/search/search_active.rs`, 73 lines, one field.

Every other file of the project already agrees with ratatui 0.30: `tui.rs` with
its 1344 lines, `player_tui.rs`, `app.rs`, and `login_tui.rs` all compile with no
change at all. The note of "nine errors of the compiler" came from a build that
held two versions of ratatui at the same time. One version gives two errors.

Therefore T-33 is a small work: write the two places again with `tui-input`.
`tui-input` gives the state only, and the caller draws the text. The functions
`value`, `visual_cursor`, and `visual_scroll` give what a `Paragraph` needs.

**What T-33 opens.** `ratatui-image` 11.0.6 asks for `ratatui ^0.30.1`, therefore
T-23 can use the newest version of that crate after this work. T-23 does not wait
for it: `ratatui-image` 9.0.0 asks for `ratatui ^0.29`.

The text below is the examination of 2026-08-10 before this note.

`ratatui` 0.30.2 and `crossterm` 0.29 are available, and the fork stays on
`ratatui` 0.29 and `crossterm` 0.28.

`tui-textarea` 0.7.0 is the newest version of that crate, and it asks for
`ratatui ^0.29.0` and `crossterm ^0.28`. A build with `ratatui` 0.30 therefore
holds two versions of `ratatui`, and the types of the two do not agree. A
measurement on 2026-08-10 gives nine errors of the compiler, and every one of
them is at the boundary of `tui-textarea`: `expected
ratatui::widgets::block::Block, found ratatui::widgets::Block`, `the trait
bound &TextArea: Widget is not satisfied`, and `the trait bound
tui_textarea::Input: From<Event> is not satisfied`.

**The work.** Wait for a `tui-textarea` that takes `ratatui` 0.30. The other
answer is to write the field of text in this project, and that work is larger
than the gain.

### T-15: the authentication does not fail at the first attempt

`4b3045` says that a login with correct credentials fails, and that the second
attempt works.

**The mechanism that explains the report.** The program read the database
before the login wrote the user. The old code started the login with
`tokio::spawn` and did not wait for it, and it set `should_exit` at once.
`main.rs` then read the database, found no user, and showed the screen of the
login again. The user saw a login that failed. The row of the first attempt was
in the database by then, therefore the second attempt found it and worked.

**The state now.** `6796d91` closed that race: `auth_input.rs` runs the login
on its own thread and waits for that thread with `join`. `auth_process` writes
the user with `db_insert_usr` before it gives its answer, and `rusqlite` writes
without a task in the background.

**The measurement of 2026-08-10.** A test made an empty database, and it then
ran the real login against the sandbox server of Audiobookshelf 2.36.0. It read
the database with no wait at all. The list of the users held the name, the
address of the server, the encrypted token, and the library that the program
selected. Therefore the race does not exist.

`tests/login_against_the_sandbox.rs` holds that test. Continuous integration
does not run it, because it needs a server. The head of that file gives the
command.

`6796d91` said that it did not close T-15, because a test of that day did not
reproduce the fault with the old code either. The report stays without a
reproduction. The one mechanism that explains it is closed, a test confirms
the rule that matters, and therefore T-15 closes.

The comment of `main.rs` said that the program "will work at the second
attempt". That comment is wrong now, and the file gives the true reason for
the wait of one second: it stops a fast loop if the screen of the login comes
back at once.

## The three sync reports, 2026-08-10

`known_bugs.md` held `9bacac`, `86384e`, and `dd9a649` under NOT YET EXAMINED.
All three describe one condition: the user plays the book X, the user plays the
book Y quickly, and then the progress of X is wrong or the session of X stays
open. The wording of the reports names VLC, and the application starts no other
program now. Therefore the reports needed a new test, and not a reading.

**The mechanism that explains the reports.** The state of the engine is one
value for the whole application. `PlayerHandle::state` gives the position, the
status, and the identity of the media that the engine plays. The key that starts
a media gives its work to a new task with `tokio::spawn`, therefore two
playbacks can run at the same time.

The loop that follows a playback read that state always. Two results follow:

1. The loop of X reads the position of Y, and it reports that position for the
   session of X. That is `9bacac`. A book Y that starts holds a position that
   is almost 0, therefore the same mechanism gives `86384e`.
2. The loop of X reads the status `Playing`, because the engine plays Y. The
   loop therefore never closes the session of X. That is `dd9a649`.

`wait_prev_session_finished` was the only guard. It waits while `is_loop_break`
is not `1`. That value is one value for the whole user, therefore it cannot
serialize two playbacks: the waiter that wakes first gives the value `0` again,
and the second waiter then has no signal at all.

**The measurement of 2026-08-10.** A test ran `follow_playback` in a real
process against a real server. The engine reported the book X at 100 seconds,
and then the book Y at 4 seconds. The loop of X sent
`{"currentTime":"4","timeListened":"0"}` to the session of X, and the loop did
not stop. A second measurement against Audiobookshelf 2.36.0 opened a real
session of X: with the old behaviour `GET /api/sessions/open` held the session
`ca2079ec` of X after the engine changed to Y.

**The correction.** Every playback has an identity now. `next_playback_id`
gives it, the engine writes it into the state, and a loop reads the state only
while that identity is its own. A loop that loses the engine closes its own
session and reports the last position that it measured itself. A loop whose
playback the engine does not start in 30 seconds also closes its session,
because a session that stays open is `dd9a649`.

`tests/playback_ownership.rs` holds the rule with no server and no sound card.
`tests/sync_against_the_sandbox.rs` holds the test against a real server, and
it carries `#[ignore]`.

### T-34: a colour of the configuration file stops the program

The examination of the three reports found this fault. Every place that read a
colour of `config.toml` took the three components with an index. A list that is
too short then stops the program, and `load_config` gives an error for a file
that a person cannot parse. The old code then read an empty list.

A measurement on 2026-08-10 ran `pop_message` in a process whose configuration
file was absent. A thread stopped with "index out of bounds: the len is 0 but
the index is 0".

`rgb_parts` in `src/config.rs` gives the three components now. It repeats the
last value for a component that the list does not give, and it gives a middle
grey for a list with no value. All eleven places use it.

### T-35: a playback that does not start stops every later playback

The examination of the three reports also found this fault.
`wait_prev_session_finished` waits while `is_loop_break` is not `1`, and it
gives that value `0` before a playback begins. The old code gave the value `1`
in the two loops that follow a playback only.

Five places come back without a loop: a server that gives an error, an item
that the server does not give, an item with no audio file, and two conditions
of the offline mode. The next playback then waited for ever, and the screen
held the message "Syncing your last listening session. Please wait...".

A measurement on 2026-08-10 ran `play` in a real process against a server that
answered 500 for `POST /api/items/:id/play`. The value stayed `0`.

`play` is a thin function now. It calls `play_media`, and it always gives the
value `1` after that call. One place owns the value.
`tests/playback_wait_flag.rs` holds the rule.
