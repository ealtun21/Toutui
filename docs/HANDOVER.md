# The handover of 2026-08-11 (the seventh session of that day)

This document is for the next session. It says what is done, what is open, and the
traps that cost real time. Read `docs/TAKEOVER-BACKLOG.md` for the evidence of each
item, and `docs/T-24-coverage.md` for the comparison with the server.

**The newest release is v0.7.33**, and the items T-66 to T-73 belong to this
session. T-47 to T-65 belong to the session before it.

## The state

`main` is clean and pushed, and `v0.7.33` is tagged. Every gate passes:

```
nice -n 19 ionice -c 3 cargo clippy --all-targets -j 16 -- -D warnings
nice -n 19 ionice -c 3 cargo fmt --check
ALSA_CONFIG_PATH=<a real null asound file> nice -n 19 ionice -c 3 cargo test -j 16
    # 838 tests pass, 17 carry #[ignore], 37 binaries
cargo tree -i openssl-sys                # finds nothing
cargo tree -i cc                         # finds libsqlite3-sys and ring only
```

**Every test of the sandbox passes too.** One run of
`cargo test -- --ignored --test-threads=1` gave 17 of 17. One thread at a time keeps
the requests of the login under the rate limit of the server.

**The fault of one run of ten did not come back.** This session ran the whole suite
**eight** times: three runs beside the work, and five runs one after the other. Every
run gave 820 of 820, and the suite holds 838 tests after T-73. The session before
this one saw one fault of ten runs and did not name it. **Keep the whole output of `cargo test`** at the next such fault: the
name of the test is the whole answer.

Two tests read the books of the survey. Those books stand outside the repository,
therefore give their directory in `TOUTUI_SURVEY_BOOKS`. A run with no such variable
reads `tests/data/alice.epub` only, and it passes.

**The tag `v0.6.6` has no release.** That tag came before the version of
`Cargo.toml`, and the workflow refused it, as it must. The work of that tag is in
v0.6.7. Do not try to publish v0.6.6.

## What this session closed

| Item | What | Keys |
|---|---|---|
| T-66 | **A media that a different client finished leaves Continue Listening**, with no request | `N` |
| T-67 | The cache of the ebooks holds a limit of one gigabyte | `e` |
| T-68 | **The book of xHE-AAC of the user, and four faults of the stream.** T-53 and T-63 close with it | `l` |
| T-69 | **That book plays now**, from a place beside the one that stops ffmpeg of the server | `l` |
| T-70 | A search of the name of an author gives the books of that author | `/` |
| T-71 | The user reads that the cache of the ebooks removed a book | `e` |
| T-72 | `config.toml` holds the limit of that cache, in a block `[reader]` | — |
| T-73 | **The narrators of the library**, in the view of the authors | `v` |
| T-51 | **The decision of the maintainer: Toutui stays GPL.** bookokrat gives ideas only | — |
| T-20 | **The decision of the maintainer: the two builds of C stay.** Both answers need a crate that is not ready | — |

**Six issues of the fork closed with a measurement and no code:** #10 (the reader of
an EPUB book and of a PDF), #17 (Opus), #18 (WMA and AWB through the stream), #22
(the series), #23 (the cover art), and #15 (the login does not fail at the first
attempt). **#24** holds a comment with the state of the comparison, and it stays open
as the umbrella. **#20** stays open, and its comment holds the decision above.
**The items T-27 to T-67 have no issue.**

### The two items to know

**T-66, the shelf of Continue Listening.** T-47 gave the mark of the line the
position of a live message. **The line itself stayed**: a different client finished a
book, and the Home view held its line until the key `R`. `mediaProgress` of
`user_updated` carries the whole account, and two of its values keep a media away
from that shelf: `isFinished` and `hideFromContinueListening`. The program holds
every line of that shelf already, therefore the render makes the lines again and it
asks the server for nothing.

**The fault of the first form of that work, and how the measurement showed it.** The
list held the **identity** of a media. One media stands on two shelves: the book
stood on Continue Listening and on Recently Added together, and both lines went away.
The count of the view fell from 22 lines to 20, and the server gives the second line.
**The list holds the number of the line now.** A pure function and eight tests would
not have found this: the shape of the data of two shelves did.

**T-67, the cache of the ebooks.** The time of the file is the time of the **last
use**, and `std::fs::FileTimes` writes it. Therefore this needed no dependency. The
book that the user reads now never goes away, and the program looks at the limit
after a new book came only.

