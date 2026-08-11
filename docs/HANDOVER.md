# The handover of 2026-08-11 (the ninth session of that day)

This document is for the next session. It says what is done, what is open, and the
traps that cost real time. Read `docs/TAKEOVER-BACKLOG.md` for the evidence of each
item, and `docs/T-24-coverage.md` for the comparison with the server.

**The newest release is v0.7.46**, and the items T-88 to T-100 belong to this
session. T-74 to T-87 belong to the session before it.

**This session made every sweep that no session had made**: the terminal of 80
columns, the offline mode, the view of the login, and the reader of a book.
**Each of them found a fault**, and T-90, T-91, T-92, T-94, and T-95 are those
faults. The sweep stays the tool that finds what a test does not.

## The state

`main` is clean and pushed, and `v0.7.46` is tagged. Every gate passes:

```
nice -n 19 ionice -c 3 cargo clippy --all-targets -j 16 -- -D warnings
nice -n 19 ionice -c 3 cargo fmt --check
ALSA_CONFIG_PATH=<a real null asound file> nice -n 19 ionice -c 3 cargo nextest run -j 16
    # 885 tests pass in 2.2 s, 23 carry #[ignore], 42 binaries
    # cargo nextest run --run-ignored all gives 908 of 908 with the sandbox up,
    # in 44 s: one test of that run waits 15 s for the time limit of a request
cargo tree -i openssl-sys                # finds nothing
cargo tree -i cc                         # finds libsqlite3-sys and ring only
```

**`cargo nextest run` gives the same tests in 2.2 seconds**, and `cargo test` gives
them in 8.7. Use nextest: `.config/nextest.toml` stands in the repository, and the
tool is on this machine. See T-74.

**Every test of the sandbox passes too.** One run of
`cargo nextest run --run-ignored all` gives **908 of 908**, and the group
`the-sandbox` of `.config/nextest.toml` runs them one at a time for the rate limit
of the login. With `cargo test`, give `-- --ignored --test-threads=1`.

**The rate limit of the login stopped three runs of this session, and T-96 closed
that fault.** Every test of the sandbox takes its token from
`tests/common/mod.rs`, and that module keeps the token in a file of
`CARGO_TARGET_TMPDIR`: one run makes **one** login, and the run after it makes
none. Three runs one after the other now give no line of the rate limiter.

**A run that says "the answer must hold a token" still means the rate limit.**
Read `podman logs abs-test` for "[RateLimiter] Rate limit exceeded", and give the
container a restart. A sweep of the view of the login uses those 40 requests.

**The fault of one run of ten has a name now: T-86.**
`the_four_requests_of_the_start_go_together` failed when the whole start took more
than two seconds, and that is a measurement of the machine as much as of the program:
one run of twelve of this session failed at 4.2 seconds while a build and a program of
tmux ran beside it. The test holds the **time of each request** now, therefore the load
of the machine changes nothing. Twelve runs after that change gave every test each
time.

Two tests read the books of the survey. Those books stand outside the repository,
therefore give their directory in `TOUTUI_SURVEY_BOOKS`. A run with no such variable
reads `tests/data/alice.epub` only, and it passes.

**The tag `v0.6.6` has no release.** That tag came before the version of
`Cargo.toml`, and the workflow refused it, as it must. The work of that tag is in
v0.6.7. Do not try to publish v0.6.6.

## What this session closed

| Item | What | Keys |
|---|---|---|
| T-88 | **A view that makes a collection or a playlist** | `m`, then `c` or `p` |
| T-89 | The box that takes a text left two columns of the view on the screen | `b`, `A`, `c`, `p` |
| T-90 | **Every footer lost its end in a terminal of 80 columns** | — |
| T-91 | The program said that the library holds no list, and the server was down | `c`, `s`, `m` |
| T-92 | **The login said "ERROR: Login failed" for every fault** | — |
| T-93 | **The keys that remove a list and that give it a new name** | `c`, then `r` and `X` |
| T-94 | The row of the item lost its end in 80 columns | — |
| T-95 | "1 items" a second time, in the view of the search | `/` |
| T-96 | **One login for every test of the sandbox**, and a feed that is not a fault | — |
| T-97 | **One request that stopped at its time limit took the server away** | — |
| T-98 | **`CARGO_TARGET_DIR` on a `tmpfs` gives nothing.** A measurement, and no code | — |
| T-99 | A terminal of 18 rows showed one line of the list | — |
| T-100 | **The description of a collection and of a playlist** | `c`, then `D` |

