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
| T-7 | #35 | The application gets all items in one request | 3 |
| T-8 | #36 | A change of the speed needs a new start of the playback | 2 |
| T-17 | — | Play an Opus file | complete, `c342f50` |
| T-18 | — | Play a WMA file and an AWB file (the last two of 19) | later |
| T-20 | — | Remove the two dependencies that compile C | later |

### T-7: the application gets all items in one request

`src/api/libraries/get_all_books.rs:98` uses `?limit=0`. That value gets every
item. A library with 10000 books makes a large answer. This uses time, memory,
and the resources of the server.

Measurement on 2026-08-09: a library of 2056 books gives 3.7 megabytes in 0.48
seconds. This is acceptable, but a larger library is not.

### T-8: a change of the speed needs a new start of the playback

The user changes the speed. The indicator changes, but the sound does not
change. The user must press `L` to start the playback again.

VLC is the cause. The audio engine in sub-project 2 can change the speed
during playback.

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

T-24 holds the comparison with Audiobookshelf, and it names the functions that
the application does not have yet.

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

### T-32: a command that forces the sync

Issue #37 of the original repository asks for a way to force the sync. The
backlog did not hold this request before. The other four open issues of that
repository are complete: #36 is T-8, #35 is T-3, T-6 and T-7, #33 is T-2, and
#32 is T-9.

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
