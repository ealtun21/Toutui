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