### The items of this session, and what each of them taught

**T-88, and the measurement that decided its shape.** The first idea was a view
that asks for a name and a library. **The server refuses it**: `POST
/api/collections` answers `400` with "No books" for a collection with no book. A
playlist with no item gives `200`, therefore the two requests do not behave in the
same way. **A new collection needs a media**, and the key `m` holds one already:
the work stands in that view, and it needs no new view at all. **Measure the
request before you draw the screen that sends it.**

The server also takes **two lists of one name**, and it gives the second one its
own identity. The measurement made two lines "Measure Collection [1 item]" on the
screen that no key tells apart, therefore the program refuses that name.

**T-89, and the shape of every box of the program.** `ask_for_a_text` makes a
`Terminal` of its own. **ratatui writes the cells that changed only**, and it
compares with the buffer that it holds itself: the terminal of the program knew
nothing of the letters that the box wrote over, therefore they stayed. `Clear`
over the whole rows and one `terminal.clear()` after the box answer it. T-42 held
the same answer for the key `R` already.

**T-90, the first screen of the sweep of 80 columns.** The footer said
"?: every k". **The area of the footer holds two rows**, and the footer wrote on
one of them: a `Paragraph` with no `Wrap` cuts every letter after the width. The
module `ui::keys` said that every footer fits in 80 columns, and no footer of more
than 80 letters did.

**T-91, the sweep of the offline mode.** With `podman stop abs-test`, three views
said something that they cannot know: "This library has no collection and no
playlist", "This library has no series", and "Press c or p to make one". **The
view of the authors held the rule already**: "The server gave no author: the
server does not answer". A view must not give a reason that the program does not
have.

**T-93, the two keys of a list, and the asymmetry of the server.** `PATCH` of a
collection with a name of no letter gives `200` and a collection with **no
name**, and the same request of a playlist keeps the old name. The server
examines the name when it **makes** a list (T-88), and it examines nothing here.
**The rules of a name therefore belong to the program**, and they are the rules
of T-88. The program asks one time before it removes a list, and the question
names the kind: every user of the server sees a collection.

**T-94 and T-95 came from the sweep of 80 columns**, and both are a fault that a
session before this one corrected in a different place. T-94 is T-90 again: the
row of the item holds three rows, it wrote on one, and 21 paragraphs of the views
held that fault. T-95 is T-85 again: the title of the search counted its own
items, and **the test of that title asked for "1 items" and it passed**.

**T-96 and T-97 came from the tests themselves.** The rate limit of the login
stopped three runs, and the answer is one login for a whole run. The measurement
of that work gave the sequence of T-97: a request of a feed stopped at its time
limit, and **every request after it said "No server address answered"** for up to
60 seconds. T-87 corrected that for a status of the server, and this corrects it
for a fault of the transport.

**T-98 is a measurement that says "do not do this work".** Three handovers
carried `CARGO_TARGET_DIR` on a `tmpfs`, and the memory is 16 percent faster than
the ZFS of the maintainer for 3000 small files. A build writes 11 gigabytes, and
that costs about 8 seconds of a disk that gives 1.4 GB/s.

**T-99, the sweep of a small terminal.** A terminal of 100 by 18 gave the Home
view **one line** of 24 media: the row of the item took 3 of the 7 rows of that
area with a fixed length, and 10 rows of the screen were empty.
`the_areas_of_a_list` gives every row to the list when the area holds 12 rows or
fewer, and a terminal of 24 rows and one of 45 rows draw what they drew before.
**The 7 rows of the player stay empty while nothing plays**, and they are the
next question of a small terminal: a view that takes them would move every line
when a playback starts, therefore that is a choice of the maintainer.

**T-100, the description of a list.** The view showed the description of a list
already, and no key made one. The same `PATCH` of T-93 takes it.

**The sweep of two accounts found no fault, and a measurement of this session
said the opposite for some minutes.** The key `l` of the view of the accounts
gives its question at the first press. The wrong measurement read the top of the
screen after the key, and then it read the row of the message 10 seconds later:
**a message of the program lives six seconds.** Read the row that holds the
message, and read it inside that time.

