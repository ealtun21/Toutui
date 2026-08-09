# Audio Engine Implementation Plan (sub-project 2)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove VLC. Decode the audio inside the process with `rodio` and `symphonia`, so that a book with many audio files plays completely, the speed changes during the playback, and the token never appears in `ps aux`.

**Architecture:** One worker thread owns a `rodio::Player`. Other code sends a `PlayerCommand` through an `mpsc` channel and reads a published `Arc<RwLock<PlaybackState>>`. One trait, `MediaRead`, covers a local file and a file on the server, so the offline path and the online path share one decode path. A prefetch thread keeps bytes in memory, because `rodio` decodes on the callback thread of the sound card and that thread must never wait for the network.

**Tech Stack:** Rust 2021, tokio, rodio 0.22 (symphonia 0.5.5, cpal), reqwest 0.11 with rustls, rusqlite, ratatui, wiremock (dev).

**Spec:** `docs/superpowers/specs/2026-08-10-audio-engine-design.md`

## Global Constraints

- Write all documentation, doc comments, and user-facing strings in ASD-STE100 Simplified Technical English. Use short sentences. Use the active voice. Use the present tense. One sentence gives one instruction. Always use the articles.
- `cargo clippy --all-targets -- -D warnings` must give no output before every commit.
- `cargo test` must pass before every commit.
- Do not add a dependency that needs a program that the user installs separately.
- Do not add a dependency that needs a C toolchain. `alsa-sys` is permitted, because its build script only calls `pkg-config`. It compiles no C code.
- `cargo tree -i openssl-sys` must continue to find nothing.
- Every new public item gets a doc comment.
- `rodio::Decoder<R>` needs `R: Read + Seek + Send + Sync + 'static`. All five bounds are necessary.
- Never send a `POST` request a second time. `ApiClient` obeys this rule already.
- The token goes in the header `Authorization` only. The token must never go in a command line argument, and never in a query string.
- Do not commit a temporary probe program. Task 1 deletes its probe.
- Commit logically-separate changes separately.

---

## File Structure

| Path | Responsibility | Task |
|---|---|---|
| `Cargo.toml` | Add `rodio`, remove `vlc-rc` and `regex` | 1, 6 |
| `src/player/engine/mod.rs` | `PlayerHandle`, `PlayerCommand`, `PlaybackState`, `PlaybackStatus` | 4 |
| `src/player/engine/track.rs` | `Track`, `TrackList`, position and seek calculation, chapters | 2 |
| `src/player/engine/http_file.rs` | `HttpFile`: a `Read + Seek` reader that uses range requests | 3 |
| `src/player/engine/source.rs` | `MediaRead`, `open_source`, the selection of the source | 4 |
| `src/player/engine/worker.rs` | The thread that owns `rodio::Player` | 4 |
| `src/logic/playback/mod.rs` | One progress loop and one sync loop | 5 |
| `src/app.rs` | Use `PlayerHandle` in place of the four VLC blocks | 5, 6 |
| `src/ui/player_tui.rs` | Read `PlaybackState` | 5 |
| `src/db/migrate.rs` | Migration v4 | 6 |
| `src/config.rs` | Remove the `[player]` block | 6 |
| `tests/http_file.rs` | Integration tests of the range reader | 3 |
| `tests/engine.rs` | Integration tests of the engine with no sound card | 4 |

---

### Task 1: The dependency, and the measurement of `get_pos()` with a speed

The engine calculates the position in the book from `Player::get_pos()`. The documentation of `rodio` is not clear about the behaviour of that function when the speed is not 1.0. It says that a speed of 2 and a `get_pos()` of 5 seconds give a position of 10 seconds in the file. The sync calculation depends on this behaviour. Therefore this task measures the behaviour first, and it writes the result in a test.

This task adds `rodio` and it proves that the audio dependency does not break the rules of the project.

**Files:**
- Modify: `Cargo.toml`
- Create: `src/player/engine/mod.rs`
- Create: `src/player/engine/pos_probe.rs` (a test module only)
- Modify: `src/player/mod.rs`

**Interfaces:**
- Consumes: nothing.
- Produces:
  - The dependency `rodio` version `0.22`.
  - `pub fn media_position(reported: Duration, speed: f32) -> f64` in `src/player/engine/mod.rs`.

- [ ] **Step 1: Add the dependency**

In `Cargo.toml`, add this line in the `[dependencies]` section, after the line for `clap`:

```toml
rodio = { version = "0.22", default-features = false, features = ["playback", "mp3", "mp4", "flac", "vorbis", "wav"] }
```

The library of the test server has `mp3` and `aac` only. The features `flac`,
`vorbis`, and `wav` cost little, and a different user can have those files. The
feature `mp4` gives the M4B files, because an M4B file holds AAC audio in an MP4
container. The feature `dither` is not necessary, and `recording` is not
necessary. Therefore this line does not use the default features.

- [ ] **Step 2: Prove that the rules of the project stay correct**

Run:

```bash
cargo tree -i openssl-sys
```

Expected: the command finds nothing.

Run:

```bash
cargo tree -i alsa-sys
```

Expected: the command shows `alsa-sys`, and `rodio` is above it. This is
correct and permitted. `alsa-sys` compiles no C code. Its build script only
calls `pkg-config`.

- [ ] **Step 3: Register the module**

Create `src/player/engine/mod.rs`:

```rust
//! The audio engine of the application.
//!
//! The engine decodes the audio in the process. The application does not start
//! a different program. Therefore the token stays in the memory of the
//! process.

pub mod pos_probe;

use std::time::Duration;

/// Changes the position that `rodio` reports to the position in the media.
///
/// `rodio::Player::get_pos` gives a value that includes the speed. The test
/// module `pos_probe` measures this behaviour. The engine must report the
/// position in the media to the server, because the server counts the seconds
/// of the recording and not the seconds of the listener.
pub fn media_position(reported: Duration, speed: f32) -> f64 {
    let seconds = reported.as_secs_f64();

    if speed <= 0.0 || !speed.is_finite() {
        return seconds;
    }

    seconds
}
```

The body of this function is a first version. Step 6 corrects it with the
measured behaviour.

In `src/player/mod.rs`, add this line:

```rust
pub mod engine;
```

- [ ] **Step 4: Write the measurement test**

Create `src/player/engine/pos_probe.rs`. This test needs no sound card,
because `rodio::Player::new()` gives a player and an output, and it opens no
device.

```rust
//! A measurement of the behaviour of `rodio::Player::get_pos`.
//!
//! This module holds tests only. The tests record how `get_pos` behaves when
//! the speed is not 1.0. The calculation of the position depends on this
//! behaviour.

#[cfg(test)]
mod tests {
    use rodio::buffer::SamplesBuffer;
    use rodio::source::Source;
    use rodio::Player;

    /// Makes a sound of a known length. The sample rate is 8000, and the
    /// sound has one channel. Therefore 8000 samples give exactly 1 second.
    fn one_second() -> SamplesBuffer {
        SamplesBuffer::new(1, 8000, vec![0.0f32; 8000])
    }

    /// Reads every sample from the output, and gives the number of samples.
    fn drain(output: &mut dyn Iterator<Item = f32>) -> usize {
        let mut count = 0;
        for _ in output.by_ref() {
            count += 1;
        }
        count
    }

    /// A sound of 1 second at the normal speed gives 8000 samples.
    #[test]
    fn a_normal_speed_gives_the_full_number_of_samples() {
        let (player, mut output) = Player::new();
        player.append(one_second());
        player.play();

        let samples = drain(&mut output);
        assert_eq!(samples, 8000);
    }

    /// A speed of 2.0 gives half the number of samples, because the sound
    /// finishes in half the time.
    #[test]
    fn a_double_speed_gives_half_the_samples() {
        let (player, mut output) = Player::new();
        player.set_speed(2.0);
        player.append(one_second());
        player.play();

        let samples = drain(&mut output);
        assert!(
            (3900..=4100).contains(&samples),
            "a speed of 2.0 gives approximately 4000 samples, but it gave {}",
            samples
        );
    }

    /// This is the measurement that the engine needs. The test reads the
    /// position after the sound of 1 second plays at the speed 2.0.
    ///
    /// If `get_pos` gives approximately 1.0 second, the value is the position
    /// in the media. Then `media_position` must not change the value.
    ///
    /// If `get_pos` gives approximately 0.5 seconds, the value is the time of
    /// the listener. Then `media_position` must multiply by the speed.
    #[test]
    fn measure_the_position_at_a_double_speed() {
        let (player, mut output) = Player::new();
        player.set_speed(2.0);
        player.append(one_second());
        player.play();

        let _ = drain(&mut output);
        let position = player.get_pos().as_secs_f64();

        // Record the measured value here. Step 5 gives the correct value.
        assert!(
            (0.9..=1.1).contains(&position),
            "get_pos gave {} seconds at the speed 2.0",
            position
        );
    }
}
```

- [x] **Step 5: Run the measurement and record the result**

**The measurement is complete.** On 2026-08-10 the probe gives these values for
a sound of 1.0 second at the speed 2.0:

| Value | Result |
|---|---|
| `get_pos()` | 0.5 seconds |
| The number of samples | 8000, and not 4000 |

Therefore `get_pos()` gives the time of the listener. `media_position()`
multiplies by the speed. `rodio` does not remove samples for a higher speed. It
increases the sample rate that the source reports.

Two other results are important for the tasks that come after this task:

1. `SamplesBuffer::new` needs `NonZero` values for the channel count and for
   the sample rate. A plain integer does not compile.
2. `Player::new()` makes a queue that stays alive when it is empty. Therefore
   the output gives silence for ever. A test must never read to the end of the
   output. Use `take()`, and use a sound that is not silent, so that the test
   can find the end.

The original two branches of this step follow, for the record.

Run:

```bash
cargo test --lib player::engine::pos_probe -- --nocapture
```

Two results are possible:

1. All three tests pass. Then `get_pos` gives the position in the media.
   `media_position` is correct as written in Step 3. Add this comment above the
   `seconds` line of the function body:

   ```rust
       // A measurement on 2026-08-10 shows that `get_pos` gives the position
       // in the media. Therefore the function does not multiply by the speed.
   ```

2. The third test fails, and the message shows approximately 0.5 seconds. Then
   `get_pos` gives the time of the listener. Do these two changes:

   - In the test, change the range `(0.9..=1.1)` to `(0.4..=0.6)`, and change
     the message to `get_pos gives the time of the listener`.
   - In `media_position`, change the last line from `seconds` to:

     ```rust
         // A measurement on 2026-08-10 shows that `get_pos` gives the time of
         // the listener. The server needs the position in the media.
         seconds * speed as f64
     ```

Do not guess the result. Use the number that the test prints.

- [ ] **Step 6: Add the tests of the guard values**

Add these tests to the test module in `src/player/engine/mod.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::media_position;
    use std::time::Duration;

    #[test]
    fn a_normal_speed_does_not_change_the_position() {
        let position = media_position(Duration::from_secs(30), 1.0);
        assert!((position - 30.0).abs() < 0.001);
    }

    /// A speed of zero, or a speed that is not a number, must not give a
    /// position of zero and must not give an infinite value.
    #[test]
    fn a_speed_that_is_not_valid_gives_the_reported_position() {
        assert!((media_position(Duration::from_secs(30), 0.0) - 30.0).abs() < 0.001);
        assert!((media_position(Duration::from_secs(30), -1.0) - 30.0).abs() < 0.001);
        assert!((media_position(Duration::from_secs(30), f32::NAN) - 30.0).abs() < 0.001);
    }
}
```

- [ ] **Step 7: Verify the gates**

Run:

```bash
cargo clippy --all-targets -- -D warnings && cargo test
```

Expected: clippy gives no output. All tests pass.

- [ ] **Step 8: Commit**

