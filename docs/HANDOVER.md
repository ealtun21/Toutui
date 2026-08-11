# The handover of 2026-08-11 (the sixth session of that day)

**The newest release is v0.7.21.** The items T-52 to T-59 came after the first
form of this document. Read them in `docs/TAKEOVER-BACKLOG.md`.

This document is for the next session. It says what is done, what is open, and
the traps that cost time. Read `docs/TAKEOVER-BACKLOG.md` for the evidence of
each item, and `docs/T-24-coverage.md` for the comparison with the server.

## The state

`main` is clean and pushed. The newest release is **v0.7.21**. Every gate
passes:

```
cargo clippy --all-targets -- -D warnings
cargo fmt --check
ALSA_CONFIG_PATH=<a real null asound file> cargo test    # 798 tests pass, 17 carry #[ignore], 36 binaries
cargo tree -i openssl-sys                                # finds nothing
```

Two tests read the books of the survey. Those books stand outside the
repository, therefore give their directory in `TOUTUI_SURVEY_BOOKS`. A run with
no such variable reads `tests/data/alice.epub` only, and it passes.

**The tag `v0.6.6` has no release.** That tag came before the version of
`Cargo.toml`, and the workflow refused it, as it must. The work of that tag is
in v0.6.7. Do not try to publish v0.6.6.

## What this session closed

| Item | What | Keys |
|---|---|---|
| T-47 | The live messages of the server, with **no new dependency** | — |
| T-48 | A book of two files played with no player and with no position | — |
| T-49 | The key `?` shows every key, and the footer holds the useful ones | `?` |
| T-50 | The cover fills the panel | — |
| T-51 | bookokrat is AGPL, and a PDF of pure Rust: a decision waits | — |
| T-52 | A fault of the reader locked the user in that view | `Q`, `Esc` |
| T-53 | **Every codec of the server**, with no new dependency | — |
| T-54 | The reader shows a PDF book, with its pictures | `e` |
| T-55 | The position of a book that ends early stays at that end | — |
| T-56 | **The queue of the media stands on the disk** | `n`, `q` |
| T-57 | A picture of a PDF of 16 bits gives a picture | — |
| T-58 | The reader says "page" for a PDF, and `?` works inside it | `?` |
| T-59 | The view of the chapters says why it holds no line, and **every message of the program stands inside the frame**: 93 calls of `pop_message` went away | `C` |
| — | T-7, T-8, and T-18 became complete: the pages of 500, the speed that changes during a playback, and a WMA file that plays through the stream | — |

### T-47, the live messages

**The decision of the handover before this one is settled: the program needs no
new dependency.** The two crates of socket.io both bring `native-tls`, therefore
`openssl-sys`, and the rule of T-20 refuses both. The transport `polling` of
socket.io is plain HTTP, and `reqwest` does it already.

`src/api/live.rs` holds the protocol, and every function that reads a packet is
pure. `src/logic/live.rs` is the box between the task and the screen.

- **The position of a different client goes to the mark of the line** of the Home
  view, and it needs no key. The program itself sends a position every ten
  seconds, and the server answers **every** such request with `user_updated` to
  the client that sent it. Therefore a notice for that message would never go
  away.
- **A change of the metadata of an item** makes every list old. The header says
  "R: the server has newer data".
- The measurement in a pseudo terminal: the two lines of one book showed 13%, a
  different client moved to 29 per cent, and both lines then showed 29% with no
  key.

### T-48, the book that played with no player

A user reported it. One book of their library played the sound, and the program
showed no player and sent no position. **The queue of the engine holds two
tracks, therefore the engine opens the track after the track that plays.** That
book held the same audio two times, as AAC-LC and as xHE-AAC, and symphonia
reads AAC-LC only. The fault of the second file stopped the whole playback while
the queue of the player played the first file.

Read T-48 of the backlog for the table of the symptoms. The rule is one pure
function: `the_fault_stops_the_playback`.

### T-49 and T-50, the screen

The user asked for both. `src/ui/keys.rs` holds every key now, and a test reads
`src/app.rs` and finds every `KeyCode::Char` of the handler in a group.
`box_of_the_picture` takes the form of the real picture, and the panel takes the
width that its height can use.

## The keys of the program

**`src/ui/keys.rs` holds the list now, and `src/app.rs` stays the authority of
the work.** The key `?` shows the list in the program. A new key needs a line in
`keys.rs`, or the test `every_key_of_the_handler_stands_in_the_list` fails.

## What is open

### The one measurement that this session could not make

**A file of xHE-AAC.** T-53 gives the program the stream of the server for a file
that no decoder of the program reads, and the measurement used a WMA file. The
book of the user holds xHE-AAC, and ffmpeg of the server copies that form into a
transport stream as LATM, which symphonia does not read. The server makes AAC of
the old form after ffmpeg gives it a fault, and the program waits for that second
try. **No program of this machine writes a file of xHE-AAC**, therefore the next
session must measure it against the book of the user. The screen says the form
that it cannot read, and it gives no silence.

### Needs a decision of the maintainer