**T-92, the sweep of the view of the login.** Two of the three messages of that
view are good work already, and the third said "ERROR: Login failed" for every
status. **The status of the answer holds the reason**, and 429 is the status that
costs the most time: the rate limit of 40 requests of 600 seconds. Three other
items came out of the same screen: the word "ERROR: " that no other message holds,
an empty username that went forward with no word, and the field of the address
that a wrong password emptied.

## What the session before this one closed

| Item | What | Keys |
|---|---|---|
| T-74 | **The run of the tests takes 2.2 seconds**, and it took 18.7 | — |
| T-75 | Two texts of the screen that a sweep of the views found | `v`, `S` |
| T-76 | **The books of a media, when an item holds more than one** | `e` |
| T-77 | **The settings write `config.toml`**, and they keep every comment | `S` |
| T-78 | The message of the program took the letters of the view below it | — |
| T-79 | The key `h` of the view of the search did nothing | `/` |
| T-80 | **The keys of the volume said nothing.** The row of the player names the volume now | `o`, `i` |
| T-81 | **The queue of the episodes that the server downloads**, and the key that empties it | `d`, `X` |
| T-82 | The choice of the library changed nothing on the screen | `S` |
| T-83 | The key `s` of a library of podcasts said nothing | `s` |
| T-84 | **The media of a collection and of a playlist** | `m`, `X` |
| T-85 | "1 items" | — |
| T-86 | The test of the requests of the start measured the machine | — |
| T-87 | **One answer of 400 took the address of the server away** | — |

### The sessions before those two

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

### The items of the session before this one, and what each of them taught

**T-74, the tests.** `cargo test` was not the slow part: two tests held 12.7 seconds
of the 18.7 of the run. One of them holds a clock of its own now
(`#[tokio::test(start_paused = true)]`), and the other polls for the evidence that
the loop acted. **Both still show the fault**: a build with the correction removed
fails. `cargo nextest` runs every test in one pool of processes, and the run takes
2.2 seconds.

**T-75, the sweep of the views.** Thirteen views in 9.7 seconds with
`docs/harness/drive.sh`. It found two texts that no test held: the footer of the
narrators said "author", and a text of the settings held a run of 22 spaces. A test
now holds every text of a view to one space between two words.

**T-87, and it is the one to know.** `ApiError::is_endpoint_fault` held every status
of `Server` as a fault of the address. One answer of **400** therefore marked the one
address of the pool down, and **every request after it said "No server address
answered"** until the examination of the address ran again. The server answers 400 for
work that a user does every day: a book that stands in a collection already, and a
podcast whose directory exists. A fault of the endpoint is a transport fault or a
status of 500 or more now.

**T-77, the settings that write.** `with_the_value` changes one line of the file
and it keeps every other line, therefore every comment of the user stays. The
write is atomic (a file beside it, and a rename), and the program uses the new
value with no restart.

**T-78 and T-79 came from the sweep of the reader and of the search**, and no
test held either. A `Paragraph` of ratatui gives its style to every cell of its
area and it writes its own text only: the message of the program therefore stood
between the letters of the book. The key `h` of the view of the search moved
nothing, because the handler held no line for that view.

**T-76, the books of a media.** `media.ebookFile` names one book, and `libraryFiles`
holds every file of the item. The key `e` inside the reader gives the list.
**The server holds one place for each media, and not one place for each file**,
therefore the place of a book that is not the book of the server stays on this
machine.

**T-81, the queue of the downloads of the server (`d`).** The key `E` gives the server
the episodes of a feed, and the server does that work alone. The view holds the episode
of now with the mark `▼` and the queue after it, and **it asks the server again at each
message `episode_download_*` of socket.io**: the list moved from 9 lines to 4 while the
screen stood open and the user pressed no key. The key `X` empties the queue of one
podcast, and it asks one time first.

**T-84, the media of a collection and of a playlist (`m` and `X`).** The program read
the lists and it changed none of them. Four requests do that work, and the server
answers **400** for a media that stands in the list already. That answer found T-87.

**T-80, T-82, T-83, and T-85 came from the sweeps too**: the keys of the volume said
nothing, the choice of the library changed no line of the screen, the key `s` of a
library of podcasts said nothing, and every title of a view said "1 items".

### The two items of an older session

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

### 2. The road to a program with no fault, in the sequence of its value

**Take one item, measure it, correct it, tag it, and go on.** Every item below
holds its evidence, and no item needs a decision of the maintainer except the
first one of the group 1.