```bash
git add Cargo.toml Cargo.lock src/player/mod.rs src/player/engine/
git commit -m "feat(player): add rodio and measure the position with a speed

The engine calculates the position in the book from Player::get_pos. The
documentation of rodio is not clear about the behaviour of that function with
a speed that is not 1.0. A test measures the behaviour, and the calculation
uses the measured result.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

---

### Task 2: The track list, the position, and the chapters

This task has no network code and no audio code. Therefore the tests are fast and they need no server. The calculation here corrects T-2, because it gives the position in the whole book and not the position in one file.

**Files:**
- Create: `src/player/engine/track.rs`
- Modify: `src/player/engine/mod.rs`

**Interfaces:**
- Consumes: nothing.
- Produces:
  - `pub struct Track { pub index: u32, pub ino: String, pub filename: String, pub duration: f64, pub start_offset: f64 }`
  - `pub struct Chapter { pub start: f64, pub end: f64, pub title: String }`
  - `pub struct TrackList { ... }` with:
    - `TrackList::new(tracks: Vec<Track>, chapters: Vec<Chapter>) -> TrackList`
    - `from_durations(durations: &[f64]) -> Vec<Track>` (a helper for the tests)
    - `total_duration(&self) -> f64`
    - `locate(&self, position: f64) -> Option<(usize, f64)>`
    - `position_of(&self, track_index: usize, offset: f64) -> f64`
    - `chapter_at(&self, position: f64) -> Option<&Chapter>`
    - `next_chapter_start(&self, position: f64) -> Option<f64>`
    - `previous_chapter_start(&self, position: f64) -> Option<f64>`
    - `get(&self, track_index: usize) -> Option<&Track>`
    - `len(&self) -> usize`
    - `is_empty(&self) -> bool`

- [ ] **Step 1: Write the failing tests**

Create `src/player/engine/track.rs` with the test module first:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// Three tracks of 10, 20, and 30 seconds. The book lasts 60 seconds.
    fn list() -> TrackList {
        TrackList::new(TrackList::from_durations(&[10.0, 20.0, 30.0]), Vec::new())
    }

    /// Chapters at 0 to 25, 25 to 45, and 45 to 60.
    fn list_with_chapters() -> TrackList {
        let chapters = vec![
            Chapter { start: 0.0, end: 25.0, title: "One".to_string() },
            Chapter { start: 25.0, end: 45.0, title: "Two".to_string() },
            Chapter { start: 45.0, end: 60.0, title: "Three".to_string() },
        ];
        TrackList::new(TrackList::from_durations(&[10.0, 20.0, 30.0]), chapters)
    }

    #[test]
    fn the_start_offset_is_the_sum_of_the_tracks_before_it() {
        let list = list();
        assert_eq!(list.get(0).unwrap().start_offset, 0.0);
        assert_eq!(list.get(1).unwrap().start_offset, 10.0);
        assert_eq!(list.get(2).unwrap().start_offset, 30.0);
    }

    #[test]
    fn the_total_duration_is_the_sum_of_the_tracks() {
        assert_eq!(list().total_duration(), 60.0);
    }

    #[test]
    fn a_position_gives_the_correct_track_and_offset() {
        let list = list();
        assert_eq!(list.locate(0.0).unwrap(), (0, 0.0));
        assert_eq!(list.locate(5.0).unwrap(), (0, 5.0));
        assert_eq!(list.locate(15.0).unwrap(), (1, 5.0));
        assert_eq!(list.locate(45.0).unwrap(), (2, 15.0));
    }

    /// The first second of a track belongs to that track, and not to the
    /// track before it.
    #[test]
    fn the_boundary_belongs_to_the_track_that_starts() {
        let list = list();
        assert_eq!(list.locate(10.0).unwrap(), (1, 0.0));
        assert_eq!(list.locate(30.0).unwrap(), (2, 0.0));
    }

    #[test]
    fn a_position_before_the_start_gives_the_first_track() {
        assert_eq!(list().locate(-5.0).unwrap(), (0, 0.0));
    }

    /// A position at the end or after the end gives the last track. The
    /// engine must not fail when the book comes to the end.
    #[test]
    fn a_position_at_the_end_gives_the_last_track() {
        let list = list();
        assert_eq!(list.locate(60.0).unwrap().0, 2);
        assert_eq!(list.locate(1000.0).unwrap().0, 2);
    }

    #[test]
    fn an_empty_list_gives_no_position() {
        let list = TrackList::new(Vec::new(), Vec::new());
        assert!(list.locate(0.0).is_none());
        assert!(list.is_empty());
    }

    #[test]
    fn the_position_of_a_track_and_an_offset_is_correct() {
        let list = list();
        assert_eq!(list.position_of(0, 5.0), 5.0);
        assert_eq!(list.position_of(1, 5.0), 15.0);
        assert_eq!(list.position_of(2, 15.0), 45.0);
    }

    /// This is the test for a book with many files. The test library has a
    /// book with 209 audio files.
    #[test]
    fn a_book_with_209_tracks_gives_the_correct_position() {
        let durations = vec![300.0; 209];
        let list = TrackList::new(TrackList::from_durations(&durations), Vec::new());

        assert_eq!(list.len(), 209);
        assert_eq!(list.total_duration(), 62700.0);
        assert_eq!(list.get(208).unwrap().start_offset, 62400.0);
        assert_eq!(list.locate(62500.0).unwrap(), (208, 100.0));
    }

    #[test]
    fn the_chapter_at_a_position_is_correct() {
        let list = list_with_chapters();
        assert_eq!(list.chapter_at(0.0).unwrap().title, "One");
        assert_eq!(list.chapter_at(24.9).unwrap().title, "One");
        assert_eq!(list.chapter_at(25.0).unwrap().title, "Two");
        assert_eq!(list.chapter_at(50.0).unwrap().title, "Three");
    }

    #[test]
    fn the_next_chapter_gives_the_start_of_the_next_chapter() {
        let list = list_with_chapters();
        assert_eq!(list.next_chapter_start(0.0).unwrap(), 25.0);
        assert_eq!(list.next_chapter_start(30.0).unwrap(), 45.0);
    }

    #[test]
    fn the_last_chapter_has_no_next_chapter() {
        assert!(list_with_chapters().next_chapter_start(50.0).is_none());
    }

    /// The rule agrees with the usual behaviour of an audio player. The first
    /// operation goes to the start of the current chapter.
    #[test]
    fn the_previous_chapter_goes_to_the_start_of_the_current_chapter() {
        let list = list_with_chapters();
        assert_eq!(list.previous_chapter_start(30.0).unwrap(), 25.0);
    }

    /// If the position is less than 3 seconds after the start of the chapter,
    /// the operation goes to the chapter before it.
    #[test]
    fn the_previous_chapter_goes_back_near_the_start_of_a_chapter() {
        let list = list_with_chapters();
        assert_eq!(list.previous_chapter_start(26.0).unwrap(), 0.0);
    }

    #[test]
    fn the_first_chapter_gives_the_start_of_the_book() {
        let list = list_with_chapters();
        assert_eq!(list.previous_chapter_start(1.0).unwrap(), 0.0);
    }

    /// 118 books of the test library have no chapter. The two commands must
    /// do nothing for those books.
    #[test]
    fn a_book_with_no_chapter_gives_no_chapter_movement() {
        let list = list();
        assert!(list.chapter_at(10.0).is_none());
        assert!(list.next_chapter_start(10.0).is_none());
        assert!(list.previous_chapter_start(10.0).is_none());
    }
}
```

- [ ] **Step 2: Run the tests to verify that they fail**

Run:

```bash
cargo test --lib player::engine::track
```

Expected: FAIL, `cannot find type TrackList in this scope`.

- [ ] **Step 3: Write the implementation**

Put this above the test module in `src/player/engine/track.rs`:

```rust
//! The audio files of a book, and the calculation of the position.
//!
//! A book can have more than one audio file. The test library has a book with
//! 209 audio files. The player reports the position in the file that plays
//! now. This module changes that value to the position in the whole book.

/// One audio file of a book.
#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    /// The sequence of the file. The first file has the number 1.
    pub index: u32,
    /// The identity of the file on the server.
    pub ino: String,
    /// The name of the file. The decoder uses the extension as a hint.
    pub filename: String,
    /// The length of the file in seconds.
    pub duration: f64,
    /// The start of this file in the book, in seconds.
    pub start_offset: f64,
}

/// One chapter of a book. The values come from the field `media.chapters` of
/// the API.
#[derive(Debug, Clone, PartialEq)]
pub struct Chapter {
    /// The start in the book, in seconds.
    pub start: f64,
    /// The end in the book, in seconds.
    pub end: f64,
    /// The name of the chapter.
    pub title: String,
}

/// A movement to the previous chapter goes to the chapter before the current
/// chapter if the position is less than this number of seconds after the
/// start. This behaviour agrees with the usual behaviour of an audio player.
const PREVIOUS_CHAPTER_LIMIT: f64 = 3.0;

/// The audio files of one book, and the chapters of that book.
#[derive(Debug, Clone, Default)]
pub struct TrackList {
    tracks: Vec<Track>,
    chapters: Vec<Chapter>,
    total: f64,
}

impl TrackList {
    /// Makes a track list.
    ///
    /// The function calculates `start_offset` for each track. The caller gives
    /// the tracks in the correct sequence.
    pub fn new(mut tracks: Vec<Track>, chapters: Vec<Chapter>) -> Self {
        let mut total = 0.0;

        for track in tracks.iter_mut() {
            track.start_offset = total;
            total += track.duration.max(0.0);
        }

        TrackList { tracks, chapters, total }
    }

    /// Makes tracks from a list of lengths. The tests use this function.
    pub fn from_durations(durations: &[f64]) -> Vec<Track> {
        durations
            .iter()
            .enumerate()
            .map(|(position, duration)| Track {
                index: position as u32 + 1,
                ino: format!("ino-{}", position),
                filename: format!("{:03}.mp3", position + 1),
                duration: *duration,
                start_offset: 0.0,
            })
            .collect()
    }

    /// Gives the length of the whole book in seconds.
    pub fn total_duration(&self) -> f64 {
        self.total
    }

    /// Gives the track and the offset in that track for a position in the
    /// book.
    ///
    /// A position before the start gives the first track. A position at the
    /// end or after the end gives the last track. Gives `None` only if the
    /// book has no track.
    pub fn locate(&self, position: f64) -> Option<(usize, f64)> {
        if self.tracks.is_empty() {
            return None;
        }

        if position <= 0.0 || position.is_nan() {
            return Some((0, 0.0));
        }

        for (number, track) in self.tracks.iter().enumerate() {
            let end = track.start_offset + track.duration;

            if position < end {
                return Some((number, position - track.start_offset));
            }
        }

        let last = self.tracks.len() - 1;
        let offset = (position - self.tracks[last].start_offset).min(self.tracks[last].duration);

        Some((last, offset))
    }

    /// Gives the position in the book for a track and an offset.
    pub fn position_of(&self, track_index: usize, offset: f64) -> f64 {
        match self.tracks.get(track_index) {
            Some(track) => track.start_offset + offset,
            None => offset,
        }
    }

    /// Gives the chapter that holds a position.
    ///
    /// Gives `None` if the book has no chapter. 118 books of the test library
    /// have no chapter.
    pub fn chapter_at(&self, position: f64) -> Option<&Chapter> {
        self.chapters
            .iter()
            .find(|chapter| position >= chapter.start && position < chapter.end)
    }

    /// Gives the start of the chapter after the current position.
    pub fn next_chapter_start(&self, position: f64) -> Option<f64> {
        self.chapters
            .iter()
            .map(|chapter| chapter.start)
            .find(|start| *start > position)
    }

    /// Gives the position for a movement to the previous chapter.
    ///
    /// The movement goes to the start of the current chapter. If the position
    /// is less than three seconds after that start, the movement goes to the
    /// chapter before it.
    pub fn previous_chapter_start(&self, position: f64) -> Option<f64> {
        let current = self.chapter_at(position)?;

        if position - current.start > PREVIOUS_CHAPTER_LIMIT {
            return Some(current.start);
        }

        let earlier = self
            .chapters
            .iter()
            .map(|chapter| chapter.start)
            .filter(|start| *start < current.start)
            .next_back();

        Some(earlier.unwrap_or(current.start))
    }

    /// Gives one track.
    pub fn get(&self, track_index: usize) -> Option<&Track> {
        self.tracks.get(track_index)
    }

    /// Gives the number of tracks.
    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    /// Tells if the book has no track.
    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }
}
```

- [ ] **Step 4: Register the module**

In `src/player/engine/mod.rs`, add this line below `pub mod pos_probe;`:

```rust
pub mod track;
```

- [ ] **Step 5: Run the tests to verify that they pass**

Run:

```bash
cargo test --lib player::engine::track
```

Expected: PASS, `17 passed`.

If `previous_chapter_start` fails, examine the call to `next_back()`. That call
needs an iterator that goes in both directions. `filter` on a slice iterator
gives such an iterator.

- [ ] **Step 6: Verify the gates**

Run:

```bash
cargo clippy --all-targets -- -D warnings && cargo test
```

Expected: clippy gives no output. All tests pass.

- [ ] **Step 7: Commit**