## The keys of the program

`src/app.rs` holds the key handler, and it is the authority. **`src/ui/keys.rs`
holds the list for the user**, and the key `?` shows it. A new key needs a line in
that file, or the test `every_key_of_the_handler_stands_in_the_list` fails.

## What is open

### 1. The book of xHE-AAC: it plays now (T-68 and T-69)

**T-53 and T-63 close with T-68, and T-69 makes the file play.** The user gave the
real file on 2026-08-11, and a piece of 10 minutes of it stands in the sandbox.

**A wrong answer of that day, and the correction.** The first form of T-68 read the
sentence "Not yet implemented in FFmpeg, patches welcome" and it said that no program
plays that form. The user said "MPV can play xHE-acc but it does it with a lot of
errors so we should be able as well", and they are right: **ffmpeg reads 77 percent of
the frames.** 60 seconds of the file gave 46.2 seconds of audio, and mpv gave 196
lines "Error decoding audio" in 20 seconds. **A sentence of an error is not a
measurement of the whole file.**

What holds now:

- symphonia reads no frame of that form, therefore the program uses the stream of the
  server. That is the shape of T-53, and it needs no decoder.
- ffmpeg of the server cannot **copy** the codec into a transport stream, therefore it
  stops with the code 183 and the server tries again with `-c:a aac` **ten seconds**
  later. No LATM exists at any moment.
- **The place of the media decides if ffmpeg lives.** Two places of the eight of the
  measurement give a frame of NaN to the encoder, ffmpeg stops with the code 234, and
  the server deletes the session. T-69 tries a place beside it, and the file plays.
- The sound holds a hole at each frame that ffmpeg drops. **No client does better
  today**, and the AAC-LC file of the same book is the better answer for a user.

T-68 and T-69 of `docs/TAKEOVER-BACKLOG.md` hold every measurement and the five
faults of the program that they found.

### 2. The work that needs no decision

1. **The list of Continue Listening of a **different** library.** T-66 holds the
   shelf of the library that the user selected. A media of a second library needs no
   work today, because the Home view shows one library.
2. **A view of the settings for the block `[reader]` (T-72).** `config.toml` holds
   `ebook_cache_mb` now, and the user must open the file with an editor. The view of
   the settings shows the values of the program, and it changes none of them.
3. **The peak of the memory of a PDF (T-62).** `Document::load` of `lopdf` reads the
   whole file, therefore a book of 500 megabytes needs a machine of a gigabyte for
   one moment. `MAX_BOOK_BYTES` of 512 megabytes holds that limit. A reader of one
   page at a time needs a different crate, and no such crate of pure Rust exists.
4. **The table of section 4 of `docs/T-24-coverage.md` holds rows that are old.**
   T-73 changed one row of `No` to `Yes`, and the rows of the live messages, of the
   series, and of the sessions of the server say `No` or `Half` for work that landed.
   **Read the code before you take a row of that table.**
5. **A view of the search that holds its own titles (T-70).** The lines of that view
   come from the lists of the library, therefore a book that the program did not load
   gives no line. `get_all_books` reads every page at the start, and 500 pages of 500
   items hold 250000 items, therefore no library of a user meets this today.

`docs/T-24-coverage.md` section 6 names every function that the program must **not**
have, with the reason. Read it before you take a row of the table that says `No`.
**A measurement changed one row of that section on 2026-08-11:** a PDF of text holds
its text, therefore T-54 reads it. A CBZ stays outside.

### 3. The decisions that the maintainer made, and what they bind

**Toutui stays `GPL-3.0-or-later` (T-51).** bookokrat is AGPL. **A person may read
that project for the idea of a function, and they must then write their own code.**
No line, and no near copy: write the code from the rule, and not from the text of the
code. A commit that comes from such an idea **names that project and the idea**. The
rule of T-20 holds too, therefore `mupdf` and every library of the machine stay
outside.

**`libsqlite3-sys` and `ring` stay (T-20).** The binary of the release needs no
library of the system. `ring` carries the TLS of every request and `libsqlite3-sys`
reads the encrypted token, therefore an alpha version and a pre-release are not
acceptable in those two places. Look again when `turso` is a release and
`rustls-rustcrypto` is beta or better.

## The harness: where the time went, with numbers

The maintainer said on 2026-08-11 that the tests take much time and that they make
the machine slow. **This session measured every part of the work.** The answer is not
the one that the session expected: `cargo test` is not the slow part.

