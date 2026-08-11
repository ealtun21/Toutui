# The handover of 2026-08-11 (the sixth session of that day)

This document is for the next session. It says what is done, what is open, and the
traps that cost real time. Read `docs/TAKEOVER-BACKLOG.md` for the evidence of each
item, and `docs/T-24-coverage.md` for the comparison with the server.

**The newest release is v0.7.26**, and the items T-47 to T-65 all belong to this
session.

## The state

`main` is clean and pushed. Every gate passes:

```
nice -n 19 ionice -c 3 cargo clippy --all-targets -j 16 -- -D warnings
nice -n 19 ionice -c 3 cargo fmt --check
ALSA_CONFIG_PATH=<a real null asound file> nice -n 19 ionice -c 3 cargo test -j 16
    # 805 tests pass, 17 carry #[ignore], 36 binaries
cargo tree -i openssl-sys                # finds nothing
```

**Every test of the sandbox passes too.** One run of
`ALSA_CONFIG_PATH=/dev/null cargo test -- --ignored --test-threads=1` gave 17 of 17.
One thread at a time keeps the requests of the login under the rate limit of the
server.

Two tests read the books of the survey. Those books stand outside the repository,
therefore give their directory in `TOUTUI_SURVEY_BOOKS`. A run with no such variable
reads `tests/data/alice.epub` only, and it passes.

**The tag `v0.6.6` has no release.** That tag came before the version of
`Cargo.toml`, and the workflow refused it, as it must. The work of that tag is in
v0.6.7. Do not try to publish v0.6.6.

## What this session closed

| Item | What | Keys |
|---|---|---|
| T-47 | **The live messages of the server**, with no new dependency | — |
| T-48 | A book of two files played with no player and with no position | — |
| T-49 | The key `?` shows every key, and the footer holds the useful ones | `?` |
| T-50 | The cover fills the panel | — |
| T-51 | bookokrat is AGPL: a decision of the maintainer waits | — |
| T-52 | A fault of the reader locked the user in that view | `Q`, `Esc` |
| T-53 | **Every codec of the server**, with no new dependency | — |
| T-54 | The reader shows a PDF book, with its pictures | `e` |
| T-55 | The position of a book that ends early stays at that end | — |
| T-56 | The queue of the media stands on the disk | `n`, `q` |
| T-57 | A picture of a PDF of 16 bits gives a picture | — |
| T-58 | The reader says "page" for a PDF, and `?` works inside it | `?` |
| T-59 | **Every message of the program stands inside the frame.** 93 calls of `pop_message` went away | — |
| T-60 | The filter of the library holds the tags. `filterdata` gives none | `f` |
| T-61 | The task of the live messages waits longer after each fault | — |
| T-62 | A book of a scan held 137 megabytes of pictures. It holds 9.5 now | `e` |
| T-63 | **The position and the movement of a playback of the stream** | `p`, `u` |
| T-64 | The build of the development filled the disk: 221 gigabytes | — |
| T-65 | The reader kept every ebook for ever. The key `X` removes it now | `X` |

**Three items of the old list became complete with a measurement and no code:**
T-7 (the pages of 500 items), T-8 (the speed changes during a playback), and T-18 (a
WMA file plays through the stream of the server).

### The two items to know

**T-53, every codec of the server.** No decoder of pure Rust reads xHE-AAC, and the
license of `libfdk-aac` does not agree with the GPL. Therefore the program cannot
decode such a file, today or soon. Audiobookshelf gives every media as a stream of
HLS as well, and ffmpeg of the server makes it: **every codec that ffmpeg reads
becomes a codec that Toutui plays**, and the work needed no new dependency.
`src/player/engine/hls.rs` reads the playlist and the transport stream, and
`hls_file.rs` gives the audio to symphonia.

**T-63, the position of such a playback.** The stream begins at the place of the
user, and the engine gave the position of the **decoder**. A book that the user left
at 26 hours would have reported 0, and the loop would have written that on the
server. Every measurement of T-53 began at the second 0, therefore the fault stayed
hidden. **A measurement of a media that the user did not start at its beginning is a
different measurement.**

## The keys of the program

`src/app.rs` holds the key handler, and it is the authority. **`src/ui/keys.rs`
holds the list for the user**, and the key `?` shows it. A new key needs a line in
that file, or the test `every_key_of_the_handler_stands_in_the_list` fails.

## What is open

### 1. The one measurement that this machine cannot make