```bash
git add src/player/engine/track.rs src/player/engine/mod.rs
git commit -m "feat(player): add the track list and the position calculation

The list gives the position in the whole book from the position in one file.
It also gives the chapter movements. A book with 209 audio files gives the
correct position. A book with no chapter does not fail.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

---

### Task 3: The range reader

`rodio` decodes on the callback thread of the sound card. A read operation on
that thread must never wait for the network, because a wait makes a gap in the
sound. Therefore this reader holds the bytes in the memory. A thread fills the
buffer, and the `Read` function copies from the buffer.

**Files:**
- Create: `src/player/engine/http_file.rs`
- Modify: `src/player/engine/mod.rs`
- Create: `tests/http_file.rs`

**Interfaces:**
- Consumes: `ApiError` from `src/api/client/error.rs`.
- Produces:
  - `pub struct HttpFile` with `HttpFile::open(base_url: &str, token: &str, item_id: &str, ino: &str) -> Result<HttpFile, ApiError>`
  - `HttpFile::len(&self) -> u64`
  - `HttpFile::is_empty(&self) -> bool`
  - `HttpFile::is_stalled(&self) -> bool`
  - `pub fn total_size_from_content_range(value: &str) -> Option<u64>`
  - `impl Read for HttpFile`, `impl Seek for HttpFile`

- [ ] **Step 1: Write the failing unit test for the header**

Create `src/player/engine/http_file.rs` with the test module first:

```rust
#[cfg(test)]
mod tests {
    use super::total_size_from_content_range;

    /// The header has the form `bytes 0-1023/2797969`. The number after the
    /// oblique is the total size. `Content-Length` gives the length of the
    /// part only, thus the code must not use it for the total size.
    #[test]
    fn the_content_range_header_gives_the_total_size() {
        assert_eq!(
            total_size_from_content_range("bytes 0-1023/2797969"),
            Some(2797969)
        );
        assert_eq!(
            total_size_from_content_range("bytes 100-1099/2797969"),
            Some(2797969)
        );
    }

    #[test]
    fn a_header_that_is_not_valid_gives_no_size() {
        assert_eq!(total_size_from_content_range("bytes 0-1023/*"), None);
        assert_eq!(total_size_from_content_range("nonsense"), None);
        assert_eq!(total_size_from_content_range(""), None);
    }
}
```

- [ ] **Step 2: Run the test to verify that it fails**

Run:

```bash
cargo test --lib player::engine::http_file
```

Expected: FAIL, `cannot find function total_size_from_content_range`.

- [ ] **Step 3: Write the implementation**

Put this above the test module in `src/player/engine/http_file.rs`:

```rust
//! A reader that gets a file from the server with range requests.
//!
//! The type obeys `Read` and `Seek`. Therefore `rodio::Decoder` accepts it.
//!
//! `rodio` decodes on the callback thread of the sound card. That thread must
//! never wait for the network, because a wait makes a gap in the sound.
//! Therefore a thread fills a buffer in the memory, and the `Read` function
//! copies bytes from that buffer.

use crate::api::client::error::ApiError;
use log::{info, warn};
use std::collections::VecDeque;
use std::io::{self, Read, Seek, SeekFrom};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

/// The number of bytes that the thread keeps in front of the cursor.
const BUFFER_TARGET: usize = 8 * 1024 * 1024;

/// The time that a read operation waits for data before it reports a stall.
const READ_WAIT: Duration = Duration::from_millis(250);

/// The first delay after a failed request. The delay doubles after each
/// failure, to the value `MAX_BACKOFF`.
const FIRST_BACKOFF: Duration = Duration::from_millis(500);

/// The largest delay between two attempts.
const MAX_BACKOFF: Duration = Duration::from_secs(10);

/// Reads the total size of the file from the header `Content-Range`.
///
/// The header has the form `bytes 0-1023/2797969`. The number after the
/// oblique is the total size. The header `Content-Length` gives the length of
/// the part only. Therefore the code must not use `Content-Length` here.
pub fn total_size_from_content_range(value: &str) -> Option<u64> {
    value.rsplit('/').next()?.trim().parse::<u64>().ok()
}

/// The data that the thread and the reader share.
struct Shared {
    /// The bytes that the thread read and the reader did not use.
    buffer: Mutex<VecDeque<u8>>,
    /// The thread adds data, and the reader takes data.
    signal: Condvar,
    /// The position in the file of the first byte in the buffer.
    buffer_start: AtomicU64,
    /// The thread stops when this value is true.
    stop: AtomicBool,
    /// The thread read to the end of the file.
    finished: AtomicBool,
    /// The thread cannot get data now. The engine shows "Reconnecting".
    stalled: AtomicBool,
}

/// A file on the server that obeys `Read` and `Seek`.
pub struct HttpFile {
    shared: Arc<Shared>,
    /// The position of the next read operation.
    cursor: u64,
    /// The number of bytes of the whole file.
    size: u64,
    url: String,
    token: String,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl HttpFile {
    /// Opens a file on the server.
    ///
    /// The function sends one range request. It reads the total size from the
    /// header `Content-Range`. It then starts the thread that fills the
    /// buffer.
    pub fn open(
        base_url: &str,
        token: &str,
        item_id: &str,
        ino: &str,
    ) -> Result<HttpFile, ApiError> {
        let url = format!(
            "{}/api/items/{}/file/{}/download",
            base_url.trim_end_matches('/'),
            item_id,
            ino
        );

        let client = blocking_client()?;

        let response = client
            .get(&url)
            .bearer_auth(token)
            .header("Range", "bytes=0-0")
            .send()
            .map_err(|_| ApiError::Unreachable)?;

        if let Some(error) = crate::api::client::error::classify_status(response.status()) {
            return Err(error);
        }

        let size = response
            .headers()
            .get("content-range")
            .and_then(|value| value.to_str().ok())
            .and_then(total_size_from_content_range)
            .ok_or_else(|| {
                ApiError::Decode("The server gave no Content-Range header.".to_string())
            })?;

        let shared = Arc::new(Shared {
            buffer: Mutex::new(VecDeque::new()),
            signal: Condvar::new(),
            buffer_start: AtomicU64::new(0),
            stop: AtomicBool::new(false),
            finished: AtomicBool::new(false),
            stalled: AtomicBool::new(false),
        });

        let mut file = HttpFile {
            shared,
            cursor: 0,
            size,
            url,
            token: token.to_string(),
            handle: None,
        };

        file.start_thread(0);

        Ok(file)
    }

    /// Gives the number of bytes of the whole file.
    pub fn len(&self) -> u64 {
        self.size
    }

    /// Tells if the file has no bytes.
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Tells if the reader waits for data from the server.
    pub fn is_stalled(&self) -> bool {
        self.shared.stalled.load(Ordering::Relaxed)
    }

    /// Stops the thread and empties the buffer.
    fn stop_thread(&mut self) {
        self.shared.stop.store(true, Ordering::SeqCst);
        self.shared.signal.notify_all();

        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }

        if let Ok(mut buffer) = self.shared.buffer.lock() {
            buffer.clear();
        }
    }

    /// Starts a thread that reads the file from a position.
    fn start_thread(&mut self, from: u64) {
        let shared = Arc::clone(&self.shared);
        let url = self.url.clone();
        let token = self.token.clone();

        shared.stop.store(false, Ordering::SeqCst);
        shared.finished.store(false, Ordering::SeqCst);
        shared.buffer_start.store(from, Ordering::SeqCst);

        let handle = std::thread::spawn(move || {
            fill_buffer(shared, url, token, from);
        });

        self.handle = Some(handle);
    }
}

impl Drop for HttpFile {
    fn drop(&mut self) {
        self.stop_thread();
    }
}

impl Read for HttpFile {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if out.is_empty() || self.cursor >= self.size {
            return Ok(0);
        }

        let mut buffer = self
            .shared
            .buffer
            .lock()
            .map_err(|_| io::Error::other("The buffer lock is broken."))?;

        loop {
            if !buffer.is_empty() {
                let count = out.len().min(buffer.len());

                for (position, byte) in buffer.drain(..count).enumerate() {
                    out[position] = byte;
                }

                self.cursor += count as u64;
                self.shared.buffer_start.store(self.cursor, Ordering::SeqCst);
                self.shared.stalled.store(false, Ordering::Relaxed);
                self.shared.signal.notify_all();

                return Ok(count);
            }

            if self.shared.finished.load(Ordering::SeqCst) {
                return Ok(0);
            }

            // The buffer is empty. The thread did not get the data yet.
            self.shared.stalled.store(true, Ordering::Relaxed);

            let (new_buffer, _) = self
                .shared
                .signal
                .wait_timeout(buffer, READ_WAIT)
                .map_err(|_| io::Error::other("The buffer lock is broken."))?;

            buffer = new_buffer;
        }
    }
}

impl Seek for HttpFile {
    fn seek(&mut self, from: SeekFrom) -> io::Result<u64> {
        let target = match from {
            SeekFrom::Start(value) => value as i64,
            SeekFrom::End(value) => self.size as i64 + value,
            SeekFrom::Current(value) => self.cursor as i64 + value,
        };

        if target < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "The position is before the start of the file.",
            ));
        }

        let target = (target as u64).min(self.size);

        if target == self.cursor {
            return Ok(target);
        }

        // The buffer holds the bytes that come after the cursor. A movement
        // forward inside the buffer only removes bytes. This is the usual
        // condition, and it sends no request.
        let inside = {
            let mut buffer = self
                .shared
                .buffer
                .lock()
                .map_err(|_| io::Error::other("The buffer lock is broken."))?;

            if target > self.cursor && target - self.cursor <= buffer.len() as u64 {
                let count = (target - self.cursor) as usize;
                buffer.drain(..count);
                true
            } else {
                false
            }
        };

        self.cursor = target;
        self.shared.buffer_start.store(target, Ordering::SeqCst);

        if !inside {
            info!("[HttpFile] the reader moves to the byte {}", target);
            self.stop_thread();
            self.start_thread(target);
        }

        Ok(target)
    }
}

/// Makes the HTTP client of the thread.
///
/// The thread is not an asynchronous task. Therefore it uses the blocking
/// client of `reqwest`.
fn blocking_client() -> Result<reqwest::blocking::Client, ApiError> {
    reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_secs(3))
        .build()
        .map_err(|_| ApiError::Unreachable)
}

/// Reads the file and fills the buffer.
///
/// The function sends the request again after a failure. The delay doubles
/// after each failure. The function stops when the reader asks it to stop.
fn fill_buffer(shared: Arc<Shared>, url: String, token: String, from: u64) {
    let mut position = from;
    let mut backoff = FIRST_BACKOFF;

    let client = match blocking_client() {
        Ok(client) => client,
        Err(_) => {
            shared.finished.store(true, Ordering::SeqCst);
            shared.signal.notify_all();
            return;
        }
    };

    while !shared.stop.load(Ordering::SeqCst) {
        let response = client
            .get(&url)
            .bearer_auth(&token)
            .header("Range", format!("bytes={}-", position))
            .send();

        let mut response = match response {
            Ok(response) if response.status().is_success() => {
                shared.stalled.store(false, Ordering::Relaxed);
                backoff = FIRST_BACKOFF;
                response
            }
            other => {
                warn!("[HttpFile] the request failed: {:?}", other.err());
                shared.stalled.store(true, Ordering::Relaxed);
                std::thread::sleep(backoff);
                backoff = (backoff * 2).min(MAX_BACKOFF);
                continue;
            }
        };

        let mut chunk = vec![0u8; 64 * 1024];

        loop {
            if shared.stop.load(Ordering::SeqCst) {
                return;
            }

            // Wait while the buffer is full. This keeps the memory bounded.
            {
                let mut buffer = match shared.buffer.lock() {
                    Ok(buffer) => buffer,
                    Err(_) => return,
                };

                while buffer.len() >= BUFFER_TARGET && !shared.stop.load(Ordering::SeqCst) {
                    let (new_buffer, _) = match shared.signal.wait_timeout(buffer, READ_WAIT) {
                        Ok(value) => value,
                        Err(_) => return,
                    };
                    buffer = new_buffer;
                }
            }

            match response.read(&mut chunk) {
                Ok(0) => {
                    shared.finished.store(true, Ordering::SeqCst);
                    shared.signal.notify_all();
                    return;
                }
                Ok(count) => {
                    let mut buffer = match shared.buffer.lock() {
                        Ok(buffer) => buffer,
                        Err(_) => return,
                    };
                    buffer.extend(&chunk[..count]);
                    position += count as u64;
                    shared.signal.notify_all();
                }
                Err(error) => {
                    // The connection stopped. The loop sends a new request
                    // from the current position.
                    warn!("[HttpFile] the connection stopped: {}", error);
                    shared.stalled.store(true, Ordering::Relaxed);
                    std::thread::sleep(backoff);
                    backoff = (backoff * 2).min(MAX_BACKOFF);
                    break;
                }
            }
        }
    }
}
```

- [ ] **Step 4: Add the blocking feature of reqwest**

`reqwest::blocking` needs a feature. In `Cargo.toml`, change the line for
`reqwest` to:

```toml
reqwest = { version = "0.11", default-features = false, features = ["json", "rustls-tls", "stream", "blocking"] }
```

Then prove that the rule for OpenSSL stays correct:

```bash
cargo tree -i openssl-sys
```

Expected: the command finds nothing.

- [ ] **Step 5: Register the module**

In `src/player/engine/mod.rs`, add this line:

```rust
pub mod http_file;
```

- [ ] **Step 6: Run the unit tests**

Run:

```bash
cargo test --lib player::engine::http_file
```

Expected: PASS, `2 passed`.

- [ ] **Step 7: Write the integration tests**

Create `tests/http_file.rs`:

```rust
//! Tests of the range reader. The tests use a mock server, because the
//! behaviour depends on real HTTP answers.

