# Design: the audio engine in the process, and the removal of VLC

Date: 2026-08-10
Status: Approved
Sub-project: 2 of 5
Items: T-2, T-5, T-6, T-8

## 1. Purpose

The application starts VLC as a separate program. It controls VLC through a
TCP remote control interface. This design removes VLC. The application decodes
the audio itself.

The removal corrects four faults at the same time:

| Item | Fault | Cause |
|---|---|---|
| T-5 | The token is visible in `ps aux` | VLC gets the token in a command line argument |
| T-6 | The address of the file holds the token | The stream address holds `?token=` |
| T-2 | A book with many audio files plays the first file only | The application gives one address to VLC, and there is no playlist |
| T-8 | A change of the speed needs a new start | The remote control interface does not change the speed of the current file |

The removal also stops a group of faults in `known_bugs.md`. These faults come
from the interface to VLC, and not from the logic of the application. The
faults are `a49eza`, `2eb9e3`, and the faults that `pkill_vlc` hides.

## 2. Scope

### 2.1 In scope

- An audio engine that uses `rodio` and `symphonia`.
- A byte source that reads a local file.
- A byte source that reads from the server with HTTP range requests.
- A queue that plays all the audio files of a book in the correct sequence.
- Position, seek, chapter movement, speed, and volume.
- One progress loop and one sync loop for books and for podcasts.
- The removal of `src/player/vlc/` and of the two dependencies that only that
  code uses.

### 2.2 Out of scope

- Pagination of the library (T-7, sub-project 3).
- The playlists and the collections view (T-9).
- The EPUB reader (T-10).
- The download of a podcast episode (T-11).
- A change to the download logic of sub-project 1b. The engine reads the files
  that the downloader writes.

## 3. Measured facts

A measurement of the test server on 2026-08-10 gives these values. The
measurement reads `GET /api/items/:id` for all 2056 books.

| Property | Value |
|---|---|
| Codecs in the library | `aac` in 3408 files, `mp3` in 6660 files |
| Other codecs | None |
| File name extensions | `.m4b` 3326, `.mp3` 6662, `.m4a` 78, `.mp4` 2 |
| Books that have more than one audio file | 297 of 2056 |
| The largest number of audio files in one book | 209 |
| Books that have chapters | 1938 of 2056 |

Two results are important:

1. The test library has `mp3` and `aac` only. A different user has other
   formats. Therefore the engine uses the feature `symphonia-all`, and not the
   smaller feature set. Section 3.1 gives the measurement.
2. The backlog says that the largest book has 79 audio files. That value is not
   correct. The largest book has 209 audio files.

### 3.1 The formats that the engine plays

A measurement on 2026-08-10 decodes one real file of each format. The feature
`symphonia-all` gives every codec and every container of `symphonia`. All of
them are pure Rust, thus the rule for the C toolchain stays correct.

| Format | Result | Note |
|---|---|---|
| MP3 | Plays | |
| M4B, M4A, MP4 (AAC) | Plays | The usual format of an audiobook |
| AAC (ADTS) | Plays | |
| FLAC | Plays | |
| WAV (PCM) | Plays | |
| OGG, OGA (Vorbis) | Plays | |
| ALAC in MP4 | Plays | `symphonia-all` gives this |
| AIFF | Plays | `symphonia-all` gives this |
| CAF (PCM) | Plays | `symphonia-all` gives this |
| MKA, WebM (Matroska) | Plays | `symphonia-all` gives this |
| WAV (ADPCM) | Plays | `symphonia-all` gives this |
| MP1, MP2 | Plays | `symphonia-all` gives this |
| **Opus** | **Does not play** | `symphonia` has no decoder |
| **WMA** | **Does not play** | `symphonia` has no reader for ASF |
| ALAC in CAF | Does not play | ALAC in MP4 plays |

The smaller feature set gave two faults that this measurement found:

1. A Matroska file gave no error and no audio. The container was correct, and
   the codec was absent. A test now counts the samples, thus a silent fault of
   this type fails the test.
2. ALAC, AIFF, ADPCM, and CAF did not play.

### 3.2 The formats of Audiobookshelf

`server/utils/constants.js` of Audiobookshelf gives the audio formats that the
server accepts. The engine plays 16 of the 19 formats:

| Format of Audiobookshelf | The engine plays it |
|---|---|
| MP3, M4B, M4A, MP4, OGG, OGA, AAC, FLAC | Yes |
| AIFF, AIF, WEBM, WEBMA, MKA, CAF | Yes |
| MPEG, MPG | Yes. These files hold MP2 audio. |
| **OPUS** | **No.** Issue 17. Section 3.3 shows that this gap has an answer. |
| **WMA** | **No.** Issue 18. `wmvkit` gives the ASF container, but no pure Rust WMA decoder is available. |
| **AWB** | **No.** Issue 18. This is AMR-WB, and no pure Rust decoder is available. |

The goal is 19 of 19. Issue 17 and issue 18 hold the three formats that stay.
Opus is the only one of the three that a person can do now.

### 3.3 The gap: Opus

Opus is the only gap that has an answer. Audiobookshelf accepts Opus files,
therefore users have them.

The tests show that the WebM container, the Matroska container, and the CAF
container all play with a different codec. Therefore the fault is the codec,
and it is not the container. `symphonia` reads the Opus stream correctly:
`symphonia-format-ogg` has a mapper for Opus.

A measurement on 2026-08-10 shows that a pure Rust decoder gives the correct
audio:

| Measurement | ffmpeg with libopus | The crate `opus-decoder` 0.1.1 |
|---|---|---|
| Samples | 96000 | 96960 |
| Highest value | 4193 | 4193 |
| Mean square value | 2894.7 | 2880.3 |
| Errors | — | 0 in 101 packets |

The highest value is the same. The difference of 960 samples is the value
`pre-skip` of the `OpusHead` header. ffmpeg removes those samples, and the
measurement program did not. This is a known correction, and it is not a
fault.

Therefore Opus is possible with no C library. Two things are necessary:

1. `rodio::Decoder` uses the fixed registry
   `symphonia::default::get_codecs()`. A codec cannot go into that registry.
   Therefore the project needs its own `rodio::Source`. That source reads the
   packets with `symphonia`, and it decodes them with the Opus crate.
2. The source must remove the `pre-skip` samples, and it must obey `try_seek`.

The new source is only for Opus. The engine uses `rodio::Decoder` for the
other 16 formats. Therefore this work cannot make a fault in a format that
operates now.

The crate is young. Version 0.1.1 has one author. The measurement above is
good evidence, but a wider test is necessary before the fork gives this
function to users.

This work is not in this sub-project. Issue 17 holds it.

`tests/formats.rs` decodes one small file of each format. The files are in
`tests/fixtures/audio`, and they use 14 kilobytes together. A measurement is
true only on the day of the measurement. These tests fail if a later version
of a dependency stops a format.

118 books have no chapters. Therefore the chapter functions must operate
correctly when the chapter list is empty.

A measurement of the crate `rodio` version 0.22.2 on the same day gives these
values:

| Property | Value |
|---|---|
| The crate opens an audio device | Yes |
| `libasound.so.2` in the dynamic dependencies | Yes |
| `alsa-sys` compiles C code | No. The build script only calls `pkg-config`. |
| The version of `symphonia` | 0.5.5 |

## 4. Architecture

One worker thread owns the audio. All other code sends a command to that
thread. The user interface reads a state snapshot.

```
user interface (ratatui, one read for each frame)
   |  reads
   v
Arc<RwLock<PlaybackState>>  <---- the worker publishes the state
                                       ^
app.rs, handle_input  --command-->  PlayerHandle  -->  worker thread
                        (mpsc)                         owns rodio::Player
                                                            |  pulls
                                                            v
                                              Decoder<Box<dyn MediaRead>>
                                                            |
                                    +-----------------------+------------------+
                                    |                                          |
                            LocalFile(std::fs::File)                       HttpFile
                                                              (a prefetch thread and
                                                               HTTP range requests)
```

`MediaRead` is `Read + Seek + Send + Sync + 'static`. The type
`rodio::Decoder<R>` needs all five of these bounds. The two byte sources obey
this trait. Therefore the engine has one decode path only. This is the reason
that `handle_l_book_offline.rs` goes away.

The engine builds the decoder with `Decoder::builder()`. It gives the file name
extension with `with_hint()`, and it sets `with_gapless(true)`. The hint stops
the format examination. That examination costs range requests on a file that
comes from the server.

### 4.1 The most important constraint

`rodio` decodes the audio on the callback thread of the sound card. A read
operation on that thread must not wait for the network. A wait makes a gap in
the sound.