**A file of xHE-AAC (T-53 and T-63).** The book of the user holds that form. ffmpeg
of the server copies the codec of a file into the transport stream when that codec
fits it, and `codecsToForceAAC` of the server holds `alac`, `ac3`, `eac3`, and
`opus` only: xHE-AAC names itself `aac`, therefore the server copies it. A transport
stream holds that form as LATM, and symphonia has no reader of LATM.

Two answers exist, and the program holds both:

1. ffmpeg gives a fault for such a copy. The server then sets `forceAAC` and it
   starts the transcode again, and the stream holds AAC of the old form. The reader
   of the parts waits for that second try.
2. ffmpeg gives LATM. The program refuses that form **before** the playback, and the
   screen says "The stream of the server holds a form that the program cannot read."

**No program of this machine writes a file of xHE-AAC**, therefore the next session
must read the log of the user after they play that book. `podman logs` of their
server names the command of ffmpeg, and `toutui.log` names the form of the stream.

### 2. The decisions of the maintainer

**The license of bookokrat (T-51).** That project is **AGPL-3.0-or-later** and
Toutui is `GPL-3.0-or-later`. No line of it may come in this repository before the
maintainer decides. A person may read it and write their own code.

**The two dependencies that compile C (T-20).** `libsqlite3-sys` and `ring`. Both
answers need a crate that is not ready: `turso` is a pre-release, and
`rustls-rustcrypto` is an alpha version. The rule of T-20 accepts both crates as
they are, therefore this item is an improvement and not a fault.

### 3. The work that needs no decision

1. **The issues of the fork.** Five open issues hold work that is complete, and each
   needs a line of the measurement and a close: **#10** (T-10, the reader of an EPUB
   book, and a PDF now), **#17** (T-17, Opus), **#18** (T-18, WMA and AWB through the
   stream), **#22** (T-22, the series), and **#23** (T-23, the cover art). **#15**
   (T-15) holds a measurement that says the fault does not occur. **#24** (T-24) is
   the umbrella of the comparison, and section 6 of that document names the
   functions that the program must **not** have. **#20** stays open.
   **The items T-27 to T-65 have no issue.** Give `-R ealtun21/Toutui` to every
   command of `gh`, because this clone resolves to the archived repository.
2. **One run of the tests of ten gave a fault, and this session did not find it.**
   That run came after a clean of the build, and nine runs after it gave 805 of 805
   — with `-j 16`, with `-j 4` beside a build of the release, and with eight threads
   of the tests. **Keep the whole output of `cargo test`** at the next such fault:
   the name of the test is the whole answer.
3. **The list of Continue Listening after a live message.** T-47 puts the position of
   a different client in the mark of the line at once. A media that a different
   client **finished** must leave that list, and the program needs the key `R` for
   it: `user_updated` gives no shelf. A rule that reads the shelves again after such
   a message would close it, and the program must then not ask the server at every
   sync of its own playback.
4. **A limit of the cache of the ebooks (T-65).** The key `X` removes the ebook of
   one media now. A user of twenty books of a scan holds twenty files, and the
   program removes none of its own. A limit needs a rule for the book that the user
   reads now.
5. **The peak of the memory of a PDF (T-62).** `Document::load` of `lopdf` reads the
   whole file, therefore a book of 500 megabytes needs a machine of a gigabyte for
   one moment. `MAX_BOOK_BYTES` of 512 megabytes holds that limit. A reader of one
   page at a time needs a different crate, and no such crate of pure Rust exists.

`docs/T-24-coverage.md` section 6 names every function that the program must **not**
have, with the reason. Read it before you take a row of the table that says `No`.

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
5. **The server takes a name of a field that does not exist.** `?sort=bogus.field`
   gives `200` and an unspecified sequence. Measure a field before you offer it.
6. **`items` of `GET /api/me/listening-stats` is a map, and not a list.**
7. **`GET /api/podcasts/:id/checknew` gives an empty list for a podcast that came one
   second before.** It compares with the time of the last examination.
8. **`convert_seconds` rounds to the minute.** It is wrong for a place in a media.
   Use `clock` of `src/utils/convert_seconds.rs`.
9. **`topGenres` of `GET /api/stats/year/:year` names its value `genre`**, and
   `topAuthors` and `topNarrators` name it `name`.
10. **The lists of the narrators and of the genres stay empty until a session comes
    after the metadata.** The server keeps a copy of the metadata inside each
    session.
11. **The first page of `GET /api/me/listening-sessions` is the page 0**, and a page
    after the last page gives `200` and an empty list.