#### Group 1: what a user sees

1. **The changelog of the fork stops at v0.6.8, and the program is at v0.7.46.**
   `src/utils/changelog.rs` holds 48 entries, and the newest of them is
   `Changelog Toutui v0.6.8`. The key `S` and then "About and changelog" shows
   that screen to a user, therefore **38 releases of this fork are not there**:
   T-27 to T-100 all went to a user who reads nothing of them. The file is a list
   of `String` in one function, and a new entry is 10 lines.
   **Take the entries from `git log --oneline` and from
   `docs/TAKEOVER-BACKLOG.md`**, and write them in the words of a user and not in
   the words of the code: "The keys c and p make a collection or a playlist" and
   not "T-88".
2. **The 7 rows of the player in a small terminal (T-99).** The layout of every
   view holds 6 rows for the player and 1 for the refresh, and they stay empty
   while no media plays. In a terminal of 18 rows they are 7 of the 18.
   **A view that takes them while nothing plays moves every line when a playback
   starts**, therefore this needs the decision of the maintainer. Ask, and hold
   the answer here.
3. **The sequence of the media inside a playlist.** T-100 closes the description
   of a list, and the sequence stays: a user cannot move an episode of a playlist
   up or down. **Measure `PATCH /api/playlists/:id` with `items` first**: T-93
   measured `name` and `description` of that path, and not `items`.
4. **The keys of a media that the server offers and the program does not.** Read
   the table of section 4 of `docs/T-24-coverage.md`, and **read the code of each
   row before you take it**: this session found four rows of that table that were
   old.

#### Group 2: the sweeps that stay

**Every sweep of a condition that no session had made found a fault**: 80
columns (T-90 and T-94), the offline mode (T-91), the view of the login (T-92),
the reader (T-95), and a terminal of 18 rows (T-99). These conditions stay:

1. **A library of one item**, and **a book of one chapter**: every "1 item" of
   the program (T-85, T-95, and T-100 each found one of those texts in a
   different place).
2. **A pool of two addresses.** `config.toml` takes more than one address of one
   server, and no sweep drove the program with two. T-97 changed the rule of that
   pool, and a measurement with two addresses would show the change of address
   that no test of a mock server shows.
3. **A media that the server holds and the disk does not**, while the server goes
   away in the middle of a playback.
4. **A terminal of 300 columns**: every text of a view that a wrap makes wide.

#### Group 3: the tests and the harness

1. **The fast suite must stay at about 2 seconds.** It holds 885 tests in 2.2 s
   today. A new test that needs a wait belongs behind `#[ignore]`, as
   `one_request_that_stops_at_its_time_limit_keeps_the_address` does with its 15
   seconds.
2. **A guard test must hold the rule of the user, and not the form of the code.**
   `no_title_of_a_view_counts_its_own_items` reads `{} items` of four files, and
   it did not find `{} item(s)` of the view of the lists (T-100 found that one by
   hand). **A better guard reads every file of `src/ui` and of `src/logic`**, and
   it names every text that counts its own items in any form.
3. **The tests of the sandbox take one token (T-96).** A new test of that kind
   takes `mod common; use common::token;` and it writes no login of its own.
4. **A test that measures the absence of a fault must give the fault the time to
   appear** (T-74). A poll of the value that the test wants is a false pass.

#### Group 4: the words for the user

1. **Every text in ASD-STE100 simplified technical English.** Short sentences,
   active voice, one instruction per sentence.
2. **A view says why it holds no line**, and it never says a reason that the
   program does not have (T-91).
3. **A key that does nothing in one view is a fault of its own** (T-79).
4. **The row of the message holds one line of 150 letters** (the trap 11), and
   **a message lives six seconds**: read it inside that time, and read the row
   that holds it.

#### Group 5: the work that waits for a user

1. **The peak of the memory of a PDF (T-62).** `Document::load` of `lopdf` reads
   the whole file, therefore a book of 500 megabytes needs a machine of a
   gigabyte for one moment. `MAX_BOOK_BYTES` of 512 megabytes holds that limit.
   A reader of one page at a time needs a different crate, and no such crate of
   pure Rust exists. **Take this when a user meets it.**
2. **The list of Continue Listening of a different library (T-66).** The Home
   view shows one library, therefore no user meets this today.