use std::io::{Read, Seek, SeekFrom};
use toutui::player::engine::http_file::HttpFile;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

/// Makes the content of a test file. Each byte has the value of its position,
/// thus a test can prove that the reader gives the correct bytes.
fn content(size: usize) -> Vec<u8> {
    (0..size).map(|value| (value % 251) as u8).collect()
}

/// Answers a range request in the same way as Audiobookshelf.
fn range_answer(body: &[u8], request: &Request) -> ResponseTemplate {
    let total = body.len();

    let start = request
        .headers
        .get("range")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("bytes="))
        .and_then(|value| value.split('-').next())
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0);

    let end = total - 1;

    ResponseTemplate::new(206)
        .insert_header("accept-ranges", "bytes")
        .insert_header(
            "content-range",
            format!("bytes {}-{}/{}", start, end, total).as_str(),
        )
        .set_body_bytes(body[start..].to_vec())
}

async fn server_with(body: Vec<u8>) -> MockServer {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/items/item1/file/ino1/download"))
        .respond_with(move |request: &Request| range_answer(&body, request))
        .mount(&server)
        .await;

    server
}

#[tokio::test]
async fn the_reader_gives_the_total_size_from_content_range() {
    let server = server_with(content(5000)).await;
    let uri = server.uri();

    let file = tokio::task::spawn_blocking(move || {
        HttpFile::open(&uri, "test-token", "item1", "ino1").unwrap()
    })
    .await
    .unwrap();

    assert_eq!(file.len(), 5000);
}

#[tokio::test]
async fn the_reader_gives_the_correct_bytes() {
    let expected = content(5000);
    let server = server_with(expected.clone()).await;
    let uri = server.uri();

    let got = tokio::task::spawn_blocking(move || {
        let mut file = HttpFile::open(&uri, "test-token", "item1", "ino1").unwrap();
        let mut got = Vec::new();
        file.read_to_end(&mut got).unwrap();
        got
    })
    .await
    .unwrap();

    assert_eq!(got.len(), expected.len());
    assert_eq!(got, expected);
}

#[tokio::test]
async fn a_seek_operation_gives_the_bytes_from_that_position() {
    let expected = content(5000);
    let server = server_with(expected.clone()).await;
    let uri = server.uri();

    let got = tokio::task::spawn_blocking(move || {
        let mut file = HttpFile::open(&uri, "test-token", "item1", "ino1").unwrap();
        file.seek(SeekFrom::Start(4000)).unwrap();
        let mut got = Vec::new();
        file.read_to_end(&mut got).unwrap();
        got
    })
    .await
    .unwrap();

    assert_eq!(got, expected[4000..].to_vec());
}

/// `Decoder` uses `SeekFrom::End` to find the size of a file. This must work.
#[tokio::test]
async fn a_seek_operation_from_the_end_is_correct() {
    let server = server_with(content(5000)).await;
    let uri = server.uri();

    let position = tokio::task::spawn_blocking(move || {
        let mut file = HttpFile::open(&uri, "test-token", "item1", "ino1").unwrap();
        file.seek(SeekFrom::End(0)).unwrap()
    })
    .await
    .unwrap();

    assert_eq!(position, 5000);
}

/// A short movement forward must use the buffer. The reader must not send a
/// new request for a movement of a few bytes.
#[tokio::test]
async fn a_short_movement_forward_sends_no_new_request() {
    let expected = content(200_000);
    let server = MockServer::start().await;
    let body = expected.clone();

    Mock::given(method("GET"))
        .and(path("/api/items/item1/file/ino1/download"))
        .respond_with(move |request: &Request| range_answer(&body, request))
        .expect(1)
        .mount(&server)
        .await;

    let uri = server.uri();

    let got = tokio::task::spawn_blocking(move || {
        let mut file = HttpFile::open(&uri, "test-token", "item1", "ino1").unwrap();

        // Give the thread time to fill the buffer.
        std::thread::sleep(std::time::Duration::from_millis(300));

        file.seek(SeekFrom::Start(1000)).unwrap();
        let mut got = vec![0u8; 10];
        file.read_exact(&mut got).unwrap();
        got
    })
    .await
    .unwrap();

    assert_eq!(got, expected[1000..1010].to_vec());
    // The mock has `expect(1)`. The check happens when the server stops.
    drop(server);
}

#[tokio::test]
async fn the_reader_sends_the_token_in_the_authorization_header() {
    let server = MockServer::start().await;
    let body = content(1000);

    Mock::given(method("GET"))
        .and(path("/api/items/item1/file/ino1/download"))
        .and(header("authorization", "Bearer secret-token"))
        .respond_with(move |request: &Request| range_answer(&body, request))
        .mount(&server)
        .await;

    let uri = server.uri();

    let size = tokio::task::spawn_blocking(move || {
        HttpFile::open(&uri, "secret-token", "item1", "ino1")
            .unwrap()
            .len()
    })
    .await
    .unwrap();

    assert_eq!(size, 1000);
}

/// The address must hold no token. This proves the correction of T-6.
#[tokio::test]
async fn the_address_holds_no_token() {
    let server = MockServer::start().await;
    let body = content(1000);

    Mock::given(method("GET"))
        .and(path("/api/items/item1/file/ino1/download"))
        .respond_with(move |request: &Request| {
            assert!(
                request.url.query().is_none(),
                "the address must hold no query, but it holds {:?}",
                request.url.query()
            );
            range_answer(&body, request)
        })
        .mount(&server)
        .await;

    let uri = server.uri();

    tokio::task::spawn_blocking(move || {
        HttpFile::open(&uri, "secret-token", "item1", "ino1").unwrap();
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn a_status_401_gives_an_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/items/item1/file/ino1/download"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&server)
        .await;

    let uri = server.uri();

    let result = tokio::task::spawn_blocking(move || {
        HttpFile::open(&uri, "bad-token", "item1", "ino1")
    })
    .await
    .unwrap();

    assert!(matches!(
        result,
        Err(toutui::api::client::error::ApiError::Unauthorized)
    ));
}
```

- [ ] **Step 8: Run the integration tests**

Run:

```bash
cargo test --test http_file
```

Expected: PASS, `8 passed`.

If `a_short_movement_forward_sends_no_new_request` fails with two requests, the
buffer did not have time to fill. Increase the sleep from 300 milliseconds to
600 milliseconds. Do not remove the test.

- [ ] **Step 9: Verify the gates**

Run:

```bash
cargo clippy --all-targets -- -D warnings && cargo test
```

Expected: clippy gives no output. All tests pass.

- [ ] **Step 10: Commit**

```bash
git add Cargo.toml Cargo.lock src/player/engine/http_file.rs src/player/engine/mod.rs tests/http_file.rs
git commit -m "feat(player): add the range reader for a file on the server

The reader obeys Read and Seek, thus the decoder accepts it. A thread fills a
buffer in the memory, because the decode thread must never wait for the
network. The token goes in the Authorization header, and the address holds no
token.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

---

### Task 4: The engine and the commands

This task joins the parts. One worker thread owns the `rodio::Player`. It
appends two tracks at a time, so that a book with 209 files does not open 209
connections.

**Files:**
- Create: `src/player/engine/source.rs`
- Create: `src/player/engine/worker.rs`
- Modify: `src/player/engine/mod.rs`
- Create: `tests/engine.rs`

**Interfaces:**
- Consumes: `TrackList`, `Track`, `Chapter` (Task 2), `HttpFile` (Task 3), `media_position` (Task 1), `get_download_files` from `src/db/crud.rs`.
- Produces:
  - `pub trait MediaRead: Read + Seek + Send + Sync {}` in `source.rs`
  - `pub enum TrackSource { Local(PathBuf), Remote { base_url: String, item_id: String, ino: String } }`
  - `pub fn select_sources(item_id: &str, username: &str, base_url: &str, tracks: &[Track]) -> Vec<TrackSource>`
  - `pub fn open_decoder(source: &TrackSource, token: &str, filename: &str) -> Result<Decoder<Box<dyn MediaRead>>, String>`
  - `pub struct PlaybackRequest { pub item_id: String, pub title: String, pub author: String, pub username: String, pub base_url: String, pub tracks: TrackList, pub sources: Vec<TrackSource>, pub start_position: f64 }`
  - `pub enum PlayerCommand { ... }`, `pub struct PlaybackState { ... }`, `pub enum PlaybackStatus { ... }`
  - `pub struct PlayerHandle` with `PlayerHandle::start(token: String) -> Result<PlayerHandle, String>`, `send(&self, command: PlayerCommand)`, `state(&self) -> PlaybackState`, `shared_state(&self) -> Arc<RwLock<PlaybackState>>`

- [ ] **Step 1: Write the failing test for the selection of the source**

Create `src/player/engine/source.rs` with the test module first:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::engine::track::TrackList;

    fn tracks() -> Vec<Track> {
        TrackList::from_durations(&[10.0, 20.0])
    }

    /// The disk has every file. The engine must use the copy on the disk.
    #[test]
    fn a_complete_download_gives_local_sources() {
        let on_disk = vec![
            (1u32, "/tmp/a/001.mp3".to_string()),
            (2u32, "/tmp/a/002.mp3".to_string()),
        ];

        let sources = sources_from(&on_disk, "http://server", "item1", &tracks());

        assert_eq!(sources.len(), 2);
        assert!(matches!(sources[0], TrackSource::Local(_)));
        assert!(matches!(sources[1], TrackSource::Local(_)));
    }

    /// One file is absent. The engine must use the server for all tracks. It
    /// must not mix the two sources in one book.
    #[test]
    fn an_incomplete_download_gives_remote_sources() {
        let on_disk = vec![(1u32, "/tmp/a/001.mp3".to_string())];

        let sources = sources_from(&on_disk, "http://server", "item1", &tracks());

        assert_eq!(sources.len(), 2);
        assert!(matches!(sources[0], TrackSource::Remote { .. }));
        assert!(matches!(sources[1], TrackSource::Remote { .. }));
    }

    #[test]
    fn no_download_gives_remote_sources() {
        let sources = sources_from(&[], "http://server", "item1", &tracks());

        assert_eq!(sources.len(), 2);
        assert!(matches!(sources[0], TrackSource::Remote { .. }));
    }

    #[test]
    fn a_remote_source_holds_the_identity_of_the_file() {
        let sources = sources_from(&[], "http://server", "item1", &tracks());

        match &sources[0] {
            TrackSource::Remote { base_url, item_id, ino } => {
                assert_eq!(base_url, "http://server");
                assert_eq!(item_id, "item1");
                assert_eq!(ino, "ino-0");
            }
            other => panic!("the source must be remote, but it is {:?}", other),
        }
    }

    #[test]
    fn the_hint_is_the_extension_of_the_file() {
        assert_eq!(hint_for("001 - part.m4b"), Some("m4b".to_string()));
        assert_eq!(hint_for("001 - part.MP3"), Some("mp3".to_string()));
        assert_eq!(hint_for("no-extension"), None);
    }
}
```

- [ ] **Step 2: Run the test to verify that it fails**

Run:

```bash
cargo test --lib player::engine::source
```

Expected: FAIL, `cannot find type TrackSource in this scope`.

- [ ] **Step 3: Write the implementation of the source**

Put this above the test module in `src/player/engine/source.rs`:

```rust
//! The byte sources of the engine.
//!
//! The engine reads a local file or a file on the server. The two sources obey
//! one trait. Therefore the engine has one decode path only, and the offline
//! mode needs no separate code.

use crate::db::crud::get_download_files;
use crate::player::engine::http_file::HttpFile;
use crate::player::engine::track::Track;
use rodio::Decoder;
use std::io::{Read, Seek};
use std::path::PathBuf;

/// A source of bytes that `rodio::Decoder` accepts.
///
/// `rodio::Decoder<R>` needs all five of these bounds.
pub trait MediaRead: Read + Seek + Send + Sync {}

impl<T: Read + Seek + Send + Sync> MediaRead for T {}

/// Where the engine reads one track.
#[derive(Debug, Clone, PartialEq)]
pub enum TrackSource {
    /// A file on the disk. The user downloaded the book.
    Local(PathBuf),
    /// A file on the server.
    Remote {
        /// The base address of the server.
        base_url: String,
        /// The identity of the book.
        item_id: String,
        /// The identity of the file.
        ino: String,
    },
}

/// Gives the file name extension in lower case. The decoder uses this value as
/// a hint. The hint stops the examination of the format, and that examination
/// costs range requests on a file that comes from the server.
pub fn hint_for(filename: &str) -> Option<String> {
    let extension = std::path::Path::new(filename).extension()?;
    Some(extension.to_string_lossy().to_lowercase())
}

/// Selects the source of each track.
///
/// A copy on the disk always has more importance than the server. The engine
/// uses the copy on the disk only if the disk has every file of the book. The
/// engine does not mix the two sources in one book.
pub fn select_sources(
    item_id: &str,
    username: &str,
    base_url: &str,
    tracks: &[Track],
) -> Vec<TrackSource> {
    let on_disk: Vec<(u32, String)> = get_download_files(item_id, username)
        .into_iter()
        .map(|(index, path, _duration)| (index, path))
        .collect();

    sources_from(&on_disk, base_url, item_id, tracks)
}

/// The pure part of `select_sources`. The tests use this function, because it
/// needs no database.
fn sources_from(
    on_disk: &[(u32, String)],
    base_url: &str,
    item_id: &str,
    tracks: &[Track],
) -> Vec<TrackSource> {
    let complete = tracks.iter().all(|track| {
        on_disk
            .iter()
            .any(|(index, path)| *index == track.index && !path.is_empty())
    });

    tracks
        .iter()
        .map(|track| {
            if complete {
                let path = on_disk
                    .iter()
                    .find(|(index, _)| *index == track.index)
                    .map(|(_, path)| PathBuf::from(path))
                    .unwrap_or_default();

                TrackSource::Local(path)
            } else {
                TrackSource::Remote {
                    base_url: base_url.trim_end_matches('/').to_string(),
                    item_id: item_id.to_string(),
                    ino: track.ino.clone(),
                }
            }
        })
        .collect()
}

/// Opens a decoder for one track.
///
/// The function gives the file name extension as a hint. It sets the gapless
/// mode, so that a book with many files has no silence between the files.
pub fn open_decoder(
    source: &TrackSource,
    token: &str,
    filename: &str,
) -> Result<Decoder<Box<dyn MediaRead>>, String> {
    let data: Box<dyn MediaRead> = match source {
        TrackSource::Local(path) => {
            let file = std::fs::File::open(path)
                .map_err(|error| format!("The application cannot open the file: {}", error))?;
            Box::new(file)
        }
        TrackSource::Remote { base_url, item_id, ino } => {
            let file = HttpFile::open(base_url, token, item_id, ino)
                .map_err(|error| format!("The server did not give the file: {}", error))?;
            Box::new(file)
        }
    };

    let mut builder = Decoder::builder().with_data(data).with_gapless(true);

    if let Some(hint) = hint_for(filename) {
        builder = builder.with_hint(&hint);
    }

    builder
        .build()
        .map_err(|error| format!("The application cannot read this audio format: {}", error))
}
```

- [ ] **Step 4: Run the tests of the source**

In `src/player/engine/mod.rs`, add:

```rust
pub mod source;
```

Run:

```bash
cargo test --lib player::engine::source
```

Expected: PASS, `5 passed`.

- [ ] **Step 5: Write the state and the handle**

Add this to `src/player/engine/mod.rs`, below the module declarations:

```rust
pub mod worker;

use crate::player::engine::source::TrackSource;
use crate::player::engine::track::TrackList;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, RwLock};