Therefore `HttpFile` does no network operation in its `Read` function. A
prefetch thread keeps a buffer in the memory full. The buffer holds
approximately 8 megabytes. The `Read` function copies bytes from that buffer.

## 5. Components

| Path | Responsibility |
|---|---|
| `src/player/engine/mod.rs` | `PlayerHandle`, `PlayerCommand`, `PlaybackState` |
| `src/player/engine/http_file.rs` | The range reader, the prefetch thread, and the retry |
| `src/player/engine/source.rs` | The `MediaRead` trait, and the selection of the source |
| `src/player/engine/track.rs` | The track list, the position calculation, and the chapters |
| `src/player/engine/worker.rs` | The thread that owns `rodio::Player` |
| `src/logic/playback/mod.rs` | The one progress loop and the one sync loop |

### 5.1 The commands

```rust
pub enum PlayerCommand {
    Start(Box<PlaybackRequest>),
    Pause,
    Resume,
    SeekTo(f64),        // the position in the book, in seconds
    SeekBy(f64),        // a relative movement, in seconds
    NextChapter,
    PreviousChapter,
    SetSpeed(f32),
    SetVolume(f32),
    Stop,
}
```

### 5.2 The state

```rust
pub struct PlaybackState {
    pub item_id:       String,
    pub title:         String,
    pub author:        String,
    pub position:      f64,      // the position in the book, in seconds
    pub duration:      f64,      // the length of the whole book, in seconds
    pub chapter_title: Option<String>,
    pub speed:         f32,
    pub volume:        f32,
    pub status:        PlaybackStatus,
}

pub enum PlaybackStatus {
    Stopped,
    Playing,
    Paused,
    /// The buffer is empty, and the engine waits for data. The user did not
    /// stop the playback.
    Stalled,
}
```

The user interface reads this structure one time for each frame. The engine
writes it. This pattern is the same pattern that the download progress map of
sub-project 1b uses.

## 6. The byte sources

### 6.1 The local file

`LocalFile` holds a `std::fs::File`. The engine uses this source when the disk
has all the audio files of the book.

### 6.2 The file on the server

`HttpFile` reads `GET /api/items/:id/file/:ino/download`. Sub-project 1b
measured that this endpoint gives `206 Partial Content` and `Accept-Ranges:
bytes`.

The structure operates in this sequence:

1. The constructor sends a range request for the first block. It reads the
   header `Content-Range` and keeps the total size. The value has the form
   `bytes 0-65535/2797969`, and the number after the oblique is the total size.
   The `Seek` function needs the total size for `SeekFrom::End`.
   `Content-Length` gives the length of the part only. Therefore the code must
   not use `Content-Length` for the total size.
2. A prefetch thread reads the data of the answer. It puts the bytes in a
   buffer of approximately 8 megabytes.
3. The `Read` function copies bytes from the buffer. It does not use the
   network.
4. The `Seek` function moves the cursor. If the new position is in the buffer,
   the function only moves the cursor. If the new position is not in the
   buffer, the function stops the prefetch thread, empties the buffer, and
   starts a new range request at the new position.

The `Seek` function must be correct. The MP4 container holds the `moov` atom,
and the decoder reads that atom before it decodes audio. A wrong `Seek`
function makes an M4B file fail.

### 6.3 The selection of the source

The engine looks in the `download_files` table for the book and the user. If
the table has a complete row for every audio file, the engine uses `LocalFile`
for all tracks. If it does not, the engine uses `HttpFile` for all tracks.

A downloaded copy always has more importance than the server. The engine does
not mix the two sources in one book.

## 7. The queue, the position, and the chapters

### 7.1 The queue

`rodio::Player::append` puts a source in a queue. The engine appends the
current track and one more track only. When `Player::len()` gives 1, the engine
appends the next track.

This rule gives continuous sound between the files. It also keeps the number of
open connections at two. A book with 209 audio files must not open 209
connections.

### 7.2 The position

`Player::get_pos()` gives the position in the current track. It starts again at
zero for each track in the queue.

A measurement on 2026-08-10 shows that `get_pos()` gives the time of the
listener, and not the position in the media. A sound of 1.0 second at the speed
2.0 gives 0.5 seconds. The server counts the seconds of the recording.
Therefore the engine multiplies by the speed:

```
position_in_book = track.start_offset + get_pos() * speed
```