3. **A view of the search that holds its own titles (T-70).** `get_all_books`
   reads every page at the start, and 500 pages of 500 items hold 250000 items:
   no library of a user meets this today.

#### How to know that the work of an item is done

- The three gates pass: `cargo clippy --all-targets -- -D warnings`,
  `cargo fmt --check`, and `cargo nextest run` with a real null asound file.
- `cargo nextest run --run-ignored all` gives every test with the sandbox up.
- **A build with the correction removed fails.** Show the fault before you
  correct it, and show that the test finds it.
- The real program in tmux shows the new behaviour, and a second program
  (`curl`, `podman logs abs-test`, or a browser) says the same.
- `docs/TAKEOVER-BACKLOG.md` holds the measurement, and this document holds the
  item, the trap, and the next work.
- The commit names the item (T-nn), and the release holds `Cargo.toml` and
  `Cargo.lock` together.

`docs/T-24-coverage.md` section 6 names every function that the program must
**not** have, with the reason. Read it before you take a row of the table that
says `No`. **A measurement changed one row of that section on 2026-08-11:** a PDF
of text holds its text, therefore T-54 reads it. A CBZ stays outside.

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
| The run of every test | **2.2 s** | `cargo nextest run -j 16` (T-74; it was 18.7 s with `cargo test`, and it is 8.7 s with it now) |
| One sweep of thirteen views in tmux | **9.7 s** | `docs/harness/drive.sh`, with a poll of the screen |
| One restart of the sandbox | 13 s | `podman restart abs-test` |

**T-74 did the work of the three sections below.** They stay here, because the
numbers say why each answer was the right one.

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

### 2. Two tests held 12.7 seconds of the 18 seconds of the run — done (T-74)

| The test | Before | After | The answer |
|---|---|---|---|
| `the_position_survives_a_playback_that_does_not_start` | 8.01 s | **0.01 s** | A clock of its own, `#[tokio::test(start_paused = true)]` |
| `playback_ownership` (two tests) | 7.72 s | **3.0 s** | A poll of the forced sync, and a poll of `is_finished` |

**Each of the two answers fits its test, and they do not exchange.** The first test
makes no request and opens no socket, therefore a clock of its own is safe. The
second test waits for a server (wiremock): a clock of its own would move to the
timeout of a request while that request is still on its way.

**Both still show the fault.** A build with the correction of T-38 removed fails with
"the position went to 0 seconds", and a build with the guard of the identity of the
playback removed fails after the limit of the poll.

**The rule that this work found, and every new test must follow it.** Some sleeps
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

### 3. `cargo test` runs the binaries one after the other — done (T-74)

34 files of `tests/` give 34 binaries, and cargo runs them in sequence: the 18 seconds
are almost the sum of their times. **`cargo-nextest` runs every test of every binary
in one pool of processes.** The run would then take the time of the slowest test, of
about 8 seconds, and 6 seconds after the two sleeps above go away.

`cargo-nextest` is a tool of the machine, and **not** a dependency of the program:
`Cargo.toml` does not change, and the rule of T-20 stays. **It stands on this machine
now**, and `.config/nextest.toml` stands in the repository. The two properties that
needed a measurement:

- nextest gives **one process for each test**. Every test of a global slot of the
  process (`logic::message`, `logic::live`, `logic::authors`, the cache of T-72)
  passes, and ten runs one after the other gave 838 of 838 each time.
- The tests of the sandbox need one thread for the rate limit of the login. The group
  `the-sandbox` of `.config/nextest.toml` holds `max-threads = 1` for every binary
  whose name ends with `_against_the_sandbox`, and one run gave 18 of 18.

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
   **This is the work that stays**, and no session measured it.
2. Fewer binaries: 35 files of `tests/` hold 300 megabytes each. **Some of those files
   stand alone for a reason** — a test that writes `XDG_CONFIG_HOME` or `XDG_DATA_HOME`
   must be alone in its binary (the trap 8 of the harness). Every other file can join a
   file of a group, and each join takes one binary of 300 megabytes away.
   **`cargo-nextest` gives one process for each test, therefore that reason goes away**
   for a run of nextest. A run of `cargo test` still needs the rule, and the workflow
   of the release uses `cargo test`.

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
28. **A view key works in the Home view, in the Library view, and in the view of the
    search only.** A sweep that presses `a`, `v`, `c`, `f`, or `s` inside the view of
    the series gets nothing, and the program is correct: `show_the_names` and the keys
    `s` and `c` hold that rule. **A sweep must come back to the Library between two
    views.** See T-75.