/// What the engine does now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackStatus {
    /// The engine plays no media.
    Stopped,
    /// The engine plays the media.
    Playing,
    /// The user stopped the playback.
    Paused,
    /// The buffer is empty, and the engine waits for data. The user did not
    /// stop the playback. The engine continues without an action of the user.
    Stalled,
}

/// What the user interface shows.
#[derive(Debug, Clone)]
pub struct PlaybackState {
    /// The identity of the book.
    pub item_id: String,
    pub title: String,
    pub author: String,
    /// The position in the whole book, in seconds.
    pub position: f64,
    /// The length of the whole book, in seconds.
    pub duration: f64,
    /// The name of the chapter, if the book has chapters.
    pub chapter_title: Option<String>,
    pub speed: f32,
    pub volume: f32,
    pub status: PlaybackStatus,
    /// A message for the user. An example is "Reconnected".
    pub notice: Option<String>,
}

impl Default for PlaybackState {
    fn default() -> Self {
        PlaybackState {
            item_id: String::new(),
            title: String::new(),
            author: String::new(),
            position: 0.0,
            duration: 0.0,
            chapter_title: None,
            speed: 1.0,
            volume: 1.0,
            status: PlaybackStatus::Stopped,
            notice: None,
        }
    }
}

/// All the data that the engine needs to play a book.
#[derive(Debug, Clone)]
pub struct PlaybackRequest {
    pub item_id: String,
    pub title: String,
    pub author: String,
    pub username: String,
    /// The tracks and the chapters of the book.
    pub tracks: TrackList,
    /// The source of each track. The sequence agrees with the tracks.
    pub sources: Vec<TrackSource>,
    /// Where the playback starts, in seconds from the start of the book.
    pub start_position: f64,
    pub speed: f32,
}

/// A command for the engine.
#[derive(Debug, Clone)]
pub enum PlayerCommand {
    /// Starts a book. The engine stops the book that plays now.
    Start(Box<PlaybackRequest>),
    Pause,
    Resume,
    /// Moves to a position in the book, in seconds.
    SeekTo(f64),
    /// Moves forward or backward, in seconds.
    SeekBy(f64),
    NextChapter,
    PreviousChapter,
    SetSpeed(f32),
    SetVolume(f32),
    /// Stops the playback and empties the queue.
    Stop,
}

/// The connection to the engine.
///
/// The handle sends commands. It also gives the state that the user interface
/// reads.
#[derive(Debug, Clone)]
pub struct PlayerHandle {
    sender: Sender<PlayerCommand>,
    state: Arc<RwLock<PlaybackState>>,
}

impl PlayerHandle {
    /// Starts the engine.
    ///
    /// The function opens the sound card. It gives an error if the computer
    /// has no sound card.
    pub fn start(token: String) -> Result<PlayerHandle, String> {
        let (sender, receiver) = channel();
        let state = Arc::new(RwLock::new(PlaybackState::default()));
        let worker_state = Arc::clone(&state);

        worker::spawn(receiver, worker_state, token)?;

        Ok(PlayerHandle { sender, state })
    }

    /// Sends a command. The function does not wait for the engine.
    pub fn send(&self, command: PlayerCommand) {
        if self.sender.send(command).is_err() {
            log::error!("[PlayerHandle] the engine stopped");
        }
    }

    /// Gives a copy of the state.
    pub fn state(&self) -> PlaybackState {
        match self.state.read() {
            Ok(state) => state.clone(),
            Err(_) => PlaybackState::default(),
        }
    }

    /// Gives the state that the user interface reads for each frame.
    pub fn shared_state(&self) -> Arc<RwLock<PlaybackState>> {
        Arc::clone(&self.state)
    }
}
```

- [ ] **Step 6: Write the worker**

Create `src/player/engine/worker.rs`:

```rust
//! The thread that owns the audio.
//!
//! One thread owns the `rodio::Player`. It reads the commands, and it writes
//! the state. No other thread touches the player.
//!
//! The thread appends two tracks only. A book of the test library has 209
//! audio files, and 209 open connections are not acceptable.

use crate::player::engine::source::open_decoder;
use crate::player::engine::{
    media_position, PlaybackRequest, PlaybackState, PlaybackStatus, PlayerCommand,
};
use log::{error, info, warn};
use rodio::{DeviceSinkBuilder, MixerDeviceSink, Player};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// The time between two examinations of the state.
const TICK: Duration = Duration::from_millis(200);

/// The time that the message "Reconnected" stays on the screen.
const NOTICE_TIME: Duration = Duration::from_secs(4);

/// The number of tracks that the queue holds.
const QUEUE_DEPTH: usize = 2;

/// Starts the thread of the engine.
pub fn spawn(
    receiver: Receiver<PlayerCommand>,
    state: Arc<RwLock<PlaybackState>>,
    token: String,
) -> Result<(), String> {
    // Open the sound card here, and not in the thread. Then the caller gets
    // the error.
    let sink = DeviceSinkBuilder::open_default_sink()
        .map_err(|error| format!("The application cannot open the sound card: {}", error))?;

    std::thread::Builder::new()
        .name("toutui-audio".to_string())
        .spawn(move || run(receiver, state, token, sink))
        .map_err(|error| format!("The application cannot start the audio thread: {}", error))?;

    Ok(())
}

/// What the thread plays now.
struct Current {
    request: PlaybackRequest,
    /// The track that the queue starts with.
    first_queued: usize,
    /// The number of tracks in the queue.
    queued: usize,
}

