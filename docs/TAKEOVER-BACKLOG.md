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
| T-28 | A tag with a capital letter gave "update available" for ever | `4221f2a` |
| T-30 | An answer with no length has a limit of size now | `ffe6a13` |
| T-29 | The update tests the proof of the origin of an archive | `f123f8b` |
| T-27 | Continuous integration builds the flake of Nix | `7f55e43` |
| T-14 | The program does not lose the configuration (examined) | this document |
| T-31 | The fork gives no bundle for macOS | this document |

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
| T-17 | — | Play an Opus file | later |
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

The rule of the project says that no dependency needs a C toolchain. A
measurement on 2026-08-10 shows that two dependencies do not obey the rule:

- `libsqlite3-sys`, from `rusqlite` with the feature `bundled`.
- `ring`, from `rustls` 0.21 through `reqwest` 0.11.

`backtrace` names `cc` as a build dependency, but it compiles no C on this
target. The audio crates of sub-project 2 compile no C.

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

### T-17: play an Opus file

Audiobookshelf accepts 19 audio formats. The audio engine plays 16 of them.
Opus is the gap that has an answer.

A measurement on 2026-08-10 shows that a pure Rust decoder gives audio that
agrees with libopus. The highest value is the same. Therefore no C library is
necessary.

The work needs a `rodio::Source` of this project, because `rodio::Decoder`
uses a fixed codec registry. WMA and AWB stay outside, because no pure Rust
reader or decoder is available.

Issue 17 holds the details.

### T-22: show the series of a library

The key `s` shows the series. The key `l` on a series shows its books, with
the number of each book first. The books come in the sequence of the series,
because a sort of the text gives `#10` before `#2`.

The endpoint `GET /api/libraries/:id/series` has an important difference from
the endpoint of the items: `limit=0` gives an empty list, and not every
series. Therefore the application always asks for a page of 500.

The work does not group the books of a series into one line in the Library
view. That part of the issue stays open.

### T-23, T-24: the user interface and full coverage

T-23 adds the cover art. A measurement on 2026-08-10
shows that `ratatui-image` 11.0.6 needs no C library, if the project does not
use its default features. That crate finds the protocol of the terminal
itself: the Kitty protocol, Sixel for `foot`, iTerm2, or blocks of Unicode.

T-24 holds the comparison with Audiobookshelf, and it names the functions that
the application does not have yet.

## Priority 5: small faults

| Id | Upstream | Title |
|---|---|---|
| T-13 | — | The description shows the HTML tags |
| T-14 | — | The application loses the configuration after an update (`255b86`). Examined on 2026-08-10: this fault does not occur in the fork. |
| T-15 | — | The authentication fails at the first attempt (`4b3045`) |
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