29. **`Wrap` of ratatui takes a space away at the start of a line that it makes, and
    it keeps every space that stands inside a line.** A text of the source that an
    old wrap left with a run of 22 spaces reaches the screen as it stands. See T-75.
30. **The server holds one place for each media (`ebookLocation`), and not one place
    for each file.** An item can hold more than one ebook, therefore a reader of a
    second book must neither read that place nor write it. See T-76.
31. **The flag of the forced sync holds one identity for the whole process.** Two
    tests of one binary that ask for a sync at the same time take the flag from each
    other. Give each test its own identity of a playback, and ask again at each step
    of the poll. See T-74.
32. **A `Paragraph` of ratatui gives its style to every cell of its area, and it
    writes its own text only.** Every letter that stood on those cells stays.
    `Clear` takes them away. The row of the message met this in the reader, and
    no list view showed it: that row holds no letter there. See T-78.
33. **A key that does nothing in one view of fifteen is a fault of its own.** The
    user learns `h` in every other view. **Press every key of a view in the
    sweep**, and not the key that opens it only. See T-79.
34. **The key `t` of the timer of sleep is not a view.** It goes to the next value
    (5, 10, 15, 30, 45, and 60 minutes, the end of the chapter, and off) and it
    writes a message. A sweep that waits for a view waits for nothing.
35. **The contents of the reader close with `l` too.** The key `t` opens them and
    it closes them, and `l` goes to the chapter and closes them. A sweep that
    presses `t` after `l` therefore **opens** them again.
36. **A status of 400 is a fault of the request, and not of the address.** The
    server answers 400 for a book that stands in a collection already, for an
    episode that stands in a playlist already, and for a podcast whose directory
    exists. **Read `is_endpoint_fault` before you add a status to it**: the pool
    marks the address down for such a fault, and a pool of one address then has
    no address at all. See T-87.
37. **The queue of the downloads of the server does not fill at once.**
    `POST /api/podcasts/:id/download-episodes` answers `200`, and a read two
    seconds later gave an empty queue. Poll it. See T-81.
38. **`GET /api/podcasts/:id/clear-queue` does not stop the episode that
    downloads now.** The server holds that episode outside the queue, and
    `currentDownload` names it. See T-81.
39. **A message of the program hides the last row of the view below it.** The row
    of the message stands above the footer, and `Clear` takes it for six seconds
    (T-78). A list that fills the whole view therefore loses its last line while
    a message lives. The line comes back with no key.
40. **A key that writes a value of the server must ask for the lines after that
    write, and not beside it.** A question that goes with the write gives the
    state of the moment before the key. See T-84.
41. **The reader keeps the book of the session while the user reads it.** The key `h`
    and a second `e` give the book with no call of `get_the_ebook`, therefore the
    time of the file does not change inside one run of the program. A measurement of
    the cache needs a second **run**. See T-67.
42. **`POST /api/collections` refuses a collection with no book.** It answers
    `400` with "Invalid collection data. No books". `POST /api/playlists` with no
    item gives `200` and an empty playlist. **The two lists do not behave in the
    same way**, therefore a view that makes a collection must hold a media. Both
    refuse a name of no letter with `400`. See T-88.
43. **The server takes two lists of one name.** It answers `200` and it gives the
    second list its own identity. The user then reads two lines that say the same
    words, and no key tells them apart. **The program refuses that name before the
    request.** See T-88.
44. **A box that makes a `Terminal` of its own writes on the cells of the view,
    and the terminal of the program does not know it.** ratatui writes the cells
    that changed only, and it compares with its own buffer. The letters of the
    view stay on the screen until a key makes the program write those rows again.
    **`Clear` over the whole rows, and then `terminal.clear()` of the loop.** See
    T-89, and T-42 for the key `R`.
45. **The area of the footer holds two rows, and a `Paragraph` with no `Wrap`
    writes one.** Every footer of more than 80 letters lost its end in a terminal
    of 80 columns, and the keys `?` and `Q` stand at the end of every one of
    them. See T-90.
46. **A view must not give a reason that the program does not have.** With no
    answer from the server, three views said "This library has no …". No request
    gave an answer, therefore the program knows nothing of that library.
    **`App::is_offline` says which of the two sentences the view takes.** See
    T-91.