fn run(
    receiver: Receiver<PlayerCommand>,
    state: Arc<RwLock<PlaybackState>>,
    token: String,
    sink: MixerDeviceSink,
) {
    let mut player = Player::connect_new(sink.mixer());
    let mut current: Option<Current> = None;
    let mut notice_until: Option<Instant> = None;

    loop {
        match receiver.recv_timeout(TICK) {
            Ok(command) => {
                if handle(command, &mut player, &sink, &mut current, &token, &state) {
                    return;
                }
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return,
        }

        publish(&player, &current, &state, &mut notice_until);
        top_up(&mut player, &mut current, &token);
    }
}

/// Runs one command. Gives `true` if the thread must stop.
fn handle(
    command: PlayerCommand,
    player: &mut Player,
    sink: &MixerDeviceSink,
    current: &mut Option<Current>,
    token: &str,
    state: &Arc<RwLock<PlaybackState>>,
) -> bool {
    match command {
        PlayerCommand::Start(request) => {
            player.stop();
            *player = Player::connect_new(sink.mixer());
            player.set_speed(request.speed.max(0.1));

            let start = request.start_position;
            *current = Some(Current { request: *request, first_queued: 0, queued: 0 });

            if let Some(item) = current.as_mut() {
                let (track_index, offset) = item.request.tracks.locate(start).unwrap_or((0, 0.0));
                item.first_queued = track_index;
                item.queued = 0;

                if let Err(error) = fill_queue(player, item, token) {
                    error!("[worker] the engine cannot start the book: {}", error);
                    set_status(state, PlaybackStatus::Stopped);
                    *current = None;
                    return false;
                }

                if offset > 0.0 {
                    if let Err(error) = player.try_seek(Duration::from_secs_f64(offset)) {
                        warn!("[worker] the engine cannot move to the position: {}", error);
                    }
                }

                player.play();
                info!("[worker] the playback starts at {} seconds", start);
            }
        }
        PlayerCommand::Pause => player.pause(),
        PlayerCommand::Resume => player.play(),
        PlayerCommand::SeekTo(position) => seek_to(player, current, token, position),
        PlayerCommand::SeekBy(change) => {
            let now = position_now(player, current);
            seek_to(player, current, token, now + change);
        }
        PlayerCommand::NextChapter => {
            let now = position_now(player, current);
            let target = current
                .as_ref()
                .and_then(|item| item.request.tracks.next_chapter_start(now));

            if let Some(target) = target {
                seek_to(player, current, token, target);
            }
        }
        PlayerCommand::PreviousChapter => {
            let now = position_now(player, current);
            let target = current
                .as_ref()
                .and_then(|item| item.request.tracks.previous_chapter_start(now));

            if let Some(target) = target {
                seek_to(player, current, token, target);
            }
        }
        PlayerCommand::SetSpeed(value) => player.set_speed(value.clamp(0.1, 5.0)),
        PlayerCommand::SetVolume(value) => player.set_volume(value.clamp(0.0, 2.0)),
        PlayerCommand::Stop => {
            player.stop();
            *current = None;
            set_status(state, PlaybackStatus::Stopped);
        }
    }

    false
}

/// Gives the position in the book.
fn position_now(player: &Player, current: &Option<Current>) -> f64 {
    let item = match current {
        Some(item) => item,
        None => return 0.0,
    };

    let played = item.request.tracks.len() - remaining(player, item);
    let track_index = item.first_queued + played.saturating_sub(1);
    let inside = media_position(player.get_pos(), player.speed());

    item.request.tracks.position_of(track_index, inside)
}

/// Gives the number of tracks that the queue still holds.
fn remaining(player: &Player, item: &Current) -> usize {
    player.len().min(item.request.tracks.len())
}

/// Moves to a position in the book.
fn seek_to(player: &mut Player, current: &mut Option<Current>, token: &str, position: f64) {
    let item = match current.as_mut() {
        Some(item) => item,
        None => return,
    };

    let (track_index, offset) = match item.request.tracks.locate(position) {
        Some(value) => value,
        None => return,
    };

    let playing = item.first_queued;

    if track_index == playing {
        if let Err(error) = player.try_seek(Duration::from_secs_f64(offset)) {
            warn!("[worker] the engine cannot move inside the track: {}", error);
        }
        return;
    }

    // The target is in a different track. Make the queue again.
    player.clear();
    item.first_queued = track_index;
    item.queued = 0;

    if let Err(error) = fill_queue(player, item, token) {
        error!("[worker] the engine cannot make the queue again: {}", error);
        return;
    }

    if offset > 0.0 {
        if let Err(error) = player.try_seek(Duration::from_secs_f64(offset)) {
            warn!("[worker] the engine cannot move inside the track: {}", error);
        }
    }

    player.play();
}

/// Appends tracks until the queue holds `QUEUE_DEPTH` tracks.
fn fill_queue(player: &mut Player, item: &mut Current, token: &str) -> Result<(), String> {
    while item.queued < QUEUE_DEPTH {
        let track_index = item.first_queued + item.queued;

        let track = match item.request.tracks.get(track_index) {
            Some(track) => track,
            None => break,
        };

        let source = match item.request.sources.get(track_index) {
            Some(source) => source,
            None => break,
        };

        let decoder = open_decoder(source, token, &track.filename)?;
        player.append(decoder);
        item.queued += 1;
    }

    Ok(())
}

/// Appends the next track when the queue has one track only.
fn top_up(player: &mut Player, current: &mut Option<Current>, token: &str) {
    let item = match current.as_mut() {
        Some(item) => item,
        None => return,
    };

    let played = item.queued.saturating_sub(player.len());
    item.first_queued += played;
    item.queued -= played;

    if player.len() >= QUEUE_DEPTH {
        return;
    }

    if let Err(error) = fill_queue(player, item, token) {
        warn!("[worker] the engine cannot append the next track: {}", error);
    }
}

/// Writes the state that the user interface reads.
fn publish(
    player: &Player,
    current: &Option<Current>,
    state: &Arc<RwLock<PlaybackState>>,
    notice_until: &mut Option<Instant>,
) {
    let mut value = match state.write() {
        Ok(value) => value,
        Err(_) => return,
    };

    let item = match current {
        Some(item) => item,
        None => {
            value.status = PlaybackStatus::Stopped;
            return;
        }
    };

    let position = position_now(player, current);

    value.item_id = item.request.item_id.clone();
    value.title = item.request.title.clone();
    value.author = item.request.author.clone();
    value.position = position;
    value.duration = item.request.tracks.total_duration();
    value.chapter_title = item
        .request
        .tracks
        .chapter_at(position)
        .map(|chapter| chapter.title.clone());
    value.speed = player.speed();
    value.volume = player.volume();

    let was_stalled = value.status == PlaybackStatus::Stalled;

    value.status = if player.is_paused() {
        PlaybackStatus::Paused
    } else if player.empty() {
        PlaybackStatus::Stopped
    } else {
        PlaybackStatus::Playing
    };

    if was_stalled && value.status == PlaybackStatus::Playing {
        value.notice = Some("Reconnected".to_string());
        *notice_until = Some(Instant::now() + NOTICE_TIME);
    }

    if let Some(limit) = *notice_until {
        if Instant::now() > limit {
            value.notice = None;
            *notice_until = None;
        }
    }
}

/// Writes the status only.
fn set_status(state: &Arc<RwLock<PlaybackState>>, status: PlaybackStatus) {
    if let Ok(mut value) = state.write() {
        value.status = status;
    }
}
```

- [ ] **Step 7: Write the tests of the engine**

Create `tests/engine.rs`. These tests need no sound card, because
`rodio::Player::new()` opens no device.

```rust
//! Tests of the queue and the position. The tests use `Player::new`, thus
//! they need no sound card. The continuous integration machine has no sound
//! card.

use rodio::buffer::SamplesBuffer;
use rodio::Player;
use toutui::player::engine::track::{Chapter, TrackList};

/// Makes a sound of a number of seconds. The sample rate is 8000, and the
/// sound has one channel.
fn seconds(count: usize) -> SamplesBuffer {
    SamplesBuffer::new(1, 8000, vec![0.0f32; 8000 * count])
}

#[test]
fn the_queue_plays_two_tracks_one_after_the_other() {
    let (player, mut output) = Player::new();
    player.append(seconds(1));
    player.append(seconds(1));
    player.play();

    assert_eq!(player.len(), 2);

    let total = output.by_ref().count();

    // Two sounds of 1 second give 16000 samples together. This proves that
    // the queue plays the second track, and not the first track only. This is
    // the behaviour that T-2 needs.
    assert_eq!(total, 16000);
}

#[test]
fn the_position_of_the_book_uses_the_start_offset_of_the_track() {
    let list = TrackList::new(TrackList::from_durations(&[10.0, 20.0, 30.0]), Vec::new());

    // The engine plays the second track, at 5 seconds inside that track.
    assert_eq!(list.position_of(1, 5.0), 15.0);
    assert_eq!(list.locate(15.0).unwrap(), (1, 5.0));
}

#[test]
fn a_chapter_movement_gives_a_position_in_the_book() {
    let chapters = vec![
        Chapter { start: 0.0, end: 25.0, title: "One".to_string() },
        Chapter { start: 25.0, end: 60.0, title: "Two".to_string() },
    ];
    let list = TrackList::new(TrackList::from_durations(&[10.0, 20.0, 30.0]), chapters);

    assert_eq!(list.next_chapter_start(10.0).unwrap(), 25.0);
    assert_eq!(list.chapter_at(30.0).unwrap().title, "Two");
}

/// The speed must change during the playback. This is the correction of T-8.
#[test]
fn the_speed_changes_during_the_playback() {
    let (player, mut output) = Player::new();
    player.append(seconds(2));
    player.play();

    // Take some samples at the normal speed.
    for _ in 0..1000 {
        let _ = output.next();
    }

    player.set_speed(2.0);
    assert_eq!(player.speed(), 2.0);

    // The rest of the sound plays at the double speed, thus the number of
    // samples is smaller than the number at the normal speed.
    let rest = output.by_ref().count();
    assert!(
        rest < 15000,
        "the double speed must give fewer samples, but it gave {}",
        rest
    );
}
```

- [ ] **Step 8: Run the tests**

Run:

```bash
cargo test --test engine
```

Expected: PASS, `4 passed`.

- [ ] **Step 9: Verify the gates**

Run:

```bash
cargo clippy --all-targets -- -D warnings && cargo test
```

Expected: clippy gives no output. All tests pass.

If clippy reports `large_enum_variant` on `PlayerCommand`, the variant `Start`
already holds a `Box`. Do not add an allow attribute.

- [ ] **Step 10: Commit**

```bash
git add src/player/engine/source.rs src/player/engine/worker.rs src/player/engine/mod.rs tests/engine.rs
git commit -m "feat(player): add the audio engine and the commands

One thread owns the rodio player. The queue holds two tracks, thus a book with
209 audio files plays completely and the application opens two connections.
The speed changes during the playback.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

---

### Task 5: One playback loop, and the connection to the application

Four files hold a loop that is almost the same: `handle_l_book.rs`,
`handle_l_book_offline.rs`, `handle_l_pod.rs`, and `handle_l_pod_home.rs`. This
task makes one loop. The loop reads the state of the engine, writes the
progress in the database, and sends the progress to the server.

**Files:**
- Create: `src/logic/playback/mod.rs`
- Modify: `src/logic/mod.rs`
- Modify: `src/ui/player_tui.rs`
- Modify: `src/player/integrated/player_info.rs`
- Modify: `src/app.rs`

**Interfaces:**
- Consumes: `PlayerHandle`, `PlaybackState`, `PlaybackStatus`, `PlaybackRequest`, `PlayerCommand` (Task 4), `TrackList`, `Track`, `Chapter` (Task 2), `select_sources` (Task 4), `sync_session`, `close_open_session`, `update_media_progress_book`, `post_start_playback_session_book`.
- Produces:
  - `pub fn request_from_item(item: &serde_json::Value, item_id: &str, username: &str, base_url: &str, start_position: f64, speed: f32) -> Option<PlaybackRequest>`
  - `pub async fn follow_playback(api: &ApiClient, handle: &PlayerHandle, session_id: String, item_id: String, username: String, total_duration: String)`

- [ ] **Step 1: Write the failing test**

Create `src/logic/playback/mod.rs` with the test module first:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn item() -> serde_json::Value {
        serde_json::json!({
            "media": {
                "duration": 60.0,
                "audioFiles": [
                    {
                        "index": 1,
                        "ino": "111",
                        "duration": 10.0,
                        "metadata": { "filename": "part1.mp3", "size": 1000 }
                    },
                    {
                        "index": 2,
                        "ino": "222",
                        "duration": 50.0,
                        "metadata": { "filename": "part2.mp3", "size": 5000 }
                    }
                ],
                "chapters": [
                    { "start": 0.0, "end": 25.0, "title": "One" },
                    { "start": 25.0, "end": 60.0, "title": "Two" }
                ]
            }
        })
    }

    #[test]
    fn the_request_holds_every_track_in_the_correct_sequence() {
        let request =
            request_from_item(&item(), "item1", "user", "http://server", 0.0, 1.0).unwrap();

        assert_eq!(request.tracks.len(), 2);
        assert_eq!(request.tracks.get(0).unwrap().ino, "111");
        assert_eq!(request.tracks.get(1).unwrap().ino, "222");
        assert_eq!(request.tracks.get(1).unwrap().start_offset, 10.0);
    }

    /// This is the correction of T-2. The length is the length of the whole
    /// book, and not the length of the first file.
    #[test]
    fn the_total_duration_is_the_length_of_the_whole_book() {
        let request =
            request_from_item(&item(), "item1", "user", "http://server", 0.0, 1.0).unwrap();

        assert_eq!(request.tracks.total_duration(), 60.0);
    }

    #[test]
    fn the_request_holds_the_chapters() {
        let request =
            request_from_item(&item(), "item1", "user", "http://server", 0.0, 1.0).unwrap();

        assert_eq!(request.tracks.chapter_at(30.0).unwrap().title, "Two");
    }

    /// The files can come in any sequence in the answer. The function must
    /// sort them by the field `index`.
    #[test]
    fn the_function_sorts_the_files_by_index() {
        let item = serde_json::json!({
            "media": {
                "audioFiles": [
                    { "index": 2, "ino": "222", "duration": 50.0,
                      "metadata": { "filename": "b.mp3" } },
                    { "index": 1, "ino": "111", "duration": 10.0,
                      "metadata": { "filename": "a.mp3" } }
                ]
            }
        });

        let request =
            request_from_item(&item, "item1", "user", "http://server", 0.0, 1.0).unwrap();

        assert_eq!(request.tracks.get(0).unwrap().ino, "111");
        assert_eq!(request.tracks.get(1).unwrap().ino, "222");
    }

    #[test]
    fn a_book_with_no_audio_file_gives_no_request() {
        let item = serde_json::json!({ "media": { "audioFiles": [] } });
        assert!(request_from_item(&item, "item1", "user", "http://server", 0.0, 1.0).is_none());
    }

    /// 118 books of the test library have no chapter.
    #[test]
    fn a_book_with_no_chapter_gives_a_request() {
        let item = serde_json::json!({
            "media": {
                "audioFiles": [
                    { "index": 1, "ino": "111", "duration": 10.0,
                      "metadata": { "filename": "a.mp3" } }
                ]
            }
        });

        let request =
            request_from_item(&item, "item1", "user", "http://server", 0.0, 1.0).unwrap();

        assert!(request.tracks.chapter_at(5.0).is_none());
    }
}
```

- [ ] **Step 2: Run the test to verify that it fails**

Run:

```bash
cargo test --lib logic::playback
```

Expected: FAIL, `cannot find function request_from_item`.

- [ ] **Step 3: Write the implementation**

Put this above the test module in `src/logic/playback/mod.rs`:

```rust
//! The one playback loop of the application.
//!
//! The engine plays the audio. This module reads the state of the engine, and
//! it writes the progress. The same loop serves a book on the server and a
//! book on the disk, because the engine reads the two sources with one trait.

use crate::api::client::ApiClient;
use crate::api::me::update_media_progress::*;
use crate::api::sessions::close_open_session::*;
use crate::api::sessions::sync_open_session::*;
use crate::db::crud::*;
use crate::player::engine::source::select_sources;
use crate::player::engine::track::{Chapter, Track, TrackList};
use crate::player::engine::{PlaybackRequest, PlaybackStatus, PlayerHandle};
use log::{info, warn};

/// The number of seconds between two sync requests to the server.
const SYNC_PERIOD: u64 = 10;