| The work | The time | The measurement |
|---|---|---|
| A cold build of every test | **21 s** | `cargo test --no-run -j 16` after `cargo clean --profile dev` |
| A build after **one** edit of the library | **6 s** | `touch src/logic/authors.rs`, then the same command |
| The run of every test | **18 s** | `cargo test -j 16`, every binary built |
| **One measurement in tmux** | **20 s to 60 s** | The session made about 30 of them |
| One restart of the sandbox | 13 s | `podman restart abs-test` |

### 1. The first frame comes after 673 milliseconds, and this session waited 17 seconds

Every measurement of this session began with `sleep 17` before the first key. A
measurement of 2026-08-11 polled the screen every 500 milliseconds instead:

```
the Home view came after 673 ms
```

**That is 16 seconds of waiting for each measurement, and this session made about 30
of them: about eight minutes of nothing.** Every `sleep` after a key holds the same
fault.

**The answer stands in the repository now: `docs/harness/drive.sh`.** Give that file
to your shell, and drive the program with it:

```bash
source docs/harness/drive.sh
start_the_program                      # it comes back at the first frame
press a; wait_for "The authors ["
press v; wait_for "The narrators ["
the_screen | head -20
the_log "\\[authors\\]" 3
stop_the_program
```

Every function polls, and every poll holds a timeout of 30 seconds. A poll that gives
up writes the reason and the first twelve lines of the screen, therefore a
measurement that fails says why.

**The measurement of that file, 2026-08-11:**

| The work | With `sleep` | With `drive.sh` |
|---|---|---|
| The start of the program | 17 s | **0.57 s** |
| A sweep of three views | 27 s | **1.6 s** |

`start_the_program "TOUTUI_EBOOK_CACHE_BYTES=200000"` gives more variables of the
environment to the program, and `SESSION`, `COLUMNS_OF_THE_SCREEN`, and `TIMEOUT`
change the harness. `wait_while` waits while a text stands on the screen: a message
of the program lives six seconds, and a measurement of the message after it must wait
for the first one to go.

### 2. Two tests hold 12.7 seconds of the 18 seconds of the run

| The test | The time | Why |
|---|---|---|
| `the_position_survives_a_playback_that_does_not_start` | 8.01 s | Four `tokio::time::sleep` of 3500, 1500, 1500, and 1500 milliseconds |
| `playback_ownership` | 4.71 s | Four more of them |
| Every other binary, together | about 5 s | — |

**Do not change those eight sleeps without care, and read this first.** Some of them
wait for a state of the engine, and a poll of that state gives the same measurement at
once. **Others wait for a fault to appear**, and a poll of the value that the test
wants is then a **false pass**.

The sleep of 3500 milliseconds of
`the_position_survives_a_playback_that_does_not_start` is of the second kind. The loop
of the playback reads the state one time each second, and the test asks: does that
loop write the **wrong** position of an engine that did not start? A poll of
"the position is the position of the user" answers `true` **before the loop ever
ran**, because the row of the database holds that value already. The test would then
pass with no measurement at all.

The correct answer for such a wait: poll for the evidence that **the loop acted** —
one write of the loop, one sync, or one line of the log — and then read the value. The
loop acts one time each second, therefore a poll of that evidence gives about 1.2
seconds in place of 3.5. **A test that measures the absence of a fault must give the
fault the time to appear.**

### 3. `cargo test` runs the binaries one after the other

34 files of `tests/` give 34 binaries, and cargo runs them in sequence: the 18 seconds
are almost the sum of their times. **`cargo-nextest` runs every test of every binary
in one pool of processes.** The run would then take the time of the slowest test, of
about 8 seconds, and 6 seconds after the two sleeps above go away.

`cargo-nextest` is a tool of the machine, and **not** a dependency of the program:
`Cargo.toml` does not change, and the rule of T-20 stays. This machine does not hold
it today. Two properties need a measurement before the change:

- nextest gives **one process for each test**. The tests of a global slot of the
  process (`logic::message`, `logic::live`, `logic::authors`, the cache of T-72) then
  become **more** isolated, and the trap 29 of this document loses its cost. Measure
  that the tests of those slots still pass.
- The tests of the sandbox need `--test-threads=1` for the rate limit of the login.
  nextest holds `--test-threads`, and it needs a group of the tests that run alone.

### 4. The build is not the slow part, and the disk is