47. **The status of a login that failed holds the reason.** The old code made one
    error for every status. **429 is the status that costs the most time**: the
    rate limit of 40 requests of 600 seconds. See T-92 and the trap 22.
48. **`PATCH` of a list does not examine the name, and `POST` of a list does.**
    A `PATCH` of a collection with a name of no letter gives `200` and a
    collection with no name; the same request of a playlist keeps the old name.
    **A rule of a name belongs to the program.** See T-93.
49. **`DELETE` of a list that went away gives `404`.** A second key of the user
    and a different client both make that condition. See T-93.
50. **A `Paragraph` with no `Wrap` writes one row for each line of its text.**
    The row of the item holds three rows, and 21 paragraphs of the views wrote
    on one of them: a terminal of 80 columns lost the end of every long line.
    **Every text of a view that can be long needs `Wrap`.** See T-94 and T-90.
51. **A test that holds the answer of the code keeps the fault of the code.**
    The test of the title of the search asked for "Search result [1 items]" and
    it passed for two years of releases. **Write the rule of the user in the
    test**, and not the words that the program gives today. See T-95.
52. **A request that stops at its time limit is not evidence that the address is
    down.** The server reads a web site for a feed and every file for a scan.
    Two such requests, one after the other, give the state `Down` now, and one
    answer of the address forgets them. See T-97.
53. **`ApiError::Timeout` reaches the caller only when a second address exists.**
    The old `send` gave `Unreachable` for a pool of one address, therefore the
    user read "No server address answered" for a server that answered slowly.
    See T-97.
54. **The key Esc quits the program from every view.** The key `h` goes back. A
    sweep that presses Esc to leave a view kills the program, and the next
    `wait_for` then says "no server running on /tmp/tmux-1000/default".

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
14. **The sweep of the view of the login uses the rate limit of the login.** Each
    attempt is one `POST /login` of the 40 of 600 seconds, and the tests of the
    sandbox need that limit too. **Run the sweep, and then `podman restart
    abs-test` before the tests.** See T-92.
15. **The offline mode needs no new configuration**: `podman stop abs-test`, and
    the program starts in that mode with the media of the disk. The header says
    "Offline". `podman start abs-test` gives the server back. See T-91.
16. **A sweep of the login needs a `XDG_CONFIG_HOME` with no database.** Two
    files make it (the trap 7), and the program then draws the field of the
    address. A second sweep needs the database of that directory removed again.
17. **A measurement of a disk must write data that no algorithm makes smaller.**
    `dd if=/dev/zero` gave 7.4 GB/s on the ZFS of the maintainer, because
    `compression=on` makes a file of zeros almost nothing. The same measurement
    with a file of `/dev/urandom` gave 1.4 GB/s. See T-98.
18. **The disk of the maintainer is not the slow part of a build.** ZFS gives
    1.4 GB/s of data that no algorithm makes smaller, and 3000 small files take
    1140 ms against 953 ms of the memory. **A `tmpfs` for `target` gives some
    seconds and it takes 11 gigabytes of the memory.** The processor makes the
    machine slow, and `nice -n 19 ionice -c 3` is the answer. See T-98.

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
  answered with nothing, and one of them was a real fault. The sweep of 2026-08-11
  read the **text** of each screen too, and it found two texts that were wrong (T-75).
- **A test may hold every text of the views to one rule.**
  `a_text_of_a_view_holds_no_run_of_spaces` reads a list of constants, and a new text
  of a view joins that list. A rule with no list is a rule that the next text breaks.
  **A rule that no list can hold reads the source**:
  `a_text_of_a_view_names_a_key_with_no_quotation_mark` reads `src/ui/tui.rs` with
  `include_str!` and it finds every text that says "Press '". See T-91.
- **Measure the request before you draw the screen that sends it.** The first
  shape of T-88 was a view that asks for a name and a library, and the server
  refuses a collection with no book. Four minutes of `curl` moved the whole work
  into a view that exists already.