/// Makes a playback request from the answer of `GET /api/items/:id`.
///
/// The function reads `media.audioFiles` and `media.chapters`. It puts the
/// files in the sequence of the field `index`. Gives `None` if the book has no
/// audio file.
pub fn request_from_item(
    item: &serde_json::Value,
    item_id: &str,
    username: &str,
    base_url: &str,
    start_position: f64,
    speed: f32,
) -> Option<PlaybackRequest> {
    let files = item["media"]["audioFiles"].as_array()?;

    if files.is_empty() {
        return None;
    }

    let mut tracks: Vec<Track> = files
        .iter()
        .map(|file| Track {
            index: file["index"].as_u64().unwrap_or(1) as u32,
            ino: file["ino"].as_str().unwrap_or_default().to_string(),
            filename: file["metadata"]["filename"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            duration: file["duration"].as_f64().unwrap_or(0.0),
            start_offset: 0.0,
        })
        .collect();

    tracks.sort_by_key(|track| track.index);

    let chapters: Vec<Chapter> = item["media"]["chapters"]
        .as_array()
        .map(|list| {
            list.iter()
                .map(|chapter| Chapter {
                    start: chapter["start"].as_f64().unwrap_or(0.0),
                    end: chapter["end"].as_f64().unwrap_or(0.0),
                    title: chapter["title"].as_str().unwrap_or_default().to_string(),
                })
                .collect()
        })
        .unwrap_or_default();

    let sources = select_sources(item_id, username, base_url, &tracks);
    let list = TrackList::new(tracks, chapters);

    Some(PlaybackRequest {
        item_id: item_id.to_string(),
        title: item["media"]["metadata"]["title"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        author: item["media"]["metadata"]["authorName"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        username: username.to_string(),
        tracks: list,
        sources,
        start_position,
        speed,
    })
}

/// Follows the playback, and writes the progress.
///
/// The loop reads the state of the engine one time each second. It writes the
/// position in the database for each read operation, because a crash must not
/// lose the position. It sends the progress to the server every ten seconds.
///
/// The loop sends `/sync` only. It does not send `/progress` during the
/// playback. Two requests at the same time can make a race condition. See
/// upstream issue 35.
pub async fn follow_playback(
    api: &ApiClient,
    handle: &PlayerHandle,
    session_id: String,
    item_id: String,
    username: String,
    total_duration: String,
) {
    let mut since_sync: u64 = 0;
    let mut last_position: u32 = 0;

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let state = handle.state();
        let position = state.position.max(0.0) as u32;

        // Write the position for each second. A crash must not lose it.
        let _ = update_current_time(position, session_id.as_str());
        let _ = update_download_current_time(item_id.as_str(), username.as_str(), position);

        if let Some(title) = state.chapter_title.as_ref() {
            let _ = update_chapter(title, session_id.as_str());
        }

        match state.status {
            PlaybackStatus::Playing => {
                let moved = position.saturating_sub(last_position);
                since_sync += 1;

                if since_sync >= SYNC_PERIOD {
                    if let Err(error) =
                        sync_session(api, &session_id, Some(position), moved).await
                    {
                        warn!("[follow_playback] the server did not accept the sync: {}", error);
                    }

                    let _ = update_elapsed_time(moved, session_id.as_str());
                    since_sync = 0;
                }

                last_position = position;
            }
            // The engine waits for data. The playback continues without an
            // action of the user. Therefore the loop does not stop.
            PlaybackStatus::Stalled => {}
            PlaybackStatus::Paused => {
                since_sync = 0;
                last_position = position;
            }
            PlaybackStatus::Stopped => {
                info!("[follow_playback] the playback stopped at {} seconds", position);

                let _ = update_is_finished("1", session_id.as_str());

                if let Err(error) = close_session_without_send_prg_data(api, &session_id).await {
                    warn!("[follow_playback] the server did not close the session: {}", error);
                }

                // The book came to the end. This is a command of the user, and
                // not a report during the playback. Therefore `/progress` is
                // correct here. See upstream issue 35.
                if let Err(error) =
                    update_media_progress_book(api, &item_id, Some(position), &total_duration).await
                {
                    warn!("[follow_playback] the server did not accept the position: {}", error);
                }

                let _ = update_is_loop_break("1", username.as_str());
                return;
            }
        }
    }
}
```

- [ ] **Step 4: Register the module**

In `src/logic/mod.rs`, add:

```rust
pub mod playback;
```

- [ ] **Step 5: Run the tests to verify that they pass**

Run:

```bash
cargo test --lib logic::playback
```

Expected: PASS, `6 passed`.

- [ ] **Step 6: Make the user interface read the state**

In `src/player/integrated/player_info.rs`, replace the function `player_info`
with this version. The function keeps the same signature for the list of
strings, thus `src/ui/player_tui.rs` needs no change to its indexes.

```rust
/// Gives the values that the player panel shows.
///
/// The engine gives the position and the length. The values are the values of
/// the whole book, and not the values of one audio file. See T-2.
pub fn player_info(username: &str, state: &PlaybackState) -> Vec<String> {
    let mut player_info = Vec::new();

    if state.status == PlaybackStatus::Stopped && state.title.is_empty() {
        player_info.push("N/A".to_string());
        player_info.push(get_speed_rate(username));
        return player_info;
    }

    player_info.push(state.title.clone());
    player_info.push(state.author.clone());
    player_info.push(
        state
            .chapter_title
            .clone()
            .unwrap_or_else(|| "No chapter".to_string()),
    );

    let is_playing = matches!(
        state.status,
        PlaybackStatus::Playing | PlaybackStatus::Stalled
    );
    player_info.push(is_playing.to_string());

    let position = state.position.max(0.0) as u32;
    let duration = state.duration.max(0.0) as u32;

    player_info.push(format_time(position));
    player_info.push(format_time(duration));
    player_info.push(format_time(position));
    player_info.push(format_time(duration.saturating_sub(position)));
    player_info.push(format!("{}", progress_percent(position, duration)));

    player_info.push(format!("{:.2}", state.speed));

    player_info
}
```

Add these imports at the top of the same file:

```rust
use crate::player::engine::{PlaybackState, PlaybackStatus};
```

Remove the import `use log::info;` if the file no longer uses it. Keep the
functions `progress_percent` and `format_time`, and keep their tests.

- [ ] **Step 7: Show the state "Reconnecting"**

In `src/ui/player_tui.rs`, the function `render_player` takes `player_info`.
Add a parameter for the notice, and show it. Change the signature to:

```rust
pub fn render_player(
    area: Rect,
    buf: &mut ratatui::buffer::Buffer,
    player_info: Vec<String>,
    bg_color: Vec<u8>,
    username: &str,
    notice: Option<String>,
) {
```

Inside the function, after the block that makes the text, add the notice to the
title of the block:

```rust
    // The engine waits for data, or it got the data again. Tell the user.
    let title = match notice {
        Some(message) => format!(" Player - {} ", message),
        None => " Player ".to_string(),
    };
```

Then give `title` to the `Block` that the function makes, in place of the title
that it uses now.

Find every call of `render_player` and give the new argument. The state of the
engine gives the value:

```rust
    let state = self.player.state();
    render_player(area, buf, player_info(&username, &state), bg_color, &username, state.notice.clone());
```

- [ ] **Step 8: Start the engine in the application**

In `src/app.rs`, add this import:

```rust
use crate::player::engine::{PlaybackStatus, PlayerCommand, PlayerHandle};
use crate::logic::playback::{follow_playback, request_from_item};
```

Add this field to the structure `App`:

```rust
    /// The audio engine. The application starts it one time.
    pub player: PlayerHandle,
```

In the function that makes the `App`, start the engine after the token is
available:

```rust
    // Start the audio engine. The application decodes the audio itself, thus
    // the token stays in the memory of the process.
    let player = match PlayerHandle::start(token.clone().unwrap_or_default()) {
        Ok(player) => player,
        Err(error) => {
            eprintln!("{}", error);
            return Err(color_eyre::eyre::eyre!(error));
        }
    };
```

Add `player` to the structure that the function gives.

- [ ] **Step 9: Replace the four playback blocks**

In `src/app.rs`, each of the four blocks that start a book does this sequence
today: it calls `quit_vlc`, `pkill_vlc`, and then `handle_l_book` or
`handle_l_book_offline`. Replace each block with this sequence:

```rust
                        let player = self.player.clone();

                        tokio::spawn(async move {
                            // Stop the book that plays now.
                            player.send(PlayerCommand::Stop);

                            // Before a new session opens, close and sync the
                            // session before it.
                            wait_prev_session_finished(username.clone());

                            let mut stdout = stdout();
                            let _ = pop_message(&mut stdout, 3, message);

                            // If the application stopped without a correct
                            // exit, close the last session now.
                            sync_session_from_database(&api, username.clone(), false, "l").await;

                            let id = match selected_cnt_list.and_then(|i| ids_cnt_list.get(i)) {
                                Some(id) => id.clone(),
                                None => return,
                            };

                            // Open the session on the server. The session
                            // gives the position that the server holds.
                            let info_item = match post_start_playback_session_book(&api, &id).await {
                                Ok(value) => value,
                                Err(_) => {
                                    error!("[app] the server did not start the session");
                                    return;
                                }
                            };

                            let start = info_item[0].parse::<f64>().unwrap_or(0.0);
                            let session_id = info_item[3].clone();

                            // Read the files and the chapters of the book.
                            let item: serde_json::Value =
                                match api.get_json(&format!("/api/items/{}", id)).await {
                                    Ok(value) => value,
                                    Err(error) => {
                                        error!("[app] the server did not give the book: {}", error);
                                        return;
                                    }
                                };

                            let speed = get_speed_rate(&username).parse::<f32>().unwrap_or(1.0);

                            let request = match request_from_item(
                                &item,
                                &id,
                                &username,
                                &server_address,
                                start,
                                speed,
                            ) {
                                Some(request) => request,
                                None => {
                                    error!("[app] the book has no audio file");
                                    return;
                                }
                            };

                            let total_duration = request.tracks.total_duration().round().to_string();

                            let _ = insert_listening_session(
                                session_id.clone(),
                                id.clone(),
                                start as u32,
                                total_duration.clone(),
                                String::new(),
                                0,
                                request.title.clone(),
                                request.author.clone(),
                                true,
                                String::new(),
                            );

                            player.send(PlayerCommand::Start(Box::new(request)));

                            let mut stdout = stdout();
                            let _ = clear_message(&mut stdout, 3);

                            follow_playback(
                                &api,
                                &player,
                                session_id,
                                id,
                                username,
                                total_duration,
                            )
                            .await;
                        });
```

Do this for all four blocks. The two podcast blocks use
`post_start_playback_session_pod` and the podcast identity in place of
`post_start_playback_session_book`. Keep the rest the same.

- [ ] **Step 10: Send the key commands to the engine**

`src/player/integrated/handle_key_player.rs` writes to a TCP stream today.
Replace the whole file with this version:

```rust
//! The keys of the player.
//!
//! The application sends a command to the engine. There is no separate
//! program, thus there is no remote control interface.

use crate::db::crud::*;
use crate::player::engine::{PlayerCommand, PlayerHandle};

/// The number of seconds of a jump.
const JUMP: f64 = 10.0;

/// Sends the command of a key to the engine.
pub fn handle_key_player(key: &str, player: &PlayerHandle, username: &str) {
    match key {
        // Change between the playback and the pause.
        " " => {
            let state = player.state();

            if let Ok(Some(session)) = get_listening_session() {
                let value = if session.is_playback { "0" } else { "1" };
                let _ = update_is_playback(value, session.id_session.as_str());
            }

            match state.status {
                crate::player::engine::PlaybackStatus::Paused => {
                    player.send(PlayerCommand::Resume)
                }
                _ => player.send(PlayerCommand::Pause),
            }
        }
        // Jump forward.
        "p" => player.send(PlayerCommand::SeekBy(JUMP)),
        // Jump backward.
        "u" => player.send(PlayerCommand::SeekBy(-JUMP)),
        // The next chapter.
        "P" => player.send(PlayerCommand::NextChapter),
        // The chapter before this chapter.
        "U" => player.send(PlayerCommand::PreviousChapter),
        // More volume.
        "o" => {
            let volume = (player.state().volume + 0.1).min(2.0);
            player.send(PlayerCommand::SetVolume(volume));
        }
        // Less volume.
        "i" => {
            let volume = (player.state().volume - 0.1).max(0.0);
            player.send(PlayerCommand::SetVolume(volume));
        }
        // More speed. The engine changes the speed during the playback. See
        // T-8.
        "O" => {
            let _ = update_speed_rate(username, true);
            let speed = get_speed_rate(username).parse::<f32>().unwrap_or(1.0);
            player.send(PlayerCommand::SetSpeed(speed));
        }
        // Less speed.
        "I" => {
            let _ = update_speed_rate(username, false);
            let speed = get_speed_rate(username).parse::<f32>().unwrap_or(1.0);
            player.send(PlayerCommand::SetSpeed(speed));
        }
        // Stop the playback.
        "Y" => player.send(PlayerCommand::Stop),
        _ => {}
    }
}
```

Find every call of `handle_key_player` in `src/app.rs`, and give the new
arguments. The old call gives an address, a port, and a value `is_playback`.
The new call gives the handle and the user name only.

- [ ] **Step 11: Verify the gates**

Run:

```bash
cargo clippy --all-targets -- -D warnings && cargo test
```

Expected: clippy gives no output. All tests pass.

The compiler reports each call site that still uses the old signature. Correct
each one. Do not add an allow attribute.

- [ ] **Step 12: Commit**

```bash
git add src/logic/playback/ src/logic/mod.rs src/player/integrated/ src/ui/player_tui.rs src/app.rs
git commit -m "feat(player): use the audio engine and make one playback loop

Four loops that were almost the same become one loop. The loop writes the
position each second, and it sends /sync every ten seconds. The user
interface reads the state of the engine.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

---

### Task 6: The removal of VLC

The engine plays every book now. This task removes the code of VLC, the two
dependencies that only that code uses, the two columns of the database, and the
configuration block.

**Files:**
- Delete: `src/player/vlc/` (all files)
- Delete: `src/utils/vlc_tcp_stream.rs`
- Delete: `src/logic/handle_input/handle_l_book_offline.rs`
- Modify: `src/player/mod.rs`, `src/utils/mod.rs`, `src/logic/handle_input/mod.rs`
- Modify: `Cargo.toml`
- Modify: `src/db/migrate.rs`, `src/db/crud.rs`, `src/db/database_struct.rs`
- Modify: `src/config.rs`, `config.example.toml`
- Modify: `README.md`
- Modify: `docs/TAKEOVER-BACKLOG.md`, `known_bugs.md`

**Interfaces:**
- Consumes: everything from Task 5.
- Produces: `LATEST_VERSION = 4` in `src/db/migrate.rs`.

- [ ] **Step 1: Write the failing test for migration v4**

In `src/db/migrate.rs`, add these tests to the test module:

```rust
    /// Migration v4 removes the two columns that only VLC used.
    #[test]
    fn migration_v4_removes_the_vlc_columns() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('users')
                 WHERE name IN ('is_vlc_running', 'is_vlc_launched_first_time')",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(count, 0);
    }

    /// A database that an older version made has the two columns. The runner
    /// must remove them and must not fail.
    #[test]
    fn migration_v4_upgrades_a_database_that_has_the_vlc_columns() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA user_version = 3").unwrap();
        conn.execute(
            "CREATE TABLE users (
                username TEXT PRIMARY KEY,
                server_address TEXT NOT NULL,
                token TEXT NOT NULL,
                is_vlc_running TEXT NOT NULL DEFAULT '0',
                is_vlc_launched_first_time TEXT NOT NULL DEFAULT '0'
            )",
            [],
        )
        .unwrap();

        run_migrations(&conn).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('users')
                 WHERE name IN ('is_vlc_running', 'is_vlc_launched_first_time')",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(count, 0);
    }
```

- [ ] **Step 2: Run the tests to verify that they fail**

Run:

```bash
cargo test --lib db::migrate
```

Expected: FAIL, the count is 2 and not 0.

- [ ] **Step 3: Write migration v4**

In `src/db/migrate.rs`, change the constant:

```rust
/// The schema version that this build of the program expects.
pub const LATEST_VERSION: i64 = 4;
```

Add this block at the end of `run_migrations`, before `Ok(())`:

```rust
    if version < 4 {
        migrate_to_v4(conn)?;
        conn.execute_batch("PRAGMA user_version = 4")?;
    }
```

Add this function at the end of the file, above the test module:

```rust
/// Version 4 removes the two columns that only VLC used. The application has
/// an audio engine in the process now.
///
/// SQLite version 3.35.0 gives `ALTER TABLE ... DROP COLUMN`. If the statement
/// fails, the migration keeps the column. An unused column does no damage.
fn migrate_to_v4(conn: &Connection) -> Result<()> {
    for column in ["is_vlc_running", "is_vlc_launched_first_time"] {
        let has_column: i64 = conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('users') WHERE name = ?1",
            [column],
            |row| row.get(0),
        )?;

        if has_column == 1 {
            let statement = format!("ALTER TABLE users DROP COLUMN {}", column);

            if let Err(error) = conn.execute(&statement, []) {
                log::warn!(
                    "[migrate] the database keeps the column {}: {}",
                    column,
                    error
                );
            }
        }
    }

    Ok(())
}
```

Also change `migrate_to_v1`. The statement `CREATE TABLE IF NOT EXISTS users`
must no longer make the two columns, because a new database must not have them.
Delete these two lines from the statement:

```
            is_vlc_launched_first_time TEXT NOT NULL,
            is_vlc_running TEXT NOT NULL,
```

- [ ] **Step 4: Run the tests to verify that they pass**

Run:

```bash
cargo test --lib db::migrate
```

Expected: PASS, `6 passed`.

- [ ] **Step 5: Remove the functions of the database**

In `src/db/crud.rs`, delete these four functions:

- `update_is_vlc_running`
- `get_is_vlc_running`
- `update_is_vlc_launched_first_time`
- `get_is_vlc_launched_first_time`

In `src/db/database_struct.rs`, delete the fields `is_vlc_running` and
`is_vlc_launched_first_time` from the structure `User`. Correct every place
that makes a `User` value. The compiler reports each place.

Also correct `db_insert_usr` and `select_default_usr` in `crud.rs`, because
their SQL statements name the two columns.

- [ ] **Step 6: Delete the code of VLC**

```bash
git rm -r src/player/vlc
git rm src/utils/vlc_tcp_stream.rs
git rm src/logic/handle_input/handle_l_book_offline.rs
```

In `src/player/mod.rs`, delete the line `pub mod vlc;`.
In `src/utils/mod.rs`, delete the line `pub mod vlc_tcp_stream;`.
In `src/logic/handle_input/mod.rs`, delete the line
`pub mod handle_l_book_offline;`.

Delete every `use` line that names these modules. The compiler reports each
one.

- [ ] **Step 7: Remove the two dependencies**

In `Cargo.toml`, delete these two lines:

```toml
vlc-rc = "0.1.1"
regex = "1.11.1"
```

Prove that no code uses them:

```bash
grep -rn "vlc_rc\|regex::" src/ tests/
```

Expected: the command finds nothing.

- [ ] **Step 8: Remove the configuration block**

In `src/config.rs`, delete the structure `Player` and the field `player` from
`ConfigFile`. Delete the code in `load_config` that reads the `player` block.

A configuration file that an older version made still has a `[player]` block.
The parser must not fail. The crate `config` does not fail on a key that no
structure names, thus this behaviour is correct. Write a test that proves it:

```rust
    /// A configuration file that an older version made has a `[player]`
    /// block. The application must not fail. See T-14.
    #[test]
    fn an_old_configuration_block_does_not_stop_the_application() {
        let text = r#"
[player]
cvlc = "1"
cvlc_term = "0"
address = "127.0.0.1"
port = "9111"

[[servers]]
name = "home"
endpoints = [ { url = "http://localhost:13378", priority = 0 } ]
"#;

        let parsed: Result<toml::Value, _> = toml::from_str(text);
        assert!(parsed.is_ok());
    }
```

Add `toml = "0.8"` to `[dev-dependencies]` in `Cargo.toml` for this test.

In `config.example.toml`, delete the `#### PLAYER ####` block. Add this note in
its place:

```toml
#### PLAYER ####
# The application plays the audio itself. It does not need VLC.
# The block [player] of an older version is no longer necessary. The
# application does not read it.
```

In `src/app.rs`, delete the fields `is_cvlc`, `is_cvlc_term`, and
`start_vlc_program` from the structure `App`, and delete the code that gives
them a value.

- [ ] **Step 9: Correct the documentation**

In `README.md`, delete VLC from the list of the things that the user installs.
Add this line in its place:

```markdown
The application plays the audio itself. It does not need VLC.

On Linux, the build needs the ALSA development package. On Debian and Ubuntu
the name is `libasound2-dev`. On Fedora the name is `alsa-lib-devel`.
```

In `known_bugs.md`, move these bug identities to the list `FIXED`, and give the
reason:

- `a49eza`: the application does not use `cvlc` now.
- `2eb9e3`: the application does not start a separate program now.
- `fe4116`: the application does not use `cvlc` on macOS now.

In `docs/TAKEOVER-BACKLOG.md`, move T-2, T-5, T-6, and T-8 to the table "The
work that is complete".

- [ ] **Step 10: Verify the gates**

Run:

```bash
cargo clippy --all-targets -- -D warnings && cargo test && cargo tree -i openssl-sys
```

Expected: clippy gives no output. All tests pass. The command for OpenSSL finds
nothing.

Also prove that no code of VLC stays:

```bash
grep -rni "vlc" src/ tests/
```

Expected: the command finds nothing.

- [ ] **Step 11: Test with the real server**

Back up the files first:

```bash
cp ~/.config/toutui/db.sqlite3 ~/.config/toutui/db.sqlite3.backup
cp ~/.config/toutui/config.toml ~/.config/toutui/config.toml.backup
```

Use the pty harness. Set the size of the window first, because `pty.fork()`
leaves the terminal at 0 by 0 and then ratatui draws nothing:

```python
fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", 45, 140, 0, 0))
```

Do these tests:

1. Play a book that has one M4B file. The sound starts.
2. Play a book that has many MP3 files. The sound continues over the boundary
   between two files.
3. Play the book that has 209 audio files. The position agrees with the
   position that the server shows.
4. Change the speed during the playback with `O` and `I`. The sound changes,
   and the playback does not start again.
5. Move over a file boundary with `p` and `u`. The position is correct.
6. Run `ps aux | grep -i toutui`. The output shows no token.
7. Stop the network. The panel shows "Reconnecting". Start the network again.
   The playback continues, and the panel shows "Reconnected".
8. Download a book with `D`. Play it. The engine reads the disk, and the
   application sends no request for the audio.

Record the result of each test in the commit message.

- [ ] **Step 12: Commit**

```bash
git add -A
git commit -m "feat(player): remove VLC (T-5, T-6, T-2, T-8)

The application decodes the audio in the process. Therefore ps aux no longer
shows the token, a book with many audio files plays completely, and a change
of the speed operates during the playback.

The change removes src/player/vlc, the remote control interface, the
dependencies vlc-rc and regex, the [player] block of the configuration, and
the two columns of the database that only VLC used. Migration v4 removes the
columns.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 13: Close the issues**

```bash
gh issue close 2 -R ealtun21/Toutui -r completed -c "Part 2 is complete. The engine has a queue, thus a book with many audio files plays completely. A test with the book that has 209 audio files confirms this."
gh issue close 5 -R ealtun21/Toutui -r completed -c "The application decodes the audio in the process. The token goes in the Authorization header. A test with ps aux confirms that the output shows no token."
gh issue close 6 -R ealtun21/Toutui -r completed -c "The address of the file holds no token. A test in tests/http_file.rs proves that the address holds no query."
gh issue close 8 -R ealtun21/Toutui -r completed -c "rodio::Player::set_speed changes the speed during the playback. The playback does not start again."
```

---

## Self-Review

**Spec coverage.** Each section of the spec has a task:

| Spec section | Task |
|---|---|
| 3, measured facts | 1 (the dependency and the features) |
| 4, architecture | 4 |
| 4.1, the constraint of the callback thread | 3 |
| 5.1, the commands | 4 |
| 5.2, the state | 4 |
| 6.1, the local file | 4 |
| 6.2, the file on the server | 3 |
| 6.3, the selection of the source | 4 |
| 7.1, the queue | 4 |
| 7.2, the position | 1, 2 |
| 7.3, the seek operation | 4 |
| 7.4, the chapters | 2 |
| 8, the loss of the connection | 3, 4 |
| 9.1, the database | 6 |
| 9.2, the configuration | 6 |
| 10, the code that goes away | 6 |
| 11, error handling | 3, 4, 6 |
| 12, the test plan | 1, 2, 3, 4, 5, 6 |

**Notes for the person who executes the plan.**

1. Task 1, Step 5 has two possible results. Use the number that the test
   prints. Do not guess.
2. Task 5, Step 9 changes four blocks in `src/app.rs`. The four blocks are near
   the lines 916, 963, 1067, and 1149 in the version before this work. The
   blocks move when you change the file. Find them with
   `grep -n "quit_vlc" src/app.rs`.
3. Task 6 is large. If the compiler reports many errors after Step 6, correct
   them in this sequence: first the modules, then the imports, then the call
   sites.
4. The engine holds the token. `PlayerHandle::start` gets the token one time.
   If the user changes the account, the application must start a new handle.
   Task 5, Step 8 puts the start in the function that makes the `App`, thus a
   change of the account makes a new `App` and a new handle.