12. **`quick-xml` 0.41 gives an entity as its own event `GeneralRef`**, and not
    inside the text. A reference makes no text node of the tree of a web page,
    therefore `cfi::Walk` must not count it as one.
13. **A comparison of two lists of texts must not read the text.** A book holds the
    word "very" two hundred times. Read the two lists together, in the sequence of
    the document.
14. **`GET /api/libraries/:id/filterdata` holds no tag.** `GET /api/tags` gives them,
    and a filter of `tags.<base64>` works. A scan of the library changes nothing.
    See T-60.
15. **The rate limit of the login is 40 requests of 600 seconds.** A run of every
    test of the sandbox reaches it, and the test then says "the answer must hold a
    token". Read `podman logs abs-test` for the line of the rate limiter, and run
    those tests with `--test-threads=1`.
16. **A test that changes data must write a value that differs.** The test of the
    live messages wrote the same subtitle at every run: the server saw no change at
    the second run, therefore it sent no message and the test waited for nothing.
17. **A value of the state that belongs to one playback must name that playback.**
    The engine clears the name of a file with no decoder when it starts a playback,
    and the command of the start is not immediate. `playback_of_the_fault` holds the
    identity. See T-53.
18. **`reqwest::blocking` stops the program inside a task of tokio.** That client
    makes a runtime of its own, and a runtime that goes away inside an asynchronous
    context gives "Cannot drop a runtime in a context where blocking is not
    allowed". The engine is a thread, therefore the real program is correct. **A
    test of such a reader must use `std::thread::spawn`.**
19. **A value that comes from the view of the user must come before the view
    changes.** `open_the_ebook` read the title of the media after it set the view to
    the reader, and the answer was then always nothing.

### Of the harness and of the machine

20. **A screen of your own is not a terminal. Use tmux.** A hand-written model of the
    screen mangled the line that the user selected, and it kept the old text of a
    line that the program wrote again. Both times the program was correct.
    ```
    tmux new-session -d -s check -x 160 -y 45 "<the program>"
    sleep 9; tmux capture-pane -p -t check        # -e keeps the colours
    ```
21. **A pseudo terminal must answer TWO questions**, and not one: `ESC [ 5 n` with
    `ESC [ 0 n` for `ratatui-image`, and `ESC [ 6 n` with `ESC [ 1 ; 1 R` for
    crossterm inside `terminal.clear()`. With no answer the program stops with "The
    cursor position could not be read within a normal duration", and that message
    never reaches the log file.
22. **ratatui writes the cells that changed only.** A change of one number therefore
    gives two bytes in the stream of the terminal. A move of the list (`j` and then
    `k`) makes the program write whole lines again.
23. **A key of a login that comes too early goes to the application.** The login
    examines the address with a request, therefore the next field is not ready at
    once: the password `claudetmp` went to the application as the keys `c`, `l`,
    `a`, `u`, `d`, `e`, and `e` opened the reader. **Log in one time, and then reuse
    the database of that isolated `XDG_CONFIG_HOME`.**
24. **An isolated `XDG_CONFIG_HOME` needs two files before the program starts.**
    `config.toml`, and `.env` with `TOUTUI_SECRET_KEY=<something>`.
25. **A test that sets `XDG_CONFIG_HOME` must be alone in its binary**, and every
    test of a global slot must stand in one function. `logic::message` gave a fault
    of one run of three before its two tests became one.
26. **`target/debug` grew to 221 gigabytes and the disk of the maintainer became
    full.** A test binary holds 300 megabytes, `cargo test` makes 36 of them, and
    cargo keeps the binary of every build that came before. Look at `du -sh target`,
    and run `cargo clean --profile dev` at the end of a session. See T-64.
27. **`playwright` wants its own browser:**
    `play.chromium.launch(executable_path="/usr/bin/chromium")`.

## The shapes that the next work should follow

- **A slot between the work and the screen.** The render is not asynchronous.
  `logic::live`, `logic::stats`, `logic::bookmarks`, `logic::authors`, and
  `logic::message` all hold that shape: a task or a key writes, and the render takes
  it at the next frame. A function that needs no `&mut App` reaches every caller.
- **A view says why it holds no line. A message of one row says nothing.** The view
  of the bookmarks held the right shape, and the view of the chapters holds it now.
- **A rule of a loop belongs in a pure function with a name.**
  `the_fault_stops_the_playback`, `the_queue_goes_on`, `wait_after_the_faults`, and
  `is_for_the_screen` are one line each, and a test holds each of them to the
  measurement that made it.