- **A test of a view must hold the two conditions of the server.** The test of the
  view of T-88 makes an `App` with an address that answers nothing, therefore that
  application is in the **offline mode**. The screen of that mode says a different
  sentence (T-91), and the test measures the two conditions one after the other.

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
> of the archived AlbanDAVID/Toutui. Newest release **v0.7.46**; `Cargo.toml` is at
> 0.7.46, so the next release bumps it first — the workflow refuses a tag that
> disagrees with `Cargo.toml`, **and it builds `--locked`, therefore the commit of
> the bump must hold the new `Cargo.lock`**.
>
> Read `docs/HANDOVER.md` first: the state, the open items, the section of the
> harness, and 72 traps that cost real time. Then `docs/TAKEOVER-BACKLOG.md` (the
> evidence of every item; T-88 to T-100 are the newest, and **T-87 and T-97 are the
> two to know**) and `docs/T-24-coverage.md` (**section 6 names what the program
> must not have, and why**).
>
> **The gates, before each commit**, under `nice -n 19 ionice -c 3` with `-j 16`:
> `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, and
> `cargo nextest run` with `ALSA_CONFIG_PATH` pointing at a real null asound file
> (`/dev/null` hangs the real binary). Baseline: **885 tests in 2.2 seconds**, and
> `cargo nextest run --run-ignored all` gives **908 of 908** with the sandbox up.
>
> **The sweep of the views is the tool that finds the faults.** Twenty-two items of
> the two sessions of 2026-08-11 came from sweeps in tmux with
> `docs/harness/drive.sh`, and no test held one of them. **Every sweep that a
> session had not made found a fault**: 80 columns cut the end of every footer
> (T-90) and of every row of an item (T-94), the offline mode made three views say
> something that the program cannot know (T-91), the view of the login said "ERROR:
> Login failed" for every fault (T-92), and the first screen of the sweep of the
> reader said "Search result [1 items]" (T-95). **Press every key of every view**,
> read the screen after each of them, and give the program the shape of the data
> that breaks the rule.
>
> **The work that stays, in the sequence of its value.**
> `docs/HANDOVER.md`, "The road to a program with no fault", holds every item
> with its evidence. The first four:
>
> 1. **The changelog of the fork stops at v0.6.8, and the program is at v0.7.46.**
>    `src/utils/changelog.rs` shows that screen with the key `S`, therefore 38
>    releases of this fork reach no user. Write the entries in the words of a
>    user, and take them from `git log` and from `docs/TAKEOVER-BACKLOG.md`.
> 2. **The 7 rows of the player stay empty while nothing plays (T-99).** They are
>    7 rows of a terminal of 18. A view that takes them moves every line when a
>    playback starts: **ask the maintainer**, and write the answer in the
>    handover.
> 3. **The sequence of the media inside a playlist.** Measure
>    `PATCH /api/playlists/:id` with `items` first: T-93 measured `name` and
>    `description` of that path only.
> 4. **The sweeps that stay:** a library of one item, a book of one chapter, a
>    pool of two addresses, and a server that goes away in the middle of a
>    playback. **Every sweep of a condition that no session had made found a
>    fault.**
>
> **Do not open these again.** The book of xHE-AAC plays (T-68 and T-69). Toutui stays
> GPL, and a person may read bookokrat for an idea and must then write their own code
> and name that project in the commit (T-51). `libsqlite3-sys` and `ring` stay (T-20).
>
> All prose and user-facing strings in ASD-STE100 simplified technical English. No
> crate needing a system library; `cargo tree -i openssl-sys` must find nothing, and
> `cargo tree -i cc` must find `libsqlite3-sys` and `ring` only. No test may need the
> network — the tests of the sandbox carry `#[ignore]` and run one at a time, because
> the login of the server permits 40 requests of 600 seconds. Never write to
> AlbanDAVID/Toutui, and keep his credit everywhere it appears. **Give
> `-R ealtun21/Toutui` to every `gh` command.**
>
> **Show a fault before you fix it, and measure against the sandbox**
> (`docs/TEST-SERVER.md`, podman on `:13399`; `podman start abs-test` gives the server
> back with every book of the sessions before, and `podman restart abs-test` after a
> transcode that died). Make the data exist first, and make it hold the shape that
> breaks the rule. **Drive the real program inside tmux for every view**; a screen of
> your own writing lies to you. Verify with a second program: `podman logs abs-test`,
> a real browser, or `curl`. Tag, push, and keep working; don't wait for CI.
>
> The user tests each release as it lands and does not want to be asked before
> publishing a patch. The server of the user is theirs alone: ask before you use it,
> always with an isolated `XDG_CONFIG_HOME`, and never write its address or its
> account into this repository. Measure against the sandbox instead.