`[profile.dev]` holds `debug = "line-tables-only"` since T-64. The build after one
edit takes 6 seconds, therefore a faster linker gives almost nothing: `ld.lld` stands
on this machine, and a measurement of `-C link-arg=-fuse-ld=lld` gave no useful
change. **Do not put a linker in `.cargo/config.toml` of the repository**: a user who
builds with `cargo install --git` and holds no `lld` would then get an error of the
link.

**The lag of the machine comes from the disk.** `target` grew from 2.1 gigabytes to 11
gigabytes inside this session, three times, and each `cargo clean --profile dev` gave
15 gigabytes back. A build of that size writes much, and the machine of the maintainer
answers slowly while it writes. Two answers to measure:

1. `CARGO_TARGET_DIR` on a different disk, or on a `tmpfs` of the memory for the tests.
2. Fewer binaries: 34 files of `tests/` hold 300 megabytes each. **Some of those files
   stand alone for a reason** — a test that writes `XDG_CONFIG_HOME` or
   `XDG_DATA_HOME` must be alone in its binary (the trap 29). Every other file can
   join a file of a group, and each join takes one binary of 300 megabytes away.
   `cargo-nextest` makes the reason of the trap 29 go away, therefore do the two
   works in that sequence.

## The traps that cost time

### Of the program and of the server

1. **`ALSA_CONFIG_PATH=/dev/null` stops the real program.** It is correct for
   `cargo test`, because no test opens a sound device. A real run writes "The pool
   has 1 address(es)" and then draws nothing. Give it a real file:
   ```
   </usr/share/alsa/alsa.conf>
   pcm.!default { type null }
   ctl.!default { type null }
   ```
2. **The device `null` plays a book of 30 minutes in two or three seconds.** A test
   of a view that needs a media that plays must press the key inside that time. One
   write of two keys does the work: `l` starts the playback and the space pauses it.
   **A key that comes too early goes to the playback that came before it**: the
   fallback of T-53 starts a second playback about 100 milliseconds after the first
   one.
3. **A playback of a few seconds leaves no session on the server.** The server drops
   a session whose `timeListening` is 0, and the sync period is ten seconds.
   **Read `podman logs abs-test` instead:** the server writes "Starting session for
   user ..." for each `POST /api/items/:id/play`.
4. **`currentTime` comes as a text, and not as a number,** in
   `GET /api/me/progress/:id`.
5. **`PATCH /api/me/progress/:id` with `isFinished` false writes the position back to
   the start, and it ignores every other field of the same request.** A body of
   `{"isFinished":false,"progress":0.5,"currentTime":900}` gives `200`, and the record
   then holds `progress` 0. A media of progress 0 stands on no shelf of Continue
   Listening. **Two requests do the work.** See T-66.
6. **A 404 of a part of a stream is not a media that is absent.** `classify_status`
   gives "The server does not have this item" for a 404, and that sentence is false
   for a part of a stream: ffmpeg writes a part when it made that part. **A stream
   that the server deleted answers 404 for ever**, and one request of the playlist
   tells the two conditions apart. See T-68.
7. **The place of the transcode of the server comes from the position of the user**,
   and **not** from the part that the client asks for. Its log says "Starting Stream
   at startTime 4:52 (User startTime 5:22)", and it takes 30 seconds of that position
   as a pre-roll. A client that moves the part it asks for changes nothing: the
   program must write the position first. See T-69.
8. **A message of the screen lives six seconds** (`message::LIFE`). A capture of the
   screen after that time gives no message, and the program is correct. See T-71.
9. **A sentence of an error of ffmpeg is not a measurement of the whole file.**
   "Not yet implemented in FFmpeg, patches welcome" came 195 times in 20 seconds of a
   file of xHE-AAC, and ffmpeg still gave 46.2 seconds of audio of 60. **Measure the
   output**, and not the lines of the fault: `ffmpeg -i <file> -f wav out.wav` and
   then `ffprobe` of the length. See T-68.
10. **A stream of the server does not open in some milliseconds.** ffmpeg of the
   server makes the first part, and a copy that fails costs ten seconds more. Every
   wait of a stream must hold that time, and `WAIT_FOR_A_FAULT` of 2500 milliseconds
   held it for a **file** only. See T-68.
11. **The row of the message of the screen holds one line.** A message of 200 letters
   loses its end in a terminal of 160 columns. Hold a message at 150 letters or
   fewer. See T-68.