- **A test may read the source of the program.**
  `every_key_of_the_handler_stands_in_the_list` reads `src/app.rs` with
  `include_str!` and it finds every key of the handler in the list of the keys.
- **Measure the dependency before you accept that you need one.** socket.io needed
  none: the second transport of that protocol is plain HTTP. Every codec of the
  server needed none: the stream of HLS is a playlist of text and packets of 188
  bytes.
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
  nothing. `libsqlite3-sys` and `ring` are the two known builds of C, and they stay.
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
> of the archived AlbanDAVID/Toutui. Newest release **v0.7.26**; `Cargo.toml` is at
> 0.7.26, so the next release bumps it first — the workflow refuses a tag that
> disagrees with `Cargo.toml`, **and it builds `--locked`, therefore the commit of
> the bump must hold the new `Cargo.lock`**.
>
> Read `docs/HANDOVER.md` first: the state, the open items, and 27 traps that cost
> real time. Then `docs/TAKEOVER-BACKLOG.md` (the evidence of every item; T-47 to
> T-65 are the newest, and T-53, T-59, and T-63 are the ones to know) and
> `docs/T-24-coverage.md` (**section 6 names what the program must not have, and
> why**).
>
> **The work, in the sequence of its value:**
>
> 1. **Measure the book of xHE-AAC of the user (T-53, T-63).** It is the one
>    measurement that this machine cannot make. The program plays every codec of the
>    server through the stream of HLS, and a file of that form comes as LATM in a
>    transport stream, which symphonia does not read. The program refuses LATM with a
>    clear message, and the server makes AAC of the old form after ffmpeg gives it a
>    fault. Ask the user for `toutui.log` and for `podman logs` of their server after
>    they play that book, and close the item with the answer.
> 2. **Close the five issues of the fork that hold work that is complete** — #10,
>    #17, #18, #22, #23 — with one line of the measurement of each, and look at #15
>    and #24. Give `-R ealtun21/Toutui` to every command of `gh`. The items T-27 to
>    T-65 have no issue.
> 3. **The list of Continue Listening after a live message.** A media that a
>    different client finished must leave that list, and the program needs the key
>    `R` for it today.
> 4. **The two decisions of the maintainer:** the license of bookokrat (T-51, that
>    project is AGPL and this one is GPL) and the two dependencies that compile C
>    (T-20, both answers need a crate that is not ready).
> 5. **The small items:** a limit of the cache of the ebooks (T-65), and the fault of
>    one run of the tests of ten that this session did not find (T-64) — keep the
>    whole output of `cargo test` when it comes.
>
> **Rules that bind every change.** Run all three gates yourself before each commit,
> under `nice -n 19 ionice -c 3` with `-j 16`: `cargo clippy --all-targets --
> -D warnings`, `cargo fmt --check`, and `cargo test` with `ALSA_CONFIG_PATH`
> pointing at a real null asound file (`/dev/null` hangs the real binary). Baseline:
> 805 tests, 17 with `#[ignore]`, 36 binaries, tree clean. Look at `du -sh target`
> and run `cargo clean --profile dev` at the end of the session: this one filled the
> disk with 221 gigabytes.
>
> All prose and user-facing strings in ASD-STE100 simplified technical English. No
> crate needing a system library; `cargo tree -i openssl-sys` must find nothing. No
> test may need the network — sandbox tests carry `#[ignore]` and run with
> `--test-threads=1`, because the login of the server permits 40 requests of 600
> seconds. Never write to AlbanDAVID/Toutui, and keep his credit everywhere it
> appears.
>
> **Show a fault before you fix it, and measure against the sandbox**
> (`docs/TEST-SERVER.md`, podman on `:13399`) before you write an endpoint — and make
> the data exist first, because an empty list shows you no shape. **Measure a media
> that the user did not start at its beginning**: T-63 hid behind a measurement that
> always began at the second 0. Verify your own work with a second program: the log
> of the server (`podman logs abs-test`), a real browser, or `curl`. **Drive the real
> program inside tmux for every view** (`tmux new-session -d -s check -x 160 -y 45
> "<the program>"`, then `tmux capture-pane -p`); a screen of your own writing lies
> to you. Tag, push, and keep working; don't wait for CI.
>
> The user tests each release as it lands and does not want to be asked before
> publishing a patch. The server of the user is theirs alone: ask before you use it,
> always with an isolated `XDG_CONFIG_HOME`, and never write its address or its
> account into this repository. Measure against the sandbox instead.