The function `media_position()` does this calculation. The test module
`src/player/engine/pos_probe.rs` holds the measurement. If a later version of
`rodio` changes this behaviour, that test fails.

The same measurement shows that a speed of 2.0 keeps the number of samples.
`rodio` does not remove samples. It increases the sample rate that the source
reports.

`DownloadPlan::start_offset(index)` in `src/logic/download/plan.rs` gives the
value `start_offset`. That function exists, and this design does not change it.

### 7.3 The seek operation

The engine changes a position in the book to a track and an offset:

- If the target is in the current track, the engine calls `Player::try_seek`.
- If the target is in a different track, the engine calls `Player::clear()`. It
  then makes the queue again from that track. It then calls `Player::try_seek`
  for the offset in that track.

### 7.4 The chapters

The chapters come from the field `media.chapters` of the API. The engine does
not read the chapters from the audio file. The downloader keeps the chapters in
`item.json`. Therefore the offline mode has the chapters.

"Next chapter" is a seek operation to the start of the next chapter. "Previous
chapter" is a seek operation to the start of the current chapter, or to the
start of the chapter before it if the position is less than 3 seconds after the
start.

If the book has no chapters, the two commands do nothing. 118 books in the test
library have no chapters.

## 8. The loss of the connection

The endpoint pool of sub-project 1 gives the addresses. The prefetch thread
sends the range request again after a short delay. The delay increases after
each failure. The pool changes to a different address if the first address does
not answer.

If the buffer becomes empty, the worker thread does this:

1. It calls `Player::pause()`. This is an internal operation.
2. It sets `status` to `Stalled`.
3. The user interface shows "Reconnecting".

The status for the user does not change to `Paused`. The user does not press a
key. When the data comes again, the worker calls `Player::play()`, sets the
status to `Playing`, and shows the message "Reconnected" for a short time.

The engine pauses the playback. It does not send silence. Silence moves
`get_pos()` forward, and then the position does not agree with the sound.

## 9. The changes to the database and to the configuration

### 9.1 The database

Migration v4 removes two columns that only VLC uses:

- `users.is_vlc_running`
- `users.is_vlc_launched_first_time`

SQLite removes a column with `ALTER TABLE ... DROP COLUMN`. SQLite version
3.35.0 gives this statement. The crate `rusqlite` has the feature `bundled`,
and that feature gives a newer version. The migration examines the column
first, in the same way as migration v2. If the statement fails, the migration
keeps the column and writes a message in the log. An unused column does no
damage.

### 9.2 The configuration

The block `[player]` has four keys. All four keys control VLC. The design
removes all four:

| Key | Use today |
|---|---|
| `cvlc` | Selects `cvlc` or `vlc` |
| `cvlc_term` | Starts `netcat` in a terminal |
| `address` | The address of the remote control interface |
| `port` | The port of the remote control interface |

The application must not fail if an old configuration file has these keys. The
parser reads the keys that it knows, and it does not examine the other keys.

## 10. The code that the design removes

| Path | Reason |
|---|---|
| `src/player/vlc/start_vlc.rs` | The engine starts the audio |
| `src/player/vlc/fetch_vlc_data.rs` | The engine gives the position |
| `src/player/vlc/exec_nc.rs` | There is no remote control interface |
| `src/player/vlc/quit_vlc.rs` | There is no separate program |
| `src/utils/vlc_tcp_stream.rs` | There is no remote control interface |
| `src/logic/handle_input/handle_l_book_offline.rs` | One source trait removes the duplication |

Two dependencies go away. `vlc-rc` and `regex` are only in
`fetch_vlc_data.rs`. A measurement on 2026-08-10 confirms this.

The dependencies `rodio` and `cpal` come in. `cpal` comes with `rodio`.

## 11. Error handling

| Condition | Behaviour |
|---|---|
| The computer has no sound card | The engine gives an error at the start. The application shows a message and stays in the list. |
| The decoder does not know the format | The engine gives an error for that book. It does not stop the application. |
| A range request fails | The prefetch thread sends the request again. See section 8. |
| A seek operation fails | The engine keeps the position and shows a message. It does not stop the playback. |
| The server refuses the token | `ApiError::Unauthorized` goes to the user interface, in the same way as today. |

## 12. Test plan

### 12.1 Tests that need no sound card