12. **The server takes a name of a field that does not exist.** `?sort=bogus.field`
   gives `200` and an unspecified sequence. Measure a field before you offer it.
13. **`items` of `GET /api/me/listening-stats` is a map, and not a list.**
14. **`GET /api/podcasts/:id/checknew` gives an empty list for a podcast that came one
   second before.** It compares with the time of the last examination.
15. **`convert_seconds` rounds to the minute.** It is wrong for a place in a media.
   Use `clock` of `src/utils/convert_seconds.rs`.
16. **`topGenres` of `GET /api/stats/year/:year` names its value `genre`**, and
    `topAuthors` and `topNarrators` name it `name`.
17. **The lists of the narrators and of the genres stay empty until a session comes
    after the metadata.** The server keeps a copy of the metadata inside each
    session.
18. **The first page of `GET /api/me/listening-sessions` is the page 0**, and a page
    after the last page gives `200` and an empty list.
19. **`quick-xml` 0.41 gives an entity as its own event `GeneralRef`**, and not
    inside the text. A reference makes no text node of the tree of a web page,
    therefore `cfi::Walk` must not count it as one.
20. **A comparison of two lists of texts must not read the text.** A book holds the
    word "very" two hundred times. Read the two lists together, in the sequence of
    the document.
21. **`GET /api/libraries/:id/filterdata` holds no tag.** `GET /api/tags` gives them,
    and a filter of `tags.<base64>` works. A scan of the library changes nothing.
    See T-60.
22. **The rate limit of the login is 40 requests of 600 seconds.** A run of every
    test of the sandbox reaches it, and the test then says "the answer must hold a
    token". Read `podman logs abs-test` for the line of the rate limiter, and run
    those tests with `--test-threads=1`.
23. **A test that changes data must write a value that differs.** The test of the
    live messages wrote the same subtitle at every run: the server saw no change at
    the second run, therefore it sent no message and the test waited for nothing.
24. **A value of the state that belongs to one playback must name that playback.**
    The engine clears the name of a file with no decoder when it starts a playback,
    and the command of the start is not immediate. `playback_of_the_fault` holds the
    identity. See T-53.
25. **`reqwest::blocking` stops the program inside a task of tokio.** That client
    makes a runtime of its own, and a runtime that goes away inside an asynchronous
    context gives "Cannot drop a runtime in a context where blocking is not
    allowed". The engine is a thread, therefore the real program is correct. **A
    test of such a reader must use `std::thread::spawn`.**
26. **A value that comes from the view of the user must come before the view
    changes.** `open_the_ebook` read the title of the media after it set the view to
    the reader, and the answer was then always nothing.
27. **One media stands on more than one shelf of the Home view.** A list that names a
    media by its identity therefore changes every line of that media. **The number of
    the line is the key of a rule of one shelf.** See T-66.
28. **The reader keeps the book of the session while the user reads it.** The key `h`
    and a second `e` give the book with no call of `get_the_ebook`, therefore the
    time of the file does not change inside one run of the program. A measurement of
    the cache needs a second **run**. See T-67.

### Of the harness and of the machine

1. **A fixed `sleep` is the largest waste of a session.** The first frame of the
   program comes after **673 milliseconds**, and this session slept 17 seconds before
   each of about 30 measurements. **Poll for a marker of the screen, with a timeout.**
   See the section of the harness.
2. **A screen of your own is not a terminal. Use tmux.** A hand-written model of the
    screen mangled the line that the user selected, and it kept the old text of a
    line that the program wrote again. Both times the program was correct.
    ```
    tmux new-session -d -s check -x 160 -y 45 "<the program>"
    sleep 9; tmux capture-pane -p -t check        # -e keeps the colours
    ```
3. **A frame of the program comes by itself.** A measurement of a live message needs
    **no key at all**: the three frames of T-66 came with no key press. A key press
    inside the measurement moves the selection, and it then looks like a fault of the
    rule of the selection.
4. **A pseudo terminal must answer TWO questions**, and not one: `ESC [ 5 n` with
    `ESC [ 0 n` for `ratatui-image`, and `ESC [ 6 n` with `ESC [ 1 ; 1 R` for
    crossterm inside `terminal.clear()`. With no answer the program stops with "The
    cursor position could not be read within a normal duration", and that message
    never reaches the log file.
5. **ratatui writes the cells that changed only.** A change of one number therefore
    gives two bytes in the stream of the terminal. A move of the list (`j` and then
    `k`) makes the program write whole lines again.