1. **The license of bookokrat (T-51).** That project is **AGPL-3.0-or-later**,
   and Toutui is `GPL-3.0-or-later`. No line of it may come in this repository
   before the maintainer decides about the license of the fork. A person may
   read it and write their own code.

### The work that needs no decision

2. ~~**A PDF of the text.**~~ **Done: T-54.** The reader shows a PDF book and its
   pictures. `lopdf` 0.44 gives the text and the pictures of each page, and one
   page is one chapter. No crate of pure Rust draws a **page** of a PDF.
3. ~~**The narrators and the tags.**~~ **Done: T-60.** The narrators came from
   `filterdata` already. **The tags did not**: that endpoint gives `tags: []` even
   after a scan, therefore the program asks `GET /api/tags` and it puts them in the
   same view.
4. ~~**The queue on the disk.**~~ **Done: T-56.** The table `queue` of the
   version 7 of the schema holds it, and a new start reads it.
5. ~~**The position of a book that ends early goes on.**~~ **Done: T-55.**
   `end_of_the_first` of `TrackList` gives the end of the tracks that play.

`docs/T-24-coverage.md` section 6 names every function that the program must
**not** have, with the reason.

## The traps that cost time

The seventeen traps of the handover before this one all stay true. These seven are
new.

1. **A screen of my own is not a terminal. Use tmux.** A hand-written model of
   the screen in the harness of the pseudo terminal mangled the line that the
   user selected: the capture held `[3552;180;...`, the escape of the style with
   the new percent **inside** it. It also left the old text of a line that the
   program wrote again. Both times the program was correct.
   ```
   tmux new-session -d -s check -x 160 -y 45 "<the program>"
   sleep 9; tmux capture-pane -p -t check
   ```
   tmux is a real terminal emulator. It answers every question of the program,
   and `capture-pane` gives the true screen. **Use it for every view.**
2. **ratatui writes the cells that changed only.** A change of one number
   therefore gives two bytes in the stream of the terminal, and a search of the
   raw bytes for the whole line finds nothing. A move of the list (`j` and then
   `k`) makes the program write both lines again.
3. **A key of a login that comes too early goes to the application.** The login
   screen examines the address with a request, therefore the next field is not
   ready at once. The password `claudetmp` went to the application as the keys
   `c`, `l`, `a`, `u`, `d`, `e`, ... and `e` opened the reader of an ebook. **Log
   in one time, and then reuse the database of that isolated `XDG_CONFIG_HOME`.**