This point is important, because the continuous integration machine has no
sound card. `rodio::Player::new()` gives a player and a
`SourcesQueueOutput`. It does not open a device. A test reads the samples from
the output and examines them.

### 12.2 Unit tests

- The calculation changes a position in the book to a track and an offset.
- The calculation gives the correct position for the last track of a book with
  209 files.
- The queue appends the next track when one track stays.
- The chapter calculation gives the next chapter and the previous chapter.
- The chapter calculation does nothing when the book has no chapters.
- The rule "previous chapter" goes to the chapter before it when the position
  is less than 3 seconds after the start.
- The selection of the source gives `LocalFile` when the disk has all files.
- The selection of the source gives `HttpFile` when one file is absent.

### 12.3 Tests with a mock server

These tests use `wiremock`:

- `HttpFile` sends the header `Range: bytes=<position>-`.
- `HttpFile` gives the bytes from the buffer, and it does not send a request
  for each read operation.
- `Seek` to a position that is not in the buffer sends a new range request.
- `Seek` to a position that is in the buffer sends no request.
- `HttpFile` accepts an answer with the status `200` when the server does not
  obey the range.
- `HttpFile` sends the request again when the connection stops.
- The token is in the header `Authorization`. The address holds no token.

### 12.4 Tests of the engine

- A generated WAV file plays through the engine, and the position increases.
- A queue of two generated files goes over the boundary, and the position
  continues to increase.
- A change of the speed changes the speed of the current file.

### 12.5 Tests with a real server

The pty harness of the earlier work drives these tests:

- A book with one M4B file plays.
- A book with many MP3 files plays through the boundary between two files.
- The book with 209 audio files plays, and the position is correct.
- A change of the speed operates during the playback, with no new start.
- A seek operation over a file boundary gives the correct position.
- `ps aux` shows no token.
- A stop of the network makes the status `Stalled`. The playback continues by
  itself when the network comes back.
- The server shows the correct progress after the playback.

## 13. Risks and limitations

| Risk | Result |
|---|---|
| ALSA is a dynamic dependency of the program on Linux | A fully static Linux binary is not possible with sound. The user accepts this. |
| `libasound2-dev` is necessary to build on Linux | The CI workflow installs this package. The rule "no C toolchain" stays correct, because `alsa-sys` only calls `pkg-config`. |
| `rodio` 0.22 uses `symphonia` 0.5.5 | The feature `symphonia-all` gives every format of `symphonia`. Section 3.1 gives the measurement of each format. A user with an Opus file or a WMA file cannot play that file. |
| The behaviour of `get_pos()` with a speed that is not 1.0 | Closed on 2026-08-10. The measurement shows that `get_pos()` gives the time of the listener. `media_position()` multiplies by the speed. A test keeps this behaviour correct. |
| macOS and Windows | `cpal` supports both. This design does not test them. The removal of VLC corrects fault `fe4116` on macOS, because `cvlc` is not necessary. |

## 14. Decisions

| Decision | Selection | Reason |
|---|---|---|
| The change to the new engine | Remove VLC in one step | Two engines keep every fault of VLC, and they do not correct T-5. |
| The byte source for the stream | A range reader with a buffer in the memory | The sound starts quickly, and the offline path and the online path use one trait. |
| The structure | One service, and a state that the user interface reads | This removes the four loops that are almost the same. |
| The source when a copy is on the disk | The copy on the disk always | The user asked for this. It also removes the network from a book that the user downloaded. |
| The behaviour at a loss of the connection | Continue without an action of the user | The user asked for this. A short message tells the user that the connection came back. |
| The source of the chapters | The API | The API gives correct chapters for 1938 books. A parser for M4B files is not necessary. |
| The number of tracks in the queue | Two | The sound is continuous, and the program opens two connections and not 209. |

## 15. The sequence of the work

| Task | Contents |
|---|---|
| 2a | The dependencies, and a measurement of `get_pos()` with a speed |
| 2b | `track.rs`: the position, the seek calculation, and the chapters |
| 2c | `http_file.rs`: the range reader and the prefetch thread |
| 2d | `source.rs` and `worker.rs`: the engine and the commands |
| 2e | `logic/playback`: one progress loop, and the connection to `app.rs` |
| 2f | The removal of VLC, migration v4, and the configuration |

Each task keeps `cargo clippy --all-targets -- -D warnings` clean. Each task
keeps `cargo test` correct.