6. **A key of a login that comes too early goes to the application.** The login
    examines the address with a request, therefore the next field is not ready at
    once: the password `claudetmp` went to the application as the keys `c`, `l`,
    `a`, `u`, `d`, `e`, and `e` opened the reader. **Log in one time, and then reuse
    the database of that isolated `XDG_CONFIG_HOME`.**
7. **An isolated `XDG_CONFIG_HOME` needs two files before the program starts.**
    `config.toml`, and `.env` with `TOUTUI_SECRET_KEY=<something>`.
8. **A test that sets `XDG_CONFIG_HOME` or `XDG_DATA_HOME` must be alone in its
    binary**, and every test of a global slot must stand in one function.
    `logic::message` gave a fault of one run of three before its two tests became
    one, and `tests/the_cache_of_the_ebooks.rs` holds one function for that reason.
9. **`target/debug` grew to 221 gigabytes and the disk of the maintainer became
    full.** A test binary holds 300 megabytes, `cargo test` makes 37 of them, and
    cargo keeps the binary of every build that came before. Look at `du -sh target`,
    and run `cargo clean --profile dev` at the end of a session. See T-64.
10. **The container of the sandbox lives longer than the session.** `podman ps -a`
    with `| head` did not show it, and `podman run` then said "the container name is
    already in use". **`podman start abs-test` gives the server back with every book
    of the session before**, therefore no session needs to make the data again.
11. **`ALSA_CONFIG_PATH` with a null device does not silence the real program.** It
    is correct for `cargo test`, because no test opens a sound device. A run of
    2026-08-11 played real sound through the sound card of the maintainer with that
    variable set, and the sound was **not smooth**: a half of a configuration of ALSA
    is worse than none. **Ask the user before a measurement that plays**, and use the
    real device: a null device hides the faults of the real path.
12. **A transcode of the server that dies leaves the server in a bad state.** Every
    new session of that media then answers "No Segments", and the log holds "Failed
    checking files" every two seconds for ever. `podman restart abs-test` is the
    answer, and a measurement of such a media must start from a server that came up
    now. See T-68.
13. **`playwright` wants its own browser:**
    `play.chromium.launch(executable_path="/usr/bin/chromium")`.

## The shapes that the next work should follow

- **A slot between the work and the screen.** The render is not asynchronous.
  `logic::live`, `logic::stats`, `logic::bookmarks`, `logic::authors`, and
  `logic::message` all hold that shape: a task or a key writes, and the render takes
  it at the next frame. A function that needs no `&mut App` reaches every caller.
- **The render may change the state, and it must do that work one time.**
  `take_the_media_that_left_away` of T-66 compares two small lists at every frame,
  and it makes the lines again at a change only. `Widget for &mut App` gives the
  frame that permission.
- **A rule that needs no request is better than a request.** T-66 holds every line of
  the shelf already, therefore the program never asks the server after a live
  message. The sync of the playback of the program makes one such message every ten
  seconds, and each costs one comparison.
- **A view says why it holds no line. A message of one row says nothing.** The view
  of the bookmarks held the right shape, and the view of the chapters holds it now.
- **A rule of a loop belongs in a pure function with a name.**
  `the_fault_stops_the_playback`, `the_queue_goes_on`, `wait_after_the_faults`,
  `is_for_the_screen`, `without_the_media_that_left`, and `the_ebooks_that_must_go`
  are small and pure, and a test holds each of them to the measurement that made it.
- **A pure function with a test can still hold a wrong key.** Eight tests of
  `without_the_media_that_left` passed while the caller named the media by its
  identity. **The shape of the real data found it.** Make the data of two shelves
  exist, and then look at the screen.
- **A test may read the source of the program.**
  `every_key_of_the_handler_stands_in_the_list` reads `src/app.rs` with
  `include_str!` and it finds every key of the handler in the list of the keys.
- **Measure the dependency before you accept that you need one.** socket.io needed
  none: the second transport of that protocol is plain HTTP. Every codec of the
  server needed none: the stream of HLS is a playlist of text and packets of 188
  bytes. The time of the last use of a file needed none: `std::fs::FileTimes`.
- **Show the fault with the old binary.** `git checkout -- <the file>` and a build in
  a different `CARGO_TARGET_DIR` give the old program beside the new one.
- **Make the data of the fault in the sandbox.** A book of one MP3 file and one WMA
  file gives the shape of a book that no decoder reads. `docs/TEST-SERVER.md` holds
  the commands of that book and of a PDF with pictures.
