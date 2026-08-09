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

T-2 is not complete. The length and the percent are correct now. VLC still
plays only the first file. Sub-project 2 corrects this.

## Priority 1: the application gives incorrect results

| Id | Upstream | Title | Sub-project |
|---|---|---|---|
| T-1 | — | The offline mode does not play a book | 1b |
| T-2 | #33 | A book that has many audio files does not play | 1b |
| T-3 | #35 | The application sends `/progress` and `/sync` at the same time | 1c |
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

Examine this fault again after the correction of T-3.

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

## Priority 4: new functions

| Id | Upstream | Title | Sub-project |
|---|---|---|---|
| T-9 | #32 | Show the playlists and the collections | 5 |
| T-10 | — | Read an EPUB book in the application | 5 |
| T-11 | — | Download a podcast episode | 5 |
| T-12 | — | Add a Nix flake | 0 |

### T-9: show the playlists and the collections

One user asks for this function. The original author agreed that collections
are also useful.

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

## Priority 5: small faults

| Id | Upstream | Title |
|---|---|---|
| T-13 | — | The description shows the HTML tags |
| T-14 | — | The application loses the configuration after an update (`255b86`) |
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