4. **An isolated `XDG_CONFIG_HOME` needs two files before the program starts.**
   `config.toml` (the program stops with "configuration file not found") and
   `.env` with `TOUTUI_SECRET_KEY=<something>` (the login writes "No secret key
   is present"). Copy the `config.toml` of the harness of the sandbox.

5. **A value of the state that belongs to one playback must name that playback.**
   The engine clears the name of a file with no decoder when it starts a playback,
   and the command of the start is not immediate. A loop that read that name saw
   the fault of the playback before it. `playback_of_the_fault` holds the identity
   now. See T-53.
6. **`reqwest::blocking` stops the program inside a task of tokio.** The client of
   that form makes a runtime of its own, and a runtime that goes away inside an
   asynchronous context gives "Cannot drop a runtime in a context where blocking
   is not allowed". The engine is a thread, therefore the real program is correct.
   **A test of such a reader must use `std::thread::spawn`.** See
   `tests/the_stream_against_the_sandbox.rs`.
7. **A value that comes from the view of the user must come before the view
   changes.** `open_the_ebook` read the title of the media after it set the view
   to the reader, and the answer was then always nothing. See T-54.

8. **A view says why it holds no line. A message of one row says nothing.** The
   key `C` with no media wrote a message with `pop_message`, and the next frame of
   the Home view took it away. The view of the bookmarks held the right shape
   already: it opens, and its title names the reason. See T-59.
9. **A sweep of every view finds what a test does not.** One run in tmux pressed
   the key of each view and counted the lines of text of each screen. Two views of
   fifteen answered with nothing, and one of them was a real fault.

10. **A `429` of the login looks like a fault of the token.** The sandbox permits
    40 requests of `POST /login` in 600 seconds. A run of every test of the
    sandbox beside some requests of `curl` reaches that limit, and the test then
    says "the answer must hold a token". Read `podman logs abs-test` for the line
    of the rate limiter. `docs/TEST-SERVER.md` holds the numbers.
11. **A test that changes data must write a value that differs.** The test of the
    live messages wrote the same subtitle at every run. The server saw no change at
    the second run, therefore it sent no message and the test waited 20 seconds for
    nothing. That test reads the value of the server now, and it writes the other
    one.

## The shapes that this session made, and that the next work should follow

- **Measure the dependency before you accept that you need one.** The handover
  before this one named socket.io as a new dependency, and the measurement showed
  that the program needs none: the second transport of that protocol is plain
  HTTP.
- **A rule of a loop belongs in a pure function with a name.**
  `the_fault_stops_the_playback` and `the_library_changed` are one line each, and
  a test holds each of them to the measurement that made it.
- **A test may read the source of the program.** `every_key_of_the_handler_stands_in_the_list`
  reads `src/app.rs` with `include_str!` and it finds every key of the handler in
  the list of the keys. A list of text that a person must keep in agreement with
  the code needs such a test.
- **Show the fault with the old binary.** `git checkout -- <the file>` and a
  build in a different `CARGO_TARGET_DIR` give the old program beside the new
  one. T-48 needed that: the same book, one binary with no player and one binary
  with a player.
- **Make the data of the fault in the sandbox.** The book of the user held one
  file that no decoder reads. `ffmpeg -f lavfi -i sine=... -c:a wmav2` makes such
  a file, and T-18 says that the program plays no WMA. The sandbox then holds the
  exact shape of the fault, and no server of a user is necessary.
- **A message of the server can hold a secret.** `user_updated` carries a new
  token of the user. The log holds the name of a message, and never the body.

## The rules that do not change

- Every document, comment, and text for the user in ASD-STE100 simplified
  technical English. Short sentences, active voice, one instruction per
  sentence.
- No crate that needs a library of the system. `cargo tree -i openssl-sys`
  must find nothing. `libsqlite3-sys` and `ring` are the two known builds of C,
  and they stay. See T-20.
- No test may need the network. A test that needs the sandbox carries
  `#[ignore]` and says how to run it. Thirteen such files exist now.
- Never write to `AlbanDAVID/Toutui`. It is archived. AlbanDAVID stays credited
  in the README, in the LICENSE, in `Cargo.toml`, and in the settings screen.
  **`gh` resolves to that repository by default in this clone**, therefore give
  `-R ealtun21/Toutui` to every `gh` command.
- Show a fault before you correct it.
- Tag, push, and go on. Do not wait for continuous integration.
- The address of the server of the user must stay outside this repository, and
  the account of the user too.

## The prompt for the next session

> Continue the Toutui takeover. Repo: `/home/nyverino/Documents/Toutui`
> (ealtun21/Toutui, branch main). Maintained fork of the archived
> AlbanDAVID/Toutui. Newest release **v0.7.16**; `Cargo.toml` is at 0.7.16, so
> the next release bumps it first — the release workflow refuses a tag that
> disagrees with `Cargo.toml`.
>
> Read `docs/HANDOVER.md` first. It has the state, the open items, and the
> traps that cost real time. Then `docs/T-24-coverage.md` (the
> function-by-function comparison against Audiobookshelf 2.36.0; **section 6
> names what the program must not have, and why**) and
> `docs/TAKEOVER-BACKLOG.md` (the evidence for every closed item; T-47 to T-51
> are the newest).
>
> What stays is in the section "What is open":
> 1. **Measure T-53 against a file of xHE-AAC.** The stream of the server plays
>    every codec of ffmpeg, and a file of that form comes as LATM in a transport
>    stream. The program refuses LATM with a clear message, and the server makes
>    AAC of the old form after ffmpeg gives it a fault. No program of this machine
>    writes such a file, therefore the measurement needs the book of the user.
> 2. **The license of bookokrat.** That project is AGPL-3.0-or-later and Toutui
>    is GPL-3.0-or-later. **No line of it comes in without the maintainer.**
> 3. Nothing of the old list stays. Read the newest items of
>    `docs/TAKEOVER-BACKLOG.md` (T-60 is the last one) and
>    `docs/T-24-coverage.md` section 5 for the work that is worth doing.
>
> Rules that bind every change: run all three gates yourself before each
> commit — `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`,
> and `cargo test` with `ALSA_CONFIG_PATH` pointing at a real null asound file
> (`/dev/null` hangs the real binary). Baseline: 768 tests, 34 binaries, tree
> clean. Baseline of the tests: 798 pass, 17 carry `#[ignore]`, 36 binaries. Run
> every cargo command under `nice -n 19 ionice -c 3`: a full build uses every core,
> and the user tests the program on the same machine. All
> prose and user-facing strings in ASD-STE100 simplified technical
> English. No crate needing a system library; `cargo tree -i openssl-sys` must
> find nothing. No test may need the network — sandbox tests carry `#[ignore]`
> — and no test may hold a path of the machine. Never write to
> AlbanDAVID/Toutui; keep his credit everywhere it appears, and give
> `-R ealtun21/Toutui` to every `gh` command. Show a fault before you fix it,
> and measure against the sandbox (`docs/TEST-SERVER.md`, podman on `:13399`)
> before you write an endpoint — and make the data exist first, because an
> empty list shows you no shape. Verify your own work with a second program:
> the log of the server (`podman logs abs-test`), a real browser, or `curl`.
> **Drive the real program inside tmux for every view** (`tmux new-session -d
> -s check -x 160 -y 45 "<the program>"`, then `tmux capture-pane -p`); a
> screen of your own writing lies to you.
> Tag, push, and keep working; don't wait for CI.
>
> The user tests each release as it lands and does not want to be asked before
> publishing a patch. The server of the user is theirs alone: ask before you
> use it, always with an isolated `XDG_CONFIG_HOME`, and never write its
> address or its account into this repository. Measure against the sandbox
> instead.