- **A message of the server can hold a secret.** `user_updated` carries a new token
  of the user. The log holds the name of a message, and never the body.
- **A sweep of every view finds what a test does not.** One run in tmux pressed the
  key of each view and counted the lines of text of each screen. Two views of fifteen
  answered with nothing, and one of them was a real fault.

## The rules that do not change

- Every document, comment, and text for the user in ASD-STE100 simplified technical
  English. Short sentences, active voice, one instruction per sentence.
- No crate that needs a library of the system. `cargo tree -i openssl-sys` must find
  nothing. `libsqlite3-sys` and `ring` are the two known builds of C, and the
  decision of 2026-08-11 keeps them.
- No test may need the network. A test that needs the sandbox carries `#[ignore]`
  and says how to run it. Seventeen such files exist now, and no test holds a path of
  this machine.
- Never write to `AlbanDAVID/Toutui`. It is archived. AlbanDAVID stays credited in
  the README, in the LICENSE, in `Cargo.toml`, and in the settings screen. **`gh`
  resolves to that repository by default in this clone**, therefore give
  `-R ealtun21/Toutui` to every `gh` command.
- Show a fault before you correct it.
- Tag, push, and go on. Do not wait for continuous integration.
- **The release builds `--locked`.** A bump of the version must hold the new
  `Cargo.lock` in the same commit: run `cargo build` after the bump, and see that
  `git status` is clean before the commit and the tag. The tag of v0.7.22 held
  `Cargo.toml` 0.7.22 and `Cargo.lock` 0.7.21, and it had to move.
- **Run every cargo command under `nice -n 19 ionice -c 3` with `-j 16`.** The
  machine has 32 cores, and the user tests the program while the tests build.
- The address of the server of the user must stay outside this repository, and the
  account of the user too.

## The prompt for the next session

> Continue the Toutui takeover, and write the next version. Repo:
> `/home/nyverino/Documents/Toutui` (ealtun21/Toutui, branch main). Maintained fork
> of the archived AlbanDAVID/Toutui. Newest release **v0.7.33**; `Cargo.toml` is at
> 0.7.33, so the next release bumps it first — the workflow refuses a tag that
> disagrees with `Cargo.toml`, **and it builds `--locked`, therefore the commit of
> the bump must hold the new `Cargo.lock`**.
>
> Read `docs/HANDOVER.md` first: the state, the open items, **the section of the
> harness with the numbers of every slow part**, and 41 traps that cost real time.
> Then `docs/TAKEOVER-BACKLOG.md` (the evidence of every item; T-66 to T-73 are the
> newest, and T-66, T-68, and T-69 are the ones to know) and
> `docs/T-24-coverage.md` (**section 6 names what the program must not have, and
> why**; the table of section 4 holds rows that are old).
>
> **Task 1, and it comes before every other task: make the harness fast.** The
> maintainer said that the tests take much time and that they make the machine slow,
> and the session of 2026-08-11 measured every part. `cargo test` is **not** the slow
> part: a build after one edit takes 6 seconds and the run takes 18. The waste stands
> in three places, and the section of the harness holds the numbers:
>
>  1. **Poll, never sleep. `docs/harness/drive.sh` holds that work already** — use
>    it, and add to it. A measurement of 2026-08-11: the start of the program takes
>    **0.57 s** with it and 17 s with a `sleep`, and a sweep of three views takes
>    **1.6 s** and 27 s. Every measurement of the next session must come from that
>    file.
>  2. **Eight fixed sleeps of two test files hold 12.7 seconds of the run of 18
>    seconds** (`the_position_survives_a_playback_that_does_not_start` and
>    `playback_ownership`). **Read the warning of the section of the harness before
>    you change them:** some of those sleeps give a **fault** the time to appear, and a
>    poll of the value that the test wants is then a false pass. Poll for the evidence
>    that the loop acted, and then read the value.
>  3. **`cargo test` runs the 34 binaries one after the other.** Measure
>    `cargo-nextest`: it runs every test in one pool, therefore the run takes the time
>    of the slowest test. It is a tool of the machine and **not** a dependency of the
>    program, therefore the rule of T-20 stays. Measure the tests of the global slots
>    (`logic::message`, `logic::live`, `logic::authors`, the cache of T-72) and the
>    tests of the sandbox that need one thread, and say in the handover what changed.
>
> **The lag of the machine comes from the disk, and not from the processor.** `target`
> grew to 11 gigabytes three times in one session. Look at `du -sh target` often, run
> `cargo clean --profile dev` at the end, and measure `CARGO_TARGET_DIR` on a
> different disk. Do **not** put a linker in `.cargo/config.toml`: a user of
> `cargo install --git` who holds no `lld` would then meet an error of the link.
>
> **Then the work of the program, in the sequence of its value:**
>
> 1. **A sweep of every view in tmux, with the fast harness.** Two faults of the
>    search came from such a sweep on 2026-08-11, and two more came from the sweep
>    before it. A sweep of fifteen views costs one minute with a poll, and eight with
>    a sleep. The key `Escape` of the view of the search closes the program: use `/`
>    again.
> 2. **The rows of section 4 of `docs/T-24-coverage.md` that are old.** T-73 changed
>    one row of `No` to `Yes`, and the rows of the live messages, of the series, and of
>    the sessions of the server name work that landed. **Read the code before you take
>    a row of that table**, and correct the row that you measure.
> 3. **A view of the settings that changes a value of `config.toml` (T-72).** The
>    block `[reader]` holds `ebook_cache_mb`, and the user must open the file with an
>    editor today. A write of that file must keep every comment of the user.
> 4. **Empty the queue of the podcast of the server**, and **the list of the ebooks of
>    an item** (an item can hold more than one ebook, and the program takes one). Both
>    are small, and section 6 forbids neither.
> 5. **The peak of the memory of a PDF (T-62)**, if a user meets it. `MAX_BOOK_BYTES`
>    of 512 megabytes holds the limit, and no crate of pure Rust reads one page at a
>    time.
>
> **The book of xHE-AAC plays now (T-68 and T-69). Do not open T-53 or T-63 again**,
> and do not look for a decoder of Rust: symphonia reads no frame of that form, and
> the stream of the server gives the sound. ffmpeg reads 77 percent of the frames,
> therefore the sound holds a hole at each frame that it drops, and mpv gives the same
> holes. **A sentence of an error of ffmpeg is not a measurement of the whole file:
> measure the output.**
>
> **The two decisions of the maintainer are made.** Toutui stays GPL, and a person may
> read bookokrat for an idea and must then write their own code, and name that project
> in the commit (T-51). `libsqlite3-sys` and `ring` stay (T-20). Do not open either
> again.
>
> **Rules that bind every change.** Run all three gates yourself before each commit,
> under `nice -n 19 ionice -c 3` with `-j 16`: `cargo clippy --all-targets --
> -D warnings`, `cargo fmt --check`, and `cargo test` with `ALSA_CONFIG_PATH`
> pointing at a real null asound file (`/dev/null` hangs the real binary). Baseline:
> **838 tests, 17 with `#[ignore]`, 37 binaries**, tree clean. **A measurement that
> plays sound needs the real device and the permission of the user**: that variable
> does not silence the real program, and a measurement with `curl` against the sandbox
> needs no sound at all.
>
> All prose and user-facing strings in ASD-STE100 simplified technical English. No
> crate needing a system library; `cargo tree -i openssl-sys` must find nothing, and
> `cargo tree -i cc` must find `libsqlite3-sys` and `ring` only. No test may need the
> network — sandbox tests carry `#[ignore]` and run with `--test-threads=1`, because
> the login of the server permits 40 requests of 600 seconds. Never write to
> AlbanDAVID/Toutui, and keep his credit everywhere it appears.
>
> **Show a fault before you fix it, and measure against the sandbox**
> (`docs/TEST-SERVER.md`, podman on `:13399`; `podman start abs-test` gives the server
> back with every book of the sessions before, and `podman restart abs-test` after a
> transcode that died). Make the data exist first, because an empty list shows you no
> shape — **and make it hold the shape that breaks the rule**: T-66 hid behind one
> shelf, and one media of two shelves showed the fault. **Drive the real program
> inside tmux for every view**; a screen of your own writing lies to you, and a frame
> of a live message needs no key at all. Verify with a second program: `podman logs
> abs-test`, a real browser, or `curl`. Tag, push, and keep working; don't wait for
> CI.
>
> The user tests each release as it lands and does not want to be asked before
> publishing a patch. The server of the user is theirs alone: ask before you use it,
> always with an isolated `XDG_CONFIG_HOME`, and never write its address or its
> account into this repository. Measure against the sandbox instead.
